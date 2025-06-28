use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::{ModelType, SelectionFactors, ModelSelection};
use crate::llm::{InferenceRequest, TaskComplexity};
use crate::utils::error::{LocalMindError, Result};

/// Intelligent model selector that chooses between TinyLlama and Mistral 7B
pub struct ModelSelector {
    selection_strategy: ModelSelection,
    performance_history: HashMap<String, ModelPerformanceHistory>,
    system_resources: SystemResources,
    user_preferences: UserPreferences,
}

/// Result of model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    pub model_type: ModelType,
    pub reasoning: String,
    pub confidence: Option<f32>,
    pub estimated_response_time_ms: Option<u64>,
    pub resource_requirements: ResourceEstimate,
}

/// Performance history for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceHistory {
    pub average_response_time_ms: f64,
    pub success_rate: f64,
    pub tokens_per_second: f64,
    pub user_satisfaction_score: f64,
    pub recent_performance: Vec<PerformanceDataPoint>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Single performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
    pub tokens_generated: u32,
    pub task_complexity: f32,
    pub user_rating: Option<u8>, // 1-5 rating
}

/// Current system resource status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub available_memory_gb: f64,
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub gpu_available: bool,
    pub gpu_memory_gb: Option<f64>,
    pub thermal_state: ThermalState,
}

/// System thermal state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThermalState {
    Normal,
    Warm,
    Hot,
    Critical,
}

/// User preferences for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_model: Option<String>,
    pub speed_vs_quality_preference: f32, // 0.0 = speed, 1.0 = quality
    pub power_saving_mode: bool,
    pub quality_threshold: f32,
    pub max_acceptable_response_time_ms: Option<u64>,
}

/// Resource requirements estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub memory_mb: u64,
    pub cpu_usage_percent: f32,
    pub estimated_power_consumption: PowerConsumption,
}

/// Power consumption estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerConsumption {
    Low,
    Medium,
    High,
}

impl ModelSelector {
    /// Create a new model selector
    pub async fn new() -> Result<Self> {
        let system_resources = Self::detect_system_resources().await?;
        
        Ok(Self {
            selection_strategy: ModelSelection::Adaptive,
            performance_history: HashMap::new(),
            system_resources,
            user_preferences: UserPreferences::default(),
        })
    }

    /// Select the most appropriate model for a request
    pub async fn select_model(
        &self,
        request: &InferenceRequest,
        task_complexity: &TaskComplexity,
    ) -> Result<SelectionResult> {
        match self.selection_strategy {
            ModelSelection::Adaptive => self.adaptive_selection(request, task_complexity).await,
            ModelSelection::Manual => self.manual_selection(request).await,
            ModelSelection::Performance => self.performance_optimized_selection().await,
            ModelSelection::Quality => self.quality_optimized_selection().await,
            ModelSelection::Balanced => self.balanced_selection(request, task_complexity).await,
        }
    }

    /// Adaptive selection based on multiple factors
    async fn adaptive_selection(
        &self,
        request: &InferenceRequest,
        task_complexity: &TaskComplexity,
    ) -> Result<SelectionResult> {
        let selection_factors = self.build_selection_factors(request, task_complexity).await;
        
        // Calculate scores for each available model
        let tinyllama_score = self.calculate_model_score(&ModelType::default(), &selection_factors).await;
        let mistral_score = self.calculate_model_score(
            &ModelType::Mistral7B {
                version: "v0.2".to_string(),
                quantization: crate::config::QuantizationType::Q4_K_M,
                context_window: 8192,
            },
            &selection_factors,
        ).await;

        // Select the model with higher score
        let (selected_model, score, reasoning) = if tinyllama_score > mistral_score {
            (
                ModelType::default(),
                tinyllama_score,
                self.build_reasoning("TinyLlama", tinyllama_score, mistral_score, &selection_factors),
            )
        } else {
            (
                ModelType::Mistral7B {
                    version: "v0.2".to_string(),
                    quantization: crate::config::QuantizationType::Q4_K_M,
                    context_window: 8192,
                },
                mistral_score,
                self.build_reasoning("Mistral7B", mistral_score, tinyllama_score, &selection_factors),
            )
        };

        // Estimate response time and resource requirements
        let estimated_response_time = self.estimate_response_time(&selected_model, task_complexity);
        let resource_requirements = self.estimate_resource_requirements(&selected_model);

        Ok(SelectionResult {
            model_type: selected_model,
            reasoning,
            confidence: Some(score),
            estimated_response_time_ms: estimated_response_time,
            resource_requirements,
        })
    }

