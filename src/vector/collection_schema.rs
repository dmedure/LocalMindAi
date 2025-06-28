use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema {
    pub name: String,
    pub description: String,
    pub vector_size: usize,
    pub distance_metric: String,
    pub fields: HashMap<String, FieldType>,
    pub indexes: Vec<IndexConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Keyword,
    Integer,
    Float,
    Boolean,
    DateTime,
    Geo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub field_name: String,
    pub field_type: FieldType,
    pub is_filterable: bool,
    pub is_facet: bool,
}

#[derive(Debug, Clone)]
pub struct VectorCollection {
    pub schema: CollectionSchema,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub point_count: usize,
}

impl CollectionSchema {
    /// Create schema for memory collection
    pub fn memory_collection() -> Self {
        let mut fields = HashMap::new();
        fields.insert("agent_id".to_string(), FieldType::Keyword);
        fields.insert("conversation_id".to_string(), FieldType::Keyword);
        fields.insert("session_id".to_string(), FieldType::Keyword);
        fields.insert("content".to_string(), FieldType::Text);
        fields.insert("layer".to_string(), FieldType::Keyword);
        fields.insert("importance_score".to_string(), FieldType::Float);
        fields.insert("access_count".to_string(), FieldType::Integer);
        fields.insert("created_at".to_string(), FieldType::DateTime);
        fields.insert("last_accessed".to_string(), FieldType::DateTime);
        fields.insert("source".to_string(), FieldType::Keyword);
        fields.insert("topics".to_string(), FieldType::Keyword);
        fields.insert("tags".to_string(), FieldType::Keyword);
        fields.insert("verification_status".to_string(), FieldType::Keyword);

        let indexes = vec![
            IndexConfig {
                field_name: "agent_id".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
            IndexConfig {
                field_name: "layer".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
            IndexConfig {
                field_name: "importance_score".to_string(),
                field_type: FieldType::Float,
                is_filterable: true,
                is_facet: false,
            },
            IndexConfig {
                field_name: "created_at".to_string(),
                field_type: FieldType::DateTime,
                is_filterable: true,
                is_facet: false,
            },
            IndexConfig {
                field_name: "topics".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
            IndexConfig {
                field_name: "source".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
        ];

        Self {
            name: "memories".to_string(),
            description: "Collection for storing memory embeddings with metadata".to_string(),
            vector_size: 384, // all-MiniLM-L6-v2 dimension
            distance_metric: "cosine".to_string(),
            fields,
            indexes,
        }
    }

    /// Create schema for document collection
    pub fn document_collection() -> Self {
        let mut fields = HashMap::new();
        fields.insert("document_id".to_string(), FieldType::Keyword);
        fields.insert("title".to_string(), FieldType::Text);
        fields.insert("content".to_string(), FieldType::Text);
        fields.insert("doc_type".to_string(), FieldType::Keyword);
        fields.insert("file_path".to_string(), FieldType::Keyword);
        fields.insert("file_size".to_string(), FieldType::Integer);
        fields.insert("created_at".to_string(), FieldType::DateTime);
        fields.insert("modified_at".to_string(), FieldType::DateTime);
        fields.insert("indexed_at".to_string(), FieldType::DateTime);
        fields.insert("categories".to_string(), FieldType::Keyword);
        fields.insert("tags".to_string(), FieldType::Keyword);
        fields.insert("language".to_string(), FieldType::Keyword);
        fields.insert("summary".to_string(), FieldType::Text);

        let indexes = vec![
            IndexConfig {
                field_name: "doc_type".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
            IndexConfig {
                field_name: "categories".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
            IndexConfig {
                field_name: "language".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
            IndexConfig {
                field_name: "created_at".to_string(),
                field_type: FieldType::DateTime,
                is_filterable: true,
                is_facet: false,
            },
            IndexConfig {
                field_name: "file_size".to_string(),
                field_type: FieldType::Integer,
                is_filterable: true,
                is_facet: false,
            },
        ];

        Self {
            name: "documents".to_string(),
            description: "Collection for storing document embeddings and metadata".to_string(),
            vector_size: 384,
            distance_metric: "cosine".to_string(),
            fields,
            indexes,
        }
    }

    /// Create schema for conversation collection
    pub fn conversation_collection() -> Self {
        let mut fields = HashMap::new();
        fields.insert("agent_id".to_string(), FieldType::Keyword);
        fields.insert("session_id".to_string(), FieldType::Keyword);
        fields.insert("conversation_id".to_string(), FieldType::Keyword);
        fields.insert("message_id".to_string(), FieldType::Keyword);
        fields.insert("sender".to_string(), FieldType::Keyword);
        fields.insert("content".to_string(), FieldType::Text);
        fields.insert("message_type".to_string(), FieldType::Keyword);
        fields.insert("timestamp".to_string(), FieldType::DateTime);
        fields.insert("model_used".to_string(), FieldType::Keyword);
        fields.insert("response_time_ms".to_string(), FieldType::Integer);
        fields.insert("token_count".to_string(), FieldType::Integer);
        fields.insert("context_length".to_string(), FieldType::Integer);

        let indexes = vec![
            IndexConfig {
                field_name: "agent_id".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
            IndexConfig {
                field_name: "sender".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
            IndexConfig {
                field_name: "timestamp".to_string(),
                field_type: FieldType::DateTime,
                is_filterable: true,
                is_facet: false,
            },
            IndexConfig {
                field_name: "model_used".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
            IndexConfig {
                field_name: "message_type".to_string(),
                field_type: FieldType::Keyword,
                is_filterable: true,
                is_facet: true,
            },
        ];

        Self {
            name: "conversations".to_string(),
            description: "Collection for storing conversation message embeddings".to_string(),
            vector_size: 384,
            distance_metric: "cosine".to_string(),
            fields,
            indexes,
        }
    }

    /// Create a custom collection schema
    pub fn custom_collection(
        name: String,
        description: String,
        vector_size: usize,
        distance_metric: String,
    ) -> Self {
        Self {
            name,
            description,
            vector_size,
            distance_metric,
            fields: HashMap::new(),
            indexes: Vec::new(),
        }
    }

    /// Add a field to the schema
    pub fn add_field(mut self, name: String, field_type: FieldType) -> Self {
        self.fields.insert(name, field_type);
        self
    }

    /// Add an index to the schema
    pub fn add_index(mut self, index: IndexConfig) -> Self {
        self.indexes.push(index);
        self
    }

    /// Get field type by name
    pub fn get_field_type(&self, field_name: &str) -> Option<&FieldType> {
        self.fields.get(field_name)
    }

    /// Check if field is indexed
    pub fn is_field_indexed(&self, field_name: &str) -> bool {
        self.indexes.iter().any(|idx| idx.field_name == field_name)
    }

    /// Get all filterable fields
    pub fn get_filterable_fields(&self) -> Vec<&IndexConfig> {
        self.indexes.iter().filter(|idx| idx.is_filterable).collect()
    }

    /// Get all facet fields
    pub fn get_facet_fields(&self) -> Vec<&IndexConfig> {
        self.indexes.iter().filter(|idx| idx.is_facet).collect()
    }

    /// Validate the schema
    pub fn validate(&self) -> Result<(), String> {
        // Check that collection name is valid
        if self.name.is_empty() {
            return Err("Collection name cannot be empty".to_string());
        }

        // Check vector size
        if self.vector_size == 0 {
            return Err("Vector size must be greater than 0".to_string());
        }

        // Check distance metric
        match self.distance_metric.as_str() {
            "cosine" | "euclidean" | "dot" => {},
            _ => return Err(format!("Unsupported distance metric: {}", self.distance_metric)),
        }

        // Check that indexed fields exist in fields
        for index in &self.indexes {
            if !self.fields.contains_key(&index.field_name) {
                return Err(format!("Indexed field '{}' not found in schema fields", index.field_name));
            }
        }

        Ok(())
    }

    /// Get memory usage estimate for this schema (in bytes)
    pub fn estimate_memory_usage(&self, point_count: usize) -> usize {
        let vector_size = self.vector_size * 4; // 4 bytes per f32
        let metadata_size = self.fields.len() * 50; // Rough estimate per field
        let index_overhead = self.indexes.len() * 100; // Rough estimate per index
        
        point_count * (vector_size + metadata_size) + index_overhead
    }
}

impl From<CollectionSchema> for VectorCollection {
    fn from(schema: CollectionSchema) -> Self {
        let now = chrono::Utc::now();
        Self {
            schema,
            created_at: now,
            last_updated: now,
            point_count: 0,
        }
    }
}

impl IndexConfig {
    /// Create a new index configuration
    pub fn new(field_name: String, field_type: FieldType) -> Self {
        Self {
            field_name,
            field_type,
            is_filterable: true,
            is_facet: false,
        }
    }

    /// Make this index filterable
    pub fn filterable(mut self) -> Self {
        self.is_filterable = true;
        self
    }

    /// Make this index a facet
    pub fn facet(mut self) -> Self {
        self.is_facet = true;
        self
    }
}

impl FieldType {
    /// Get the storage size estimate for this field type (in bytes)
    pub fn storage_size_estimate(&self) -> usize {
        match self {
            FieldType::Text => 100,      // Variable, estimated average
            FieldType::Keyword => 20,    // Short strings
            FieldType::Integer => 8,     // 64-bit integer
            FieldType::Float => 8,       // 64-bit float
            FieldType::Boolean => 1,     // Single byte
            FieldType::DateTime => 8,    // 64-bit timestamp
            FieldType::Geo => 16,        // Lat/lon pair
        }
    }

    /// Check if this field type supports full-text search
    pub fn supports_full_text_search(&self) -> bool {
        matches!(self, FieldType::Text)
    }

    /// Check if this field type supports range queries
    pub fn supports_range_queries(&self) -> bool {
        matches!(self, FieldType::Integer | FieldType::Float | FieldType::DateTime)
    }

    /// Check if this field type supports exact matching
    pub fn supports_exact_match(&self) -> bool {
        matches!(self, FieldType::Keyword | FieldType::Boolean | FieldType::Integer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_collection_schema() {
        let schema = CollectionSchema::memory_collection();
        
        assert_eq!(schema.name, "memories");
        assert_eq!(schema.vector_size, 384);
        assert_eq!(schema.distance_metric, "cosine");
        assert!(schema.fields.contains_key("agent_id"));
        assert!(schema.fields.contains_key("content"));
        assert!(!schema.indexes.is_empty());
        
        // Validate the schema
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_document_collection_schema() {
        let schema = CollectionSchema::document_collection();
        
        assert_eq!(schema.name, "documents");
        assert!(schema.fields.contains_key("title"));
        assert!(schema.fields.contains_key("doc_type"));
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_conversation_collection_schema() {
        let schema = CollectionSchema::conversation_collection();
        
        assert_eq!(schema.name, "conversations");
        assert!(schema.fields.contains_key("sender"));
        assert!(schema.fields.contains_key("timestamp"));
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_custom_collection_schema() {
        let schema = CollectionSchema::custom_collection(
            "test_collection".to_string(),
            "Test collection".to_string(),
            512,
            "euclidean".to_string(),
        )
        .add_field("custom_field".to_string(), FieldType::Text)
        .add_index(IndexConfig::new("custom_field".to_string(), FieldType::Text).filterable());

        assert_eq!(schema.name, "test_collection");
        assert_eq!(schema.vector_size, 512);
        assert!(schema.fields.contains_key("custom_field"));
        assert!(!schema.indexes.is_empty());
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_schema_validation() {
        // Test empty name
        let invalid_schema = CollectionSchema::custom_collection(
            "".to_string(),
            "Description".to_string(),
            384,
            "cosine".to_string(),
        );
        assert!(invalid_schema.validate().is_err());

        // Test zero vector size
        let invalid_schema = CollectionSchema::custom_collection(
            "test".to_string(),
            "Description".to_string(),
            0,
            "cosine".to_string(),
        );
        assert!(invalid_schema.validate().is_err());

        // Test invalid distance metric
        let invalid_schema = CollectionSchema::custom_collection(
            "test".to_string(),
            "Description".to_string(),
            384,
            "invalid_metric".to_string(),
        );
        assert!(invalid_schema.validate().is_err());
    }

    #[test]
    fn test_field_type_properties() {
        assert!(FieldType::Text.supports_full_text_search());
        assert!(!FieldType::Keyword.supports_full_text_search());
        
        assert!(FieldType::Integer.supports_range_queries());
        assert!(FieldType::Float.supports_range_queries());
        assert!(FieldType::DateTime.supports_range_queries());
        assert!(!FieldType::Text.supports_range_queries());
        
        assert!(FieldType::Keyword.supports_exact_match());
        assert!(FieldType::Boolean.supports_exact_match());
        assert!(!FieldType::Text.supports_exact_match());
    }

    #[test]
    fn test_schema_queries() {
        let schema = CollectionSchema::memory_collection();
        
        // Test field type lookup
        assert!(matches!(schema.get_field_type("agent_id"), Some(FieldType::Keyword)));
        assert!(matches!(schema.get_field_type("importance_score"), Some(FieldType::Float)));
        assert!(schema.get_field_type("nonexistent_field").is_none());
        
        // Test index checking
        assert!(schema.is_field_indexed("agent_id"));
        assert!(!schema.is_field_indexed("content")); // Not indexed by default
        
        // Test filterable fields
        let filterable = schema.get_filterable_fields();
        assert!(!filterable.is_empty());
        assert!(filterable.iter().any(|idx| idx.field_name == "agent_id"));
        
        // Test facet fields
        let facets = schema.get_facet_fields();
        assert!(facets.iter().any(|idx| idx.field_name == "layer"));
    }

    #[test]
    fn test_memory_usage_estimation() {
        let schema = CollectionSchema::memory_collection();
        let usage = schema.estimate_memory_usage(1000);
        
        // Should be a reasonable estimate
        assert!(usage > 0);
        assert!(usage > 1000 * 384 * 4); // At least the vector storage
    }

    #[test]
    fn test_vector_collection_conversion() {
        let schema = CollectionSchema::memory_collection();
        let collection: VectorCollection = schema.into();
        
        assert_eq!(collection.schema.name, "memories");
        assert_eq!(collection.point_count, 0);
        assert!(collection.created_at <= chrono::Utc::now());
    }
}