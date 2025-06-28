use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ResourceMonitor {
    system: Arc<RwLock<System>>,
    metrics: Arc<RwLock<ResourceMetrics>>,
    update_interval: Duration,
    monitoring_active: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_usage_percent: f32,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub memory_available_mb: u64,
    pub disk_total_gb: u64,
    pub disk_used_gb: u64,
    pub disk_available_gb: u64,
    pub gpu_available: bool,
    pub gpu_memory_mb: Option<u64>,
    pub network_active: bool,
    pub process_count: usize,
    pub system_uptime_seconds: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub performance_level: PerformanceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceLevel {
    Excellent,  // All resources abundant
    Good,       // Normal operation
    Moderate,   // Some resource constraints
    Poor,       // Significant constraints
    Critical,   // System under stress
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub recommended_model: String,
    pub reason: String,
    pub confidence: f32,
    pub alternative_models: Vec<String>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            metrics: Arc::new(RwLock::new(ResourceMetrics::default())),
            update_interval: Duration::from_secs(5),
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        let mut monitoring_active = self.monitoring_active.write().await;
        if *monitoring_active {
            log::warn!("Resource monitoring is already active");
            return Ok(());
        }
        *monitoring_active = true;

        log::info!("Starting resource monitoring...");
        
        let system = Arc::clone(&self.system);
        let metrics = Arc::clone(&self.metrics);
        let update_interval = self.update_interval;
        let monitoring_flag = Arc::clone(&self.monitoring_active);

        tokio::spawn(async move {
            while *monitoring_flag.read().await {
                if let Err(e) = Self::update_metrics(&system, &metrics).await {
                    log::error!("Error updating system metrics: {}", e);
                }
                
                tokio::time::sleep(update_interval).await;
            }
            
            log::info!("Resource monitoring stopped");
        });

        Ok(())
    }

    pub async fn stop_monitoring(&self) {
        let mut monitoring_active = self.monitoring_active.write().await;
        *monitoring_active = false;
        log::info!("Stopping resource monitoring...");
    }

    pub async fn get_current_metrics(&self) -> ResourceMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    pub async fn get_model_recommendation(&self, task_complexity: f32) -> ModelRecommendation {
        let metrics = self.get_current_metrics().await;
        
        // Analyze current system state
        let memory_pressure = 1.0 - (metrics.memory_available_mb as f32 / metrics.memory_total_mb as f32);
        let cpu_load = metrics.cpu_usage_percent / 100.0;
        
        let recommendation = if memory_pressure > 0.8 || cpu_load > 0.9 {
            // System under stress - recommend lightweight model
            ModelRecommendation {
                recommended_model: "TinyLlama".to_string(),
                reason: "System resources are constrained".to_string(),
                confidence: 0.9,
                alternative_models: vec![],
            }
        } else if task_complexity > 0.7 && memory_pressure < 0.6 && cpu_load < 0.5 {
            // Complex task with good resources - recommend powerful model
            ModelRecommendation {
                recommended_model: "Mistral7B".to_string(),
                reason: "Complex task detected with sufficient resources available".to_string(),
                confidence: 0.8,
                alternative_models: vec!["TinyLlama".to_string()],
            }
        } else if task_complexity < 0.3 {
            // Simple task - lightweight model is sufficient
            ModelRecommendation {
                recommended_model: "TinyLlama".to_string(),
                reason: "Simple task - lightweight model is sufficient".to_string(),
                confidence: 0.85,
                alternative_models: vec!["Mistral7B".to_string()],
            }
        } else {
            // Balanced case - choose based on current performance level
            match metrics.performance_level {
                PerformanceLevel::Excellent | PerformanceLevel::Good => {
                    ModelRecommendation {
                        recommended_model: "Mistral7B".to_string(),
                        reason: "Good system performance allows for quality model".to_string(),
                        confidence: 0.7,
                        alternative_models: vec!["TinyLlama".to_string()],
                    }
                }
                _ => {
                    ModelRecommendation {
                        recommended_model: "TinyLlama".to_string(),
                        reason: "Optimizing for performance under current conditions".to_string(),
                        confidence: 0.7,
                        alternative_models: vec!["Mistral7B".to_string()],
                    }
                }
            }
        };

        log::debug!("Model recommendation: {} (confidence: {:.1}%) - {}", 
                   recommendation.recommended_model, 
                   recommendation.confidence * 100.0,
                   recommendation.reason);

        recommendation
    }

    pub async fn can_load_model(&self, model_name: &str) -> bool {
        let metrics = self.get_current_metrics().await;
        
        // Estimate model memory requirements
        let required_memory_mb = match model_name {
            "TinyLlama" => 1024,  // ~1GB
            "Mistral7B" => 4096,  // ~4GB
            _ => 2048,            // Default estimate
        };

        metrics.memory_available_mb >= required_memory_mb
    }

    async fn update_metrics(
        system: &Arc<RwLock<System>>,
        metrics: &Arc<RwLock<ResourceMetrics>>,
    ) -> Result<()> {
        let mut sys = system.write().await;
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_total = sys.total_memory() / 1024 / 1024; // Convert to MB
        let memory_used = sys.used_memory() / 1024 / 1024;
        let memory_available = sys.available_memory() / 1024 / 1024;

        // Get disk information (primary disk)
        let (disk_total, disk_used) = if let Some(disk) = sys.disks().first() {
            let total = disk.total_space() / 1024 / 1024 / 1024; // Convert to GB
            let available = disk.available_space() / 1024 / 1024 / 1024;
            let used = total - available;
            (total, used)
        } else {
            (0, 0)
        };

        let process_count = sys.processes().len();
        let system_uptime = sys.uptime();

        // Determine performance level
        let memory_pressure = memory_used as f32 / memory_total as f32;
        let cpu_load = cpu_usage / 100.0;
        
        let performance_level = if memory_pressure < 0.5 && cpu_load < 0.3 {
            PerformanceLevel::Excellent
        } else if memory_pressure < 0.7 && cpu_load < 0.6 {
            PerformanceLevel::Good
        } else if memory_pressure < 0.8 && cpu_load < 0.8 {
            PerformanceLevel::Moderate
        } else if memory_pressure < 0.9 && cpu_load < 0.9 {
            PerformanceLevel::Poor
        } else {
            PerformanceLevel::Critical
        };

        let new_metrics = ResourceMetrics {
            cpu_usage_percent: cpu_usage,
            memory_total_mb: memory_total,
            memory_used_mb: memory_used,
            memory_available_mb: memory_available,
            disk_total_gb: disk_total,
            disk_used_gb: disk_used,
            disk_available_gb: disk_total - disk_used,
            gpu_available: Self::detect_gpu().await,
            gpu_memory_mb: Self::get_gpu_memory().await,
            network_active: true, // TODO: Implement actual network detection
            process_count,
            system_uptime_seconds: system_uptime,
            last_updated: chrono::Utc::now(),
            performance_level,
        };

        let mut current_metrics = metrics.write().await;
        *current_metrics = new_metrics;

        Ok(())
    }

    async fn detect_gpu() -> bool {
        // TODO: Implement GPU detection using appropriate library
        // For now, return false as placeholder
        false
    }

    async fn get_gpu_memory() -> Option<u64> {
        // TODO: Implement GPU memory detection
        // For now, return None as placeholder
        None
    }

    pub async fn get_system_info(&self) -> SystemInfo {
        let sys = self.system.read().await;
        
        SystemInfo {
            os_name: sys.name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: sys.os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: sys.kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: sys.host_name().unwrap_or_else(|| "Unknown".to_string()),
            cpu_count: sys.cpus().len(),
            cpu_brand: sys.cpus().first().map(|cpu| cpu.brand().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            total_memory_gb: sys.total_memory() / 1024 / 1024 / 1024,
            architecture: std::env::consts::ARCH.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub total_memory_gb: u64,
    pub architecture: String,
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_total_mb: 0,
            memory_used_mb: 0,
            memory_available_mb: 0,
            disk_total_gb: 0,
            disk_used_gb: 0,
            disk_available_gb: 0,
            gpu_available: false,
            gpu_memory_mb: None,
            network_active: false,
            process_count: 0,
            system_uptime_seconds: 0,
            last_updated: chrono::Utc::now(),
            performance_level: PerformanceLevel::Good,
        }
    }
}