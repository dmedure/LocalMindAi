// Commands module - Tauri command handlers

pub mod agent_commands;
pub mod chat_commands;
pub mod system_commands;
pub mod document_commands;

// Re-export all commands for easy registration
pub use agent_commands::*;
pub use chat_commands::*;
pub use system_commands::*;
pub use document_commands::*;