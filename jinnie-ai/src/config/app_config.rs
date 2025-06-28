use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::model_config::LLMConfig;
use super::platform_config::get_platform_paths;
use super::ConfigDefaults;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub models: LLMConfig,
    pub memory: MemoryConfig,
    pub vector: VectorConfig,
    pub performance: PerformanceConfig,
    pub privacy: PrivacyConfig,
    pub ui: UIConfig,
    pub paths: PathConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub name: String,
    pub version: String,
    pub data_dir: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub working_memory_size: usize,
    pub short_term_size: usize,
    pub consolidation_threshold: usize,
    pub consolidation_interval: u64, // seconds
    pub importance_decay_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    pub qdrant_host: String,
    pub qdrant_port: u16,
    pub qdrant_api_key: Option<String>,
    pub collection_prefix: String,
    pub embedding_model: String,
    pub embedding_dimension: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub cache_size_mb: usize,
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
    pub stream_buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub telemetry_enabled: bool,
    pub crash_reports_enabled: bool,
    pub memory_encryption: bool,
    pub auto_cleanup_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String, // "light", "dark", "auto"
    pub language: String,
    pub show_model_indicator: bool,
    pub show_performance_stats: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub data_dir: String,
    pub models_dir: String,
    pub logs_dir: String,
    pub exports_dir: String,
    pub cache_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let platform_paths = get_platform_paths();
        
        Self {
            app: AppSettings {
                name: ConfigDefaults::APP_NAME.to_string(),
                version: ConfigDefaults::VERSION.to_string(),
                data_dir: platform_paths.data_dir.to_string_lossy().to_string(),
                log_level: ConfigDefaults::DEFAULT_LOG_LEVEL.to_string(),
            },
            models: LLMConfig::default(),
            memory: MemoryConfig {
                working_memory_size: 20,
                short_term_size: 100,
                consolidation_threshold: 50,
                consolidation_interval: 3600, // 1 hour
                importance_decay_rate: 0.95,
            },
            vector: VectorConfig {
                qdrant_host: ConfigDefaults::DEFAULT_QDRANT_HOST.to_string(),
                qdrant_port: ConfigDefaults::DEFAULT_QDRANT_PORT,
                qdrant_api_key: None,
                collection_prefix: "localmind".to_string(),
                embedding_model: ConfigDefaults::DEFAULT_EMBEDDING_MODEL.to_string(),
                embedding_dimension: 384,
            },
            performance: PerformanceConfig {
                cache_size_mb: 512,
                max_concurrent_requests: 10,
                request_timeout_seconds: 300,
                stream_buffer_size: 1024,
            },
            privacy: PrivacyConfig {
                telemetry_enabled: false,
                crash_reports_enabled: false,
                memory_encryption: true,
                auto_cleanup_days: Some(90),
            },
            ui: UIConfig {
                theme: "auto".to_string(),
                language: "en".to_string(),
                show_model_indicator: true,
                show_performance_stats: false,
            },
            paths: PathConfig {
                data_dir: platform_paths.data_dir.to_string_lossy().to_string(),
                models_dir: platform_paths.models_dir.to_string_lossy().to_string(),
                logs_dir: platform_paths.logs_dir.to_string_lossy().to_string(),
                exports_dir: platform_paths.exports_dir.to_string_lossy().to_string(),
                cache_dir: platform_paths.cache_dir.to_string_lossy().to_string(),
            },
        }
    }
}

impl AppConfig {
    /// Create a new configuration with custom data directory
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        let mut config = Self::default();
        config.app.data_dir = data_dir.to_string_lossy().to_string();
        config.paths.data_dir = data_dir.to_string_lossy().to_string();
        
        // Update other paths relative to data directory
        config.paths.models_dir = data_dir.join("models").to_string_lossy().to_string();
        config.paths.logs_dir = data_dir.join("logs").to_string_lossy().to_string();
        config.paths.exports_dir = data_dir.join("exports").to_string_lossy().to_string();
        config.paths.cache_dir = data_dir.join("cache").to_string_lossy().to_string();
        
