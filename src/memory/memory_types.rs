use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Memory hierarchy following MemGPT principles
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryLayer {
    /// Current conversation context (10-20 items)
    Working,
    /// Recent interactions (50-100 items) 
    ShortTerm,
    /// Important persistent facts
    LongTerm,
    /// Specific events and experiences
    Episodic,
    /// General knowledge and concepts
    Semantic,
    /// AI-generated insights and patterns
    Reflective,
}

impl MemoryLayer {
    /// Get the typical capacity for each memory layer
    pub fn typical_capacity(&self) -> usize {
        match self {
            MemoryLayer::Working => 20,
            MemoryLayer::ShortTerm => 100,
            MemoryLayer::LongTerm => 1000,
            MemoryLayer::Episodic => 5000,
            MemoryLayer::Semantic => 10000,
            MemoryLayer::Reflective => 500,
        }
    }

    /// Get the retention priority (higher = more important to keep)
    pub fn retention_priority(&self) -> u8 {
        match self {
            MemoryLayer::Working => 10,
            MemoryLayer::ShortTerm => 5,
            MemoryLayer::LongTerm => 9,
            MemoryLayer::Episodic => 6,
            MemoryLayer::Semantic => 8,
            MemoryLayer::Reflective => 7,
        }
    }
}

/// Core memory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub layer: MemoryLayer,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: MemoryMetadata,
    pub importance_score: f32,
    pub access_count: u32,
    pub last_accessed: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub associations: Vec<Uuid>,
    pub tags: Vec<String>,
}

impl Memory {
    pub fn new(content: String, layer: MemoryLayer, metadata: MemoryMetadata) -> Self {
        Self {
            id: Uuid::new_v4(),
            layer,
            content,
            embedding: None,
            metadata,
            importance_score: 0.5, // Default neutral importance
            access_count: 0,
            last_accessed: Utc::now(),
            created_at: Utc::now(),
            associations: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Calculate recency score (1.0 = very recent, 0.0 = very old)
    pub fn recency_score(&self) -> f32 {
        let now = Utc::now();
        let age_hours = now.signed_duration_since(self.created_at).num_hours() as f32;
        
        // Exponential decay with half-life of 24 hours
        (-age_hours / 24.0).exp()
    }

    /// Calculate frequency score based on access count
    pub fn frequency_score(&self) -> f32 {
        // Logarithmic scaling to prevent runaway frequency scores
        (self.access_count as f32 + 1.0).ln() / 10.0
    }

    /// Calculate overall memory strength for retention decisions
    pub fn memory_strength(&self) -> f32 {
        let importance_weight = 0.4;
        let recency_weight = 0.3;
        let frequency_weight = 0.2;
        let layer_weight = 0.1;
        
        importance_weight * self.importance_score +
        recency_weight * self.recency_score() +
        frequency_weight * self.frequency_score() +
        layer_weight * (self.layer.retention_priority() as f32 / 10.0)
    }

    /// Update access statistics
    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }
}

/// Metadata associated with a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub source: MemorySource,
    pub agent_id: String,
    pub conversation_id: Option<String>,
    pub session_id: Option<String>,
    pub topics: Vec<String>,
    pub entities: Vec<Entity>,
    pub sentiment: Option<Sentiment>,
    pub context_window: Option<ContextWindow>,
    pub verification_status: VerificationStatus,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Source of the memory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemorySource {
    /// Direct user input
    UserInput,
    /// Agent response
    AgentResponse,
    /// System-generated insight
    SystemInsight,
    /// Imported from external source
    ExternalImport,
    /// Generated through reflection
    Reflection,
    /// Extracted from document
    DocumentExtraction,
    /// Consolidated from multiple memories
    Consolidation,
}

/// Named entities extracted from memory content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: EntityType,
    pub confidence: f32,
    pub mentions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Person,
    Place,
    Organization,
    Date,
    Event,
    Concept,
    Product,
    Technology,
    Other(String),
}

/// Sentiment analysis of memory content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    pub polarity: f32,    // -1.0 (negative) to 1.0 (positive)
    pub magnitude: f32,   // 0.0 (neutral) to 1.0 (strong)
    pub confidence: f32,  // 0.0 to 1.0
}

/// Context window information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub start_message_id: String,
    pub end_message_id: String,
    pub token_count: usize,
    pub message_count: usize,
}

/// Verification status for memory accuracy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    Unverified,
    Verified,
    Disputed,
    Deprecated,
}

/// Memory query structure for searching and filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub text_query: Option<String>,
    pub semantic_query: Option<Vec<f32>>,
    pub layers: Option<Vec<MemoryLayer>>,
    pub agent_id: Option<String>,
    pub date_range: Option<DateRange>,
    pub importance_threshold: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub entities: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl MemoryQuery {
    pub fn new() -> Self {
        Self {
            text_query: None,
            semantic_query: None,
            layers: None,
            agent_id: None,
            date_range: None,
            importance_threshold: None,
            tags: None,
            entities: None,
            limit: Some(10),
            offset: None,
        }
    }

    pub fn with_text(mut self, query: String) -> Self {
        self.text_query = Some(query);
        self
    }

    pub fn with_agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn with_layers(mut self, layers: Vec<MemoryLayer>) -> Self {
        self.layers = Some(layers);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Memory update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUpdate {
    pub content: Option<String>,
    pub importance_score: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub associations: Option<Vec<Uuid>>,
    pub metadata_updates: Option<HashMap<String, serde_json::Value>>,
}

/// Memory consolidation strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsolidationStrategy {
    /// Use LLM to create summary
    Summarize,
    /// Simple merge of similar memories
    Merge,
    /// Keep all memories (high importance)
    Preserve,
    /// Archive to long-term storage
    Archive,
    /// Remove redundant information
    Deduplicate,
}

/// Result of memory consolidation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationReport {
    pub memories_processed: usize,
    pub memories_consolidated: usize,
    pub memories_archived: usize,
    pub memories_deleted: usize,
    pub new_insights: Vec<Memory>,
    pub processing_time_ms: u64,
    pub space_saved_bytes: usize,
}

