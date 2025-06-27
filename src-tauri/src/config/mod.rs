pub mod app_config;
pub mod platform;

pub use app_config::{AppConfig, LLMConfig, MemoryConfig, VectorConfig};
pub use platform::{PlatformConfig, detect_platform};