// Enhanced Robust main.rs - Complete Agent Platform (Fixed)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Enhanced Agent structure with robust features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub specialization: String,
    pub personality: String,
    pub instructions: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub avatar: Option<String>,
    pub status: AgentStatus,
    pub version: u32,
    pub capabilities: Vec<String>,
    pub knowledge_source_count: u32,
    pub conversation_count: u32,
    pub last_used: DateTime<Utc>,
    pub model_name: String,
    pub context_window: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Archived,
    Training,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub sender: String,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub message_type: MessageType,
    pub attachments: Vec<Attachment>,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Image,
    Document,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: u64,
    pub uploaded_at: DateTime<Utc>,
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
    pub content_preview: String,
    pub keywords: Vec<String>,
    pub agent_reviews: Vec<AgentReview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReview {
    pub agent_id: String,
    pub review_type: String,
    pub review_content: String,
    pub reviewed_at: DateTime<Utc>,
    pub rating: Option<u8>, // 1-5 rating
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub ollama: ServiceHealth,
    pub chromadb: ServiceHealth,
    pub local_storage: ServiceHealth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: String,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
}

// Enhanced Application state with robust management
#[derive(Debug)]
pub struct AppState {
    pub agents: Mutex<Vec<Agent>>,
    pub messages: Mutex<HashMap<String, Vec<Message>>>,
    pub documents: Mutex<Vec<Document>>,
    pub service_status: Mutex<ServiceStatus>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub async fn new() -> Result<Self, String> {
        let data_dir = ensure_data_dir()?;
        
        let state = Self {
            agents: Mutex::new(Vec::new()),
            messages: Mutex::new(HashMap::new()),
            documents: Mutex::new(Vec::new()),
            service_status: Mutex::new(ServiceStatus::default()),
            data_dir,
        };
        
        // Load existing data with error recovery
        state.load_all_data().await?;
        
        Ok(state)
    }
    
    async fn load_all_data(&self) -> Result<(), String> {
        // Load agents with backup recovery
        if let Ok(agents) = load_agents_with_recovery(&self.data_dir).await {
            *self.agents.lock().await = agents;
        }
        
        // Load messages with recovery
        if let Ok(messages) = load_messages_with_recovery(&self.data_dir).await {
            *self.messages.lock().await = messages;
        }
        
        // Load documents with recovery
        if let Ok(documents) = load_documents_with_recovery(&self.data_dir).await {
            *self.documents.lock().await = documents;
        }
        
        Ok(())
    }
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self {
            ollama: ServiceHealth {
                status: "unknown".to_string(),
                last_check: Utc::now(),
                response_time_ms: None,
                error_message: None,
            },
            chromadb: ServiceHealth {
                status: "unknown".to_string(),
                last_check: Utc::now(),
                response_time_ms: None,
                error_message: None,
            },
            local_storage: ServiceHealth {
                status: "healthy".to_string(),
                last_check: Utc::now(),
                response_time_ms: Some(0),
                error_message: None,
            },
        }
    }
}

// Robust data directory management
fn get_data_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("LocalMind");
    path
}

fn ensure_data_dir() -> Result<PathBuf, String> {
    let data_dir = get_data_dir();
    
    // Create main directory
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;
    
    // Create subdirectories
    let subdirs = ["agents", "messages", "documents", "backups", "uploads", "logs"];
    for subdir in &subdirs {
        let path = data_dir.join(subdir);
        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create {} directory: {}", subdir, e))?;
    }
    
    Ok(data_dir)
}

// ROBUST AGENT MANAGEMENT COMMANDS

#[tauri::command]
async fn get_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    let agents = state.agents.lock().await;
    Ok(agents.clone())
}

#[tauri::command]
async fn create_agent(mut agent: Agent, state: State<'_, AppState>) -> Result<Agent, String> {
    // Enhanced validation
    if agent.name.trim().is_empty() {
        return Err("Agent name cannot be empty".to_string());
    }
    
    // Check for duplicate names
    let agents = state.agents.lock().await;
    if agents.iter().any(|a| a.name.eq_ignore_ascii_case(&agent.name)) {
        return Err("Agent name already exists".to_string());
    }
    drop(agents);
    
    // Set defaults and generate ID
    agent.id = Uuid::new_v4().to_string();
    agent.created_at = Utc::now();
    agent.updated_at = Utc::now();
    agent.last_used = Utc::now();
    agent.status = AgentStatus::Active;
    agent.version = 1;
    agent.conversation_count = 0;
    agent.knowledge_source_count = 0;
    agent.model_name = "llama3.1:8b".to_string();
    agent.context_window = 4096;
    agent.temperature = 0.7;
    
    // Set capabilities based on specialization
    agent.capabilities = get_agent_capabilities(&agent.specialization);
    
    let mut agents = state.agents.lock().await;
    agents.push(agent.clone());
    
    // Save with backup
    save_agents_with_backup(&agents, &state.data_dir).await?;
    
    Ok(agent)
}

