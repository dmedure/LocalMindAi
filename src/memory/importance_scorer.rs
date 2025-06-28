use anyhow::Result;
use std::collections::HashMap;
use regex::Regex;

use super::memory_types::{Memory, MemorySource, MemoryLayer, Entity, EntityType};

/// System for calculating memory importance scores
pub struct ImportanceScorer {
    /// Keywords that indicate high importance
    importance_keywords: HashMap<String, f32>,
    /// Patterns that indicate importance
    importance_patterns: Vec<ImportancePattern>,
    /// Entity type weights
    entity_weights: HashMap<EntityType, f32>,
}

#[derive(Debug, Clone)]
struct ImportancePattern {
    pattern: Regex,
    weight: f32,
    description: String,
}

impl ImportanceScorer {
    pub fn new() -> Self {
        let mut importance_keywords = HashMap::new();
        
        // High importance keywords
        importance_keywords.insert("important".to_string(), 0.8);
        importance_keywords.insert("urgent".to_string(), 0.9);
        importance_keywords.insert("critical".to_string(), 0.95);
        importance_keywords.insert("remember".to_string(), 0.7);
        importance_keywords.insert("deadline".to_string(), 0.8);
        importance_keywords.insert("meeting".to_string(), 0.6);
        importance_keywords.insert("appointment".to_string(), 0.7);
        importance_keywords.insert("password".to_string(), 0.9);
        importance_keywords.insert("secret".to_string(), 0.9);
        importance_keywords.insert("confidential".to_string(), 0.85);
        importance_keywords.insert("project".to_string(), 0.6);
        importance_keywords.insert("goal".to_string(), 0.7);
        importance_keywords.insert("objective".to_string(), 0.7);
        importance_keywords.insert("milestone".to_string(), 0.75);
        importance_keywords.insert("personal".to_string(), 0.6);
        importance_keywords.insert("private".to_string(), 0.7);
        importance_keywords.insert("preference".to_string(), 0.6);
        importance_keywords.insert("dislike".to_string(), 0.6);
        importance_keywords.insert("love".to_string(), 0.6);
        importance_keywords.insert("hate".to_string(), 0.6);
        importance_keywords.insert("favorite".to_string(), 0.6);
        importance_keywords.insert("birthday".to_string(), 0.8);
        importance_keywords.insert("anniversary".to_string(), 0.8);
        importance_keywords.insert("address".to_string(), 0.7);
        importance_keywords.insert("phone".to_string(), 0.7);
        importance_keywords.insert("email".to_string(), 0.6);
        importance_keywords.insert("contact".to_string(), 0.6);
        
        // Technical keywords
        importance_keywords.insert("bug".to_string(), 0.7);
        importance_keywords.insert("fix".to_string(), 0.6);
        importance_keywords.insert("solution".to_string(), 0.7);
        importance_keywords.insert("workaround".to_string(), 0.7);
        importance_keywords.insert("api".to_string(), 0.5);
        importance_keywords.insert("key".to_string(), 0.6);
        importance_keywords.insert("token".to_string(), 0.7);
        importance_keywords.insert("configuration".to_string(), 0.6);
        importance_keywords.insert("settings".to_string(), 0.5);
        
        let mut importance_patterns = Vec::new();
        
        // Date patterns (appointments, deadlines)
        if let Ok(date_pattern) = Regex::new(r"\b\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4}\b") {
            importance_patterns.push(ImportancePattern {
                pattern: date_pattern,
                weight: 0.6,
                description: "Contains date".to_string(),
            });
        }
        
        // Time patterns
        if let Ok(time_pattern) = Regex::new(r"\b\d{1,2}:\d{2}\s*(AM|PM|am|pm)?\b") {
            importance_patterns.push(ImportancePattern {
                pattern: time_pattern,
                weight: 0.5,
                description: "Contains time".to_string(),
            });
        }
        
        // Email patterns
        if let Ok(email_pattern) = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b") {
            importance_patterns.push(ImportancePattern {
                pattern: email_pattern,
                weight: 0.6,
                description: "Contains email address".to_string(),
            });
        }
        
