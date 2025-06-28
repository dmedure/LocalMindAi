use crate::state::AppState;
use crate::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Initialize the LLM engine and preload models if configured
pub async fn initialize_llm_engine(state: &AppState) -> Result<()> {
    log::info!("Initializing LLM engine");
    
    let config = state.config.read().await;
    let model_manager = state.model_manager.clone();
    
    // Preload models based on configuration
    if config.models.tinyllama.preload {
        log::info!("Preloading TinyLlama model");
        model_manager.lock().await
            .preload_model(crate::llm::ModelType::TinyLlama).await?;
    }
    
    if config.models.mistral7b.preload {
        log::info!("Preloading Mistral 7B model");
        model_manager.lock().await
            .preload_model(crate::llm::ModelType::Mistral7B).await?;
    }
    
    log::info!("LLM engine initialized");
    Ok(())
}