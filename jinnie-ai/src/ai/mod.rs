// AI module - AI response generation and prompt building

pub mod response_generator;
pub mod prompt_builder;

// Re-export main functionality
pub use response_generator::generate_agent_response;
pub use prompt_builder::build_agent_system_prompt;