use anyhow::Result;
use crate::types::{Agent, Message, Document};
use crate::state::AppState;

/// Commands module for handling application commands
/// These were previously Tauri commands, now integrated directly

pub async fn get_agents(state: &AppState) -> Result<Vec<Agent>> {
    let agents = state.agents.lock().await;
    Ok(agents.values().cloned().collect())
}

pub async fn create_agent(
    state: &AppState,
    name: String,
    model: String,
    description: String,
    system_prompt: String,
) -> Result<Agent> {
    let agent = Agent::new(name, model)
        .with_description(description)
        .with_system_prompt(system_prompt);
    
    let mut agents = state.agents.lock().await;
    agents.insert(agent.id.clone(), agent.clone());
    
    // Save to storage
    crate::storage::AgentStorage::save(&agents).await?;
    
    Ok(agent)
}

pub async fn update_agent(
    state: &AppState,
    agent_id: String,
    updates: AgentUpdate,
) -> Result<Agent> {
    let mut agents = state.agents.lock().await;
    
    let agent = agents.get_mut(&agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;
    
    // Apply updates
    if let Some(name) = updates.name {
        agent.name = name;
    }
    if let Some(description) = updates.description {
        agent.description = description;
    }
    if let Some(system_prompt) = updates.system_prompt {
        agent.system_prompt = system_prompt;
    }
    if let Some(personality) = updates.personality {
        agent.personality = personality;
    }
    
    let updated_agent = agent.clone();
    
    // Save to storage
    crate::storage::AgentStorage::save(&agents).await?;
    
    Ok(updated_agent)
}

pub async fn delete_agent(
    state: &AppState,
    agent_id: String,
) -> Result<()> {
    let mut agents = state.agents.lock().await;
    agents.remove(&agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;
    
    // Save to storage
    crate::storage::AgentStorage::save(&agents).await?;
    
    // Also remove associated messages
    let mut messages = state.messages.lock().await;
    messages.remove(&agent_id);
    crate::storage::MessageStorage::save(&messages).await?;
    
    Ok(())
}

pub async fn get_agent_messages(
    state: &AppState,
    agent_id: String,
) -> Result<Vec<Message>> {
    let messages = state.messages.lock().await;
    Ok(messages.get(&agent_id).cloned().unwrap_or_default())
}

pub async fn send_message_to_agent(
    state: &AppState,
    agent_id: String,
    message: String,
) -> Result<String> {
    // Get the agent
    let agents = state.agents.lock().await;
    let agent = agents.get(&agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found"))?
        .clone();
    drop(agents);
    
    // Generate AI response
    let response = crate::ai::generate_agent_response(&agent, &message).await?;
    
    // Store messages
    let mut messages = state.messages.lock().await;
    let agent_messages = messages.entry(agent_id.clone()).or_insert_with(Vec::new);
    
    // Add user message
    agent_messages.push(Message::new(
        message,
        agent_id.clone(),
        crate::types::message::MessageRole::User,
    ));
    
    // Add AI response
    agent_messages.push(Message::new(
        response.clone(),
        agent_id,
        crate::types::message::MessageRole::Assistant,
    ));
    
    // Save to storage
    crate::storage::MessageStorage::save(&messages).await?;
    
    Ok(response)
}

pub async fn clear_chat(
    state: &AppState,
    agent_id: String,
) -> Result<()> {
    let mut messages = state.messages.lock().await;
    messages.remove(&agent_id);
    
    // Save to storage
    crate::storage::MessageStorage::save(&messages).await?;
    
    Ok(())
}

pub async fn search_memories(
    state: &AppState,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<crate::memory::memory_types::Memory>> {
    // If memory system is initialized, search through it
    if let Some(memory_coordinator) = &state.memory_system {
        let results = memory_coordinator.search(&query, limit.unwrap_or(10)).await?;
        Ok(results)
    } else {
        Ok(vec![])
    }
}

/// Agent update structure
#[derive(Clone, Debug)]
pub struct AgentUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub personality: Option<String>,
}