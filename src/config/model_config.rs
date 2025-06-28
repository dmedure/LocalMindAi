use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::platform_config::get_platform_paths;
use super::ConfigDefaults;

/// LLM configuration for multi-model support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub default_strategy: String, // "adaptive", "manual", "performance", "quality"
    pub default_context_length: usize,
    pub tinyllama: ModelSettings,
    pub mistral7b: ModelSettings,
}

/// Individual model settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    pub enabled: bool,
    pub path: String,
    pub context_length: usize,
    pub batch_size: usize,
    pub threads: usize,
    pub gpu_layers: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: Option<u32>,
    pub repeat_penalty: f32,
    pub mmap: bool,
    pub mlock: bool,
}

/// Combined model configuration wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub llm: LLMConfig,
    pub embedding: EmbeddingConfig,
}

/// Embedding model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub model_path: Option<String>,
    pub dimension: usize,
    pub batch_size: usize,
    pub max_sequence_length: usize,
    pub normalize_embeddings: bool,
}

impl Default for LLMConfig {
    fn default() -> Self {
        let platform_paths = get_platform_paths();
        
        Self {
            default_strategy: "adaptive".to_string(),
            default_context_length: ConfigDefaults::DEFAULT_CONTEXT_LENGTH,
            tinyllama: ModelSettings {
                enabled: true,
                path: platform_paths.models_dir
                    .join("tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf")
                    .to_string_lossy()
                    .to_string(),
                context_length: 2048,
                batch_size: 512,
                threads: 4,
                gpu_layers: 0,
                temperature: 0.7,
                top_p: 0.9,
                top_k: Some(40),
                repeat_penalty: 1.1,
                mmap: true,
                mlock: false,
            },
            mistral7b: ModelSettings {
                enabled: true,
                path: platform_paths.models_dir
                    .join("mistral-7b-instruct-v0.2.Q4_K_M.gguf")
                    .to_string_lossy()
                    .to_string(),
                context_length: 8192,
                batch_size: 512,
                threads: 8,
                gpu_layers: 35,
                temperature: 0.7,
                top_p: 0.9,
                top_k: Some(40),
                repeat_penalty: 1.1,
                mmap: true,
                mlock: false,
            },
        }
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: ConfigDefaults::DEFAULT_EMBEDDING_MODEL.to_string(),
            model_path: None, // Will use built-in model
            dimension: 384,
            batch_size: 32,
            max_sequence_length: 256,
            normalize_embeddings: true,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig::default(),
            embedding: EmbeddingConfig::default(),
        }
    }
}

impl ModelSettings {
    /// Create optimized settings for performance
    pub fn performance_optimized() -> Self {
        Self {
            enabled: true,
            path: String::new(), // Will be set based on model
            context_length: 2048, // Smaller context for speed
            batch_size: 1024,     // Larger batch size
            threads: num_cpus::get(), // Use all available cores
            gpu_layers: 0,         // CPU only for consistency
            temperature: 0.3,      // Lower temperature for deterministic responses
            top_p: 0.8,
            top_k: Some(20),       // Smaller top_k for speed
            repeat_penalty: 1.05,
            mmap: true,
            mlock: true,           // Lock in memory for speed
        }
    }

    /// Create optimized settings for quality
    pub fn quality_optimized() -> Self {
        Self {
            enabled: true,
            path: String::new(),
            context_length: 8192,  // Larger context for better understanding
            batch_size: 256,       // Smaller batch for quality
            threads: num_cpus::get().min(8), // Don't overwhelm system
            gpu_layers: 100,       // Use GPU if available
            temperature: 0.7,      // Balanced creativity
            top_p: 0.95,          // Higher diversity
            top_k: Some(50),       // More options
            repeat_penalty: 1.1,
            mmap: true,
            mlock: false,
        }
    }

    /// Create balanced settings
    pub fn balanced() -> Self {
        Self {
            enabled: true,
            path: String::new(),
            context_length: 4096,
            batch_size: 512,
            threads: num_cpus::get() / 2,
            gpu_layers: 20,
            temperature: 0.7,
            top_p: 0.9,
            top_k: Some(40),
            repeat_penalty: 1.1,
            mmap: true,
            mlock: false,
        }
    }

    /// Validate model settings
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled && self.path.is_empty() {
            return Err("Model path cannot be empty when model is enabled".to_string());
        }

        if self.context_length == 0 {
            return Err("Context length must be greater than 0".to_string());
        }

        if self.context_length > 32768 {
            return Err("Context length cannot exceed 32768 tokens".to_string());
        }

