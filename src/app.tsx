import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { listen } from '@tauri-apps/api/event';
import './app.css';

interface ChatMessage {
  id: string;
  content: string;
  role: 'user' | 'assistant';
  timestamp: string;
}

interface ChatResponse {
  message: ChatMessage;
  sources: string[];
}

interface Document {
  id: string;
  content: string;
  source: string;
  metadata: {
    title: string;
    file_type: string;
    size: number;
    created_at: string;
    keywords: string[];
    summary: string;
  };
}

interface SystemInfo {
  os: string;
  version: string;
  total_memory: number;
  available_memory: number;
  cpu_count: number;
  cpu_brand: string;
}

interface AgentStats {
  documents_count: number;
  total_conversations: number;
  knowledge_categories: string[];
  last_updated: string;
  storage_size: number;
}

interface TransferStats {
  documents_transferred: number;
  conversations_transferred: number;
  workflows_transferred: number;
  preferences_transferred: number;
  transfer_time_ms: number;
}

function App() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [ollamaStatus, setOllamaStatus] = useState<boolean | null>(null);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [documents, setDocuments] = useState<Document[]>([]);
  const [agentStats, setAgentStats] = useState<AgentStats | null>(null);
  const [activeTab, setActiveTab] = useState<'chat' | 'documents' | 'transfer' | 'system'>('chat');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    checkOllamaStatus();
    getSystemInfo();
    getAgentStats();
    
    // Add welcome message
    const welcomeMessage: ChatMessage = {
      id: 'welcome',
      content: 'Hello! I\'m your local AI assistant. I can help you with documents, answer questions, and assist with various tasks. How can I help you today?',
      role: 'assistant',
      timestamp: new Date().toISOString(),
    };
    setMessages([welcomeMessage]);
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const checkOllamaStatus = async () => {
    try {
      const status = await invoke<boolean>('check_ollama_status');
      setOllamaStatus(status);
    } catch (error) {
      console.error('Failed to check Ollama status:', error);
      setOllamaStatus(false);
    }
  };

  const getAgentStats = async () => {
    try {
      const stats = await invoke<AgentStats>('get_agent_stats');
      setAgentStats(stats);
    } catch (error) {
      console.error('Failed to get agent stats:', error);
    }
  };

  const exportKnowledge = async (categories: string[], encrypt: boolean) => {
    try {
      const exportPath = await open({
        defaultPath: 'knowledge_export.json',
        filters: [{
          name: 'JSON Files',
          extensions: ['json']
        }]
      });

      if (exportPath && typeof exportPath === 'string') {
        setIsLoading(true);
        const result = await invoke<string>('export_agent_knowledge', {
          categories,
          exportPath,
          encrypt,
        });
        
        alert(result);
        await getAgentStats(); // Refresh stats
      }
    } catch (error) {
      console.error('Failed to export knowledge:', error);
      alert(`Failed to export knowledge: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const importKnowledge = async (mergeStrategy: string) => {
    try {
      const importPath = await open({
        filters: [{
          name: 'JSON Files',
          extensions: ['json']
        }]
      });

      if (importPath && typeof importPath === 'string') {
        setIsLoading(true);
        const result = await invoke<string>('import_agent_knowledge', {
          importPath,
          mergeStrategy,
        });
        
        alert(result);
        await getAgentStats(); // Refresh stats
        await searchDocuments(''); // Refresh documents
      }
    } catch (error) {
      console.error('Failed to import knowledge:', error);
      alert(`Failed to import knowledge: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const exportCompleteAgent = async () => {
    try {
      const exportPath = await open({
        defaultPath: 'complete_agent_export.json',
        filters: [{
          name: 'JSON Files',
          extensions: ['json']
        }]
      });

      if (exportPath && typeof exportPath === 'string') {
        setIsLoading(true);
        const result = await invoke<string>('export_complete_agent', {
          exportPath,
        });
        
        alert(result);
      }
    } catch (error) {
      console.error('Failed to export complete agent:', error);
      alert(`Failed to export complete agent: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const createSpecializedAgent = async (domain: string) => {
    try {
      const sourceExportPath = await open({
        filters: [{
          name: 'JSON Files',
          extensions: ['json']
        }]
      });

      if (!sourceExportPath || typeof sourceExportPath !== 'string') return;

      const targetPath = await open({
        defaultPath: `${domain}_agent.json`,
        filters: [{
          name: 'JSON Files',
          extensions: ['json']
        }]
      });

      if (targetPath && typeof targetPath === 'string') {
        setIsLoading(true);
        const result = await invoke<string>('create_specialized_agent', {
          domain,
          sourceExportPath,
          targetPath,
        });
        
        alert(result);
      }
    } catch (error) {
      console.error('Failed to create specialized agent:', error);
      alert(`Failed to create specialized agent: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const getSystemInfo = async () => {
    try {
      const info = await invoke<SystemInfo>('get_system_info');
      setSystemInfo(info);
    } catch (error) {
      console.error('Failed to get system info:', error);
    }
  };

  const handleSendMessage = async () => {
    if (!input.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      content: input,
      role: 'user',
      timestamp: new Date().toISOString(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);

    try {
      const response = await invoke<ChatResponse>('send_message', {
        content: input,
      });

      setMessages(prev => [...prev, response.message]);
    } catch (error) {
      console.error('Failed to send message:', error);
      const errorMessage: ChatMessage = {
        id: Date.now().toString(),
        content: `Sorry, I encountered an error: ${error}`,
        role: 'assistant',
        timestamp: new Date().toISOString(),
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  const handleIndexDocument = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Documents',
          extensions: ['txt', 'md', 'pdf', 'docx', 'doc']
        }]
      });

      if (selected && typeof selected === 'string') {
        setIsLoading(true);
        const result = await invoke<string>('index_document', {
          filePath: selected,
        });
        
        alert(result);
        
        // Refresh documents list
        await searchDocuments('');
      }
    } catch (error) {
      console.error('Failed to index document:', error);
      alert(`Failed to index document: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const searchDocuments = async (query: string) => {
    try {
      const results = await invoke<Document[]>('search_documents', {
        query,
        limit: 20,
      });
      setDocuments(results);
    } catch (error) {
      console.error('Failed to search documents:', error);
    }
  };

  const formatFileSize = (bytes: number) => {
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    if (bytes === 0) return '0 Bytes';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  };

  const formatMemory = (bytes: number) => {
    return Math.round(bytes / 1024 / 1024 / 1024 * 100) / 100 + ' GB';
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>🤖 Local AI Agent</h1>
        <div className="status-indicators">
          <div className={`status ${ollamaStatus ? 'online' : 'offline'}`}>
            Ollama: {ollamaStatus ? 'Online' : 'Offline'}
          </div>
        </div>
      </header>

      <nav className="tab-navigation">
        <button 
          className={activeTab === 'chat' ? 'active' : ''}
          onClick={() => setActiveTab('chat')}
        >
          💬 Chat
        </button>
        <button 
          className={activeTab === 'documents' ? 'active' : ''}
          onClick={() => setActiveTab('documents')}
        >
          📄 Documents
        </button>
        <button 
          className={activeTab === 'transfer' ? 'active' : ''}
          onClick={() => setActiveTab('transfer')}
        >
          🔄 Transfer
        </button>
        <button 
          className={activeTab === 'system' ? 'active' : ''}
          onClick={() => setActiveTab('system')}
        >
          ⚙️ System
        </button>
      </nav>

      <main className="main-content">
        {activeTab === 'chat' && (
          <div className="chat-container">
            <div className="messages">
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
              {isLoading && (
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
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder="Type your message here..."
                disabled={isLoading || !ollamaStatus}
                rows={3}
              />
              <button 
                onClick={handleSendMessage}
                disabled={isLoading || !ollamaStatus || !input.trim()}
              >
                Send
              </button>
            </div>
          </div>
        )}

        {activeTab === 'documents' && (
          <div className="documents-container">
            <div className="documents-header">
              <button onClick={handleIndexDocument} disabled={isLoading}>
                📁 Add Document
              </button>
              <input
                type="text"
                placeholder="Search documents..."
                onChange={(e) => searchDocuments(e.target.value)}
                className="search-input"
              />
            </div>
            
            <div className="documents-list">
              {documents.length === 0 ? (
                <div className="empty-state">
                  <p>No documents indexed yet.</p>
                  <p>Click "Add Document" to get started!</p>
                </div>
              ) : (
                documents.map((doc) => (
                  <div key={doc.id} className="document-card">
                    <div className="doc-header">
                      <h3>{doc.metadata.title}</h3>
                      <span className="doc-type">{doc.metadata.file_type}</span>
                    </div>
                    <div className="doc-details">
                      <p>Size: {formatFileSize(doc.metadata.size)}</p>
                      <p>Source: {doc.source}</p>
                      {doc.metadata.summary && (
                        <p className="doc-summary">{doc.metadata.summary}</p>
                      )}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {activeTab === 'transfer' && (
          <div className="transfer-container">
            <div className="transfer-section">
              <h2>📤 Export Knowledge</h2>
              <div className="transfer-actions">
                <div className="action-group">
                  <h3>Export by Category</h3>
                  <div className="category-export">
                    <div className="category-tags">
                      {agentStats?.knowledge_categories.map(category => (
                        <label key={category} className="category-tag">
                          <input type="checkbox" value={category} />
                          <span>{category}</span>
                        </label>
                      ))}
                    </div>
                    <div className="export-options">
                      <label>
                        <input type="checkbox" />
                        Encrypt Export
                      </label>
                    </div>
                    <button 
                      onClick={() => {
                        const selectedCategories = Array.from(
                          document.querySelectorAll('.category-tag input:checked')
                        ).map(input => (input as HTMLInputElement).value);
                        const encrypt = (document.querySelector('.export-options input') as HTMLInputElement)?.checked || false;
                        exportKnowledge(selectedCategories, encrypt);
                      }}
                      disabled={isLoading}
                    >
                      Export Selected Categories
                    </button>
                  </div>
                </div>
                
                <div className="action-group">
                  <h3>Complete Agent Export</h3>
                  <p>Export everything: documents, conversations, preferences, and workflows</p>
                  <button 
                    onClick={exportCompleteAgent}
                    disabled={isLoading}
                    className="primary-action"
                  >
                    Export Complete Agent
                  </button>
                </div>
              </div>
            </div>

            <div className="transfer-section">
              <h2>📥 Import Knowledge</h2>
              <div className="transfer-actions">
                <div className="action-group">
                  <h3>Import Strategy</h3>
                  <div className="import-strategies">
                    <button 
                      onClick={() => importKnowledge('merge')}
                      disabled={isLoading}
                    >
                      Smart Merge
                    </button>
                    <button 
                      onClick={() => importKnowledge('append')}
                      disabled={isLoading}
                    >
                      Add New Only
                    </button>
                    <button 
                      onClick={() => importKnowledge('replace')}
                      disabled={isLoading}
                      className="warning-action"
                    >
                      Replace All
                    </button>
                  </div>
                  <div className="strategy-descriptions">
                    <small>
                      <strong>Smart Merge:</strong> Updates existing, adds new<br/>
                      <strong>Add New Only:</strong> Appends without checking duplicates<br/>
                      <strong>Replace All:</strong> ⚠️ Completely replaces current knowledge
                    </small>
                  </div>
                </div>
              </div>
            </div>

            <div className="transfer-section">
              <h2>🎯 Specialized Agents</h2>
              <div className="transfer-actions">
                <div className="action-group">
                  <h3>Create Domain-Specific Agent</h3>
                  <div className="specialized-creation">
                    <input 
                      type="text" 
                      placeholder="Enter domain (e.g., 'coding', 'writing', 'research')"
                      id="domain-input"
                      className="domain-input"
                    />
                    <button 
                      onClick={() => {
                        const domain = (document.getElementById('domain-input') as HTMLInputElement)?.value;
                        if (domain) {
                          createSpecializedAgent(domain);
                        }
                      }}
                      disabled={isLoading}
                    >
                      Create Specialized Agent
                    </button>
                  </div>
                  <p className="help-text">
                    Creates a focused agent with only knowledge relevant to the specified domain
                  </p>
                </div>
              </div>
            </div>

            <div className="transfer-section">
              <h2>📊 Agent Statistics</h2>
              {agentStats ? (
                <div className="stats-grid">
                  <div className="stat-card">
                    <div className="stat-number">{agentStats.documents_count}</div>
                    <div className="stat-label">Documents</div>
                  </div>
                  <div className="stat-card">
                    <div className="stat-number">{agentStats.total_conversations}</div>
                    <div className="stat-label">Conversations</div>
                  </div>
                  <div className="stat-card">
                    <div className="stat-number">{agentStats.knowledge_categories.length}</div>
                    <div className="stat-label">Categories</div>
                  </div>
                  <div className="stat-card">
                    <div className="stat-number">{formatFileSize(agentStats.storage_size)}</div>
                    <div className="stat-label">Storage Used</div>
                  </div>
                </div>
              ) : (
                <p>Loading agent statistics...</p>
              )}
            </div>
          </div>
        )}

        {activeTab === 'system' && (
          <div className="system-container">
            <div className="system-section">
              <h2>🖥️ System Information</h2>
              {systemInfo ? (
                <div className="system-info">
                  <div className="info-row">
                    <span>Operating System:</span>
                    <span>{systemInfo.os} {systemInfo.version}</span>
                  </div>
                  <div className="info-row">
                    <span>CPU:</span>
                    <span>{systemInfo.cpu_brand} ({systemInfo.cpu_count} cores)</span>
                  </div>
                  <div className="info-row">
                    <span>Total Memory:</span>
                    <span>{formatMemory(systemInfo.total_memory)}</span>
                  </div>
                  <div className="info-row">
                    <span>Available Memory:</span>
                    <span>{formatMemory(systemInfo.available_memory)}</span>
                  </div>
                </div>
              ) : (
                <p>Loading system information...</p>
              )}
            </div>

            <div className="system-section">
              <h2>🔧 Services Status</h2>
              <div className="services-status">
                <div className="service-item">
                  <span>Ollama (LLM)</span>
                  <span className={`status ${ollamaStatus ? 'online' : 'offline'}`}>
                    {ollamaStatus ? '✅ Running' : '❌ Offline'}
                  </span>
                </div>
                <div className="service-item">
                  <span>ChromaDB</span>
                  <span className="status unknown">❓ Unknown</span>
                </div>
              </div>
            </div>

            <div className="system-section">
              <h2>📊 Statistics</h2>
              <div className="stats">
                <div className="stat-item">
                  <span>Documents Indexed:</span>
                  <span>{documents.length}</span>
                </div>
                <div className="stat-item">
                  <span>Conversations:</span>
                  <span>{Math.max(0, Math.floor(messages.length / 2))}</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;