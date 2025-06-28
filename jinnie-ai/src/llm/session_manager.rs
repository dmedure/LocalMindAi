use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::ModelType;
use crate::utils::error::{LocalMindError, Result};

/// Manages conversation sessions and context
pub struct SessionManager {
    active_sessions: HashMap<String, ConversationSession>,
    max_sessions: usize,
    max_context_length: usize,
    session_timeout_minutes: u64,
}

/// A conversation session with an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    pub session_id: String,
    pub agent_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub messages: Vec<SessionMessage>,
    pub context_summary: Option<String>,
    pub preferred_model: Option<ModelType>,
    pub session_metadata: SessionMetadata,
}

/// Message within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub model_used: Option<String>,
    pub tokens: Option<u32>,
    pub processing_time_ms: Option<u64>,
}

/// Role of message sender
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Session metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub total_messages: usize,
    pub total_tokens: u32,
    pub average_response_time_ms: f64,
    pub models_used: HashMap<String, u32>, // model_name -> usage_count
    pub topics_discussed: Vec<String>,
    pub complexity_trend: f32,
    pub user_satisfaction: Option<f32>,
}

/// Context window for model input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub messages: Vec<SessionMessage>,
    pub total_tokens: u32,
    pub summary: Option<String>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            max_sessions: 100,
            max_context_length: 4096, // Maximum tokens in context
            session_timeout_minutes: 60, // 1 hour timeout
        }
    }

    /// Start a new conversation session
    pub async fn start_session(&mut self, agent_id: String) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        let session = ConversationSession {
            session_id: session_id.clone(),
            agent_id,
            started_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            messages: Vec::new(),
            context_summary: None,
            preferred_model: None,
            session_metadata: SessionMetadata::new(),
        };

        // Check if we need to clean up old sessions
        if self.active_sessions.len() >= self.max_sessions {
            self.cleanup_old_sessions(self.session_timeout_minutes).await?;
        }

        self.active_sessions.insert(session_id.clone(), session);
        log::debug!("Started new session: {} for agent: {}", session_id, session.agent_id);
        
        Ok(session_id)
    }

    /// Add a message to a session
    pub async fn add_message(
        &mut self,
        session_id: &str,
        role: MessageRole,
        content: String,
        model_used: Option<String>,
        tokens: Option<u32>,
        processing_time_ms: Option<u64>,
    ) -> Result<()> {
        let session = self.active_sessions.get_mut(session_id)
            .ok_or_else(|| LocalMindError::Message(format!("Session not found: {}", session_id)))?;

        let message = SessionMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: role.clone(),
            content,
            timestamp: chrono::Utc::now(),
            model_used: model_used.clone(),
            tokens,
            processing_time_ms,
        };

        session.messages.push(message);
        session.last_activity = chrono::Utc::now();

        // Update session metadata
        session.session_metadata.total_messages += 1;
        
        if let Some(token_count) = tokens {
            session.session_metadata.total_tokens += token_count;
        }

        if let Some(time_ms) = processing_time_ms {
            let current_avg = session.session_metadata.average_response_time_ms;
            let message_count = session.session_metadata.total_messages as f64;
            session.session_metadata.average_response_time_ms = 
                (current_avg * (message_count - 1.0) + time_ms as f64) / message_count;
        }

        if let Some(model) = model_used {
            *session.session_metadata.models_used.entry(model).or_insert(0) += 1;
        }

        // Trim context if it's getting too long
        self.manage_context_length(session).await?;

        Ok(())
    }

    /// Update session with user and assistant messages
    pub async fn update_session(
        &mut self,
        session_id: &str,
        agent_id: &str,
        user_message: &str,
        assistant_response: &str,
        model_used: &ModelType,
    ) -> Result<()> {
        // Ensure session exists or create it
        if !self.active_sessions.contains_key(session_id) {
            let new_session = ConversationSession {
                session_id: session_id.to_string(),
                agent_id: agent_id.to_string(),
                started_at: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
                messages: Vec::new(),
                context_summary: None,
                preferred_model: Some(model_used.clone()),
                session_metadata: SessionMetadata::new(),
            };
            self.active_sessions.insert(session_id.to_string(), new_session);
        }

        // Add user message
        self.add_message(
            session_id,
            MessageRole::User,
            user_message.to_string(),
            None,
            Some(estimate_tokens(user_message)),
            None,
        ).await?;

        // Add assistant response
        self.add_message(
            session_id,
            MessageRole::Assistant,
            assistant_response.to_string(),
            Some(model_used.display_name()),
            Some(estimate_tokens(assistant_response)),
            None,
        ).await?;

        Ok(())
    }

    /// Get context for a session (recent messages within token limit)
    pub async fn get_context(&self, session_id: &str, max_messages: usize) -> Result<Option<String>> {
        let session = self.active_sessions.get(session_id)
            .ok_or_else(|| LocalMindError::Message(format!("Session not found: {}", session_id)))?;

        if session.messages.is_empty() {
            return Ok(None);
        }

        let context_window = self.build_context_window(session, max_messages)?;
        Ok(Some(self.format_context_for_model(&context_window)))
    }

    /// Build context window for model input
    fn build_context_window(&self, session: &ConversationSession, max_messages: usize) -> Result<ContextWindow> {
        let mut total_tokens = 0;
        let mut selected_messages = Vec::new();

        // Start from the most recent messages and work backwards
        for message in session.messages.iter().rev().take(max_messages) {
            let message_tokens = message.tokens.unwrap_or_else(|| estimate_tokens(&message.content));
            
            if total_tokens + message_tokens > self.max_context_length as u32 {
                break; // Would exceed context limit
            }

            total_tokens += message_tokens;
            selected_messages.insert(0, message.clone()); // Insert at beginning to maintain order
        }

        Ok(ContextWindow {
            messages: selected_messages,
            total_tokens,
            summary: session.context_summary.clone(),
        })
    }

    /// Format context window for model consumption
    fn format_context_for_model(&self, context: &ContextWindow) -> String {
        let mut formatted = String::new();

        // Add summary if available
        if let Some(summary) = &context.summary {
            formatted.push_str("Previous conversation summary:\n");
            formatted.push_str(summary);
            formatted.push_str("\n\nRecent conversation:\n");
        }

        // Add recent messages
        for message in &context.messages {
            let role_str = match message.role {
                MessageRole::User => "Human",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
            };
            formatted.push_str(&format!("{}: {}\n", role_str, message.content));
        }

        formatted
    }

    /// Manage context length by summarizing old messages
    async fn manage_context_length(&mut self, session: &mut ConversationSession) -> Result<()> {
        let total_tokens: u32 = session.messages.iter()
            .map(|m| m.tokens.unwrap_or_else(|| estimate_tokens(&m.content)))
            .sum();

        if total_tokens > self.max_context_length as u32 {
            // Need to summarize older messages
            let cutoff_point = session.messages.len() / 2; // Keep recent half
            let messages_to_summarize = &session.messages[..cutoff_point];
            
            if !messages_to_summarize.is_empty() {
                let summary = self.summarize_messages(messages_to_summarize).await?;
                session.context_summary = Some(summary);
                
                // Remove the summarized messages
                session.messages.drain(..cutoff_point);
                
                log::debug!("Summarized {} messages for session {}", cutoff_point, session.session_id);
            }
        }

        Ok(())
    }

    /// Summarize a sequence of messages
    async fn summarize_messages(&self, messages: &[SessionMessage]) -> Result<String> {
        let mut content = String::new();
        for message in messages {
            let role_str = match message.role {
                MessageRole::User => "Human",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
            };
            content.push_str(&format!("{}: {}\n", role_str, message.content));
        }

        // In a real implementation, this would use the AI model to create a summary
        // For now, create a simple summary
        let summary = format!(
            "Previous conversation ({} messages): Discussed various topics including the user's questions and assistant's responses. Key themes and context preserved.",
            messages.len()
        );

        Ok(summary)
    }

    /// Get session information
    pub async fn get_session(&self, session_id: &str) -> Result<Option<ConversationSession>> {
        Ok(self.active_sessions.get(session_id).cloned())
    }

    /// Check if session exists
    pub async fn session_exists(&self, session_id: &str) -> Result<bool> {
        Ok(self.active_sessions.contains_key(session_id))
    }

    /// End a session
    pub async fn end_session(&mut self, session_id: &str) -> Result<bool> {
        let removed = self.active_sessions.remove(session_id).is_some();
        if removed {
            log::debug!("Ended session: {}", session_id);
        }
        Ok(removed)
    }

    /// Get active session count
    pub fn get_active_session_count(&self) -> usize {
        self.active_sessions.len()
    }

    /// Clean up old sessions
    pub async fn cleanup_old_sessions(&mut self, max_age_minutes: u64) -> Result<usize> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::minutes(max_age_minutes as i64);
        let mut sessions_to_remove = Vec::new();

        for (session_id, session) in &self.active_sessions {
            if session.last_activity < cutoff_time {
                sessions_to_remove.push(session_id.clone());
            }
        }

        let removed_count = sessions_to_remove.len();
        for session_id in sessions_to_remove {
            self.active_sessions.remove(&session_id);
        }

        if removed_count > 0 {
            log::info!("Cleaned up {} old sessions", removed_count);
        }

        Ok(removed_count)
    }

    /// Get session statistics
    pub async fn get_session_stats(&self) -> SessionManagerStats {
        let total_sessions = self.active_sessions.len();
        let total_messages: usize = self.active_sessions.values()
            .map(|session| session.messages.len())
            .sum();

        let average_session_length = if total_sessions > 0 {
            total_messages as f64 / total_sessions as f64
        } else {
            0.0
        };

        let oldest_session = self.active_sessions.values()
            .min_by_key(|session| session.started_at)
            .map(|session| session.started_at);

        SessionManagerStats {
            total_active_sessions: total_sessions,
            total_messages,
            average_session_length,
            oldest_session_age: oldest_session.map(|start| {
                (chrono::Utc::now() - start).num_minutes()
            }),
        }
    }

    /// Update session preferences
    pub async fn update_session_preferences(
        &mut self,
        session_id: &str,
        preferred_model: Option<ModelType>,
    ) -> Result<()> {
        let session = self.active_sessions.get_mut(session_id)
            .ok_or_else(|| LocalMindError::Message(format!("Session not found: {}", session_id)))?;

        session.preferred_model = preferred_model;
        session.last_activity = chrono::Utc::now();

        Ok(())
    }

    /// Rate the session (user satisfaction)
    pub async fn rate_session(&mut self, session_id: &str, rating: f32) -> Result<()> {
        let session = self.active_sessions.get_mut(session_id)
            .ok_or_else(|| LocalMindError::Message(format!("Session not found: {}", session_id)))?;

        session.session_metadata.user_satisfaction = Some(rating.clamp(1.0, 5.0));
        Ok(())
    }

    /// Export session for backup or analysis
    pub async fn export_session(&self, session_id: &str) -> Result<String> {
        let session = self.active_sessions.get(session_id)
            .ok_or_else(|| LocalMindError::Message(format!("Session not found: {}", session_id)))?;

        serde_json::to_string_pretty(session)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to export session: {}", e)))
    }

    /// Import session from backup
    pub async fn import_session(&mut self, session_data: &str) -> Result<String> {
        let session: ConversationSession = serde_json::from_str(session_data)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to import session: {}", e)))?;

        let session_id = session.session_id.clone();
        self.active_sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }
}

