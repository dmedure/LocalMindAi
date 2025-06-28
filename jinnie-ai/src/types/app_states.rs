use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;
use chrono::{DateTime, Utc};

use super::{Agent, Message, Document};
use crate::config::AppConfig;
use crate::llm::LLMEngine;
use crate::memory::MemoryManager;
use crate::vector::QdrantManager;

#[derive(Debug)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub agents: Arc<RwLock<HashMap<String, Agent>>>,
    pub messages: Arc<Mutex<HashMap<String, Vec<Message>>>>, // agent_id -> messages
    pub documents: Arc<RwLock<Vec<Document>>>,
    pub service_status: Arc<RwLock<ServiceStatus>>,
    pub llm_engine: Arc<Mutex<Option<LLMEngine>>>,
    pub memory_manager: Arc<Mutex<Option<MemoryManager>>>,
    pub vector_manager: Arc<Mutex<Option<QdrantManager>>>,
    pub system_metrics: Arc<RwLock<SystemMetrics>>,
    pub active_sessions: Arc<RwLock<HashMap<String, ChatSession>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub ollama: bool,
    pub qdrant: bool,
    pub llm_engine: EngineStatus,
    pub memory_system: bool,
    pub vector_search: bool,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub loaded_models: Vec<String>,
    pub active_model: Option<String>,
    pub memory_usage_mb: f64,
    pub last_inference_ms: Option<u64>,
    pub total_inferences: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub available_memory_mb: u64,
    pub disk_usage_mb: u64,
    pub gpu_available: bool,
    pub gpu_memory_mb: Option<u64>,
    pub last_updated: DateTime<Utc>,
    pub performance_history: PerformanceHistory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHistory {
    pub response_times: Vec<(DateTime<Utc>, u64)>, // timestamp, ms
    pub memory_usage: Vec<(DateTime<Utc>, u64)>,   // timestamp, MB
    pub cpu_usage: Vec<(DateTime<Utc>, f32)>,      // timestamp, %
    pub max_history_points: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub agent_id: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub message_count: u32,
    pub context_length: usize,
    pub active_model: Option<String>,
    pub session_metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub importance_score: f32,
    pub summary: Option<String>,
    pub memory_anchors: Vec<String>, // Important memory IDs to preserve
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppHealth {
    pub overall_status: HealthStatus,
    pub services: HashMap<String, ServiceHealth>,
    pub performance_score: f32, // 0.0-1.0
    pub issues: Vec<HealthIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub error_rate: f32,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub severity: IssueSeverity,
    pub service: String,
    pub description: String,
    pub suggested_action: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AppConfig::default())),
            agents: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            documents: Arc::new(RwLock::new(Vec::new())),
            service_status: Arc::new(RwLock::new(ServiceStatus::default())),
            llm_engine: Arc::new(Mutex::new(None)),
            memory_manager: Arc::new(Mutex::new(None)),
            vector_manager: Arc::new(Mutex::new(None)),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Initialize all subsystems
        log::info!("Initializing application state...");
        
        // Load configuration
        // Initialize services based on config
        // Set up monitoring
        
        Ok(())
    }
    
    pub async fn get_agent(&self, agent_id: &str) -> Option<Agent> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }
    
    pub async fn add_agent(&self, agent: Agent) -> Result<(), String> {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id.clone(), agent);
        Ok(())
    }
    
    pub async fn get_agent_messages(&self, agent_id: &str) -> Vec<Message> {
        let messages = self.messages.lock().await;
        messages.get(agent_id).cloned().unwrap_or_default()
    }
    
    pub async fn add_message(&self, agent_id: String, message: Message) -> Result<(), String> {
        let mut messages = self.messages.lock().await;
        messages.entry(agent_id).or_insert_with(Vec::new).push(message);
        Ok(())
    }
    
    pub async fn create_session(&self, agent_id: String) -> ChatSession {
        let session = ChatSession::new(agent_id);
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        session
    }
    
    pub async fn get_session(&self, session_id: &str) -> Option<ChatSession> {
        let sessions = self.active_sessions.read().await;
        sessions.get(session_id).cloned()
    }
    
    pub async fn update_session_activity(&self, session_id: &str) {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_active = Utc::now();
        }
    }
    
    pub async fn get_system_health(&self) -> AppHealth {
        let metrics = self.system_metrics.read().await;
        let service_status = self.service_status.read().await;
        
        // Calculate overall health based on metrics and service status
        let mut services = HashMap::new();
        
        services.insert("llm_engine".to_string(), ServiceHealth {
            status: if service_status.llm_engine.active_model.is_some() {
                HealthStatus::Healthy
            } else {
                HealthStatus::Warning
            },
            last_check: service_status.last_updated,
            response_time_ms: service_status.llm_engine.last_inference_ms,
            error_rate: 0.0, // TODO: Track actual error rates
            uptime_seconds: 0, // TODO: Track actual uptime
        });
        
        AppHealth {
            overall_status: HealthStatus::Healthy, // TODO: Calculate based on services
            services,
            performance_score: 0.8, // TODO: Calculate based on metrics
            issues: Vec::new(),
            recommendations: Vec::new(),
        }
    }
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self {
            ollama: false,
            qdrant: false,
            llm_engine: EngineStatus::default(),
            memory_system: false,
            vector_search: false,
            last_updated: Utc::now(),
        }
    }
}

impl Default for EngineStatus {
    fn default() -> Self {
        Self {
            loaded_models: Vec::new(),
            active_model: None,
            memory_usage_mb: 0.0,
            last_inference_ms: None,
            total_inferences: 0,
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0,
            available_memory_mb: 0,
            disk_usage_mb: 0,
            gpu_available: false,
            gpu_memory_mb: None,
            last_updated: Utc::now(),
            performance_history: PerformanceHistory::default(),
        }
    }
}

impl Default for PerformanceHistory {
    fn default() -> Self {
        Self {
            response_times: Vec::new(),
            memory_usage: Vec::new(),
            cpu_usage: Vec::new(),
            max_history_points: 100,
        }
    }
}

impl ChatSession {
    pub fn new(agent_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id,
            created_at: Utc::now(),
            last_active: Utc::now(),
            message_count: 0,
            context_length: 0,
            active_model: None,
            session_metadata: SessionMetadata::default(),
        }
    }
    
    pub fn increment_message_count(&mut self) {
        self.message_count += 1;
        self.last_active = Utc::now();
    }
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            title: None,
            tags: Vec::new(),
            importance_score: 0.5,
            summary: None,
            memory_anchors: Vec::new(),
        }
    }
}

impl PerformanceHistory {
    pub fn add_response_time(&mut self, time_ms: u64) {
        self.response_times.push((Utc::now(), time_ms));
        if self.response_times.len() > self.max_history_points {
            self.response_times.remove(0);
        }
    }
    
    pub fn add_memory_usage(&mut self, usage_mb: u64) {
        self.memory_usage.push((Utc::now(), usage_mb));
        if self.memory_usage.len() > self.max_history_points {
            self.memory_usage.remove(0);
        }
    }
    
    pub fn add_cpu_usage(&mut self, usage_percent: f32) {
        self.cpu_usage.push((Utc::now(), usage_percent));
        if self.cpu_usage.len() > self.max_history_points {
            self.cpu_usage.remove(0);
        }
    }
}