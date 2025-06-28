use tauri::State;
use crate::types::{Agent, AppState};
use crate::storage::{AgentStorage, MessageStorage};
use crate::utils::{validation, error::LocalMindError, Result};

/// Get all agents
#[tauri::command]
pub async fn get_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    let agents = state.agents.lock().await;
    Ok(agents.clone())
}

/// Create a new agent
#[tauri::command]
pub async fn create_agent(agent: Agent, state: State<'_, AppState>) -> Result<(), String> {
    // Validate agent data
    validation::validate_agent_name(&agent.name)
        .map_err(|e| e.to_string())?;
    validation::validate_specialization(&agent.specialization)
        .map_err(|e| e.to_string())?;
    validation::validate_personality(&agent.personality)
        .map_err(|e| e.to_string())?;
    validation::validate_instructions(&agent.instructions)
        .map_err(|e| e.to_string())?;

    // Add to state
    let mut agents = state.agents.lock().await;
    agents.push(agent);
    
    // Save to file
    AgentStorage::save(&agents).await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Update an existing agent
#[tauri::command]
pub async fn update_agent(agent_id: String, updated_agent: Agent, state: State<'_, AppState>) -> Result<Agent, String> {
    // Validate agent data
    validation::validate_agent_name(&updated_agent.name)
        .map_err(|e| e.to_string())?;
    validation::validate_specialization(&updated_agent.specialization)
        .map_err(|e| e.to_string())?;
    validation::validate_personality(&updated_agent.personality)
        .map_err(|e| e.to_string())?;
    validation::validate_instructions(&updated_agent.instructions)
        .map_err(|e| e.to_string())?;

    let mut agents = state.agents.lock().await;
    
    // Find and update the agent
    let agent_index = agents
        .iter()
        .position(|a| a.id == agent_id)
        .ok_or_else(|| LocalMindError::agent_not_found(&agent_id).to_string())?;
    
    // Preserve the original ID and creation date
    let mut agent_to_update = updated_agent;
    agent_to_update.id = agent_id;
    agent_to_update.created_at = agents[agent_index].created_at.clone();
    
    agents[agent_index] = agent_to_update.clone();
    
    // Save to file
    AgentStorage::save(&agents).await
        .map_err(|e| e.to_string())?;
    
    Ok(agent_to_update)
}

/// Delete an agent
#[tauri::command]
pub async fn delete_agent(agent_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut agents = state.agents.lock().await;
    
    // Find the agent
    let initial_len = agents.len();
    agents.retain(|a| a.id != agent_id);
    
    if agents.len() == initial_len {
        return Err(LocalMindError::agent_not_found(&agent_id).to_string());
    }
    
    // Save updated agents list
    AgentStorage::save(&agents).await
        .map_err(|e| e.to_string())?;
    
    // Clear the agent's messages
    MessageStorage::clear_agent_messages(&agent_id).await
        .map_err(|e| e.to_string())?;
    
    // Also clear from state
    state.messages.lock().await.remove(&agent_id);
    
    Ok(())
}

/// Get agent by ID
#[tauri::command]
pub async fn get_agent_by_id(agent_id: String, state: State<'_, AppState>) -> Result<Option<Agent>, String> {
    let agents = state.agents.lock().await;
    Ok(agents.iter().find(|a| a.id == agent_id).cloned())
}

/// Get agent statistics
#[tauri::command]
pub async fn get_agent_statistics(agent_id: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    // Verify agent exists
    let agents = state.agents.lock().await;
    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| LocalMindError::agent_not_found(&agent_id).to_string())?;
    
    // Get message count for this agent
    let message_count = state.get_agent_messages(&agent_id).await.len();
    
    // Calculate days since creation
    let created_at = chrono::DateTime::parse_from_rfc3339(&agent.created_at)
        .map_err(|e| format!("Invalid creation date: {}", e))?;
    let days_active = (chrono::Utc::now().signed_duration_since(created_at).num_days()).max(0);
    
    Ok(serde_json::json!({
        "agent_id": agent_id,
        "name": agent.name,
        "specialization": agent.specialization,
        "personality": agent.personality,
        "message_count": message_count,
        "created_at": agent.created_at,
        "days_active": days_active,
        "has_instructions": agent.has_instructions(),
    }))
}

/// List all agent specializations
#[tauri::command]
pub async fn get_agent_specializations() -> Result<Vec<String>, String> {
    Ok(vec![
        "general".to_string(),
        "work".to_string(),
        "coding".to_string(),
        "research".to_string(),
        "writing".to_string(),
        "personal".to_string(),
        "creative".to_string(),
        "technical".to_string(),
    ])
}

/// List all agent personalities
#[tauri::command]
pub async fn get_agent_personalities() -> Result<Vec<String>, String> {
    Ok(vec![
        "professional".to_string(),
        "friendly".to_string(),
        "analytical".to_string(),
        "creative".to_string(),
        "concise".to_string(),
        "detailed".to_string(),
    ])
}

/// Clone an agent with a new name
#[tauri::command]
pub async fn clone_agent(agent_id: String, new_name: String, state: State<'_, AppState>) -> Result<Agent, String> {
    validation::validate_agent_name(&new_name)
        .map_err(|e| e.to_string())?;

    let agents = state.agents.lock().await;
    let source_agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| LocalMindError::agent_not_found(&agent_id).to_string())?;
    
    // Create a new agent based on the source agent
    let cloned_agent = Agent::new(
        new_name,
        source_agent.specialization.clone(),
        source_agent.personality.clone(),
        source_agent.instructions.clone(),
    );
    
    drop(agents); // Release the lock before calling create_agent
    
    // Use the existing create_agent command to handle the creation
    create_agent(cloned_agent.clone(), state).await?;
    
    Ok(cloned_agent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AppState;
    use std::sync::Arc;

    fn create_test_state() -> Arc<AppState> {
        Arc::new(AppState::new())
    }

    #[tokio::test]
    async fn test_create_and_get_agent() {
        let state = create_test_state();
        let tauri_state = State::from(state.as_ref());
        
        let test_agent = Agent::new(
            "Test Agent".to_string(),
            "general".to_string(),
            "friendly".to_string(),
            None,
        );
        
        // Create agent
        let result = create_agent(test_agent.clone(), tauri_state).await;
        assert!(result.is_ok());
        
        // Get agents
        let agents = get_agents(tauri_state).await.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, test_agent.name);
    }

    #[tokio::test]
    async fn test_agent_validation() {
        let state = create_test_state();
        let tauri_state = State::from(state.as_ref());
        
        // Test invalid name
        let invalid_agent = Agent::new(
            "".to_string(), // Empty name should fail
            "general".to_string(),
            "friendly".to_string(),
            None,
        );
        
        let result = create_agent(invalid_agent, tauri_state).await;
        assert!(result.is_err());
    }
}