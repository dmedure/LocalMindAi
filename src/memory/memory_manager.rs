use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::memory_types::*;
use super::importance_scorer::ImportanceScorer;
use super::consolidation::ConsolidationEngine;
use super::retrieval::MemoryRetrieval;
use crate::vector::VectorStore;
use crate::config::AppConfig;

/// Core memory management system implementing MemGPT-style hierarchical memory
pub struct MemoryManager {
    /// Memory storage organized by layer
    memories_by_layer: HashMap<MemoryLayer, HashMap<Uuid, Memory>>,
    /// Memory associations
    associations: HashMap<Uuid, MemoryAssociation>,
    /// Importance scoring system
    importance_scorer: ImportanceScorer,
    /// Memory consolidation engine
    consolidation_engine: ConsolidationEngine,
    /// Memory retrieval system
    retrieval_engine: MemoryRetrieval,
    /// Vector store for semantic search
    vector_store: Option<VectorStore>,
    /// Configuration
    config: AppConfig,
    /// Statistics
    stats: MemoryStats,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub memories_by_layer: HashMap<MemoryLayer, usize>,
    pub total_associations: usize,
    pub last_consolidation: Option<DateTime<Utc>>,
    pub average_importance: f32,
}

impl MemoryManager {
    pub async fn new(config: AppConfig, vector_store: Option<VectorStore>) -> Result<Self> {
        let importance_scorer = ImportanceScorer::new();
        let consolidation_engine = ConsolidationEngine::new();
        let retrieval_engine = MemoryRetrieval::new();

        let mut memories_by_layer = HashMap::new();
        for layer in [
            MemoryLayer::Working,
            MemoryLayer::ShortTerm,
            MemoryLayer::LongTerm,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
            MemoryLayer::Reflective,
        ] {
            memories_by_layer.insert(layer, HashMap::new());
        }

        Ok(Self {
            memories_by_layer,
            associations: HashMap::new(),
            importance_scorer,
            consolidation_engine,
            retrieval_engine,
            vector_store,
            config,
            stats: MemoryStats {
                total_memories: 0,
                memories_by_layer: HashMap::new(),
                total_associations: 0,
                last_consolidation: None,
                average_importance: 0.0,
            },
        })
    }

    /// Store a new memory
    pub async fn store(&mut self, content: String, metadata: MemoryMetadata) -> Result<Memory> {
        // Create the memory
        let mut memory = Memory::new(content.clone(), MemoryLayer::Working, metadata.clone());
        
        // Calculate importance score
        memory.importance_score = self.importance_scorer.calculate_importance(&memory).await?;
        
        // Extract entities and topics
        self.extract_metadata(&mut memory).await?;
        
        // Generate embedding if vector store is available
        if let Some(ref vector_store) = self.vector_store {
            memory.embedding = Some(vector_store.generate_embedding(&content).await?);
        }
        
        // Determine appropriate layer based on content and importance
        let target_layer = self.determine_initial_layer(&memory);
        memory.layer = target_layer;
        
        // Store the memory
        let memory_id = memory.id;
        self.memories_by_layer
            .get_mut(&target_layer)
            .unwrap()
            .insert(memory_id, memory.clone());
        
        // Update statistics
        self.update_stats();
        
        // Check if we need to manage capacity
        self.manage_layer_capacity(target_layer).await?;
        
        // Create associations with similar memories
        self.create_automatic_associations(&memory).await?;
        
        Ok(memory)
    }

    /// Retrieve memories based on query
    pub async fn retrieve(&self, query: MemoryQuery) -> Result<Vec<Memory>> {
        self.retrieval_engine.search(self, query).await
    }

    /// Update an existing memory
    pub async fn update(&mut self, id: Uuid, updates: MemoryUpdate) -> Result<Memory> {
        // Find the memory
        let (layer, mut memory) = self.find_memory_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Memory not found: {}", id))?;
        
        // Apply updates
        if let Some(content) = updates.content {
            memory.content = content;
            // Recalculate importance if content changed
            memory.importance_score = self.importance_scorer.calculate_importance(&memory).await?;
        }
        
        if let Some(importance) = updates.importance_score {
            memory.importance_score = importance;
        }
        
        if let Some(tags) = updates.tags {
            memory.tags = tags;
        }
        
        if let Some(associations) = updates.associations {
            memory.associations = associations;
        }
        
        if let Some(metadata_updates) = updates.metadata_updates {
            for (key, value) in metadata_updates {
                memory.metadata.custom_fields.insert(key, value);
            }
        }
        
        let updated_memory = memory.clone();
        
        // Check if memory should move to a different layer
        let new_layer = self.determine_target_layer(&memory);
        if new_layer != layer {
            self.move_memory_to_layer(id, layer, new_layer)?;
        }
        
        self.update_stats();
        Ok(updated_memory)
    }

