use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use crate::utils::error::{LocalMindError, Result};

/// Classifies task complexity to inform model selection
pub struct TaskClassifier {
    complexity_keywords: HashMap<ComplexityLevel, Vec<String>>,
    topic_patterns: HashMap<String, Regex>,
    task_type_patterns: HashMap<TaskType, Vec<Regex>>,
    language_detector: LanguageDetector,
}

/// Task complexity assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComplexity {
    pub score: f32,                    // 0.0-1.0, where 1.0 is most complex
    pub reasoning_required: bool,      // Whether task requires multi-step reasoning
    pub detected_topics: Vec<String>,  // Identified topics/domains
    pub estimated_tokens: u32,         // Estimated response length needed
    pub task_type: TaskType,          // Type of task identified
}

/// Complexity levels for categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Simple,      // 0.0-0.3: Basic conversations, simple questions
    Moderate,    // 0.3-0.6: Explanations, summaries, basic analysis
    Complex,     // 0.6-0.8: Multi-step reasoning, detailed analysis
    Advanced,    // 0.8-1.0: Research, code generation, expert-level tasks
}

/// Types of tasks that can be identified
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    Conversation,        // Casual chat, greetings
    QuestionAnswering,   // Direct questions
    Explanation,         // "Explain X", "How does Y work"
    Analysis,           // "Analyze", "Compare", "Evaluate"
    CodeGeneration,     // Programming tasks
    Creative,           // Writing, brainstorming, creative tasks
    Research,           // In-depth investigation, citations needed
    Summarization,      // Summarize documents, articles
    Translation,        // Language translation
    Math,               // Mathematical problems
    Technical,          // Technical documentation, troubleshooting
    Planning,           // Step-by-step planning, project management
}

/// Language characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDetector;

/// Context about the conversation/session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub previous_tasks: Vec<TaskType>,
    pub complexity_trend: f32,
    pub user_expertise_level: ExpertiseLevel,
    pub session_length: usize,
}

/// User expertise level estimation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpertiseLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

impl TaskClassifier {
    /// Create a new task classifier
    pub fn new() -> Self {
        Self {
            complexity_keywords: Self::build_complexity_keywords(),
            topic_patterns: Self::build_topic_patterns(),
            task_type_patterns: Self::build_task_type_patterns(),
            language_detector: LanguageDetector,
        }
    }

    /// Classify the complexity of a prompt
    pub async fn classify_prompt(&self, prompt: &str) -> Result<TaskComplexity> {
        let cleaned_prompt = self.preprocess_prompt(prompt);
        
        // Analyze different aspects
        let base_complexity = self.analyze_base_complexity(&cleaned_prompt);
        let reasoning_complexity = self.analyze_reasoning_requirements(&cleaned_prompt);
        let topic_complexity = self.analyze_topic_complexity(&cleaned_prompt);
        let length_complexity = self.analyze_length_complexity(&cleaned_prompt);
        
        // Combine complexity scores
        let combined_score = (base_complexity * 0.3) + 
                           (reasoning_complexity * 0.3) + 
                           (topic_complexity * 0.2) + 
                           (length_complexity * 0.2);

        // Detect task type
        let task_type = self.detect_task_type(&cleaned_prompt);
        
        // Detect topics
        let detected_topics = self.detect_topics(&cleaned_prompt);
        
        // Check if reasoning is required
        let reasoning_required = self.requires_reasoning(&cleaned_prompt, &task_type);
        
        // Estimate required response length
        let estimated_tokens = self.estimate_response_tokens(&cleaned_prompt, &task_type);

        Ok(TaskComplexity {
            score: combined_score.min(1.0).max(0.0),
            reasoning_required,
            detected_topics,
            estimated_tokens,
            task_type,
        })
    }

    /// Preprocess prompt for analysis
    fn preprocess_prompt(&self, prompt: &str) -> String {
        prompt
            .to_lowercase()
            .trim()
            .to_string()
    }