        config
    }

    /// Get the configuration file path
    pub fn config_file_path() -> Result<PathBuf> {
        let platform_paths = get_platform_paths();
        Ok(platform_paths.config_dir.join("config.toml"))
    }

    /// Get data directory as PathBuf
    pub fn data_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.paths.data_dir)
    }

    /// Get models directory as PathBuf
    pub fn models_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.paths.models_dir)
    }

    /// Get logs directory as PathBuf
    pub fn logs_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.paths.logs_dir)
    }

    /// Get exports directory as PathBuf
    pub fn exports_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.paths.exports_dir)
    }

    /// Get cache directory as PathBuf
    pub fn cache_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.paths.cache_dir)
    }

    /// Update configuration with environment variable overrides
    pub fn apply_env_overrides(&mut self) -> Result<()> {
        // Override with environment variables if present
        if let Ok(log_level) = std::env::var("LOCALMIND_LOG_LEVEL") {
            self.app.log_level = log_level;
        }

        if let Ok(data_dir) = std::env::var("LOCALMIND_DATA_DIR") {
            self.app.data_dir = data_dir.clone();
            self.paths.data_dir = data_dir;
        }

        if let Ok(qdrant_host) = std::env::var("LOCALMIND_QDRANT_HOST") {
            self.vector.qdrant_host = qdrant_host;
        }

        if let Ok(qdrant_port) = std::env::var("LOCALMIND_QDRANT_PORT") {
            self.vector.qdrant_port = qdrant_port.parse().unwrap_or(self.vector.qdrant_port);
        }

        if let Ok(api_key) = std::env::var("LOCALMIND_QDRANT_API_KEY") {
            self.vector.qdrant_api_key = Some(api_key);
        }

        if let Ok(telemetry) = std::env::var("LOCALMIND_TELEMETRY") {
            self.privacy.telemetry_enabled = telemetry.parse().unwrap_or(false);
        }

        Ok(())
    }

    /// Validate configuration consistency
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Validate app settings
        if self.app.name.is_empty() {
            issues.push("App name cannot be empty".to_string());
        }

        // Validate memory settings
        if self.memory.working_memory_size == 0 {
            issues.push("Working memory size must be greater than 0".to_string());
        }

        if self.memory.importance_decay_rate <= 0.0 || self.memory.importance_decay_rate >= 1.0 {
            issues.push("Importance decay rate must be between 0 and 1".to_string());
        }

        // Validate vector settings
        if self.vector.qdrant_host.is_empty() {
            issues.push("Qdrant host cannot be empty".to_string());
        }

        if self.vector.qdrant_port == 0 {
            issues.push("Qdrant port must be valid".to_string());
        }

        if self.vector.embedding_dimension == 0 {
            issues.push("Embedding dimension must be greater than 0".to_string());
        }

        // Validate performance settings
        if self.performance.max_concurrent_requests == 0 {
            issues.push("Max concurrent requests must be greater than 0".to_string());
        }

        if self.performance.request_timeout_seconds == 0 {
            issues.push("Request timeout must be greater than 0".to_string());
        }

        // Validate paths
        if self.paths.data_dir.is_empty() {
            issues.push("Data directory path cannot be empty".to_string());
        }

        Ok(issues)
    }
}

/// Configuration loading and saving
pub async fn load_config() -> Result<AppConfig> {
    let config_path = AppConfig::config_file_path()?;
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Configuration file not found"));
    }

    let config_content = tokio::fs::read_to_string(&config_path).await?;
    let mut config: AppConfig = toml::from_str(&config_content)?;
    
    // Apply environment variable overrides
    config.apply_env_overrides()?;
    
    Ok(config)
}

pub async fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = AppConfig::config_file_path()?;
    
    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    let config_content = toml::to_string_pretty(config)?;
    tokio::fs::write(&config_path, config_content).await?;
    
    Ok(())
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found")]
    NotFound,
    
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] toml::ser::Error),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.app.name, "LocalMind");
        assert_eq!(config.app.version, "0.3.0");
        assert!(!config.privacy.telemetry_enabled);
        assert!(config.privacy.memory_encryption);
    }

    #[test]
    fn test_config_with_custom_data_dir() {
        let custom_dir = PathBuf::from("/custom/data/dir");
        let config = AppConfig::with_data_dir(custom_dir.clone());
        
        assert_eq!(config.data_dir_path(), custom_dir);
        assert!(config.models_dir_path().starts_with(&custom_dir));
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        
        // Valid config should have no issues
        let issues = config.validate().unwrap();
        assert!(issues.is_empty());
        
        // Invalid config should report issues
        config.app.name = "".to_string();
        config.memory.working_memory_size = 0;
        let issues = config.validate().unwrap();
        assert!(!issues.is_empty());
        assert!(issues.len() >= 2);
    }

    #[test]
    fn test_env_overrides() {
        let mut config = AppConfig::default();
        
        // Set environment variables
        std::env::set_var("LOCALMIND_LOG_LEVEL", "debug");
        std::env::set_var("LOCALMIND_QDRANT_HOST", "remote-qdrant");
        std::env::set_var("LOCALMIND_QDRANT_PORT", "6334");
        
        config.apply_env_overrides().unwrap();
        
        assert_eq!(config.app.log_level, "debug");
        assert_eq!(config.vector.qdrant_host, "remote-qdrant");
        assert_eq!(config.vector.qdrant_port, 6334);
        
        // Clean up
        std::env::remove_var("LOCALMIND_LOG_LEVEL");
        std::env::remove_var("LOCALMIND_QDRANT_HOST");
        std::env::remove_var("LOCALMIND_QDRANT_PORT");
    }

    #[tokio::test]
    async fn test_config_serialization() {
        let config = AppConfig::default();
        
        // Test TOML serialization
        let toml_string = toml::to_string_pretty(&config).unwrap();
        assert!(toml_string.contains("[app]"));
        assert!(toml_string.contains("[models]"));
        
        // Test deserialization
        let deserialized: AppConfig = toml::from_str(&toml_string).unwrap();
        assert_eq!(deserialized.app.name, config.app.name);
        assert_eq!(deserialized.vector.qdrant_port, config.vector.qdrant_port);
    }

    #[test]
    fn test_path_helpers() {
        let config = AppConfig::default();
        
        assert!(config.data_dir_path().is_absolute() || config.data_dir_path().is_relative());
        assert!(config.models_dir_path().to_string_lossy().contains("models"));
        assert!(config.logs_dir_path().to_string_lossy().contains("logs"));
        assert!(config.exports_dir_path().to_string_lossy().contains("exports"));
        assert!(config.cache_dir_path().to_string_lossy().contains("cache"));
    }
}