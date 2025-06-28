use anyhow::Result;
use std::collections::HashMap;

use super::memory_types::{Memory, MemoryQuery, MemoryLayer, MemorySource};
use super::memory_manager::MemoryManager;

/// Memory retrieval and search engine
pub struct MemoryRetrieval {
    /// Search result cache
    search_cache: HashMap<String, Vec<Memory>>,
    /// Maximum cache size
    max_cache_size: usize,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub memory: Memory,
    pub relevance_score: f32,
    pub match_reasons: Vec<String>,
}

impl MemoryRetrieval {
    pub fn new() -> Self {
        Self {
            search_cache: HashMap::new(),
            max_cache_size: 100,
        }
    }

    /// Search memories based on query parameters
    pub async fn search(&self, memory_manager: &MemoryManager, query: MemoryQuery) -> Result<Vec<Memory>> {
        // Generate cache key
        let cache_key = self.generate_cache_key(&query);
        
        // Check cache first
        if let Some(cached_results) = self.search_cache.get(&cache_key) {
            return Ok(self.apply_pagination(cached_results.clone(), &query));
        }

        // Perform actual search
        let results = self.perform_search(memory_manager, &query).await?;
        
        // Note: In a real implementation, we'd update the cache here
        // For now, we'll just return the results
        Ok(self.apply_pagination(results, &query))
    }

    /// Search memories with detailed scoring
    pub async fn search_with_scores(&self, memory_manager: &MemoryManager, query: MemoryQuery) -> Result<Vec<SearchResult>> {
        let memories = self.perform_search(memory_manager, &query).await?;
        let mut scored_results = Vec::new();

        for memory in memories {
            let (score, reasons) = self.calculate_relevance_score(&memory, &query);
            scored_results.push(SearchResult {
                memory,
                relevance_score: score,
                match_reasons: reasons,
            });
        }

        // Sort by relevance score (highest first)
        scored_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        Ok(scored_results)
    }

    /// Find similar memories to a given memory
    pub async fn find_similar(&self, memory_manager: &MemoryManager, target_memory: &Memory, limit: usize) -> Result<Vec<Memory>> {
        let mut similar_memories = Vec::new();

        // Search across all layers
        for layer in [
            MemoryLayer::Working,
            MemoryLayer::ShortTerm, 
            MemoryLayer::LongTerm,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
            MemoryLayer::Reflective,
        ] {
            let layer_memories = memory_manager.get_memories_by_layer(layer);
            
            for memory in layer_memories {
                if memory.id == target_memory.id {
                    continue; // Skip the target memory itself
                }

                let similarity = self.calculate_similarity(target_memory, memory);
                if similarity > 0.3 { // Minimum similarity threshold
                    similar_memories.push((memory.clone(), similarity));
                }
            }
        }

        // Sort by similarity and take top results
        similar_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        Ok(similar_memories
            .into_iter()
            .take(limit)
            .map(|(memory, _)| memory)
            .collect())
    }

