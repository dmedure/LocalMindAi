// Prevent additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{Manager, State};
use tokio::sync::Mutex;
use uuid::Uuid;

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub specialization: String,
    pub personality: String,
    pub instructions: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub agent_id: String,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub ollama: String,
    pub chromadb: String,
}

// Application state
pub struct AppState {
    pub agents: Mutex<Vec<Agent>>,
    pub messages: Mutex<HashMap<String, Vec<Message>>>, // agent_id -> messages
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("LocalMind");
        
        // Ensure data directory exists
        fs::create_dir_all(&data_dir).ok();
        
        Self {
            agents: Mutex::new(Vec::new()),
            messages: Mutex::new(HashMap::new()),
            data_dir,
        }
    }
    
    // Load agents from disk
    pub async fn load_agents(&self) -> Result<(), Box<dyn std::error::Error>> {
        let agents_file = self.data_dir.join("agents.json");
        if agents_file.exists() {
            let content = fs::read_to_string(agents_file)?;
            let agents: Vec<Agent> = serde_json::from_str(&content)?;
            *self.agents.lock().await = agents;
        }
        Ok(())
    }
    
    // Save agents to disk
    pub async fn save_agents(&self) -> Result<(), Box<dyn std::error::Error>> {
        let agents_file = self.data_dir.join("agents.json");
        let agents = self.agents.lock().await;
        let content = serde_json::to_string_pretty(&*agents)?;
        fs::write(agents_file, content)?;
        Ok(())
    }
    
    // Load messages for a specific agent
    pub async fn load_agent_messages(&self, agent_id: &str) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
        let messages_file = self.data_dir.join(format!("messages_{}.json", agent_id));
        if messages_file.exists() {
            let content = fs::read_to_string(messages_file)?;
            let messages: Vec<Message> = serde_json::from_str(&content)?;
            
            // Update in-memory cache
            self.messages.lock().await.insert(agent_id.to_string(), messages.clone());
            
            Ok(messages)
        } else {
            Ok(Vec::new())
        }
    }
    
    // Save messages for a specific agent
    pub async fn save_agent_messages(&self, agent_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let messages_file = self.data_dir.join(format!("messages_{}.json", agent_id));
        let messages_map = self.messages.lock().await;
        
        if let Some(messages) = messages_map.get(agent_id) {
            let content = serde_json::to_string_pretty(messages)?;
            fs::write(messages_file, content)?;
        }
        
        Ok(())
    }
}

// Tauri commands
#[tauri::command]
async fn get_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    println!("Getting agents...");
    
    // Load agents from disk first
    if let Err(e) = state.load_agents().await {
        eprintln!("Failed to load agents from disk: {}", e);
    }
    
    let agents = state.agents.lock().await.clone();
    println!("Found {} agents", agents.len());
    
    Ok(agents)
}

