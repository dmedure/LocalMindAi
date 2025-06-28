use crate::utils::error::{LocalMindError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ChromaDB service configuration
#[derive(Debug, Clone)]
pub struct ChromaConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl Default for ChromaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8000".to_string(),
            api_key: None,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

/// ChromaDB collection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaCollection {
    pub name: String,
    pub id: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// ChromaDB document for insertion/query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaDocument {
    pub id: String,
    pub content: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub embedding: Option<Vec<f32>>,
}

/// ChromaDB query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaQuery {
    pub query_texts: Option<Vec<String>>,
    pub query_embeddings: Option<Vec<Vec<f32>>>,
    pub n_results: Option<usize>,
    pub where_clause: Option<HashMap<String, serde_json::Value>>,
    pub include: Option<Vec<String>>, // ["metadatas", "documents", "distances"]
}

/// ChromaDB query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaQueryResult {
    pub ids: Vec<Vec<String>>,
    pub distances: Option<Vec<Vec<f32>>>,
    pub metadatas: Option<Vec<Vec<Option<HashMap<String, serde_json::Value>>>>>,
    pub documents: Option<Vec<Vec<Option<String>>>>,
    pub embeddings: Option<Vec<Vec<Option<Vec<f32>>>>>,
}

/// ChromaDB client
pub struct ChromaClient {
    config: ChromaConfig,
    client: reqwest::Client,
}

impl ChromaClient {
    /// Create a new ChromaDB client with default configuration
    pub fn new() -> Self {
        Self::with_config(ChromaConfig::default())
    }

    /// Create a new ChromaDB client with custom configuration
    pub fn with_config(config: ChromaConfig) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        if let Some(api_key) = &config.api_key {
            headers.insert("Authorization", format!("Bearer {}", api_key).parse().unwrap());
        }

        let client = reqwest::Client::builder()
            .timeout(tokio::time::Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// Check if ChromaDB service is available
    pub async fn is_available(&self) -> bool {
        match self.client.get(&format!("{}/api/v1/heartbeat", self.config.base_url)).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Get ChromaDB version
    pub async fn version(&self) -> Result<String> {
        let url = format!("{}/api/v1/version", self.config.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to connect to ChromaDB: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "ChromaDB API error: {}",
                response.status()
            )));
        }

        let version: String = response
            .text()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse version: {}", e)))?;

        Ok(version)
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<ChromaCollection>> {
        let url = format!("{}/api/v1/collections", self.config.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to list collections: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to list collections: {}",
                response.status()
            )));
        }

        let collections: Vec<ChromaCollection> = response
            .json()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse collections: {}", e)))?;

        Ok(collections)
    }

    /// Create a new collection
    pub async fn create_collection(
        &self,
        name: &str,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<ChromaCollection> {
        let url = format!("{}/api/v1/collections", self.config.base_url);
        let request = serde_json::json!({
            "name": name,
            "metadata": metadata
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to create collection: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to create collection '{}': {}",
                name,
                response.status()
            )));
        }

        let collection: ChromaCollection = response
            .json()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse collection: {}", e)))?;

        Ok(collection)
    }

    /// Get a collection by name
    pub async fn get_collection(&self, name: &str) -> Result<ChromaCollection> {
        let url = format!("{}/api/v1/collections/{}", self.config.base_url, name);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to get collection: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to get collection '{}': {}",
                name,
                response.status()
            )));
        }

        let collection: ChromaCollection = response
            .json()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse collection: {}", e)))?;

        Ok(collection)
    }

    /// Delete a collection
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        let url = format!("{}/api/v1/collections/{}", self.config.base_url, name);
        let response = self.client
            .delete(&url)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to delete collection: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to delete collection '{}': {}",
                name,
                response.status()
            )));
        }

        Ok(())
    }

    /// Add documents to a collection
    pub async fn add_documents(
        &self,
        collection_name: &str,
        documents: &[ChromaDocument],
    ) -> Result<()> {
        let url = format!(
            "{}/api/v1/collections/{}/add",
            self.config.base_url, collection_name
        );

        let ids: Vec<&str> = documents.iter().map(|doc| doc.id.as_str()).collect();
        let documents_content: Vec<&str> = documents.iter().map(|doc| doc.content.as_str()).collect();
        let metadatas: Vec<Option<&HashMap<String, serde_json::Value>>> =
            documents.iter().map(|doc| doc.metadata.as_ref()).collect();
        let embeddings: Vec<Option<&Vec<f32>>> =
            documents.iter().map(|doc| doc.embedding.as_ref()).collect();

        let request = serde_json::json!({
            "ids": ids,
            "documents": documents_content,
            "metadatas": metadatas,
            "embeddings": embeddings
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to add documents: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to add documents to '{}': {}",
                collection_name,
                response.status()
            )));
        }

        Ok(())
    }

    /// Query a collection
    pub async fn query_collection(
        &self,
        collection_name: &str,
        query: &ChromaQuery,
    ) -> Result<ChromaQueryResult> {
        let url = format!(
            "{}/api/v1/collections/{}/query",
            self.config.base_url, collection_name
        );

        let response = self.client
            .post(&url)
            .json(query)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to query collection: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to query collection '{}': {}",
                collection_name,
                response.status()
            )));
        }

        let result: ChromaQueryResult = response
            .json()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse query result: {}", e)))?;

        Ok(result)
    }

    /// Update documents in a collection
    pub async fn update_documents(
        &self,
        collection_name: &str,
        documents: &[ChromaDocument],
    ) -> Result<()> {
        let url = format!(
            "{}/api/v1/collections/{}/update",
            self.config.base_url, collection_name
        );

        let ids: Vec<&str> = documents.iter().map(|doc| doc.id.as_str()).collect();
        let documents_content: Vec<&str> = documents.iter().map(|doc| doc.content.as_str()).collect();
        let metadatas: Vec<Option<&HashMap<String, serde_json::Value>>> =
            documents.iter().map(|doc| doc.metadata.as_ref()).collect();
        let embeddings: Vec<Option<&Vec<f32>>> =
            documents.iter().map(|doc| doc.embedding.as_ref()).collect();

        let request = serde_json::json!({
            "ids": ids,
            "documents": documents_content,
            "metadatas": metadatas,
            "embeddings": embeddings
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to update documents: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to update documents in '{}': {}",
                collection_name,
                response.status()
            )));
        }

        Ok(())
    }

    /// Delete documents from a collection
    pub async fn delete_documents(
        &self,
        collection_name: &str,
        document_ids: &[String],
    ) -> Result<()> {
        let url = format!(
            "{}/api/v1/collections/{}/delete",
            self.config.base_url, collection_name
        );

        let request = serde_json::json!({
            "ids": document_ids
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to delete documents: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to delete documents from '{}': {}",
                collection_name,
                response.status()
            )));
        }

        Ok(())
    }

    /// Get collection count
    pub async fn get_collection_count(&self, collection_name: &str) -> Result<usize> {
        let url = format!(
            "{}/api/v1/collections/{}/count",
            self.config.base_url, collection_name
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to get collection count: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to get count for collection '{}': {}",
                collection_name,
                response.status()
            )));
        }

        let count: usize = response
            .json()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse count: {}", e)))?;

        Ok(count)
    }
}

