use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<String, ManagedProcess>>>,
}

#[derive(Debug)]
pub struct ManagedProcess {
    pub id: String,
    pub name: String,
    pub child: Child,
    pub config: ProcessConfig,
    pub status: ProcessStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub binary_path: PathBuf,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub auto_restart: bool,
    pub health_check_url: Option<String>,
    pub startup_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_process(&self, id: String, name: String, config: ProcessConfig) -> Result<()> {
        log::info!("Starting process: {} ({})", name, id);

        // Check if process already exists
        {
            let processes = self.processes.read().await;
            if processes.contains_key(&id) {
                return Err(anyhow!("Process with id '{}' already exists", id));
            }
        }

        // Verify binary exists
        if !config.binary_path.exists() {
            return Err(anyhow!("Binary not found: {:?}", config.binary_path));
        }

        // Build command
        let mut command = Command::new(&config.binary_path);
        command.args(&config.args);
        
        for (key, value) in &config.env_vars {
            command.env(key, value);
        }

        if let Some(working_dir) = &config.working_dir {
            command.current_dir(working_dir);
        }

        // Configure stdio
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Start process
        let child = command.spawn()
            .map_err(|e| anyhow!("Failed to start process '{}': {}", name, e))?;

        let managed_process = ManagedProcess {
            id: id.clone(),
            name: name.clone(),
            child,
            config: config.clone(),
            status: ProcessStatus::Starting,
            started_at: chrono::Utc::now(),
        };

        // Add to managed processes
        {
            let mut processes = self.processes.write().await;
            processes.insert(id.clone(), managed_process);
        }

        // Start health monitoring if configured
        if let Some(health_url) = &config.health_check_url {
            self.start_health_monitoring(id.clone(), health_url.clone(), config.startup_timeout_seconds).await;
        } else {
            // Mark as running immediately if no health check
            self.update_process_status(&id, ProcessStatus::Running).await?;
        }

        log::info!("Process '{}' started successfully", name);
        Ok(())
    }

    pub async fn stop_process(&self, id: &str) -> Result<()> {
        log::info!("Stopping process: {}", id);

        let mut processes = self.processes.write().await;
        if let Some(managed_process) = processes.get_mut(id) {
            managed_process.status = ProcessStatus::Stopping;
            
            // Try graceful shutdown first
            #[cfg(unix)]
            {
                // Send SIGTERM
                unsafe {
                    libc::kill(managed_process.child.id() as i32, libc::SIGTERM);
                }
                
                // Wait for a few seconds
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }

            // Force kill if still running
            if let Err(e) = managed_process.child.kill() {
                log::warn!("Failed to kill process {}: {}", id, e);
            }

            match managed_process.child.wait() {
                Ok(status) => {
                    log::info!("Process '{}' stopped with status: {:?}", managed_process.name, status);
                }
                Err(e) => {
                    log::error!("Error waiting for process '{}': {}", managed_process.name, e);
                }
            }

            managed_process.status = ProcessStatus::Stopped;
        } else {
            return Err(anyhow!("Process with id '{}' not found", id));
        }

        Ok(())
    }

    pub async fn restart_process(&self, id: &str) -> Result<()> {
        log::info!("Restarting process: {}", id);

        let (name, config) = {
            let processes = self.processes.read().await;
            if let Some(managed_process) = processes.get(id) {
                (managed_process.name.clone(), managed_process.config.clone())
            } else {
                return Err(anyhow!("Process with id '{}' not found", id));
            }
        };

        self.stop_process(id).await?;
        self.remove_process(id).await?;
        self.start_process(id.to_string(), name, config).await?;

        Ok(())
    }

    pub async fn get_process_status(&self, id: &str) -> Option<ProcessStatus> {
        let processes = self.processes.read().await;
        processes.get(id).map(|p| p.status.clone())
    }

    pub async fn list_processes(&self) -> Vec<(String, String, ProcessStatus)> {
        let processes = self.processes.read().await;
        processes.values()
            .map(|p| (p.id.clone(), p.name.clone(), p.status.clone()))
            .collect()
    }

    pub async fn is_process_running(&self, id: &str) -> bool {
        matches!(self.get_process_status(id).await, Some(ProcessStatus::Running))
    }

    async fn update_process_status(&self, id: &str, status: ProcessStatus) -> Result<()> {
        let mut processes = self.processes.write().await;
        if let Some(managed_process) = processes.get_mut(id) {
            managed_process.status = status;
            Ok(())
        } else {
            Err(anyhow!("Process with id '{}' not found", id))
        }
    }

    async fn remove_process(&self, id: &str) -> Result<()> {
        let mut processes = self.processes.write().await;
        processes.remove(id);
        Ok(())
    }

    async fn start_health_monitoring(&self, id: String, health_url: String, timeout_seconds: u64) {
        let manager = self.clone();
        
        tokio::spawn(async move {
            let start_time = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(timeout_seconds);
            
            loop {
                if start_time.elapsed() > timeout {
                    log::error!("Process '{}' failed to start within timeout", id);
                    let _ = manager.update_process_status(&id, ProcessStatus::Failed("Startup timeout".to_string())).await;
                    break;
                }

                // Check health
                match reqwest::get(&health_url).await {
                    Ok(response) if response.status().is_success() => {
                        log::info!("Process '{}' is healthy", id);
                        let _ = manager.update_process_status(&id, ProcessStatus::Running).await;
                        break;
                    }
                    Ok(response) => {
                        log::debug!("Process '{}' health check failed: {}", id, response.status());
                    }
                    Err(e) => {
                        log::debug!("Process '{}' health check error: {}", id, e);
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    }
}

impl Clone for ProcessManager {
    fn clone(&self) -> Self {
        Self {
            processes: Arc::clone(&self.processes),
        }
    }
}

// Qdrant-specific helper
impl ProcessManager {
    pub async fn start_qdrant(&self, binary_path: PathBuf, data_dir: PathBuf, port: u16) -> Result<()> {
        let config = ProcessConfig {
            binary_path,
            args: vec![
                "--config-path".to_string(),
                data_dir.join("config.yaml").to_string_lossy().to_string(),
            ],
            env_vars: [
                ("QDRANT__SERVICE__HTTP_PORT".to_string(), port.to_string()),
                ("QDRANT__STORAGE__STORAGE_PATH".to_string(), data_dir.to_string_lossy().to_string()),
            ].iter().cloned().collect(),
            working_dir: Some(data_dir),
            auto_restart: true,
            health_check_url: Some(format!("http://localhost:{}/health", port)),
            startup_timeout_seconds: 30,
        };

        self.start_process("qdrant".to_string(), "Qdrant Vector Database".to_string(), config).await
    }

    pub async fn stop_qdrant(&self) -> Result<()> {
        self.stop_process("qdrant").await
    }

    pub async fn is_qdrant_running(&self) -> bool {
        self.is_process_running("qdrant").await
    }
}