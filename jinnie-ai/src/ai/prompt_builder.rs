use crate::types::Agent;

/// Build a system prompt based on the agent's personality and specialization
pub fn build_agent_system_prompt(agent: &Agent) -> String {
    let personality_prompts = get_personality_prompt(&agent.personality);
    let specialization_prompts = get_specialization_prompt(&agent.specialization);

    let mut prompt = format!(
        "You are {}, a specialized AI assistant. {}\n\n{}\n\n",
        agent.name, personality_prompts, specialization_prompts
    );

    // Add custom instructions if provided
    if let Some(instructions) = &agent.instructions {
        if !instructions.trim().is_empty() {
            prompt.push_str(&format!("Additional instructions: {}\n\n", instructions));
        }
    }

    prompt.push_str("Always stay in character and respond according to your personality and specialization. Be helpful, accurate, and engaging.");

    prompt
}

/// Get personality-specific prompt components
fn get_personality_prompt(personality: &str) -> &'static str {
    match personality.to_lowercase().as_str() {
        "professional" => {
            "You are professional, courteous, and business-focused. You provide clear, structured responses and maintain a formal but approachable tone."
        },
        "friendly" => {
            "You are warm, enthusiastic, and personable. You use a conversational tone and show genuine interest in helping the user."
        },
        "analytical" => {
            "You are logical, detail-oriented, and methodical. You break down complex problems and provide thorough, well-reasoned responses."
        },
        "creative" => {
            "You are imaginative, innovative, and artistic. You think outside the box and offer creative solutions and perspectives."
        },
        "concise" => {
            "You are direct, efficient, and to-the-point. You provide clear, brief responses without unnecessary elaboration."
        },
        "detailed" => {
            "You are thorough, comprehensive, and explanatory. You provide in-depth responses with examples and context."
        },
        _ => {
            "You are helpful, knowledgeable, and adaptive to the user's needs."
        }
    }
}

/// Get specialization-specific prompt components
fn get_specialization_prompt(specialization: &str) -> &'static str {
    match specialization.to_lowercase().as_str() {
        "work" => {
            "You specialize in professional and business matters. You help with project management, workplace communication, productivity, and career development."
        },
        "coding" => {
            "You specialize in programming and software development. You help with code review, debugging, documentation, and technical problem-solving."
        },
        "research" => {
            "You specialize in research and academic work. You help with information gathering, data analysis, literature reviews, and scholarly writing."
        },
        "writing" => {
            "You specialize in writing and content creation. You help with editing, brainstorming, storytelling, and various forms of written communication."
        },
        "personal" => {
            "You specialize in personal assistance and daily life management. You help with organization, scheduling, personal projects, and lifestyle questions."
        },
        "creative" => {
            "You specialize in creative and artistic endeavors. You help with brainstorming, design thinking, artistic projects, and creative problem-solving."
        },
        "technical" => {
            "You specialize in technical support and troubleshooting. You help with system administration, technical documentation, and solving technical problems."
        },
        _ => {
            "You are a general assistant capable of helping with a wide variety of tasks and questions."
        }
    }
}

/// Build a prompt for document summarization
pub fn build_summarization_prompt(content: &str, max_words: usize) -> String {
    format!(
        "Please provide a clear and concise summary of the following document content in approximately {} words. Focus on the main ideas, key points, and important conclusions:\n\n{}\n\nSummary:",
        max_words,
        content.chars().take(4000).collect::<String>()
    )
}

/// Build a prompt for keyword extraction
pub fn build_keyword_extraction_prompt(content: &str, max_keywords: usize) -> String {
    format!(
        "Extract the {} most important keywords and key phrases from the following text. Return only the keywords separated by commas, without any additional text:\n\n{}\n\nKeywords:",
        max_keywords,
        content.chars().take(3000).collect::<String>()
    )
}

/// Build a prompt for question answering based on context
pub fn build_context_qa_prompt(context: &str, question: &str) -> String {
    format!(
        "Based on the following context, please answer the question. If the answer is not available in the context, please say so clearly.\n\nContext:\n{}\n\nQuestion: {}\n\nAnswer:",
        context.chars().take(5000).collect::<String>(),
        question
    )
}

/// Build a prompt for content classification
pub fn build_classification_prompt(content: &str, categories: &[String]) -> String {
    let categories_str = categories.join(", ");
    format!(
        "Classify the following content into one of these categories: {}. Respond with only the category name.\n\nContent:\n{}\n\nCategory:",
        categories_str,
        content.chars().take(2000).collect::<String>()
    )
}

/// Build a prompt for sentiment analysis
pub fn build_sentiment_prompt(content: &str) -> String {
    format!(
        "Analyze the sentiment of the following text. Respond with only one word: positive, negative, or neutral.\n\nText:\n{}\n\nSentiment:",
        content.chars().take(1500).collect::<String>()
    )
}