        // Phone number patterns
        if let Ok(phone_pattern) = Regex::new(r"\b\(?(\d{3})\)?[-.\s]?(\d{3})[-.\s]?(\d{4})\b") {
            importance_patterns.push(ImportancePattern {
                pattern: phone_pattern,
                weight: 0.7,
                description: "Contains phone number".to_string(),
            });
        }
        
        // URL patterns
        if let Ok(url_pattern) = Regex::new(r"https?://[^\s]+") {
            importance_patterns.push(ImportancePattern {
                pattern: url_pattern,
                weight: 0.4,
                description: "Contains URL".to_string(),
            });
        }
        
        // Questions patterns (user seeking information)
        if let Ok(question_pattern) = Regex::new(r"\b(how|what|why|when|where|who)\b.*\?") {
            importance_patterns.push(ImportancePattern {
                pattern: question_pattern,
                weight: 0.3,
                description: "Contains question".to_string(),
            });
        }
        
        // Exclamation patterns (emphasis)
        if let Ok(exclamation_pattern) = Regex::new(r"!{1,3}") {
            importance_patterns.push(ImportancePattern {
                pattern: exclamation_pattern,
                weight: 0.2,
                description: "Contains exclamation".to_string(),
            });
        }
        
        // ALL CAPS patterns (emphasis)
        if let Ok(caps_pattern) = Regex::new(r"\b[A-Z]{3,}\b") {
            importance_patterns.push(ImportancePattern {
                pattern: caps_pattern,
                weight: 0.3,
                description: "Contains emphasized text".to_string(),
            });
        }
        
        let mut entity_weights = HashMap::new();
        entity_weights.insert(EntityType::Person, 0.7);
        entity_weights.insert(EntityType::Place, 0.5);
        entity_weights.insert(EntityType::Organization, 0.6);
        entity_weights.insert(EntityType::Date, 0.8);
        entity_weights.insert(EntityType::Event, 0.7);
        entity_weights.insert(EntityType::Concept, 0.5);
        entity_weights.insert(EntityType::Product, 0.4);
        entity_weights.insert(EntityType::Technology, 0.5);
        entity_weights.insert(EntityType::Other(_), 0.3);

