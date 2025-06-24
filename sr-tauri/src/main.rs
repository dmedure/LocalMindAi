// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{SystemTray, SystemTrayMenu, SystemTrayMenuItem, Manager, State};
use uuid::Uuid;

mod agent;
mod models;
mod knowledge;
mod tools;
mod knowledge_transfer;

use agent::Agent;
use knowledge::KnowledgeBase;
use knowledge_transfer::{ExportOptions, KnowledgeTransfer};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub content: String,
    pub role: String, // "user" or "assistant"
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub sources: Vec<String>,
}

pub struct AppState {
    pub agent: Arc<Mutex<Agent>>,
    pub knowledge_base: Arc<Mutex<KnowledgeBase>>,
}

#[tauri::command]
async fn send_message(
    content: String,
    state: State<'_, AppState>,
) -> Result<ChatResponse, String> {
    let agent = state.agent.lock().await;
    let kb = state.knowledge_base.lock().await;
    
    // Search for relevant context in knowledge base
    let context = kb.search(&content, 5).await
        .map_err(|e| format!("Knowledge search failed: {}", e))?;
    
    // Generate response using local LLM
    let response = agent.generate_response(&content, &context).await
        .map_err(|e| format!("Response generation failed: {}", e))?;
    
    let message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        content: response.clone(),
        role: "assistant".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    Ok(ChatResponse {
        message,
        sources: context.iter().map(|doc| doc.source.clone()).collect(),
    })
}

#[tauri::command]
async fn index_document(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut kb = state.knowledge_base.lock().await;
    
    kb.index_document(&file_path).await
        .map_err(|e| format!("Document indexing failed: {}", e))?;
    
    Ok(format!("Successfully indexed: {}", file_path))
}

#[tauri::command]
async fn search_documents(
    query: String,
    limit: usize,
    state: State<'_, AppState>,
) -> Result<Vec<knowledge::Document>, String> {
    let kb = state.knowledge_base.lock().await;
    
    kb.search(&query, limit).await
        .map_err(|e| format!("Document search failed: {}", e))
}

#[tauri::command]
async fn get_system_info() -> Result<serde_json::Value, String> {
    use sysinfo::{System, SystemExt, ProcessorExt};
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let info = serde_json::json!({
        "os": sys.name(),
        "version": sys.os_version(),
        "total_memory": sys.total_memory(),
        "available_memory": sys.available_memory(),
        "cpu_count": sys.processors().len(),
        "cpu_brand": sys.processors().first().map(|p| p.brand()),
    });
    
    Ok(info)
}

#[tauri::command]
async fn export_agent_knowledge(
    categories: Vec<String>,
    export_path: String,
    encrypt: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let agent = state.agent.lock().await;
    let kb = state.knowledge_base.lock().await;
    
    let options = ExportOptions {
        categories,
        encrypt,
        anonymize_personal_data: true,
        include_source_files: false,
        compression: knowledge_transfer::CompressionType::Gzip,
    };
    
    kb.export_knowledge(&export_path, options).await
        .map_err(|e| format!("Export failed: {}", e))
}

#[tauri::command]
async fn import_agent_knowledge(
    import_path: String,
    merge_strategy: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut kb = state.knowledge_base.lock().await;
    
    let strategy = match merge_strategy.as_str() {
        "replace" => knowledge_transfer::MergeStrategy::Replace,
        "append" => knowledge_transfer::MergeStrategy::Append,
        "merge" => knowledge_transfer::MergeStrategy::Merge,
        _ => knowledge_transfer::MergeStrategy::Merge,
    };
    
    kb.import_knowledge(&import_path, strategy).await
        .map_err(|e| format!("Import failed: {}", e))
}

#[tauri::command]
async fn export_complete_agent(
    export_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let agent = state.agent.lock().await;
    let kb = state.knowledge_base.lock().await;
    
    agent.export_complete_state(&export_path, &*kb).await
        .map_err(|e| format!("Complete export failed: {}", e))
}

#[tauri::command]
async fn get_agent_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let kb = state.knowledge_base.lock().await;
    
    let stats = serde_json::json!({
        "documents_count": kb.get_document_count(),
        "total_conversations": kb.get_conversation_count(),
        "knowledge_categories": kb.get_categories(),
        "last_updated": kb.get_last_updated(),
        "storage_size": kb.estimate_storage_size(),
    });
    
    Ok(stats)
}

#[tauri::command]
async fn create_specialized_agent(
    domain: String,
    source_export_path: String,
    target_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let transfer = KnowledgeTransfer::new();
    
    transfer.create_specialized_agent(&domain, &source_export_path, &target_path).await
        .map_err(|e| format!("Specialized agent creation failed: {}", e))
}

#[tauri::command]
async fn check_ollama_status() -> Result<bool, String> {
    match reqwest::get("http://localhost:11434/api/tags").await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

fn create_system_tray() -> SystemTray {
    let quit = SystemTrayMenuItem::new("Quit", "quit");
    let show = SystemTrayMenuItem::new("Show", "show");
    let separator = SystemTrayMenuItem::Separator;
    
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(separator)
        .add_item(quit);
    
    SystemTray::new().with_menu(tray_menu)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Initialize knowledge base
    let kb = KnowledgeBase::new().await
        .expect("Failed to initialize knowledge base");
    
    // Initialize agent
    let agent = Agent::new("llama3.1:8b").await
        .expect("Failed to initialize agent");
    
    let app_state = AppState {
        agent: Arc::new(Mutex::new(agent)),
        knowledge_base: Arc::new(Mutex::new(kb)),
    };
    
    tauri::Builder::default()
        .manage(app_state)
        .system_tray(create_system_tray())
        .on_system_tray_event(|app, event| match event {
            tauri::SystemTrayEvent::DoubleClick {
                position: _,
                size: _,
                ..
            } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            send_message,
            index_document,
            search_documents,
            get_system_info,
            check_ollama_status,
            export_agent_knowledge,
            import_agent_knowledge,
            export_complete_agent,
            get_agent_stats,
            create_specialized_agent
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}