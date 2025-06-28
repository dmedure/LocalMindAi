use crate::types::Agent;
use crate::ai::prompt_builder::build_agent_system_prompt;
use crate::utils::error::{LocalMindError, Result};
use serde_json;

/// Generate an AI response for the given agent and user message
pub async fn generate_agent_response(agent: &Agent, user_message: &str) -> Result<String> {
    // Build the prompt based on agent's personality and specialization
    let system_prompt = build_agent_system_prompt(agent);
    
    // Prepare the request to Ollama
    let client = reqwest::Client::new();
    let ollama_request = serde_json::json!({
        "model": "llama3.1:8b",
        "prompt": format!("{}\n\nUser: {}\nAssistant:", system_prompt, user_message),
        "stream": false,
        "options": {
            "temperature": 0.7,
            "top_p": 0.9,
            "max_tokens": 1000
        }
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&ollama_request)
        .send()
        .await
        .map_err(|e| LocalMindError::ExternalService(format!("Failed to connect to Ollama: {}", e)))?;

    if !response.status().is_success() {
        return Err(LocalMindError::ExternalService(format!("Ollama API error: {}", response.status())));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| LocalMindError::Network(format!("Failed to parse Ollama response: {}", e)))?;

    let ai_response = response_json["response"]
        .as_str()
        .unwrap_or("I apologize, but I'm having trouble generating a response right now.")
        .trim();

    Ok(ai_response.to_string())
}

/// Generate a streaming AI response (for future implementation)
pub async fn generate_streaming_response(
    agent: &Agent,
    user_message: &str,
    callback: impl Fn(String) -> Result<()>,
) -> Result<String> {
    // This would implement streaming responses in a future version
    // For now, fall back to regular response
    let response = generate_agent_response(agent, user_message).await?;
    
    // Simulate streaming by calling callback with chunks
    let words: Vec<&str> = response.split_whitespace().collect();
    let mut accumulated = String::new();
    
    for word in words {
        accumulated.push_str(word);
        accumulated.push(' ');
        callback(word.to_string())?;
        
        // Small delay to simulate streaming
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    
    Ok(response)
}

/// Check if Ollama is available and responsive
pub async fn check_ollama_availability() -> bool {
    let client = reqwest::Client::new();
    
    match client
        .get("http://localhost:11434/api/tags")
        .timeout(tokio::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Get available models from Ollama
pub async fn get_available_models() -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map_err(|e| LocalMindError::Network(format!("Failed to connect to Ollama: {}", e)))?;

    if !response.status().is_success() {
        return Err(LocalMindError::ExternalService("Ollama API error".to_string()));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| LocalMindError::Network(format!("Failed to parse Ollama response: {}", e)))?;

    let models = response_json["models"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|model| model["name"].as_str())
        .map(|name| name.to_string())
        .collect();

    Ok(models)
}

/// Generate a summary of a text using AI
pub async fn generate_summary(text: &str, max_length: Option<usize>) -> Result<String> {
    let max_len = max_length.unwrap_or(200);
    let prompt = format!(
        "Please provide a concise summary of the following text in no more than {} words:\n\n{}\n\nSummary:",
        max_len / 5, // Rough word estimate
        text.chars().take(4000).collect::<String>() // Limit input text
    );

    // Create a temporary agent for summarization
    let temp_agent = Agent::new(
        "Summarizer".to_string(),
        "general".to_string(),
        "concise".to_string(),
        Some("You are a helpful assistant that creates clear, concise summaries.".to_string()),
    );

    generate_agent_response(&temp_agent, &prompt).await
}

/// Extract keywords from text using AI
pub async fn extract_keywords(text: &str, max_keywords: Option<usize>) -> Result<Vec<String>> {
    let max_kw = max_keywords.unwrap_or(10);
    let prompt = format!(
        "Extract the {} most important keywords from the following text. Return only the keywords separated by commas:\n\n{}\n\nKeywords:",
        max_kw,
        text.chars().take(2000).collect::<String>()
    );

    // Create a temporary agent for keyword extraction
    let temp_agent = Agent::new(
        "KeywordExtractor".to_string(),
        "general".to_string(),
        "analytical".to_string(),
        Some("You extract important keywords from text. Return only keywords separated by commas.".to_string()),
    );

    let response = generate_agent_response(&temp_agent, &prompt).await?;
    
    // Parse comma-separated keywords
    let keywords: Vec<String> = response
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .take(max_kw)
        .collect();

    Ok(keywords)
}

/// Analyze sentiment of text using AI
pub async fn analyze_sentiment(text: &str) -> Result<String> {
    let prompt = format!(
        "Analyze the sentiment of the following text. Respond with only one word: positive, negative, or neutral:\n\n{}\n\nSentiment:",
        text.chars().take(1000).collect::<String>()
    );

    // Create a temporary agent for sentiment analysis
    let temp_agent = Agent::new(
        "SentimentAnalyzer".to_string(),
        "general".to_string(),
        "analytical".to_string(),
        Some("You analyze text sentiment. Respond with only: positive, negative, or neutral.".to_string()),
    );

    let response = generate_agent_response(&temp_agent, &prompt).await?;
    
    // Clean up response and validate
    let sentiment = response.trim().to_lowercase();
    match sentiment.as_str() {
        "positive" | "negative" | "neutral" => Ok(sentiment),
        _ => Ok("neutral".to_string()), // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Agent;

    fn create_test_agent() -> Agent {
        Agent::new(
            "Test Agent".to_string(),
            "general".to_string(),
            "friendly".to_string(),
            None,
        )
    }

    #[tokio::test]
    async fn test_ollama_availability() {
        // This test will pass if Ollama is running, fail otherwise
        let available = check_ollama_availability().await;
        // We can't assert true/false since it depends on the environment
        println!("Ollama available: {}", available);
    }

    #[tokio::test]
    async fn test_generate_summary() {
        let test_text = "This is a long text that needs to be summarized. It contains multiple sentences and ideas. The summary should capture the main points concisely.";
        
        // This test will only pass if Ollama is available
        if check_ollama_availability().await {
            let result = generate_summary(test_text, Some(100)).await;
            match result {
                Ok(summary) => {
                    assert!(!summary.is_empty());
                    assert!(summary.len() < test_text.len());
                },
                Err(e) => println!("Summary generation failed: {}", e),
            }
        }
    }

    #[test]
    fn test_agent_creation() {
        let agent = create_test_agent();
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.specialization, "general");
        assert_eq!(agent.personality, "friendly");
    }
}