        Self {
            importance_keywords,
            importance_patterns,
            entity_weights,
        }
    }

    /// Calculate importance score for a memory (0.0 to 1.0)
    pub async fn calculate_importance(&self, memory: &Memory) -> Result<f32> {
        let mut score = 0.0;
        let content_lower = memory.content.to_lowercase();
        
        // Base score from source type
        score += self.get_source_importance(&memory.metadata.source);
        
        // Score from memory layer
        score += self.get_layer_importance(&memory.layer);
        
        // Score from keywords
        score += self.score_keywords(&content_lower);
        
        // Score from patterns
        score += self.score_patterns(&memory.content);
        
        // Score from entities
        score += self.score_entities(&memory.metadata.entities);
        
        // Score from content length (very short or very long might be more important)
        score += self.score_content_length(&memory.content);
        
        // Score from sentiment if available
        if let Some(ref sentiment) = memory.metadata.sentiment {
            score += self.score_sentiment(sentiment);
        }
        
        // Score from user-defined topics
        score += self.score_topics(&memory.metadata.topics);
        
        // Score from recency (recent memories slightly more important)
        score += memory.recency_score() * 0.1;
        
        // Score from access frequency
        score += memory.frequency_score() * 0.1;
        
        // Normalize score to 0.0-1.0 range
        Ok(score.min(1.0).max(0.0))
    }

    /// Update importance score based on user feedback
    pub fn update_importance_with_feedback(&mut self, memory: &Memory, user_rating: f32) -> f32 {
        let current_score = memory.importance_score;
        let feedback_weight = 0.3; // How much user feedback influences the score
        
        // Blend current score with user feedback
        let new_score = current_score * (1.0 - feedback_weight) + user_rating * feedback_weight;
        
        // Learn from user feedback by adjusting keyword weights
        self.adjust_keyword_weights(memory, user_rating);
        
        new_score.clamp(0.0, 1.0)
    }

    /// Analyze importance trends for an agent
    pub fn analyze_importance_trends(&self, memories: &[Memory]) -> ImportanceTrends {
        let mut trends = ImportanceTrends {
            average_importance: 0.0,
            high_importance_ratio: 0.0,
            most_important_topics: Vec::new(),
            importance_by_source: HashMap::new(),
            importance_over_time: Vec::new(),
        };

        if memories.is_empty() {
            return trends;
        }

        // Calculate average importance
        let total_importance: f32 = memories.iter().map(|m| m.importance_score).sum();
        trends.average_importance = total_importance / memories.len() as f32;

        // Calculate high importance ratio (>0.7)
        let high_importance_count = memories.iter().filter(|m| m.importance_score > 0.7).count();
        trends.high_importance_ratio = high_importance_count as f32 / memories.len() as f32;

        // Find most important topics
        let mut topic_importance: HashMap<String, Vec<f32>> = HashMap::new();
        for memory in memories {
            for topic in &memory.metadata.topics {
                topic_importance.entry(topic.clone()).or_insert_with(Vec::new).push(memory.importance_score);
            }
        }

        let mut topic_scores: Vec<(String, f32)> = topic_importance
            .into_iter()
            .map(|(topic, scores)| {
                let avg_score = scores.iter().sum::<f32>() / scores.len() as f32;
                (topic, avg_score)
            })
            .collect();

        topic_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        trends.most_important_topics = topic_scores.into_iter().take(5).collect();

        // Importance by source
        for memory in memories {
            let source_scores = trends.importance_by_source.entry(memory.metadata.source.clone()).or_insert_with(Vec::new);
            source_scores.push(memory.importance_score);
        }

        trends
    }

    // Private helper methods

    fn get_source_importance(&self, source: &MemorySource) -> f32 {
        match source {
            MemorySource::UserInput => 0.3,
            MemorySource::AgentResponse => 0.2,
            MemorySource::SystemInsight => 0.4,
            MemorySource::ExternalImport => 0.3,
            MemorySource::Reflection => 0.5,
            MemorySource::DocumentExtraction => 0.4,
            MemorySource::Consolidation => 0.6,
        }
    }

    fn get_layer_importance(&self, layer: &MemoryLayer) -> f32 {
        match layer {
            MemoryLayer::Working => 0.1,
            MemoryLayer::ShortTerm => 0.2,
            MemoryLayer::LongTerm => 0.4,
            MemoryLayer::Episodic => 0.3,
            MemoryLayer::Semantic => 0.35,
            MemoryLayer::Reflective => 0.5,
        }
    }

    fn score_keywords(&self, content: &str) -> f32 {
        let mut keyword_score = 0.0;
        let words: Vec<&str> = content.split_whitespace().collect();
        
        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            if let Some(&weight) = self.importance_keywords.get(&clean_word) {
                keyword_score += weight;
            }
        }
        
        // Normalize by content length to prevent bias toward long content
        let word_count = content.split_whitespace().count() as f32;
        if word_count > 0.0 {
            keyword_score / word_count.sqrt()
        } else {
            0.0
        }
    }

    fn score_patterns(&self, content: &str) -> f32 {
        let mut pattern_score = 0.0;
        
        for pattern in &self.importance_patterns {
            if pattern.pattern.is_match(content) {
                pattern_score += pattern.weight;
            }
        }
        
        pattern_score
    }

    fn score_entities(&self, entities: &[Entity]) -> f32 {
        let mut entity_score = 0.0;
        
        for entity in entities {
            if let Some(&weight) = self.entity_weights.get(&entity.entity_type) {
                entity_score += weight * entity.confidence;
            }
        }
        
        // Normalize by number of entities to prevent bias
        if !entities.is_empty() {
            entity_score / entities.len() as f32
        } else {
            0.0
        }
    }

    fn score_content_length(&self, content: &str) -> f32 {
        let length = content.len();
        
        // Very short content (< 10 chars) might be important (e.g., passwords, codes)
        if length < 10 {
            return 0.2;
        }
        
        // Very long content (> 1000 chars) might be important (e.g., detailed instructions)
        if length > 1000 {
            return 0.1;
        }
        
        // Medium length content gets neutral score
        0.0
    }

    fn score_sentiment(&self, sentiment: &super::memory_types::Sentiment) -> f32 {
        // Strong emotions (positive or negative) might indicate importance
        let emotion_strength = sentiment.magnitude;
        let emotion_intensity = sentiment.polarity.abs();
        
        (emotion_strength * emotion_intensity) * 0.2
    }

    fn score_topics(&self, topics: &[String]) -> f32 {
        // More topics might indicate more complex/important content
        let topic_count = topics.len() as f32;
        
        // Logarithmic scaling to prevent overwhelming bias
        if topic_count > 0.0 {
            (topic_count.ln() + 1.0) * 0.1
        } else {
            0.0
        }
    }

    fn adjust_keyword_weights(&mut self, memory: &Memory, user_rating: f32) {
        let content_lower = memory.content.to_lowercase();
        let words: Vec<&str> = content_lower.split_whitespace().collect();
        
        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            
            if let Some(current_weight) = self.importance_keywords.get_mut(clean_word) {
                // Adjust weight based on user feedback
                let adjustment = (user_rating - 0.5) * 0.1; // Small adjustment
                *current_weight = (*current_weight + adjustment).clamp(0.0, 1.0);
            } else if user_rating > 0.7 && clean_word.len() > 3 {
                // Add new keyword if user rates it highly
                self.importance_keywords.insert(clean_word.to_string(), 0.3);
            }
        }
    }
}