    /// Calculate model suitability score
    async fn calculate_model_score(&self, model_type: &ModelType, factors: &SelectionFactors) -> f32 {
        let mut score = 0.0;

        // Base model capabilities
        let (speed_score, quality_score) = match model_type {
            ModelType::TinyLlama { .. } => (0.9, 0.6), // Fast but lower quality
            ModelType::Mistral7B { .. } => (0.6, 0.9), // Slower but higher quality
        };

        // Weight by user preferences and task requirements
        let speed_weight = 1.0 - factors.quality_requirement;
        let quality_weight = factors.quality_requirement;
        score += (speed_score * speed_weight) + (quality_score * quality_weight);

        // Resource availability factor
        let resource_factor = self.calculate_resource_factor(model_type, factors);
        score *= resource_factor;

        // Task complexity alignment
        let complexity_factor = self.calculate_complexity_alignment(model_type, factors.task_complexity);
        score *= complexity_factor;

        // Historical performance factor
        if let Some(history) = self.performance_history.get(&model_type.identifier()) {
            let history_factor = self.calculate_history_factor(history, factors);
            score *= history_factor;
        }

        // User preference factor
        let preference_factor = self.calculate_preference_factor(model_type);
        score *= preference_factor;

        // System load factor
        let load_factor = 1.0 - (factors.cpu_load * 0.3); // Reduce score if system is under load
        score *= load_factor;

        // Thermal throttling consideration
        let thermal_factor = self.calculate_thermal_factor(model_type);
        score *= thermal_factor;

        score.min(1.0).max(0.0)
    }

    /// Calculate resource availability factor
    fn calculate_resource_factor(&self, model_type: &ModelType, factors: &SelectionFactors) -> f32 {
        let required_memory_gb = match model_type {
            ModelType::TinyLlama { .. } => 2.0,
            ModelType::Mistral7B { .. } => 6.0,
        };

        let available_memory_gb = factors.available_memory as f64 / 1_000_000_000.0;
        
        if available_memory_gb < required_memory_gb {
            0.1 // Heavily penalize if insufficient memory
        } else {
            (available_memory_gb / (required_memory_gb * 1.5)).min(1.0) as f32
        }
    }

    /// Calculate task complexity alignment
    fn calculate_complexity_alignment(&self, model_type: &ModelType, complexity: f32) -> f32 {
        match model_type {
            ModelType::TinyLlama { .. } => {
                // TinyLlama is better for simple tasks
                if complexity < 0.5 {
                    1.0
                } else {
                    0.7 + (1.0 - complexity) * 0.3
                }
            },
            ModelType::Mistral7B { .. } => {
                // Mistral is better for complex tasks
                if complexity > 0.5 {
                    1.0
                } else {
                    0.7 + complexity * 0.3
                }
            },
        }
    }

    /// Calculate historical performance factor
    fn calculate_history_factor(&self, history: &ModelPerformanceHistory, factors: &SelectionFactors) -> f32 {
        let mut factor = 1.0;

        // Success rate factor
        factor *= history.success_rate as f32;

        // Response time factor (relative to urgency)
        if factors.response_urgency > 0.7 {
            let time_factor = if history.average_response_time_ms < 1000.0 {
                1.0
            } else {
                1000.0 / history.average_response_time_ms as f32
            };
            factor *= time_factor;
        }

        // User satisfaction factor
        factor *= (history.user_satisfaction_score / 5.0) as f32;

        factor.min(1.0).max(0.2)
    }

    /// Calculate user preference factor
    fn calculate_preference_factor(&self, model_type: &ModelType) -> f32 {
        if let Some(ref preferred) = self.user_preferences.preferred_model {
            if *preferred == model_type.identifier() {
                1.2 // Boost preferred model
            } else {
                0.9 // Slight penalty for non-preferred
            }
        } else {
            1.0 // No preference
        }
    }

    /// Calculate thermal factor
    fn calculate_thermal_factor(&self, model_type: &ModelType) -> f32 {
        match self.system_resources.thermal_state {
            ThermalState::Normal => 1.0,
            ThermalState::Warm => {
                match model_type {
                    ModelType::TinyLlama { .. } => 1.0, // Less affected by heat
                    ModelType::Mistral7B { .. } => 0.9, // Slightly affected
                }
            },
            ThermalState::Hot => {
                match model_type {
                    ModelType::TinyLlama { .. } => 1.0,
                    ModelType::Mistral7B { .. } => 0.7, // More affected by heat
                }
            },
            ThermalState::Critical => {
                match model_type {
                    ModelType::TinyLlama { .. } => 0.9,
                    ModelType::Mistral7B { .. } => 0.3, // Heavily penalized in critical thermal state
                }
            },
        }
    }

