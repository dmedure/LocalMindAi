// Storage module - File-based persistence and data management

pub mod file_storage;
pub mod paths;

// Re-export storage functionality
pub use file_storage::{AgentStorage, MessageStorage, DocumentStorage};
pub use paths::{get_data_dir, ensure_data_dir, get_agents_file_path, get_messages_file_path};