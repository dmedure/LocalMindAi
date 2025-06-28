// State module - Application state management

pub mod app_state;

// Re-export state management functionality
pub use app_state::{initialize_app_state, AppStateManager};