/// Check ChromaDB service status (simplified function for compatibility)
pub async fn check_chromadb_status() -> bool {
    let client = ChromaClient::new();
    client.is_available().await
}

/// Get ChromaDB version (simplified function)
pub async fn get_chromadb_version() -> Result<String> {
    let client = ChromaClient::new();
    client.version().await
}

/// Ensure required collections exist
pub async fn ensure_collections(collection_names: &[&str]) -> Result<Vec<String>> {
    let client = ChromaClient::new();
    let existing_collections = client.list_collections().await?;
    let existing_names: Vec<String> = existing_collections.into_iter().map(|c| c.name).collect();

    let mut missing_collections = Vec::new();
    for name in collection_names {
        if !existing_names.contains(&name.to_string()) {
            missing_collections.push(name.to_string());
        }
    }

    Ok(missing_collections)
}

/// Helper function to create a simple text query
pub fn create_text_query(
    query_text: &str,
    n_results: Option<usize>,
    include_distances: bool,
) -> ChromaQuery {
    let include = if include_distances {
        Some(vec!["metadatas".to_string(), "documents".to_string(), "distances".to_string()])
    } else {
        Some(vec!["metadatas".to_string(), "documents".to_string()])
    };

    ChromaQuery {
        query_texts: Some(vec![query_text.to_string()]),
        query_embeddings: None,
        n_results,
        where_clause: None,
        include,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chroma_client_creation() {
        let client = ChromaClient::new();
        assert_eq!(client.config.base_url, "http://localhost:8000");
        assert_eq!(client.config.timeout_seconds, 30);
        assert!(client.config.api_key.is_none());
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = ChromaConfig {
            base_url: "http://custom:8000".to_string(),
            api_key: Some("test-key".to_string()),
            timeout_seconds: 60,
            max_retries: 5,
        };
        let client = ChromaClient::with_config(config);
        assert_eq!(client.config.base_url, "http://custom:8000");
        assert_eq!(client.config.api_key, Some("test-key".to_string()));
    }

    #[tokio::test]
    async fn test_create_text_query() {
        let query = create_text_query("test query", Some(10), true);
        assert_eq!(query.query_texts, Some(vec!["test query".to_string()]));
        assert_eq!(query.n_results, Some(10));
        assert!(query.include.is_some());
    }

    #[tokio::test]
    async fn test_service_availability_check() {
        // This test will pass/fail based on whether ChromaDB is running
        let status = check_chromadb_status().await;
        println!("ChromaDB status: {}", status);
        // We don't assert since it depends on the environment
    }
}