    /// Search memories by semantic similarity (requires embeddings)
    pub async fn semantic_search(&self, memory_manager: &MemoryManager, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<Memory>> {
        let mut results = Vec::new();

        // Search across all layers for memories with embeddings
        for layer in [
            MemoryLayer::Working,
            MemoryLayer::ShortTerm,
            MemoryLayer::LongTerm,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
            MemoryLayer::Reflective,
        ] {
            let layer_memories = memory_manager.get_memories_by_layer(layer);
            
            for memory in layer_memories {
                if let Some(ref embedding) = memory.embedding {
                    let similarity = self.cosine_similarity(&query_embedding, embedding);
                    if similarity > 0.5 { // Minimum semantic similarity threshold
                        results.push((memory.clone(), similarity));
                    }
                }
            }
        }

        // Sort by semantic similarity
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        Ok(results
            .into_iter()
            .take(limit)
            .map(|(memory, _)| memory)
            .collect())
    }

    /// Get recent memories for context
    pub async fn get_recent_context(&self, memory_manager: &MemoryManager, agent_id: &str, limit: usize) -> Result<Vec<Memory>> {
        let mut recent_memories = Vec::new();

        // Prioritize working and short-term memories for context
        for layer in [MemoryLayer::Working, MemoryLayer::ShortTerm] {
            let layer_memories = memory_manager.get_memories_by_layer(layer);
            
            for memory in layer_memories {
                if memory.metadata.agent_id == agent_id {
                    recent_memories.push(memory.clone());
                }
            }
        }

        // Sort by creation time (most recent first)
        recent_memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(recent_memories.into_iter().take(limit).collect())
    }

    /// Search memories by topic
    pub async fn search_by_topic(&self, memory_manager: &MemoryManager, topic: &str, agent_id: Option<&str>) -> Result<Vec<Memory>> {
        let mut topic_memories = Vec::new();
        let topic_lower = topic.to_lowercase();

        for layer in [
            MemoryLayer::Working,
            MemoryLayer::ShortTerm,
            MemoryLayer::LongTerm,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
            MemoryLayer::Reflective,
        ] {
            let layer_memories = memory_manager.get_memories_by_layer(layer);
            
            for memory in layer_memories {
                // Filter by agent if specified
                if let Some(agent_id) = agent_id {
                    if memory.metadata.agent_id != agent_id {
                        continue;
                    }
                }

                // Check if memory contains the topic
                let has_topic = memory.metadata.topics.iter()
                    .any(|t| t.to_lowercase().contains(&topic_lower));

                if has_topic {
                    topic_memories.push(memory.clone());
                }
            }
        }

        // Sort by importance and recency
        topic_memories.sort_by(|a, b| {
            let score_a = a.importance_score + a.recency_score() * 0.3;
            let score_b = b.importance_score + b.recency_score() * 0.3;
            score_b.partial_cmp(&score_a).unwrap()
        });

        Ok(topic_memories)
    }

    /// Search memories containing specific entities
    pub async fn search_by_entity(&self, memory_manager: &MemoryManager, entity_name: &str, agent_id: Option<&str>) -> Result<Vec<Memory>> {
        let mut entity_memories = Vec::new();
        let entity_lower = entity_name.to_lowercase();

        for layer in [
            MemoryLayer::Working,
            MemoryLayer::ShortTerm,
            MemoryLayer::LongTerm,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
            MemoryLayer::Reflective,
        ] {
            let layer_memories = memory_manager.get_memories_by_layer(layer);
            
            for memory in layer_memories {
                // Filter by agent if specified
                if let Some(agent_id) = agent_id {
                    if memory.metadata.agent_id != agent_id {
                        continue;
                    }
                }

                // Check if memory contains the entity
                let has_entity = memory.metadata.entities.iter()
                    .any(|entity| entity.name.to_lowercase().contains(&entity_lower));

                if has_entity {
                    entity_memories.push(memory.clone());
                }
            }
        }

        // Sort by importance
        entity_memories.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap());

        Ok(entity_memories)
    }

    // Private helper methods

    async fn perform_search(&self, memory_manager: &MemoryManager, query: &MemoryQuery) -> Result<Vec<Memory>> {
        let mut candidates = Vec::new();

        // Determine which layers to search
        let layers_to_search = if let Some(ref layers) = query.layers {
            layers.clone()
        } else {
            vec![
                MemoryLayer::Working,
                MemoryLayer::ShortTerm,
                MemoryLayer::LongTerm,
                MemoryLayer::Episodic,
                MemoryLayer::Semantic,
                MemoryLayer::Reflective,
            ]
        };

        // Collect candidate memories
        for layer in layers_to_search {
            let layer_memories = memory_manager.get_memories_by_layer(layer);
            candidates.extend(layer_memories.into_iter().cloned());
        }

        // Apply filters
        let mut filtered_memories = Vec::new();
        
        for memory in candidates {
            if self.matches_query(&memory, query) {
                filtered_memories.push(memory);
            }
        }

        // Score and sort results
        let mut scored_memories: Vec<(Memory, f32)> = filtered_memories
            .into_iter()
            .map(|memory| {
                let (score, _) = self.calculate_relevance_score(&memory, query);
                (memory, score)
            })
            .collect();

        scored_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(scored_memories.into_iter().map(|(memory, _)| memory).collect())
    }

    fn matches_query(&self, memory: &Memory, query: &MemoryQuery) -> bool {
        // Agent filter
        if let Some(ref agent_id) = query.agent_id {
            if memory.metadata.agent_id != *agent_id {
                return false;
            }
        }

        // Importance threshold filter
        if let Some(threshold) = query.importance_threshold {
            if memory.importance_score < threshold {
                return false;
            }
        }

        // Date range filter
        if let Some(ref date_range) = query.date_range {
            if memory.created_at < date_range.start || memory.created_at > date_range.end {
                return false;
            }
        }

        // Tags filter
        if let Some(ref tags) = query.tags {
            let has_any_tag = tags.iter().any(|tag| memory.tags.contains(tag));
            if !has_any_tag {
                return false;
            }
        }

        // Entities filter
        if let Some(ref entities) = query.entities {
            let has_any_entity = entities.iter().any(|entity_name| {
                memory.metadata.entities.iter().any(|entity| entity.name == *entity_name)
            });
            if !has_any_entity {
                return false;
            }
        }

        // Text query filter
        if let Some(ref text_query) = query.text_query {
            if !self.matches_text_query(memory, text_query) {
                return false;
            }
        }

        true
    }

