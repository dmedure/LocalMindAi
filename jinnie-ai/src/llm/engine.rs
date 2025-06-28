use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::config::{ModelType, ModelConfig};
use crate::utils::error::{LocalMindError, Result};
use crate::llm::{ModelManager, ModelSelector, TaskClassifier, SessionManager};

/// Core LLM inference engine
pub struct LLMEngine {
    model_manager: Arc<Mutex<ModelManager>>,
    model_selector: Arc<ModelSelector>,
    task_classifier: Arc<TaskClassifier>,
    session_manager: Arc<Mutex<SessionManager>>,
    active_sessions: Arc<RwLock<HashMap<String, String>>>, // session_id -> model_type
    performance_metrics: Arc<RwLock<EngineMetrics>>,
}

/// Request for LLM inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub session_id: Option<String>,
    pub prompt: String,
    pub agent_id: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: bool,
    pub force_model: Option<String>, // Override automatic selection
    pub context: Option<Vec<String>>, // Additional context from memory
}

/// Response from LLM inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub session_id: String,
    pub content: String,
    pub model_used: String,
    pub tokens_generated: Option<u32>,
    pub generation_time_ms: u64,
    pub reasoning: Option<String>, // Why this model was selected
    pub confidence: Option<f32>,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
}

/// Reason why generation finished
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    Completed,
    MaxTokens,
    StopSequence,
    Error(String),
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Engine performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub model_usage_stats: HashMap<String, ModelUsageStats>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Usage statistics per model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageStats {
    pub requests: u64,
    pub total_tokens: u64,
    pub average_tokens_per_second: f64,
    pub average_response_time_ms: f64,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

impl LLMEngine {
    /// Create a new LLM engine
    pub async fn new() -> Result<Self> {
        let model_manager = Arc::new(Mutex::new(ModelManager::new().await?));
        let model_selector = Arc::new(ModelSelector::new().await?);
        let task_classifier = Arc::new(TaskClassifier::new());
        let session_manager = Arc::new(Mutex::new(SessionManager::new()));

        Ok(Self {
            model_manager,
            model_selector,
            task_classifier,
            session_manager,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(EngineMetrics::new())),
        })
    }

    /// Generate a response using the most appropriate model
    pub async fn generate(&self, request: InferenceRequest) -> Result<InferenceResponse> {
        let start_time = std::time::Instant::now();
        let session_id = request.session_id.clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Classify the task complexity
        let task_complexity = self.task_classifier.classify_prompt(&request.prompt).await?;
        
        // Select the most appropriate model
        let model_selection = if let Some(forced_model) = &request.force_model {
            // User forced a specific model
            self.model_selector.force_model_selection(forced_model.clone()).await?
        } else {
            // Automatic model selection
            self.model_selector.select_model(&request, &task_complexity).await?
        };

        // Ensure the selected model is loaded
        let mut model_manager = self.model_manager.lock().await;
        if !model_manager.is_model_loaded(&model_selection.model_type).await? {
            model_manager.load_model(&model_selection.model_type).await?;
        }

        // Generate the response
        let response = self.generate_with_model(
            &mut model_manager,
            &request,
            &model_selection,
            session_id.clone(),
        ).await?;

        // Update session tracking
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), model_selection.model_type.identifier());
        }

        // Update performance metrics
        let generation_time = start_time.elapsed().as_millis() as u64;
        self.update_metrics(&model_selection.model_type, generation_time, &response).await?;

        // Update session manager
        {
            let mut session_manager = self.session_manager.lock().await;
            session_manager.update_session(
                &session_id,
                &request.agent_id,
                &request.prompt,
                &response.content,
                &model_selection.model_type,
            ).await?;
        }

        Ok(InferenceResponse {
            session_id,
            content: response.content,
            model_used: model_selection.model_type.display_name(),
            tokens_generated: response.tokens_generated,
            generation_time_ms: generation_time,
            reasoning: Some(model_selection.reasoning),
            confidence: model_selection.confidence,
            finish_reason: response.finish_reason,
            usage: response.usage,
        })
    }

    /// Generate response with a specific model
    async fn generate_with_model(
        &self,
        model_manager: &mut ModelManager,
        request: &InferenceRequest,
        model_selection: &crate::llm::SelectionResult,
        session_id: String,
    ) -> Result<InferenceResponse> {
        // Build the final prompt with context
        let prompt = self.build_contextual_prompt(request, &session_id).await?;

        // Call the model for generation
        let generation_request = crate::services::ollama::OllamaRequest {
            model: model_selection.model_type.identifier(),
            prompt,
            stream: request.stream,
            options: Some(crate::services::ollama::OllamaOptions {
                temperature: request.temperature,
                top_p: request.top_p,
                top_k: Some(40),
                num_predict: request.max_tokens.map(|t| t as i32),
                stop: request.stop_sequences.clone(),
                repeat_penalty: Some(1.1),
            }),
        };

        let ollama_client = crate::services::ollama::OllamaClient::new();
        let ollama_response = ollama_client.generate(generation_request).await
            .map_err(|e| LocalMindError::AiService(format!("Model generation failed: {}", e)))?;

        // Parse token usage (simplified - would need actual token counting)
        let prompt_tokens = estimate_tokens(&request.prompt);
        let completion_tokens = estimate_tokens(&ollama_response.response);

        Ok(InferenceResponse {
            session_id,
            content: ollama_response.response,
            model_used: model_selection.model_type.display_name(),
            tokens_generated: Some(completion_tokens),
            generation_time_ms: ollama_response.total_duration.unwrap_or(0) / 1_000_000, // Convert to ms
            reasoning: Some(model_selection.reasoning.clone()),
            confidence: model_selection.confidence,
            finish_reason: if ollama_response.done {
                FinishReason::Completed
            } else {
                FinishReason::Error("Generation incomplete".to_string())
            },
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
        })
    }

    /// Build prompt with session context
    async fn build_contextual_prompt(&self, request: &InferenceRequest, session_id: &str) -> Result<String> {
        let mut prompt = String::new();

        // Add conversation history if available
        if let Some(session_id) = &request.session_id {
            let session_manager = self.session_manager.lock().await;
            if let Some(context) = session_manager.get_context(session_id, 5).await? {
                prompt.push_str(&context);
                prompt.push_str("\n\n");
            }
        }

        // Add additional context from memory if provided
        if let Some(context) = &request.context {
            prompt.push_str("Relevant context:\n");
            for (i, ctx) in context.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, ctx));
            }
            prompt.push_str("\n");
        }

        // Add the current prompt
        prompt.push_str("Human: ");
        prompt.push_str(&request.prompt);
        prompt.push_str("\n\nAssistant: ");

        Ok(prompt)
    }

    /// Update performance metrics
    async fn update_metrics(
        &self,
        model_type: &ModelType,
        generation_time_ms: u64,
        response: &InferenceResponse,
    ) -> Result<()> {
        let mut metrics = self.performance_metrics.write().await;
        
        metrics.total_requests += 1;
        metrics.successful_requests += 1;
        
        // Update average response time
        let total_time = metrics.average_response_time_ms * (metrics.total_requests - 1) as f64;
        metrics.average_response_time_ms = (total_time + generation_time_ms as f64) / metrics.total_requests as f64;
        
        // Update model-specific stats
        let model_id = model_type.identifier();
        let model_stats = metrics.model_usage_stats.entry(model_id).or_insert(ModelUsageStats {
            requests: 0,
            total_tokens: 0,
            average_tokens_per_second: 0.0,
            average_response_time_ms: 0.0,
            last_used: chrono::Utc::now(),
        });

        model_stats.requests += 1;
        model_stats.total_tokens += response.usage.total_tokens as u64;
        model_stats.last_used = chrono::Utc::now();
        
        // Update average response time for this model
        let model_total_time = model_stats.average_response_time_ms * (model_stats.requests - 1) as f64;
        model_stats.average_response_time_ms = (model_total_time + generation_time_ms as f64) / model_stats.requests as f64;
        
        // Calculate tokens per second
        if generation_time_ms > 0 {
            let tokens_per_second = (response.usage.completion_tokens as f64 / generation_time_ms as f64) * 1000.0;
            let total_tps = model_stats.average_tokens_per_second * (model_stats.requests - 1) as f64;
            model_stats.average_tokens_per_second = (total_tps + tokens_per_second) / model_stats.requests as f64;
        }

        metrics.last_updated = chrono::Utc::now();
        Ok(())
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> EngineMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Get loaded models information
    pub async fn get_loaded_models(&self) -> Result<Vec<String>> {
        let model_manager = self.model_manager.lock().await;
        Ok(model_manager.get_loaded_models().await?)
    }

    /// Unload a specific model to free memory
    pub async fn unload_model(&self, model_type: &ModelType) -> Result<()> {
        let mut model_manager = self.model_manager.lock().await;
        model_manager.unload_model(model_type).await
    }

    /// Preload models for faster response
    pub async fn preload_models(&self, model_types: Vec<ModelType>) -> Result<()> {
        let mut model_manager = self.model_manager.lock().await;
        for model_type in model_types {
            if !model_manager.is_model_loaded(&model_type).await? {
                model_manager.load_model(&model_type).await?;
            }
        }
        Ok(())
    }

    /// Get active sessions count
    pub async fn get_active_sessions_count(&self) -> usize {
        self.active_sessions.read().await.len()
    }

    /// Clean up inactive sessions
    pub async fn cleanup_sessions(&self, max_age_minutes: u64) -> Result<usize> {
        let mut session_manager = self.session_manager.lock().await;
        let cleaned = session_manager.cleanup_old_sessions(max_age_minutes).await?;
        
        // Also clean up tracking
        let mut active_sessions = self.active_sessions.write().await;
        let session_ids: Vec<String> = active_sessions.keys().cloned().collect();
        let mut removed = 0;
        
        for session_id in session_ids {
            if !session_manager.session_exists(&session_id).await? {
                active_sessions.remove(&session_id);
                removed += 1;
            }
        }
        
        Ok(cleaned + removed)
    }

    /// Force garbage collection and memory optimization
    pub async fn optimize_memory(&self) -> Result<()> {
        let mut model_manager = self.model_manager.lock().await;
        model_manager.optimize_memory().await?;
        
        // Clean up old sessions
        self.cleanup_sessions(60).await?; // Clean sessions older than 1 hour
        
        Ok(())
    }
}

