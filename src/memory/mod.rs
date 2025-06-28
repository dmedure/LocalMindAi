//! Memory management module implementing MemGPT-style hierarchical memory
//! 
//! This module provides a sophisticated memory system with multiple layers:
//! - Working: Current conversation context
//! - Short-term: Recent interactions  
//! - Long-term: Important persistent facts
//! - Episodic: Specific events and experiences
//! - Semantic: General knowledge and concepts
//! - Reflective: AI-generated insights and patterns

pub mod memory_types;
pub mod memory_manager;
pub mod importance_scorer;
pub mod consolidation;
pub mod retrieval;

// Re-export commonly used types and structs
pub use memory_types::{
    Memory, MemoryLayer, MemoryMetadata, MemorySource, MemoryQuery, MemoryUpdate,
    ConsolidationStrategy, ConsolidationReport, PruningStrategy, PruningReport,
    Insight, InsightType, Entity, EntityType, Sentiment, VerificationStatus,
    AssociationType, MemoryAssociation, DateRange, ContextWindow
};

pub use memory_manager::{MemoryManager, MemoryStats};
pub use importance_scorer::{ImportanceScorer, ImportanceTrends};
pub use consolidation::ConsolidationEngine;
pub use retrieval::{MemoryRetrieval, SearchResult};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::vector::VectorStore;

/// Main memory coordinator that manages the entire memory system
pub struct MemoryCoordinator {
    memory_manager: Arc<Mutex<MemoryManager>>,
    importance_scorer: Arc<ImportanceScorer>,
    consolidation_engine: Arc<ConsolidationEngine>,
    retrieval_engine: Arc<MemoryRetrieval>,
}

impl MemoryCoordinator {
    /// Create a new memory coordinator
    pub async fn new(config: AppConfig, vector_store: Option<VectorStore>) -> Result<Self> {
        let memory_manager = Arc::new(Mutex::new(
            MemoryManager::new(config, vector_store).await?
        ));
        let importance_scorer = Arc::new(ImportanceScorer::new());
        let consolidation_engine = Arc::new(ConsolidationEngine::new());
        let retrieval_engine = Arc::new(MemoryRetrieval::new());

        Ok(Self {
            memory_manager,
            importance_scorer,
            consolidation_engine,
            retrieval_engine,
        })
    }

    /// Get a reference to the memory manager
    pub fn memory_manager(&self) -> Arc<Mutex<MemoryManager>> {
        Arc::clone(&self.memory_manager)
    }

    /// Get a reference to the importance scorer
    pub fn importance_scorer(&self) -> Arc<ImportanceScorer> {
        Arc::clone(&self.importance_scorer)
    }

    /// Get a reference to the consolidation engine
    pub fn consolidation_engine(&self) -> Arc<ConsolidationEngine> {
        Arc::clone(&self.consolidation_engine)
    }

    /// Get a reference to the retrieval engine
    pub fn retrieval_engine(&self) -> Arc<MemoryRetrieval> {
        Arc::clone(&self.retrieval_engine)
    }

    /// Store a new memory with automatic importance scoring
    pub async fn store_memory(&self, content: String, metadata: MemoryMetadata) -> Result<Memory> {
        let mut manager = self.memory_manager.lock().await;
        manager.store(content, metadata).await
    }

    /// Retrieve memories based on query
    pub async fn retrieve_memories(&self, query: MemoryQuery) -> Result<Vec<Memory>> {
        let manager = self.memory_manager.lock().await;
        self.retrieval_engine.search(&*manager, query).await
    }

    /// Perform memory consolidation
    pub async fn consolidate_memories(&self) -> Result<ConsolidationReport> {
        let mut manager = self.memory_manager.lock().await;
        self.consolidation_engine.consolidate_memories(&mut *manager).await
    }

    /// Generate insights from memories
    pub async fn generate_insights(&self) -> Result<Vec<Insight>> {
        let manager = self.memory_manager.lock().await;
        self.consolidation_engine.generate_insights(&*manager).await
    }

