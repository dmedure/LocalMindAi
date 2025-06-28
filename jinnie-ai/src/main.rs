#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder, LogicalSize};
use tokio::runtime::Runtime;

// Your existing imports
use local_ai_agent::initialize_app;

// UI imports
mod ui;
use ui::app::App;

/// Application state that bridges your backend with the Dioxus frontend
#[derive(Clone)]
pub struct AppState {
    /// Your existing app state from local_ai_agent
    pub backend_state: Arc<local_ai_agent::AppState>,
    /// Tokio runtime handle for async operations
    pub runtime_handle: tokio::runtime::Handle,
}

fn main() {
    // Initialize logging first
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("🦀 Starting Jinnie.ai - 100% Rust-Powered AI Assistant");

    // Create Tokio runtime for async operations
    let runtime = Runtime::new()
        .expect("Failed to create Tokio runtime");

    // Initialize your existing backend state
    let backend_state = runtime.block_on(async {
        log::info!("Initializing LocalMind backend...");
        initialize_app().await
            .expect("Failed to initialize LocalMind backend")
    });

    log::info!("✅ Backend initialized successfully");

    // Create shared app state
    let app_state = AppState {
        backend_state: Arc::new(backend_state),
        runtime_handle: runtime.handle().clone(),
    };

    // Launch Dioxus desktop application
    log::info!("🚀 Launching Dioxus UI...");
    
    dioxus_desktop::launch_cfg(
        move || {
            // Provide app state to the UI
            rsx! {
                AppWithState {
                    app_state: app_state.clone()
                }
            }
        },
        Config::default()
            .with_window(
                WindowBuilder::new()
                    .with_title("Jinnie.ai - Rust-Powered AI Assistant")
                    .with_inner_size(LogicalSize::new(1400.0, 900.0))
                    .with_min_inner_size(LogicalSize::new(1000.0, 700.0))
                    .with_resizable(true)
            )
            .with_custom_head(r#"
                <style>
                    body { 
                        margin: 0; 
                        overflow: hidden; 
                        background: #1A1414;
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
                    }
                </style>
            "#.to_string())
    );
}

/// Wrapper component that provides app state to the UI
#[derive(Props, Clone)]
struct AppWithStateProps {
    app_state: AppState,
}

fn AppWithState(props: AppWithStateProps) -> Element {
    // Provide the app state as context for the entire UI tree
    use_context_provider(|| props.app_state.clone());

    rsx! {
        App {}
    }
}

/// Backend integration functions that your UI can call
impl AppState {
    /// Send a message to an agent (replaces the old Tauri command)
    pub async fn send_message_to_agent(
        &self,
        agent_id: String,
        message: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Integrate with your existing send_message_to_agent logic
        // This is where you'd call your backend's message handling
        log::info!("Sending message to agent {}: {}", agent_id, message);
        
        // Placeholder response - replace with your actual backend call
        Ok(format!("Response from agent {}: I received your message '{}'", agent_id, message))
    }

    /// Get agent messages (replaces the old Tauri command)
    pub async fn get_agent_messages(
        &self,
        agent_id: String,
    ) -> Result<Vec<local_ai_agent::types::message::Message>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Integrate with your existing get_agent_messages logic
        log::info!("Getting messages for agent: {}", agent_id);
        
        // Placeholder - replace with your actual backend call
        Ok(vec![])
    }

    /// Get available agents (replaces the old Tauri command)
    pub async fn get_agents(&self) -> Result<Vec<local_ai_agent::types::agent::Agent>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Integrate with your existing get_agents logic
        log::info!("Getting available agents");
        
        // Placeholder - replace with your actual backend call
        Ok(vec![])
    }

    /// Create a new agent (replaces the old Tauri command)
    pub async fn create_agent(
        &self,
        name: String,
        model: String,
        description: String,
        system_prompt: String,
    ) -> Result<local_ai_agent::types::agent::Agent, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Integrate with your existing create_agent logic
        log::info!("Creating new agent: {}", name);
        
        // Placeholder - replace with your actual backend call
        Err("Not implemented yet".into())
    }

    /// Clear chat history (replaces the old Tauri command)
    pub async fn clear_chat(&self, agent_id: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Integrate with your existing clear_chat logic
        log::info!("Clearing chat for agent: {}", agent_id);
        
        // Placeholder - replace with your actual backend call
        Ok(())
    }

    /// Search memories (replaces the old Tauri command)
    pub async fn search_memories(
        &self,
        query: String,
        limit: Option<usize>,
    ) -> Result<Vec<local_ai_agent::types::memory::Memory>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Integrate with your existing search_memories logic
        log::info!("Searching memories with query: {}", query);
        
        // Placeholder - replace with your actual backend call
        Ok(vec![])
    }

    /// Get system information (replaces the old Tauri command)
    pub async fn get_system_info(&self) -> Result<local_ai_agent::types::system::SystemInfo, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Integrate with your existing get_system_info logic
        log::info!("Getting system information");
        
        // Placeholder - replace with your actual backend call
        Err("Not implemented yet".into())
    }

    /// Execute async operations on the runtime
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime_handle.spawn(future)
    }
}

// Utility hook for UI components to access the app state
pub fn use_app_state() -> AppState {
    use_context::<AppState>()
}