    /// Build selection factors from request and context
    async fn build_selection_factors(&self, request: &InferenceRequest, task_complexity: &TaskComplexity) -> SelectionFactors {
        SelectionFactors {
            task_complexity: task_complexity.score,
            available_memory: (self.system_resources.available_memory_gb * 1_000_000_000.0) as u64,
            cpu_load: self.system_resources.cpu_usage_percent,
            response_urgency: self.estimate_urgency(request),
            quality_requirement: self.estimate_quality_requirement(request, task_complexity),
            user_preference: self.selection_strategy.clone(),
            conversation_context: crate::config::ConversationState {
                message_count: 1, // Would be tracked in session
                average_complexity: task_complexity.score,
                current_topic: task_complexity.detected_topics.first().cloned(),
                user_satisfaction: None,
            },
        }
    }

    /// Estimate response urgency from request
    fn estimate_urgency(&self, request: &InferenceRequest) -> f32 {
        // This could be based on:
        // - User settings
        // - Request metadata
        // - Historical patterns
        // - Time of day
        
        if request.stream {
            0.8 // Streaming suggests user wants quick response
        } else {
            0.5 // Default moderate urgency
        }
    }

    /// Estimate quality requirement
    fn estimate_quality_requirement(&self, request: &InferenceRequest, task_complexity: &TaskComplexity) -> f32 {
        let mut quality_req = task_complexity.score;

        // Adjust based on user preferences
        quality_req = quality_req * 0.7 + self.user_preferences.speed_vs_quality_preference * 0.3;

        // Certain task types always require high quality
        if task_complexity.detected_topics.iter().any(|topic| {
            topic.contains("code") || topic.contains("analysis") || topic.contains("research")
        }) {
            quality_req = quality_req.max(0.7);
        }

        quality_req.min(1.0)
    }

    /// Build human-readable reasoning for model selection
    fn build_reasoning(&self, selected_model: &str, selected_score: f32, other_score: f32, factors: &SelectionFactors) -> String {
        let mut reasons = Vec::new();

        if factors.task_complexity > 0.7 {
            if selected_model == "Mistral7B" {
                reasons.push("Complex task requiring high-quality reasoning".to_string());
            } else {
                reasons.push("Despite complexity, system constraints favor faster model".to_string());
            }
        } else if factors.task_complexity < 0.3 {
            if selected_model == "TinyLlama" {
                reasons.push("Simple task suitable for fast inference".to_string());
            } else {
                reasons.push("Quality preference overrides task simplicity".to_string());
            }
        }

        if factors.response_urgency > 0.8 {
            reasons.push("High urgency requires fast response".to_string());
        }

        if factors.quality_requirement > 0.8 {
            reasons.push("High quality requirement".to_string());
        }

        let memory_gb = factors.available_memory as f64 / 1_000_000_000.0;
        if memory_gb < 4.0 {
            reasons.push("Limited memory constrains model choice".to_string());
        }

        if factors.cpu_load > 0.8 {
            reasons.push("High CPU load favors lighter model".to_string());
        }

        match self.system_resources.thermal_state {
            ThermalState::Hot | ThermalState::Critical => {
                reasons.push("High thermal state constrains performance".to_string());
            },
            _ => {},
        }

        if reasons.is_empty() {
            reasons.push(format!("Selected based on overall suitability score ({:.2} vs {:.2})", selected_score, other_score));
        }

        format!("{}: {}", selected_model, reasons.join(", "))
    }

    /// Force selection of a specific model
    pub async fn force_model_selection(&self, model_name: String) -> Result<SelectionResult> {
        let model_type = match model_name.to_lowercase().as_str() {
            "tinyllama" => ModelType::default(),
            "mistral" | "mistral7b" => ModelType::Mistral7B {
                version: "v0.2".to_string(),
                quantization: crate::config::QuantizationType::Q4_K_M,
                context_window: 8192,
            },
            _ => return Err(LocalMindError::Configuration(format!("Unknown model: {}", model_name))),
        };

        Ok(SelectionResult {
            model_type: model_type.clone(),
            reasoning: format!("Manually selected: {}", model_name),
            confidence: Some(1.0),
            estimated_response_time_ms: self.estimate_response_time(&model_type, &TaskComplexity::default()),
            resource_requirements: self.estimate_resource_requirements(&model_type),
        })
    }

