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
use crate::state::AppState;

/// Main memory coordinator that manages the entire memory system
pub struct MemoryCoordinator {
    memory_manager: Arc<Mutex<MemoryManager>>,
    importance_scorer: Arc<ImportanceScorer>,
    consolidation_engine: Arc<ConsolidationEngine>,
    retrieval_engine: Arc<MemoryRetrieval>,
}

impl MemoryCoordinator {
    /// Create a new memory coordinator
    pub async fn new(config: AppConfig, vector_store: Option<Arc<VectorStore>>) -> Result<Self> {
        let memory_manager = Arc::new(Mutex::new(
            MemoryManager::new(config.clone(), vector_store.clone()).await?
        ));

        let importance_scorer = Arc::new(ImportanceScorer::new());
        let consolidation_engine = Arc::new(ConsolidationEngine::new());
        let retrieval_engine = Arc::new(MemoryRetrieval::new(vector_store));

        Ok(Self {
            memory_manager,
            importance_scorer,
            consolidation_engine,
            retrieval_engine,
        })
    }
    
    /// Search memories
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        self.retrieval_engine.search(query, limit).await
    }
    
    /// Add a new memory
    pub async fn add_memory(&self, memory: Memory) -> Result<()> {
        let mut manager = self.memory_manager.lock().await;
        manager.add_memory(memory).await
    }
    
    /// Get memory statistics
    pub async fn get_stats(&self) -> Result<MemoryStats> {
        let manager = self.memory_manager.lock().await;
        Ok(manager.get_stats())
    }
}

/// Initialize the memory system
pub async fn initialize_memory_system(state: &AppState) -> Result<()> {
    log::info!("Initializing memory system...");
    
    if state.memory_system.is_some() {
        log::info!("Memory system already initialized");
        return Ok(());
    }
    
    // Memory system initialization is handled in state initialization
    // This function is here for compatibility and future extensions
    
    if state.memory_system.is_some() {
        log::info!("Memory system initialized successfully");
    } else {
        log::warn!("Memory system not initialized - may be missing required features");
    }
    
    Ok(())
}