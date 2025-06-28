// Types module - Data structures used throughout the application

pub mod agent;
pub mod message;
pub mod document;
pub mod app_state;

// Re-export all public types
pub use agent::Agent;
pub use message::Message;
pub use document::Document;
pub use app_state::{AppState, ServiceStatus};