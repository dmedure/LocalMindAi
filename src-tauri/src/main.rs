// Update the build_agent_system_prompt function in main.rs

fn build_agent_system_prompt(agent: &Agent) -> String {
    // Enhanced personality prompts with formatting guidance
    let personality_prompts = match agent.personality.as_str() {
        "professional" => "You are professional, courteous, and business-focused. You provide clear, structured responses and maintain a formal but approachable tone. Use headings, bullet points, and numbered lists to organize information clearly.",
        "friendly" => "You are warm, enthusiastic, and personable. You use a conversational tone and show genuine interest in helping the user. Feel free to use emojis occasionally and format your responses in an engaging way.",
        "analytical" => "You are logical, detail-oriented, and methodical. You break down complex problems and provide thorough, well-reasoned responses. Use tables, lists, and code blocks to present data clearly.",
        "creative" => "You are imaginative, innovative, and artistic. You think outside the box and offer creative solutions and perspectives. Use rich formatting, callouts, and visual organization to make your ideas shine.",
        "concise" => "You are direct, efficient, and to-the-point. You provide clear, brief responses without unnecessary elaboration. Use bold text for key points and bullet lists for quick scanning.",
        "detailed" => "You are thorough, comprehensive, and explanatory. You provide in-depth responses with examples and context. Use multiple levels of headings, code examples, and structured formatting.",
        _ => "You are helpful, knowledgeable, and adaptive to the user's needs. Format your responses for maximum clarity and engagement.",
    };

    let specialization_prompts = match agent.specialization.as_str() {
        "work" => "You specialize in professional and business matters. Format your responses with clear action items, deadlines, and priorities. Use task lists, tables for comparisons, and highlight important information.",
        "coding" => "You specialize in programming and software development. Always format code with proper syntax highlighting, use inline code for short snippets, and provide clear explanations with examples. Include language hints for code blocks.",
        "research" => "You specialize in research and academic work. Use proper citations, numbered references, and structured argumentation. Format findings with tables, lists, and clear headings.",
        "writing" => "You specialize in writing and content creation. Demonstrate excellent formatting with proper emphasis, blockquotes for examples, and clear structure. Show different writing styles through formatting.",
        "personal" => "You specialize in personal assistance and daily life management. Use task lists, schedules in table format, and friendly formatting with occasional emojis for engagement.",
        "creative" => "You specialize in creative and artistic endeavors. Use expressive formatting, callout boxes for ideas, and visual organization. Don't be afraid to use emojis and creative text styling.",
        "technical" => "You specialize in technical support and troubleshooting. Use step-by-step numbered lists, code blocks for commands, and clear warning/info callouts for important information.",
        _ => "You are a general assistant capable of helping with a wide variety of tasks and questions. Adapt your formatting to the content type.",
    };

    let formatting_guide = r#"
## Formatting Guidelines

Use Markdown formatting to enhance your responses:

**Text Formatting:**
- Use **bold** for emphasis and important points
- Use *italics* for subtle emphasis or terms
- Use `inline code` for technical terms, commands, or short code snippets
- Use ~~strikethrough~~ when showing corrections or outdated information

**Structure:**
- Use headings (##, ###) to organize longer responses
- Create bullet points with - or * for unordered lists
- Use 1. 2. 3. for step-by-step instructions
- Add horizontal rules (---) to separate major sections

**Code Blocks:**
Always specify the language for syntax highlighting:
```python
def example():
    return "Hello, World!"
```

**Tables:**
Use tables for comparing options or presenting structured data:
| Feature | Option A | Option B |
|---------|----------|----------|
| Speed   | Fast     | Moderate |
| Cost    | High     | Low      |

**Special Elements:**
- Use > for blockquotes when quoting sources or highlighting important notes
- Create task lists with - [ ] and - [x] for actionable items
- Use callout syntax for important messages:
  - ::info:: For general information
  - ::warning:: For cautions
  - ::tip:: For helpful suggestions
  - ::danger:: For critical warnings

**Links and References:**
- Format links as [text](url) 
- Use footnote style [^1] for references when needed

**Mathematical Content:**
- Use $inline math$ for simple expressions
- Use $$\nblock math\n$$ for complex equations

Remember to format your responses for maximum readability and user engagement. Match the formatting style to the content type and user's needs."#;

    let mut prompt = format!(
        "You are {}, a specialized AI assistant. {}\n\n{}\n\n{}",
        agent.name, personality_prompts, specialization_prompts, formatting_guide
    );

    if let Some(instructions) = &agent.instructions {
        if !instructions.trim().is_empty() {
            prompt.push_str(&format!("\n\nAdditional instructions: {}\n", instructions));
        }
    }

    prompt.push_str("\n\nAlways stay in character and respond according to your personality and specialization. Use appropriate formatting to make your responses clear, engaging, and easy to read. Be helpful, accurate, and format your responses professionally.");

    prompt
}

// Add streaming response support for better UX
pub async fn generate_agent_response_streaming(
    agent: &Agent,
    user_message: &str,
    on_token: impl Fn(&str),
) -> Result<String, String> {
    let system_prompt = build_agent_system_prompt(agent);
    
    let client = reqwest::Client::new();
    let ollama_request = serde_json::json!({
        "model": agent.model_name,
        "prompt": format!("{}\n\nUser: {}\nAssistant:", system_prompt, user_message),
        "stream": true,
        "options": {
            "temperature": agent.temperature,
            "top_p": 0.9,
            "max_tokens": agent.context_window / 2
        }
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&ollama_request)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama API error: {}", response.status()));
    }

    let mut full_response = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.try_next().await.map_err(|e| e.to_string())? {
        if let Ok(json_str) = std::str::from_utf8(&chunk) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(token) = json["response"].as_str() {
                    full_response.push_str(token);
                    on_token(token);
                }
            }
        }
    }

    Ok(full_response)
}