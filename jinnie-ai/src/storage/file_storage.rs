use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::types::{Agent, Message, Document};
use crate::utils::error::{LocalMindError, Result};
use super::paths::{get_agents_file_path, get_messages_file_path, get_documents_file_path};

/// Agent storage operations
pub struct AgentStorage;

impl AgentStorage {
    /// Save agents to file
    pub async fn save(agents: &Vec<Agent>) -> Result<()> {
        let agents_file = get_agents_file_path()?;
        
        let json = serde_json::to_string_pretty(agents)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to serialize agents: {}", e)))?;
        
        fs::write(&agents_file, json)
            .map_err(|e| LocalMindError::storage_write_failed(&agents_file.display().to_string()))?;
        
        Ok(())
    }

    /// Load agents from file
    pub async fn load() -> Result<Vec<Agent>> {
        let agents_file = get_agents_file_path()?;
        
        if !agents_file.exists() {
            return Ok(Vec::new());
        }
        
        let json = fs::read_to_string(&agents_file)
            .map_err(|e| LocalMindError::storage_read_failed(&agents_file.display().to_string()))?;
        
        let agents: Vec<Agent> = serde_json::from_str(&json)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to parse agents file: {}", e)))?;
        
        Ok(agents)
    }

    /// Get the size of the agents file
    pub fn get_file_size() -> Result<u64> {
        let agents_file = get_agents_file_path()?;
        if !agents_file.exists() {
            return Ok(0);
        }
        
        let metadata = fs::metadata(agents_file)
            .map_err(|e| LocalMindError::FileSystem(e.to_string()))?;
        Ok(metadata.len())
    }
}

/// Message storage operations
pub struct MessageStorage;

impl MessageStorage {
    /// Save messages to file
    pub async fn save(messages: &HashMap<String, Vec<Message>>) -> Result<()> {
        let messages_file = get_messages_file_path()?;
        
        let json = serde_json::to_string_pretty(messages)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to serialize messages: {}", e)))?;
        
        fs::write(&messages_file, json)
            .map_err(|e| LocalMindError::storage_write_failed(&messages_file.display().to_string()))?;
        
        Ok(())
    }

    /// Load messages from file
    pub async fn load() -> Result<HashMap<String, Vec<Message>>> {
        let messages_file = get_messages_file_path()?;
        
        if !messages_file.exists() {
            return Ok(HashMap::new());
        }
        
        let json = fs::read_to_string(&messages_file)
            .map_err(|e| LocalMindError::storage_read_failed(&messages_file.display().to_string()))?;
        
        let messages: HashMap<String, Vec<Message>> = serde_json::from_str(&json)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to parse messages file: {}", e)))?;
        
        Ok(messages)
    }

    /// Save messages for a specific agent
    pub async fn save_agent_messages(agent_id: &str, messages: &Vec<Message>) -> Result<()> {
        let mut all_messages = Self::load().await?;
        all_messages.insert(agent_id.to_string(), messages.clone());
        Self::save(&all_messages).await
    }

    /// Load messages for a specific agent
    pub async fn load_agent_messages(agent_id: &str) -> Result<Vec<Message>> {
        let all_messages = Self::load().await?;
        Ok(all_messages.get(agent_id).cloned().unwrap_or_default())
    }

    /// Clear messages for a specific agent
    pub async fn clear_agent_messages(agent_id: &str) -> Result<()> {
        let mut all_messages = Self::load().await?;
        all_messages.remove(agent_id);
        Self::save(&all_messages).await
    }

    /// Get the total number of messages across all agents
    pub async fn get_total_message_count() -> Result<usize> {
        let all_messages = Self::load().await?;
        Ok(all_messages.values().map(|messages| messages.len()).sum())
    }

    /// Get the size of the messages file
    pub fn get_file_size() -> Result<u64> {
        let messages_file = get_messages_file_path()?;
        if !messages_file.exists() {
            return Ok(0);
        }
        
        let metadata = fs::metadata(messages_file)
            .map_err(|e| LocalMindError::FileSystem(e.to_string()))?;
        Ok(metadata.len())
    }
}

/// Document storage operations
pub struct DocumentStorage;

impl DocumentStorage {
    /// Save documents to file
    pub async fn save(documents: &Vec<Document>) -> Result<()> {
        let documents_file = get_documents_file_path()?;
        
        let json = serde_json::to_string_pretty(documents)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to serialize documents: {}", e)))?;
        
        fs::write(&documents_file, json)
            .map_err(|e| LocalMindError::storage_write_failed(&documents_file.display().to_string()))?;
        
        Ok(())
    }

