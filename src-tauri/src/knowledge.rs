use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::path::Path;
use std::fs;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::knowledge_transfer::{ExportOptions, MergeStrategy};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub source: String,
    pub metadata: DocumentMetadata,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: String,
    pub file_type: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub keywords: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChromaCollection {
    pub name: String,
    pub id: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChromaAddRequest {
    pub documents: Vec<String>,
    pub metadatas: Vec<serde_json::Value>,
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChromaQueryRequest {
    pub query_texts: Vec<String>,
    pub n_results: usize,
    pub include: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChromaQueryResponse {
    pub ids: Vec<Vec<String>>,
    pub documents: Vec<Vec<String>>,
    pub metadatas: Vec<Vec<serde_json::Value>>,
    pub distances: Vec<Vec<f32>>,
}

pub struct KnowledgeBase {
    client: reqwest::Client,
    base_url: String,
    collection_name: String,
    documents: Vec<Document>, // Local cache
}

impl KnowledgeBase {
    pub async fn new() -> Result<Self> {
        let client = reqwest::Client::new();
        let base_url = "http://localhost:8000".to_string(); // Default ChromaDB port
        let collection_name = "local_agent_docs".to_string();
        
        let mut kb = KnowledgeBase {
            client,
            base_url,
            collection_name,
            documents: Vec::new(),
        };
        
        // Initialize collection
        kb.ensure_collection().await?;
        
        Ok(kb)
    }
    
    async fn ensure_collection(&self) -> Result<()> {
        // Check if collection exists
        let collections_url = format!("{}/api/v1/collections", self.base_url);
        let response = self.client.get(&collections_url).send().await;
        
        match response {
            Ok(resp) => {
                let collections: Vec<ChromaCollection> = resp.json().await.unwrap_or_default();
                let collection_exists = collections.iter()
                    .any(|c| c.name == self.collection_name);
                
                if !collection_exists {
                    self.create_collection().await?;
                }
            }
            Err(_) => {
                // ChromaDB might not be running, create a fallback
                log::warn!("ChromaDB not available, using local storage only");
            }
        }
        
        Ok(())
    }
    
    async fn create_collection(&self) -> Result<()> {
        let create_url = format!("{}/api/v1/collections", self.base_url);
        let payload = serde_json::json!({
            "name": self.collection_name,
            "metadata": {
                "description": "Local AI Agent Documents"
            }
        });
        
        let response = self.client
            .post(&create_url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to create ChromaDB collection"));
        }
        
        Ok(())
    }
    
    pub async fn index_document(&mut self, file_path: &str) -> Result<String> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }
        
        // Read file content
        let content = match path.extension().and_then(|e| e.to_str()) {
            Some("txt") | Some("md") => fs::read_to_string(path)?,
            Some("pdf") => {
                // TODO: Implement PDF parsing
                return Err(anyhow!("PDF parsing not yet implemented"));
            }
            Some("docx") => {
                // TODO: Implement DOCX parsing
                return Err(anyhow!("DOCX parsing not yet implemented"));
            }
            _ => fs::read_to_string(path)?, // Try as text
        };
        
        // Create document
        let doc_id = Uuid::new_v4().to_string();
        let metadata = std::fs::metadata(path)?;
        
        let document = Document {
            id: doc_id.clone(),
            content: content.clone(),
            source: file_path.to_string(),
            metadata: DocumentMetadata {
                title: path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                file_type: path.extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                size: metadata.len(),
                created_at: Utc::now(),
                last_modified: metadata.modified()
                    .unwrap_or(std::time::SystemTime::now())
                    .into(),
                keywords: Vec::new(), // TODO: Extract keywords using LLM
                summary: String::new(), // TODO: Generate summary using LLM
            },
            embedding: None,
        };
        
        // Add to ChromaDB if available
        self.add_to_chroma(&document).await.ok(); // Don't fail if ChromaDB is unavailable
        
        // Add to local cache
        self.documents.push(document);
        
        Ok(doc_id)
    }
    
    async fn add_to_chroma(&self, document: &Document) -> Result<()> {
        let add_url = format!("{}/api/v1/collections/{}/add", 
            self.base_url, self.collection_name);
        
        let request = ChromaAddRequest {
            documents: vec![document.content.clone()],
            metadatas: vec![serde_json::to_value(&document.metadata)?],
            ids: vec![document.id.clone()],
        };
        
        let response = self.client
            .post(&add_url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to add document to ChromaDB"));
        }
        
        Ok(())
    }
    
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<Document>> {
        // Try ChromaDB first
        if let Ok(results) = self.search_chroma(query, limit).await {
            return Ok(results);
        }
        
        // Fallback to simple text search in local cache
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        for doc in &self.documents {
            if doc.content.to_lowercase().contains(&query_lower) 
                || doc.metadata.title.to_lowercase().contains(&query_lower) {
                results.push(doc.clone());
                if results.len() >= limit {
                    break;
                }
            }
        }
        
        Ok(results)
    }
    
    async fn search_chroma(&self, query: &str, limit: usize) -> Result<Vec<Document>> {
        let query_url = format!("{}/api/v1/collections/{}/query", 
            self.base_url, self.collection_name);
        
        let request = ChromaQueryRequest {
            query_texts: vec![query.to_string()],
            n_results: limit,
            include: vec!["documents".to_string(), "metadatas".to_string()],
        };
        
        let response = self.client
            .post(&query_url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("ChromaDB query failed"));
        }
        
        let query_response: ChromaQueryResponse = response.json().await?;
        
        let mut results = Vec::new();
        if let (Some(docs), Some(metadatas)) = (
            query_response.documents.first(),
            query_response.metadatas.first()
        ) {
            for (i, content) in docs.iter().enumerate() {
                if let Some(metadata_value) = metadatas.get(i) {
                    if let Ok(metadata) = serde_json::from_value::<DocumentMetadata>(metadata_value.clone()) {
                        results.push(Document {
                            id: Uuid::new_v4().to_string(), // ChromaDB doesn't return IDs in this format
                            content: content.clone(),
                            source: metadata.title.clone(), // Use title as source for now
                            metadata,
                            embedding: None,
                        });
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    pub fn get_document_count(&self) -> usize {
        self.documents.len()
    }
    
    pub fn get_all_documents(&self) -> &[Document] {
        &self.documents
    }
    
    // Knowledge Transfer Methods
    pub async fn export_knowledge(
        &self, 
        export_path: &str, 
        options: ExportOptions
    ) -> Result<String> {
        let mut filtered_documents = Vec::new();
        
        // Filter documents by categories if specified
        for doc in &self.documents {
            if options.categories.is_empty() || 
               options.categories.iter().any(|cat| 
                   doc.metadata.keywords.contains(cat) || 
                   doc.source.contains(cat)
               ) {
                filtered_documents.push(doc.clone());
            }
        }
        
        let export_data = serde_json::json!({
            "version": "1.0",
            "exported_at": Utc::now(),
            "document_count": filtered_documents.len(),
            "documents": filtered_documents,
            "metadata": {
                "collection_name": self.collection_name,
                "export_options": options
            }
        });
        
        let json_string = if options.compress {
            // TODO: Implement compression
            serde_json::to_string_pretty(&export_data)?
        } else {
            serde_json::to_string_pretty(&export_data)?
        };
        
        if options.encrypt {
            // TODO: Implement encryption
            fs::write(export_path, &json_string)?;
        } else {
            fs::write(export_path, &json_string)?;
        }
        
        Ok(format!("Exported {} documents to {}", filtered_documents.len(), export_path))
    }
    
    pub async fn import_knowledge(
        &mut self, 
        import_path: &str, 
        strategy: MergeStrategy
    ) -> Result<String> {
        let import_data: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(import_path)?
        )?;
        
        let imported_documents: Vec<Document> = serde_json::from_value(
            import_data["documents"].clone()
        )?;
        
        let mut imported_count = 0;
        
        match strategy {
            MergeStrategy::Replace => {
                self.documents = imported_documents;
                imported_count = self.documents.len();
            },
            MergeStrategy::Append => {
                for doc in imported_documents {
                    self.documents.push(doc);
                    imported_count += 1;
                }
            },
            MergeStrategy::Merge => {
                for imported_doc in imported_documents {
                    // Check if document already exists by source
                    if let Some(existing_index) = self.documents.iter()
                        .position(|d| d.source == imported_doc.source) {
                        // Update existing document
                        self.documents[existing_index] = imported_doc;
                    } else {
                        // Add new document
                        self.documents.push(imported_doc);
                    }
                    imported_count += 1;
                }
            }
        }
        
        // Re-index in ChromaDB if available
        for doc in &self.documents[self.documents.len() - imported_count..] {
            self.add_to_chroma(doc).await.ok();
        }
        
        Ok(format!("Imported {} documents using {:?} strategy", imported_count, strategy))
    }
    
    pub fn get_conversation_count(&self) -> usize {
        // TODO: Implement conversation tracking
        0
    }
    
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories = std::collections::HashSet::new();
        
        for doc in &self.documents {
            for keyword in &doc.metadata.keywords {
                categories.insert(keyword.clone());
            }
        }
        
        categories.into_iter().collect()
    }
    
    pub fn get_last_updated(&self) -> Option<DateTime<Utc>> {
        self.documents.iter()
            .map(|d| d.metadata.last_modified)
            .max()
    }
    
    pub fn estimate_storage_size(&self) -> usize {
        self.documents.iter()
            .map(|d| d.content.len() + d.source.len())
            .sum()
    }
}