    fn matches_text_query(&self, memory: &Memory, text_query: &str) -> bool {
        let content_lower = memory.content.to_lowercase();
        let query_lower = text_query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        // Check for exact phrase match
        if content_lower.contains(&query_lower) {
            return true;
        }

        // Check for individual word matches
        let word_matches = query_words.iter()
            .filter(|word| content_lower.contains(*word))
            .count();

        // Require at least 50% of words to match
        word_matches as f32 / query_words.len() as f32 >= 0.5
    }

    fn calculate_relevance_score(&self, memory: &Memory, query: &MemoryQuery) -> (f32, Vec<String>) {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // Base importance score
        score += memory.importance_score * 0.3;
        reasons.push(format!("Importance: {:.2}", memory.importance_score));

        // Recency score
        let recency = memory.recency_score();
        score += recency * 0.2;
        reasons.push(format!("Recency: {:.2}", recency));

        // Access frequency score
        let frequency = memory.frequency_score();
        score += frequency * 0.1;
        if frequency > 0.0 {
            reasons.push(format!("Access frequency: {:.2}", frequency));
        }

        // Text relevance score
        if let Some(ref text_query) = query.text_query {
            let text_score = self.calculate_text_relevance_score(memory, text_query);
            score += text_score * 0.4;
            if text_score > 0.0 {
                reasons.push(format!("Text relevance: {:.2}", text_score));
            }
        }

        // Layer relevance (working and short-term memories are more relevant for current context)
        let layer_score = match memory.layer {
            MemoryLayer::Working => 0.1,
            MemoryLayer::ShortTerm => 0.08,
            MemoryLayer::LongTerm => 0.06,
            MemoryLayer::Episodic => 0.04,
            MemoryLayer::Semantic => 0.05,
            MemoryLayer::Reflective => 0.07,
        };
        score += layer_score;

        (score.min(1.0), reasons)
    }

    fn calculate_text_relevance_score(&self, memory: &Memory, text_query: &str) -> f32 {
        let content_lower = memory.content.to_lowercase();
        let query_lower = text_query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        if query_words.is_empty() {
            return 0.0;
        }

        // Exact phrase match gets highest score
        if content_lower.contains(&query_lower) {
            return 1.0;
        }

        // Calculate word overlap score
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();
        let content_word_set: std::collections::HashSet<_> = content_words.iter().collect();
        let query_word_set: std::collections::HashSet<_> = query_words.iter().collect();

        let intersection = content_word_set.intersection(&query_word_set).count();
        let union = content_word_set.union(&query_word_set).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    fn calculate_similarity(&self, memory1: &Memory, memory2: &Memory) -> f32 {
        let mut similarity = 0.0;
        let mut factors = 0;

        // Topic similarity
        let topics1: std::collections::HashSet<_> = memory1.metadata.topics.iter().collect();
        let topics2: std::collections::HashSet<_> = memory2.metadata.topics.iter().collect();
        
        if !topics1.is_empty() || !topics2.is_empty() {
            let intersection = topics1.intersection(&topics2).count();
            let union = topics1.union(&topics2).count();
            if union > 0 {
                similarity += intersection as f32 / union as f32;
                factors += 1;
            }
        }

        // Content similarity
        let content_similarity = self.calculate_text_similarity(&memory1.content, &memory2.content);
        similarity += content_similarity;
        factors += 1;

        // Entity similarity
        let entities1: std::collections::HashSet<_> = memory1.metadata.entities.iter().map(|e| &e.name).collect();
        let entities2: std::collections::HashSet<_> = memory2.metadata.entities.iter().map(|e| &e.name).collect();
        
        if !entities1.is_empty() || !entities2.is_empty() {
            let intersection = entities1.intersection(&entities2).count();
            let union = entities1.union(&entities2).count();
            if union > 0 {
                similarity += intersection as f32 / union as f32;
                factors += 1;
            }
        }

        if factors > 0 {
            similarity / factors as f32
        } else {
            0.0
        }
    }

    fn calculate_text_similarity(&self, text1: &str, text2: &str) -> f32 {
        let words1: std::collections::HashSet<_> = text1
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();
        let words2: std::collections::HashSet<_> = text2
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let magnitude1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude1 * magnitude2)
    }

