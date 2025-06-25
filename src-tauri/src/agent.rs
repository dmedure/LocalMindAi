use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use crate::knowledge::{Document, KnowledgeBase};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentMemory {
    pub conversation_history: Vec<ChatMessage>,
    pub user_preferences: HashMap<String, serde_json::Value>,
    pub learned_patterns: Vec<String>,
    pub custom_workflows: Vec<Workflow>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub content: String,
    pub role: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteAgentExport {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub model_info: ModelInfo,
    pub memory: AgentMemory,
    pub configuration: AgentConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub base_model: String,
    pub fine_tuned: bool,
    pub checkpoint_path: Option<String>,
    pub model_size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: Option<u32>,
    pub system_prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub options: OllamaOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaOptions {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
}

pub struct Agent {
    client: reqwest::Client,
    model: String,
    base_url: String,
    memory: AgentMemory,
    config: AgentConfig,
}

impl Agent {
    pub async fn new(model: &str) -> Result<Self> {
        let client = reqwest::Client::new();
        let base_url = "http://localhost:11434".to_string();
        
        // Test connection to Ollama
        let test_url = format!("{}/api/tags", base_url);
        client.get(&test_url).send().await
            .map_err(|_| anyhow!("Failed to connect to Ollama. Make sure Ollama is running."))?;
        
        let memory = AgentMemory {
            conversation_history: Vec::new(),
            user_preferences: HashMap::new(),
            learned_patterns: Vec::new(),
            custom_workflows: Vec::new(),
            last_updated: Utc::now(),
        };
        
        let config = AgentConfig {
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: Some(2048),
            system_prompt: "You are a helpful local AI assistant.".to_string(),
        };
        
        Ok(Agent {
            client,
            model: model.to_string(),
            base_url,
            memory,
            config,
        })
    }
    
    pub async fn generate_response(&self, user_input: &str, context: &[Document]) -> Result<String> {
        let prompt = self.build_prompt(user_input, context);
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: self.config.temperature,
                top_p: self.config.top_p,
                max_tokens: self.config.max_tokens,
            },
        };
        
        let url = format!("{}/api/generate", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Ollama request failed: {}", response.status()));
        }
        
        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response)
    }
    
    fn build_prompt(&self, user_input: &str, context: &[Document]) -> String {
        let mut prompt = String::new();
        
        // System prompt
        prompt.push_str("You are a helpful local AI assistant. You have access to the user's documents and can help with various tasks. Always be concise and accurate.\n\n");
        
        // Add context if available
        if !context.is_empty() {
            prompt.push_str("Relevant context from your knowledge base:\n");
            for (i, doc) in context.iter().enumerate() {
                prompt.push_str(&format!("{}. {} (from: {})\n", 
                    i + 1, 
                    doc.content.chars().take(200).collect::<String>(),
                    doc.source
                ));
            }
            prompt.push_str("\n");
        }
        
        // Add user input
        prompt.push_str("User: ");
        prompt.push_str(user_input);
        prompt.push_str("\n\nAssistant: ");
        
        prompt
    }
    
    pub async fn summarize_document(&self, content: &str) -> Result<String> {
        let prompt = format!(
            "Please provide a concise summary of the following document:\n\n{}\n\nSummary:",
            content.chars().take(4000).collect::<String>()
        );
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: 0.3,
                top_p: 0.8,
                max_tokens: Some(512),
            },
        };
        
        let url = format!("{}/api/generate", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response)
    }
    
    pub async fn extract_keywords(&self, content: &str) -> Result<Vec<String>> {
        let prompt = format!(
            "Extract 5-10 important keywords from the following text. Return only the keywords separated by commas:\n\n{}\n\nKeywords:",
            content.chars().take(2000).collect::<String>()
        );
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: 0.1,
                top_p: 0.5,
                max_tokens: Some(100),
            },
        };
        
        let url = format!("{}/api/generate", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        let ollama_response: OllamaResponse = response.json().await?;
        
        // Parse comma-separated keywords
        let keywords: Vec<String> = ollama_response.response
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        Ok(keywords)
    }
    
    pub async fn is_model_available(&self, model: &str) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        let text = response.text().await?;
        Ok(text.contains(model))
    }
    
    // Export/Import Methods
    pub async fn export_complete_state(
        &self, 
        export_path: &str,
        knowledge_base: &KnowledgeBase
    ) -> Result<String> {
        let model_info = ModelInfo {
            base_model: self.model.clone(),
            fine_tuned: false, // TODO: Track if model is fine-tuned
            checkpoint_path: None,
            model_size: "8B".to_string(),
        };
        
        let export = CompleteAgentExport {
            version: "1.0".to_string(),
            exported_at: Utc::now(),
            model_info,
            memory: self.memory.clone(),
            configuration: self.config.clone(),
        };
        
        let json_string = serde_json::to_string_pretty(&export)?;
        fs::write(export_path, json_string)?;
        
        Ok(format!("Complete agent state exported to {}", export_path))
    }
    
    pub async fn import_complete_state(
        &mut self,
        import_path: &str
    ) -> Result<String> {
        let import_data: CompleteAgentExport = serde_json::from_str(
            &fs::read_to_string(import_path)?
        )?;
        
        self.memory = import_data.memory;
        self.config = import_data.configuration;
        // Note: model info is imported but not applied (would need model switching)
        
        Ok(format!("Agent state imported from {}", import_path))
    }
    
    pub fn export_memory(&self) -> Result<AgentMemory> {
        Ok(self.memory.clone())
    }
    
    pub fn import_memory(&mut self, memory: AgentMemory) -> Result<()> {
        self.memory = memory;
        Ok(())
    }
    
    pub fn add_workflow(&mut self, workflow: Workflow) {
        self.memory.custom_workflows.push(workflow);
        self.memory.last_updated = Utc::now();
    }
    
    pub fn get_workflows(&self) -> &[Workflow] {
        &self.memory.custom_workflows
    }
    
    pub fn update_preferences(&mut self, key: String, value: serde_json::Value) {
        self.memory.user_preferences.insert(key, value);
        self.memory.last_updated = Utc::now();
    }
    
    pub fn get_preference(&self, key: &str) -> Option<&serde_json::Value> {
        self.memory.user_preferences.get(key)
    }