    /// Load documents from file
    pub async fn load() -> Result<Vec<Document>> {
        let documents_file = get_documents_file_path()?;
        
        if !documents_file.exists() {
            return Ok(Vec::new());
        }
        
        let json = fs::read_to_string(&documents_file)
            .map_err(|e| LocalMindError::storage_read_failed(&documents_file.display().to_string()))?;
        
        let documents: Vec<Document> = serde_json::from_str(&json)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to parse documents file: {}", e)))?;
        
        Ok(documents)
    }

    /// Add a new document
    pub async fn add_document(document: Document) -> Result<()> {
        let mut documents = Self::load().await?;
        documents.push(document);
        Self::save(&documents).await
    }

    /// Remove a document by ID
    pub async fn remove_document(document_id: &str) -> Result<bool> {
        let mut documents = Self::load().await?;
        let initial_len = documents.len();
        documents.retain(|doc| doc.id != document_id);
        
        if documents.len() < initial_len {
            Self::save(&documents).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Find a document by ID
    pub async fn find_document(document_id: &str) -> Result<Option<Document>> {
        let documents = Self::load().await?;
        Ok(documents.into_iter().find(|doc| doc.id == document_id))
    }

    /// Get the size of the documents file
    pub fn get_file_size() -> Result<u64> {
        let documents_file = get_documents_file_path()?;
        if !documents_file.exists() {
            return Ok(0);
        }
        
        let metadata = fs::metadata(documents_file)
            .map_err(|e| LocalMindError::FileSystem(e.to_string()))?;
        Ok(metadata.len())
    }
}

/// Backup and restore operations
pub struct BackupStorage;

impl BackupStorage {
    /// Create a backup of all data
    pub async fn create_backup() -> Result<PathBuf> {
        use crate::storage::paths::get_exports_dir;
        
        let exports_dir = get_exports_dir()?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = exports_dir.join(format!("backup_{}.json", timestamp));
        
        let backup_data = serde_json::json!({
            "version": "1.0",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "agents": AgentStorage::load().await?,
            "messages": MessageStorage::load().await?,
            "documents": DocumentStorage::load().await?,
        });
        
        let json = serde_json::to_string_pretty(&backup_data)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to serialize backup: {}", e)))?;
        
        fs::write(&backup_file, json)
            .map_err(|e| LocalMindError::storage_write_failed(&backup_file.display().to_string()))?;
        
        Ok(backup_file)
    }

    /// Restore from a backup file
    pub async fn restore_backup(backup_path: &PathBuf) -> Result<()> {
        let json = fs::read_to_string(backup_path)
            .map_err(|e| LocalMindError::storage_read_failed(&backup_path.display().to_string()))?;
        
        let backup_data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| LocalMindError::Serialization(format!("Failed to parse backup file: {}", e)))?;
        
        // Extract and restore agents
        if let Some(agents_data) = backup_data.get("agents") {
            let agents: Vec<Agent> = serde_json::from_value(agents_data.clone())
                .map_err(|e| LocalMindError::Serialization(format!("Failed to parse agents from backup: {}", e)))?;
            AgentStorage::save(&agents).await?;
        }
        
        // Extract and restore messages
        if let Some(messages_data) = backup_data.get("messages") {
            let messages: HashMap<String, Vec<Message>> = serde_json::from_value(messages_data.clone())
                .map_err(|e| LocalMindError::Serialization(format!("Failed to parse messages from backup: {}", e)))?;
            MessageStorage::save(&messages).await?;
        }
        
        // Extract and restore documents
        if let Some(documents_data) = backup_data.get("documents") {
            let documents: Vec<Document> = serde_json::from_value(documents_data.clone())
                .map_err(|e| LocalMindError::Serialization(format!("Failed to parse documents from backup: {}", e)))?;
            DocumentStorage::save(&documents).await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Agent, Message};

    #[tokio::test]
    async fn test_agent_storage() {
        let test_agent = Agent::new(
            "Test Agent".to_string(),
            "general".to_string(),
            "friendly".to_string(),
            None,
        );
        
        let agents = vec![test_agent.clone()];
        
        // Test save
        let result = AgentStorage::save(&agents).await;
        assert!(result.is_ok());
        
        // Test load
        let loaded_agents = AgentStorage::load().await.unwrap();
        assert_eq!(loaded_agents.len(), 1);
        assert_eq!(loaded_agents[0].name, test_agent.name);
    }

    #[tokio::test]
    async fn test_message_storage() {
        let agent_id = "test_agent_id".to_string();
        let test_message = Message::new_user_message(
            "Hello world".to_string(),
            agent_id.clone(),
        );
        
        let messages = vec![test_message.clone()];
        
        // Test save agent messages
        let result = MessageStorage::save_agent_messages(&agent_id, &messages).await;
        assert!(result.is_ok());
        
        // Test load agent messages
        let loaded_messages = MessageStorage::load_agent_messages(&agent_id).await.unwrap();
        assert_eq!(loaded_messages.len(), 1);
        assert_eq!(loaded_messages[0].content, test_message.content);
    }
}