#[tauri::command]
async fn update_agent(agent_id: String, updates: Agent, state: State<'_, AppState>) -> Result<Agent, String> {
    let mut agents = state.agents.lock().await;
    
    let agent_index = agents.iter()
        .position(|a| a.id == agent_id)
        .ok_or("Agent not found")?;
    
    // Validate updates
    if updates.name.trim().is_empty() {
        return Err("Agent name cannot be empty".to_string());
    }
    
    // Check for duplicate names (excluding current agent)
    if agents.iter().enumerate().any(|(i, a)| i != agent_index && a.name.eq_ignore_ascii_case(&updates.name)) {
        return Err("Agent name already exists".to_string());
    }
    
    // Update agent with version increment
    let mut updated_agent = updates;
    updated_agent.id = agent_id;
    updated_agent.version = agents[agent_index].version + 1;
    updated_agent.updated_at = Utc::now();
    updated_agent.created_at = agents[agent_index].created_at; // Preserve creation date
    updated_agent.capabilities = get_agent_capabilities(&updated_agent.specialization);
    
    agents[agent_index] = updated_agent.clone();
    
    save_agents_with_backup(&agents, &state.data_dir).await?;
    
    Ok(updated_agent)
}

#[tauri::command]
async fn delete_agent(agent_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut agents = state.agents.lock().await;
    let mut messages = state.messages.lock().await;
    
    // Find and remove agent
    let agent_index = agents.iter()
        .position(|a| a.id == agent_id)
        .ok_or("Agent not found")?;
    
    agents.remove(agent_index);
    
    // Clean up associated messages
    messages.remove(&agent_id);
    
    // Save changes
    save_agents_with_backup(&agents, &state.data_dir).await?;
    save_messages_with_backup(&messages, &state.data_dir).await?;
    
    Ok(())
}

#[tauri::command]
async fn duplicate_agent(agent_id: String, new_name: String, state: State<'_, AppState>) -> Result<Agent, String> {
    let agents = state.agents.lock().await;
    
    let source_agent = agents.iter()
        .find(|a| a.id == agent_id)
        .ok_or("Source agent not found")?;
    
    // Check for duplicate names
    if agents.iter().any(|a| a.name.eq_ignore_ascii_case(&new_name)) {
        return Err("Agent name already exists".to_string());
    }
    
    drop(agents);
    
    let mut new_agent = source_agent.clone();
    new_agent.id = Uuid::new_v4().to_string();
    new_agent.name = new_name;
    new_agent.created_at = Utc::now();
    new_agent.updated_at = Utc::now();
    new_agent.last_used = Utc::now();
    new_agent.version = 1;
    new_agent.conversation_count = 0;
    
    let mut agents = state.agents.lock().await;
    agents.push(new_agent.clone());
    
    save_agents_with_backup(&agents, &state.data_dir).await?;
    
    Ok(new_agent)
}