#[tauri::command]
async fn create_agent(
    name: String,
    specialization: String,
    personality: String,
    instructions: String,
    state: State<'_, AppState>,
) -> Result<Agent, String> {
    println!("Creating agent: {} ({})", name, specialization);
    
    let agent = Agent {
        id: Uuid::new_v4().to_string(),
        name: name.clone(),
        specialization,
        personality,
        instructions,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    // Add to in-memory state
    state.agents.lock().await.push(agent.clone());
    
    // Save to disk
    if let Err(e) = state.save_agents().await {
        eprintln!("Failed to save agents: {}", e);
        return Err("Failed to save agent".to_string());
    }
    
    println!("Successfully created agent: {}", agent.id);
    Ok(agent)
}

// FIXED: Improved agent message loading with better error handling
#[tauri::command]
async fn get_agent_messages(
    agent_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Message>, String> {
    println!("Loading messages for agent: {}", agent_id);
    
    // First check in-memory cache
    {
        let messages_map = state.messages.lock().await;
        if let Some(messages) = messages_map.get(&agent_id) {
            println!("Found {} cached messages for agent {}", messages.len(), agent_id);
            return Ok(messages.clone());
        }
    }
    
    // If not in cache, load from disk
    match state.load_agent_messages(&agent_id).await {
        Ok(messages) => {
            println!("Loaded {} messages from disk for agent {}", messages.len(), agent_id);
            Ok(messages)
        }
        Err(e) => {
            eprintln!("Failed to load messages for agent {}: {}", agent_id, e);
            // Return empty messages instead of error to prevent UI blocking
            Ok(Vec::new())
        }
    }
}

#[tauri::command]
async fn send_message(
    agent_id: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    println!("Sending message to agent {}: {}", agent_id, message);
    
    // Find the agent
    let agents = state.agents.lock().await;
    let agent = agents.iter().find(|a| a.id == agent_id);
    
    let agent = match agent {
        Some(a) => a.clone(),
        None => {
            eprintln!("Agent not found: {}", agent_id);
            return Err("Agent not found".to_string());
        }
    };
    drop(agents);
    
    // Create user message
    let user_message = Message {
        id: Uuid::new_v4().to_string(),
        agent_id: agent_id.clone(),
        role: "user".to_string(),
        content: message.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    // Add user message to state
    {
        let mut messages_map = state.messages.lock().await;
        let agent_messages = messages_map.entry(agent_id.clone()).or_insert_with(Vec::new);
        agent_messages.push(user_message);
    }
    
    // Generate AI response
    let ai_response = match generate_ai_response(&agent, &message).await {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Failed to generate AI response: {}", e);
            "I apologize, but I'm having trouble connecting to the AI service. Please check that Ollama is running and try again.".to_string()
        }
    };
    
    // Create assistant message
    let assistant_message = Message {
        id: Uuid::new_v4().to_string(),
        agent_id: agent_id.clone(),
        role: "assistant".to_string(),
        content: ai_response.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    // Add assistant message to state
    {
        let mut messages_map = state.messages.lock().await;
        let agent_messages = messages_map.entry(agent_id.clone()).or_insert_with(Vec::new);
        agent_messages.push(assistant_message);
    }
    
    // Save messages to disk
    if let Err(e) = state.save_agent_messages(&agent_id).await {
        eprintln!("Failed to save messages: {}", e);
    }
    
    println!("Successfully processed message for agent {}", agent_id);
    Ok(ai_response)
}

async fn generate_ai_response(agent: &Agent, message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Build the prompt with agent personality
    let system_prompt = format!(
        "You are {}, a {} AI assistant specializing in {}. {}",
        agent.name, agent.personality, agent.specialization, agent.instructions
    );
    
    let prompt = format!("{}\n\nUser: {}\nAssistant:", system_prompt, message);
    
    let request_body = serde_json::json!({
        "model": "llama3.1:8b",
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.7,
            "top_p": 0.9,
            "num_predict": 512
        }
    });
    
    println!("Sending request to Ollama...");
    
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Ollama API error: {}", error_text).into());
    }
    
    let response_json: serde_json::Value = response.json().await?;
    
    let ai_response = response_json["response"]
        .as_str()
        .unwrap_or("I apologize, but I couldn't generate a proper response.")
        .trim()
        .to_string();
    
    println!("Received response from Ollama: {} chars", ai_response.len());
    Ok(ai_response)
}

#[tauri::command]
async fn check_system_status() -> Result<SystemStatus, String> {
    println!("Checking system status...");
    
    let ollama_status = check_ollama_status().await;
    let chromadb_status = check_chromadb_status().await;
    
    Ok(SystemStatus {
        ollama: ollama_status,
        chromadb: chromadb_status,
    })
}

async fn check_ollama_status() -> String {
    match reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            println!("Ollama is online");
            "online".to_string()
        }
        Ok(_) => {
            println!("Ollama responded but with error status");
            "offline".to_string()
        }
        Err(e) => {
            println!("Ollama connection failed: {}", e);
            "offline".to_string()
        }
    }
}

async fn check_chromadb_status() -> String {
    match reqwest::Client::new()
        .get("http://localhost:8000/api/v1/heartbeat")
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            println!("ChromaDB is online");
            "online".to_string()
        }
        Ok(_) => {
            println!("ChromaDB responded but with error status");
            "offline".to_string()
        }
        Err(_) => {
            println!("ChromaDB connection failed (this is optional)");
            "offline".to_string()
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|_app| {
            // App setup complete - agents will be loaded on first request
            println!("LocalMind AI Agent starting up...");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_agents,
            create_agent,
            get_agent_messages,
            send_message,
            check_system_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}