/// Analysis of importance trends for an agent
#[derive(Debug, Clone)]
pub struct ImportanceTrends {
    pub average_importance: f32,
    pub high_importance_ratio: f32,
    pub most_important_topics: Vec<(String, f32)>,
    pub importance_by_source: HashMap<MemorySource, Vec<f32>>,
    pub importance_over_time: Vec<(chrono::DateTime<chrono::Utc>, f32)>,
}

impl Default for ImportanceScorer {
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

    fn create_test_memory(content: &str) -> Memory {
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

        Memory::new(content.to_string(), MemoryLayer::Working, metadata)
    }

    #[tokio::test]
    async fn test_importance_scoring() {
        let scorer = ImportanceScorer::new();
        
        // Test high importance content
        let important_memory = create_test_memory("URGENT: Remember to call the client at 3:00 PM about the critical project deadline!");
        let score = scorer.calculate_importance(&important_memory).await.unwrap();
        assert!(score > 0.5, "Important memory should have high score, got {}", score);
        
        // Test low importance content
        let casual_memory = create_test_memory("It's a nice day today.");
        let score = scorer.calculate_importance(&casual_memory).await.unwrap();
        assert!(score < 0.5, "Casual memory should have low score, got {}", score);
    }

    #[tokio::test]
    async fn test_keyword_scoring() {
        let scorer = ImportanceScorer::new();
        
        let memory_with_keywords = create_test_memory("Remember this important password: abc123");
        let score = scorer.calculate_importance(&memory_with_keywords).await.unwrap();
        assert!(score > 0.4);
    }

    #[tokio::test]
    async fn test_pattern_scoring() {
        let scorer = ImportanceScorer::new();
        
        // Test email pattern
        let memory_with_email = create_test_memory("Contact me at john@example.com");
        let score = scorer.calculate_importance(&memory_with_email).await.unwrap();
        assert!(score > 0.3);
        
        // Test date pattern
        let memory_with_date = create_test_memory("Meeting scheduled for 12/25/2024");
        let score = scorer.calculate_importance(&memory_with_date).await.unwrap();
        assert!(score > 0.3);
    }

    #[test]
    fn test_user_feedback_learning() {
        let mut scorer = ImportanceScorer::new();
        let memory = create_test_memory("custom important content");
        
        // Simulate user rating this as highly important
        let new_score = scorer.update_importance_with_feedback(&memory, 0.9);
        assert!(new_score > memory.importance_score);
    }

    #[test]
    fn test_importance_trends() {
        let scorer = ImportanceScorer::new();
        
        let memories = vec![
            {
                let mut m = create_test_memory("important stuff");
                m.importance_score = 0.8;
                m
            },
            {
                let mut m = create_test_memory("casual chat");
                m.importance_score = 0.3;
                m
            },
            {
                let mut m = create_test_memory("more important things");
                m.importance_score = 0.7;
                m
            },
        ];
        
        let trends = scorer.analyze_importance_trends(&memories);
        assert!(trends.average_importance > 0.5);
        assert!(trends.high_importance_ratio > 0.0);
    }
}