use tauri::State;
use crate::types::{Message, AppState};
use crate::storage::MessageStorage;
use crate::ai::response_generator::generate_agent_response;
use crate::utils::{validation, error::LocalMindError, Result};

/// Get messages for a specific agent
#[tauri::command]
pub async fn get_agent_messages(agent_id: String, state: State<'_, AppState>) -> Result<Vec<Message>, String> {
    validation::validate_uuid(&agent_id)
        .map_err(|e| e.to_string())?;

    let messages = state.messages.lock().await;
    Ok(messages.get(&agent_id).cloned().unwrap_or_default())
}

/// Send a message to an agent and get a response
#[tauri::command]
pub async fn send_message_to_agent(
    agent_id: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Validate inputs
    validation::validate_uuid(&agent_id)
        .map_err(|e| e.to_string())?;
    validation::validate_message_content(&message)
        .map_err(|e| e.to_string())?;

    // Get the agent info
    let agents = state.agents.lock().await;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| LocalMindError::agent_not_found(&agent_id).to_string())?
        .clone();
    drop(agents);

    // Save user message
    let user_message = Message::new_user_message(message.clone(), agent_id.clone());
    
    {
        let mut messages = state.messages.lock().await;
        messages
            .entry(agent_id.clone())
            .or_insert_with(Vec::new)
            .push(user_message);
    }

    // Generate AI response based on agent personality and specialization
    let ai_response = generate_agent_response(&agent, &message).await
        .map_err(|e| e.to_string())?;

    // Save AI response
    let ai_message = Message::new_agent_message(ai_response.clone(), agent_id.clone());

    {
        let mut messages = state.messages.lock().await;
        messages
            .entry(agent_id)
            .or_insert_with(Vec::new)
            .push(ai_message);
        
        // Save to disk
        MessageStorage::save(&messages).await
            .map_err(|e| e.to_string())?;
    }

    Ok(ai_response)
}