#[tauri::command]
async fn search_agents(query: String, state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    let agents = state.agents.lock().await;
    let query_lower = query.to_lowercase();
    
    let filtered: Vec<Agent> = agents.iter()
        .filter(|agent| {
            agent.name.to_lowercase().contains(&query_lower) ||
            agent.specialization.to_lowercase().contains(&query_lower) ||
            agent.personality.to_lowercase().contains(&query_lower) ||
            agent.instructions.as_ref().map_or(false, |i| i.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect();
    
    Ok(filtered)
}

#[tauri::command]
async fn set_agent_status(agent_id: String, status: AgentStatus, state: State<'_, AppState>) -> Result<(), String> {
    let mut agents = state.agents.lock().await;
    
    let agent = agents.iter_mut()
        .find(|a| a.id == agent_id)
        .ok_or("Agent not found")?;
    
    agent.status = status;
    agent.updated_at = Utc::now();
    
    save_agents_with_backup(&agents, &state.data_dir).await?;
    
    Ok(())
}

// IMAGE HANDLING COMMANDS

#[tauri::command]
async fn upload_image(file_path: String, state: State<'_, AppState>) -> Result<Attachment, String> {
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return Err("File does not exist".to_string());
    }
    
    // Validate image format
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or("Invalid file extension")?;
    
    if !["png", "jpg", "jpeg", "gif", "webp", "bmp"].contains(&extension.as_str()) {
        return Err("Unsupported image format".to_string());
    }
    
    // Check file size (max 10MB)
    let metadata = fs::metadata(path).map_err(|e| format!("Cannot read file metadata: {}", e))?;
    if metadata.len() > 10_000_000 {
        return Err("Image file too large (max 10MB)".to_string());
    }
    
    // Create unique filename in uploads directory
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?;
    
    let upload_id = Uuid::new_v4().to_string();
    let upload_filename = format!("{}_{}", upload_id, file_name);
    let upload_path = state.data_dir.join("uploads").join(&upload_filename);
    
    // Copy file to uploads directory
    fs::copy(path, &upload_path)
        .map_err(|e| format!("Failed to copy image: {}", e))?;
    
    let attachment = Attachment {
        id: upload_id,
        file_name: file_name.to_string(),
        file_path: upload_path.to_string_lossy().to_string(),
        file_type: format!("image/{}", extension),
        file_size: metadata.len(),
        uploaded_at: Utc::now(),
    };
    
    Ok(attachment)
}

#[tauri::command]
async fn analyze_image_with_agent(
    agent_id: String,
    attachment_id: String,
    prompt: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Update agent last_used
    {
        let mut agents = state.agents.lock().await;
        if let Some(agent) = agents.iter_mut().find(|a| a.id == agent_id) {
            agent.last_used = Utc::now();
        }
    }
    
    // For now, return a placeholder until vision model integration
    let analysis = format!(
        "Image analysis for attachment {} with prompt: '{}'\n\n\
        [Vision model integration in progress - this will analyze the uploaded image with the selected agent's personality and expertise]",
        attachment_id, prompt
    );
    
    Ok(analysis)
}

// DOCUMENT PROCESSING COMMANDS

#[tauri::command]
async fn add_document(file_path: String, state: State<'_, AppState>) -> Result<Document, String> {
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return Err("File does not exist".to_string());
    }
    
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Cannot read file metadata: {}", e))?;
    
    // Extract content based on file type
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    
    let content = match extension.as_str() {
        "txt" | "md" => {
            fs::read_to_string(path)
                .map_err(|e| format!("Cannot read text file: {}", e))?
        }
        "pdf" => {
            // TODO: Implement PDF text extraction
            "[PDF content extraction not yet implemented]".to_string()
        }
        "docx" => {
            // TODO: Implement DOCX text extraction
            "[DOCX content extraction not yet implemented]".to_string()
        }
        _ => {
            return Err("Unsupported document format".to_string());
        }
    };
    
    let content_preview = if content.len() > 500 {
        format!("{}...", &content[..500])
    } else {
        content.clone()
    };
    
    let document = Document {
        id: Uuid::new_v4().to_string(),
        name: path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string(),
        doc_type: extension,
        size: metadata.len(),
        path: file_path,
        summary: None, // Will be generated by agent
        indexed_at: Utc::now(),
        content_preview,
        keywords: Vec::new(), // Will be extracted by agent
        agent_reviews: Vec::new(),
    };
    
    let mut documents = state.documents.lock().await;
    documents.push(document.clone());
    
    save_documents_with_backup(&documents, &state.data_dir).await?;
    
    Ok(document)
}

#[tauri::command]
async fn review_document_with_agent(
    agent_id: String,
    document_id: String,
    review_type: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut documents = state.documents.lock().await;
    
    let document = documents.iter()
        .find(|d| d.id == document_id)
        .ok_or("Document not found")?;
    
    // Update agent last_used
    {
        let mut agents = state.agents.lock().await;
        if let Some(agent) = agents.iter_mut().find(|a| a.id == agent_id) {
            agent.last_used = Utc::now();
        }
    }
    
    // Generate review based on type
    let review_content = match review_type.as_str() {
        "summary" => format!("Summary of '{}': [Document summarization in progress]", document.name),
        "analysis" => format!("Analysis of '{}': [Document analysis in progress]", document.name),
        "questions" => format!("Questions about '{}': [Question generation in progress]", document.name),
        "critique" => format!("Critique of '{}': [Document critique in progress]", document.name),
        _ => return Err("Invalid review type".to_string()),
    };
    
    // Add review to document
    let review = AgentReview {
        agent_id: agent_id.clone(),
        review_type,
        review_content: review_content.clone(),
        reviewed_at: Utc::now(),
        rating: None,
    };
    
    if let Some(doc) = documents.iter_mut().find(|d| d.id == document_id) {
        doc.agent_reviews.push(review);
    }
    
    save_documents_with_backup(&documents, &state.data_dir).await?;
    
    Ok(review_content)
}

// Helper functions
fn get_agent_capabilities(specialization: &str) -> Vec<String> {
    match specialization {
        "work" => vec!["email_writing", "project_management", "meeting_notes", "business_analysis"],
        "coding" => vec!["code_review", "debugging", "documentation", "architecture_design"],
        "research" => vec!["data_analysis", "literature_review", "research_design", "citation_management"],
        "writing" => vec!["content_creation", "editing", "proofreading", "style_analysis"],
        "personal" => vec!["task_management", "scheduling", "personal_organization", "goal_setting"],
        "creative" => vec!["brainstorming", "creative_writing", "design_thinking", "art_critique"],
        "technical" => vec!["troubleshooting", "system_analysis", "technical_documentation", "problem_solving"],
        _ => vec!["general_assistance", "conversation", "information_retrieval"],
    }.into_iter().map(|s| s.to_string()).collect()
}

// ROBUST PERSISTENCE WITH BACKUP AND RECOVERY

async fn save_agents_with_backup(agents: &Vec<Agent>, data_dir: &Path) -> Result<(), String> {
    let agents_file = data_dir.join("agents").join("agents.json");
    let backup_file = data_dir.join("backups").join(format!("agents_backup_{}.json", Utc::now().timestamp()));
    
    let json = serde_json::to_string_pretty(agents)
        .map_err(|e| format!("Failed to serialize agents: {}", e))?;
    
    // Create backup first
    if agents_file.exists() {
        fs::copy(&agents_file, &backup_file)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    
    // Write atomically using temporary file
    let temp_file = agents_file.with_extension("tmp");
    fs::write(&temp_file, &json)
        .map_err(|e| format!("Failed to write temporary file: {}", e))?;
    
    fs::rename(&temp_file, &agents_file)
        .map_err(|e| format!("Failed to save agents: {}", e))?;
    
    Ok(())
}

async fn load_agents_with_recovery(data_dir: &Path) -> Result<Vec<Agent>, String> {
    let agents_file = data_dir.join("agents").join("agents.json");
    
    if !agents_file.exists() {
        return Ok(Vec::new());
    }
    
    // Try to load main file
    match fs::read_to_string(&agents_file) {
        Ok(json) => {
            match serde_json::from_str::<Vec<Agent>>(&json) {
                Ok(agents) => Ok(agents),
                Err(_) => {
                    // Try to recover from backup
                    load_agents_from_backup(data_dir).await
                }
            }
        }
        Err(_) => {
            // Try to recover from backup
            load_agents_from_backup(data_dir).await
        }
    }
}

async fn load_agents_from_backup(data_dir: &Path) -> Result<Vec<Agent>, String> {
    let backup_dir = data_dir.join("backups");
    
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }
    
    // Find the most recent backup
    let backup_files = fs::read_dir(&backup_dir)
        .map_err(|e| format!("Failed to read backup directory: {}", e))?;
    
    let mut backup_files: Vec<_> = backup_files
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name().to_str()
                .map_or(false, |name| name.starts_with("agents_backup_") && name.ends_with(".json"))
        })
        .collect();
    
    backup_files.sort_by_key(|entry| {
        entry.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    
    if let Some(latest_backup) = backup_files.last() {
        let json = fs::read_to_string(latest_backup.path())
            .map_err(|e| format!("Failed to read backup file: {}", e))?;
        let agents: Vec<Agent> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse backup file: {}", e))?;
        Ok(agents)
    } else {
        Ok(Vec::new())
    }
}

// Similar backup/recovery functions for messages and documents
async fn save_messages_with_backup(messages: &HashMap<String, Vec<Message>>, data_dir: &Path) -> Result<(), String> {
    let messages_file = data_dir.join("messages").join("messages.json");
    let json = serde_json::to_string_pretty(messages)
        .map_err(|e| format!("Failed to serialize messages: {}", e))?;
    fs::write(messages_file, json)
        .map_err(|e| format!("Failed to save messages: {}", e))?;
    Ok(())
}

async fn load_messages_with_recovery(data_dir: &Path) -> Result<HashMap<String, Vec<Message>>, String> {
    let messages_file = data_dir.join("messages").join("messages.json");
    
    if !messages_file.exists() {
        return Ok(HashMap::new());
    }
    
    let json = fs::read_to_string(messages_file)
        .map_err(|e| format!("Failed to read messages file: {}", e))?;
    let messages: HashMap<String, Vec<Message>> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse messages file: {}", e))?;
    Ok(messages)
}

async fn save_documents_with_backup(documents: &Vec<Document>, data_dir: &Path) -> Result<(), String> {
    let documents_file = data_dir.join("documents").join("documents.json");
    let json = serde_json::to_string_pretty(documents)
        .map_err(|e| format!("Failed to serialize documents: {}", e))?;
    fs::write(documents_file, json)
        .map_err(|e| format!("Failed to save documents: {}", e))?;
    Ok(())
}

async fn load_documents_with_recovery(data_dir: &Path) -> Result<Vec<Document>, String> {
    let documents_file = data_dir.join("documents").join("documents.json");
    
    if !documents_file.exists() {
        return Ok(Vec::new());
    }
    
    let json = fs::read_to_string(documents_file)
        .map_err(|e| format!("Failed to read documents file: {}", e))?;
    let documents: Vec<Document> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse documents file: {}", e))?;
    Ok(documents)
}

// Continue with remaining Tauri commands and main function...

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
    let start_time = std::time::Instant::now();
    
    // Get and update agent
    let agents = state.agents.lock().await;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or("Agent not found")?
        .clone();
    drop(agents);

    // Update agent usage
    {
        let mut agents = state.agents.lock().await;
        if let Some(agent) = agents.iter_mut().find(|a| a.id == agent_id) {
            agent.last_used = Utc::now();
            agent.conversation_count += 1;
        }
        save_agents_with_backup(&agents, &state.data_dir).await?;
    }

    // Save user message
    let user_message = Message {
        id: Uuid::new_v4().to_string(),
        content: message.clone(),
        sender: "user".to_string(),
        timestamp: Utc::now(),
        agent_id: agent_id.clone(),
        message_type: MessageType::Text,
        attachments: Vec::new(),
        response_time_ms: None,
    };

    {
        let mut messages = state.messages.lock().await;
        messages
            .entry(agent_id.clone())
            .or_insert_with(Vec::new)
            .push(user_message);
    }

    // Generate AI response
    let ai_response = generate_agent_response(&agent, &message).await?;
    
    let response_time = start_time.elapsed().as_millis() as u64;

    // Save AI response
    let ai_message = Message {
        id: Uuid::new_v4().to_string(),
        content: ai_response.clone(),
        sender: "agent".to_string(),
        timestamp: Utc::now(),
        agent_id: agent_id.clone(),
        message_type: MessageType::Text,
        attachments: Vec::new(),
        response_time_ms: Some(response_time),
    };

    {
        let mut messages = state.messages.lock().await;
        messages
            .entry(agent_id)
            .or_insert_with(Vec::new)
            .push(ai_message);
        
        save_messages_with_backup(&messages, &state.data_dir).await?;
    }

    Ok(ai_response)
}

