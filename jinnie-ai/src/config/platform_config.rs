use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::utils::error::{LocalMindError, Result};

/// Platform types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

/// Architecture types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Unknown,
}

/// Platform-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub platform: Platform,
    pub architecture: Architecture,
    pub paths: PlatformPaths,
    pub capabilities: PlatformCapabilities,
    pub defaults: PlatformDefaults,
}

/// Platform-specific paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPaths {
    pub home_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub models_dir: PathBuf,
    pub logs_dir: PathBuf,
}

/// Platform capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    pub gpu_acceleration: bool,
    pub metal_performance_shaders: bool, // macOS
    pub cuda_support: bool,              // NVIDIA
    pub rocm_support: bool,              // AMD
    pub opencl_support: bool,
    pub max_memory_gb: Option<u64>,
    pub cpu_cores: usize,
    pub supports_memory_mapping: bool,
}

/// Platform-specific defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDefaults {
    pub default_threads: usize,
    pub default_gpu_layers: u32,
    pub memory_allocation_strategy: MemoryStrategy,
    pub file_watcher_enabled: bool,
    pub system_tray_enabled: bool,
}

/// Memory allocation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryStrategy {
    Conservative, // Use less memory, slower
    Balanced,     // Balance memory and speed
    Aggressive,   // Use more memory, faster
}

/// Detect current platform
pub fn detect_platform() -> Platform {
    match std::env::consts::OS {
        "windows" => Platform::Windows,
        "macos" => Platform::MacOS,
        "linux" => Platform::Linux,
        _ => Platform::Unknown,
    }
}

/// Detect current architecture
pub fn detect_architecture() -> Architecture {
    match std::env::consts::ARCH {
        "x86_64" => Architecture::X86_64,
        "aarch64" => Architecture::Aarch64,
        _ => Architecture::Unknown,
    }
}

impl PlatformConfig {
    /// Create platform configuration for current system
    pub fn detect() -> Result<Self> {
        let platform = detect_platform();
        let architecture = detect_architecture();
        let paths = PlatformPaths::detect(&platform)?;
        let capabilities = PlatformCapabilities::detect(&platform, &architecture)?;
        let defaults = PlatformDefaults::for_platform(&platform, &capabilities);

        Ok(Self {
            platform,
            architecture,
            paths,
            capabilities,
            defaults,
        })
    }

    /// Get recommended model configuration for this platform
    pub fn get_model_recommendations(&self) -> ModelRecommendations {
        let max_memory_gb = self.capabilities.max_memory_gb.unwrap_or(8);
        let cpu_cores = self.capabilities.cpu_cores;

        ModelRecommendations {
            tinyllama_enabled: true, // Always available
            tinyllama_threads: (cpu_cores / 2).max(2).min(8),
            tinyllama_gpu_layers: if self.capabilities.gpu_acceleration { 
                self.defaults.default_gpu_layers / 2 
            } else { 
                0 
            },
            
            mistral7b_enabled: max_memory_gb >= 6,
            mistral7b_threads: cpu_cores.max(4).min(16),
            mistral7b_gpu_layers: if self.capabilities.gpu_acceleration && max_memory_gb >= 8 {
                self.defaults.default_gpu_layers
            } else {
                0
            },
            
            memory_strategy: if max_memory_gb >= 16 {
                MemoryStrategy::Aggressive
            } else if max_memory_gb >= 8 {
                MemoryStrategy::Balanced
            } else {
                MemoryStrategy::Conservative
            },
        }
    }

    /// Check if the platform supports a specific feature
    pub fn supports_feature(&self, feature: PlatformFeature) -> bool {
        match feature {
            PlatformFeature::SystemTray => self.defaults.system_tray_enabled,
            PlatformFeature::FileWatcher => self.defaults.file_watcher_enabled,
            PlatformFeature::GpuAcceleration => self.capabilities.gpu_acceleration,
            PlatformFeature::MemoryMapping => self.capabilities.supports_memory_mapping,
            PlatformFeature::MetalPerformanceShaders => self.capabilities.metal_performance_shaders,
            PlatformFeature::CudaSupport => self.capabilities.cuda_support,
            PlatformFeature::RocmSupport => self.capabilities.rocm_support,
        }
    }

