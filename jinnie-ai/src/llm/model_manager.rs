use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::{ModelType, ModelConfig};
use crate::utils::error::{LocalMindError, Result};

/// Manages model lifecycle (loading, unloading, monitoring)
pub struct ModelManager {
    loaded_models: Arc<RwLock<HashMap<String, LoadedModel>>>,
    model_configs: HashMap<String, ModelConfig>,
    max_models_loaded: usize,
    total_memory_limit_mb: u64,
    ollama_client: crate::services::ollama::OllamaClient,
}

/// Information about a loaded model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedModel {
    pub model_type: ModelType,
    pub config: ModelConfig,
    pub load_time: chrono::DateTime<chrono::Utc>,
    pub memory_usage_mb: Option<u64>,
    pub status: ModelStatus,
    pub performance_metrics: ModelPerformanceMetrics,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
}

/// Model loading status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModelStatus {
    Loading,
    Ready,
    Error(String),
    Unloading,
}

/// Performance metrics for a loaded model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub average_inference_time_ms: f64,
    pub tokens_per_second: f64,
    pub total_inferences: u64,
    pub error_count: u64,
    pub last_inference_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Model loading strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadingStrategy {
    Eager,      // Load immediately when needed
    Lazy,       // Load on first use
    Preload,    // Load all configured models at startup
    OnDemand,   // Load based on prediction
}

/// Memory management strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryStrategy {
    LRU,        // Least Recently Used eviction
    LFU,        // Least Frequently Used eviction
    Priority,   // Based on model priority/importance
    Manual,     // Manual control only
}

impl ModelManager {
    /// Create a new model manager
    pub async fn new() -> Result<Self> {
        let ollama_client = crate::services::ollama::OllamaClient::new();
        
        // Check if Ollama is available
        if !ollama_client.is_available().await {
            return Err(LocalMindError::ExternalService(
                "Ollama service is not available. Please ensure Ollama is running.".to_string()
            ));
        }

        Ok(Self {
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
            model_configs: Self::create_default_model_configs(),
            max_models_loaded: 2, // TinyLlama + Mistral typically
            total_memory_limit_mb: 8192, // 8GB default limit
            ollama_client,
        })
    }

