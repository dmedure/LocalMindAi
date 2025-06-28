// Services module - External service integrations

pub mod ollama;
pub mod chroma;

// Re-export service status checkers
pub use ollama::check_ollama_status;
pub use chroma::check_chromadb_status;