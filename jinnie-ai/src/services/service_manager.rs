use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Error(String),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
    pub url: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

pub struct ServiceManager {
    services: HashMap<String, ServiceInfo>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
    
    pub fn register_service(&mut self, name: String, url: String) {
        let info = ServiceInfo {
            name: name.clone(),
            status: ServiceStatus::Unknown,
            url,
            last_check: chrono::Utc::now(),
        };
        self.services.insert(name, info);
    }
    
    pub async fn check_all_services(&mut self) -> Result<()> {
        for (_name, info) in self.services.iter_mut() {
            // Simple health check - try to connect to the service
            match reqwest::get(&format!("{}/health", info.url)).await {
                Ok(response) if response.status().is_success() => {
                    info.status = ServiceStatus::Running;
                }
                Ok(_) => {
                    info.status = ServiceStatus::Error("Unhealthy response".to_string());
                }
                Err(e) => {
                    info.status = ServiceStatus::Error(format!("Connection failed: {}", e));
                }
            }
            info.last_check = chrono::Utc::now();
        }
        Ok(())
    }
    
    pub fn get_service_status(&self, name: &str) -> Option<&ServiceStatus> {
        self.services.get(name).map(|info| &info.status)
    }
    
    pub fn all_services_running(&self) -> bool {
        self.services.values().all(|info| matches!(info.status, ServiceStatus::Running))
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}