use crate::types::AppState;
use crate::storage::{AgentStorage, MessageStorage, DocumentStorage};
use crate::utils::error::{LocalMindError, Result};

/// Application state manager with initialization and persistence logic
pub struct AppStateManager;

impl AppStateManager {
    /// Initialize application state with data loaded from storage
    pub async fn initialize_data(state: &mut AppState) -> Result<()> {
        // Load existing data from storage
        match Self::load_persisted_data(state).await {
            Ok(_) => {
                log::info!("Successfully loaded persisted data");
            },
            Err(e) => {
                log::warn!("Failed to load some persisted data: {}", e);
                // Continue anyway - first run or corrupted data
            }
        }
        
        Ok(())
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

        log::info!("All state data saved successfully");
        Ok(())
    }

    /// Create default agents for first-time users
    pub async fn create_default_agents(state: &AppState) -> Result<()> {
        use crate::types::Agent;
        
        let mut agents = state.agents.lock().await;
        
        if agents.is_empty() {
            log::info!("Creating default agents for first-time user");
            
            // General Assistant
            let general = Agent::new("General Assistant".to_string(), "TinyLlama".to_string())
                .with_description("A helpful AI assistant for general tasks and conversations")
                .with_system_prompt("You are a helpful, friendly, and knowledgeable AI assistant. Provide clear, accurate, and helpful responses to user queries.")
                .with_personality("Friendly");
            
            agents.insert(general.id.clone(), general);
            
            // Code Expert
            let coder = Agent::new("Code Expert".to_string(), "TinyLlama".to_string())
                .with_description("Specialized in programming, debugging, and software development")
                .with_system_prompt("You are an expert programmer proficient in multiple languages. Help with coding, debugging, architecture design, and best practices. Provide clear code examples and explanations.")
                .with_personality("Professional")
                .with_specialization("Programming");
            
            agents.insert(coder.id.clone(), coder);
            
            // Creative Writer
            let writer = Agent::new("Creative Writer".to_string(), "TinyLlama".to_string())
                .with_description("Assists with creative writing, storytelling, and content creation")
                .with_system_prompt("You are a creative writing assistant. Help with story ideas, character development, writing techniques, and content creation. Be imaginative and inspiring.")
                .with_personality("Creative")
                .with_specialization("Writing");
            
            agents.insert(writer.id.clone(), writer);
            
            // Save the default agents
            AgentStorage::save(&agents).await?;
            
            log::info!("Created {} default agents", agents.len());
        }
        
        Ok(())
    }
}