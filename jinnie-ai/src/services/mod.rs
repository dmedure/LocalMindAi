pub mod ollama;
pub mod chroma;
pub mod service_manager;

pub use ollama::{OllamaService, OllamaConfig};
pub use chroma::{ChromaService, ChromaConfig};
pub use service_manager::{ServiceManager, ServiceStatus};

use crate::state::AppState;
use anyhow::Result;

/// Initialize all external services
pub async fn initialize_services(state: &AppState) -> Result<()> {
    log::info!("Initializing external services...");
    
    // Initialize Ollama service
    let ollama_service = OllamaService::new(OllamaConfig::default());
    if ollama_service.is_available().await {
        log::info!("Ollama service is available");
    } else {
        log::warn!("Ollama service is not available - AI features may be limited");
    }
    
    // Initialize ChromaDB service if vector features are enabled
    #[cfg(feature = "vector-db")]
    {
        let chroma_service = ChromaService::new(ChromaConfig::default());
        if chroma_service.is_available().await {
            log::info!("ChromaDB service is available");
        } else {
            log::warn!("ChromaDB service is not available - vector search features disabled");
        }
    }
    
    Ok(())
}