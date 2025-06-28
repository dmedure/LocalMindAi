use crate::utils::error::{LocalMindError, Result};
use serde::{Deserialize, Serialize};

/// Ollama service configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

/// Ollama model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: Option<OllamaModelDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelDetails {
    pub format: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: String,
    pub quantization_level: String,
}

/// Ollama generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub options: Option<OllamaOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub num_predict: Option<i32>,
    pub stop: Option<Vec<String>>,
    pub repeat_penalty: Option<f32>,
}

/// Ollama generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub context: Option<Vec<u32>>,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u64>,
}

/// Ollama service client
pub struct OllamaClient {
    config: OllamaConfig,
    client: reqwest::Client,
}

impl OllamaClient {
    /// Create a new Ollama client with default configuration
    pub fn new() -> Self {
        Self::with_config(OllamaConfig::default())
    }

    /// Create a new Ollama client with custom configuration
    pub fn with_config(config: OllamaConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(tokio::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// Check if Ollama service is available
    pub async fn is_available(&self) -> bool {
        match self.client.get(&format!("{}/api/tags", self.config.base_url)).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Get list of available models
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.config.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to connect to Ollama: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse response: {}", e)))?;

        let models: Vec<OllamaModel> = response_json["models"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|model| serde_json::from_value(model.clone()).ok())
            .collect();

        Ok(models)
    }

    /// Generate a response using the specified model
    pub async fn generate(&self, request: OllamaRequest) -> Result<OllamaResponse> {
        let url = format!("{}/api/generate", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Ollama generation failed: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse response: {}", e)))?;

        Ok(ollama_response)
    }

    /// Check if a specific model is available
    pub async fn has_model(&self, model_name: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|model| model.name == model_name))
    }

    /// Pull a model from Ollama registry
    pub async fn pull_model(&self, model_name: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.config.base_url);
        let request = serde_json::json!({
            "name": model_name
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to pull model: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to pull model '{}': {}",
                model_name,
                response.status()
            )));
        }

        Ok(())
    }

    /// Delete a model from local storage
    pub async fn delete_model(&self, model_name: &str) -> Result<()> {
        let url = format!("{}/api/delete", self.config.base_url);
        let request = serde_json::json!({
            "name": model_name
        });

        let response = self.client
            .delete(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to delete model: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to delete model '{}': {}",
                model_name,
                response.status()
            )));
        }

        Ok(())
    }

    /// Get model information
    pub async fn show_model(&self, model_name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/show", self.config.base_url);
        let request = serde_json::json!({
            "name": model_name
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to get model info: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to get info for model '{}': {}",
                model_name,
                response.status()
            )));
        }

        let model_info: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse model info: {}", e)))?;

        Ok(model_info)
    }

    /// Get Ollama version information
    pub async fn version(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/version", self.config.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to get version: {}", e)))?;

        if !response.status().is_success() {
            return Err(LocalMindError::ExternalService(format!(
                "Failed to get Ollama version: {}",
                response.status()
            )));
        }

        let version_info: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LocalMindError::Network(format!("Failed to parse version info: {}", e)))?;

        Ok(version_info)
    }
}

/// Check Ollama service status (simplified function for compatibility)
pub async fn check_ollama_status() -> bool {
    let client = OllamaClient::new();
    client.is_available().await
}

/// Get available models (simplified function)
pub async fn get_ollama_models() -> Result<Vec<String>> {
    let client = OllamaClient::new();
    let models = client.list_models().await?;
    Ok(models.into_iter().map(|model| model.name).collect())
}

/// Ensure required models are available
pub async fn ensure_models(required_models: &[&str]) -> Result<Vec<String>> {
    let client = OllamaClient::new();
    let mut missing_models = Vec::new();

    for model_name in required_models {
        if !client.has_model(model_name).await? {
            missing_models.push(model_name.to_string());
        }
    }

    Ok(missing_models)
}

/// Helper function to create standard generation options
pub fn create_generation_options(
    temperature: Option<f32>,
    max_tokens: Option<i32>,
) -> OllamaOptions {
    OllamaOptions {
        temperature,
        top_p: Some(0.9),
        top_k: Some(40),
        num_predict: max_tokens,
        stop: None,
        repeat_penalty: Some(1.1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_client_creation() {
        let client = OllamaClient::new();
        assert_eq!(client.config.base_url, "http://localhost:11434");
        assert_eq!(client.config.timeout_seconds, 30);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = OllamaConfig {
            base_url: "http://custom:11434".to_string(),
            timeout_seconds: 60,
            max_retries: 5,
        };
        let client = OllamaClient::with_config(config);
        assert_eq!(client.config.base_url, "http://custom:11434");
        assert_eq!(client.config.timeout_seconds, 60);
    }

    #[tokio::test]
    async fn test_create_generation_options() {
        let options = create_generation_options(Some(0.7), Some(100));
        assert_eq!(options.temperature, Some(0.7));
        assert_eq!(options.num_predict, Some(100));
        assert_eq!(options.top_p, Some(0.9));
    }

    #[tokio::test]
    async fn test_service_availability_check() {
        // This test will pass/fail based on whether Ollama is running
        let status = check_ollama_status().await;
        println!("Ollama status: {}", status);
        // We don't assert since it depends on the environment
    }
}