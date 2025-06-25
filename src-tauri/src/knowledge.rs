use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::path::Path;
use std::fs;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub source: String,
    pub content: String,
    pub content_preview: String,
    pub doc_type: String,
    pub size: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
    pub embedding: Option<Vec<f32>>,
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f64,
    pub snippet: String,
}

pub struct KnowledgeBase {
    documents: HashMap<String, Document>,
    use_chroma: bool,
}

impl KnowledgeBase {
    pub async fn new() -> Result<Self> {
        // Try to connect to ChromaDB, fall back to local storage if not available
        let use_chroma = Self::test_chroma_connection().await;
        
        if use_chroma {
            println!("Connected to ChromaDB for vector search");
        } else {
            println!("ChromaDB not available, using local text search");
        }
        
        Ok(KnowledgeBase {
            documents: HashMap::new(),
            use_chroma,
        })
    }
    
    async fn test_chroma_connection() -> bool {
        match reqwest::get("http://localhost:8000/api/v1/heartbeat").await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
    
    pub async fn index_document(&mut self, file_path: &str) -> Result<String> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }
        
        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read file: {}", e))?;
        
        // Extract metadata
        let metadata = fs::metadata(path)
            .map_err(|e| anyhow!("Failed to get file metadata: {}", e))?;
        
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let doc_type = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_uppercase();
        
        // Create content preview (first 200 characters)
        let content_preview = if content.len() > 200 {
            format!("{}...", &content[..200])
        } else {
            content.clone()
        };
        
        // Create document
        let document = Document {
            id: Uuid::new_v4().to_string(),
            title: file_name.clone(),
            source: file_path.to_string(),
            content: content.clone(),
            content_preview,
            doc_type,
            size: metadata.len(),
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
            embedding: None, // TODO: Generate embeddings if ChromaDB is available
            categories: self.extract_categories(&content),
        };
        
        // Store document
        self.documents.insert(document.id.clone(), document.clone());
        
        // Index in ChromaDB if available
        if self.use_chroma {
            self.index_in_chroma(&document).await?;
        }
        
