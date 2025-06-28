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
        
        // Create conversation collection
        let conversation_schema = CollectionSchema::conversation_collection();
        manager.create_collection(&conversation_schema).await?;

        // Register collections
        let mut collections = self.collections.lock().await;
        collections.insert("memories".to_string(), memory_schema.into());
        collections.insert("documents".to_string(), document_schema.into());
        collections.insert("conversations".to_string(), conversation_schema.into());

        log::info!("Vector store initialized with {} collections", collections.len());
        Ok(())
    }

    /// Generate embeddings for text content
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.embedding_engine.generate_embedding(text).await
    }

    /// Store a vector point in a collection
    pub async fn store_vector(
        &self,
        collection_name: &str,
        point: VectorPoint,
    ) -> Result<()> {
        let manager = self.qdrant_manager.lock().await;
        manager.upsert_point(collection_name, point).await
    }

    /// Store multiple vector points
    pub async fn store_vectors(
        &self,
        collection_name: &str,
        points: Vec<VectorPoint>,
    ) -> Result<()> {
        let manager = self.qdrant_manager.lock().await;
        manager.upsert_points(collection_name, points).await
    }

    /// Search for similar vectors
    pub async fn search_similar(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        let search_query = SearchQuery {
            vector: query_vector,
            collection: collection_name.to_string(),
            limit,
            threshold,
            filter: None,
        };

        let manager = self.qdrant_manager.lock().await;
        self.search_engine.search(&*manager, search_query).await
    }

    /// Search for similar content using text query
    pub async fn semantic_search(
        &self,
        collection_name: &str,
        text_query: &str,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        // Generate embedding for the query text
        let query_vector = self.generate_embedding(text_query).await?;
        
        // Search using the embedding
        self.search_similar(collection_name, query_vector, limit, threshold).await
    }

    /// Delete a vector by ID
    pub async fn delete_vector(&self, collection_name: &str, id: Uuid) -> Result<()> {
        let manager = self.qdrant_manager.lock().await;
        manager.delete_point(collection_name, id).await
    }

    /// Delete embedding (alias for delete_vector for compatibility)
    pub async fn delete_embedding(&self, id: Uuid) -> Result<()> {
        // Try to delete from all collections (we don't know which one it's in)
        let collections = self.collections.lock().await;
        let collection_names: Vec<String> = collections.keys().cloned().collect();
        drop(collections);

        for collection_name in collection_names {
            // Ignore errors since the point might not exist in this collection
            let _ = self.delete_vector(&collection_name, id).await;
        }

        Ok(())
    }

    /// Get vector store statistics
    pub async fn get_statistics(&self) -> Result<VectorStoreStats> {
        let manager = self.qdrant_manager.lock().await;
        let collections = self.collections.lock().await;
        
        let mut stats = VectorStoreStats {
            total_collections: collections.len(),
            total_vectors: 0,
            collections_info: std::collections::HashMap::new(),
            embedding_model: self.embedding_engine.model_info().await?,
            qdrant_status: manager.get_status().await?,
        };

        for collection_name in collections.keys() {
            if let Ok(info) = manager.get_collection_info(collection_name).await {
                stats.total_vectors += info.vectors_count;
                stats.collections_info.insert(collection_name.clone(), info);
            }
        }

        Ok(stats)
    }

    /// Check if Qdrant is available and responding
    pub async fn health_check(&self) -> Result<bool> {
        let manager = self.qdrant_manager.lock().await;
        manager.health_check().await
    }

    /// Create a new collection
    pub async fn create_collection(&self, schema: CollectionSchema) -> Result<()> {
        let manager = self.qdrant_manager.lock().await;
        manager.create_collection(&schema).await?;

        // Register the collection
        let mut collections = self.collections.lock().await;
        collections.insert(schema.name.clone(), schema.into());

        Ok(())
    }

    /// Delete a collection
    pub async fn delete_collection(&self, collection_name: &str) -> Result<()> {
        let manager = self.qdrant_manager.lock().await;
        manager.delete_collection(collection_name).await?;

        // Unregister the collection
        let mut collections = self.collections.lock().await;
        collections.remove(collection_name);

        Ok(())
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let collections = self.collections.lock().await;
        Ok(collections.keys().cloned().collect())
    }

    /// Clear all vectors in a collection
    pub async fn clear_collection(&self, collection_name: &str) -> Result<()> {
        let manager = self.qdrant_manager.lock().await;
        manager.clear_collection(collection_name).await
    }

    /// Store memory embedding
    pub async fn store_memory_embedding(
        &self,
        memory_id: Uuid,
        content: &str,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let embedding = self.generate_embedding(content).await?;
        
        let mut payload = metadata;
        payload.insert("content".to_string(), serde_json::Value::String(content.to_string()));
        payload.insert("type".to_string(), serde_json::Value::String("memory".to_string()));

        let point = VectorPoint {
            id: memory_id,
            vector: embedding,
            payload,
        };

        self.store_vector("memories", point).await
    }

    /// Store document embedding
    pub async fn store_document_embedding(
        &self,
        document_id: Uuid,
        content: &str,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let embedding = self.generate_embedding(content).await?;
        
        let mut payload = metadata;
        payload.insert("content".to_string(), serde_json::Value::String(content.to_string()));
        payload.insert("type".to_string(), serde_json::Value::String("document".to_string()));

        let point = VectorPoint {
            id: document_id,
            vector: embedding,
            payload,
        };

        self.store_vector("documents", point).await
    }

    /// Search memories by semantic similarity
    pub async fn search_memories(
        &self,
        query: &str,
        agent_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        let mut results = self.semantic_search("memories", query, limit * 2, None).await?;

        // Filter by agent if specified
        if let Some(agent_id) = agent_id {
            results = results.into_iter()
                .filter(|result| {
                    result.payload.get("agent_id")
                        .and_then(|v| v.as_str())
                        .map(|id| id == agent_id)
                        .unwrap_or(false)
                })
                .take(limit)
                .collect();
        }

        Ok(results)
    }

    /// Search documents by semantic similarity
    pub async fn search_documents(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        self.semantic_search("documents", query, limit, None).await
    }

    /// Get embedding dimension
    pub async fn embedding_dimension(&self) -> Result<usize> {
        self.embedding_engine.dimension().await
    }

    /// Batch process embeddings for multiple texts
    pub async fn batch_generate_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        self.embedding_engine.batch_generate_embeddings(texts).await
    }

    /// Shutdown and cleanup vector store
    pub async fn shutdown(&self) -> Result<()> {
        // Close Qdrant connections
        let manager = self.qdrant_manager.lock().await;
        manager.shutdown().await?;

        log::info!("Vector store shutdown completed");
        Ok(())
    }
}

