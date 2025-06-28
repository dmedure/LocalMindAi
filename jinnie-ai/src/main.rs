#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder, LogicalSize};
use tokio::runtime::Runtime;

// Import from your own crate (jinnie_ai) instead of local_ai_agent
use jinnie_ai::initialize_app;

// UI imports
mod ui;
use ui::app::App;

/// Application state that bridges your backend with the Dioxus frontend
#[derive(Clone)]
pub struct AppState {
    /// Your existing app state from jinnie_ai
    pub backend_state: Arc<jinnie_ai::AppState>,
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

    // Initialize your backend state
    let backend_state = runtime.block_on(async {
        log::info!("Initializing Jinnie AI backend...");
        initialize_app().await
            .expect("Failed to initialize backend")
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
    /// Send a message to an agent
    pub async fn send_message_to_agent(
        &self,
        agent_id: String,
        message: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Use your existing backend functionality
        let agents = self.backend_state.agents.lock().await;
        let agent = agents.get(&agent_id)
            .ok_or("Agent not found")?;
        
        // Generate AI response using your AI module
        let response = jinnie_ai::ai::generate_agent_response(agent, &message).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
        // Store the message exchange
        let mut messages = self.backend_state.messages.lock().await;
        
        // Add user message
        let user_message = jinnie_ai::Message::new(
            message,
            agent_id.clone(),
            jinnie_ai::types::message::MessageRole::User,
        );
        messages.entry(agent_id.clone())
            .or_insert_with(Vec::new)
            .push(user_message);
        
        // Add AI response
        let ai_message = jinnie_ai::Message::new(
            response.clone(),
            agent_id.clone(),
            jinnie_ai::types::message::MessageRole::Assistant,
        );
        messages.entry(agent_id)
            .or_insert_with(Vec::new)
            .push(ai_message);
        
        Ok(response)
    }

    /// Get agent messages
    pub async fn get_agent_messages(
        &self,
        agent_id: String,
    ) -> Result<Vec<jinnie_ai::Message>, Box<dyn std::error::Error + Send + Sync>> {
        let messages = self.backend_state.messages.lock().await;
        Ok(messages.get(&agent_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Get available agents
    pub async fn get_agents(&self) -> Result<Vec<jinnie_ai::Agent>, Box<dyn std::error::Error + Send + Sync>> {
        let agents = self.backend_state.agents.lock().await;
        Ok(agents.values().cloned().collect())
    }

    /// Create a new agent
    pub async fn create_agent(
        &self,
        name: String,
        model: String,
        description: String,
        system_prompt: String,
    ) -> Result<jinnie_ai::Agent, Box<dyn std::error::Error + Send + Sync>> {
        let agent = jinnie_ai::Agent::new(name, model)
            .with_description(description)
            .with_system_prompt(system_prompt);
        
        let mut agents = self.backend_state.agents.lock().await;
        agents.insert(agent.id.clone(), agent.clone());
        
        // Save to storage
        jinnie_ai::storage::AgentStorage::save(&agents).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
        Ok(agent)
    }

    /// Clear chat history
    pub async fn clear_chat(&self, agent_id: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut messages = self.backend_state.messages.lock().await;
        messages.remove(&agent_id);
        
        // Save updated messages
        jinnie_ai::storage::MessageStorage::save(&messages).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
        Ok(())
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