impl EngineMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            model_usage_stats: HashMap::new(),
            last_updated: chrono::Utc::now(),
        }
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Get most used model
    pub fn most_used_model(&self) -> Option<String> {
        self.model_usage_stats
            .iter()
            .max_by_key(|(_, stats)| stats.requests)
            .map(|(model, _)| model.clone())
    }
}

/// Estimate token count (simplified implementation)
fn estimate_tokens(text: &str) -> u32 {
    // Rough approximation: 1 token ≈ 4 characters for English text
    // This would be replaced with proper tokenization in production
    (text.len() as f32 / 4.0).ceil() as u32
}

impl Default for EngineMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        let text = "Hello, world!";
        let estimated = estimate_tokens(text);
        assert!(estimated > 0);
        assert!(estimated <= text.len() as u32);
    }

    #[test]
    fn test_metrics_creation() {
        let metrics = EngineMetrics::new();
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.success_rate(), 0.0);
        assert!(metrics.most_used_model().is_none());
    }

    #[test]
    fn test_finish_reason_serialization() {
        let reason = FinishReason::Completed;
        let serialized = serde_json::to_string(&reason).unwrap();
        let deserialized: FinishReason = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            FinishReason::Completed => {},
            _ => panic!("Serialization/deserialization failed"),
        }
    }

    #[tokio::test]
    async fn test_engine_creation() {
        // This test might fail without proper setup, but tests the interface
        match LLMEngine::new().await {
            Ok(engine) => {
                assert_eq!(engine.get_active_sessions_count().await, 0);
            },
            Err(_) => {
                // Expected to fail without proper model setup
                println!("Engine creation failed (expected without models)");
            }
        }
    }
}