    /// Analyze base complexity through keyword detection
    fn analyze_base_complexity(&self, prompt: &str) -> f32 {
        let mut total_score = 0.0;
        let mut keyword_count = 0;

        for (level, keywords) in &self.complexity_keywords {
            for keyword in keywords {
                if prompt.contains(keyword) {
                    let score = match level {
                        ComplexityLevel::Simple => 0.2,
                        ComplexityLevel::Moderate => 0.4,
                        ComplexityLevel::Complex => 0.7,
                        ComplexityLevel::Advanced => 0.9,
                    };
                    total_score += score;
                    keyword_count += 1;
                }
            }
        }

        if keyword_count > 0 {
            total_score / keyword_count as f32
        } else {
            0.3 // Default moderate complexity if no keywords found
        }
    }

    /// Analyze reasoning requirements
    fn analyze_reasoning_requirements(&self, prompt: &str) -> f32 {
        let reasoning_indicators = [
            "why", "how", "explain", "analyze", "compare", "evaluate", "assess",
            "determine", "conclude", "infer", "deduce", "prove", "justify",
            "reason", "logic", "because", "therefore", "consequently", "thus",
            "step by step", "break down", "walk through", "think through"
        ];

        let multi_step_indicators = [
            "first", "second", "then", "next", "finally", "steps", "process",
            "procedure", "methodology", "approach", "strategy", "plan"
        ];

        let mut reasoning_score = 0.0;

        // Check for reasoning keywords
        for indicator in &reasoning_indicators {
            if prompt.contains(indicator) {
                reasoning_score += 0.1;
            }
        }

        // Check for multi-step indicators
        for indicator in &multi_step_indicators {
            if prompt.contains(indicator) {
                reasoning_score += 0.15;
            }
        }

        // Question complexity
        let question_count = prompt.matches('?').count();
        if question_count > 1 {
            reasoning_score += 0.2; // Multiple questions increase complexity
        }

        reasoning_score.min(1.0)
    }

    /// Analyze topic-specific complexity
    fn analyze_topic_complexity(&self, prompt: &str) -> f32 {
        let technical_topics = [
            "algorithm", "programming", "code", "software", "database", "neural network",
            "machine learning", "ai", "artificial intelligence", "quantum", "cryptography",
            "mathematics", "calculus", "statistics", "physics", "chemistry", "biology",
            "engineering", "architecture", "research", "academic", "scientific"
        ];

        let creative_topics = [
            "creative", "story", "poem", "write", "brainstorm", "imagine", "design",
            "art", "music", "novel", "character", "plot", "theme", "style"
        ];

        let business_topics = [
            "business", "strategy", "market", "analysis", "finance", "investment",
            "management", "leadership", "project", "planning", "proposal"
        ];

        let mut complexity_score = 0.3; // Base score

        // Technical topics add complexity
        for topic in &technical_topics {
            if prompt.contains(topic) {
                complexity_score += 0.2;
                break;
            }
        }

        // Creative topics are moderately complex
        for topic in &creative_topics {
            if prompt.contains(topic) {
                complexity_score += 0.1;
                break;
            }
        }

        // Business topics are moderately complex
        for topic in &business_topics {
            if prompt.contains(topic) {
                complexity_score += 0.15;
                break;
            }
        }

        complexity_score.min(1.0)
    }

    /// Analyze complexity based on prompt length
    fn analyze_length_complexity(&self, prompt: &str) -> f32 {
        let word_count = prompt.split_whitespace().count();
        let sentence_count = prompt.matches('.').count() + prompt.matches('?').count() + prompt.matches('!').count();
        
        let mut length_score = 0.0;

        // Word count factor
        if word_count > 100 {
            length_score += 0.4;
        } else if word_count > 50 {
            length_score += 0.3;
        } else if word_count > 20 {
            length_score += 0.2;
        } else {
            length_score += 0.1;
        }

        // Sentence count factor
        if sentence_count > 5 {
            length_score += 0.3;
        } else if sentence_count > 2 {
            length_score += 0.2;
        } else {
            length_score += 0.1;
        }

        length_score.min(1.0)
    }

