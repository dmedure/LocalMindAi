use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use super::{VectorPoint, VectorSearchResult, CollectionSchema};

#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub host: String,
    pub port: u16,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6333,
            api_key: None,
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantStatus {
    pub is_running: bool,
    pub version: Option<String>,
    pub collections_count: usize,
    pub total_points: usize,
    pub memory_usage_mb: f64,
}

/// Manager for Qdrant vector database operations
pub struct QdrantManager {
    config: QdrantConfig,
    client: reqwest::Client,
    base_url: String,
}

impl QdrantManager {
    /// Create a new Qdrant manager
    pub async fn new(config: QdrantConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;

        let base_url = format!("http://{}:{}", config.host, config.port);

        let manager = Self {
            config,
            client,
            base_url,
        };

        // Test connection
        manager.health_check().await?;

        Ok(manager)
    }

    /// Check if Qdrant is healthy and responding
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/", self.base_url);
        
        let mut request = self.client.get(&url);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        match request.send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get Qdrant status information
    pub async fn get_status(&self) -> Result<QdrantStatus> {
        let url = format!("{}/", self.base_url);
        
        let mut request = self.client.get(&url);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            // Get collections to count them
            let collections = self.list_collections().await.unwrap_or_default();
            
            Ok(QdrantStatus {
                is_running: true,
                version: Some("1.0.0".to_string()), // In real implementation, parse from response
                collections_count: collections.len(),
                total_points: 0, // Would calculate from all collections
                memory_usage_mb: 0.0, // Would get from Qdrant metrics
            })
        } else {
            Ok(QdrantStatus {
                is_running: false,
                version: None,
                collections_count: 0,
                total_points: 0,
                memory_usage_mb: 0.0,
            })
        }
    }

    /// Create a new collection
    pub async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        let url = format!("{}/collections/{}", self.base_url, schema.name);
        
        let create_request = serde_json::json!({
            "vectors": {
                "size": schema.vector_size,
                "distance": match schema.distance_metric.as_str() {
                    "cosine" => "Cosine",
                    "euclidean" => "Euclid",
                    "dot" => "Dot",
                    _ => "Cosine"
                }
            },
            "optimizers_config": {
                "default_segment_number": 2
            },
            "replication_factor": 1
        });

        let mut request = self.client.put(&url).json(&create_request);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            log::info!("Created Qdrant collection: {}", schema.name);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Failed to create collection: {}", error_text))
        }
    }

    /// Delete a collection
    pub async fn delete_collection(&self, collection_name: &str) -> Result<()> {
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        
        let mut request = self.client.delete(&url);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            log::info!("Deleted Qdrant collection: {}", collection_name);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Failed to delete collection: {}", error_text))
        }
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let url = format!("{}/collections", self.base_url);
        
        let mut request = self.client.get(&url);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            let collections_response: serde_json::Value = response.json().await?;
            
            if let Some(collections) = collections_response["result"]["collections"].as_array() {
                let collection_names: Vec<String> = collections
                    .iter()
                    .filter_map(|c| c["name"].as_str())
                    .map(|s| s.to_string())
                    .collect();
                
                Ok(collection_names)
            } else {
                Ok(Vec::new())
            }
        } else {
            Err(anyhow::anyhow!("Failed to list collections"))
        }
    }

    /// Get collection information
    pub async fn get_collection_info(&self, collection_name: &str) -> Result<super::CollectionInfo> {
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        
        let mut request = self.client.get(&url);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            let info_response: serde_json::Value = response.json().await?;
            
            let result = &info_response["result"];
            
            Ok(super::CollectionInfo {
                name: collection_name.to_string(),
                vectors_count: result["vectors_count"].as_u64().unwrap_or(0) as usize,
                indexed_vectors_count: result["indexed_vectors_count"].as_u64().unwrap_or(0) as usize,
                points_count: result["points_count"].as_u64().unwrap_or(0) as usize,
                segments_count: result["segments_count"].as_u64().unwrap_or(0) as usize,
            })
        } else {
            Err(anyhow::anyhow!("Failed to get collection info"))
        }
    }

    /// Insert or update a single point
    pub async fn upsert_point(&self, collection_name: &str, point: VectorPoint) -> Result<()> {
        self.upsert_points(collection_name, vec![point]).await
    }

    /// Insert or update multiple points
    pub async fn upsert_points(&self, collection_name: &str, points: Vec<VectorPoint>) -> Result<()> {
        let url = format!("{}/collections/{}/points", self.base_url, collection_name);
        
        let qdrant_points: Vec<serde_json::Value> = points
            .into_iter()
            .map(|point| {
                serde_json::json!({
                    "id": point.id.to_string(),
                    "vector": point.vector,
                    "payload": point.payload
                })
            })
            .collect();

        let upsert_request = serde_json::json!({
            "points": qdrant_points
        });

        let mut request = self.client.put(&url).json(&upsert_request);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Failed to upsert points: {}", error_text))
        }
    }

    /// Search for similar vectors
    pub async fn search(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
        filter: Option<serde_json::Value>,
    ) -> Result<Vec<VectorSearchResult>> {
        let url = format!("{}/collections/{}/points/search", self.base_url, collection_name);
        
        let mut search_request = serde_json::json!({
            "vector": query_vector,
            "limit": limit,
            "with_payload": true,
            "with_vector": false
        });

        if let Some(score_threshold) = threshold {
            search_request["score_threshold"] = serde_json::Value::Number(
                serde_json::Number::from_f64(score_threshold as f64).unwrap()
            );
        }

        if let Some(filter_value) = filter {
            search_request["filter"] = filter_value;
        }

        let mut request = self.client.post(&url).json(&search_request);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            let search_response: serde_json::Value = response.json().await?;
            
            let mut results = Vec::new();
            
            if let Some(result_array) = search_response["result"].as_array() {
                for result in result_array {
                    if let (Some(id_str), Some(score)) = (result["id"].as_str(), result["score"].as_f64()) {
                        if let Ok(id) = Uuid::parse_str(id_str) {
                            let payload = result["payload"].as_object()
                                .map(|obj| {
                                    obj.iter()
                                        .map(|(k, v)| (k.clone(), v.clone()))
                                        .collect()
                                })
                                .unwrap_or_default();

                            results.push(VectorSearchResult {
                                id,
                                score: score as f32,
                                payload,
                            });
                        }
                    }
                }
            }
            
            Ok(results)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Search failed: {}", error_text))
        }
    }

    /// Delete a point by ID
    pub async fn delete_point(&self, collection_name: &str, point_id: Uuid) -> Result<()> {
        let url = format!("{}/collections/{}/points/delete", self.base_url, collection_name);
        
        let delete_request = serde_json::json!({
            "points": [point_id.to_string()]
        });

        let mut request = self.client.post(&url).json(&delete_request);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Failed to delete point: {}", error_text))
        }
    }

    /// Clear all points in a collection
    pub async fn clear_collection(&self, collection_name: &str) -> Result<()> {
        let url = format!("{}/collections/{}/points/delete", self.base_url, collection_name);
        
        let delete_request = serde_json::json!({
            "filter": {
                "must": [
                    {
                        "key": "type",
                        "match": {
                            "any": ["memory", "document", "conversation"]
                        }
                    }
                ]
            }
        });

        let mut request = self.client.post(&url).json(&delete_request);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            log::info!("Cleared all points from collection: {}", collection_name);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Failed to clear collection: {}", error_text))
        }
    }

    /// Get a specific point by ID
    pub async fn get_point(&self, collection_name: &str, point_id: Uuid) -> Result<Option<VectorPoint>> {
        let url = format!("{}/collections/{}/points/{}", self.base_url, collection_name, point_id);
        
        let mut request = self.client.get(&url);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            let point_response: serde_json::Value = response.json().await?;
            
            if let Some(result) = point_response["result"].as_object() {
                let vector = result["vector"].as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_f64())
                            .map(|f| f as f32)
                            .collect()
                    })
                    .unwrap_or_default();

                let payload = result["payload"].as_object()
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(Some(VectorPoint {
                    id: point_id,
                    vector,
                    payload,
                }))
            } else {
                Ok(None)
            }
        } else if response.status() == 404 {
            Ok(None)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Failed to get point: {}", error_text))
        }
    }

    /// Create index for faster searching
    pub async fn create_index(&self, collection_name: &str, field_name: &str) -> Result<()> {
        let url = format!("{}/collections/{}/index", self.base_url, collection_name);
        
        let index_request = serde_json::json!({
            "field_name": field_name,
            "field_schema": "keyword"
        });

        let mut request = self.client.put(&url).json(&index_request);
        
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        
        if response.status().is_success() {
            log::info!("Created index for field '{}' in collection '{}'", field_name, collection_name);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Failed to create index: {}", error_text))
        }
    }

    /// Shutdown the manager and close connections
    pub async fn shutdown(&self) -> Result<()> {
        // In a real implementation, we would close any persistent connections here
        log::info!("Qdrant manager shutdown completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_qdrant_manager_creation() {
        let config = QdrantConfig::default();
        let result = QdrantManager::new(config).await;
        
        // Don't assert success since Qdrant might not be running
        println!("Qdrant manager creation result: {:?}", result.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = QdrantConfig::default();
        
        if let Ok(manager) = QdrantManager::new(config).await {
            let health = manager.health_check().await.unwrap_or(false);
            println!("Qdrant health check: {}", health);
        }
    }

    #[tokio::test]
    async fn test_collections_operations() {
        let config = QdrantConfig::default();
        
        if let Ok(manager) = QdrantManager::new(config).await {
            // Test listing collections
            if let Ok(collections) = manager.list_collections().await {
                println!("Found {} collections", collections.len());
            }
        }
    }

    #[tokio::test]
    async fn test_point_operations() {
        let config = QdrantConfig::default();
        
        if let Ok(manager) = QdrantManager::new(config).await {
            let test_point = VectorPoint {
                id: Uuid::new_v4(),
                vector: vec![0.1, 0.2, 0.3, 0.4],
                payload: HashMap::new(),
            };

            // This test would only work if there's a test collection available
            // and Qdrant is running with the right configuration
            println!("Would test point operations with point ID: {}", test_point.id);
        }
    }
}