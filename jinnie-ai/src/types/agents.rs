use serde::{Deserialize, Serialize};

/// Represents an AI agent with specific personality and specialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub specialization: String,
    pub personality: String,
    pub instructions: Option<String>,
    pub created_at: String, // ISO 8601 format for JS compatibility
}

/// Agent specialization types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentSpecialization {
    General,
    Work,
    Coding,
    Research,
    Writing,
    Personal,
    Creative,
    Technical,
}

impl Default for AgentSpecialization {
    fn default() -> Self {
        AgentSpecialization::General
    }
}

/// Agent personality types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentPersonality {
    Professional,
    Friendly,
    Analytical,
    Creative,
    Concise,
    Detailed,
}

impl Default for AgentPersonality {
    fn default() -> Self {
        AgentPersonality::Friendly
    }
}

impl Agent {
    /// Create a new agent with the given parameters
    pub fn new(
        name: String,
        specialization: String,
        personality: String,
        instructions: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            specialization,
            personality,
            instructions,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Check if the agent has custom instructions
    pub fn has_instructions(&self) -> bool {
        self.instructions.is_some() && !self.instructions.as_ref().unwrap().trim().is_empty()
    }
}