/// Session manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagerStats {
    pub total_active_sessions: usize,
    pub total_messages: usize,
    pub average_session_length: f64,
    pub oldest_session_age: Option<i64>, // in minutes
}

impl SessionMetadata {
    /// Create new empty metadata
    fn new() -> Self {
        Self {
            total_messages: 0,
            total_tokens: 0,
            average_response_time_ms: 0.0,
            models_used: HashMap::new(),
            topics_discussed: Vec::new(),
            complexity_trend: 0.5,
            user_satisfaction: None,
        }
    }

    /// Get most used model
    pub fn most_used_model(&self) -> Option<String> {
        self.models_used
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(model, _)| model.clone())
    }

    /// Get usage statistics
    pub fn get_usage_summary(&self) -> String {
        format!(
            "{} messages, {} tokens, {:.1}ms avg response, {} models used",
            self.total_messages,
            self.total_tokens,
            self.average_response_time_ms,
            self.models_used.len()
        )
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate token count for text (simplified)
fn estimate_tokens(text: &str) -> u32 {
    // Rough approximation: 1 token ≈ 4 characters for English text
    (text.len() as f32 / 4.0).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let mut manager = SessionManager::new();
        let session_id = manager.start_session("test_agent".to_string()).await.unwrap();
        
        assert!(manager.session_exists(&session_id).await.unwrap());
        assert_eq!(manager.get_active_session_count(), 1);
    }

    #[tokio::test]
    async fn test_message_addition() {
        let mut manager = SessionManager::new();
        let session_id = manager.start_session("test_agent".to_string()).await.unwrap();
        
        manager.add_message(
            &session_id,
            MessageRole::User,
            "Hello".to_string(),
            None,
            Some(5),
            None,
        ).await.unwrap();

        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.session_metadata.total_messages, 1);
        assert_eq!(session.session_metadata.total_tokens, 5);
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let mut manager = SessionManager::new();
        let session_id = manager.start_session("test_agent".to_string()).await.unwrap();
        
        // Manually set old timestamp
        if let Some(session) = manager.active_sessions.get_mut(&session_id) {
            session.last_activity = chrono::Utc::now() - chrono::Duration::hours(2);
        }
        
        let cleaned = manager.cleanup_old_sessions(60).await.unwrap(); // 1 hour timeout
        assert_eq!(cleaned, 1);
        assert_eq!(manager.get_active_session_count(), 0);
    }

    #[tokio::test]
    async fn test_context_building() {
        let mut manager = SessionManager::new();
        let session_id = manager.start_session("test_agent".to_string()).await.unwrap();
        
        // Add some messages
        for i in 1..=5 {
            manager.add_message(
                &session_id,
                if i % 2 == 1 { MessageRole::User } else { MessageRole::Assistant },
                format!("Message {}", i),
                None,
                Some(10),
                None,
            ).await.unwrap();
        }

        let context = manager.get_context(&session_id, 3).await.unwrap();
        assert!(context.is_some());
        
        let context_str = context.unwrap();
        assert!(context_str.contains("Message"));
    }

    #[test]
    fn test_token_estimation() {
        assert_eq!(estimate_tokens("Hello world"), 3);
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("A"), 1);
    }

    #[test]
    fn test_session_metadata() {
        let mut metadata = SessionMetadata::new();
        assert_eq!(metadata.total_messages, 0);
        assert!(metadata.most_used_model().is_none());
        
        metadata.models_used.insert("TinyLlama".to_string(), 5);
        metadata.models_used.insert("Mistral7B".to_string(), 3);
        
        assert_eq!(metadata.most_used_model(), Some("TinyLlama".to_string()));
    }
}