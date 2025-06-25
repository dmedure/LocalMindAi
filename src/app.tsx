import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './App.css';

interface Agent {
  id: string;
  name: string;
  specialization: string;
  personality: string;
  instructions: string;
  created_at: string;
}

interface Message {
  id: string;
  agent_id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
}

interface SystemStatus {
  ollama: 'online' | 'offline' | 'unknown';
  chromadb: 'online' | 'offline' | 'unknown';
}

function App() {
  // Core state
  const [currentTab, setCurrentTab] = useState('chat');
  const [agents, setAgents] = useState<Agent[]>([]);
  const [currentAgent, setCurrentAgent] = useState<Agent | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputMessage, setInputMessage] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isFirstTime, setIsFirstTime] = useState(true);
  const [showAgentModal, setShowAgentModal] = useState(false);
  const [isTyping, setIsTyping] = useState(false);
  const [systemStatus, setSystemStatus] = useState<SystemStatus>({
    ollama: 'unknown',
    chromadb: 'unknown'
  });

  // Agent switching state
  const [isSwitchingAgent, setIsSwitchingAgent] = useState(false);
  const [switchError, setSwitchError] = useState<string | null>(null);

  // Agent creation state
  const [newAgent, setNewAgent] = useState({
    name: '',
    specialization: 'general',
    personality: 'helpful',
    instructions: ''
  });

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Scroll to bottom of messages
  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // Initialize app
  useEffect(() => {
    initializeApp();
    checkSystemStatus();
  }, []);

  const initializeApp = async () => {
    try {
      setIsLoading(true);
      const existingAgents = await invoke<Agent[]>('get_agents');
      
      if (existingAgents && existingAgents.length > 0) {
        setAgents(existingAgents);
        setIsFirstTime(false);
        // Load the first agent by default
        await switchToAgent(existingAgents[0], false);
      } else {
        setIsFirstTime(true);
      }
    } catch (error) {
      console.error('Failed to initialize app:', error);
      setIsFirstTime(true);
    } finally {
      setIsLoading(false);
    }
  };

  const checkSystemStatus = async () => {
    try {
      const status = await invoke<SystemStatus>('check_system_status');
      setSystemStatus(status);
    } catch (error) {
      console.error('Failed to check system status:', error);
      setSystemStatus({ ollama: 'offline', chromadb: 'offline' });
    }
  };

  // FIXED: Improved agent switching with proper error handling and state management
  const switchToAgent = async (agent: Agent, showLoading = true) => {
    if (isSwitchingAgent || currentAgent?.id === agent.id) {
      return; // Prevent double-switching or switching to same agent
    }

    try {
      if (showLoading) {
        setIsSwitchingAgent(true);
      }
      setSwitchError(null);
      
      // Clear current state first
      setMessages([]);
      setInputMessage('');
      setIsTyping(false);

      console.log(`Switching to agent: ${agent.name} (${agent.id})`);
      
      // Set current agent immediately to update UI
      setCurrentAgent(agent);
      
      // Load agent's message history
      const agentMessages = await invoke<Message[]>('get_agent_messages', {
        agentId: agent.id
      });
      
      console.log(`Loaded ${agentMessages.length} messages for agent ${agent.name}`);
      setMessages(agentMessages || []);
      
    } catch (error) {
      console.error('Failed to switch agent:', error);
      setSwitchError(`Failed to switch to ${agent.name}. Please try again.`);
      
      // Reset to a known good state
      setMessages([]);
      setCurrentAgent(null);
    } finally {
      setIsSwitchingAgent(false);
    }
  };

  const createAgent = async () => {
    if (!newAgent.name.trim()) {
      alert('Please enter an agent name');
      return;
    }

    try {
      setIsLoading(true);
      
      const agent = await invoke<Agent>('create_agent', {
        name: newAgent.name.trim(),
        specialization: newAgent.specialization,
        personality: newAgent.personality,
        instructions: newAgent.instructions.trim() || `I am ${newAgent.name}, a ${newAgent.personality} AI assistant specializing in ${newAgent.specialization}. How can I help you today?`
      });

      // Update agents list
      const updatedAgents = [...agents, agent];
      setAgents(updatedAgents);
      setIsFirstTime(false);
      
      // Switch to new agent
      await switchToAgent(agent);
      
      // Close modal and reset form
      setShowAgentModal(false);
      setNewAgent({
        name: '',
        specialization: 'general',
        personality: 'helpful',
        instructions: ''
      });

      console.log('Created and switched to new agent:', agent.name);
    } catch (error) {
      console.error('Failed to create agent:', error);
      alert('Failed to create agent. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const sendMessage = async () => {
    if (!inputMessage.trim() || isLoading || !currentAgent) return;

    const userMessage = inputMessage.trim();
    setInputMessage('');
    setIsLoading(true);
    setIsTyping(true);

    // Add user message to UI immediately
    const newUserMessage: Message = {
      id: `user-${Date.now()}`,
      agent_id: currentAgent.id,
      role: 'user',
      content: userMessage,
      timestamp: new Date().toISOString(),
    };

    setMessages(prev => [...prev, newUserMessage]);

    try {
      const response = await invoke<string>('send_message', {
        agentId: currentAgent.id,
        message: userMessage,
      });

      // Add assistant response
      const assistantMessage: Message = {
        id: `assistant-${Date.now()}`,
        agent_id: currentAgent.id,
        role: 'assistant',
        content: response,
        timestamp: new Date().toISOString(),
      };

      setMessages(prev => [...prev, assistantMessage]);
    } catch (error) {
      console.error('Failed to send message:', error);
      
      // Add error message
      const errorMessage: Message = {
        id: `error-${Date.now()}`,
        agent_id: currentAgent.id,
        role: 'assistant',
        content: 'Sorry, I encountered an error. Please check that Ollama is running and try again.',
        timestamp: new Date().toISOString(),
      };

      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
      setIsTyping(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  // FIXED: Better error display and loading states
  const renderAgentSelector = () => {
    if (agents.length === 0) return null;

    return (
      <div className="agent-selector">
        <select 
          value={currentAgent?.id || ''} 
          onChange={(e) => {
            const selectedAgent = agents.find(a => a.id === e.target.value);
            if (selectedAgent) {
              switchToAgent(selectedAgent);
            }
          }}
          disabled={isSwitchingAgent || isLoading}
          className="agent-dropdown"
        >
          {!currentAgent && <option value="">Select an agent...</option>}
          {agents.map(agent => (
            <option key={agent.id} value={agent.id}>
              {agent.name} ({agent.specialization})
            </option>
          ))}
        </select>
        
        <button 
          onClick={() => setShowAgentModal(true)}
          className="create-agent-btn"
          disabled={isSwitchingAgent || isLoading}
        >
          + New Agent
        </button>
        
        {/* Show switching status */}
        {isSwitchingAgent && (
          <span className="switching-status">Switching agents...</span>
        )}
        
        {/* Show switch error */}
        {switchError && (
          <span className="switch-error" onClick={() => setSwitchError(null)}>
            {switchError} ✕
          </span>
        )}
      </div>
    );
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online': return '#22c55e';
      case 'offline': return '#ef4444';
      default: return '#a855f7';
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'online': return 'Online';
      case 'offline': return 'Offline';
      default: return 'Unknown';
    }
  };

  // Welcome screen for first-time users
  if (isFirstTime && !isLoading) {
    return (
      <div className="app">
        <div className="welcome-screen">
          <div className="welcome-content">
            <h1>Welcome to LocalMind!</h1>
            <p>Create your first AI agent to get started. Each agent can have its own personality and specialization.</p>
            <button 
              onClick={() => setShowAgentModal(true)}
              className="primary-action"
            >
              Create Your First Agent
            </button>
          </div>
        </div>
        {renderAgentCreationModal()}
      </div>
    );
  }

  // FIXED: Better loading state handling
  if (isLoading && !currentAgent) {
    return (
      <div className="app">
        <div className="loading-screen">
          <div className="loading-content">
            <div className="loading-spinner"></div>
            <h2>Loading LocalMind...</h2>
            <p>Setting up your AI agents...</p>
          </div>
        </div>
      </div>
    );
  }

  // Agent creation modal
  function renderAgentCreationModal() {
    if (!showAgentModal) return null;

    return (
      <div className="modal-overlay" onClick={() => setShowAgentModal(false)}>
        <div className="modal-content" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h2>Create New Agent</h2>
            <button 
              onClick={() => setShowAgentModal(false)}
              className="modal-close"
            >
              ✕
            </button>
          </div>
          
          <div className="modal-body">
            <div className="form-group">
              <label>Agent Name</label>
              <input
                type="text"
                value={newAgent.name}
                onChange={(e) => setNewAgent({...newAgent, name: e.target.value})}
                placeholder="e.g., WorkBot, CodeMaster, ResearchPal"
                className="form-input"
              />
            </div>

            <div className="form-group">
              <label>Specialization</label>
              <select
                value={newAgent.specialization}
                onChange={(e) => setNewAgent({...newAgent, specialization: e.target.value})}
                className="form-select"
              >
                <option value="general">General Assistant</option>
                <option value="professional">Professional/Business</option>
                <option value="coding">Programming & Development</option>
                <option value="research">Research & Analysis</option>
                <option value="writing">Writing & Content</option>
                <option value="creative">Creative & Design</option>
                <option value="education">Education & Learning</option>
                <option value="health">Health & Wellness</option>
              </select>
            </div>

            <div className="form-group">
              <label>Personality</label>
              <select
                value={newAgent.personality}
                onChange={(e) => setNewAgent({...newAgent, personality: e.target.value})}
                className="form-select"
              >
                <option value="helpful">Helpful & Supportive</option>
                <option value="professional">Professional & Formal</option>
                <option value="friendly">Friendly & Casual</option>
                <option value="analytical">Analytical & Precise</option>
                <option value="creative">Creative & Imaginative</option>
                <option value="enthusiastic">Enthusiastic & Energetic</option>
              </select>
            </div>

            <div className="form-group">
              <label>Custom Instructions (Optional)</label>
              <textarea
                value={newAgent.instructions}
                onChange={(e) => setNewAgent({...newAgent, instructions: e.target.value})}
                placeholder="Any specific instructions for how this agent should behave..."
                className="form-textarea"
                rows={3}
              />
            </div>
          </div>

          <div className="modal-footer">
            <button 
              onClick={() => setShowAgentModal(false)}
              className="secondary-action"
              disabled={isLoading}
            >
              Cancel
            </button>
            <button 
              onClick={createAgent}
              className="primary-action"
              disabled={isLoading || !newAgent.name.trim()}
            >
              {isLoading ? 'Creating...' : 'Create Agent'}
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      {/* Header */}
      <header className="app-header">
        <div className="header-left">
          <h1>LocalMind AI Agent</h1>
          {renderAgentSelector()}
        </div>
        
        <div className="status-indicators">
          <div 
            className={`status ${systemStatus.ollama}`}
            style={{ '--status-color': getStatusColor(systemStatus.ollama) } as React.CSSProperties}
          >
            Ollama: {getStatusText(systemStatus.ollama)}
          </div>
          <div 
            className={`status ${systemStatus.chromadb}`}
            style={{ '--status-color': getStatusColor(systemStatus.chromadb) } as React.CSSProperties}
          >
            ChromaDB: {getStatusText(systemStatus.chromadb)}
          </div>
        </div>
      </header>

      {/* Navigation */}
      <nav className="tab-navigation">
        <button 
          className={currentTab === 'chat' ? 'active' : ''}
          onClick={() => setCurrentTab('chat')}
        >
          💬 Chat
        </button>
        <button 
          className={currentTab === 'documents' ? 'active' : ''}
          onClick={() => setCurrentTab('documents')}
        >
          📄 Documents
        </button>
        <button 
          className={currentTab === 'transfer' ? 'active' : ''}
          onClick={() => setCurrentTab('transfer')}
        >
          🔄 Transfer
        </button>
        <button 
          className={currentTab === 'system' ? 'active' : ''}
          onClick={() => setCurrentTab('system')}
        >
          ⚙️ System
        </button>
      </nav>

      {/* Main Content */}
      <main className="main-content">
        {currentTab === 'chat' && (
          <div className="chat-container">
            {/* FIXED: Better loading and error states */}
            {isSwitchingAgent ? (
              <div className="switching-overlay">
                <div className="switching-content">
                  <div className="loading-spinner"></div>
                  <h3>Switching to {currentAgent?.name || 'agent'}...</h3>
                  <p>Loading conversation history...</p>
                </div>
              </div>
            ) : (
              <>
                <div className="messages">
                  {currentAgent && messages.length === 0 && !isLoading && (
                    <div className="agent-intro">
                      <div className="intro-avatar">🤖</div>
                      <div className="intro-content">
                        <h3>Hello! I'm {currentAgent.name}</h3>
                        <p>I'm your {currentAgent.personality} AI assistant specializing in {currentAgent.specialization}.</p>
                        <p>How can I help you today?</p>
                      </div>
                    </div>
                  )}

                  {messages.map((message) => (
                    <div key={message.id} className={`message ${message.role}`}>
                      <div className="message-content">
                        {message.content}
                      </div>
                      <div className="message-time">
                        {new Date(message.timestamp).toLocaleTimeString()}
                      </div>
                    </div>
                  ))}

                  {isTyping && (
                    <div className="message assistant">
                      <div className="message-content">
                        <div className="typing-indicator">
                          <span></span>
                          <span></span>
                          <span></span>
                        </div>
                      </div>
                    </div>
                  )}
                  
                  <div ref={messagesEndRef} />
                </div>

                <div className="input-container">
                  <textarea
                    ref={inputRef}
                    value={inputMessage}
                    onChange={(e) => setInputMessage(e.target.value)}
                    onKeyPress={handleKeyPress}
                    placeholder={currentAgent ? `Message ${currentAgent.name}...` : "Select an agent to start chatting..."}
                    className="message-input"
                    rows={1}
                    disabled={!currentAgent || isLoading}
                  />
                  <button
                    onClick={sendMessage}
                    disabled={!inputMessage.trim() || isLoading || !currentAgent}
                    className="send-button"
                  >
                    Send
                  </button>
                </div>
              </>
            )}
          </div>
        )}

        {currentTab === 'documents' && (
          <div className="documents-container">
            <div className="coming-soon">
              <h2>📄 Document Management</h2>
              <p>Document indexing and search features coming soon!</p>
            </div>
          </div>
        )}

        {currentTab === 'transfer' && (
          <div className="transfer-container">
            <div className="coming-soon">
              <h2>🔄 Knowledge Transfer</h2>
              <p>Agent knowledge export/import features coming soon!</p>
            </div>
          </div>
        )}

        {currentTab === 'system' && (
          <div className="system-container">
            <div className="system-section">
              <h2>⚙️ System Status</h2>
              <div className="system-info">
                <div className="info-row">
                  <span>Ollama Service</span>
                  <span className={`status ${systemStatus.ollama}`}>
                    {getStatusText(systemStatus.ollama)}
                  </span>
                </div>
                <div className="info-row">
                  <span>ChromaDB Service</span>
                  <span className={`status ${systemStatus.chromadb}`}>
                    {getStatusText(systemStatus.chromadb)}
                  </span>
                </div>
                <div className="info-row">
                  <span>Total Agents</span>
                  <span>{agents.length}</span>
                </div>
                <div className="info-row">
                  <span>Current Agent</span>
                  <span>{currentAgent?.name || 'None'}</span>
                </div>
              </div>
              
              <button 
                onClick={checkSystemStatus}
                className="primary-action"
                style={{ marginTop: '1rem' }}
              >
                Refresh Status
              </button>
            </div>
          </div>
        )}
      </main>

      {/* Agent Creation Modal */}
      {renderAgentCreationModal()}
    </div>
  );
}

export default App;