    /// Get platform-specific binary name
    pub fn get_binary_name(&self, base_name: &str) -> String {
        match self.platform {
            Platform::Windows => format!("{}.exe", base_name),
            _ => base_name.to_string(),
        }
    }

    /// Get platform-specific library extension
    pub fn get_library_extension(&self) -> &'static str {
        match self.platform {
            Platform::Windows => "dll",
            Platform::MacOS => "dylib",
            Platform::Linux => "so",
            Platform::Unknown => "so",
        }
    }
}

impl PlatformPaths {
    /// Detect platform-specific paths
    pub fn detect(platform: &Platform) -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| LocalMindError::Configuration("Cannot determine home directory".to_string()))?;

        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| home_dir.join(".config"))
            .join("localmind");

        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| home_dir.join(".local/share"))
            .join("localmind");

        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| home_dir.join(".cache"))
            .join("localmind");

        let temp_dir = std::env::temp_dir().join("localmind");

        let models_dir = data_dir.join("models");
        let logs_dir = data_dir.join("logs");

        Ok(Self {
            home_dir,
            config_dir,
            data_dir,
            cache_dir,
            temp_dir,
            models_dir,
            logs_dir,
        })
    }

    /// Ensure all directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        let dirs_to_create = [
            &self.config_dir,
            &self.data_dir,
            &self.cache_dir,
            &self.temp_dir,
            &self.models_dir,
            &self.logs_dir,
        ];

        for dir in &dirs_to_create {
            std::fs::create_dir_all(dir)
                .map_err(|e| LocalMindError::Configuration(
                    format!("Failed to create directory {}: {}", dir.display(), e)
                ))?;
        }

        Ok(())
    }
}

impl PlatformCapabilities {
    /// Detect platform capabilities
    pub fn detect(platform: &Platform, architecture: &Architecture) -> Result<Self> {
        let cpu_cores = num_cpus::get();
        let max_memory_gb = Self::detect_memory();
        
        // Basic GPU detection (simplified)
        let gpu_acceleration = Self::detect_gpu_support(platform);
        let metal_performance_shaders = matches!(platform, Platform::MacOS) && 
                                       matches!(architecture, Architecture::Aarch64);
        
        // CUDA/ROCm detection would require more sophisticated checks
        let cuda_support = false; // TODO: Implement proper CUDA detection
        let rocm_support = false; // TODO: Implement proper ROCm detection
        let opencl_support = false; // TODO: Implement proper OpenCL detection

        let supports_memory_mapping = !matches!(platform, Platform::Unknown);

        Ok(Self {
            gpu_acceleration,
            metal_performance_shaders,
            cuda_support,
            rocm_support,
            opencl_support,
            max_memory_gb,
            cpu_cores,
            supports_memory_mapping,
        })
    }

    /// Detect available system memory
    fn detect_memory() -> Option<u64> {
        // This is a simplified implementation
        // In a real implementation, you'd use system-specific APIs
        if let Ok(info) = sys_info::mem_info() {
            Some(info.total / 1024 / 1024 / 1024) // Convert to GB
        } else {
            None
        }
    }

    /// Basic GPU support detection
    fn detect_gpu_support(platform: &Platform) -> bool {
        match platform {
            Platform::MacOS => true, // Metal support
            Platform::Windows | Platform::Linux => {
                // In a real implementation, check for NVIDIA/AMD drivers
                false // Conservative default
            },
            Platform::Unknown => false,
        }
    }
}

impl PlatformDefaults {
    /// Create platform-specific defaults
    pub fn for_platform(platform: &Platform, capabilities: &PlatformCapabilities) -> Self {
        let default_threads = (capabilities.cpu_cores / 2).max(2).min(8);
        let default_gpu_layers = if capabilities.gpu_acceleration { 35 } else { 0 };

        let memory_strategy = match capabilities.max_memory_gb.unwrap_or(8) {
            0..=4 => MemoryStrategy::Conservative,
            5..=12 => MemoryStrategy::Balanced,
            _ => MemoryStrategy::Aggressive,
        };

        let file_watcher_enabled = !matches!(platform, Platform::Unknown);
        let system_tray_enabled = matches!(platform, Platform::Windows | Platform::Linux);

        Self {
            default_threads,
            default_gpu_layers,
            memory_allocation_strategy: memory_strategy,
            file_watcher_enabled,
            system_tray_enabled,
        }
    }
}

