use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::path::Path;
use std::fs;
use crate::knowledge::{KnowledgeBase, Document};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportOptions {
    pub categories: Vec<String>,
    pub include_conversations: bool,
    pub include_documents: bool,
    pub encrypt: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgePackage {
    pub metadata: PackageMetadata,
    pub documents: Vec<Document>,
    pub conversations: Vec<ConversationExport>,
    pub agent_configs: Vec<AgentConfigExport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub package_type: String,
    pub categories: Vec<String>,
    pub total_documents: usize,
    pub total_conversations: usize,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationExport {
    pub agent_id: String,
    pub agent_name: String,
    pub messages: Vec<MessageExport>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageExport {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfigExport {
    pub id: String,
    pub name: String,
    pub specialization: String,
    pub personality: String,
    pub system_prompt: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct KnowledgeTransfer;

impl KnowledgeTransfer {
    pub fn new() -> Self {
        KnowledgeTransfer
    }
    
    pub async fn export_knowledge(
        &self,
        knowledge_base: &KnowledgeBase,
        export_path: &str,
        options: ExportOptions,
    ) -> Result<()> {
        // Get documents filtered by categories
        let documents = if options.categories.is_empty() {
            knowledge_base.get_all_documents().await?
        } else {
            let mut filtered_docs = Vec::new();
            for category in &options.categories {
                let mut category_docs = knowledge_base.get_documents_by_category(category).await?;
                filtered_docs.append(&mut category_docs);
            }
            // Remove duplicates
            filtered_docs.sort_by(|a, b| a.id.cmp(&b.id));
            filtered_docs.dedup_by(|a, b| a.id == b.id);
            filtered_docs
        };
        
        // Create package metadata
        let metadata = PackageMetadata {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
            package_type: "knowledge_export".to_string(),
            categories: options.categories.clone(),
            total_documents: documents.len(),
            total_conversations: 0, // TODO: Add conversation count
            checksum: self.calculate_checksum(&documents)?,
        };
        
        // Create knowledge package
        let package = KnowledgePackage {
            metadata,
            documents,
            conversations: Vec::new(), // TODO: Export conversations if requested
            agent_configs: Vec::new(), // TODO: Export agent configs if requested
        };
        
        // Serialize and save
        let json_data = serde_json::to_string_pretty(&package)?;
        
        if options.encrypt {
            let encrypted_data = self.encrypt_data(&json_data)?;
            fs::write(export_path, encrypted_data)?;
        } else {
            fs::write(export_path, json_data)?;
        }
        
        println!("Exported {} documents to {}", package.documents.len(), export_path);
        Ok(())
    }
    
    pub async fn import_knowledge(
        &self,
        knowledge_base: &mut KnowledgeBase,
        import_path: &str,
        merge_strategy: &str,
    ) -> Result<usize> {
        // Read and decrypt if necessary
        let file_content = fs::read_to_string(import_path)?;
        let json_data = if self.is_encrypted(&file_content) {
            self.decrypt_data(&file_content)?
        } else {
            file_content
        };
        
        // Parse knowledge package
        let package: KnowledgePackage = serde_json::from_str(&json_data)?;
        
        // Verify package integrity
        self.verify_package(&package)?;
        
        // Import based on strategy
        match merge_strategy {
            "merge" => self.merge_import(knowledge_base, package).await,
            "append" => self.append_import(knowledge_base, package).await,
            "replace" => self.replace_import(knowledge_base, package).await,
            _ => Err(anyhow!("Unknown merge strategy: {}", merge_strategy)),
        }
    }
    
    async fn merge_import(
        &self,
        knowledge_base: &mut KnowledgeBase,
        package: KnowledgePackage,
    ) -> Result<usize> {
        // Smart merge: Import new documents, update existing ones if newer
        let mut imported_count = 0;
        
        for document in package.documents {
            // Check if document exists by source path
            if let Ok(existing_doc) = knowledge_base.get_document(&document.id).await {
                if let Some(existing) = existing_doc {
                    // Compare timestamps and update if newer
                    if document.created_at > existing.created_at {
                        // Update existing document
                        // TODO: Implement document update
                        imported_count += 1;
                    }
                } else {
                    // New document, import it
                    knowledge_base.import_documents(vec![document]).await?;
                    imported_count += 1;
                }
            } else {
                // Import new document
                knowledge_base.import_documents(vec![document]).await?;
                imported_count += 1;
            }
        }
        
        Ok(imported_count)
    }
    
    async fn append_import(
        &self,
        knowledge_base: &mut KnowledgeBase,
        package: KnowledgePackage,
    ) -> Result<usize> {
        // Append only: Import all documents, skip duplicates
        let imported_count = knowledge_base.import_documents(package.documents).await?;
        Ok(imported_count)
    }
    
    async fn replace_import(
        &self,
        knowledge_base: &mut KnowledgeBase,
        package: KnowledgePackage,
    ) -> Result<usize> {
        // Replace all: Clear existing and import new
        knowledge_base.clear_all_documents().await?;
        let imported_count = knowledge_base.import_documents(package.documents).await?;
        Ok(imported_count)
    }
    
    pub async fn create_specialized_agent(
        &self,
        domain: &str,
        source_export_path: &str,
        target_path: &str,
    ) -> Result<()> {
        // Read source package
        let file_content = fs::read_to_string(source_export_path)?;
        let json_data = if self.is_encrypted(&file_content) {
            self.decrypt_data(&file_content)?
        } else {
            file_content
        };
        
        let mut package: KnowledgePackage = serde_json::from_str(&json_data)?;
        
        // Filter documents relevant to the domain
        package.documents = self.filter_documents_by_domain(&package.documents, domain)?;
        
        // Update metadata
        package.metadata.package_type = format!("specialized_agent_{}", domain);
        package.metadata.categories = vec![domain.to_string()];
        package.metadata.total_documents = package.documents.len();
        package.metadata.created_at = chrono::Utc::now();
        package.metadata.checksum = self.calculate_checksum(&package.documents)?;
        
        // Create specialized agent config
        let agent_config = AgentConfigExport {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{}Specialist", domain),
            specialization: domain.to_string(),
            personality: "Professional".to_string(),
            system_prompt: self.create_domain_prompt(domain),
            created_at: chrono::Utc::now(),
        };
        
        package.agent_configs = vec![agent_config];
        
        // Save specialized package
        let json_data = serde_json::to_string_pretty(&package)?;
        fs::write(target_path, json_data)?;
        
        println!("Created specialized agent for {} with {} documents", domain, package.documents.len());
        Ok(())
    }
    
    fn filter_documents_by_domain(&self, documents: &[Document], domain: &str) -> Result<Vec<Document>> {
        let domain_lower = domain.to_lowercase();
        let mut filtered = Vec::new();
        
        // Domain-specific keywords
        let domain_keywords = match domain_lower.as_str() {
            "coding" | "programming" => vec!["code", "programming", "software", "algorithm", "function", "class", "api", "debug"],
            "research" => vec!["research", "study", "analysis", "data", "experiment", "hypothesis", "methodology", "findings"],
            "writing" => vec!["writing", "content", "article", "blog", "story", "draft", "edit", "publish"],
            "business" => vec!["business", "strategy", "market", "client", "revenue", "project", "meeting", "sales"],
            _ => vec![&domain_lower],
        };
        
        for document in documents {
            let content_lower = document.content.to_lowercase();
            let title_lower = document.title.to_lowercase();
            
            // Check if document is relevant to domain
            let relevance_score = domain_keywords.iter()
                .map(|keyword| {
                    let title_matches = title_lower.matches(keyword).count() * 3; // Title matches are weighted more
                    let content_matches = content_lower.matches(keyword).count();
                    title_matches + content_matches
                })
                .sum::<usize>();
            
            // Include document if it has sufficient relevance
            if relevance_score >= 2 || document.categories.iter().any(|cat| cat.to_lowercase().contains(&domain_lower)) {
                filtered.push(document.clone());
            }
        }
        
        Ok(filtered)
    }
    
    fn create_domain_prompt(&self, domain: &str) -> String {
        match domain.to_lowercase().as_str() {
            "coding" | "programming" => {
                "You are a specialized programming assistant with expertise in software development, code review, debugging, and technical architecture. You provide clear, accurate coding solutions and explain complex programming concepts in an understandable way."
            },
            "research" => {
                "You are a specialized research assistant focused on academic and professional research. You excel at data analysis, research methodology, source evaluation, and synthesizing complex information into clear insights."
            },
            "writing" => {
                "You are a specialized writing assistant focused on content creation, editing, and creative writing. You help with writing techniques, grammar, style improvement, and various forms of written communication."
            },
            "business" => {
                "You are a specialized business assistant focused on professional tasks, strategic planning, and organizational efficiency. You provide insights on business operations, management, and professional communication."
            },
            _ => {
                &format!("You are a specialized assistant with deep expertise in {}. You provide knowledgeable, targeted assistance for {}-related tasks and questions.", domain, domain)
            }
        }.to_string()
    }
    
    fn calculate_checksum(&self, documents: &[Document]) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        for document in documents {
            document.id.hash(&mut hasher);
            document.content.hash(&mut hasher);
            document.created_at.hash(&mut hasher);
        }
        
        Ok(format!("{:x}", hasher.finish()))
    }
    
    fn verify_package(&self, package: &KnowledgePackage) -> Result<()> {
        // Verify checksum
        let calculated_checksum = self.calculate_checksum(&package.documents)?;
        if calculated_checksum != package.metadata.checksum {
            return Err(anyhow!("Package checksum verification failed"));
        }
        
        // Verify document count
        if package.documents.len() != package.metadata.total_documents {
            return Err(anyhow!("Document count mismatch in package"));
        }
        
        Ok(())
    }
    
    fn encrypt_data(&self, data: &str) -> Result<Vec<u8>> {
        // Simple encryption placeholder - in production, use proper encryption
        // For now, just return the data as bytes with a simple XOR
        let key = b"localmind_key_123"; // In production, use proper key management
        let mut encrypted = Vec::new();
        
        for (i, byte) in data.bytes().enumerate() {
            encrypted.push(byte ^ key[i % key.len()]);
        }
        
        Ok(encrypted)
    }
    
    fn decrypt_data(&self, data: &str) -> Result<String> {
        // Simple decryption placeholder - matches encrypt_data
        let key = b"localmind_key_123";
        let bytes = data.as_bytes();
        let mut decrypted = Vec::new();
        
        for (i, &byte) in bytes.iter().enumerate() {
            decrypted.push(byte ^ key[i % key.len()]);
        }
        
        String::from_utf8(decrypted).map_err(|e| anyhow!("Decryption failed: {}", e))
    }
    
    fn is_encrypted(&self, data: &str) -> bool {
        // Simple check - in production, use proper encryption markers
        // For now, assume it's encrypted if it contains non-printable characters
        data.chars().any(|c| !c.is_ascii() || c.is_control())
    }
    
    pub async fn get_export_preview(
        &self,
        knowledge_base: &KnowledgeBase,
        categories: &[String],
    ) -> Result<serde_json::Value> {
        let documents = if categories.is_empty() {
            knowledge_base.get_all_documents().await?
        } else {
            let mut filtered_docs = Vec::new();
            for category in categories {
                let mut category_docs = knowledge_base.get_documents_by_category(category).await?;
                filtered_docs.append(&mut category_docs);
            }
            filtered_docs
        };
        
        let total_size: u64 = documents.iter().map(|doc| doc.size).sum();
        let doc_types: std::collections::HashMap<String, usize> = documents
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, doc| {
                *acc.entry(doc.doc_type.clone()).or_insert(0) += 1;
                acc
            });
        
        Ok(serde_json::json!({
            "total_documents": documents.len(),
            "total_size_bytes": total_size,
            "document_types": doc_types,
            "categories": categories,
            "preview": documents.iter().take(5).map(|doc| {
                serde_json::json!({
                    "title": doc.title,
                    "type": doc.doc_type,
                    "size": doc.size,
                    "categories": doc.categories
                })
            }).collect::<Vec<_>>()
        }))
    }
}