    /// Performance-optimized selection (always TinyLlama)
    async fn performance_optimized_selection(&self) -> Result<SelectionResult> {
        let model_type = ModelType::default();
        Ok(SelectionResult {
            model_type: model_type.clone(),
            reasoning: "Performance-optimized strategy: selected fastest model".to_string(),
            confidence: Some(1.0),
            estimated_response_time_ms: self.estimate_response_time(&model_type, &TaskComplexity::default()),
            resource_requirements: self.estimate_resource_requirements(&model_type),
        })
    }

    /// Quality-optimized selection (always Mistral 7B)
    async fn quality_optimized_selection(&self) -> Result<SelectionResult> {
        let model_type = ModelType::Mistral7B {
            version: "v0.2".to_string(),
            quantization: crate::config::QuantizationType::Q4_K_M,
            context_window: 8192,
        };
        
        Ok(SelectionResult {
            model_type: model_type.clone(),
            reasoning: "Quality-optimized strategy: selected highest quality model".to_string(),
            confidence: Some(1.0),
            estimated_response_time_ms: self.estimate_response_time(&model_type, &TaskComplexity::default()),
            resource_requirements: self.estimate_resource_requirements(&model_type),
        })
    }

    /// Balanced selection
    async fn balanced_selection(&self, request: &InferenceRequest, task_complexity: &TaskComplexity) -> Result<SelectionResult> {
        // Use adaptive selection but with balanced weights
        let mut factors = self.build_selection_factors(request, task_complexity).await;
        factors.quality_requirement = 0.5; // Force balanced approach
        factors.response_urgency = 0.5;

        self.adaptive_selection(request, task_complexity).await
    }

    /// Manual selection based on user preference
    async fn manual_selection(&self, _request: &InferenceRequest) -> Result<SelectionResult> {
        if let Some(ref preferred) = self.user_preferences.preferred_model {
            self.force_model_selection(preferred.clone()).await
        } else {
            // Default to TinyLlama if no preference set
            self.force_model_selection("tinyllama".to_string()).await
        }
    }

    /// Estimate response time for a model and task
    fn estimate_response_time(&self, model_type: &ModelType, task_complexity: &TaskComplexity) -> Option<u64> {
        let base_time_ms = match model_type {
            ModelType::TinyLlama { .. } => 500,  // Fast base time
            ModelType::Mistral7B { .. } => 2000, // Slower base time
        };

        let complexity_multiplier = 1.0 + task_complexity.score;
        let estimated_time = (base_time_ms as f32 * complexity_multiplier) as u64;

        // Adjust for system load
        let load_multiplier = 1.0 + self.system_resources.cpu_usage_percent;
        let final_time = (estimated_time as f32 * load_multiplier) as u64;

        Some(final_time)
    }

    /// Estimate resource requirements
    fn estimate_resource_requirements(&self, model_type: &ModelType) -> ResourceEstimate {
        match model_type {
            ModelType::TinyLlama { .. } => ResourceEstimate {
                memory_mb: 2048,
                cpu_usage_percent: 20.0,
                estimated_power_consumption: PowerConsumption::Low,
            },
            ModelType::Mistral7B { .. } => ResourceEstimate {
                memory_mb: 6144,
                cpu_usage_percent: 60.0,
                estimated_power_consumption: PowerConsumption::High,
            },
        }
    }

    /// Detect current system resources
    async fn detect_system_resources() -> Result<SystemResources> {
        // This would use system APIs to detect actual resources
        // For now, using simplified detection
        
        let available_memory_gb = 8.0; // Default assumption
        let cpu_usage_percent = 0.3;   // Default assumption
        let memory_usage_percent = 0.5; // Default assumption
        
        Ok(SystemResources {
            available_memory_gb,
            cpu_usage_percent,
            memory_usage_percent,
            gpu_available: false, // Would detect actual GPU
            gpu_memory_gb: None,
            thermal_state: ThermalState::Normal, // Would detect actual thermal state
        })
    }

