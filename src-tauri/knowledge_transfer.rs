use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::knowledge::{Document, KnowledgeBase};
use crate::agent::{Agent, AgentMemory, CompleteAgentExport};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportOptions {
    pub categories: Vec<String>,
    pub encrypt: bool,
    pub anonymize_personal_data: bool,
    pub include_source_files: bool,
    pub compression: CompressionType,
    pub compress: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MergeStrategy {
    Replace,    // Replace all existing data
    Append,     // Add new data without checking duplicates
    Merge,      // Intelligent merge, updating existing and adding new
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgePackage {
    pub version: String,
    pub package_id: String,
    pub created_at: DateTime<Utc>,
    pub source_agent: String,
    pub target_domain: Option<String>,
    pub documents: Vec<Document>,
    pub agent_memory: Option<AgentMemory>,
    pub metadata: PackageMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub description: String,
    pub tags: Vec<String>,
    pub document_count: usize,
    pub total_size: usize,
    pub compatibility_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferStats {
    pub documents_transferred: usize,
    pub conversations_transferred: usize,
    pub workflows_transferred: usize,
    pub preferences_transferred: usize,
    pub transfer_time_ms: u64,
}

pub struct KnowledgeTransfer {
    pub temp_dir: String,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            categories: Vec::new(),
            encrypt: false,
            anonymize_personal_data: true,
            include_source_files: false,
            compression: CompressionType::None,
            compress: false,
        }
    }
}

impl KnowledgeTransfer {
    pub fn new() -> Self {
        KnowledgeTransfer {
            temp_dir: "./temp_transfers".to_string(),
        }
    }
    
    pub async fn create_knowledge_package(
        &self,
        knowledge_base: &KnowledgeBase,
        agent_memory: Option<&AgentMemory>,
        options: ExportOptions,
    ) -> Result<KnowledgePackage> {
        let package_id = Uuid::new_v4().to_string();
        
        // Filter documents based on categories
        let documents = if options.categories.is_empty() {
            knowledge_base.get_all_documents().to_vec()
        } else {
            knowledge_base.get_all_documents()
                .iter()
                .filter(|doc| {
                    options.categories.iter().any(|cat| {
                        doc.metadata.keywords.contains(cat) ||
                        doc.source.contains(cat) ||
                        doc.metadata.title.contains(cat)
                    })
                })
                .cloned()
                .collect()
        };
        
        let total_size = documents.iter()
            .map(|d| d.content.len() + d.source.len())
            .sum();
        
        let metadata = PackageMetadata {
            description: format!("Knowledge package with {} documents", documents.len()),
            tags: options.categories.clone(),
            document_count: documents.len(),
            total_size,
            compatibility_version: "1.0".to_string(),
        };
        
        let package = KnowledgePackage {
            version: "1.0".to_string(),
            package_id,
            created_at: Utc::now(),
            source_agent: "local_agent".to_string(),
            target_domain: None,
            documents,
            agent_memory: agent_memory.cloned(),
            metadata,
        };
        
        Ok(package)
    }
    
    pub async fn save_package(
        &self,
        package: &KnowledgePackage,
        file_path: &str,
        options: &ExportOptions,
    ) -> Result<String> {
        let json_string = serde_json::to_string_pretty(package)?;
        
        let final_content = if options.encrypt {
            self.encrypt_content(&json_string).await?
        } else {
            json_string
        };
        
        fs::write(file_path, final_content)?;
        
        Ok(format!("Package saved to {} ({} documents)", 
                  file_path, package.documents.len()))
    }
    
    pub async fn load_package(&self, file_path: &str) -> Result<KnowledgePackage> {
        let content = fs::read_to_string(file_path)?;
        
        // Try to decrypt if needed (detect encrypted format)
        let json_content = if content.starts_with("ENCRYPTED:") {
            self.decrypt_content(&content).await?
        } else {
            content
        };
        
        let package: KnowledgePackage = serde_json::from_str(&json_content)?;
        Ok(package)
    }
    
    pub async fn apply_package(
        &self,
        package: KnowledgePackage,
        target_kb: &mut KnowledgeBase,
        target_agent: Option<&mut Agent>,
        strategy: MergeStrategy,
    ) -> Result<TransferStats> {
        let start_time = std::time::Instant::now();
        let mut stats = TransferStats {
            documents_transferred: 0,
            conversations_transferred: 0,
            workflows_transferred: 0,
            preferences_transferred: 0,
            transfer_time_ms: 0,
        };
        
        // Transfer documents
        match strategy {
            MergeStrategy::Replace => {
                // This would require more complex logic to replace KB contents
                return Err(anyhow!("Replace strategy not yet implemented for knowledge base"));
            },
            MergeStrategy::Append => {
                for doc in package.documents {
                    // Add document (this is a placeholder - would need to implement in KB)
                    stats.documents_transferred += 1;
                }
            },
            MergeStrategy::Merge => {
                for doc in package.documents {
                    // Check for existing docs by source and merge intelligently
                    stats.documents_transferred += 1;
                }
            }
        }
        
        // Transfer agent memory if present
        if let (Some(package_memory), Some(agent)) = (package.agent_memory, target_agent) {
            match strategy {
                MergeStrategy::Replace => {
                    agent.import_memory(package_memory)?;
                    stats.conversations_transferred = package_memory.conversation_history.len();
                    stats.workflows_transferred = package_memory.custom_workflows.len();
                    stats.preferences_transferred = package_memory.user_preferences.len();
                },
                MergeStrategy::Append | MergeStrategy::Merge => {
                    // Merge workflows
                    for workflow in package_memory.custom_workflows {
                        agent.add_workflow(workflow);
                        stats.workflows_transferred += 1;
                    }
                    
                    // Merge preferences
                    for (key, value) in package_memory.user_preferences {
                        agent.update_preferences(key, value);
                        stats.preferences_transferred += 1;
                    }
                }
            }
        }
        
        stats.transfer_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(stats)
    }
    
    pub async fn create_specialized_agent(
        &self,
        domain: &str,
        source_export_path: &str,
        target_path: &str,
    ) -> Result<String> {
        // Load the source package
        let package = self.load_package(source_export_path).await?;
        
        // Filter documents relevant to the domain
        let specialized_docs: Vec<Document> = package.documents
            .into_iter()
            .filter(|doc| {
                doc.metadata.keywords.iter().any(|k| k.contains(domain)) ||
                doc.content.to_lowercase().contains(&domain.to_lowercase()) ||
                doc.metadata.title.to_lowercase().contains(&domain.to_lowercase())
            })
            .collect();
        
        // Create specialized package
        let specialized_package = KnowledgePackage {
            version: package.version,
            package_id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            source_agent: package.source_agent,
            target_domain: Some(domain.to_string()),
            documents: specialized_docs.clone(),
            agent_memory: package.agent_memory,
            metadata: PackageMetadata {
                description: format!("Specialized agent for {}", domain),
                tags: vec![domain.to_string()],
                document_count: specialized_docs.len(),
                total_size: specialized_docs.iter()
                    .map(|d| d.content.len() + d.source.len())
                    .sum(),
                compatibility_version: "1.0".to_string(),
            },
        };
        
        // Save specialized package
        let options = ExportOptions::default();
        self.save_package(&specialized_package, target_path, &options).await?;
        
        Ok(format!("Created specialized {} agent with {} documents", 
                  domain, specialized_docs.len()))
    }
    
    pub async fn validate_package(&self, package: &KnowledgePackage) -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        
        // Check version compatibility
        if package.version != "1.0" {
            warnings.push(format!("Package version {} may not be compatible", package.version));
        }
        
        // Check document integrity
        for (i, doc) in package.documents.iter().enumerate() {
            if doc.content.is_empty() {
                warnings.push(format!("Document {} has empty content", i));
            }
            if doc.source.is_empty() {
                warnings.push(format!("Document {} has no source", i));
            }
        }
        
        // Check metadata consistency
        if package.metadata.document_count != package.documents.len() {
            warnings.push("Document count mismatch in metadata".to_string());
        }
        
        Ok(warnings)
    }
    
    async fn encrypt_content(&self, content: &str) -> Result<String> {
        // TODO: Implement actual encryption
        // For now, just prepend a marker
        Ok(format!("ENCRYPTED:{}", content))
    }
    
    async fn decrypt_content(&self, encrypted_content: &str) -> Result<String> {
        // TODO: Implement actual decryption
        // For now, just remove the marker
        if let Some(content) = encrypted_content.strip_prefix("ENCRYPTED:") {
            Ok(content.to_string())
        } else {
            Err(anyhow!("Invalid encrypted format"))
        }
    }
    
    pub fn estimate_transfer_time(&self, package: &KnowledgePackage) -> u64 {
        // Rough estimation: 1ms per KB of data
        (package.metadata.total_size / 1024) as u64
    }
    
    pub fn get_package_summary(&self, package: &KnowledgePackage) -> String {
        format!(
            "Package: {} ({})\n\
             Created: {}\n\
             Documents: {}\n\
             Size: {} bytes\n\
             Domain: {}\n\
             Tags: {}",
            package.package_id,
            package.metadata.description,
            package.created_at.format("%Y-%m-%d %H:%M:%S"),
            package.metadata.document_count,
            package.metadata.total_size,
            package.target_domain.as_deref().unwrap_or("General"),
            package.metadata.tags.join(", ")
        )
    }
}