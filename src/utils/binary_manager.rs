use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;

#[derive(Debug, Clone)]
pub struct BinaryManager {
    binaries: HashMap<BinaryType, BinaryInfo>,
    base_path: PathBuf,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum BinaryType {
    Qdrant,
    LlamaCpp,
}

#[derive(Debug, Clone)]
pub struct BinaryInfo {
    pub name: String,
    pub version: String,
    pub download_urls: HashMap<Platform, String>,
    pub checksums: HashMap<Platform, String>,
    pub executable_name: String,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Platform {
    WindowsX64,
    MacosX64,
    MacosArm64,
    LinuxX64,
    LinuxArm64,
}

impl BinaryManager {
    pub fn new() -> Self {
        let base_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("LocalMind")
            .join("binaries");

        let mut binaries = HashMap::new();
        
        // Qdrant binary configuration
        binaries.insert(BinaryType::Qdrant, BinaryInfo {
            name: "qdrant".to_string(),
            version: "1.7.0".to_string(),
            download_urls: [
                (Platform::WindowsX64, "https://github.com/qdrant/qdrant/releases/download/v1.7.0/qdrant-x86_64-pc-windows-msvc.zip".to_string()),
                (Platform::MacosX64, "https://github.com/qdrant/qdrant/releases/download/v1.7.0/qdrant-x86_64-apple-darwin.tar.gz".to_string()),
                (Platform::MacosArm64, "https://github.com/qdrant/qdrant/releases/download/v1.7.0/qdrant-aarch64-apple-darwin.tar.gz".to_string()),
                (Platform::LinuxX64, "https://github.com/qdrant/qdrant/releases/download/v1.7.0/qdrant-x86_64-unknown-linux-gnu.tar.gz".to_string()),
                (Platform::LinuxArm64, "https://github.com/qdrant/qdrant/releases/download/v1.7.0/qdrant-aarch64-unknown-linux-gnu.tar.gz".to_string()),
            ].iter().cloned().collect(),
            checksums: HashMap::new(), // TODO: Add actual checksums
            executable_name: if cfg!(windows) { "qdrant.exe" } else { "qdrant" }.to_string(),
        });

        Self {
            binaries,
            base_path,
        }
    }

    pub async fn ensure_binary(&self, binary_type: BinaryType) -> Result<PathBuf> {
        let binary_info = self.binaries.get(&binary_type)
            .ok_or_else(|| anyhow!("Unknown binary type: {:?}", binary_type))?;

        let platform = self.detect_platform()?;
        let binary_dir = self.base_path.join(format!("{}-{}", binary_info.name, binary_info.version));
        let binary_path = binary_dir.join(&binary_info.executable_name);

        // Check if binary already exists and is executable
        if binary_path.exists() && self.is_executable(&binary_path).await? {
            log::info!("Binary already exists: {:?}", binary_path);
            return Ok(binary_path);
        }

        // Download and install binary
        log::info!("Downloading binary: {:?}", binary_type);
        self.download_and_install(binary_type, platform, &binary_dir).await?;

        Ok(binary_path)
    }

    pub async fn verify_dependencies(&self) -> Result<()> {
        log::info!("Verifying binary dependencies...");
        
        for binary_type in [BinaryType::Qdrant] {
            match self.ensure_binary(binary_type.clone()).await {
                Ok(path) => log::info!("✓ {:?} available at: {:?}", binary_type, path),
                Err(e) => log::warn!("✗ {:?} not available: {}", binary_type, e),
            }
        }
        
        Ok(())
    }

    fn detect_platform(&self) -> Result<Platform> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => Ok(Platform::WindowsX64),
            ("macos", "x86_64") => Ok(Platform::MacosX64),
            ("macos", "aarch64") => Ok(Platform::MacosArm64),
            ("linux", "x86_64") => Ok(Platform::LinuxX64),
            ("linux", "aarch64") => Ok(Platform::LinuxArm64),
            (os, arch) => Err(anyhow!("Unsupported platform: {} {}", os, arch)),
        }
    }

    async fn download_and_install(
        &self,
        binary_type: BinaryType,
        platform: Platform,
        install_dir: &Path,
    ) -> Result<()> {
        let binary_info = &self.binaries[&binary_type];
        let download_url = binary_info.download_urls.get(&platform)
            .ok_or_else(|| anyhow!("No download URL for platform: {:?}", platform))?;

        // Create install directory
        fs::create_dir_all(install_dir).await?;

        // Download archive
        let response = reqwest::get(download_url).await?;
        let archive_data = response.bytes().await?;

        // Extract based on file extension
        if download_url.ends_with(".zip") {
            self.extract_zip(&archive_data, install_dir).await?;
        } else if download_url.ends_with(".tar.gz") {
            self.extract_tar_gz(&archive_data, install_dir).await?;
        } else {
            return Err(anyhow!("Unsupported archive format"));
        }

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            let binary_path = install_dir.join(&binary_info.executable_name);
            if binary_path.exists() {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&binary_path).await?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&binary_path, perms).await?;
            }
        }

        Ok(())
    }

    async fn extract_zip(&self, data: &bytes::Bytes, target_dir: &Path) -> Result<()> {
        // TODO: Implement ZIP extraction
        // For now, this is a placeholder
        Err(anyhow!("ZIP extraction not yet implemented"))
    }

    async fn extract_tar_gz(&self, data: &bytes::Bytes, target_dir: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;
        use std::io::Cursor;

        let cursor = Cursor::new(data);
        let gz_decoder = GzDecoder::new(cursor);
        let mut archive = Archive::new(gz_decoder);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let target_path = target_dir.join(path);

            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            entry.unpack(target_path)?;
        }

        Ok(())
    }

    async fn is_executable(&self, path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        // Try to run with --version or --help to verify it's executable
        let output = Command::new(path)
            .arg("--version")
            .output();

        match output {
            Ok(_) => Ok(true),
            Err(_) => {
                // Try with --help
                let output = Command::new(path)
                    .arg("--help")
                    .output();
                Ok(output.is_ok())
            }
        }
    }
}