        if self.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }

        if self.threads == 0 {
            return Err("Thread count must be greater than 0".to_string());
        }

        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err("Temperature must be between 0.0 and 2.0".to_string());
        }

        if self.top_p <= 0.0 || self.top_p > 1.0 {
            return Err("Top-p must be between 0.0 and 1.0".to_string());
        }

        if let Some(top_k) = self.top_k {
            if top_k == 0 {
                return Err("Top-k must be greater than 0 if specified".to_string());
            }
        }

        if self.repeat_penalty < 0.0 {
            return Err("Repeat penalty must be positive".to_string());
        }

        Ok(())
    }

    /// Get estimated memory usage in MB
    pub fn estimate_memory_usage(&self) -> f64 {
        // Rough estimation based on model size and settings
        let base_model_size = if self.path.contains("tinyllama") {
            1100.0 // MB for TinyLlama 1.1B
        } else if self.path.contains("mistral") {
            7000.0 // MB for Mistral 7B
        } else {
            4000.0 // Default estimate
        };

        let context_overhead = (self.context_length as f64 * 4.0) / 1_000_000.0; // 4 bytes per token
        let batch_overhead = (self.batch_size as f64 * 1024.0) / 1_000_000.0; // Rough estimate

        base_model_size + context_overhead + batch_overhead
    }

    /// Check if GPU acceleration is configured
    pub fn uses_gpu(&self) -> bool {
        self.gpu_layers > 0
    }

    /// Get performance tier
    pub fn performance_tier(&self) -> PerformanceTier {
        if self.context_length <= 2048 && self.batch_size >= 1024 && self.threads >= 6 {
            PerformanceTier::High
        } else if self.context_length <= 4096 && self.batch_size >= 512 {
            PerformanceTier::Medium
        } else {
            PerformanceTier::Low
        }
    }
}

impl LLMConfig {
    /// Get the model selection strategy
    pub fn get_strategy(&self) -> ModelSelectionStrategy {
        match self.default_strategy.as_str() {
            "adaptive" => ModelSelectionStrategy::Adaptive,
            "manual" => ModelSelectionStrategy::Manual,
            "performance" => ModelSelectionStrategy::Performance,
            "quality" => ModelSelectionStrategy::Quality,
            _ => ModelSelectionStrategy::Adaptive,
        }
    }

    /// Get enabled models
    pub fn enabled_models(&self) -> Vec<&str> {
        let mut models = Vec::new();
        if self.tinyllama.enabled {
            models.push("tinyllama");
        }
        if self.mistral7b.enabled {
            models.push("mistral7b");
        }
        models
    }

    /// Get model settings by name
    pub fn get_model_settings(&self, model_name: &str) -> Option<&ModelSettings> {
        match model_name {
            "tinyllama" => Some(&self.tinyllama),
            "mistral7b" => Some(&self.mistral7b),
            _ => None,
        }
    }

    /// Get mutable model settings by name
    pub fn get_model_settings_mut(&mut self, model_name: &str) -> Option<&mut ModelSettings> {
        match model_name {
            "tinyllama" => Some(&mut self.tinyllama),
            "mistral7b" => Some(&mut self.mistral7b),
            _ => None,
        }
    }

    /// Update model path
    pub fn set_model_path(&mut self, model_name: &str, path: PathBuf) -> Result<(), String> {
        match model_name {
            "tinyllama" => {
                self.tinyllama.path = path.to_string_lossy().to_string();
                Ok(())
            }
            "mistral7b" => {
                self.mistral7b.path = path.to_string_lossy().to_string();
                Ok(())
            }
            _ => Err(format!("Unknown model: {}", model_name)),
        }
    }

    /// Validate all model configurations
    pub fn validate(&self) -> Result<(), String> {
        if !self.tinyllama.enabled && !self.mistral7b.enabled {
            return Err("At least one model must be enabled".to_string());
        }

        if self.default_context_length == 0 {
            return Err("Default context length must be greater than 0".to_string());
        }

        if self.tinyllama.enabled {
            self.tinyllama.validate().map_err(|e| format!("TinyLlama config error: {}", e))?;
        }

        if self.mistral7b.enabled {
            self.mistral7b.validate().map_err(|e| format!("Mistral 7B config error: {}", e))?;
        }

        Ok(())
    }

    /// Get total estimated memory usage
    pub fn total_memory_usage(&self) -> f64 {
        let mut total = 0.0;
        if self.tinyllama.enabled {
            total += self.tinyllama.estimate_memory_usage();
        }
        if self.mistral7b.enabled {
            total += self.mistral7b.estimate_memory_usage();
        }
        total
    }
}

