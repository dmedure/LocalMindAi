use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::types::{Agent, Message, Document};
use crate::config::AppConfig;
use crate::memory::MemoryCoordinator;
use crate::vector::VectorStore;

/// Main application state container
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    pub config: AppConfig,
    
    /// All agents, keyed by agent ID
    pub agents: Arc<Mutex<HashMap<String, Agent>>>,
    
    /// All messages, keyed by agent ID
    pub messages: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    
    /// All documents, keyed by document ID
    pub documents: Arc<Mutex<HashMap<String, Document>>>,
    
    /// Service status tracking
    pub service_status: Arc<Mutex<ServiceStatus>>,
    
    /// Memory system coordinator (optional, requires features)
    pub memory_system: Option<Arc<MemoryCoordinator>>,
    
    /// Vector store (optional, requires features)
    pub vector_store: Option<Arc<VectorStore>>,
}

/// Status of external services
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub ollama_available: bool,
    pub chroma_available: bool,
    pub qdrant_available: bool,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self {
            ollama_available: false,
            chroma_available: false,
            qdrant_available: false,
            last_check: chrono::Utc::now(),
        }
    }
}

impl AppState {
    /// Create a new application state with the given configuration
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            agents: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            documents: Arc::new(Mutex::new(HashMap::new())),
            service_status: Arc::new(Mutex::new(ServiceStatus::default())),
            memory_system: None,
            vector_store: None,
        }
    }
    
    /// Set the memory system coordinator
    pub fn with_memory_system(mut self, memory_system: MemoryCoordinator) -> Self {
        self.memory_system = Some(Arc::new(memory_system));
        self
    }
    
    /// Set the vector store
    pub fn with_vector_store(mut self, vector_store: VectorStore) -> Self {
        self.vector_store = Some(Arc::new(vector_store));
        self
    }
    
    /// Update service status
    pub async fn update_service_status(
        &self,
        ollama: Option<bool>,
        chroma: Option<bool>,
        qdrant: Option<bool>,
    ) {
        let mut status = self.service_status.lock().await;
        
        if let Some(ollama_status) = ollama {
            status.ollama_available = ollama_status;
        }
        if let Some(chroma_status) = chroma {
            status.chroma_available = chroma_status;
        }
        if let Some(qdrant_status) = qdrant {
            status.qdrant_available = qdrant_status;
        }
        
        status.last_check = chrono::Utc::now();
    }
    
    /// Get the current service status
    pub async fn get_service_status(&self) -> ServiceStatus {
        self.service_status.lock().await.clone()
    }
    
    /// Check if AI services are available
    pub async fn is_ai_available(&self) -> bool {
        let status = self.service_status.lock().await;
        status.ollama_available
    }
    
    /// Check if vector search is available
    pub async fn is_vector_search_available(&self) -> bool {
        let status = self.service_status.lock().await;
        status.chroma_available || status.qdrant_available
    }
}