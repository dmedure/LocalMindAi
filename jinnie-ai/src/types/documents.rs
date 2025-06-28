use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub doc_type: String,
    pub size: u64,
    pub path: String,
    pub summary: Option<String>,
    pub indexed_at: String, // ISO 8601 format for JS compatibility
    pub content: Option<String>, // Full content for processing
    pub metadata: DocumentMetadata,
    pub embedding_id: Option<String>, // Reference to vector embedding
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub language: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
    pub word_count: Option<usize>,
    pub read_time_minutes: Option<u32>,
    pub extracted_entities: Vec<String>,
    pub sentiment_score: Option<f32>,
    pub importance_score: f32,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub chunk_index: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub embedding: Option<Vec<f32>>,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub heading: Option<String>,
    pub section_type: Option<String>,
    pub importance_score: f32,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSearchResult {
    pub document: Document,
    pub relevance_score: f32,
    pub matching_chunks: Vec<DocumentChunk>,
    pub snippet: String,
    pub highlight_ranges: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndex {
    pub total_documents: usize,
    pub total_size_bytes: u64,
    pub document_types: HashMap<String, usize>,
    pub categories: HashMap<String, usize>,
    pub last_updated: DateTime<Utc>,
    pub embedding_model: String,
}

impl Document {
    pub fn new(name: String, path: String, doc_type: String, size: u64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            doc_type,
            size,
            path,
            summary: None,
            indexed_at: Utc::now().to_rfc3339(),
            content: None,
            metadata: DocumentMetadata::default(),
            embedding_id: None,
        }
    }
    
    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content.clone());
        self.metadata.word_count = Some(content.split_whitespace().count());
        self.metadata.read_time_minutes = Some((content.len() / 1000).max(1) as u32);
        self
    }
    
    pub fn add_category(&mut self, category: String) {
        if !self.metadata.categories.contains(&category) {
            self.metadata.categories.push(category);
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.metadata.tags.contains(&tag) {
            self.metadata.tags.push(tag);
        }
    }
    
    pub fn set_importance(&mut self, score: f32) {
        self.metadata.importance_score = score.clamp(0.0, 1.0);
    }
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            categories: Vec::new(),
            tags: Vec::new(),
            language: None,
            author: None,
            created_at: None,
            modified_at: None,
            word_count: None,
            read_time_minutes: None,
            extracted_entities: Vec::new(),
            sentiment_score: None,
            importance_score: 0.5,
            custom_fields: HashMap::new(),
        }
    }
}

impl DocumentChunk {
    pub fn new(
        document_id: String,
        content: String,
        chunk_index: usize,
        start_char: usize,
        end_char: usize,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            document_id,
            content,
            chunk_index,
            start_char,
            end_char,
            embedding: None,
            metadata: ChunkMetadata::default(),
        }
    }
}

impl Default for ChunkMetadata {
    fn default() -> Self {
        Self {
            heading: None,
            section_type: None,
            importance_score: 0.5,
            summary: None,
        }
    }
}