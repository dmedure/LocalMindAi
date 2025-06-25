import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './App.css';

interface Agent {
  id: string;
  name: string;
  specialization: string;
  personality: string;
  instructions?: string;
  created_at: string; // ISO 8601 string
}

interface Message {
  id: string;
  content: string;
  sender: 'user' | 'agent';
  timestamp: string; // ISO 8601 string
  agent_id: string;
}

interface Document {
  id: string;
  name: string;
  doc_type: string;
  size: number;
  path: string;
  summary?: string;
  indexed_at: string; // ISO 8601 string
}

function App() {
  // State management
  const [currentView, setCurrentView] = useState<'welcome' | 'chat' | 'documents' | 'transfer' | 'system'>('welcome');
  const [agents, setAgents] = useState<Agent[]>([]);
  const [currentAgent, setCurrentAgent] = useState<Agent | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [documents, setDocuments] = useState<Document[]>([]);
  const [inputMessage, setInputMessage] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [showCreateAgentModal, setShowCreateAgentModal] = useState(false);
  const [showAgentDropdown, setShowAgentDropdown] = useState(false);

  // Agent creation form state
  const [newAgentName, setNewAgentName] = useState('');
  const [newAgentSpecialization, setNewAgentSpecialization] = useState('general');
  const [newAgentPersonality, setNewAgentPersonality] = useState('');
  const [newAgentInstructions, setNewAgentInstructions] = useState('');

  // Service status
  const [ollamaStatus, setOllamaStatus] = useState<'online' | 'offline' | 'unknown'>('unknown');
  const [chromaStatus, setChromaStatus] = useState<'online' | 'offline' | 'unknown'>('unknown');

  // Load agents and check service status on startup
  useEffect(() => {
    loadAgents();
    checkServiceStatus();
  }, []);

  // Load messages when agent changes
  useEffect(() => {
    if (currentAgent) {
      loadMessagesForAgent(currentAgent.id);
    }
  }, [currentAgent]);

  const loadAgents = async () => {
    try {
      const agentList = await invoke<Agent[]>('get_agents');
      setAgents(agentList);
      
      // If no agents exist, show welcome screen
      if (agentList.length === 0) {
        setCurrentView('welcome');
      } else if (!currentAgent) {
        // Select first agent if none selected
        setCurrentAgent(agentList[0]);
        setCurrentView('chat');
      }
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  const loadMessagesForAgent = async (agentId: string) => {
    try {
      const agentMessages = await invoke<Message[]>('get_agent_messages', { agentId });
      setMessages(agentMessages);
    } catch (error) {
      console.error('Failed to load messages:', error);
      setMessages([]);
    }
  };

  const checkServiceStatus = async () => {
    try {
      const status = await invoke<{ollama: boolean, chromadb: boolean}>('check_service_status');
      setOllamaStatus(status.ollama ? 'online' : 'offline');
      setChromaStatus(status.chromadb ? 'online' : 'offline');
    } catch (error) {
      console.error('Failed to check service status:', error);
    }
  };

  const createAgent = async () => {
    if (!newAgentName.trim()) {
      alert('Please enter an agent name');
      return;
    }

    try {
      const agent: Agent = {
        id: crypto.randomUUID(),
        name: newAgentName,
        specialization: newAgentSpecialization,
        personality: newAgentPersonality || 'friendly',
        instructions: newAgentInstructions,
        created_at: new Date().toISOString()
      };

      await invoke('create_agent', { agent });
      await loadAgents();
      
      // Select the new agent and switch to chat
      setCurrentAgent(agent);
      setCurrentView('chat');
      
      // Reset form and close modal
      setNewAgentName('');
      setNewAgentSpecialization('general');
      setNewAgentPersonality('');
      setNewAgentInstructions('');
      setShowCreateAgentModal(false);
    } catch (error) {
      console.error('Failed to create agent:', error);
      alert('Failed to create agent. Please try again.');
    }
  };

  const sendMessage = async () => {
    if (!inputMessage.trim() || !currentAgent || isLoading) return;

    const userMessage: Message = {
      id: crypto.randomUUID(),
      content: inputMessage.trim(),
      sender: 'user',
      timestamp: new Date().toISOString(),
      agent_id: currentAgent.id
    };

    setMessages(prev => [...prev, userMessage]);
    setInputMessage('');
    setIsLoading(true);

    try {
      const response = await invoke<string>('send_message_to_agent', {
        agentId: currentAgent.id,
        message: inputMessage.trim()
      });

      const agentMessage: Message = {
        id: crypto.randomUUID(),
        content: response,
        sender: 'agent',
        timestamp: new Date().toISOString(),
        agent_id: currentAgent.id
      };

      setMessages(prev => [...prev, agentMessage]);
    } catch (error) {
      console.error('Failed to send message:', error);
      const errorMessage: Message = {
        id: crypto.randomUUID(),
        content: 'Sorry, I encountered an error processing your message. Please try again.',
        sender: 'agent',
        timestamp: new Date().toISOString(),
        agent_id: currentAgent.id
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const selectAgent = (agent: Agent) => {
    setCurrentAgent(agent);
    setCurrentView('chat');
    setShowAgentDropdown(false);
  };

  const getAgentIntroduction = (agent: Agent) => {
    const introductions = {
      work: `I'm your professional assistant, specialized in handling work tasks, project management, and business communications. I have access to your work documents and can help you stay organized and productive.`,
      coding: `I'm your coding companion! I specialize in programming, code review, debugging, and technical documentation. I can help you with any development challenges you're facing.`,
      research: `I'm your research assistant, focused on academic work, data analysis, and in-depth investigation. I can help you find information, analyze data, and organize your research.`,
      writing: `I'm your writing partner! I specialize in content creation, editing, brainstorming, and helping you communicate effectively through written word.`,
      personal: `I'm your personal assistant, here to help with daily tasks, organization, scheduling, and anything else you need to manage your personal life.`,
      creative: `I'm your creative companion! I specialize in brainstorming, artistic projects, design thinking, and helping you explore your creative potential.`,
      technical: `I'm your technical support specialist, focused on troubleshooting, system administration, and helping you with technical challenges.`,
      general: `I'm your general assistant, ready to help with a wide variety of tasks and questions. I adapt to your needs and learn from our conversations.`
    };

    return introductions[agent.specialization as keyof typeof introductions] || introductions.general;
  };

  const formatTime = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div className="app">
      {/* Header */}
      <div className="app-header">
        <h1>🧠 LocalMind AI Agent</h1>
        <div className="header-controls">
          <div className="status-indicators">
            <div className={`status ${ollamaStatus}`}>
              Ollama: {ollamaStatus === 'online' ? 'Online' : 'Offline'}
            </div>
            <div className={`status ${chromaStatus}`}>
              ChromaDB: {chromaStatus === 'online' ? 'Connected' : 'Disconnected'}
            </div>
          </div>
          
          {agents.length > 0 && (
            <div className="agent-selector">
              <div className="agent-menu">
                <div 
                  className="current-agent" 
                  onClick={() => setShowAgentDropdown(!showAgentDropdown)}
                >
                  {currentAgent ? currentAgent.name : 'Select Agent'} ▼
                </div>
                {showAgentDropdown && (
                  <div className="agent-dropdown">
                    {agents.map(agent => (
                      <div 
                        key={agent.id}
                        className="agent-option"
                        onClick={() => selectAgent(agent)}
                      >
                        {getAgentIcon(agent.specialization)} {agent.name}
                        <span className="agent-spec">({agent.specialization})</span>
                      </div>
                    ))}
                    <hr />
                    <div 
                      className="agent-option"
                      onClick={() => setShowCreateAgentModal(true)}
                    >
                      ➕ Create New Agent
                    </div>
                  </div>
                )}
              </div>
              <button 
                className="create-agent-btn"
                onClick={() => setShowCreateAgentModal(true)}
              >
                + New Agent
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Navigation - only show if we have agents */}
      {agents.length > 0 && currentAgent && (
        <div className="tab-navigation">
          <button 
            className={currentView === 'chat' ? 'active' : ''}
            onClick={() => setCurrentView('chat')}
          >
            💬 Chat
          </button>
          <button 
            className={currentView === 'documents' ? 'active' : ''}
            onClick={() => setCurrentView('documents')}
          >
            📄 Documents
          </button>
          <button 
            className={currentView === 'transfer' ? 'active' : ''}
            onClick={() => setCurrentView('transfer')}
          >
            🔄 Transfer
          </button>
          <button 
            className={currentView === 'system' ? 'active' : ''}
            onClick={() => setCurrentView('system')}
          >
            ⚙️ System
          </button>
        </div>
      )}

      {/* Main Content */}
      <div className="main-content">
        {/* Welcome Screen */}
        {currentView === 'welcome' && (
          <div className="welcome-screen">
            <h2>Welcome to LocalMind!</h2>
            <p>
              Create your first AI agent to get started. Each agent can have its own personality, 
              specialization, and knowledge base - all running privately on your device.
            </p>
            <button 
              className="get-started-btn"
              onClick={() => setShowCreateAgentModal(true)}
            >
              Create Your First Agent
            </button>
            
            <div className="agent-ideas">
              <h3>Agent Ideas:</h3>
              <div className="ideas-grid">
                <div>
                  <strong>💼 Work Assistant</strong>
                  <p>Handle emails, meetings, project management</p>
                </div>
                <div>
                  <strong>💻 Coding Buddy</strong>
                  <p>Code review, debugging, documentation</p>
                </div>
                <div>
                  <strong>🔬 Research Helper</strong>
                  <p>Academic research, data analysis</p>
                </div>
                <div>
                  <strong>📝 Writing Partner</strong>
                  <p>Content creation, editing, brainstorming</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Chat Interface */}
        {currentView === 'chat' && currentAgent && (
          <div className="chat-container">
            <div className="agent-intro">
              <h3>👋 Hello! I'm {currentAgent.name}</h3>
              <p>{getAgentIntroduction(currentAgent)}</p>
              {currentAgent.instructions && (
                <div className="custom-instructions">
                  <strong>Special Focus:</strong> {currentAgent.instructions}
                </div>
              )}
            </div>

            <div className="messages">
              {messages.map((message) => (
                <div key={message.id} className={`message ${message.sender}`}>
                  <div className="message-content">
                    {message.content}
                  </div>
                  <div className="message-time">
                    {formatTime(message.timestamp)}
                  </div>
                </div>
              ))}
              {isLoading && (
                <div className="message agent">
                  <div className="message-content">
                    <div className="typing-indicator">
                      <span></span>
                      <span></span>
                      <span></span>
                    </div>
                  </div>
                </div>
              )}
            </div>

            <div className="input-container">
              <textarea
                value={inputMessage}
                onChange={(e) => setInputMessage(e.target.value)}
                placeholder={`Ask ${currentAgent.name} anything...`}
                rows={2}
                onKeyPress={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    sendMessage();
                  }
                }}
              />
              <button 
                onClick={sendMessage} 
                disabled={!inputMessage.trim() || isLoading}
              >
                Send
              </button>
            </div>
          </div>
        )}

        {/* Documents Tab */}
        {currentView === 'documents' && (
          <div className="documents-container">
            <div className="documents-header">
              <button onClick={() => invoke('add_document')}>
                📁 Add Document
              </button>
              <input
                className="search-input"
                placeholder="Search your indexed documents..."
              />
            </div>
            <div className="documents-list">
              {documents.length === 0 ? (
                <div className="empty-state">
                  <p>No documents indexed yet.</p>
                  <p>Add some documents to get started!</p>
                </div>
              ) : (
                documents.map((doc) => (
                  <div key={doc.id} className="document-card">
                    <div className="doc-header">
                      <h3>{doc.name}</h3>
                      <div className="doc-type">{doc.doc_type}</div>
                    </div>
                    <div className="doc-details">
                      <p><strong>Size:</strong> {formatFileSize(doc.size)}</p>
                      <p><strong>Indexed:</strong> {formatTime(doc.indexed_at)}</p>
                      {doc.summary && (
                        <div className="doc-summary">
                          {doc.summary}
                        </div>
                      )}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {/* Transfer Tab */}
        {currentView === 'transfer' && (
          <div className="transfer-container">
            <div className="transfer-section">
              <h2>📤 Export Agent Knowledge</h2>
              <p>Export your agent's knowledge for backup or sharing</p>
              <div className="transfer-actions">
                <button className="primary-action">
                  Export {currentAgent?.name} Knowledge
                </button>
              </div>
            </div>

            <div className="transfer-section">
              <h2>📥 Import Knowledge</h2>
              <div className="import-strategies">
                <button>Smart Merge</button>
                <button>Append Only</button>
                <button className="warning-action">Replace All</button>
              </div>
              <div className="strategy-descriptions">
                <small>
                  Smart Merge: Intelligently combines knowledge without conflicts<br/>
                  Append Only: Adds new knowledge without overwriting existing<br/>
                  Replace All: Completely replaces current knowledge
                </small>
              </div>
            </div>

            <div className="transfer-section">
              <h2>🎯 Create Specialized Agent</h2>
              <div className="specialized-creation">
                <input
                  className="domain-input"
                  placeholder="e.g., coding, research, writing"
                />
                <button className="primary-action">Create Specialized Agent</button>
              </div>
              <p className="help-text">
                Create a new agent specialized in a specific domain from your current agent's knowledge
              </p>
            </div>
          </div>
        )}

        {/* System Tab */}
        {currentView === 'system' && (
          <div className="system-container">
            <div className="system-section">
              <h2>📊 Agent Statistics</h2>
              <div className="stats-grid">
                <div className="stat-card">
                  <div className="stat-number">{agents.length}</div>
                  <div className="stat-label">Active Agents</div>
                </div>
                <div className="stat-card">
                  <div className="stat-number">{messages.length}</div>
                  <div className="stat-label">Total Messages</div>
                </div>
                <div className="stat-card">
                  <div className="stat-number">{documents.length}</div>
                  <div className="stat-label">Documents</div>
                </div>
                <div className="stat-card">
                  <div className="stat-number">0</div>
                  <div className="stat-label">Knowledge Transfers</div>
                </div>
              </div>
            </div>

            <div className="system-section">
              <h2>🔧 Service Status</h2>
              <div className="service-status">
                <div className="service-item">
                  <span>Ollama Service</span>
                  <span className={ollamaStatus === 'online' ? 'online' : 'offline'}>
                    {ollamaStatus === 'online' ? '🟢 Online' : '🔴 Offline'}
                  </span>
                </div>
                <div className="service-item">
                  <span>ChromaDB</span>
                  <span className={chromaStatus === 'online' ? 'online' : 'offline'}>
                    {chromaStatus === 'online' ? '🟢 Connected' : '🔴 Disconnected'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Agent Creation Modal */}
      {showCreateAgentModal && (
        <div className="modal-overlay" onClick={() => setShowCreateAgentModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>🤖 Create New AI Agent</h3>
            
            <div className="form-group">
              <label>Agent Name</label>
              <input
                type="text"
                value={newAgentName}
                onChange={(e) => setNewAgentName(e.target.value)}
                placeholder="e.g., WorkBot, CodeMaster, ResearchPal"
              />
            </div>

            <div className="form-group">
              <label>Specialization</label>
              <select
                value={newAgentSpecialization}
                onChange={(e) => setNewAgentSpecialization(e.target.value)}
              >
                <option value="general">General Assistant</option>
                <option value="work">Professional/Business</option>
                <option value="coding">Programming/Development</option>
                <option value="research">Research/Academic</option>
                <option value="writing">Writing/Content</option>
                <option value="personal">Personal Assistant</option>
                <option value="creative">Creative/Design</option>
                <option value="technical">Technical Support</option>
              </select>
            </div>

            <div className="form-group">
              <label>Personality Style</label>
              <div className="personality-options">
                {['professional', 'friendly', 'analytical', 'creative', 'concise', 'detailed'].map(personality => (
                  <div
                    key={personality}
                    className={`personality-option ${newAgentPersonality === personality ? 'selected' : ''}`}
                    onClick={() => setNewAgentPersonality(personality)}
                  >
                    {getPersonalityIcon(personality)} {personality.charAt(0).toUpperCase() + personality.slice(1)}
                  </div>
                ))}
              </div>
            </div>

            <div className="form-group">
              <label>Custom Instructions (Optional)</label>
              <textarea
                value={newAgentInstructions}
                onChange={(e) => setNewAgentInstructions(e.target.value)}
                placeholder="Any specific instructions for how this agent should behave..."
              />
            </div>

            <div className="modal-actions">
              <button 
                className="btn-secondary"
                onClick={() => setShowCreateAgentModal(false)}
              >
                Cancel
              </button>
              <button 
                className="btn-primary"
                onClick={createAgent}
              >
                Create Agent
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function getAgentIcon(specialization: string): string {
  const icons = {
    work: '💼',
    coding: '💻',
    research: '🔬',
    writing: '📝',
    personal: '👤',
    creative: '🎨',
    technical: '🔧',
    general: '🤖'
  };
  return icons[specialization as keyof typeof icons] || '🤖';
}

function getPersonalityIcon(personality: string): string {
  const icons = {
    professional: '💼',
    friendly: '😊',
    analytical: '🔍',
    creative: '🎨',
    concise: '⚡',
    detailed: '📝'
  };
  return icons[personality as keyof typeof icons] || '😊';
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export default App;