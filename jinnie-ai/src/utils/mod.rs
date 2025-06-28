pub mod binary_manager;
pub mod process_manager;
pub mod resource_monitor;
pub mod platform_utils;
pub mod error;

pub use binary_manager::*;
pub use process_manager::*;
pub use resource_monitor::*;
pub use platform_utils::*;
pub use error::{LocalMindError, Result};

use anyhow::Result as AnyhowResult;

/// Initialize platform-specific components
pub async fn init() -> AnyhowResult<()> {
    log::info!("Initializing platform components...");
    
    // Initialize resource monitoring
    let resource_monitor = ResourceMonitor::new();
    resource_monitor.start_monitoring().await?;
    
    // Check for required binaries
    let binary_manager = BinaryManager::new();
    binary_manager.verify_dependencies().await?;
    
    Ok(())
}