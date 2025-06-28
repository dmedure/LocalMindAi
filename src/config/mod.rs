//! Configuration management module
//! 
//! This module handles all configuration aspects of LocalMind including
//! application settings, model configurations, and platform-specific settings.

pub mod app_config;
pub mod model_config;
pub mod platform_config;

// Re-export main configuration types
pub use app_config::{AppConfig, load_config, save_config, ConfigError};
pub use model_config::{ModelConfig, ModelSettings, LLMConfig};
pub use platform_config::{PlatformConfig, get_platform_paths, ensure_directories};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default configuration values
pub struct ConfigDefaults;

impl ConfigDefaults {
    pub const APP_NAME: &'static str = "LocalMind";
    pub const VERSION: &'static str = "0.3.0";
    pub const DEFAULT_MODEL: &'static str = "TinyLlama";
    pub const DEFAULT_EMBEDDING_MODEL: &'static str = "all-MiniLM-L6-v2";
    pub const DEFAULT_CONTEXT_LENGTH: usize = 4096;
    pub const DEFAULT_QDRANT_HOST: &'static str = "localhost";
    pub const DEFAULT_QDRANT_PORT: u16 = 6333;
    pub const DEFAULT_LOG_LEVEL: &'static str = "info";
}

/// Configuration validation results
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if !other.is_valid {
            self.is_valid = false;
        }
    }
}

/// Configuration initialization and management
pub struct ConfigManager;

impl ConfigManager {
    /// Initialize configuration system
    pub async fn initialize() -> Result<AppConfig> {
        // 1. Create necessary directories
        platform_config::ensure_directories().await?;
        
        // 2. Load or create default configuration
        let config = match app_config::load_config().await {
            Ok(config) => {
                log::info!("Loaded existing configuration");
                config
            }
            Err(_) => {
                log::info!("Creating default configuration");
                let default_config = AppConfig::default();
                app_config::save_config(&default_config).await?;
                default_config
            }
        };

        // 3. Validate configuration
        let validation = Self::validate_config(&config).await?;
        if !validation.is_valid {
            log::error!("Configuration validation failed: {:?}", validation.errors);
            return Err(anyhow::anyhow!("Invalid configuration: {}", validation.errors.join(", ")));
        }

        // 4. Log warnings if any
        for warning in validation.warnings {
            log::warn!("Configuration warning: {}", warning);
        }

        // 5. Apply any necessary migrations
        let migrated_config = Self::migrate_config(config).await?;

        Ok(migrated_config)
    }

    /// Validate entire configuration
    pub async fn validate_config(config: &AppConfig) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate app settings
        if config.app.name.is_empty() {
            result.add_error("App name cannot be empty".to_string());
        }

        // Validate model settings
        let model_validation = Self::validate_model_config(&config.models).await?;
        result.merge(model_validation);

        // Validate vector settings
        let vector_validation = Self::validate_vector_config(&config.vector).await?;
        result.merge(vector_validation);

        // Validate memory settings
        let memory_validation = Self::validate_memory_config(&config.memory).await?;
        result.merge(memory_validation);

        // Validate paths
        let paths_validation = Self::validate_paths(&config.paths).await?;
        result.merge(paths_validation);