    /// Get memory statistics
    pub async fn get_memory_stats(&self) -> Result<MemoryStats> {
        let manager = self.memory_manager.lock().await;
        Ok(manager.get_stats().clone())
    }

    /// Search memories with detailed scoring
    pub async fn search_with_scores(&self, query: MemoryQuery) -> Result<Vec<SearchResult>> {
        let manager = self.memory_manager.lock().await;
        self.retrieval_engine.search_with_scores(&*manager, query).await
    }

    /// Find similar memories to a given memory
    pub async fn find_similar_memories(&self, target_memory: &Memory, limit: usize) -> Result<Vec<Memory>> {
        let manager = self.memory_manager.lock().await;
        self.retrieval_engine.find_similar(&*manager, target_memory, limit).await
    }

    /// Get recent memories for context
    pub async fn get_recent_context(&self, agent_id: &str, limit: usize) -> Result<Vec<Memory>> {
        let manager = self.memory_manager.lock().await;
        self.retrieval_engine.get_recent_context(&*manager, agent_id, limit).await
    }

    /// Update memory importance based on user feedback
    pub async fn update_memory_importance(&self, memory_id: uuid::Uuid, user_rating: f32) -> Result<()> {
        let mut manager = self.memory_manager.lock().await;
        
        // Find the memory
        if let Some((_, memory)) = manager.find_memory_mut(memory_id) {
            // Update importance using the scorer's feedback learning
            let new_importance = self.importance_scorer.update_importance_with_feedback(memory, user_rating);
            memory.importance_score = new_importance;
        }
        
        Ok(())
    }

    /// Analyze importance trends for an agent
    pub async fn analyze_importance_trends(&self, agent_id: &str) -> Result<ImportanceTrends> {
        let manager = self.memory_manager.lock().await;
        let agent_memories = manager.get_agent_memories(agent_id).await?;
        Ok(self.importance_scorer.analyze_importance_trends(&agent_memories))
    }

    /// Clear all memories for an agent
    pub async fn clear_agent_memories(&self, agent_id: &str) -> Result<usize> {
        let mut manager = self.memory_manager.lock().await;
        manager.clear_agent_memories(agent_id).await
    }

    /// Prune memories based on strategy
    pub async fn prune_memories(&self, strategy: PruningStrategy) -> Result<PruningReport> {
        let mut manager = self.memory_manager.lock().await;
        manager.prune(strategy).await
    }

    /// Create association between memories
    pub async fn create_memory_association(
        &self,
        memory1: uuid::Uuid,
        memory2: uuid::Uuid,
        association_type: AssociationType,
        strength: f32,
    ) -> Result<()> {
        let mut manager = self.memory_manager.lock().await;
        manager.associate(memory1, memory2, association_type, strength).await
    }

    /// Schedule automatic consolidation
    pub async fn schedule_consolidation(&self) -> Result<()> {
        // In a real implementation, this would set up a background task
        // to periodically run consolidation based on configuration
        
        let stats = {
            let manager = self.memory_manager.lock().await;
            manager.get_stats().clone()
        };

        // Check if consolidation is needed
        let working_memory_count = stats.memories_by_layer.get(&MemoryLayer::Working).unwrap_or(&0);
        let short_term_count = stats.memories_by_layer.get(&MemoryLayer::ShortTerm).unwrap_or(&0);

        // Trigger consolidation if memory layers are getting full
        if *working_memory_count > MemoryLayer::Working.typical_capacity() / 2 ||
           *short_term_count > MemoryLayer::ShortTerm.typical_capacity() / 2 {
            let _report = self.consolidate_memories().await?;
            log::info!("Automatic memory consolidation completed");
        }

        Ok(())
    }

    /// Initialize background memory management tasks
    pub async fn start_background_tasks(&self) -> Result<()> {
        // In a real implementation, this would start background tasks for:
        // - Periodic consolidation
        // - Memory aging and migration between layers
        // - Automatic pruning of old, low-importance memories
        // - Insight generation
        // - Performance monitoring

        log::info!("Memory background tasks initialized");
        Ok(())
    }

