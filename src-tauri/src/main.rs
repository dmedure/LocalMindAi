// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub specialization: String,
    pub personality: String,
    pub instructions: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub sender: String, // "user" or "agent"
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub doc_type: String,
    pub size: u64,
    pub path: String,
    pub summary: Option<String>,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub ollama: bool,
    pub chromadb: bool,
}

// Application state
#[derive(Debug, Default)]
pub struct AppState {
    pub agents: Mutex<Vec<Agent>>,
    pub messages: Mutex<HashMap<String, Vec<Message>>>, // agent_id -> messages
    pub documents: Mutex<Vec<Document>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            agents: Mutex::new(Vec::new()),
            messages: Mutex::new(HashMap::new()),
            documents: Mutex::new(Vec::new()),
        }
    }
}

// Helper functions
fn get_data_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("LocalMind");
    path
}

fn ensure_data_dir() -> Result<PathBuf, String> {
    let data_dir = get_data_dir();
    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
    Ok(data_dir)
}

async fn save_agents(agents: &Vec<Agent>) -> Result<(), String> {
    let data_dir = ensure_data_dir()?;
    let agents_file = data_dir.join("agents.json");
    
    let json = serde_json::to_string_pretty(agents)
        .map_err(|e| format!("Failed to serialize agents: {}", e))?;
    
    fs::write(agents_file, json)
        .map_err(|e| format!("Failed to save agents: {}", e))?;
    
    Ok(())
}

async fn load_agents() -> Result<Vec<Agent>, String> {
    let data_dir = get_data_dir();
    let agents_file = data_dir.join("agents.json");
    
    if !agents_file.exists() {
        return Ok(Vec::new());
    }
    
    let json = fs::read_to_string(agents_file)
        .map_err(|e| format!("Failed to read agents file: {}", e))?;
    
    let agents: Vec<Agent> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse agents file: {}", e))?;
    
    Ok(agents)
}

async fn save_messages(messages: &HashMap<String, Vec<Message>>) -> Result<(), String> {
    let data_dir = ensure_data_dir()?;
    let messages_file = data_dir.join("messages.json");
    
    let json = serde_json::to_string_pretty(messages)
        .map_err(|e| format!("Failed to serialize messages: {}", e))?;
    
    fs::write(messages_file, json)
        .map_err(|e| format!("Failed to save messages: {}", e))?;
    
    Ok(())
}

async fn load_messages() -> Result<HashMap<String, Vec<Message>>, String> {
    let data_dir = get_data_dir();
    let messages_file = data_dir.join("messages.json");
    
    if !messages_file.exists() {
        return Ok(HashMap::new());
    }
    
    let json = fs::read_to_string(messages_file)
        .map_err(|e| format!("Failed to read messages file: {}", e))?;
    
    let messages: HashMap<String, Vec<Message>> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse messages file: {}", e))?;
    
    Ok(messages)
}

// Tauri commands
#[tauri::command]
async fn get_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    let agents = state.agents.lock().await;
    Ok(agents.clone())
}

#[tauri::command]
async fn create_agent(agent: Agent, state: State<'_, AppState>) -> Result<(), String> {
    let mut agents = state.agents.lock().await;
    agents.push(agent);
    save_agents(&agents).await?;
    Ok(())
}

#[tauri::command]
async fn get_agent_messages(agent_id: String, state: State<'_, AppState>) -> Result<Vec<Message>, String> {
    let messages = state.messages.lock().await;
    Ok(messages.get(&agent_id).cloned().unwrap_or_default())
}

#[tauri::command]
async fn send_message_to_agent(
    agent_id: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Get the agent info
    let agents = state.agents.lock().await;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or("Agent not found")?
        .clone();
    drop(agents);

    // Save user message
    let user_message = Message {
        id: Uuid::new_v4().to_string(),
        content: message.clone(),
        sender: "user".to_string(),
        timestamp: Utc::now(),
        agent_id: agent_id.clone(),
    };

    {
        let mut messages = state.messages.lock().await;
        messages
            .entry(agent_id.clone())
            .or_insert_with(Vec::new)
            .push(user_message);
    }

    // Generate AI response based on agent personality and specialization
    let ai_response = generate_agent_response(&agent, &message).await?;

    // Save AI response
    let ai_message = Message {
        id: Uuid::new_v4().to_string(),
        content: ai_response.clone(),
        sender: "agent".to_string(),
        timestamp: Utc::now(),
        agent_id: agent_id.clone(),
    };

    {
        let mut messages = state.messages.lock().await;
        messages
            .entry(agent_id)
            .or_insert_with(Vec::new)
            .push(ai_message);
        
        // Save to disk
        save_messages(&messages).await?;
    }

    Ok(ai_response)
}

