use serde::{Deserialize, Serialize};

/// Main error type for LocalMind application
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum LocalMindError {
    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Message error: {0}")]
    Message(String),

    #[error("Document error: {0}")]
    Document(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("AI service error: {0}")]
    AiService(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for LocalMindError {
    fn from(error: std::io::Error) -> Self {
        LocalMindError::FileSystem(error.to_string())
    }
}

impl From<serde_json::Error> for LocalMindError {
    fn from(error: serde_json::Error) -> Self {
        LocalMindError::Serialization(error.to_string())
    }
}

impl From<reqwest::Error> for LocalMindError {
    fn from(error: reqwest::Error) -> Self {
        LocalMindError::Network(error.to_string())
    }
}

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, LocalMindError>;

/// Helper functions for creating specific error types
impl LocalMindError {
    pub fn agent_not_found(agent_id: &str) -> Self {
        LocalMindError::Agent(format!("Agent not found: {}", agent_id))
    }

    pub fn invalid_agent_data(reason: &str) -> Self {
        LocalMindError::Agent(format!("Invalid agent data: {}", reason))
    }

    pub fn message_send_failed(reason: &str) -> Self {
        LocalMindError::Message(format!("Failed to send message: {}", reason))
    }

    pub fn ollama_unavailable() -> Self {
        LocalMindError::ExternalService("Ollama service is not available".to_string())
    }

    pub fn chroma_unavailable() -> Self {
        LocalMindError::ExternalService("ChromaDB service is not available".to_string())
    }

    pub fn storage_read_failed(path: &str) -> Self {
        LocalMindError::Storage(format!("Failed to read from: {}", path))
    }

    pub fn storage_write_failed(path: &str) -> Self {
        LocalMindError::Storage(format!("Failed to write to: {}", path))
    }

    pub fn validation_failed(field: &str, reason: &str) -> Self {
        LocalMindError::Validation(format!("Validation failed for {}: {}", field, reason))
    }
}