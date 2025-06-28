// Core modules
pub mod types;
pub mod utils;
pub mod storage;
pub mod state;
pub mod config;

// Feature modules
pub mod llm;
pub mod memory;
pub mod vector;
pub mod ai;
pub mod services;
pub mod commands;

// Platform modules
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

// Re-exports for convenience
pub use types::{Agent, Message, Document};
pub use state::AppState;
pub use config::{AppConfig, ModelConfig};

// Additional needed modules based on code review
pub mod knowledge;
pub mod knowledge_transfer;
pub mod platform;
pub mod ui;

// Error handling
use anyhow::Result;

/// Initialize the Jinnie AI application
pub async fn initialize_app() -> Result<AppState> {
    // Initialize logging
    log::info!("Starting Jinnie AI Assistant");

    // Load configuration
    let config = config::ConfigManager::initialize().await?;
    
    // Initialize application state
    let state = state::initialize_app_state(config).await?;
    
    // Initialize services
    services::initialize_services(&state).await?;
    
    // Initialize LLM engine
    llm::initialize_llm_engine(&state).await?;
    
    // Initialize vector store if feature is enabled
    #[cfg(feature = "vector-db")]
    vector::initialize_vector_store(&state).await?;
    
    // Initialize memory system
    memory::initialize_memory_system(&state).await?;
    
    log::info!("Jinnie AI initialized successfully");
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_initialization() {
        // Test that the app can initialize without errors
        let result = initialize_app().await;
        assert!(result.is_ok());
    }
}