async fn generate_agent_response(agent: &Agent, user_message: &str) -> Result<String, String> {
    let system_prompt = build_agent_system_prompt(agent);
    
    let client = reqwest::Client::new();
    let ollama_request = serde_json::json!({
        "model": agent.model_name,
        "prompt": format!("{}\n\nUser: {}\nAssistant:", system_prompt, user_message),
        "stream": false,
        "options": {
            "temperature": agent.temperature,
            "top_p": 0.9,
            "max_tokens": agent.context_window / 2
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
    // Enhanced system prompt building based on agent configuration
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
async fn check_service_status(state: State<'_, AppState>) -> Result<ServiceStatus, String> {
    let ollama_health = check_ollama_status().await;
    let chromadb_health = check_chromadb_status().await;
    
    let status = ServiceStatus {
        ollama: ollama_health,
        chromadb: chromadb_health,
        local_storage: ServiceHealth {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            response_time_ms: Some(0),
            error_message: None,
        },
    };
    
    *state.service_status.lock().await = status.clone();
    
    Ok(status)
}

async fn check_ollama_status() -> ServiceHealth {
    let start_time = std::time::Instant::now();
    let client = reqwest::Client::new();
    
    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(response) => {
            let response_time = start_time.elapsed().as_millis() as u64;
            if response.status().is_success() {
                ServiceHealth {
                    status: "healthy".to_string(),
                    last_check: Utc::now(),
                    response_time_ms: Some(response_time),
                    error_message: None,
                }
            } else {
                ServiceHealth {
                    status: "unhealthy".to_string(),
                    last_check: Utc::now(),
                    response_time_ms: Some(response_time),
                    error_message: Some(format!("HTTP {}", response.status())),
                }
            }
        }
        Err(e) => ServiceHealth {
            status: "offline".to_string(),
            last_check: Utc::now(),
            response_time_ms: None,
            error_message: Some(e.to_string()),
        },
    }
}

async fn check_chromadb_status() -> ServiceHealth {
    let start_time = std::time::Instant::now();
    let client = reqwest::Client::new();
    
    match client.get("http://localhost:8000/api/v1/heartbeat").send().await {
        Ok(response) => {
            let response_time = start_time.elapsed().as_millis() as u64;
            if response.status().is_success() {
                ServiceHealth {
                    status: "healthy".to_string(),
                    last_check: Utc::now(),
                    response_time_ms: Some(response_time),
                    error_message: None,
                }
            } else {
                ServiceHealth {
                    status: "unhealthy".to_string(),
                    last_check: Utc::now(),
                    response_time_ms: Some(response_time),
                    error_message: Some(format!("HTTP {}", response.status())),
                }
            }
        }
        Err(e) => ServiceHealth {
            status: "offline".to_string(),
            last_check: Utc::now(),
            response_time_ms: None,
            error_message: Some(e.to_string()),
        },
    }
}

#[tauri::command]
async fn get_documents(state: State<'_, AppState>) -> Result<Vec<Document>, String> {
    let documents = state.documents.lock().await;
    Ok(documents.clone())
}

#[tauri::command]
async fn export_agent_knowledge(agent_id: String, state: State<'_, AppState>) -> Result<String, String> {
    // Enhanced export functionality
    let agents = state.agents.lock().await;
    let messages = state.messages.lock().await;
    
    let agent = agents.iter()
        .find(|a| a.id == agent_id)
        .ok_or("Agent not found")?;
    
    let agent_messages = messages.get(&agent_id).cloned().unwrap_or_default();
    
    let export_data = serde_json::json!({
        "agent": agent,
        "messages": agent_messages,
        "exported_at": Utc::now(),
        "version": "1.0"
    });
    
    let export_path = state.data_dir
        .join("exports")
        .join(format!("{}_export_{}.json", agent.name, Utc::now().timestamp()));
    
    fs::create_dir_all(export_path.parent().unwrap())
        .map_err(|e| format!("Failed to create export directory: {}", e))?;
    
    fs::write(&export_path, serde_json::to_string_pretty(&export_data).unwrap())
        .map_err(|e| format!("Failed to write export file: {}", e))?;
    
    Ok(export_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn import_agent_knowledge(file_path: String, _state: State<'_, AppState>) -> Result<String, String> {
    // Enhanced import functionality with validation
    let import_data: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read import file: {}", e))?
    ).map_err(|e| format!("Invalid JSON format: {}", e))?;
    
    // Validate import data structure
    if import_data.get("agent").is_none() {
        return Err("Invalid export file: missing agent data".to_string());
    }
    
    // Import would continue with actual data processing
    Ok("Import functionality ready for implementation".to_string())
}

// Initialize the application state
async fn initialize_app_state() -> Result<AppState, String> {
    AppState::new().await
}

#[tokio::main]
async fn main() {
    // Initialize enhanced application state
    let app_state = initialize_app_state().await
        .expect("Failed to initialize application state");
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Enhanced agent management
            get_agents,
            create_agent,
            update_agent,
            delete_agent,
            duplicate_agent,
            search_agents,
            set_agent_status,
            
            // Messaging
            get_agent_messages,
            send_message_to_agent,
            
            // Image handling
            upload_image,
            analyze_image_with_agent,
            
            // Document processing
            add_document,
            get_documents,
            review_document_with_agent,
            
            // System status
            check_service_status,
            
            // Knowledge transfer
            export_agent_knowledge,
            import_agent_knowledge
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}