/// Model recommendations for current platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendations {
    pub tinyllama_enabled: bool,
    pub tinyllama_threads: usize,
    pub tinyllama_gpu_layers: u32,
    pub mistral7b_enabled: bool,
    pub mistral7b_threads: usize,
    pub mistral7b_gpu_layers: u32,
    pub memory_strategy: MemoryStrategy,
}

/// Platform features that can be checked
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformFeature {
    SystemTray,
    FileWatcher,
    GpuAcceleration,
    MemoryMapping,
    MetalPerformanceShaders,
    CudaSupport,
    RocmSupport,
}

impl Platform {
    /// Get the display name for the platform
    pub fn display_name(&self) -> &'static str {
        match self {
            Platform::Windows => "Windows",
            Platform::MacOS => "macOS",
            Platform::Linux => "Linux",
            Platform::Unknown => "Unknown",
        }
    }

    /// Check if this is a Unix-like platform
    pub fn is_unix(&self) -> bool {
        matches!(self, Platform::MacOS | Platform::Linux)
    }

    /// Get the default shell for the platform
    pub fn default_shell(&self) -> &'static str {
        match self {
            Platform::Windows => "cmd",
            Platform::MacOS | Platform::Linux => "bash",
            Platform::Unknown => "sh",
        }
    }
}

impl Architecture {
    /// Get the display name for the architecture
    pub fn display_name(&self) -> &'static str {
        match self {
            Architecture::X86_64 => "x86_64",
            Architecture::Aarch64 => "ARM64",
            Architecture::Unknown => "Unknown",
        }
    }

    /// Check if this is an ARM architecture
    pub fn is_arm(&self) -> bool {
        matches!(self, Architecture::Aarch64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = detect_platform();
        assert_ne!(platform, Platform::Unknown); // Should detect something
        
        let architecture = detect_architecture();
        assert_ne!(architecture, Architecture::Unknown);
    }

    #[test]
    fn test_platform_config_creation() {
        let config = PlatformConfig::detect();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert!(config.capabilities.cpu_cores > 0);
    }

    #[test]
    fn test_model_recommendations() {
        let config = PlatformConfig::detect().unwrap();
        let recommendations = config.get_model_recommendations();
        
        assert!(recommendations.tinyllama_enabled); // Should always be enabled
        assert!(recommendations.tinyllama_threads >= 2);
    }

    #[test]
    fn test_platform_features() {
        let config = PlatformConfig::detect().unwrap();
        
        // These should always work
        let _file_watcher = config.supports_feature(PlatformFeature::FileWatcher);
        let _memory_mapping = config.supports_feature(PlatformFeature::MemoryMapping);
    }

    #[test]
    fn test_binary_names() {
        let windows_config = PlatformConfig {
            platform: Platform::Windows,
            architecture: Architecture::X86_64,
            paths: PlatformPaths::detect(&Platform::Windows).unwrap(),
            capabilities: PlatformCapabilities::detect(&Platform::Windows, &Architecture::X86_64).unwrap(),
            defaults: PlatformDefaults::for_platform(
                &Platform::Windows, 
                &PlatformCapabilities::detect(&Platform::Windows, &Architecture::X86_64).unwrap()
            ),
        };

        assert_eq!(windows_config.get_binary_name("test"), "test.exe");

        let linux_config = PlatformConfig {
            platform: Platform::Linux,
            architecture: Architecture::X86_64,
            paths: PlatformPaths::detect(&Platform::Linux).unwrap(),
            capabilities: PlatformCapabilities::detect(&Platform::Linux, &Architecture::X86_64).unwrap(),
            defaults: PlatformDefaults::for_platform(
                &Platform::Linux, 
                &PlatformCapabilities::detect(&Platform::Linux, &Architecture::X86_64).unwrap()
            ),
        };

        assert_eq!(linux_config.get_binary_name("test"), "test");
    }
}