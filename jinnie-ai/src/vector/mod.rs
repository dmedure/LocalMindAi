//! Vector database module for semantic search using Qdrant
//! 
//! This module provides vector storage and semantic search capabilities
//! for LocalMind's memory system. It integrates with local Qdrant instances
//! and manages embeddings for memories, documents, and other content.

pub mod qdrant_manager;
pub mod embedding_engine;
pub mod collection_schema;
pub mod search_engine;

// Re-export commonly used types and structs
pub use qdrant_manager::{QdrantManager, QdrantConfig, QdrantStatus};
pub use embedding_engine::{EmbeddingEngine, EmbeddingModel, EmbeddingResult};
pub use collection_schema::{CollectionSchema, VectorCollection, FieldType};
pub use search_engine::{SemanticSearchEngine, SearchQuery, SearchResult, SimilarityMetric};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::state::AppState;

/// Main vector store coordinator that manages all vector operations
pub struct VectorStore {
    qdrant_manager: Arc<Mutex<QdrantManager>>,
    embedding_engine: Arc<EmbeddingEngine>,
    search_engine: Arc<SemanticSearchEngine>,
    collections: Arc<Mutex<std::collections::HashMap<String, VectorCollection>>>,
}

#[derive(Debug, Clone)]
pub struct VectorPoint {
    pub id: Uuid,
    pub vector: Vec<f32>,
    pub payload: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub id: Uuid,
    pub score: f32,
    pub payload: std::collections::HashMap<String, serde_json::Value>,
}

impl VectorStore {
    /// Create a new vector store
    pub async fn new(config: AppConfig) -> Result<Self> {
        let qdrant_config = QdrantConfig {
            host: config.vector.qdrant_host.clone(),
            port: config.vector.qdrant_port,
            api_key: config.vector.qdrant_api_key.clone(),
            timeout_secs: 30,
        };

        let qdrant_manager = Arc::new(Mutex::new(
            QdrantManager::new(qdrant_config).await?
        ));

        let embedding_engine = Arc::new(
            EmbeddingEngine::new(&config.vector.embedding_model).await?
        );

        let search_engine = Arc::new(SemanticSearchEngine::new());

        let collections = Arc::new(Mutex::new(std::collections::HashMap::new()));

        Ok(Self {
            qdrant_manager,
            embedding_engine,
            search_engine,
            collections,
        })
    }

    /// Initialize the vector store with required collections
    pub async fn initialize(&self) -> Result<()> {
        let manager = self.qdrant_manager.lock().await;
        
        // Create memory collection
        let memory_schema = CollectionSchema::memory_collection();
        manager.create_collection(&memory_schema).await?;
        
        // Create document collection
        let document_schema = CollectionSchema::document_collection();
        manager.create_collection(&document_schema).await?;
        
        log::info!("Vector store collections initialized");
        Ok(())
    }
    
    /// Check if the vector store is available
    pub async fn is_available(&self) -> bool {
        let manager = self.qdrant_manager.lock().await;
        manager.check_health().await.is_ok()
    }
}

/// Initialize the vector store
pub async fn initialize_vector_store(state: &AppState) -> Result<()> {
    log::info!("Initializing vector store...");
    
    if state.vector_store.is_some() {
        log::info!("Vector store already initialized");
        return Ok(());
    }
    
    // Check if Qdrant is available
    let qdrant_available = if let Some(vector_store) = &state.vector_store {
        vector_store.is_available().await
    } else {
        false
    };
    
    // Update service status
    state.update_service_status(None, None, Some(qdrant_available)).await;
    
    if qdrant_available {
        log::info!("Qdrant vector database is available");
    } else {
        log::warn!("Qdrant not available - vector search features will be disabled");
        log::info!("To enable vector search, please install and run Qdrant from https://qdrant.tech");
    }
    
    Ok(())
}