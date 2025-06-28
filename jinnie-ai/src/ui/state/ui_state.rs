use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Import types from the main crate
use crate::types::{
    message::{Message as BackendMessage, MessageRole as BackendMessageRole},
    agent::Agent as BackendAgent,
};

// UI-specific message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub role: MessageRole,
    pub timestamp: u64,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl From<BackendMessageRole> for MessageRole {
    fn from(role: BackendMessageRole) -> Self {
        match role {
            BackendMessageRole::User => MessageRole::User,
            BackendMessageRole::Assistant => MessageRole::Assistant,
            BackendMessageRole::System => MessageRole::System,
        }
    }
}

// UI-specific agent type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub model: String,
    pub description: String,
    pub system_prompt: String,
    pub is_active: bool,
}

impl From<BackendAgent> for Agent {
    fn from(agent: BackendAgent) -> Self {
        Self {
            id: agent.id,
            name: agent.name,
            model: agent.model,
            description: agent.description,
            system_prompt: agent.system_prompt,
            is_active: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub agent_id: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Default)]
pub struct UIState {
    pub messages: Vec<Message>,
    pub chats: Vec<Chat>,
    pub agents: Vec<Agent>,
    pub current_chat_id: Option<String>,
    pub current_agent_id: String,
    pub is_ai_typing: bool,
    pub is_streaming: bool,
    pub current_input: String,
    pub sidebar_collapsed: bool,
    pub agent_sidebar_collapsed: bool,
    pub loading_agents: bool,
    pub loading_messages: bool,
    pub error_message: Option<String>,
}

impl UIState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_current_agent(&mut self, agent_id: String) {
        self.current_agent_id = agent_id;
        // Mark the agent as active
        for agent in &mut self.agents {
            agent.is_active = agent.id == self.current_agent_id;
        }
    }
    
    pub fn add_message(&mut self, content: String, role: MessageRole) {
        let message = Message {
            id: Uuid::new_v4().to_string(),
            content,
            role,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            agent_id: self.current_agent_id.clone(),
        };
        self.messages.push(message);
    }
    
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
    
    pub fn set_typing(&mut self, typing: bool) {
        self.is_ai_typing = typing;
    }
    
    pub fn set_error(&mut self, error: Option<String>) {
        self.error_message = error;
    }
    
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
    }
    
    pub fn toggle_agent_sidebar(&mut self) {
        self.agent_sidebar_collapsed = !self.agent_sidebar_collapsed;
    }
    
    pub fn load_agents(&mut self, backend_agents: Vec<BackendAgent>) {
        self.agents = backend_agents.into_iter()
            .map(|agent| agent.into())
            .collect();
        
        // Set first agent as active if none selected
        if self.current_agent_id.is_empty() && !self.agents.is_empty() {
            self.set_current_agent(self.agents[0].id.clone());
        }
    }
    
    pub fn set_loading_agents(&mut self, loading: bool) {
        self.loading_agents = loading;
    }
    
    pub fn set_loading_messages(&mut self, loading: bool) {
        self.loading_messages = loading;
    }
}