/// Memory pruning strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PruningStrategy {
    /// Remove least recently used
    LRU,
    /// Remove lowest importance score
    LowestImportance,
    /// Remove by age threshold
    AgeThreshold(chrono::Duration),
    /// Remove by access frequency
    LowFrequency,
    /// Custom scoring function
    CustomScore,
}

/// Result of memory pruning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningReport {
    pub memories_removed: usize,
    pub space_freed_bytes: usize,
    pub processing_time_ms: u64,
    pub retention_criteria: String,
}

/// AI-generated insights from reflection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: Uuid,
    pub insight_type: InsightType,
    pub content: String,
    pub confidence: f32,
    pub supporting_memories: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InsightType {
    /// Behavioral pattern detected
    Pattern,
    /// User preference inferred
    Preference,
    /// Recurring theme identified
    Theme,
    /// Relationship discovered
    Relationship,
    /// Contradiction detected
    Contradiction,
    /// Goal inferred
    Goal,
    /// Skill level assessment
    SkillAssessment,
}

/// Memory association types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssociationType {
    /// Temporal relationship (happened around same time)
    Temporal,
    /// Semantic similarity
    Semantic,
    /// Causal relationship
    Causal,
    /// Contradictory information
    Contradictory,
    /// Supporting evidence
    Supporting,
    /// Related topic
    TopicalRelation,
    /// User-defined relationship
    UserDefined(String),
}

/// Association between memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAssociation {
    pub id: Uuid,
    pub memory_a: Uuid,
    pub memory_b: Uuid,
    pub association_type: AssociationType,
    pub strength: f32, // 0.0 to 1.0
    pub created_at: DateTime<Utc>,
    pub notes: Option<String>,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let metadata = MemoryMetadata {
            source: MemorySource::UserInput,
            agent_id: "test-agent".to_string(),
            conversation_id: Some("test-conv".to_string()),
            session_id: None,
            topics: vec!["test".to_string()],
            entities: vec![],
            sentiment: None,
            context_window: None,
            verification_status: VerificationStatus::Unverified,
            custom_fields: HashMap::new(),
        };

        let memory = Memory::new(
            "Test memory content".to_string(),
            MemoryLayer::Working,
            metadata
        );

        assert_eq!(memory.content, "Test memory content");
        assert_eq!(memory.layer, MemoryLayer::Working);
        assert_eq!(memory.access_count, 0);
    }

    #[test]
    fn test_memory_strength_calculation() {
        let metadata = MemoryMetadata {
            source: MemorySource::UserInput,
            agent_id: "test-agent".to_string(),
            conversation_id: None,
            session_id: None,
            topics: vec![],
            entities: vec![],
            sentiment: None,
            context_window: None,
            verification_status: VerificationStatus::Verified,
            custom_fields: HashMap::new(),
        };

        let mut memory = Memory::new(
            "Test content".to_string(),
            MemoryLayer::LongTerm,
            metadata
        );
        
        memory.importance_score = 0.8;
        memory.access_count = 5;

        let strength = memory.memory_strength();
        assert!(strength > 0.0);
        assert!(strength <= 1.0);
    }

    #[test]
    fn test_memory_query_builder() {
        let query = MemoryQuery::new()
            .with_text("test query".to_string())
            .with_agent("agent-123".to_string())
            .with_layers(vec![MemoryLayer::Working, MemoryLayer::ShortTerm])
            .with_limit(20);

        assert_eq!(query.text_query, Some("test query".to_string()));
        assert_eq!(query.agent_id, Some("agent-123".to_string()));
        assert_eq!(query.limit, Some(20));
    }

    #[test]
    fn test_layer_priorities() {
        assert!(MemoryLayer::Working.retention_priority() > MemoryLayer::ShortTerm.retention_priority());
        assert!(MemoryLayer::LongTerm.retention_priority() > MemoryLayer::Episodic.retention_priority());
    }

    #[test]
    fn test_recency_score() {
        let metadata = MemoryMetadata {
            source: MemorySource::UserInput,
            agent_id: "test".to_string(),
            conversation_id: None,
            session_id: None,
            topics: vec![],
            entities: vec![],
            sentiment: None,
            context_window: None,
            verification_status: VerificationStatus::Unverified,
            custom_fields: HashMap::new(),
        };

        let memory = Memory::new("test".to_string(), MemoryLayer::Working, metadata);
        let recency = memory.recency_score();
        
        // New memory should have high recency
        assert!(recency > 0.8);
        assert!(recency <= 1.0);
    }
}