    /// Update performance history with new data point
    pub fn update_performance_history(&mut self, model_type: &ModelType, data_point: PerformanceDataPoint) {
        let model_id = model_type.identifier();
        let history = self.performance_history.entry(model_id).or_insert_with(|| {
            ModelPerformanceHistory {
                average_response_time_ms: 0.0,
                success_rate: 1.0,
                tokens_per_second: 0.0,
                user_satisfaction_score: 3.0,
                recent_performance: Vec::new(),
                last_updated: chrono::Utc::now(),
            }
        });

        // Add new data point
        history.recent_performance.push(data_point.clone());
        
        // Keep only recent performance (last 100 data points)
        if history.recent_performance.len() > 100 {
            history.recent_performance.remove(0);
        }

        // Update aggregated metrics
        let total_points = history.recent_performance.len() as f64;
        history.average_response_time_ms = history.recent_performance.iter()
            .map(|p| p.response_time_ms as f64)
            .sum::<f64>() / total_points;

        history.tokens_per_second = history.recent_performance.iter()
            .map(|p| p.tokens_generated as f64 / (p.response_time_ms as f64 / 1000.0))
            .sum::<f64>() / total_points;

        if let Some(rating) = data_point.user_rating {
            let total_ratings = history.recent_performance.iter()
                .filter_map(|p| p.user_rating)
                .count() as f64;
            
            if total_ratings > 0.0 {
                history.user_satisfaction_score = history.recent_performance.iter()
                    .filter_map(|p| p.user_rating)
                    .map(|r| r as f64)
                    .sum::<f64>() / total_ratings;
            }
        }

        history.last_updated = chrono::Utc::now();
    }

    /// Update user preferences
    pub fn update_preferences(&mut self, preferences: UserPreferences) {
        self.user_preferences = preferences;
    }

    /// Set selection strategy
    pub fn set_strategy(&mut self, strategy: ModelSelection) {
        self.selection_strategy = strategy;
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_model: None,
            speed_vs_quality_preference: 0.5, // Balanced
            power_saving_mode: false,
            quality_threshold: 0.7,
            max_acceptable_response_time_ms: Some(5000), // 5 seconds
        }
    }
}

impl Default for TaskComplexity {
    fn default() -> Self {
        Self {
            score: 0.5,
            reasoning_required: false,
            detected_topics: Vec::new(),
            estimated_tokens: 100,
            task_type: crate::llm::TaskType::Conversation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_factor_calculation() {
        let selector = ModelSelector {
            selection_strategy: ModelSelection::Adaptive,
            performance_history: HashMap::new(),
            system_resources: SystemResources {
                available_memory_gb: 8.0,
                cpu_usage_percent: 0.3,
                memory_usage_percent: 0.5,
                gpu_available: false,
                gpu_memory_gb: None,
                thermal_state: ThermalState::Normal,
            },
            user_preferences: UserPreferences::default(),
        };

        let tinyllama = ModelType::default();
        let factors = SelectionFactors::default();
        
        let factor = selector.calculate_resource_factor(&tinyllama, &factors);
        assert!(factor > 0.0 && factor <= 1.0);
    }

    #[test]
    fn test_complexity_alignment() {
        let selector = ModelSelector {
            selection_strategy: ModelSelection::Adaptive,
            performance_history: HashMap::new(),
            system_resources: SystemResources {
                available_memory_gb: 8.0,
                cpu_usage_percent: 0.3,
                memory_usage_percent: 0.5,
                gpu_available: false,
                gpu_memory_gb: None,
                thermal_state: ThermalState::Normal,
            },
            user_preferences: UserPreferences::default(),
        };

        let tinyllama = ModelType::default();
        
        // TinyLlama should be better for simple tasks
        let simple_alignment = selector.calculate_complexity_alignment(&tinyllama, 0.2);
        let complex_alignment = selector.calculate_complexity_alignment(&tinyllama, 0.8);
        
        assert!(simple_alignment > complex_alignment);
    }

    #[test]
    fn test_thermal_factor() {
        let mut selector = ModelSelector {
            selection_strategy: ModelSelection::Adaptive,
            performance_history: HashMap::new(),
            system_resources: SystemResources {
                available_memory_gb: 8.0,
                cpu_usage_percent: 0.3,
                memory_usage_percent: 0.5,
                gpu_available: false,
                gpu_memory_gb: None,
                thermal_state: ThermalState::Critical,
            },
            user_preferences: UserPreferences::default(),
        };

        let mistral = ModelType::Mistral7B {
            version: "v0.2".to_string(),
            quantization: crate::config::QuantizationType::Q4_K_M,
            context_window: 8192,
        };

        let critical_factor = selector.calculate_thermal_factor(&mistral);
        
        // Switch to normal thermal state
        selector.system_resources.thermal_state = ThermalState::Normal;
        let normal_factor = selector.calculate_thermal_factor(&mistral);

        assert!(normal_factor > critical_factor);
    }

    #[tokio::test]
    async fn test_selector_creation() {
        let result = ModelSelector::new().await;
        assert!(result.is_ok());
    }
}