    fn apply_pagination(&self, mut memories: Vec<Memory>, query: &MemoryQuery) -> Vec<Memory> {
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(10);

        if offset >= memories.len() {
            return Vec::new();
        }

        let end = (offset + limit).min(memories.len());
        memories.drain(offset..end).collect()
    }

    fn generate_cache_key(&self, query: &MemoryQuery) -> String {
        // Generate a simple cache key based on query parameters
        format!(
            "{}|{}|{}|{}",
            query.text_query.as_deref().unwrap_or(""),
            query.agent_id.as_deref().unwrap_or(""),
            query.importance_threshold.unwrap_or(0.0),
            query.limit.unwrap_or(10)
        )
    }
}

impl Default for MemoryRetrieval {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::memory_types::*;
    use std::collections::HashMap;
    use chrono::Utc;

    fn create_test_memory(content: &str, agent_id: &str, importance: f32) -> Memory {
        let metadata = MemoryMetadata {
            source: MemorySource::UserInput,
            agent_id: agent_id.to_string(),
            conversation_id: None,
            session_id: None,
            topics: vec!["test".to_string()],
            entities: vec![],
            sentiment: None,
            context_window: None,
            verification_status: VerificationStatus::Unverified,
            custom_fields: HashMap::new(),
        };

        let mut memory = Memory::new(content.to_string(), MemoryLayer::Working, metadata);
        memory.importance_score = importance;
        memory
    }

    #[test]
    fn test_text_relevance_scoring() {
        let retrieval = MemoryRetrieval::new();
        let memory = create_test_memory("I love programming in Python", "agent1", 0.5);
        
        let high_relevance = retrieval.calculate_text_relevance_score(&memory, "Python programming");
        let low_relevance = retrieval.calculate_text_relevance_score(&memory, "Java development");
        
        assert!(high_relevance > low_relevance);
        assert!(high_relevance > 0.5);
    }

    #[test]
    fn test_text_query_matching() {
        let retrieval = MemoryRetrieval::new();
        let memory = create_test_memory("I need to remember my Python programming tasks", "agent1", 0.5);
        
        assert!(retrieval.matches_text_query(&memory, "Python programming"));
        assert!(retrieval.matches_text_query(&memory, "remember tasks"));
        assert!(!retrieval.matches_text_query(&memory, "Java development"));
    }

    #[test]
    fn test_memory_similarity() {
        let retrieval = MemoryRetrieval::new();
        
        let memory1 = create_test_memory("I love Python programming", "agent1", 0.5);
        let memory2 = create_test_memory("Python is great for development", "agent1", 0.5);
        let memory3 = create_test_memory("Weather is nice today", "agent1", 0.5);
        
        let similarity_high = retrieval.calculate_similarity(&memory1, &memory2);
        let similarity_low = retrieval.calculate_similarity(&memory1, &memory3);
        
        assert!(similarity_high > similarity_low);
    }

    #[test]
    fn test_cosine_similarity() {
        let retrieval = MemoryRetrieval::new();
        
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];
        let vec3 = vec![0.0, 1.0, 0.0];
        
        assert_eq!(retrieval.cosine_similarity(&vec1, &vec2), 1.0);
        assert_eq!(retrieval.cosine_similarity(&vec1, &vec3), 0.0);
    }

    #[test]
    fn test_query_matching() {
        let retrieval = MemoryRetrieval::new();
        let memory = create_test_memory("Test content", "agent1", 0.7);
        
        // Test agent filter
        let query = MemoryQuery::new().with_agent("agent1".to_string());
        assert!(retrieval.matches_query(&memory, &query));
        
        let query = MemoryQuery::new().with_agent("agent2".to_string());
        assert!(!retrieval.matches_query(&memory, &query));
        
        // Test importance threshold
        let mut query = MemoryQuery::new();
        query.importance_threshold = Some(0.5);
        assert!(retrieval.matches_query(&memory, &query));
        
        query.importance_threshold = Some(0.9);
        assert!(!retrieval.matches_query(&memory, &query));
    }

    #[test]
    fn test_pagination() {
        let retrieval = MemoryRetrieval::new();
        
        let memories = vec![
            create_test_memory("Memory 1", "agent1", 0.5),
            create_test_memory("Memory 2", "agent1", 0.6),
            create_test_memory("Memory 3", "agent1", 0.7),
            create_test_memory("Memory 4", "agent1", 0.8),
            create_test_memory("Memory 5", "agent1", 0.9),
        ];
        
        let mut query = MemoryQuery::new();
        query.limit = Some(3);
        query.offset = Some(1);
        
        let paginated = retrieval.apply_pagination(memories, &query);
        assert_eq!(paginated.len(), 3);
    }
}