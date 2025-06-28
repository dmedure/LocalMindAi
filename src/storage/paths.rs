use std::path::PathBuf;
use crate::utils::error::{LocalMindError, Result};

/// Get the application data directory
pub fn get_data_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("LocalMind");
    path
}

/// Ensure the data directory exists, creating it if necessary
pub fn ensure_data_dir() -> Result<PathBuf> {
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| LocalMindError::Storage(format!("Failed to create data directory: {}", e)))?;
    Ok(data_dir)
}

/// Get the path to the agents file
pub fn get_agents_file_path() -> Result<PathBuf> {
    let data_dir = ensure_data_dir()?;
    Ok(data_dir.join("agents.json"))
}

/// Get the path to the messages file
pub fn get_messages_file_path() -> Result<PathBuf> {
    let data_dir = ensure_data_dir()?;
    Ok(data_dir.join("messages.json"))
}

/// Get the path to the documents file
pub fn get_documents_file_path() -> Result<PathBuf> {
    let data_dir = ensure_data_dir()?;
    Ok(data_dir.join("documents.json"))
}

/// Get the path to the configuration file
pub fn get_config_file_path() -> Result<PathBuf> {
    let data_dir = ensure_data_dir()?;
    Ok(data_dir.join("config.toml"))
}

/// Get the path to the logs directory
pub fn get_logs_dir() -> Result<PathBuf> {
    let data_dir = ensure_data_dir()?;
    let logs_dir = data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| LocalMindError::Storage(format!("Failed to create logs directory: {}", e)))?;
    Ok(logs_dir)
}

/// Get the path to the models directory
pub fn get_models_dir() -> Result<PathBuf> {
    let data_dir = ensure_data_dir()?;
    let models_dir = data_dir.join("models");
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| LocalMindError::Storage(format!("Failed to create models directory: {}", e)))?;
    Ok(models_dir)
}

/// Get the path to the exports directory
pub fn get_exports_dir() -> Result<PathBuf> {
    let data_dir = ensure_data_dir()?;
    let exports_dir = data_dir.join("exports");
    std::fs::create_dir_all(&exports_dir)
        .map_err(|e| LocalMindError::Storage(format!("Failed to create exports directory: {}", e)))?;
    Ok(exports_dir)
}

/// Check if the application is running for the first time
pub fn is_first_run() -> bool {
    !get_data_dir().exists()
}

/// Get total disk space used by the application
pub fn get_total_disk_usage() -> Result<u64> {
    let data_dir = get_data_dir();
    if !data_dir.exists() {
        return Ok(0);
    }

    fn dir_size(path: &std::path::Path) -> std::io::Result<u64> {
        let mut size = 0;
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                if metadata.is_dir() {
                    size += dir_size(&entry.path())?;
                } else {
                    size += metadata.len();
                }
            }
        }
        Ok(size)
    }

    dir_size(&data_dir).map_err(|e| LocalMindError::FileSystem(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_data_dir() {
        let data_dir = get_data_dir();
        assert!(data_dir.to_string_lossy().contains("LocalMind"));
    }

    #[test]
    fn test_ensure_data_dir() {
        // This test will actually create the directory
        let result = ensure_data_dir();
        assert!(result.is_ok());
        
        let data_dir = result.unwrap();
        assert!(data_dir.exists());
        assert!(data_dir.is_dir());
    }
}