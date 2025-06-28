use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub sender: String, // "user" or "agent"
    pub timestamp: String, // ISO 8601 format for JS compatibility
    pub agent_id: String,
    pub attachments: Option<Vec<MessageAttachment>>,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub model_used: Option<String>,
    pub response_time_ms: Option<u64>,
    pub token_count: Option<u32>,
    pub memory_accessed: Option<Vec<String>>,
    pub confidence_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub agent_id: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub context_window: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResponse {
    pub message_id: String,
    pub token: Option<String>,
    pub model_used: Option<String>,
    pub complete: bool,
    pub error: Option<String>,
}

impl Message {
    pub fn new_user_message(content: String, agent_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            sender: "user".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            agent_id,
            attachments: None,
            metadata: None,
        }
    }
    
    pub fn new_agent_message(content: String, agent_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            sender: "agent".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            agent_id,
            attachments: None,
            metadata: None,
        }
    }
    
    pub fn with_metadata(mut self, metadata: MessageMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    pub fn add_attachment(&mut self, attachment: MessageAttachment) {
        if let Some(ref mut attachments) = self.attachments {
            attachments.push(attachment);
        } else {
            self.attachments = Some(vec![attachment]);
        }
    }
}