async fn generate_agent_response(agent: &Agent, user_message: &str) -> Result<String, String> {
    // Build the prompt based on agent's personality and specialization
    let system_prompt = build_agent_system_prompt(agent);
    
    // Prepare the request to Ollama
    let client = reqwest::Client::new();
    let ollama_request = serde_json::json!({
        "model": "llama3.1:8b",
        "prompt": format!("{}\n\nUser: {}\nAssistant:", system_prompt, user_message),
        "stream": false,
        "options": {
            "temperature": 0.7,
            "top_p": 0.9,
            "max_tokens": 1000
        }
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&ollama_request)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama API error: {}", response.status()));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    let ai_response = response_json["response"]
        .as_str()
        .unwrap_or("I apologize, but I'm having trouble generating a response right now.")
        .trim();

    Ok(ai_response.to_string())
}

fn build_agent_system_prompt(agent: &Agent) -> String {
    let personality_prompts = match agent.personality.as_str() {
        "professional" => "You are professional, courteous, and business-focused. You provide clear, structured responses and maintain a formal but approachable tone.",
        "friendly" => "You are warm, enthusiastic, and personable. You use a conversational tone and show genuine interest in helping the user.",
        "analytical" => "You are logical, detail-oriented, and methodical. You break down complex problems and provide thorough, well-reasoned responses.",
        "creative" => "You are imaginative, innovative, and artistic. You think outside the box and offer creative solutions and perspectives.",
        "concise" => "You are direct, efficient, and to-the-point. You provide clear, brief responses without unnecessary elaboration.",
        "detailed" => "You are thorough, comprehensive, and explanatory. You provide in-depth responses with examples and context.",
        _ => "You are helpful, knowledgeable, and adaptive to the user's needs.",
    };

    let specialization_prompts = match agent.specialization.as_str() {
        "work" => "You specialize in professional and business matters. You help with project management, workplace communication, productivity, and career development.",
        "coding" => "You specialize in programming and software development. You help with code review, debugging, documentation, and technical problem-solving.",
        "research" => "You specialize in research and academic work. You help with information gathering, data analysis, literature reviews, and scholarly writing.",
        "writing" => "You specialize in writing and content creation. You help with editing, brainstorming, storytelling, and various forms of written communication.",
        "personal" => "You specialize in personal assistance and daily life management. You help with organization, scheduling, personal projects, and lifestyle questions.",
        "creative" => "You specialize in creative and artistic endeavors. You help with brainstorming, design thinking, artistic projects, and creative problem-solving.",
        "technical" => "You specialize in technical support and troubleshooting. You help with system administration, technical documentation, and solving technical problems.",
        _ => "You are a general assistant capable of helping with a wide variety of tasks and questions.",
    };

    let mut prompt = format!(
        "You are {}, a specialized AI assistant. {}\n\n{}\n\n",
        agent.name, personality_prompts, specialization_prompts
    );

    if let Some(instructions) = &agent.instructions {
        if !instructions.trim().is_empty() {
            prompt.push_str(&format!("Additional instructions: {}\n\n", instructions));
        }
    }

    prompt.push_str("Always stay in character and respond according to your personality and specialization. Be helpful, accurate, and engaging.");

    prompt
}

#[tauri::command]
async fn check_service_status() -> Result<ServiceStatus, String> {
    let ollama_status = check_ollama_status().await;
    let chromadb_status = check_chromadb_status().await;
    
    Ok(ServiceStatus {
        ollama: ollama_status,
        chromadb: chromadb_status,
    })
}

async fn check_ollama_status() -> bool {
    let client = reqwest::Client::new();
    match client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

async fn check_chromadb_status() -> bool {
    let client = reqwest::Client::new();
    match client
        .get("http://localhost:8000/api/v1/heartbeat")
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

#[tauri::command]
async fn add_document() -> Result<(), String> {
    // This would open a file dialog and process the selected document
    // For now, we'll return a placeholder
    Err("Document indexing not yet implemented".to_string())
}

#[tauri::command]
async fn export_agent_knowledge(agent_id: String) -> Result<(), String> {
    // This would export the agent's knowledge to a file
    // For now, we'll return a placeholder
    Err("Knowledge export not yet implemented".to_string())
}

#[tauri::command]
async fn import_agent_knowledge(file_path: String) -> Result<(), String> {
    // This would import knowledge from a file
    // For now, we'll return a placeholder
    Err("Knowledge import not yet implemented".to_string())
}

// Initialize the application state
async fn initialize_app_state() -> AppState {
    let state = AppState::new();
    
    // Load existing data
    if let Ok(agents) = load_agents().await {
        *state.agents.lock().await = agents;
    }
    
    if let Ok(messages) = load_messages().await {
        *state.messages.lock().await = messages;
    }
    
    state
}

#[tokio::main]
async fn main() {
    // Initialize application state
    let app_state = initialize_app_state().await;
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_agents,
            create_agent,
            get_agent_messages,
            send_message_to_agent,
            check_service_status,
            add_document,
            export_agent_knowledge,
            import_agent_knowledge
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}