/// Build a prompt for translation
pub fn build_translation_prompt(content: &str, target_language: &str) -> String {
    format!(
        "Translate the following text to {}. Provide only the translation without any additional text:\n\n{}\n\nTranslation:",
        target_language,
        content.chars().take(2000).collect::<String>()
    )
}

/// Build a prompt for code explanation
pub fn build_code_explanation_prompt(code: &str, language: Option<&str>) -> String {
    let lang_part = if let Some(lang) = language {
        format!(" (written in {})", lang)
    } else {
        String::new()
    };

    format!(
        "Explain what the following code{} does. Describe its functionality, key components, and any important details:\n\n```\n{}\n```\n\nExplanation:",
        lang_part,
        code.chars().take(3000).collect::<String>()
    )
}

/// Build a prompt for brainstorming
pub fn build_brainstorming_prompt(topic: &str, context: Option<&str>) -> String {
    let context_part = if let Some(ctx) = context {
        format!(" Context: {}", ctx)
    } else {
        String::new()
    };

    format!(
        "Help me brainstorm ideas about: {}.{} Please provide creative, practical, and diverse suggestions:\n\nIdeas:",
        topic,
        context_part
    )
}

/// Build a conversation starter prompt
pub fn build_conversation_starter(agent: &Agent) -> String {
    let greeting = match agent.personality.to_lowercase().as_str() {
        "professional" => "Good day! I'm here to assist you with your professional needs.",
        "friendly" => "Hello there! I'm excited to help you today. What can we work on together?",
        "analytical" => "Greetings. I'm ready to help you analyze and solve problems systematically.",
        "creative" => "Hey! I'm here to spark some creativity and explore new ideas with you.",
        "concise" => "Hi. How can I help you efficiently today?",
        "detailed" => "Hello! I'm here to provide you with comprehensive assistance and detailed explanations.",
        _ => "Hello! I'm here to help you with whatever you need.",
    };

    let specialization_intro = match agent.specialization.to_lowercase().as_str() {
        "work" => " I specialize in professional tasks, project management, and workplace productivity.",
        "coding" => " I'm focused on programming, software development, and technical problem-solving.",
        "research" => " I excel at research, data analysis, and academic work.",
        "writing" => " I'm here to help with all your writing and content creation needs.",
        "personal" => " I'm your personal assistant for daily life management and organization.",
        "creative" => " I love helping with creative projects and artistic endeavors.",
        "technical" => " I specialize in technical support and troubleshooting.",
        _ => " I'm a general assistant ready to help with various tasks.",
    };

    format!("{}{} What would you like to explore today?", greeting, specialization_intro)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Agent;

    fn create_test_agent() -> Agent {
        Agent::new(
            "Test Agent".to_string(),
            "coding".to_string(),
            "analytical".to_string(),
            Some("Focus on Python programming".to_string()),
        )
    }

    #[test]
    fn test_build_agent_system_prompt() {
        let agent = create_test_agent();
        let prompt = build_agent_system_prompt(&agent);
        
        assert!(prompt.contains("Test Agent"));
        assert!(prompt.contains("analytical"));
        assert!(prompt.contains("programming"));
        assert!(prompt.contains("Focus on Python programming"));
    }

    #[test]
    fn test_personality_prompts() {
        assert!(get_personality_prompt("professional").contains("professional"));
        assert!(get_personality_prompt("friendly").contains("warm"));
        assert!(get_personality_prompt("analytical").contains("logical"));
        assert!(get_personality_prompt("creative").contains("imaginative"));
        assert!(get_personality_prompt("concise").contains("direct"));
        assert!(get_personality_prompt("detailed").contains("thorough"));
    }

    #[test]
    fn test_specialization_prompts() {
        assert!(get_specialization_prompt("coding").contains("programming"));
        assert!(get_specialization_prompt("writing").contains("content creation"));
        assert!(get_specialization_prompt("research").contains("data analysis"));
        assert!(get_specialization_prompt("work").contains("business"));
    }

    #[test]
    fn test_build_summarization_prompt() {
        let content = "This is test content for summarization.";
        let prompt = build_summarization_prompt(content, 50);
        
        assert!(prompt.contains("50 words"));
        assert!(prompt.contains(content));
        assert!(prompt.contains("Summary:"));
    }

    #[test]
    fn test_build_keyword_extraction_prompt() {
        let content = "This is test content for keyword extraction.";
        let prompt = build_keyword_extraction_prompt(content, 5);
        
        assert!(prompt.contains("5 most important"));
        assert!(prompt.contains(content));
        assert!(prompt.contains("Keywords:"));
    }

    #[test]
    fn test_conversation_starter() {
        let agent = create_test_agent();
        let starter = build_conversation_starter(&agent);
        
        assert!(starter.contains("Hello") || starter.contains("Greetings"));
        assert!(starter.contains("programming") || starter.contains("technical"));
    }
}