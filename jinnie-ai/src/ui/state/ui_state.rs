use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export your existing types if available, otherwise use these local ones
pub use local_ai_agent::types::{
    message::{Message as BackendMessage, MessageRole as BackendMessageRole},
    agent::Agent as BackendAgent,
};

// UI-specific message type that wraps your backend message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub role: MessageRole,
    pub timestamp: u64,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

// UI-specific agent type that wraps your backend agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub model: String,
    pub description: String,
    pub system_prompt: String,
    pub is_active: bool,
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
        Self {
            messages: Vec::new(),
            chats: Vec::new(),
            agents: Vec::new(),
            current_chat_id: None,
            current_agent_id: String::new(),
            is_ai_typing: false,
            is_streaming: false,
            current_input: String::new(),
            sidebar_collapsed: false,
            agent_sidebar_collapsed: false,
            loading_agents: false,
            loading_messages: false,
            error_message: None,
        }
    }

    /// Add a message to the current chat
    pub fn add_message(&mut self, content: String, role: MessageRole) {
        let message = Message {
            id: Uuid::new_v4().to_string(),
            content,
            role,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            agent_id: self.current_agent_id.clone(),
        };
        self.messages.push(message);
    }

    /// Set AI typing status
    pub fn set_typing(&mut self, is_typing: bool) {
        self.is_ai_typing = is_typing;
    }

    /// Set streaming status
    pub fn set_streaming(&mut self, is_streaming: bool) {
        self.is_streaming = is_streaming;
    }

    /// Update current input
    pub fn set_current_input(&mut self, input: String) {
        self.current_input = input;
    }

    /// Get the currently active agent
    pub fn get_current_agent(&self) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == self.current_agent_id)
    }

    /// Switch to a different agent
    pub fn switch_agent(&mut self, agent_id: String) {
        // Deactivate all agents
        for agent in &mut self.agents {
            agent.is_active = false;
        }
        
        // Activate the selected agent
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.is_active = true;
            self.current_agent_id = agent_id;
        }
    }

    /// Create a new chat
    pub fn create_new_chat(&mut self, title: String) -> String {
        let chat_id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let chat = Chat {
            id: chat_id.clone(),
            title,
            agent_id: self.current_agent_id.clone(),
            created_at: now,
            updated_at: now,
        };
        
        self.chats.push(chat);
        self.current_chat_id = Some(chat_id.clone());
        
        // Clear messages for new chat
        self.messages.clear();
        
        chat_id
    }

    /// Get the current chat
    pub fn get_current_chat(&self) -> Option<&Chat> {
        if let Some(chat_id) = &self.current_chat_id {
            self.chats.iter().find(|c| &c.id == chat_id)
        } else {
            None
        }
    }

    /// Switch to a different chat
    pub fn switch_chat(&mut self, chat_id: String) {
        self.current_chat_id = Some(chat_id);
        // TODO: Load messages for this chat from backend
        self.messages.clear();
        self.loading_messages = true;
    }

    /// Toggle history sidebar
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
    }

    /// Toggle agent sidebar
    pub fn toggle_agent_sidebar(&mut self) {
        self.agent_sidebar_collapsed = !self.agent_sidebar_collapsed;
    }

    /// Set loading state for agents
    pub fn set_loading_agents(&mut self, loading: bool) {
        self.loading_agents = loading;
    }

    /// Set loading state for messages
    pub fn set_loading_messages(&mut self, loading: bool) {
        self.loading_messages = loading;
    }

    /// Set error message
    pub fn set_error(&mut self, error: Option<String>) {
        self.error_message = error;
    }

    /// Load agents from backend
    pub fn load_agents(&mut self, backend_agents: Vec<BackendAgent>) {
        self.agents = backend_agents.into_iter().map(|agent| {
            Agent {
                id: agent.id,
                name: agent.name,
                model: agent.model_name,
                description: agent.description.unwrap_or_default(),
                system_prompt: agent.system_prompt.unwrap_or_default(),
                is_active: false,
            }
        }).collect();

        // Set first agent as active if none is selected
        if self.current_agent_id.is_empty() && !self.agents.is_empty() {
            self.agents[0].is_active = true;
            self.current_agent_id = self.agents[0].id.clone();
        }
    }

    /// Load messages from backend
    pub fn load_messages(&mut self, backend_messages: Vec<BackendMessage>) {
        self.messages = backend_messages.into_iter().map(|msg| {
            Message {
                id: msg.id,
                content: msg.content,
                role: match msg.role {
                    BackendMessageRole::User => MessageRole::User,
                    BackendMessageRole::Assistant => MessageRole::Assistant,
                    BackendMessageRole::System => MessageRole::System,
                },
                timestamp: msg.timestamp.unwrap_or(0),
                agent_id: msg.agent_id,
            }
        }).collect();
    }

    /// Clear all messages in current chat
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Add a streaming message (for real-time AI responses)
    pub fn add_streaming_message(&mut self, content: String) {
        if let Some(last_message) = self.messages.last_mut() {
            if matches!(last_message.role, MessageRole::Assistant) && self.is_streaming {
                // Update the last assistant message with new content
                last_message.content = content;
                return;
            }
        }
        
        // Create new assistant message
        self.add_message(content, MessageRole::Assistant);
    }
}

// Conversion helpers
impl From<BackendMessageRole> for MessageRole {
    fn from(role: BackendMessageRole) -> Self {
        match role {
            BackendMessageRole::User => MessageRole::User,
            BackendMessageRole::Assistant => MessageRole::Assistant,
            BackendMessageRole::System => MessageRole::System,
        }
    }
}

impl From<MessageRole> for BackendMessageRole {
    fn from(role: MessageRole) -> Self {
        match role {
            MessageRole::User => BackendMessageRole::User,
            MessageRole::Assistant => BackendMessageRole::Assistant,
            MessageRole::System => BackendMessageRole::System,
        }
    }
}