/// Clear all messages for a specific agent
#[tauri::command]
pub async fn clear_agent_messages(agent_id: String, state: State<'_, AppState>) -> Result<(), String> {
    validation::validate_uuid(&agent_id)
        .map_err(|e| e.to_string())?;

    // Clear from state
    state.messages.lock().await.remove(&agent_id);
    
    // Clear from storage
    MessageStorage::clear_agent_messages(&agent_id).await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Get message statistics for an agent
#[tauri::command]
pub async fn get_message_statistics(agent_id: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    validation::validate_uuid(&agent_id)
        .map_err(|e| e.to_string())?;

    let messages = state.get_agent_messages(&agent_id).await;
    
    let user_messages = messages.iter().filter(|m| m.is_from_user()).count();
    let agent_messages = messages.iter().filter(|m| m.is_from_agent()).count();
    
    // Calculate conversation start time
    let first_message_time = messages
        .first()
        .map(|m| m.timestamp.clone())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    
    // Calculate last activity time
    let last_message_time = messages
        .last()
        .map(|m| m.timestamp.clone())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    
    // Calculate total character count
    let total_characters: usize = messages.iter().map(|m| m.content.len()).sum();
    
    Ok(serde_json::json!({
        "agent_id": agent_id,
        "total_messages": messages.len(),
        "user_messages": user_messages,
        "agent_messages": agent_messages,
        "total_characters": total_characters,
        "first_message_at": first_message_time,
        "last_message_at": last_message_time,
        "average_message_length": if messages.is_empty() { 0 } else { total_characters / messages.len() },
    }))
}

/// Search messages by content
#[tauri::command]
pub async fn search_messages(
    query: String,
    agent_id: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<Message>, String> {
    if query.trim().is_empty() {
        return Err("Search query cannot be empty".to_string());
    }

    let query_lower = query.to_lowercase();
    let limit = limit.unwrap_or(50).min(500); // Cap at 500 results
    
    let messages = state.messages.lock().await;
    let mut matching_messages = Vec::new();

    // Search in specific agent's messages or all messages
    if let Some(agent_id) = agent_id {
        validation::validate_uuid(&agent_id)
            .map_err(|e| e.to_string())?;
        
        if let Some(agent_messages) = messages.get(&agent_id) {
            for message in agent_messages {
                if message.content.to_lowercase().contains(&query_lower) {
                    matching_messages.push(message.clone());
                    if matching_messages.len() >= limit {
                        break;
                    }
                }
            }
        }
    } else {
        // Search all messages
        for agent_messages in messages.values() {
            for message in agent_messages {
                if message.content.to_lowercase().contains(&query_lower) {
                    matching_messages.push(message.clone());
                    if matching_messages.len() >= limit {
                        break;
                    }
                }
            }
            if matching_messages.len() >= limit {
                break;
            }
        }
    }

    // Sort by timestamp (newest first)
    matching_messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(matching_messages)
}

/// Get recent messages across all agents
#[tauri::command]
pub async fn get_recent_messages(limit: Option<usize>, state: State<'_, AppState>) -> Result<Vec<Message>, String> {
    let limit = limit.unwrap_or(20).min(100); // Cap at 100 results
    let messages = state.messages.lock().await;
    
    let mut all_messages = Vec::new();
    
    // Collect all messages
    for agent_messages in messages.values() {
        all_messages.extend(agent_messages.clone());
    }
    
    // Sort by timestamp (newest first)
    all_messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // Take the requested number
    all_messages.truncate(limit);
    
    Ok(all_messages)
}

/// Export messages for an agent to a text format
#[tauri::command]
pub async fn export_agent_messages(agent_id: String, format: String, state: State<'_, AppState>) -> Result<String, String> {
    validation::validate_uuid(&agent_id)
        .map_err(|e| e.to_string())?;

    let messages = state.get_agent_messages(&agent_id).await;
    
    match format.to_lowercase().as_str() {
        "json" => {
            serde_json::to_string_pretty(&messages)
                .map_err(|e| format!("Failed to serialize messages: {}", e))
        },
        "txt" | "text" => {
            let mut output = String::new();
            output.push_str(&format!("Chat Export for Agent: {}\n", agent_id));
            output.push_str(&format!("Exported at: {}\n", chrono::Utc::now().to_rfc3339()));
            output.push_str("=" .repeat(50));
            output.push('\n');
            
            for message in messages {
                let sender = if message.is_from_user() { "User" } else { "Agent" };
                output.push_str(&format!("\n[{}] {} at {}:\n", sender, message.id, message.timestamp));
                output.push_str(&message.content);
                output.push_str("\n");
            }
            
            Ok(output)
        },
        _ => Err("Unsupported format. Use 'json' or 'text'".to_string())
    }
}

/// Get total message count across all agents
#[tauri::command]
pub async fn get_total_message_count(state: State<'_, AppState>) -> Result<usize, String> {
    Ok(state.total_message_count().await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Agent, AppState};
    use std::sync::Arc;

    fn create_test_state() -> Arc<AppState> {
        Arc::new(AppState::new())
    }

    async fn create_test_agent(state: &AppState) -> Agent {
        let agent = Agent::new(
            "Test Agent".to_string(),
            "general".to_string(),
            "friendly".to_string(),
            None,
        );
        
        state.agents.lock().await.push(agent.clone());
        agent
    }

    #[tokio::test]
    async fn test_send_and_get_messages() {
        let state = create_test_state();
        let tauri_state = State::from(state.as_ref());
        
        let agent = create_test_agent(&state).await;
        
        // Send a message (Note: This will fail without proper AI setup, but tests the validation)
        let result = send_message_to_agent(
            agent.id.clone(),
            "Hello test".to_string(),
            tauri_state,
        ).await;
        
        // Should fail because AI service isn't set up, but validation should pass
        if result.is_err() {
            // Get messages should still work
            let messages = get_agent_messages(agent.id, tauri_state).await.unwrap();
            // User message should be saved even if AI response failed
            assert_eq!(messages.len(), 1);
            assert!(messages[0].is_from_user());
        }
    }

    #[tokio::test]
    async fn test_message_validation() {
        let state = create_test_state();
        let tauri_state = State::from(state.as_ref());
        
        let agent = create_test_agent(&state).await;
        
        // Test empty message
        let result = send_message_to_agent(
            agent.id,
            "".to_string(),
            tauri_state,
        ).await;
        
        assert!(result.is_err());
    }
}