/// Vector store statistics
#[derive(Debug, Clone)]
pub struct VectorStoreStats {
    pub total_collections: usize,
    pub total_vectors: usize,
    pub collections_info: std::collections::HashMap<String, CollectionInfo>,
    pub embedding_model: String,
    pub qdrant_status: QdrantStatus,
}

#[derive(Debug, Clone)]
pub struct CollectionInfo {
    pub name: String,
    pub vectors_count: usize,
    pub indexed_vectors_count: usize,
    pub points_count: usize,
    pub segments_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn test_vector_store_creation() {
        let config = AppConfig::default();
        // Note: This test might fail if Qdrant is not available
        let result = VectorStore::new(config).await;
        
        // Don't assert success since Qdrant might not be running in test environment
        println!("Vector store creation result: {:?}", result.is_ok());
    }

    #[tokio::test]
    async fn test_embedding_generation() {
        let config = AppConfig::default();
        
        if let Ok(vector_store) = VectorStore::new(config).await {
            let embedding_result = vector_store.generate_embedding("test text").await;
            
            // Test might fail if embedding model is not available
            if let Ok(embedding) = embedding_result {
                assert!(!embedding.is_empty());
                println!("Generated embedding with dimension: {}", embedding.len());
            }
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AppConfig::default();
        
        if let Ok(vector_store) = VectorStore::new(config).await {
            let health_result = vector_store.health_check().await;
            println!("Health check result: {:?}", health_result);
        }
    }
}