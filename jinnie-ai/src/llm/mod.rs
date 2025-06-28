pub mod engine;
pub mod session_manager;
pub mod model_manager;
pub mod model_selectors;
pub mod task_classifiers;
pub mod model_downloader;

pub use engine::{LLMEngine, InferenceRequest, InferenceResponse};
pub use session_manager::SessionManager;
pub use model_manager::ModelManager;
pub use model_selectors::ModelSelector;
pub use task_classifiers::TaskClassifier;
pub use model_downloader::{ModelDownloader, DownloadProgress};

use crate::state::AppState;
use anyhow::Result;

/// Initialize the LLM engine
pub async fn initialize_llm_engine(state: &AppState) -> Result<()> {
    log::info!("Initializing LLM engine...");
    
    // Check if Ollama is available
    let ollama_service = crate::services::OllamaService::new(
        crate::services::OllamaConfig::default()
    );
    
    let ollama_available = ollama_service.is_available().await;
    
    // Update service status
    state.update_service_status(Some(ollama_available), None, None).await;
    
    if ollama_available {
        log::info!("Ollama service detected and available");
        
        // Check available models
        match ollama_service.list_models().await {
            Ok(models) => {
                log::info!("Found {} models in Ollama", models.len());
                for model in &models {
                    log::debug!("  - {}: {} ({})", model.name, model.model, model.size);
                }
            }
            Err(e) => {
                log::warn!("Failed to list Ollama models: {}", e);
            }
        }
    } else {
        log::warn!("Ollama service not available - AI features will be limited");
        log::info!("To enable AI features, please install and run Ollama from https://ollama.ai");
    }
    
    Ok(())
}