    /// Detect the type of task
    fn detect_task_type(&self, prompt: &str) -> TaskType {
        // Check each task type pattern
        for (task_type, patterns) in &self.task_type_patterns {
            for pattern in patterns {
                if pattern.is_match(prompt) {
                    return task_type.clone();
                }
            }
        }

        // Fallback logic based on simple patterns
        if prompt.contains("code") || prompt.contains("program") || prompt.contains("function") {
            TaskType::CodeGeneration
        } else if prompt.contains("explain") || prompt.contains("what is") || prompt.contains("how does") {
            TaskType::Explanation
        } else if prompt.contains("analyze") || prompt.contains("compare") || prompt.contains("evaluate") {
            TaskType::Analysis
        } else if prompt.contains("write") || prompt.contains("create") || prompt.contains("generate") {
            TaskType::Creative
        } else if prompt.contains("summarize") || prompt.contains("summary") {
            TaskType::Summarization
        } else if prompt.contains("translate") {
            TaskType::Translation
        } else if prompt.contains("plan") || prompt.contains("steps") || prompt.contains("organize") {
            TaskType::Planning
        } else if prompt.contains('?') {
            TaskType::QuestionAnswering
        } else {
            TaskType::Conversation
        }
    }

    /// Detect topics mentioned in the prompt
    fn detect_topics(&self, prompt: &str) -> Vec<String> {
        let mut topics = Vec::new();

        for (topic, pattern) in &self.topic_patterns {
            if pattern.is_match(prompt) {
                topics.push(topic.clone());
            }
        }

        // If no specific topics found, infer from common domains
        if topics.is_empty() {
            if prompt.contains("code") || prompt.contains("programming") {
                topics.push("programming".to_string());
            } else if prompt.contains("business") || prompt.contains("company") {
                topics.push("business".to_string());
            } else if prompt.contains("science") || prompt.contains("research") {
                topics.push("science".to_string());
            } else {
                topics.push("general".to_string());
            }
        }

        topics
    }

    /// Check if task requires multi-step reasoning
    fn requires_reasoning(&self, prompt: &str, task_type: &TaskType) -> bool {
        let reasoning_indicators = [
            "why", "how", "explain", "analyze", "compare", "step by step",
            "walk through", "break down", "think through", "reason", "logic"
        ];

        let has_reasoning_keywords = reasoning_indicators.iter()
            .any(|&indicator| prompt.contains(indicator));

        // Certain task types inherently require reasoning
        let reasoning_task_types = [
            TaskType::Analysis,
            TaskType::Research,
            TaskType::CodeGeneration,
            TaskType::Planning,
            TaskType::Explanation,
        ];

        has_reasoning_keywords || reasoning_task_types.contains(task_type)
    }

    /// Estimate required response length in tokens
    fn estimate_response_tokens(&self, prompt: &str, task_type: &TaskType) -> u32 {
        let base_tokens = match task_type {
            TaskType::Conversation => 50,
            TaskType::QuestionAnswering => 100,
            TaskType::Explanation => 200,
            TaskType::Analysis => 300,
            TaskType::CodeGeneration => 150,
            TaskType::Creative => 250,
            TaskType::Research => 400,
            TaskType::Summarization => 150,
            TaskType::Translation => 100,
            TaskType::Math => 120,
            TaskType::Technical => 200,
            TaskType::Planning => 250,
        };

        // Adjust based on prompt complexity indicators
        let mut multiplier = 1.0;

        if prompt.contains("detailed") || prompt.contains("comprehensive") {
            multiplier *= 1.5;
        }
        if prompt.contains("brief") || prompt.contains("short") {
            multiplier *= 0.7;
        }
        if prompt.contains("examples") {
            multiplier *= 1.3;
        }
        if prompt.len() > 200 {
            multiplier *= 1.2; // Longer prompts often need longer responses
        }

        (base_tokens as f32 * multiplier) as u32
    }

    /// Build complexity keyword mappings
    fn build_complexity_keywords() -> HashMap<ComplexityLevel, Vec<String>> {
        let mut keywords = HashMap::new();

        keywords.insert(ComplexityLevel::Simple, vec![
            "hello".to_string(), "hi".to_string(), "thanks".to_string(), "yes".to_string(), "no".to_string(),
            "what".to_string(), "when".to_string(), "where".to_string(), "simple".to_string(), "basic".to_string(),
        ]);

        keywords.insert(ComplexityLevel::Moderate, vec![
            "explain".to_string(), "describe".to_string(), "tell me about".to_string(), "how".to_string(),
            "why".to_string(), "summarize".to_string(), "overview".to_string(), "introduction".to_string(),
        ]);

        keywords.insert(ComplexityLevel::Complex, vec![
            "analyze".to_string(), "compare".to_string(), "evaluate".to_string(), "assess".to_string(),
            "examine".to_string(), "investigate".to_string(), "determine".to_string(), "calculate".to_string(),
        ]);

        keywords.insert(ComplexityLevel::Advanced, vec![
            "research".to_string(), "comprehensive".to_string(), "detailed analysis".to_string(),
            "algorithm".to_string(), "implement".to_string(), "optimize".to_string(), "design".to_string(),
            "architecture".to_string(), "methodology".to_string(), "systematic".to_string(),
        ]);

        keywords
    }