impl EmbeddingConfig {
    /// Validate embedding configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.model_name.is_empty() {
            return Err("Embedding model name cannot be empty".to_string());
        }

        if self.dimension == 0 {
            return Err("Embedding dimension must be greater than 0".to_string());
        }

        if self.dimension > 4096 {
            return Err("Embedding dimension seems unusually large".to_string());
        }

        if self.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }

        if self.max_sequence_length == 0 {
            return Err("Max sequence length must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Check if using local model file
    pub fn is_local_model(&self) -> bool {
        self.model_path.is_some()
    }

    /// Get model identifier for downloads
    pub fn model_identifier(&self) -> String {
        if let Some(ref path) = self.model_path {
            format!("local:{}", path)
        } else {
            format!("huggingface:{}", self.model_name)
        }
    }
}

/// Model selection strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelSelectionStrategy {
    /// Automatically choose based on task complexity and resources
    Adaptive,
    /// User manually selects model
    Manual,
    /// Always prefer fastest model
    Performance,
    /// Always prefer highest quality model
    Quality,
}

/// Performance tiers for models
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceTier {
    High,
    Medium,
    Low,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_llm_config() {
        let config = LLMConfig::default();
        assert_eq!(config.default_strategy, "adaptive");
        assert!(config.tinyllama.enabled);
        assert!(config.mistral7b.enabled);
        assert!(config.default_context_length > 0);
    }

    #[test]
    fn test_model_settings_validation() {
        let mut settings = ModelSettings::balanced();
        settings.path = "test.gguf".to_string();
        
        // Valid settings should pass
        assert!(settings.validate().is_ok());
        
        // Invalid temperature should fail
        settings.temperature = -1.0;
        assert!(settings.validate().is_err());
        
        // Invalid context length should fail
        settings.temperature = 0.7;
        settings.context_length = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_model_settings_presets() {
        let performance = ModelSettings::performance_optimized();
        let quality = ModelSettings::quality_optimized();
        let balanced = ModelSettings::balanced();
        
        assert!(performance.context_length <= balanced.context_length);
        assert!(balanced.context_length <= quality.context_length);
        assert!(performance.temperature <= quality.temperature);
    }

    #[test]
    fn test_memory_usage_estimation() {
        let tinyllama = ModelSettings {
            enabled: true,
            path: "tinyllama.gguf".to_string(),
            context_length: 2048,
            batch_size: 512,
            ..ModelSettings::balanced()
        };
        
        let memory_usage = tinyllama.estimate_memory_usage();
        assert!(memory_usage > 1000.0); // Should be at least 1GB
        assert!(memory_usage < 3000.0); // Should be less than 3GB for TinyLlama
    }

    #[test]
    fn test_llm_config_enabled_models() {
        let mut config = LLMConfig::default();
        let enabled = config.enabled_models();
        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&"tinyllama"));
        assert!(enabled.contains(&"mistral7b"));
        
        config.tinyllama.enabled = false;
        let enabled = config.enabled_models();
        assert_eq!(enabled.len(), 1);
        assert!(enabled.contains(&"mistral7b"));
    }

    #[test]
    fn test_model_selection_strategy() {
        let mut config = LLMConfig::default();
        assert_eq!(config.get_strategy(), ModelSelectionStrategy::Adaptive);
        
        config.default_strategy = "performance".to_string();
        assert_eq!(config.get_strategy(), ModelSelectionStrategy::Performance);
        
        config.default_strategy = "quality".to_string();
        assert_eq!(config.get_strategy(), ModelSelectionStrategy::Quality);
    }

    #[test]
    fn test_embedding_config() {
        let config = EmbeddingConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.model_name, "all-MiniLM-L6-v2");
        assert_eq!(config.dimension, 384);
        assert!(!config.is_local_model());
    }

    #[test]
    fn test_performance_tier_classification() {
        let high_perf = ModelSettings {
            context_length: 2048,
            batch_size: 1024,
            threads: 8,
            ..ModelSettings::balanced()
        };
        assert_eq!(high_perf.performance_tier(), PerformanceTier::High);
        
        let low_perf = ModelSettings {
            context_length: 8192,
            batch_size: 128,
            threads: 2,
            ..ModelSettings::balanced()
        };
        assert_eq!(low_perf.performance_tier(), PerformanceTier::Low);
    }

    #[test]
    fn test_gpu_detection() {
        let mut settings = ModelSettings::balanced();
        settings.gpu_layers = 0;
        assert!(!settings.uses_gpu());
        
        settings.gpu_layers = 20;
        assert!(settings.uses_gpu());
    }
}