    /// Delete a memory
    pub async fn forget(&mut self, id: Uuid) -> Result<()> {
        // Find and remove the memory
        let layer = self.find_memory_layer(id)
            .ok_or_else(|| anyhow::anyhow!("Memory not found: {}", id))?;
        
        self.memories_by_layer
            .get_mut(&layer)
            .unwrap()
            .remove(&id);
        
        // Remove associated associations
        self.associations.retain(|_, assoc| {
            assoc.memory_a != id && assoc.memory_b != id
        });
        
        // Remove from vector store if available
        if let Some(ref vector_store) = self.vector_store {
            vector_store.delete_embedding(id).await?;
        }
        
        self.update_stats();
        Ok(())
    }

    /// Consolidate memories to manage capacity and create insights
    pub async fn consolidate(&mut self) -> Result<ConsolidationReport> {
        let report = self.consolidation_engine.consolidate_memories(self).await?;
        
        self.stats.last_consolidation = Some(Utc::now());
        self.update_stats();
        
        Ok(report)
    }

    /// Generate reflective insights from memories
    pub async fn reflect(&mut self) -> Result<Vec<Insight>> {
        self.consolidation_engine.generate_insights(self).await
    }

    /// Create an association between two memories
    pub async fn associate(&mut self, memory1: Uuid, memory2: Uuid, association_type: AssociationType, strength: f32) -> Result<()> {
        let association = MemoryAssociation {
            id: Uuid::new_v4(),
            memory_a: memory1,
            memory_b: memory2,
            association_type,
            strength: strength.clamp(0.0, 1.0),
            created_at: Utc::now(),
            notes: None,
        };
        
        self.associations.insert(association.id, association);
        
        // Update memory association lists
        if let Some((_, memory)) = self.find_memory_mut(memory1) {
            if !memory.associations.contains(&memory2) {
                memory.associations.push(memory2);
            }
        }
        
        if let Some((_, memory)) = self.find_memory_mut(memory2) {
            if !memory.associations.contains(&memory1) {
                memory.associations.push(memory1);
            }
        }
        
        Ok(())
    }

    /// Prune memories based on strategy
    pub async fn prune(&mut self, strategy: PruningStrategy) -> Result<PruningReport> {
        let start_time = std::time::Instant::now();
        let initial_count = self.total_memory_count();
        
        let mut removed_count = 0;
        let mut space_freed = 0;
        
        match strategy {
            PruningStrategy::LRU => {
                removed_count = self.prune_lru().await?;
            },
            PruningStrategy::LowestImportance => {
                removed_count = self.prune_by_importance().await?;
            },
            PruningStrategy::AgeThreshold(threshold) => {
                removed_count = self.prune_by_age(threshold).await?;
            },
            PruningStrategy::LowFrequency => {
                removed_count = self.prune_by_frequency().await?;
            },
            PruningStrategy::CustomScore => {
                removed_count = self.prune_by_custom_score().await?;
            },
        }
        
        // Estimate space freed (rough calculation)
        space_freed = removed_count * 1024; // Assume 1KB per memory on average
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        self.update_stats();
        
        Ok(PruningReport {
            memories_removed: removed_count,
            space_freed_bytes: space_freed,
            processing_time_ms: processing_time,
            retention_criteria: format!("{:?}", strategy),
        })
    }

    /// Get all memories for an agent
    pub async fn get_agent_memories(&self, agent_id: &str) -> Result<Vec<Memory>> {
        let mut memories = Vec::new();
        
        for layer_memories in self.memories_by_layer.values() {
            for memory in layer_memories.values() {
                if memory.metadata.agent_id == agent_id {
                    memories.push(memory.clone());
                }
            }
        }
        
        // Sort by creation time, most recent first
        memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(memories)
    }