    /// Build topic detection patterns
    fn build_topic_patterns() -> HashMap<String, Regex> {
        let mut patterns = HashMap::new();

        // Programming/Technology
        patterns.insert("programming".to_string(), 
            Regex::new(r"\b(code|coding|program|programming|software|algorithm|function|class|variable|array|loop|api|framework|library|database|sql|javascript|python|rust|java|c\+\+)\b").unwrap()
        );

        // Science/Research
        patterns.insert("science".to_string(),
            Regex::new(r"\b(research|study|experiment|hypothesis|theory|analysis|data|statistics|mathematics|physics|chemistry|biology|scientific|academic|publication)\b").unwrap()
        );

        // Business/Finance
        patterns.insert("business".to_string(),
            Regex::new(r"\b(business|company|market|finance|investment|strategy|management|revenue|profit|sales|customer|client|project|planning)\b").unwrap()
        );

        // Creative/Writing
        patterns.insert("creative".to_string(),
            Regex::new(r"\b(write|writing|story|poem|creative|art|design|music|novel|character|plot|theme|style|brainstorm|imagine)\b").unwrap()
        );

        // Education/Learning
        patterns.insert("education".to_string(),
            Regex::new(r"\b(learn|learning|teach|education|tutorial|lesson|course|study|homework|assignment|exam|test)\b").unwrap()
        );

        patterns
    }

    /// Build task type detection patterns
    fn build_task_type_patterns() -> HashMap<TaskType, Vec<Regex>> {
        let mut patterns = HashMap::new();

        patterns.insert(TaskType::CodeGeneration, vec![
            Regex::new(r"\b(write|create|implement|build|develop|code|program)\s+(a|an|the)?\s*(function|class|script|program|application|algorithm)\b").unwrap(),
            Regex::new(r"\b(show me|give me|provide)\s+(code|implementation|example)\b").unwrap(),
        ]);

        patterns.insert(TaskType::Explanation, vec![
            Regex::new(r"\b(explain|describe|what is|how does|tell me about|can you explain)\b").unwrap(),
            Regex::new(r"\b(help me understand|clarify|elaborate)\b").unwrap(),
        ]);

        patterns.insert(TaskType::Analysis, vec![
            Regex::new(r"\b(analyze|compare|evaluate|assess|examine|review|critique)\b").unwrap(),
            Regex::new(r"\b(what are the (pros and cons|advantages and disadvantages|differences|similarities))\b").unwrap(),
        ]);

        patterns.insert(TaskType::Creative, vec![
            Regex::new(r"\b(write|create|generate|compose)\s+(a|an)?\s*(story|poem|song|article|essay|script)\b").unwrap(),
            Regex::new(r"\b(brainstorm|imagine|creative|invent|design)\b").unwrap(),
        ]);

        patterns.insert(TaskType::Summarization, vec![
            Regex::new(r"\b(summarize|summary|key points|main ideas|brief overview)\b").unwrap(),
            Regex::new(r"\b(tldr|in short|condensed version)\b").unwrap(),
        ]);

        patterns.insert(TaskType::Planning, vec![
            Regex::new(r"\b(plan|planning|organize|schedule|steps|process|procedure|methodology)\b").unwrap(),
            Regex::new(r"\b(how to|step by step|walk me through)\b").unwrap(),
        ]);

        patterns
    }

