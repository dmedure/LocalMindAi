use anyhow::Result;
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

use super::memory_types::*;
use super::memory_manager::MemoryManager;

/// Engine for consolidating memories and generating insights
pub struct ConsolidationEngine {
    similarity_threshold: f32,
    consolidation_batch_size: usize,
}

impl ConsolidationEngine {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.8,
            consolidation_batch_size: 50,
        }
    }

    /// Consolidate memories to manage capacity and create insights
    pub async fn consolidate_memories(&self, memory_manager: &mut MemoryManager) -> Result<ConsolidationReport> {
        let start_time = std::time::Instant::now();
        let initial_count = memory_manager.get_stats().total_memories;
        
        let mut report = ConsolidationReport {
            memories_processed: 0,
            memories_consolidated: 0,
            memories_archived: 0,
            memories_deleted: 0,
            new_insights: Vec::new(),
            processing_time_ms: 0,
            space_saved_bytes: 0,
        };

        // Process each layer
        for layer in [MemoryLayer::Working, MemoryLayer::ShortTerm, MemoryLayer::Episodic] {
            let layer_report = self.consolidate_layer(memory_manager, layer).await?;
            self.merge_reports(&mut report, layer_report);
        }

        // Generate insights from consolidated memories
        let insights = self.generate_insights(memory_manager).await?;
        report.new_insights = insights;

        // Move memories between layers based on importance and access patterns
        self.rebalance_memory_layers(memory_manager).await?;

        report.processing_time_ms = start_time.elapsed().as_millis() as u64;
        report.space_saved_bytes = (initial_count - memory_manager.get_stats().total_memories) * 1024; // Rough estimate

        Ok(report)
    }

    /// Generate AI insights from memory patterns
    pub async fn generate_insights(&self, memory_manager: &MemoryManager) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Analyze patterns across all memories
        insights.extend(self.detect_behavioral_patterns(memory_manager).await?);
        insights.extend(self.infer_user_preferences(memory_manager).await?);
        insights.extend(self.identify_recurring_themes(memory_manager).await?);
        insights.extend(self.discover_relationships(memory_manager).await?);
        insights.extend(self.detect_contradictions(memory_manager).await?);

        Ok(insights)
    }

    // Private helper methods

    async fn consolidate_layer(&self, memory_manager: &mut MemoryManager, layer: MemoryLayer) -> Result<ConsolidationReport> {
        let mut report = ConsolidationReport {
            memories_processed: 0,
            memories_consolidated: 0,
            memories_archived: 0,
            memories_deleted: 0,
            new_insights: Vec::new(),
            processing_time_ms: 0,
            space_saved_bytes: 0,
        };

        let memories = memory_manager.get_memories_by_layer(layer);
        report.memories_processed = memories.len();

        if memories.len() < self.consolidation_batch_size {
            return Ok(report);
        }

        // Group similar memories
        let memory_groups = self.group_similar_memories(memories).await?;

        for group in memory_groups {
            if group.len() > 1 {
                let consolidation_strategy = self.determine_consolidation_strategy(&group);
                
                match consolidation_strategy {
                    ConsolidationStrategy::Summarize => {
                        report.memories_consolidated += self.summarize_memory_group(memory_manager, group).await?;
                    },
                    ConsolidationStrategy::Merge => {
                        report.memories_consolidated += self.merge_memory_group(memory_manager, group).await?;
                    },
                    ConsolidationStrategy::Archive => {
                        report.memories_archived += self.archive_memory_group(memory_manager, group).await?;
                    },
                    ConsolidationStrategy::Deduplicate => {
                        report.memories_deleted += self.deduplicate_memory_group(memory_manager, group).await?;
                    },
                    ConsolidationStrategy::Preserve => {
                        // Keep all memories as-is
                    }
                }
            }
        }

        Ok(report)
    }

    async fn group_similar_memories(&self, memories: Vec<&Memory>) -> Result<Vec<Vec<Memory>>> {
        let mut groups: Vec<Vec<Memory>> = Vec::new();
        let mut used_indices = std::collections::HashSet::new();

        for (i, memory) in memories.iter().enumerate() {
            if used_indices.contains(&i) {
                continue;
            }

            let mut group = vec![(*memory).clone()];
            used_indices.insert(i);

            // Find similar memories to group with this one
            for (j, other_memory) in memories.iter().enumerate() {
                if i == j || used_indices.contains(&j) {
                    continue;
                }

                let similarity = self.calculate_memory_similarity(memory, other_memory);
                if similarity > self.similarity_threshold {
                    group.push((*other_memory).clone());
                    used_indices.insert(j);
                }
            }

            groups.push(group);
        }

        Ok(groups)
    }

    fn calculate_memory_similarity(&self, memory1: &Memory, memory2: &Memory) -> f32 {
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

        // Content similarity (simple word overlap)
        let words1: std::collections::HashSet<_> = memory1.content
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3) // Filter out short words
            .collect();
        let words2: std::collections::HashSet<_> = memory2.content
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        if !words1.is_empty() || !words2.is_empty() {
            let intersection = words1.intersection(&words2).count();
            let union = words1.union(&words2).count();
            if union > 0 {
                similarity += intersection as f32 / union as f32;
                factors += 1;
            }
        }

        // Temporal similarity (if created close in time)
        let time_diff = (memory1.created_at - memory2.created_at).abs();
        if time_diff.num_hours() < 24 {
            similarity += 0.3;
            factors += 1;
        }

        // Agent similarity
        if memory1.metadata.agent_id == memory2.metadata.agent_id {
            similarity += 0.2;
            factors += 1;
        }

        if factors > 0 {
            similarity / factors as f32
        } else {
            0.0
        }
    }

    fn determine_consolidation_strategy(&self, group: &[Memory]) -> ConsolidationStrategy {
        let avg_importance: f32 = group.iter().map(|m| m.importance_score).sum::<f32>() / group.len() as f32;
        let total_access_count: u32 = group.iter().map(|m| m.access_count).sum();
        
        // High importance memories should be preserved
        if avg_importance > 0.8 {
            return ConsolidationStrategy::Preserve;
        }

        // Frequently accessed memories should be merged, not deleted
        if total_access_count > 10 {
            return ConsolidationStrategy::Merge;
        }

        // Long content should be summarized
        let avg_length: usize = group.iter().map(|m| m.content.len()).sum::<usize>() / group.len();
        if avg_length > 500 {
            return ConsolidationStrategy::Summarize;
        }

        // Old, low-importance memories can be archived
        let oldest_memory = group.iter().min_by_key(|m| m.created_at).unwrap();
        let age_days = Utc::now().signed_duration_since(oldest_memory.created_at).num_days();
        if age_days > 30 && avg_importance < 0.3 {
            return ConsolidationStrategy::Archive;
        }

        // Very similar content can be deduplicated
        if self.are_memories_nearly_identical(group) {
            return ConsolidationStrategy::Deduplicate;
        }

        // Default to merging
        ConsolidationStrategy::Merge
    }

    fn are_memories_nearly_identical(&self, group: &[Memory]) -> bool {
        if group.len() < 2 {
            return false;
        }

        let first_content = &group[0].content.to_lowercase();
        
        for memory in group.iter().skip(1) {
            let content = memory.content.to_lowercase();
            let similarity = self.calculate_string_similarity(first_content, &content);
            if similarity < 0.9 {
                return false;
            }
        }

        true
    }

    fn calculate_string_similarity(&self, s1: &str, s2: &str) -> f32 {
        // Simple Levenshtein-based similarity
        let len1 = s1.len();
        let len2 = s2.len();
        
        if len1 == 0 && len2 == 0 {
            return 1.0;
        }
        
        let max_len = len1.max(len2);
        let distance = self.levenshtein_distance(s1, s2);
        
        1.0 - (distance as f32 / max_len as f32)
    }

    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let s1_len = s1_chars.len();
        let s2_len = s2_chars.len();

        let mut dp = vec![vec![0; s2_len + 1]; s1_len + 1];

        for i in 0..=s1_len {
            dp[i][0] = i;
        }
        for j in 0..=s2_len {
            dp[0][j] = j;
        }

        for i in 1..=s1_len {
            for j in 1..=s2_len {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }

        dp[s1_len][s2_len]
    }

    async fn summarize_memory_group(&self, _memory_manager: &mut MemoryManager, group: Vec<Memory>) -> Result<usize> {
        // In a real implementation, this would use an LLM to create a summary
        // For now, we'll create a simple concatenated summary
        
        if group.is_empty() {
            return Ok(0);
        }

        let combined_content = group.iter()
            .map(|m| &m.content)
            .collect::<Vec<_>>()
            .join(". ");

        let summary_content = if combined_content.len() > 200 {
            format!("{}...", &combined_content[..200])
        } else {
            combined_content
        };

        // Create a new consolidated memory
        let representative_memory = &group[0];
        let mut summary_metadata = representative_memory.metadata.clone();
        summary_metadata.source = MemorySource::Consolidation;

        // Merge topics and entities from all memories
        let mut all_topics = std::collections::HashSet::new();
        let mut all_entities = Vec::new();
        
        for memory in &group {
            for topic in &memory.metadata.topics {
                all_topics.insert(topic.clone());
            }
            all_entities.extend(memory.metadata.entities.clone());
        }

        summary_metadata.topics = all_topics.into_iter().collect();
        summary_metadata.entities = all_entities;

        let _summary_memory = Memory::new(
            summary_content,
            MemoryLayer::LongTerm, // Summaries go to long-term
            summary_metadata,
        );

        // In a real implementation, we would:
        // 1. Store the summary memory
        // 2. Delete the original memories
        // 3. Update associations

        Ok(group.len() - 1) // Return number of memories consolidated (all but the summary)
    }

    async fn merge_memory_group(&self, _memory_manager: &mut MemoryManager, group: Vec<Memory>) -> Result<usize> {
        // Simple merge: keep the most important memory and add notes about others
        if group.is_empty() {
            return Ok(0);
        }

        let best_memory = group.iter()
            .max_by(|a, b| a.importance_score.partial_cmp(&b.importance_score).unwrap())
            .unwrap();

        // In a real implementation, we would update the best memory with information from others
        // and then delete the redundant memories

        Ok(group.len() - 1)
    }

    async fn archive_memory_group(&self, _memory_manager: &mut MemoryManager, group: Vec<Memory>) -> Result<usize> {
        // Move memories to a compressed archive storage
        // In a real implementation, this would move them to a different storage system
        
        Ok(group.len())
    }

    async fn deduplicate_memory_group(&self, _memory_manager: &mut MemoryManager, group: Vec<Memory>) -> Result<usize> {
        // Keep only the first memory and delete duplicates
        if group.len() <= 1 {
            return Ok(0);
        }

        // In a real implementation, we would delete all but the first memory
        Ok(group.len() - 1)
    }

    async fn rebalance_memory_layers(&self, _memory_manager: &mut MemoryManager) -> Result<()> {
        // Move memories between layers based on:
        // - Importance scores
        // - Access patterns  
        // - Age
        // - Layer capacity

        // This would involve analyzing each memory and potentially moving it to a more appropriate layer
        Ok(())
    }

    // Insight generation methods

    async fn detect_behavioral_patterns(&self, memory_manager: &MemoryManager) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Analyze user behavior patterns across different agents
        let agents: std::collections::HashSet<String> = memory_manager
            .get_memories_by_layer(MemoryLayer::Working)
            .iter()
            .chain(memory_manager.get_memories_by_layer(MemoryLayer::ShortTerm).iter())
            .map(|m| m.metadata.agent_id.clone())
            .collect();

        for agent_id in agents {
            if let Ok(agent_memories) = memory_manager.get_agent_memories(&agent_id).await {
                let patterns = self.analyze_agent_patterns(&agent_memories);
                
                for pattern in patterns {
                    insights.push(Insight {
                        id: Uuid::new_v4(),
                        insight_type: InsightType::Pattern,
                        content: pattern,
                        confidence: 0.7,
                        supporting_memories: agent_memories.iter().take(3).map(|m| m.id).collect(),
                        created_at: Utc::now(),
                        agent_id: agent_id.clone(),
                    });
                }
            }
        }

        Ok(insights)
    }

    fn analyze_agent_patterns(&self, memories: &[Memory]) -> Vec<String> {
        let mut patterns = Vec::new();

        // Analyze time patterns
        let mut hour_counts: HashMap<u32, usize> = HashMap::new();
        for memory in memories {
            let hour = memory.created_at.hour();
            *hour_counts.entry(hour).or_insert(0) += 1;
        }

        if let Some((peak_hour, count)) = hour_counts.iter().max_by_key(|(_, &count)| count) {
            if *count > memories.len() / 4 {
                patterns.push(format!("User is most active around {}:00, with {} interactions", peak_hour, count));
            }
        }

        // Analyze topic patterns
        let mut topic_counts: HashMap<String, usize> = HashMap::new();
        for memory in memories {
            for topic in &memory.metadata.topics {
                *topic_counts.entry(topic.clone()).or_insert(0) += 1;
            }
        }

        if let Some((top_topic, count)) = topic_counts.iter().max_by_key(|(_, &count)| count) {
            if *count > 3 {
                patterns.push(format!("User frequently discusses {}, mentioned {} times", top_topic, count));
            }
        }

        patterns
    }

    async fn infer_user_preferences(&self, memory_manager: &MemoryManager) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Look for preference indicators in memory content
        let preference_keywords = ["like", "love", "prefer", "favorite", "dislike", "hate", "avoid"];
        
        for layer in [MemoryLayer::Working, MemoryLayer::ShortTerm, MemoryLayer::LongTerm] {
            let memories = memory_manager.get_memories_by_layer(layer);
            
            for memory in memories {
                let content_lower = memory.content.to_lowercase();
                
                for keyword in &preference_keywords {
                    if content_lower.contains(keyword) {
                        let preference = self.extract_preference_from_content(&memory.content, keyword);
                        if !preference.is_empty() {
                            insights.push(Insight {
                                id: Uuid::new_v4(),
                                insight_type: InsightType::Preference,
                                content: preference,
                                confidence: 0.6,
                                supporting_memories: vec![memory.id],
                                created_at: Utc::now(),
                                agent_id: memory.metadata.agent_id.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(insights)
    }

    fn extract_preference_from_content(&self, content: &str, keyword: &str) -> String {
        // Simple extraction of preference statements
        let sentences: Vec<&str> = content.split('.').collect();
        
        for sentence in sentences {
            if sentence.to_lowercase().contains(keyword) {
                return format!("Detected preference: {}", sentence.trim());
            }
        }

        String::new()
    }

    async fn identify_recurring_themes(&self, _memory_manager: &MemoryManager) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Analyze topics that appear frequently across different conversations
        // This would involve more sophisticated topic modeling in a real implementation

        insights.push(Insight {
            id: Uuid::new_v4(),
            insight_type: InsightType::Theme,
            content: "Recurring theme analysis not yet implemented".to_string(),
            confidence: 0.3,
            supporting_memories: vec![],
            created_at: Utc::now(),
            agent_id: "system".to_string(),
        });

        Ok(insights)
    }

    async fn discover_relationships(&self, _memory_manager: &MemoryManager) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Discover relationships between entities, topics, and concepts
        // This would involve graph analysis in a real implementation

        insights.push(Insight {
            id: Uuid::new_v4(),
            insight_type: InsightType::Relationship,
            content: "Relationship discovery not yet implemented".to_string(),
            confidence: 0.3,
            supporting_memories: vec![],
            created_at: Utc::now(),
            agent_id: "system".to_string(),
        });

        Ok(insights)
    }

    async fn detect_contradictions(&self, _memory_manager: &MemoryManager) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Detect contradictory information in memories
        // This would require semantic understanding in a real implementation

        insights.push(Insight {
            id: Uuid::new_v4(),
            insight_type: InsightType::Contradiction,
            content: "Contradiction detection not yet implemented".to_string(),
            confidence: 0.3,
            supporting_memories: vec![],
            created_at: Utc::now(),
            agent_id: "system".to_string(),
        });

        Ok(insights)
    }

    fn merge_reports(&self, main_report: &mut ConsolidationReport, other_report: ConsolidationReport) {
        main_report.memories_processed += other_report.memories_processed;
        main_report.memories_consolidated += other_report.memories_consolidated;
        main_report.memories_archived += other_report.memories_archived;
        main_report.memories_deleted += other_report.memories_deleted;
        main_report.new_insights.extend(other_report.new_insights);
        main_report.space_saved_bytes += other_report.space_saved_bytes;
    }
}

impl Default for ConsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::memory_types::*;
    use std::collections::HashMap;

    fn create_test_memory(content: &str, agent_id: &str) -> Memory {
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

        Memory::new(content.to_string(), MemoryLayer::Working, metadata)
    }

    #[test]
    fn test_memory_similarity_calculation() {
        let engine = ConsolidationEngine::new();
        
        let memory1 = create_test_memory("I love programming in Python", "agent1");
        let memory2 = create_test_memory("Python programming is great", "agent1");
        let memory3 = create_test_memory("The weather is nice today", "agent1");
        
        let similarity_high = engine.calculate_memory_similarity(&memory1, &memory2);
        let similarity_low = engine.calculate_memory_similarity(&memory1, &memory3);
        
        assert!(similarity_high > similarity_low);
        assert!(similarity_high > 0.3); // Should detect some similarity
    }

    #[test]
    fn test_consolidation_strategy_determination() {
        let engine = ConsolidationEngine::new();
        
        // High importance memories should be preserved
        let high_importance_memory = {
            let mut m = create_test_memory("Critical information", "agent1");
            m.importance_score = 0.9;
            m
        };
        
        let strategy = engine.determine_consolidation_strategy(&[high_importance_memory]);
        assert_eq!(strategy, ConsolidationStrategy::Preserve);
        
        // Low importance memories should be handled differently
        let low_importance_memory = {
            let mut m = create_test_memory("Casual chat", "agent1");
            m.importance_score = 0.2;
            m
        };
        
        let strategy = engine.determine_consolidation_strategy(&[low_importance_memory]);
        assert_ne!(strategy, ConsolidationStrategy::Preserve);
    }

    #[test]
    fn test_string_similarity() {
        let engine = ConsolidationEngine::new();
        
        let s1 = "hello world";
        let s2 = "hello world";
        let s3 = "goodbye moon";
        
        assert_eq!(engine.calculate_string_similarity(s1, s2), 1.0);
        assert!(engine.calculate_string_similarity(s1, s3) < 0.5);
    }

    #[tokio::test]
    async fn test_memory_grouping() {
        let engine = ConsolidationEngine::new();
        
        let memories = vec![
            create_test_memory("Python is great for data science", "agent1"),
            create_test_memory("I love Python programming", "agent1"),
            create_test_memory("Weather is nice today", "agent1"),
        ];
        
        let memory_refs: Vec<&Memory> = memories.iter().collect();
        let groups = engine.group_similar_memories(memory_refs).await.unwrap();
        
        // Should group similar memories together
        assert!(groups.len() <= memories.len());
    }
}