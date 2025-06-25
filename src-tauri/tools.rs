use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::process::Command;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub extension: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub version: String,
    pub hostname: String,
    pub username: String,
    pub home_dir: String,
    pub documents_dir: String,
}

pub struct SystemTools;

impl SystemTools {
    pub fn new() -> Self {
        SystemTools
    }
    
    // File System Operations
    pub fn list_files(&self, directory: &str) -> Result<Vec<FileInfo>> {
        let path = std::path::Path::new(directory);
        
        if !path.exists() {
            return Err(anyhow!("Directory does not exist: {}", directory));
        }
        
        if !path.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", directory));
        }
        
        let mut files = Vec::new();
        
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let file_path = entry.path();
            
            let file_info = FileInfo {
                path: file_path.to_string_lossy().to_string(),
                name: entry.file_name().to_string_lossy().to_string(),
                size: metadata.len(),
                is_directory: metadata.is_dir(),
                last_modified: metadata.modified()?.into(),
                extension: file_path.extension()
                    .map(|e| e.to_string_lossy().to_string()),
            };
            
            files.push(file_info);
        }
        
        files.sort_by(|a, b| {
            // Directories first, then by name
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        Ok(files)
    }
    
    pub fn search_files(&self, directory: &str, pattern: &str) -> Result<Vec<FileInfo>> {
        let files = self.list_files(directory)?;
        let pattern_lower = pattern.to_lowercase();
        
        let filtered: Vec<FileInfo> = files
            .into_iter()
            .filter(|file| {
                file.name.to_lowercase().contains(&pattern_lower)
            })
            .collect();
        
        Ok(filtered)
    }
    
    pub fn get_file_content(&self, file_path: &str) -> Result<String> {
        let path = std::path::Path::new(file_path);
        
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }
        
        // Check file size to avoid loading huge files
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > 10_000_000 { // 10MB limit
            return Err(anyhow!("File too large (>10MB): {}", file_path));
        }
        
        let content = std::fs::read_to_string(path)?;
        Ok(content)
    }
    
    // System Information
    pub fn get_system_info(&self) -> Result<SystemInfo> {
        let info = SystemInfo {
            os: std::env::consts::OS.to_string(),
            version: get_os_version(),
            hostname: get_hostname(),
            username: get_username(),
            home_dir: dirs::home_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            documents_dir: dirs::document_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        };
        
        Ok(info)
    }
    
    // Application Control
    pub fn open_file(&self, file_path: &str) -> Result<()> {
        let path = std::path::Path::new(file_path);
        
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }
        
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/C", "start", "", file_path])
                .spawn()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(file_path)
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(file_path)
                .spawn()?;
        }
        
        Ok(())
    }
    
    pub fn open_directory(&self, directory: &str) -> Result<()> {
        let path = std::path::Path::new(directory);
        
        if !path.exists() || !path.is_dir() {
            return Err(anyhow!("Directory does not exist: {}", directory));
        }
        
        self.open_file(directory)
    }
    
    // Process Management
    pub fn list_running_applications(&self) -> Result<Vec<String>> {
        let mut apps = Vec::new();
        
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("tasklist")
                .args(["/fo", "csv", "/nh"])
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(name) = line.split(',').next() {
                    let clean_name = name.trim_matches('"');
                    if !clean_name.is_empty() {
                        apps.push(clean_name.to_string());
                    }
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("ps")
                .args(["-axo", "comm"])
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) { // Skip header
                let app_name = line.trim();
                if !app_name.is_empty() {
                    apps.push(app_name.to_string());
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("ps")
                .args(["-eo", "comm"])
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) { // Skip header
                let app_name = line.trim();
                if !app_name.is_empty() {
                    apps.push(app_name.to_string());
                }
            }
        }
        
        // Remove duplicates and sort
        apps.sort();
        apps.dedup();
        
        Ok(apps)
    }
    
    // Network Operations
    pub fn check_internet_connection(&self) -> Result<bool> {
        let output = Command::new("ping")
            .args(["-c", "1", "8.8.8.8"]) // Google DNS
            .output();
        
        match output {
            Ok(result) => Ok(result.status.success()),
            Err(_) => Ok(false),
        }
    }
}

// Helper functions
fn get_os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        match Command::new("ver").output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
            Err(_) => "Unknown".to_string(),
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        match Command::new("sw_vers").args(["-productVersion"]).output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
            Err(_) => "Unknown".to_string(),
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        match std::fs::read_to_string("/etc/os-release") {
            Ok(content) => {
                for line in content.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        return line.split('=').nth(1)
                            .unwrap_or("Unknown")
                            .trim_matches('"')
                            .to_string();
                    }
                }
                "Unknown".to_string()
            }
            Err(_) => "Unknown".to_string(),
        }
    }
}

fn get_hostname() -> String {
    match Command::new("hostname").output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => "Unknown".to_string(),
    }
}

fn get_username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "Unknown".to_string())
}