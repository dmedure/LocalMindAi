use crate::types::AppState;
use crate::storage::{AgentStorage, MessageStorage, DocumentStorage};
use crate::utils::error::{LocalMindError, Result};

/// Application state manager with initialization and persistence logic
pub struct AppStateManager;

impl AppStateManager {
    /// Initialize application state with data loaded from storage
    pub async fn initialize() -> Result<AppState> {
        let state = AppState::new();
        
        // Load existing data from storage
        match Self::load_persisted_data(&state).await {
            Ok(_) => {
                log::info!("Successfully loaded persisted data");
            },
            Err(e) => {
                log::warn!("Failed to load some persisted data: {}", e);
                // Continue anyway - first run or corrupted data
            }
        }
        
        Ok(state)
    }

    /// Load all persisted data into the application state
    async fn load_persisted_data(state: &AppState) -> Result<()> {
        // Load agents
        match AgentStorage::load().await {
            Ok(agents) => {
                *state.agents.lock().await = agents;
                log::debug!("Loaded {} agents from storage", state.agents.lock().await.len());
            },
            Err(e) => {
                log::error!("Failed to load agents: {}", e);
                return Err(e);
            }
        }

        // Load messages
        match MessageStorage::load().await {
            Ok(messages) => {
                *state.messages.lock().await = messages;
                let total_messages: usize = state.messages.lock().await.values().map(|v| v.len()).sum();
                log::debug!("Loaded {} total messages from storage", total_messages);
            },
            Err(e) => {
                log::error!("Failed to load messages: {}", e);
                return Err(e);
            }
        }

        // Load documents
        match DocumentStorage::load().await {
            Ok(documents) => {
                *state.documents.lock().await = documents;
                log::debug!("Loaded {} documents from storage", state.documents.lock().await.len());
            },
            Err(e) => {
                log::error!("Failed to load documents: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Save all current state data to storage
    pub async fn save_all_data(state: &AppState) -> Result<()> {
        // Save agents
        let agents = state.agents.lock().await;
        AgentStorage::save(&agents).await
            .map_err(|e| {
                log::error!("Failed to save agents: {}", e);
                e
            })?;
        drop(agents);

        // Save messages
        let messages = state.messages.lock().await;
        MessageStorage::save(&messages).await
            .map_err(|e| {
                log::error!("Failed to save messages: {}", e);
                e
            })?;
        drop(messages);

        // Save documents
        let documents = state.documents.lock().await;
        DocumentStorage::save(&documents).await
            .map_err(|e| {
                log::error!("Failed to save documents: {}", e);
                e
            })?;
        drop(documents);

        log::info!("Successfully saved all application data");
        Ok(())
    }

    /// Perform a graceful shutdown, saving all data
    pub async fn shutdown(state: &AppState) -> Result<()> {
        log::info!("Initiating graceful shutdown...");
        
        Self::save_all_data(state).await?;
        
        log::info!("Graceful shutdown completed");
        Ok(())
    }

    /// Validate application state integrity
    pub async fn validate_state(state: &AppState) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check agents
        let agents = state.agents.lock().await;
        for agent in agents.iter() {
            if agent.name.trim().is_empty() {
                issues.push(format!("Agent {} has empty name", agent.id));
            }
            
            // Validate agent data
            if let Err(e) = crate::utils::validation::validate_agent_name(&agent.name) {
                issues.push(format!("Agent {}: {}", agent.id, e));
            }
        }
        drop(agents);

        // Check messages consistency
        let messages = state.messages.lock().await;
        let agents = state.agents.lock().await;
        let agent_ids: std::collections::HashSet<String> = agents.iter().map(|a| a.id.clone()).collect();
        
        for (agent_id, agent_messages) in messages.iter() {
            if !agent_ids.contains(agent_id) {
                issues.push(format!("Messages exist for non-existent agent: {}", agent_id));
            }
            
            // Check message timestamps
            for message in agent_messages {
                if let Err(e) = crate::utils::validation::validate_timestamp(&message.timestamp) {
                    issues.push(format!("Message {} has invalid timestamp: {}", message.id, e));
                }
            }
        }
        drop(messages);
        drop(agents);

        // Check documents
        let documents = state.documents.lock().await;
        for document in documents.iter() {
            if !std::path::Path::new(&document.path).exists() {
                issues.push(format!("Document {} references missing file: {}", document.id, document.path));
            }
        }
        drop(documents);

        if issues.is_empty() {
            log::info!("Application state validation passed");
        } else {
            log::warn!("Application state validation found {} issues", issues.len());
        }

        Ok(issues)
    }

    /// Clean up orphaned data (messages without agents, etc.)
    pub async fn cleanup_orphaned_data(state: &AppState) -> Result<usize> {
        let mut cleaned_count = 0;

        // Get valid agent IDs
        let agents = state.agents.lock().await;
        let valid_agent_ids: std::collections::HashSet<String> = agents.iter().map(|a| a.id.clone()).collect();
        drop(agents);

        // Clean up orphaned messages
        let mut messages = state.messages.lock().await;
        let orphaned_agents: Vec<String> = messages.keys()
            .filter(|agent_id| !valid_agent_ids.contains(*agent_id))
            .cloned()
            .collect();

        for agent_id in orphaned_agents {
            if let Some(orphaned_messages) = messages.remove(&agent_id) {
                cleaned_count += orphaned_messages.len();
                log::info!("Cleaned up {} orphaned messages for agent {}", orphaned_messages.len(), agent_id);
            }
        }
        drop(messages);

        // Save cleaned data
        if cleaned_count > 0 {
            Self::save_all_data(state).await?;
            log::info!("Cleaned up {} orphaned items", cleaned_count);
        }

        Ok(cleaned_count)
    }

    /// Get application state statistics
    pub async fn get_state_statistics(state: &AppState) -> serde_json::Value {
        let agent_count = state.agent_count().await;
        let total_messages = state.total_message_count().await;
        let document_count = state.document_count().await;

        // Get storage sizes
        let agents_size = AgentStorage::get_file_size().unwrap_or(0);
        let messages_size = MessageStorage::get_file_size().unwrap_or(0);
        let documents_size = DocumentStorage::get_file_size().unwrap_or(0);

        // Get service status
        let service_status = state.service_status.lock().await.clone();

        serde_json::json!({
            "agents": {
                "count": agent_count,
                "storage_bytes": agents_size
            },
            "messages": {
                "count": total_messages,
                "storage_bytes": messages_size
            },
            "documents": {
                "count": document_count,
                "storage_bytes": documents_size
            },
            "storage": {
                "total_bytes": agents_size + messages_size + documents_size,
                "agents_percentage": if agents_size + messages_size + documents_size > 0 {
                    (agents_size as f64 / (agents_size + messages_size + documents_size) as f64) * 100.0
                } else { 0.0 },
                "messages_percentage": if agents_size + messages_size + documents_size > 0 {
                    (messages_size as f64 / (agents_size + messages_size + documents_size) as f64) * 100.0
                } else { 0.0 },
                "documents_percentage": if agents_size + messages_size + documents_size > 0 {
                    (documents_size as f64 / (agents_size + messages_size + documents_size) as f64) * 100.0
                } else { 0.0 }
            },
            "services": service_status,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Reset application state (clear all data)
    pub async fn reset_state(state: &AppState) -> Result<()> {
        log::warn!("Resetting application state - all data will be lost");
        
        // Clear in-memory state
        state.agents.lock().await.clear();
        state.messages.lock().await.clear();
        state.documents.lock().await.clear();
        
        // Save empty state to disk
        Self::save_all_data(state).await?;
        
        log::info!("Application state reset completed");
        Ok(())
    }

    /// Create a backup of current state
    pub async fn create_backup(state: &AppState) -> Result<std::path::PathBuf> {
        // Use the backup functionality from storage
        crate::storage::file_storage::BackupStorage::create_backup().await
    }

    /// Restore state from backup
    pub async fn restore_from_backup(state: &AppState, backup_path: &std::path::PathBuf) -> Result<()> {
        // Restore from backup file
        crate::storage::file_storage::BackupStorage::restore_backup(backup_path).await?;
        
        // Reload the restored data into current state
        Self::load_persisted_data(state).await?;
        
        log::info!("Successfully restored state from backup: {}", backup_path.display());
        Ok(())
    }
}

/// Initialize the application state (convenience function)
pub async fn initialize_app_state() -> Result<AppState> {
    AppStateManager::initialize().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Agent;

    #[tokio::test]
    async fn test_state_initialization() {
        let state = AppStateManager::initialize().await;
        assert!(state.is_ok());
        
        let state = state.unwrap();
        assert_eq!(state.agent_count().await, 0); // Should start empty for tests
    }

    #[tokio::test]
    async fn test_state_validation() {
        let state = AppState::new();
        
        // Add a test agent
        let agent = Agent::new(
            "Test Agent".to_string(),
            "general".to_string(),
            "friendly".to_string(),
            None,
        );
        state.agents.lock().await.push(agent);
        
        let issues = AppStateManager::validate_state(&state).await.unwrap();
        assert_eq!(issues.len(), 0); // Should have no validation issues
    }

    #[tokio::test]
    async fn test_cleanup_orphaned_data() {
        let state = AppState::new();
        
        // Add orphaned message (agent doesn't exist)
        let orphaned_message = crate::types::Message::new_user_message(
            "Orphaned message".to_string(),
            "non-existent-agent".to_string(),
        );
        state.add_message("non-existent-agent".to_string(), orphaned_message).await;
        
        let cleaned_count = AppStateManager::cleanup_orphaned_data(&state).await.unwrap();
        assert_eq!(cleaned_count, 1);
        
        // Verify the orphaned message was removed
        let messages = state.get_agent_messages("non-existent-agent").await;
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_state_statistics() {
        let state = AppState::new();
        let stats = AppStateManager::get_state_statistics(&state).await;
        
        assert_eq!(stats["agents"]["count"], 0);
        assert_eq!(stats["messages"]["count"], 0);
        assert_eq!(stats["documents"]["count"], 0);
        assert!(stats["timestamp"].is_string());
    }
}