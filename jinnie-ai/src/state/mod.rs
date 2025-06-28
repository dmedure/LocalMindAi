pub mod app_state;

pub use app_state::{AppStateManager};
use crate::types::AppState;
use crate::config::AppConfig;
use crate::utils::error::Result;

/// Initialize application state with the given configuration
pub async fn initialize_app_state(config: AppConfig) -> Result<AppState> {
    log::info!("Initializing application state...");
    
    // Create base state
    let mut state = AppState::new(config.clone());
    
    // Load persisted data
    AppStateManager::initialize_data(&mut state).await?;
    
    // Initialize memory system if feature is enabled
    #[cfg(feature = "basic-ai")]
    {
        use crate::memory::MemoryCoordinator;
        match MemoryCoordinator::new(config.clone(), state.vector_store.clone()).await {
            Ok(memory_system) => {
                state = state.with_memory_system(memory_system);
                log::info!("Memory system initialized");
            }
            Err(e) => {
                log::warn!("Failed to initialize memory system: {}", e);
            }
        }
    }
    
    // Initialize vector store if feature is enabled
    #[cfg(feature = "vector-db")]
    {
        use crate::vector::VectorStore;
        match VectorStore::new(config).await {
            Ok(vector_store) => {
                vector_store.initialize().await?;
                state = state.with_vector_store(vector_store);
                log::info!("Vector store initialized");
            }
            Err(e) => {
                log::warn!("Failed to initialize vector store: {}", e);
            }
        }
    }
    
    log::info!("Application state initialized successfully");
    Ok(state)
}