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

// Command modules
pub mod commands;

// Re-exports for convenience
pub use types::{Agent, Message, Document};
pub use utils::{LocalMindError, Result};
pub use state::{AppState, initialize_app_state};
pub use config::{AppConfig, ModelConfig, PlatformConfig};

// Platform-specific modules
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

/// Initialize the LocalMind application
pub async fn initialize_app() -> Result<AppState> {
    // Initialize logging
    env_logger::init();
    log::info!("Starting LocalMind AI Agent");

    // Load configuration
    let config = config::initialize_config().await?;
    
    // Initialize application state
    let state = initialize_app_state(config).await?;
    
    // Initialize services
    services::initialize_services(&state).await?;
    
    // Initialize LLM engine
    llm::initialize_llm_engine(&state).await?;
    
    // Initialize vector store
    vector::initialize_vector_store(&state).await?;
    
    // Initialize memory system
    memory::initialize_memory_system(&state).await?;
    
    log::info!("LocalMind initialized successfully");
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