    /// Create default model configurations
    fn create_default_model_configs() -> HashMap<String, ModelConfig> {
        let mut configs = HashMap::new();
        
        // TinyLlama configuration
        let tinyllama_config = ModelConfig::tinyllama("models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf".to_string());
        configs.insert("tinyllama".to_string(), tinyllama_config);

        // Mistral 7B configuration
        let mistral_config = ModelConfig::mistral7b("models/mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string());
        configs.insert("mistral7b".to_string(), mistral_config);

        configs
    }

    /// Load a model into memory
    pub async fn load_model(&mut self, model_type: &ModelType) -> Result<()> {
        let model_id = model_type.identifier();
        
        // Check if already loaded
        {
            let loaded_models = self.loaded_models.read().await;
            if let Some(model) = loaded_models.get(&model_id) {
                if model.status == ModelStatus::Ready {
                    return Ok(()); // Already loaded and ready
                }
            }
        }

        // Get model configuration
        let model_config = self.model_configs.get(&model_id)
            .ok_or_else(|| LocalMindError::Configuration(format!("No configuration found for model: {}", model_id)))?
            .clone();

        // Check if model file exists
        if !model_config.file_exists() {
            return Err(LocalMindError::Configuration(
                format!("Model file not found: {}", model_config.file_path)
            ));
        }

        // Check memory limits and free space if needed
        self.ensure_memory_available(&model_config).await?;

        // Mark as loading
        {
            let mut loaded_models = self.loaded_models.write().await;
            loaded_models.insert(model_id.clone(), LoadedModel {
                model_type: model_type.clone(),
                config: model_config.clone(),
                load_time: chrono::Utc::now(),
                memory_usage_mb: None,
                status: ModelStatus::Loading,
                performance_metrics: ModelPerformanceMetrics::default(),
                last_used: chrono::Utc::now(),
                usage_count: 0,
            });
        }

        // Attempt to load the model via Ollama
        match self.load_model_via_ollama(&model_id).await {
            Ok(()) => {
                // Update status to ready
                let mut loaded_models = self.loaded_models.write().await;
                if let Some(model) = loaded_models.get_mut(&model_id) {
                    model.status = ModelStatus::Ready;
                    model.memory_usage_mb = Some(self.estimate_model_memory(&model_config));
                }
                log::info!("Successfully loaded model: {}", model_id);
                Ok(())
            },
            Err(e) => {
                // Mark as error and remove from loaded models
                {
                    let mut loaded_models = self.loaded_models.write().await;
                    if let Some(model) = loaded_models.get_mut(&model_id) {
                        model.status = ModelStatus::Error(e.to_string());
                    }
                }
                log::error!("Failed to load model {}: {}", model_id, e);
                Err(e)
            }
        }
    }

    /// Load model through Ollama
    async fn load_model_via_ollama(&self, model_id: &str) -> Result<()> {
        // Check if model is available in Ollama
        let available_models = self.ollama_client.list_models().await?;
        let model_name = format!("{}:latest", model_id); // Adjust naming convention as needed
        
        let model_exists = available_models.iter()
            .any(|m| m.name.contains(model_id) || m.name == model_name);

        if !model_exists {
            // Try to pull the model if not available
            log::info!("Model {} not found in Ollama, attempting to pull...", model_id);
            self.ollama_client.pull_model(&model_name).await
                .map_err(|e| LocalMindError::ExternalService(
                    format!("Failed to pull model {}: {}", model_name, e)
                ))?;
        }

        // Test the model with a simple generation
        let test_request = crate::services::ollama::OllamaRequest {
            model: model_name,
            prompt: "Hello".to_string(),
            stream: false,
            options: Some(crate::services::ollama::OllamaOptions {
                temperature: Some(0.1),
                top_p: Some(0.9),
                top_k: Some(10),
                num_predict: Some(1),
                stop: None,
                repeat_penalty: Some(1.0),
            }),
        };

        self.ollama_client.generate(test_request).await
            .map_err(|e| LocalMindError::ExternalService(
                format!("Model {} failed test generation: {}", model_id, e)
            ))?;

        Ok(())
    }

    /// Unload a model from memory
    pub async fn unload_model(&mut self, model_type: &ModelType) -> Result<()> {
        let model_id = model_type.identifier();
        
        {
            let mut loaded_models = self.loaded_models.write().await;
            if let Some(model) = loaded_models.get_mut(&model_id) {
                model.status = ModelStatus::Unloading;
            }
        }

        // In a full implementation, this would actually unload the model from Ollama
        // For now, we just remove it from our tracking
        {
            let mut loaded_models = self.loaded_models.write().await;
            loaded_models.remove(&model_id);
        }

        log::info!("Unloaded model: {}", model_id);
        Ok(())
    }

    /// Check if a model is loaded and ready
    pub async fn is_model_loaded(&self, model_type: &ModelType) -> Result<bool> {
        let model_id = model_type.identifier();
        let loaded_models = self.loaded_models.read().await;
        
        Ok(loaded_models.get(&model_id)
            .map(|model| model.status == ModelStatus::Ready)
            .unwrap_or(false))
    }

    /// Get list of loaded models
    pub async fn get_loaded_models(&self) -> Result<Vec<String>> {
        let loaded_models = self.loaded_models.read().await;
        Ok(loaded_models.keys().cloned().collect())
    }

    /// Get model information
    pub async fn get_model_info(&self, model_type: &ModelType) -> Result<Option<LoadedModel>> {
        let model_id = model_type.identifier();
        let loaded_models = self.loaded_models.read().await;
        Ok(loaded_models.get(&model_id).cloned())
    }

    /// Update model usage statistics
    pub async fn update_model_usage(&self, model_type: &ModelType, inference_time_ms: u64) -> Result<()> {
        let model_id = model_type.identifier();
        let mut loaded_models = self.loaded_models.write().await;
        
        if let Some(model) = loaded_models.get_mut(&model_id) {
            model.last_used = chrono::Utc::now();
            model.usage_count += 1;
            
            // Update performance metrics
            let metrics = &mut model.performance_metrics;
            let total_time = metrics.average_inference_time_ms * (metrics.total_inferences as f64);
            metrics.total_inferences += 1;
            metrics.average_inference_time_ms = (total_time + inference_time_ms as f64) / metrics.total_inferences as f64;
            metrics.last_inference_time = Some(chrono::Utc::now());
        }

        Ok(())
    }

    /// Ensure enough memory is available for a model
    async fn ensure_memory_available(&mut self, model_config: &ModelConfig) -> Result<()> {
        let required_memory = model_config.resource_requirements.recommended_ram_mb;
        let current_memory_usage = self.get_total_memory_usage().await;

        if current_memory_usage + required_memory > self.total_memory_limit_mb {
            // Need to free up memory
            self.free_memory(required_memory).await?;
        }

        Ok(())
    }

    /// Calculate total memory usage of loaded models
    async fn get_total_memory_usage(&self) -> u64 {
        let loaded_models = self.loaded_models.read().await;
        loaded_models.values()
            .filter_map(|model| model.memory_usage_mb)
            .sum()
    }

    /// Free up memory by unloading least recently used models
    async fn free_memory(&mut self, required_mb: u64) -> Result<()> {
        let mut models_to_unload = Vec::new();
        
        {
            let loaded_models = self.loaded_models.read().await;
            let mut models: Vec<_> = loaded_models.values().collect();
            
            // Sort by last used time (oldest first)
            models.sort_by_key(|model| model.last_used);
            
            let mut freed_memory = 0;
            for model in models {
                if freed_memory >= required_mb {
                    break;
                }
                
                if let Some(memory_usage) = model.memory_usage_mb {
                    models_to_unload.push(model.model_type.clone());
                    freed_memory += memory_usage;
                }
            }
        }

        // Unload the selected models
        for model_type in models_to_unload {
            log::info!("Unloading model {} to free memory", model_type.identifier());
            self.unload_model(&model_type).await?;
        }

        Ok(())
    }

    /// Estimate memory usage for a model
    fn estimate_model_memory(&self, model_config: &ModelConfig) -> u64 {
        // This is a simplified estimation
        // In reality, you'd monitor actual memory usage
        model_config.resource_requirements.recommended_ram_mb
    }

    /// Get model performance statistics
    pub async fn get_performance_stats(&self) -> Result<HashMap<String, ModelPerformanceMetrics>> {
        let loaded_models = self.loaded_models.read().await;
        let mut stats = HashMap::new();
        
        for (model_id, model) in loaded_models.iter() {
            stats.insert(model_id.clone(), model.performance_metrics.clone());
        }
        
        Ok(stats)
    }

    /// Optimize memory usage
    pub async fn optimize_memory(&mut self) -> Result<()> {
        // Force garbage collection and memory optimization
        log::info!("Optimizing model memory usage...");
        
        // In a real implementation, this might:
        // - Defragment model memory
        // - Clear unused caches
        // - Optimize model quantization
        // - Reload models with better memory layout
        
        Ok(())
    }

    /// Preload models based on prediction
    pub async fn preload_models(&mut self, model_types: Vec<ModelType>) -> Result<()> {
        for model_type in model_types {
            if !self.is_model_loaded(&model_type).await? {
                log::info!("Preloading model: {}", model_type.identifier());
                if let Err(e) = self.load_model(&model_type).await {
                    log::warn!("Failed to preload model {}: {}", model_type.identifier(), e);
                }
            }
        }
        Ok(())
    }

    /// Get memory usage summary
    pub async fn get_memory_summary(&self) -> ModelMemorySummary {
        let loaded_models = self.loaded_models.read().await;
        let total_used = self.get_total_memory_usage().await;
        let model_count = loaded_models.len();
        
        ModelMemorySummary {
            total_memory_limit_mb: self.total_memory_limit_mb,
            total_memory_used_mb: total_used,
            available_memory_mb: self.total_memory_limit_mb.saturating_sub(total_used),
            loaded_model_count: model_count,
            memory_utilization_percent: if self.total_memory_limit_mb > 0 {
                (total_used as f64 / self.total_memory_limit_mb as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Update memory limits
    pub fn set_memory_limit(&mut self, limit_mb: u64) {
        self.total_memory_limit_mb = limit_mb;
        log::info!("Updated memory limit to {} MB", limit_mb);
    }

    /// Update max models limit
    pub fn set_max_models(&mut self, max_models: usize) {
        self.max_models_loaded = max_models;
        log::info!("Updated max models limit to {}", max_models);
    }
}

/// Memory usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMemorySummary {
    pub total_memory_limit_mb: u64,
    pub total_memory_used_mb: u64,
    pub available_memory_mb: u64,
    pub loaded_model_count: usize,
    pub memory_utilization_percent: f64,
}

impl Default for ModelPerformanceMetrics {
    fn default() -> Self {
        Self {
            average_inference_time_ms: 0.0,
            tokens_per_second: 0.0,
            total_inferences: 0,
            error_count: 0,
            last_inference_time: None,
        }
    }
}

impl ModelStatus {
    /// Check if the model is ready for inference
    pub fn is_ready(&self) -> bool {
        matches!(self, ModelStatus::Ready)
    }

    /// Check if the model is in an error state
    pub fn is_error(&self) -> bool {
        matches!(self, ModelStatus::Error(_))
    }

    /// Get error message if in error state
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ModelStatus::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ModelType;

    #[test]
    fn test_model_status() {
        let ready = ModelStatus::Ready;
        assert!(ready.is_ready());
        assert!(!ready.is_error());

        let error = ModelStatus::Error("Test error".to_string());
        assert!(!error.is_ready());
        assert!(error.is_error());
        assert_eq!(error.error_message(), Some("Test error"));
    }

    #[test]
    fn test_memory_summary() {
        let summary = ModelMemorySummary {
            total_memory_limit_mb: 1000,
            total_memory_used_mb: 600,
            available_memory_mb: 400,
            loaded_model_count: 2,
            memory_utilization_percent: 60.0,
        };

        assert_eq!(summary.memory_utilization_percent, 60.0);
        assert_eq!(summary.available_memory_mb, 400);
    }

    #[tokio::test]
    async fn test_model_manager_creation() {
        // This test might fail without Ollama setup
        match ModelManager::new().await {
            Ok(manager) => {
                assert_eq!(manager.get_loaded_models().await.unwrap().len(), 0);
            },
            Err(_) => {
                // Expected to fail without Ollama
                println!("ModelManager creation failed (expected without Ollama)");
            }
        }
    }
}