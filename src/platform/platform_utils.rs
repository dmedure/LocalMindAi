use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::env;

/// Get the platform-specific data directory for LocalMind
pub fn get_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LocalMind")
}

/// Get the platform-specific config directory for LocalMind
pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LocalMind")
}

/// Get the platform-specific cache directory for LocalMind
pub fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LocalMind")
}

/// Get the platform-specific logs directory for LocalMind
pub fn get_logs_dir() -> PathBuf {
    get_data_dir().join("logs")
}

/// Get the platform-specific models directory for LocalMind
pub fn get_models_dir() -> PathBuf {
    get_data_dir().join("models")
}

/// Get the platform-specific binaries directory for LocalMind
pub fn get_binaries_dir() -> PathBuf {
    get_data_dir().join("binaries")
}

/// Ensure all required directories exist
pub async fn ensure_directories() -> Result<()> {
    let dirs = [
        get_data_dir(),
        get_config_dir(),
        get_cache_dir(),
        get_logs_dir(),
        get_models_dir(),
        get_binaries_dir(),
    ];

    for dir in &dirs {
        tokio::fs::create_dir_all(dir).await
            .map_err(|e| anyhow!("Failed to create directory {:?}: {}", dir, e))?;
    }

    log::info!("All required directories ensured");
    Ok(())
}

/// Get the correct executable extension for the current platform
pub fn get_executable_extension() -> &'static str {
    if cfg!(windows) {
        ".exe"
    } else {
        ""
    }
}

/// Get the current platform identifier
pub fn get_platform_identifier() -> String {
    format!("{}-{}", env::consts::OS, env::consts::ARCH)
}

/// Check if we're running in development mode
pub fn is_development_mode() -> bool {
    cfg!(debug_assertions) || env::var("LOCALMIND_DEV").is_ok()
}

/// Get the default configuration based on platform capabilities
pub fn get_platform_defaults() -> PlatformDefaults {
    let total_memory = get_total_system_memory();
    let cpu_cores = num_cpus::get();
    
    // Adjust defaults based on system capabilities
    let (default_model, max_models) = if total_memory >= 16 * 1024 * 1024 * 1024 { // 16GB
        ("Mistral7B", 2)
    } else if total_memory >= 8 * 1024 * 1024 * 1024 { // 8GB
        ("TinyLlama", 1)
    } else {
        ("TinyLlama", 1)
    };

    let worker_threads = (cpu_cores / 2).max(1).min(8);

    PlatformDefaults {
        default_model: default_model.to_string(),
        max_concurrent_models: max_models,
        worker_threads,
        memory_limit_mb: (total_memory / 1024 / 1024 / 4) as u64, // Use 1/4 of system memory
        enable_gpu: detect_gpu_support(),
        cache_size_mb: 512,
    }
}

#[derive(Debug, Clone)]
pub struct PlatformDefaults {
    pub default_model: String,
    pub max_concurrent_models: usize,
    pub worker_threads: usize,
    pub memory_limit_mb: u64,
    pub enable_gpu: bool,
    pub cache_size_mb: u64,
}

fn get_total_system_memory() -> u64 {
    // This is a simplified version - in practice you'd use sysinfo
    use sysinfo::{System, SystemExt};
    let system = System::new_all();
    system.total_memory()
}

fn detect_gpu_support() -> bool {
    // Placeholder for GPU detection
    // In a real implementation, you'd check for CUDA, Metal, or OpenCL
    false
}

/// Platform-specific path handling
pub struct PathHelpers;

impl PathHelpers {
    /// Convert a path to be platform-appropriate
    pub fn normalize_path(path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();
        
        for component in path.components() {
            match component {
                std::path::Component::Normal(os_str) => {
                    if let Some(str_path) = os_str.to_str() {
                        // Replace forbidden characters on Windows
                        if cfg!(windows) {
                            let clean = str_path
                                .replace('<', "_")
                                .replace('>', "_")
                                .replace(':', "_")
                                .replace('"', "_")
                                .replace('|', "_")
                                .replace('?', "_")
                                .replace('*', "_");
                            normalized.push(clean);
                        } else {
                            normalized.push(str_path);
                        }
                    }
                }
                _ => normalized.push(component),
            }
        }
        
        normalized
    }

    /// Check if a path is valid for the current platform
    pub fn is_valid_path(path: &Path) -> bool {
        if cfg!(windows) {
            // Windows path validation
            if let Some(path_str) = path.to_str() {
                !path_str.chars().any(|c| "<>:\"|?*".contains(c)) &&
                path_str.len() <= 260 // MAX_PATH on Windows
            } else {
                false
            }
        } else {
            // Unix-like systems are more permissive
            path.to_str().is_some()
        }
    }

    /// Get a safe filename for the current platform
    pub fn safe_filename(name: &str) -> String {
        if cfg!(windows) {
            name.chars()
                .map(|c| if "<>:\"|?*".contains(c) { '_' } else { c })
                .collect::<String>()
                .trim_end_matches('.')
                .to_string()
        } else {
            name.replace('/', "_")
        }
    }
}

/// Environment variable helpers
pub struct EnvHelpers;

impl EnvHelpers {
    /// Get an environment variable with a default value
    pub fn get_var_or_default(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get a boolean environment variable
    pub fn get_bool_var(key: &str, default: bool) -> bool {
        env::var(key)
            .map(|val| val.to_lowercase() == "true" || val == "1")
            .unwrap_or(default)
    }

    /// Get a numeric environment variable
    pub fn get_numeric_var<T>(key: &str, default: T) -> T
    where
        T: std::str::FromStr + Copy,
    {
        env::var(key)
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(default)
    }

    /// Set platform-specific environment variables for child processes
    pub fn setup_child_env() -> Vec<(String, String)> {
        let mut env_vars = Vec::new();

        // Ensure UTF-8 encoding
        if cfg!(windows) {
            env_vars.push(("PYTHONIOENCODING".to_string(), "utf-8".to_string()));
        }

        // Set locale for consistent behavior
        env_vars.push(("LC_ALL".to_string(), "C.UTF-8".to_string()));
        
        // Disable telemetry for child processes
        env_vars.push(("DO_NOT_TRACK".to_string(), "1".to_string()));

        env_vars
    }
}

/// System capability detection
pub struct CapabilityDetector;

impl CapabilityDetector {
    /// Check if the system can run a specific model
    pub fn can_run_model(model_name: &str) -> Result<bool> {
        let required_memory = match model_name {
            "TinyLlama" => 1_000_000_000,  // 1GB
            "Mistral7B" => 4_000_000_000,  // 4GB
            _ => 2_000_000_000,            // 2GB default
        };

        let system_memory = get_total_system_memory();
        let available_memory = system_memory.saturating_sub(2_000_000_000); // Reserve 2GB for system

        Ok(available_memory >= required_memory)
    }

    /// Check if GPU acceleration is available
    pub fn has_gpu_acceleration() -> bool {
        // Check for NVIDIA GPUs (CUDA)
        if std::process::Command::new("nvidia-smi")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        // Check for AMD GPUs (ROCm)
        if std::process::Command::new("rocm-smi")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        // Check for Apple Metal (macOS)
        if cfg!(target_os = "macos") {
            // On macOS, Metal is generally available on modern systems
            return true;
        }

        false
    }

    /// Get recommended thread count for the system
    pub fn get_recommended_threads() -> usize {
        let cpu_count = num_cpus::get();
        
        // Use half the CPU cores, with a minimum of 1 and maximum of 16
        (cpu_count / 2).max(1).min(16)
    }
}