    /// Shutdown and cleanup memory system
    pub async fn shutdown(&self) -> Result<()> {
        // Perform final consolidation before shutdown
        let _report = self.consolidate_memories().await?;
        
        // In a real implementation, this would:
        // - Stop background tasks
        // - Flush any pending writes
        // - Close database connections
        // - Save memory statistics

        log::info!("Memory system shutdown completed");
        Ok(())
    }
}

/// Helper functions for memory operations

/// Create a memory query builder for easier query construction
pub fn memory_query() -> MemoryQuery {
    MemoryQuery::new()
}

/// Create metadata for user input memories
pub fn user_input_metadata(agent_id: String, conversation_id: Option<String>) -> MemoryMetadata {
    MemoryMetadata {
        source: MemorySource::UserInput,
        agent_id,
        conversation_id,
        session_id: None,
        topics: Vec::new(),
        entities: Vec::new(),
        sentiment: None,
        context_window: None,
        verification_status: VerificationStatus::Unverified,
        custom_fields: std::collections::HashMap::new(),
    }
}

/// Create metadata for agent response memories
pub fn agent_response_metadata(agent_id: String, conversation_id: Option<String>) -> MemoryMetadata {
    MemoryMetadata {
        source: MemorySource::AgentResponse,
        agent_id,
        conversation_id,
        session_id: None,
        topics: Vec::new(),
        entities: Vec::new(),
        sentiment: None,
        context_window: None,
        verification_status: VerificationStatus::Unverified,
        custom_fields: std::collections::HashMap::new(),
    }
}

/// Create metadata for system insights
pub fn system_insight_metadata(agent_id: String) -> MemoryMetadata {
    MemoryMetadata {
        source: MemorySource::SystemInsight,
        agent_id,
        conversation_id: None,
        session_id: None,
        topics: Vec::new(),
        entities: Vec::new(),
        sentiment: None,
        context_window: None,
        verification_status: VerificationStatus::Verified,
        custom_fields: std::collections::HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn test_memory_coordinator_creation() {
        let config = AppConfig::default();
        let coordinator = MemoryCoordinator::new(config, None).await;
        assert!(coordinator.is_ok());
    }

    #[tokio::test]
    async fn test_memory_storage_and_retrieval() {
        let config = AppConfig::default();
        let coordinator = MemoryCoordinator::new(config, None).await.unwrap();
        
        let metadata = user_input_metadata("test-agent".to_string(), None);
        let stored_memory = coordinator.store_memory("Test memory content".to_string(), metadata).await.unwrap();
        
        let query = memory_query().with_agent("test-agent".to_string());
        let retrieved = coordinator.retrieve_memories(query).await.unwrap();
        
        assert!(!retrieved.is_empty());
        assert_eq!(retrieved[0].id, stored_memory.id);
    }

    #[tokio::test]
    async fn test_memory_consolidation() {
        let config = AppConfig::default();
        let coordinator = MemoryCoordinator::new(config, None).await.unwrap();
        
        // Add several memories
        for i in 0..5 {
            let metadata = user_input_metadata("test-agent".to_string(), None);
            coordinator.store_memory(format!("Test memory {}", i), metadata).await.unwrap();
        }
        
        let report = coordinator.consolidate_memories().await.unwrap();
        assert!(report.memories_processed >= 5);
    }

    #[tokio::test]
    async fn test_memory_search_with_scores() {
        let config = AppConfig::default();
        let coordinator = MemoryCoordinator::new(config, None).await.unwrap();
        
        let metadata = user_input_metadata("test-agent".to_string(), None);
        coordinator.store_memory("Important programming information".to_string(), metadata).await.unwrap();
        
        let query = memory_query()
            .with_text("programming".to_string())
            .with_agent("test-agent".to_string());
        
        let results = coordinator.search_with_scores(query).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].relevance_score > 0.0);
    }
}