    /// Classify with conversation context
    pub async fn classify_with_context(
        &self,
        prompt: &str,
        context: &ConversationContext,
    ) -> Result<TaskComplexity> {
        let mut base_complexity = self.classify_prompt(prompt).await?;

        // Adjust based on context
        if context.user_expertise_level == ExpertiseLevel::Expert {
            base_complexity.score = (base_complexity.score * 1.2).min(1.0);
        } else if context.user_expertise_level == ExpertiseLevel::Beginner {
            base_complexity.score = (base_complexity.score * 0.8).max(0.1);
        }

        // Consider complexity trend
        let trend_adjustment = (context.complexity_trend - 0.5) * 0.2;
        base_complexity.score = (base_complexity.score + trend_adjustment).min(1.0).max(0.0);

        Ok(base_complexity)
    }

    /// Get complexity level from score
    pub fn get_complexity_level(score: f32) -> ComplexityLevel {
        if score < 0.3 {
            ComplexityLevel::Simple
        } else if score < 0.6 {
            ComplexityLevel::Moderate
        } else if score < 0.8 {
            ComplexityLevel::Complex
        } else {
            ComplexityLevel::Advanced
        }
    }
}

impl Default for TaskComplexity {
    fn default() -> Self {
        Self {
            score: 0.5,
            reasoning_required: false,
            detected_topics: vec!["general".to_string()],
            estimated_tokens: 100,
            task_type: TaskType::Conversation,
        }
    }
}

impl TaskComplexity {
    /// Get human-readable complexity description
    pub fn complexity_description(&self) -> String {
        let level = TaskClassifier::get_complexity_level(self.score);
        match level {
            ComplexityLevel::Simple => "Simple task - quick response appropriate".to_string(),
            ComplexityLevel::Moderate => "Moderate complexity - balanced approach needed".to_string(),
            ComplexityLevel::Complex => "Complex task - detailed reasoning required".to_string(),
            ComplexityLevel::Advanced => "Advanced task - high-quality response essential".to_string(),
        }
    }

    /// Check if task should use high-quality model
    pub fn should_use_quality_model(&self) -> bool {
        self.score > 0.6 || self.reasoning_required || 
        matches!(self.task_type, TaskType::CodeGeneration | TaskType::Research | TaskType::Analysis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_task_classification() {
        let classifier = TaskClassifier::new();
        let result = classifier.classify_prompt("Hello, how are you?").await.unwrap();
        
        assert!(result.score < 0.5);
        assert_eq!(result.task_type, TaskType::Conversation);
        assert!(!result.reasoning_required);
    }

    #[tokio::test]
    async fn test_complex_task_classification() {
        let classifier = TaskClassifier::new();
        let result = classifier.classify_prompt(
            "Analyze the time complexity of this sorting algorithm and explain how to optimize it for large datasets"
        ).await.unwrap();
        
        assert!(result.score > 0.6);
        assert!(result.reasoning_required);
        assert!(result.detected_topics.contains(&"programming".to_string()));
    }

    #[tokio::test]
    async fn test_code_generation_task() {
        let classifier = TaskClassifier::new();
        let result = classifier.classify_prompt(
            "Write a Python function that implements binary search"
        ).await.unwrap();
        
        assert_eq!(result.task_type, TaskType::CodeGeneration);
        assert!(result.detected_topics.contains(&"programming".to_string()));
    }

    #[test]
    fn test_complexity_level_mapping() {
        assert_eq!(TaskClassifier::get_complexity_level(0.2), ComplexityLevel::Simple);
        assert_eq!(TaskClassifier::get_complexity_level(0.5), ComplexityLevel::Moderate);
        assert_eq!(TaskClassifier::get_complexity_level(0.7), ComplexityLevel::Complex);
        assert_eq!(TaskClassifier::get_complexity_level(0.9), ComplexityLevel::Advanced);
    }

    #[test]
    fn test_quality_model_recommendation() {
        let simple_task = TaskComplexity {
            score: 0.2,
            reasoning_required: false,
            detected_topics: vec!["general".to_string()],
            estimated_tokens: 50,
            task_type: TaskType::Conversation,
        };
        assert!(!simple_task.should_use_quality_model());

        let complex_task = TaskComplexity {
            score: 0.8,
            reasoning_required: true,
            detected_topics: vec!["programming".to_string()],
            estimated_tokens: 200,
            task_type: TaskType::CodeGeneration,
        };
        assert!(complex_task.should_use_quality_model());
    }
}