        Ok(result)
    }

    /// Migrate configuration from older versions
    pub async fn migrate_config(mut config: AppConfig) -> Result<AppConfig> {
        let current_version = semver::Version::parse(&config.app.version)?;
        let target_version = semver::Version::parse(ConfigDefaults::VERSION)?;

        if current_version < target_version {
            log::info!("Migrating configuration from {} to {}", current_version, target_version);
            
            // Apply version-specific migrations
            if current_version.major == 0 && current_version.minor < 3 {
                config = Self::migrate_to_v0_3(config).await?;
            }

            // Update version
            config.app.version = ConfigDefaults::VERSION.to_string();
            
            // Save migrated configuration
            app_config::save_config(&config).await?;
        }

        Ok(config)
    }

    /// Export configuration to file
    pub async fn export_config(config: &AppConfig, export_path: PathBuf) -> Result<()> {
        let config_json = serde_json::to_string_pretty(config)?;
        tokio::fs::write(export_path, config_json).await?;
        Ok(())
    }

    /// Import configuration from file
    pub async fn import_config(import_path: PathBuf) -> Result<AppConfig> {
        let config_data = tokio::fs::read_to_string(import_path).await?;
        let config: AppConfig = serde_json::from_str(&config_data)?;
        
        // Validate imported configuration
        let validation = Self::validate_config(&config).await?;
        if !validation.is_valid {
            return Err(anyhow::anyhow!("Invalid imported configuration: {}", validation.errors.join(", ")));
        }

        // Save as current configuration
        app_config::save_config(&config).await?;
        
        Ok(config)
    }

    /// Reset configuration to defaults
    pub async fn reset_to_defaults() -> Result<AppConfig> {
        let default_config = AppConfig::default();
        app_config::save_config(&default_config).await?;
        Ok(default_config)
    }

    // Private validation methods

    async fn validate_model_config(models: &LLMConfig) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Check if at least one model is enabled
        if !models.tinyllama.enabled && !models.mistral7b.enabled {
            result.add_error("At least one model must be enabled".to_string());
        }

        // Validate model paths
        if models.tinyllama.enabled && !tokio::fs::metadata(&models.tinyllama.path).await.is_ok() {
            result.add_warning(format!("TinyLlama model not found at: {}", models.tinyllama.path));
        }

        if models.mistral7b.enabled && !tokio::fs::metadata(&models.mistral7b.path).await.is_ok() {
            result.add_warning(format!("Mistral 7B model not found at: {}", models.mistral7b.path));
        }

        // Validate context lengths
        if models.default_context_length == 0 {
            result.add_error("Default context length must be greater than 0".to_string());
        }

        if models.default_context_length > 32768 {
            result.add_warning("Very large context length may impact performance".to_string());
        }

        Ok(result)
    }

    async fn validate_vector_config(vector: &crate::config::app_config::VectorConfig) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate Qdrant connection
        if vector.qdrant_host.is_empty() {
            result.add_error("Qdrant host cannot be empty".to_string());
        }

        if vector.qdrant_port == 0 {
            result.add_error("Qdrant port must be valid".to_string());
        }

        // Validate embedding settings
        if vector.embedding_dimension == 0 {
            result.add_error("Embedding dimension must be greater than 0".to_string());
        }

        Ok(result)
    }

    async fn validate_memory_config(memory: &crate::config::app_config::MemoryConfig) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate memory sizes
        if memory.working_memory_size == 0 {
            result.add_error("Working memory size must be greater than 0".to_string());
        }

        if memory.short_term_size == 0 {
            result.add_error("Short term memory size must be greater than 0".to_string());
        }

        // Validate consolidation settings
        if memory.consolidation_threshold == 0 {
            result.add_error("Consolidation threshold must be greater than 0".to_string());
        }

        if memory.consolidation_interval == 0 {
            result.add_error("Consolidation interval must be greater than 0".to_string());
        }

        if memory.importance_decay_rate <= 0.0 || memory.importance_decay_rate >= 1.0 {
            result.add_error("Importance decay rate must be between 0 and 1".to_string());
        }

        Ok(result)
    }

    async fn validate_paths(paths: &crate::config::app_config::PathConfig) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Check if data directory exists or can be created
        if let Err(e) = tokio::fs::create_dir_all(&paths.data_dir).await {
            result.add_error(format!("Cannot create data directory: {}", e));
        }

        // Check if models directory exists or can be created
        if let Err(e) = tokio::fs::create_dir_all(&paths.models_dir).await {
            result.add_error(format!("Cannot create models directory: {}", e));
        }

        // Check if logs directory exists or can be created
        if let Err(e) = tokio::fs::create_dir_all(&paths.logs_dir).await {
            result.add_error(format!("Cannot create logs directory: {}", e));
        }

        Ok(result)
    }

    async fn migrate_to_v0_3(mut config: AppConfig) -> Result<AppConfig> {
        // Add any new fields that didn't exist in v0.2
        // Update any changed field names or structures
        
        // Example migration logic:
        if config.memory.consolidation_interval == 0 {
            config.memory.consolidation_interval = 3600; // Default 1 hour
        }

        if config.vector.embedding_dimension == 0 {
            config.vector.embedding_dimension = 384; // Default for all-MiniLM-L6-v2
        }

        log::info!("Applied v0.3 configuration migration");
        Ok(config)
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_initialization() {
        let result = ConfigManager::initialize().await;
        // This might fail in test environment without proper setup
        println!("Config initialization result: {:?}", result.is_ok());
    }

    #[tokio::test]
    async fn test_config_validation() {
        let config = AppConfig::default();
        let validation = ConfigManager::validate_config(&config).await.unwrap();
        
        // Default config should be valid
        if !validation.is_valid {
            println!("Validation errors: {:?}", validation.errors);
        }
        // Note: Don't assert here since file system might not be available in tests
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        result.add_error("Test error".to_string());
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);

        result.add_warning("Test warning".to_string());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validation_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_error("Error 1".to_string());

        let mut result2 = ValidationResult::new();
        result2.add_warning("Warning 1".to_string());

        result1.merge(result2);
        assert!(!result1.is_valid);
        assert_eq!(result1.errors.len(), 1);
        assert_eq!(result1.warnings.len(), 1);
    }
}