        Ok(document.id)
    }
    
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<Document>> {
        if self.use_chroma {
            self.search_with_chroma(query, limit).await
        } else {
            Ok(self.search_local(query, limit))
        }
    }
    
    fn search_local(&self, query: &str, limit: usize) -> Vec<Document> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(Document, f64)> = Vec::new();
        
        for document in self.documents.values() {
            let title_score = if document.title.to_lowercase().contains(&query_lower) { 2.0 } else { 0.0 };
            let content_score = self.calculate_content_score(&document.content, &query_lower);
            let total_score = title_score + content_score;
            
            if total_score > 0.0 {
                results.push((document.clone(), total_score));
            }
        }
        
        // Sort by score (highest first) and take top results
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.into_iter()
            .take(limit)
            .map(|(doc, _)| doc)
            .collect()
    }
    
    fn calculate_content_score(&self, content: &str, query: &str) -> f64 {
        let content_lower = content.to_lowercase();
        let query_words: Vec<&str> = query.split_whitespace().collect();
        
        let mut score = 0.0;
        for word in query_words {
            if word.len() > 2 { // Skip very short words
                let occurrences = content_lower.matches(word).count();
                score += occurrences as f64 * word.len() as f64;
            }
        }
        
        // Normalize by content length
        if content.len() > 0 {
            score / (content.len() as f64).sqrt()
        } else {
            0.0
        }
    }
    
    async fn search_with_chroma(&self, query: &str, limit: usize) -> Result<Vec<Document>> {
        // TODO: Implement ChromaDB vector search
        // For now, fall back to local search
        Ok(self.search_local(query, limit))
    }
    
    async fn index_in_chroma(&self, document: &Document) -> Result<()> {
        // TODO: Implement ChromaDB indexing
        // This would involve:
        // 1. Generate embeddings for document content
        // 2. Store document and embeddings in ChromaDB
        // 3. Handle any ChromaDB-specific metadata
        Ok(())
    }
    
    pub async fn get_all_documents(&self) -> Result<Vec<Document>> {
        Ok(self.documents.values().cloned().collect())
    }
    
    pub async fn get_document(&self, doc_id: &str) -> Result<Option<Document>> {
        Ok(self.documents.get(doc_id).cloned())
    }
    
    pub async fn delete_document(&mut self, doc_id: &str) -> Result<bool> {
        let removed = self.documents.remove(doc_id).is_some();
        
        if removed && self.use_chroma {
            self.delete_from_chroma(doc_id).await?;
        }
        
        Ok(removed)
    }
    
    async fn delete_from_chroma(&self, doc_id: &str) -> Result<()> {
        // TODO: Implement ChromaDB deletion
        Ok(())
    }
    
    pub async fn get_document_count(&self) -> Result<usize> {
        Ok(self.documents.len())
    }
    
    pub async fn get_documents_by_category(&self, category: &str) -> Result<Vec<Document>> {
        let documents: Vec<Document> = self.documents
            .values()
            .filter(|doc| doc.categories.contains(&category.to_string()))
            .cloned()
            .collect();
        
        Ok(documents)
    }
    
    pub async fn get_categories(&self) -> Result<Vec<String>> {
        let mut categories = std::collections::HashSet::new();
        
        for document in self.documents.values() {
            for category in &document.categories {
                categories.insert(category.clone());
            }
        }
        
        let mut category_list: Vec<String> = categories.into_iter().collect();
        category_list.sort();
        Ok(category_list)
    }
    
    fn extract_categories(&self, content: &str) -> Vec<String> {
        let mut categories = Vec::new();
        let content_lower = content.to_lowercase();
        
        // Simple keyword-based category detection
        let category_keywords = vec![
            ("technical", vec!["code", "programming", "software", "algorithm", "database", "api", "function", "class", "variable"]),
            ("business", vec!["meeting", "project", "client", "revenue", "strategy", "market", "sales", "customer"]),
            ("research", vec!["study", "analysis", "data", "research", "experiment", "hypothesis", "conclusion", "methodology"]),
            ("personal", vec!["personal", "private", "diary", "journal", "notes", "thoughts", "idea", "reminder"]),
            ("work", vec!["work", "office", "task", "deadline", "colleague", "manager", "report", "presentation"]),
        ];
        
        for (category, keywords) in category_keywords {
            let mut matches = 0;
            for keyword in keywords {
                if content_lower.contains(keyword) {
                    matches += 1;
                }
            }
            
            // If at least 2 keywords match, add the category
            if matches >= 2 {
                categories.push(category.to_string());
            }
        }
        
        // Default category if none detected
        if categories.is_empty() {
            categories.push("general".to_string());
        }
        
        categories
    }
    
    pub async fn export_documents(&self, categories: Option<Vec<String>>) -> Result<Vec<Document>> {
        let documents = if let Some(cats) = categories {
            self.documents
                .values()
                .filter(|doc| doc.categories.iter().any(|c| cats.contains(c)))
                .cloned()
                .collect()
        } else {
            self.documents.values().cloned().collect()
        };
        
        Ok(documents)
    }
    
    pub async fn import_documents(&mut self, documents: Vec<Document>) -> Result<usize> {
        let mut imported_count = 0;
        
        for document in documents {
            // Check if document already exists (by source path)
            let exists = self.documents
                .values()
                .any(|existing| existing.source == document.source);
            
            if !exists {
                self.documents.insert(document.id.clone(), document.clone());
                
                // Index in ChromaDB if available
                if self.use_chroma {
                    if let Err(e) = self.index_in_chroma(&document).await {
                        println!("Warning: Failed to index document {} in ChromaDB: {}", document.id, e);
                    }
                }
                
                imported_count += 1;
            }
        }
        
        Ok(imported_count)
    }
    
    pub async fn get_storage_stats(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();
        
        let total_documents = self.documents.len();
        let total_size: u64 = self.documents.values().map(|doc| doc.size).sum();
        
        let mut categories_count = HashMap::new();
        for document in self.documents.values() {
            for category in &document.categories {
                *categories_count.entry(category.clone()).or_insert(0) += 1;
            }
        }
        
        let mut type_count = HashMap::new();
        for document in self.documents.values() {
            *type_count.entry(document.doc_type.clone()).or_insert(0) += 1;
        }
        
        stats.insert("total_documents".to_string(), serde_json::Value::Number(serde_json::Number::from(total_documents)));
        stats.insert("total_size_bytes".to_string(), serde_json::Value::Number(serde_json::Number::from(total_size)));
        stats.insert("categories".to_string(), serde_json::to_value(categories_count)?);
        stats.insert("document_types".to_string(), serde_json::to_value(type_count)?);
        stats.insert("chroma_enabled".to_string(), serde_json::Value::Bool(self.use_chroma));
        
        Ok(stats)
    }
    
    pub async fn clear_all_documents(&mut self) -> Result<usize> {
        let count = self.documents.len();
        self.documents.clear();
        
        // TODO: Clear ChromaDB collection if available
        if self.use_chroma {
            // self.clear_chroma_collection().await?;
        }
        
        Ok(count)
    }
    
    pub fn get_document_summary(&self, doc_id: &str) -> Option<String> {
        self.documents.get(doc_id).map(|doc| {
            format!(
                "Document: {}\nType: {}\nSize: {} bytes\nCategories: {}\nPreview: {}",
                doc.title,
                doc.doc_type,
                doc.size,
                doc.categories.join(", "),
                doc.content_preview
            )
        })
    }
}