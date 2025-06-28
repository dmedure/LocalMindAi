use crate::utils::error::{LocalMindError, Result};

/// Validation utilities for user input and data integrity

/// Validate agent name
pub fn validate_agent_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    
    if trimmed.is_empty() {
        return Err(LocalMindError::validation_failed("agent_name", "Name cannot be empty"));
    }
    
    if trimmed.len() > 100 {
        return Err(LocalMindError::validation_failed("agent_name", "Name cannot exceed 100 characters"));
    }
    
    // Check for invalid characters
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(LocalMindError::validation_failed("agent_name", "Name cannot contain line breaks"));
    }
    
    Ok(())
}

/// Validate message content
pub fn validate_message_content(content: &str) -> Result<()> {
    let trimmed = content.trim();
    
    if trimmed.is_empty() {
        return Err(LocalMindError::validation_failed("message_content", "Message cannot be empty"));
    }
    
    if trimmed.len() > 10000 {
        return Err(LocalMindError::validation_failed("message_content", "Message cannot exceed 10,000 characters"));
    }
    
    Ok(())
}

/// Validate agent specialization
pub fn validate_specialization(specialization: &str) -> Result<()> {
    let valid_specializations = [
        "general", "work", "coding", "research", 
        "writing", "personal", "creative", "technical"
    ];
    
    if !valid_specializations.contains(&specialization.to_lowercase().as_str()) {
        return Err(LocalMindError::validation_failed(
            "specialization", 
            &format!("Invalid specialization. Must be one of: {}", valid_specializations.join(", "))
        ));
    }
    
    Ok(())
}

/// Validate agent personality
pub fn validate_personality(personality: &str) -> Result<()> {
    let valid_personalities = [
        "professional", "friendly", "analytical", 
        "creative", "concise", "detailed"
    ];
    
    if !valid_personalities.contains(&personality.to_lowercase().as_str()) {
        return Err(LocalMindError::validation_failed(
            "personality", 
            &format!("Invalid personality. Must be one of: {}", valid_personalities.join(", "))
        ));
    }
    
    Ok(())
}

/// Validate agent instructions (if provided)
pub fn validate_instructions(instructions: &Option<String>) -> Result<()> {
    if let Some(instructions) = instructions {
        let trimmed = instructions.trim();
        
        if trimmed.len() > 1000 {
            return Err(LocalMindError::validation_failed("instructions", "Instructions cannot exceed 1,000 characters"));
        }
    }
    
    Ok(())
}

/// Validate UUID format
pub fn validate_uuid(id: &str) -> Result<()> {
    uuid::Uuid::parse_str(id)
        .map_err(|_| LocalMindError::validation_failed("id", "Invalid UUID format"))?;
    Ok(())
}

/// Validate file path exists and is readable
pub fn validate_file_path(path: &str) -> Result<()> {
    let path = std::path::Path::new(path);
    
    if !path.exists() {
        return Err(LocalMindError::validation_failed("file_path", "File does not exist"));
    }
    
    if !path.is_file() {
        return Err(LocalMindError::validation_failed("file_path", "Path is not a file"));
    }
    
    // Try to read file metadata to check if we have read permissions
    std::fs::metadata(path)
        .map_err(|_| LocalMindError::validation_failed("file_path", "Cannot read file metadata"))?;
    
    Ok(())
}

/// Sanitize user input by removing potentially harmful characters
pub fn sanitize_input(input: &str) -> String {
    input
        .trim()
        .replace('\0', "") // Remove null bytes
        .replace('\r', "") // Remove carriage returns
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t') // Keep only printable chars + newlines/tabs
        .collect()
}

/// Check if a string is a valid ISO 8601 timestamp
pub fn validate_timestamp(timestamp: &str) -> Result<()> {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map_err(|_| LocalMindError::validation_failed("timestamp", "Invalid ISO 8601 timestamp format"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_agent_name() {
        assert!(validate_agent_name("Valid Name").is_ok());
        assert!(validate_agent_name("").is_err());
        assert!(validate_agent_name("   ").is_err());
        assert!(validate_agent_name("Name\nwith\nnewlines").is_err());
        assert!(validate_agent_name(&"x".repeat(101)).is_err());
    }

    #[test]
    fn test_validate_message_content() {
        assert!(validate_message_content("Hello world").is_ok());
        assert!(validate_message_content("").is_err());
        assert!(validate_message_content("   ").is_err());
        assert!(validate_message_content(&"x".repeat(10001)).is_err());
    }

    #[test]
    fn test_validate_specialization() {
        assert!(validate_specialization("general").is_ok());
        assert!(validate_specialization("coding").is_ok());
        assert!(validate_specialization("invalid").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(sanitize_input("  hello world  "), "hello world");
        assert_eq!(sanitize_input("hello\0world"), "helloworld");
        assert_eq!(sanitize_input("hello\rworld"), "helloworld");
    }
}