    /// Get memories by layer
    pub fn get_memories_by_layer(&self, layer: MemoryLayer) -> Vec<&Memory> {
        self.memories_by_layer
            .get(&layer)
            .map(|memories| memories.values().collect())
            .unwrap_or_default()
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Clear all memories for an agent
    pub async fn clear_agent_memories(&mut self, agent_id: &str) -> Result<usize> {
        let mut removed_count = 0;
        
        for layer_memories in self.memories_by_layer.values_mut() {
            layer_memories.retain(|_, memory| {
                if memory.metadata.agent_id == agent_id {
                    removed_count += 1;
                    false
                } else {
                    true
                }
            });
        }
        
        // Remove associations for deleted memories
        self.associations.retain(|_, assoc| {
            // Check if either memory in the association belonged to the agent
            !self.association_involves_agent(&assoc, agent_id)
        });
        
        self.update_stats();
        Ok(removed_count)
    }

    // Private helper methods

    fn find_memory_mut(&mut self, id: Uuid) -> Option<(MemoryLayer, &mut Memory)> {
        for (layer, memories) in self.memories_by_layer.iter_mut() {
            if let Some(memory) = memories.get_mut(&id) {
                return Some(*layer, memory);
            }
        }
        None
    }

    fn find_memory_layer(&self, id: Uuid) -> Option<MemoryLayer> {
        for (layer, memories) in self.memories_by_layer.iter() {
            if memories.contains_key(&id) {
                return Some(*layer);
            }
        }
        None
    }

    fn move_memory_to_layer(&mut self, id: Uuid, from_layer: MemoryLayer, to_layer: MemoryLayer) -> Result<()> {
        let memory = self.memories_by_layer
            .get_mut(&from_layer)
            .unwrap()
            .remove(&id)
            .ok_or_else(|| anyhow::anyhow!("Memory not found in source layer"))?;
        
        self.memories_by_layer
            .get_mut(&to_layer)
            .unwrap()
            .insert(id, memory);
        
        Ok(())
    }

    fn determine_initial_layer(&self, memory: &Memory) -> MemoryLayer {
        // New memories start in Working layer
        // They'll be moved during consolidation based on importance and access patterns
        MemoryLayer::Working
    }

    fn determine_target_layer(&self, memory: &Memory) -> MemoryLayer {
        let strength = memory.memory_strength();
        let importance = memory.importance_score;
        
        match memory.metadata.source {
            MemorySource::Reflection => MemoryLayer::Reflective,
            MemorySource::SystemInsight => MemoryLayer::Semantic,
            _ => {
                if importance > 0.8 || strength > 0.9 {
                    MemoryLayer::LongTerm
                } else if importance > 0.6 || strength > 0.7 {
                    MemoryLayer::Episodic
                } else if memory.access_count > 3 {
                    MemoryLayer::ShortTerm
                } else {
                    MemoryLayer::Working
                }
            }
        }
    }

    async fn manage_layer_capacity(&mut self, layer: MemoryLayer) -> Result<()> {
        let capacity = layer.typical_capacity();
        let current_count = self.memories_by_layer[&layer].len();
        
        if current_count > capacity {
            let excess = current_count - capacity;
            self.remove_least_important_from_layer(layer, excess).await?;
        }
        
        Ok(())
    }

    async fn remove_least_important_from_layer(&mut self, layer: MemoryLayer, count: usize) -> Result<()> {
        let layer_memories = self.memories_by_layer.get(&layer).unwrap();
        let mut memory_scores: Vec<(Uuid, f32)> = layer_memories
            .iter()
            .map(|(id, memory)| (*id, memory.memory_strength()))
            .collect();
        
        // Sort by strength (lowest first)
        memory_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Remove the lowest scoring memories
        for (id, _) in memory_scores.iter().take(count) {
            self.forget(*id).await?;
        }
        
        Ok(())
    }

    async fn extract_metadata(&mut self, memory: &mut Memory) -> Result<()> {
        // Simple keyword-based topic extraction
        let content_lower = memory.content.to_lowercase();
        let mut topics = Vec::new();
        
        // Basic topic keywords - in a real implementation, use NLP
        let topic_keywords = [
            ("coding", vec!["code", "programming", "function", "variable", "debug"]),
            ("work", vec!["meeting", "project", "deadline", "task", "client"]),
            ("personal", vec!["family", "friend", "hobby", "vacation", "health"]),
        ];
        
        for (topic, keywords) in topic_keywords.iter() {
            if keywords.iter().any(|keyword| content_lower.contains(keyword)) {
                topics.push(topic.to_string());
            }
        }
        
        memory.metadata.topics = topics;
        
        // Extract simple entities (capitalized words)
        let words: Vec<&str> = memory.content.split_whitespace().collect();
        let mut entities = Vec::new();
        
        for word in words {
            if word.chars().next().unwrap_or_default().is_uppercase() && word.len() > 2 {
                entities.push(Entity {
                    name: word.to_string(),
                    entity_type: EntityType::Other("detected".to_string()),
                    confidence: 0.7,
                    mentions: vec![word.to_string()],
                });
            }
        }
        
        memory.metadata.entities = entities;
        
        Ok(())
    }

    async fn create_automatic_associations(&mut self, memory: &Memory) -> Result<()> {
        // Find similar memories for association
        let similar_memories = self.find_similar_memories(memory, 5).await?;
        
        for similar_memory in similar_memories {
            let similarity = self.calculate_similarity(memory, &similar_memory);
            if similarity > 0.7 {
                self.associate(
                    memory.id,
                    similar_memory.id,
                    AssociationType::Semantic,
                    similarity
                ).await?;
            }
        }
        
        Ok(())
    }

    async fn find_similar_memories(&self, memory: &Memory, limit: usize) -> Result<Vec<Memory>> {
        // Simple similarity based on shared topics and entities
        let mut similar = Vec::new();
        
        for layer_memories in self.memories_by_layer.values() {
            for candidate in layer_memories.values() {
                if candidate.id == memory.id {
                    continue;
                }
                
                let similarity = self.calculate_similarity(memory, candidate);
                if similarity > 0.5 {
                    similar.push((candidate.clone(), similarity));
                }
            }
        }
        
        // Sort by similarity and take top results
        similar.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(similar.into_iter().take(limit).map(|(mem, _)| mem).collect())
    }

    fn calculate_similarity(&self, memory1: &Memory, memory2: &Memory) -> f32 {
        // Simple Jaccard similarity for topics
        let topics1: std::collections::HashSet<_> = memory1.metadata.topics.iter().collect();
        let topics2: std::collections::HashSet<_> = memory2.metadata.topics.iter().collect();
        
        let intersection = topics1.intersection(&topics2).count();
        let union = topics1.union(&topics2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    fn total_memory_count(&self) -> usize {
        self.memories_by_layer.values().map(|m| m.len()).sum()
    }

    fn update_stats(&mut self) {
        self.stats.total_memories = self.total_memory_count();
        
        self.stats.memories_by_layer.clear();
        for (layer, memories) in &self.memories_by_layer {
            self.stats.memories_by_layer.insert(*layer, memories.len());
        }
        
        self.stats.total_associations = self.associations.len();
        
        // Calculate average importance
        let all_memories: Vec<&Memory> = self.memories_by_layer
            .values()
            .flat_map(|m| m.values())
            .collect();
        
        if !all_memories.is_empty() {
            let total_importance: f32 = all_memories.iter().map(|m| m.importance_score).sum();
            self.stats.average_importance = total_importance / all_memories.len() as f32;
        }
    }

    async fn prune_lru(&mut self) -> Result<usize> {
        // Implementation for LRU pruning
        // Find least recently used memories and remove them
        // This is a simplified version
        Ok(0)
    }

    async fn prune_by_importance(&mut self) -> Result<usize> {
        // Remove memories with lowest importance scores
        Ok(0)
    }

    async fn prune_by_age(&mut self, _threshold: chrono::Duration) -> Result<usize> {
        // Remove memories older than threshold
        Ok(0)
    }

    async fn prune_by_frequency(&mut self) -> Result<usize> {
        // Remove memories with low access frequency
        Ok(0)
    }

    async fn prune_by_custom_score(&mut self) -> Result<usize> {
        // Custom scoring algorithm for pruning
        Ok(0)
    }

    fn association_involves_agent(&self, _association: &MemoryAssociation, _agent_id: &str) -> bool {
        // Check if association involves memories from the specified agent
        // This would require looking up the memories by ID
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn create_test_metadata(agent_id: &str) -> MemoryMetadata {
        MemoryMetadata {
            source: MemorySource::UserInput,
            agent_id: agent_id.to_string(),
            conversation_id: None,
            session_id: None,
            topics: vec![],
            entities: vec![],
            sentiment: None,
            context_window: None,
            verification_status: VerificationStatus::Unverified,
            custom_fields: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let config = AppConfig::default();
        let manager = MemoryManager::new(config, None).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_store_memory() {
        let config = AppConfig::default();
        let mut manager = MemoryManager::new(config, None).await.unwrap();
        
        let metadata = create_test_metadata("test-agent");
        let memory = manager.store("Test memory content".to_string(), metadata).await.unwrap();
        
        assert_eq!(memory.content, "Test memory content");
        assert_eq!(memory.layer, MemoryLayer::Working);
    }

    #[tokio::test]
    async fn test_retrieve_memory() {
        let config = AppConfig::default();
        let mut manager = MemoryManager::new(config, None).await.unwrap();
        
        let metadata = create_test_metadata("test-agent");
        let stored_memory = manager.store("Test memory content".to_string(), metadata).await.unwrap();
        
        let query = MemoryQuery::new().with_agent("test-agent".to_string());
        let retrieved = manager.retrieve(query).await.unwrap();
        
        assert!(!retrieved.is_empty());
        assert_eq!(retrieved[0].id, stored_memory.id);
    }
}