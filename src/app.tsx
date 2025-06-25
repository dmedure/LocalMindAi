import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import './App.css';

// Enhanced interfaces matching backend
interface Agent {
  id: string;
  name: string;
  specialization: string;
  personality: string;
  instructions?: string;
  created_at: string;
  updated_at: string;
  avatar?: string;
  status: 'Active' | 'Archived' | 'Training' | 'Disabled';
  version: number;
  capabilities: string[];
  knowledge_source_count: number;
  conversation_count: number;
  last_used: string;
  model_name: string;
  context_window: number;
  temperature: number;
}

interface Message {
  id: string;
  content: string;
  sender: 'user' | 'agent';
  timestamp: string;
  agent_id: string;
  message_type: 'Text' | 'Image' | 'Document' | 'System';
  attachments: Attachment[];
  response_time_ms?: number;
}

interface Attachment {
  id: string;
  file_name: string;
  file_path: string;
  file_type: string;
  file_size: number;
  uploaded_at: string;
}

interface Document {
  id: string;
  name: string;
  doc_type: string;
  size: number;
  path: string;
  summary?: string;
  indexed_at: string;
  content_preview: string;
  keywords: string[];
  agent_reviews: AgentReview[];
}

interface AgentReview {
  agent_id: string;
  review_type: string;
  review_content: string;
  reviewed_at: string;
  rating?: number;
}

interface ServiceStatus {
  ollama: ServiceHealth;
  chromadb: ServiceHealth;
  local_storage: ServiceHealth;
}

interface ServiceHealth {
  status: string;
  last_check: string;
  response_time_ms?: number;
  error_message?: string;
}

function App() {
  // Enhanced state management
  const [currentView, setCurrentView] = useState<'welcome' | 'chat' | 'documents' | 'transfer' | 'system'>('welcome');
  const [agents, setAgents] = useState<Agent[]>([]);
  const [currentAgent, setCurrentAgent] = useState<Agent | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [documents, setDocuments] = useState<Document[]>([]);
  const [inputMessage, setInputMessage] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [serviceStatus, setServiceStatus] = useState<ServiceStatus | null>(null);
  
  // Agent management modals
  const [showCreateAgentModal, setShowCreateAgentModal] = useState(false);
  const [showEditAgentModal, setShowEditAgentModal] = useState(false);
  const [showDeleteConfirmModal, setShowDeleteConfirmModal] = useState(false);
  const [showAgentDropdown, setShowAgentDropdown] = useState(false);
  const [agentToEdit, setAgentToEdit] = useState<Agent | null>(null);
  const [agentToDelete, setAgentToDelete] = useState<Agent | null>(null);
  
  // Search and filtering
  const [agentSearchQuery, setAgentSearchQuery] = useState('');
  const [filteredAgents, setFilteredAgents] = useState<Agent[]>([]);
  const [selectedAgentStatus, setSelectedAgentStatus] = useState<string>('all');
  
  // File handling
  const [dragOver, setDragOver] = useState(false);
  const [uploadingFile, setUploadingFile] = useState(false);
  
  // Agent creation/edit form state
  const [formAgent, setFormAgent] = useState<Partial<Agent>>({
    name: '',
    specialization: 'general',
    personality: 'friendly',
    instructions: '',
    model_name: 'llama3.1:8b',
    temperature: 0.7,
    context_window: 4096,
  });
  
  // References
  const messagesEndRef = useRef<HTMLDivElement>(null);
  
  // Load data on startup
  useEffect(() => {
    loadAgents();
    checkServiceStatus();
    loadDocuments();
    
    // Check service status periodically
    const statusInterval = setInterval(checkServiceStatus, 30000);
    return () => clearInterval(statusInterval);
  }, []);
  
  // Filter agents based on search and status
  useEffect(() => {
    let filtered = agents;
    
    if (agentSearchQuery) {
      const query = agentSearchQuery.toLowerCase();
      filtered = filtered.filter(agent =>
        agent.name.toLowerCase().includes(query) ||
        agent.specialization.toLowerCase().includes(query) ||
        agent.personality.toLowerCase().includes(query)
      );
    }
    
    if (selectedAgentStatus !== 'all') {
      filtered = filtered.filter(agent => agent.status === selectedAgentStatus);
    }
    
    setFilteredAgents(filtered);
  }, [agents, agentSearchQuery, selectedAgentStatus]);
  
  // Auto-scroll messages
  useEffect(() => {
    scrollToBottom();
  }, [messages]);
  
  // Load messages when agent changes
  useEffect(() => {
    if (currentAgent) {
      loadMessagesForAgent(currentAgent.id);
    }
  }, [currentAgent]);
  
  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };
  
  const loadAgents = async () => {
    try {
      const agentList = await invoke<Agent[]>('get_agents');
      setAgents(agentList);
      
      if (agentList.length === 0) {
        setCurrentView('welcome');
        setCurrentAgent(null);
      } else if (!currentAgent || !agentList.find(a => a.id === currentAgent.id)) {
        // Select most recently used agent
        const sortedAgents = [...agentList].sort((a, b) => 
          new Date(b.last_used).getTime() - new Date(a.last_used).getTime()
        );
        setCurrentAgent(sortedAgents[0]);
        if (currentView === 'welcome') {
          setCurrentView('chat');
        }
      }
    } catch (error) {
      console.error('Failed to load agents:', error);
      showErrorNotification('Failed to load agents');
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
  
  const loadDocuments = async () => {
    try {
      const documentList = await invoke<Document[]>('get_documents');
      setDocuments(documentList);
    } catch (error) {
      console.error('Failed to load documents:', error);
    }
  };
  
  const checkServiceStatus = async () => {
    try {
      const status = await invoke<ServiceStatus>('check_service_status');
      setServiceStatus(status);
    } catch (error) {
      console.error('Failed to check service status:', error);
    }
  };
  
  // ENHANCED AGENT MANAGEMENT
  
  const createAgent = async () => {
    if (!formAgent.name?.trim()) {
      showErrorNotification('Please enter an agent name');
      return;
    }
    
    try {
      const newAgent = await invoke<Agent>('create_agent', { agent: formAgent });
      await loadAgents();
      setCurrentAgent(newAgent);
      setCurrentView('chat');
      resetAgentForm();
      setShowCreateAgentModal(false);
      showSuccessNotification(`Agent "${newAgent.name}" created successfully!`);
    } catch (error) {
      console.error('Failed to create agent:', error);
      showErrorNotification(error as string);
    }
  };
  
  const updateAgent = async () => {
    if (!agentToEdit || !formAgent.name?.trim()) {
      showErrorNotification('Please enter an agent name');
      return;
    }
    
    try {
      const updatedAgent = await invoke<Agent>('update_agent', {
        agentId: agentToEdit.id,
        updates: { ...agentToEdit, ...formAgent }
      });
      
      await loadAgents();
      
      // Update current agent if it's the one being edited
      if (currentAgent?.id === updatedAgent.id) {
        setCurrentAgent(updatedAgent);
      }
      
      resetAgentForm();
      setShowEditAgentModal(false);
      setAgentToEdit(null);
      showSuccessNotification(`Agent "${updatedAgent.name}" updated successfully!`);
    } catch (error) {
      console.error('Failed to update agent:', error);
      showErrorNotification(error as string);
    }
  };
  
  const deleteAgent = async () => {
    if (!agentToDelete) return;
    
    try {
      await invoke('delete_agent', { agentId: agentToDelete.id });
      
      // If deleted agent was current, switch to another
      if (currentAgent?.id === agentToDelete.id) {
        const remainingAgents = agents.filter(a => a.id !== agentToDelete.id);
        if (remainingAgents.length > 0) {
          setCurrentAgent(remainingAgents[0]);
        } else {
          setCurrentAgent(null);
          setCurrentView('welcome');
        }
      }
      
      await loadAgents();
      setShowDeleteConfirmModal(false);
      setAgentToDelete(null);
      showSuccessNotification(`Agent "${agentToDelete.name}" deleted successfully!`);
    } catch (error) {
      console.error('Failed to delete agent:', error);
      showErrorNotification(error as string);
    }
  };
  
  const duplicateAgent = async (sourceAgent: Agent) => {
    const newName = `${sourceAgent.name} (Copy)`;
    
    try {
      const duplicatedAgent = await invoke<Agent>('duplicate_agent', {
        agentId: sourceAgent.id,
        newName
      });
      
      await loadAgents();
      setCurrentAgent(duplicatedAgent);
      showSuccessNotification(`Agent duplicated as "${duplicatedAgent.name}"!`);
    } catch (error) {
      console.error('Failed to duplicate agent:', error);
      showErrorNotification(error as string);
    }
  };
  
  const setAgentStatus = async (agentId: string, status: Agent['status']) => {
    try {
      await invoke('set_agent_status', { agentId, status });
      await loadAgents();
      showSuccessNotification('Agent status updated');
    } catch (error) {
      console.error('Failed to update agent status:', error);
      showErrorNotification(error as string);
    }
  };
  
  const searchAgents = async (query: string) => {
    if (!query.trim()) {
      setFilteredAgents(agents);
      return;
    }
    
    try {
      const results = await invoke<Agent[]>('search_agents', { query });
      setFilteredAgents(results);
    } catch (error) {
      console.error('Failed to search agents:', error);
    }
  };
  
  const resetAgentForm = () => {
    setFormAgent({
      name: '',
      specialization: 'general',
      personality: 'friendly',
      instructions: '',
      model_name: 'llama3.1:8b',
      temperature: 0.7,
      context_window: 4096,
    });
  };
  
  const openEditAgentModal = (agent: Agent) => {
    setAgentToEdit(agent);
    setFormAgent({
      name: agent.name,
      specialization: agent.specialization,
      personality: agent.personality,
      instructions: agent.instructions || '',
      model_name: agent.model_name,
      temperature: agent.temperature,
      context_window: agent.context_window,
    });
    setShowEditAgentModal(true);
  };
  
  const openDeleteConfirmModal = (agent: Agent) => {
    setAgentToDelete(agent);
    setShowDeleteConfirmModal(true);
  };
  
  // MESSAGING WITH ENHANCED FEATURES
  
  const sendMessage = async () => {
    if (!inputMessage.trim() || !currentAgent || isLoading) return;
    
    setIsLoading(true);
    const messageText = inputMessage.trim();
    setInputMessage('');
    
    try {
      const response = await invoke<string>('send_message_to_agent', {
        agentId: currentAgent.id,
        message: messageText
      });
      
      // Reload messages to get the complete conversation
      await loadMessagesForAgent(currentAgent.id);
      await loadAgents(); // Update agent stats
    } catch (error) {
      console.error('Failed to send message:', error);
      showErrorNotification('Failed to send message. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };
  
  // FILE HANDLING
  
  const handleFileUpload = async (file: File | { name: string; path: string; type: string }) => {
    setUploadingFile(true);
    
    try {
      const filePath = 'path' in file ? file.path : file.name;
      
      if (file.type.startsWith('image/')) {
        // Handle image upload
        const attachment = await invoke<Attachment>('upload_image', {
          filePath: filePath
        });
        
        showSuccessNotification(`Image "${file.name}" uploaded successfully!`);
        
        // Optionally start image analysis
        if (currentAgent) {
          const analysisPrompt = 'Please analyze this image and describe what you see.';
          const analysis = await invoke<string>('analyze_image_with_agent', {
            agentId: currentAgent.id,
            attachmentId: attachment.id,
            prompt: analysisPrompt
          });
          
          // Add analysis as a message
          await loadMessagesForAgent(currentAgent.id);
        }
      } else if (file.type === 'application/pdf' || 
                 file.type === 'application/vnd.openxmlformats-officedocument.wordprocessingml.document' ||
                 file.type === 'text/plain') {
        // Handle document upload
        const document = await invoke<Document>('add_document', {
          filePath: filePath
        });
        
        await loadDocuments();
        showSuccessNotification(`Document "${file.name}" added successfully!`);
      } else {
        showErrorNotification('Unsupported file type');
      }
    } catch (error) {
      console.error('Failed to upload file:', error);
      showErrorNotification(error as string);
    } finally {
      setUploadingFile(false);
    }
  };
  
  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      handleFileUpload(files[0]);
    }
  };
  
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  };
  
  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  };
  
  const openFileDialog = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Supported Files',
          extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'pdf', 'docx', 'txt', 'md']
        }]
      });
      
      if (selected && typeof selected === 'string') {
        // Create a mock file object for handling
        const fileName = selected.split('/').pop() || 'unknown';
        const mockFile = {
          name: fileName,
          path: selected,
          type: getFileType(fileName)
        };
        
        await handleFileUpload(mockFile);
      }
    } catch (error) {
      console.error('Failed to open file dialog:', error);
    }
  };
  
  const getFileType = (fileName: string): string => {
    const ext = fileName.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'png': case 'jpg': case 'jpeg': case 'gif': case 'webp':
        return `image/${ext}`;
      case 'pdf':
        return 'application/pdf';
      case 'docx':
        return 'application/vnd.openxmlformats-officedocument.wordprocessingml.document';
      case 'txt': case 'md':
        return 'text/plain';
      default:
        return 'application/octet-stream';
    }
  };
  
  // DOCUMENT REVIEW
  
  const reviewDocumentWithAgent = async (documentId: string, reviewType: string) => {
    if (!currentAgent) {
      showErrorNotification('Please select an agent first');
      return;
    }
    
    try {
      const review = await invoke<string>('review_document_with_agent', {
        agentId: currentAgent.id,
        documentId,
        reviewType
      });
      
      await loadDocuments(); // Reload to get updated reviews
      showSuccessNotification(`Document ${reviewType} completed!`);
      
      // Optionally show the review in a modal or add as a message
      console.log('Review result:', review);
    } catch (error) {
      console.error('Failed to review document:', error);
      showErrorNotification(error as string);
    }
  };
  
  // UTILITY FUNCTIONS
  
  const showSuccessNotification = (message: string) => {
    // In a real app, you'd use a proper notification system
    console.log('Success:', message);
  };
  
  const showErrorNotification = (message: string) => {
    // In a real app, you'd use a proper notification system
    console.error('Error:', message);
    alert(message); // Temporary solution
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
  
  const formatTime = (dateString: string) => {
    return new Date(dateString).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };
  
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };
  
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': case 'online': return 'online';
      case 'offline': case 'unhealthy': return 'offline';
      default: return 'unknown';
    }
  };
  
  const getAgentStatusBadge = (status: Agent['status']) => {
    const colors = {
      Active: 'bg-green-500',
      Archived: 'bg-gray-500',
      Training: 'bg-yellow-500',
      Disabled: 'bg-red-500'
    };
    
    return `px-2 py-1 rounded text-xs font-medium text-white ${colors[status]}`;
  };

  const getAgentIcon = (specialization: string): string => {
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
  };

  const getPersonalityIcon = (personality: string): string => {
    const icons = {
      professional: '💼',
      friendly: '😊',
      analytical: '🔍',
      creative: '🎨',
      concise: '⚡',
      detailed: '📝'
    };
    return icons[personality as keyof typeof icons] || '😊';
  };

  return (
    <div 
      className="app"
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
    >
      {/* Enhanced Header */}
      <div className="app-header">
        <h1>🧠 LocalMind AI Agent</h1>
        <div className="header-controls">
          {/* Service Status */}
          <div className="status-indicators">
            <div className={`status ${getStatusColor(serviceStatus?.ollama.status || 'unknown')}`}>
              Ollama: {serviceStatus?.ollama.status || 'Unknown'}
              {serviceStatus?.ollama.response_time_ms && (
                <span className="ml-1 text-xs">({serviceStatus.ollama.response_time_ms}ms)</span>
              )}
            </div>
            <div className={`status ${getStatusColor(serviceStatus?.chromadb.status || 'unknown')}`}>
              ChromaDB: {serviceStatus?.chromadb.status || 'Unknown'}
            </div>
          </div>
          
          {/* Enhanced Agent Selector */}
          {agents.length > 0 && (
            <div className="agent-selector">
              <div className="agent-menu">
                <div 
                  className="current-agent" 
                  onClick={() => setShowAgentDropdown(!showAgentDropdown)}
                >
                  {currentAgent ? (
                    <>
                      {getAgentIcon(currentAgent.specialization)} {currentAgent.name}
                      <span className={getAgentStatusBadge(currentAgent.status)}>
                        {currentAgent.status}
                      </span>
                    </>
                  ) : 'Select Agent'} ▼
                </div>
                
                {showAgentDropdown && (
                  <div className="agent-dropdown">
                    {/* Agent Search */}
                    <div className="p-2 border-b border-gray-600">
                      <input
                        type="text"
                        placeholder="Search agents..."
                        className="w-full bg-gray-700 text-white px-2 py-1 rounded text-sm"
                        value={agentSearchQuery}
                        onChange={(e) => {
                          setAgentSearchQuery(e.target.value);
                          searchAgents(e.target.value);
                        }}
                      />
                    </div>
                    
                    {/* Status Filter */}
                    <div className="p-2 border-b border-gray-600">
                      <select
                        className="w-full bg-gray-700 text-white px-2 py-1 rounded text-sm"
                        value={selectedAgentStatus}
                        onChange={(e) => setSelectedAgentStatus(e.target.value)}
                      >
                        <option value="all">All Agents</option>
                        <option value="Active">Active</option>
                        <option value="Archived">Archived</option>
                        <option value="Training">Training</option>
                        <option value="Disabled">Disabled</option>
                      </select>
                    </div>
                    
                    {/* Agent List */}
                    <div className="max-h-64 overflow-y-auto">
                      {filteredAgents.map(agent => (
                        <div key={agent.id} className="agent-option-enhanced">
                          <div 
                            className="flex-1 cursor-pointer"
                            onClick={() => selectAgent(agent)}
                          >
                            <div className="flex items-center justify-between">
                              <span>{getAgentIcon(agent.specialization)} {agent.name}</span>
                              <span className={getAgentStatusBadge(agent.status)}>
                                {agent.status}
                              </span>
                            </div>
                            <div className="text-xs text-gray-400 mt-1">
                              {agent.specialization} • {agent.conversation_count} chats • 
                              Last used: {formatTime(agent.last_used)}
                            </div>
                          </div>
                          
                          {/* Agent Actions */}
                          <div className="flex gap-1 ml-2">
                            <button
                              className="text-blue-400 hover:text-blue-300 text-xs"
                              onClick={(e) => {
                                e.stopPropagation();
                                openEditAgentModal(agent);
                                setShowAgentDropdown(false);
                              }}
                              title="Edit Agent"
                            >
                              ✏️
                            </button>
                            <button
                              className="text-green-400 hover:text-green-300 text-xs"
                              onClick={(e) => {
                                e.stopPropagation();
                                duplicateAgent(agent);
                                setShowAgentDropdown(false);
                              }}
                              title="Duplicate Agent"
                            >
                              📋
                            </button>
                            <button
                              className="text-red-400 hover:text-red-300 text-xs"
                              onClick={(e) => {
                                e.stopPropagation();
                                openDeleteConfirmModal(agent);
                                setShowAgentDropdown(false);
                              }}
                              title="Delete Agent"
                            >
                              🗑️
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                    
                    <div className="p-2 border-t border-gray-600">
                      <button 
                        className="w-full text-left agent-option"
                        onClick={() => {
                          setShowCreateAgentModal(true);
                          setShowAgentDropdown(false);
                        }}
                      >
                        ➕ Create New Agent
                      </button>
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

      {/* File Drop Zone Overlay */}
      {dragOver && (
        <div className="fixed inset-0 bg-blue-500 bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white text-blue-900 p-8 rounded-lg text-center">
            <h3 className="text-xl font-bold mb-2">Drop files here</h3>
            <p>Images and documents supported</p>
          </div>
        </div>
      )}

      {/* Navigation */}
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
            📄 Documents ({documents.length})
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

        {/* Enhanced Chat Interface */}
        {currentView === 'chat' && currentAgent && (
          <div className="chat-container">
            {/* Agent Information Panel */}
            <div className="agent-intro">
              <div className="flex items-center justify-between">
                <div>
                  <h3>👋 Hello! I'm {currentAgent.name}</h3>
                  <p>{getAgentIntroduction(currentAgent)}</p>
                  {currentAgent.instructions && (
                    <div className="custom-instructions">
                      <strong>Special Focus:</strong> {currentAgent.instructions}
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <span className={getAgentStatusBadge(currentAgent.status)}>
                    {currentAgent.status}
                  </span>
                  <span className="text-sm text-gray-400">
                    v{currentAgent.version} • {currentAgent.conversation_count} chats
                  </span>
                </div>
              </div>
            </div>

            {/* Messages Area */}
            <div className="messages">
              {messages.map((message) => (
                <div key={message.id} className={`message ${message.sender}`}>
                  <div className="message-content">
                    {message.content}
                    
                    {/* Show attachments */}
                    {message.attachments.length > 0 && (
                      <div className="mt-2">
                        {message.attachments.map(attachment => (
                          <div key={attachment.id} className="attachment-preview">
                            {attachment.file_type.startsWith('image/') ? (
                              <img 
                                src={`file://${attachment.file_path}`} 
                                alt={attachment.file_name}
                                className="max-w-xs rounded"
                              />
                            ) : (
                              <div className="document-attachment">
                                📄 {attachment.file_name} ({formatFileSize(attachment.file_size)})
                              </div>
                            )}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                  <div className="message-time">
                    {formatTime(message.timestamp)}
                    {message.response_time_ms && (
                      <span className="ml-2 text-xs">({message.response_time_ms}ms)</span>
                    )}
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
              <div ref={messagesEndRef} />
            </div>

            {/* Enhanced Input Container */}
            <div className="input-container">
              <div className="flex items-center gap-2">
                <button
                  onClick={openFileDialog}
                  className="p-2 rounded bg-gray-700 hover:bg-gray-600 transition-colors"
                  title="Upload file"
                  disabled={uploadingFile}
                >
                  {uploadingFile ? '⏳' : '📎'}
                </button>
                <textarea
                  value={inputMessage}
                  onChange={(e) => setInputMessage(e.target.value)}
                  placeholder={`Ask ${currentAgent.name} anything, or drag & drop files...`}
                  rows={2}
                  onKeyPress={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      sendMessage();
                    }
                  }}
                  className="flex-1"
                />
                <button 
                  onClick={sendMessage} 
                  disabled={!inputMessage.trim() || isLoading}
                  className="px-4 py-2"
                >
                  {isLoading ? '⏳' : 'Send'}
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Enhanced Documents Tab */}
        {currentView === 'documents' && (
          <div className="documents-container">
            <div className="documents-header">
              <button onClick={openFileDialog} disabled={uploadingFile}>
                {uploadingFile ? '⏳ Uploading...' : '📁 Add Document'}
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
                  <button onClick={openFileDialog} className="mt-4 px-4 py-2 bg-blue-600 rounded">
                    Upload First Document
                  </button>
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
                      <p><strong>Keywords:</strong> {doc.keywords.join(', ') || 'None'}</p>
                      
                      {doc.content_preview && (
                        <div className="doc-summary">
                          <strong>Preview:</strong> {doc.content_preview}
                        </div>
                      )}
                      
                      {/* Agent Review Actions */}
                      {currentAgent && (
                        <div className="mt-3 space-y-2">
                          <div className="flex gap-2 flex-wrap">
                            <button
                              onClick={() => reviewDocumentWithAgent(doc.id, 'summary')}
                              className="px-3 py-1 bg-blue-600 text-white rounded text-sm"
                            >
                              📝 Summarize
                            </button>
                            <button
                              onClick={() => reviewDocumentWithAgent(doc.id, 'analysis')}
                              className="px-3 py-1 bg-green-600 text-white rounded text-sm"
                            >
                              🔍 Analyze
                            </button>
                            <button
                              onClick={() => reviewDocumentWithAgent(doc.id, 'questions')}
                              className="px-3 py-1 bg-purple-600 text-white rounded text-sm"
                            >
                              ❓ Questions
                            </button>
                            <button
                              onClick={() => reviewDocumentWithAgent(doc.id, 'critique')}
                              className="px-3 py-1 bg-orange-600 text-white rounded text-sm"
                            >
                              🎯 Critique
                            </button>
                          </div>
                          
                          {/* Show existing reviews */}
                          {doc.agent_reviews.length > 0 && (
                            <div className="mt-2">
                              <details className="text-sm">
                                <summary className="cursor-pointer text-gray-400">
                                  {doc.agent_reviews.length} review(s) by agents
                                </summary>
                                <div className="mt-2 space-y-2">
                                  {doc.agent_reviews.map((review, index) => (
                                    <div key={index} className="bg-gray-800 p-2 rounded text-xs">
                                      <div className="flex justify-between items-center mb-1">
                                        <span className="font-semibold">{review.review_type}</span>
                                        <span className="text-gray-500">
                                          {formatTime(review.reviewed_at)}
                                        </span>
                                      </div>
                                      <p>{review.review_content}</p>
                                    </div>
                                  ))}
                                </div>
                              </details>
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {/* Transfer Tab - Enhanced */}
        {currentView === 'transfer' && (
          <div className="transfer-container">
            <div className="transfer-section">
              <h2>📤 Export Agent Knowledge</h2>
              <p>Export your agent's knowledge for backup or sharing</p>
              <div className="transfer-actions">
                <button 
                  className="primary-action"
                  onClick={async () => {
                    if (!currentAgent) return;
                    try {
                      const exportPath = await invoke<string>('export_agent_knowledge', {
                        agentId: currentAgent.id
                      });
                      showSuccessNotification(`Knowledge exported to: ${exportPath}`);
                    } catch (error) {
                      showErrorNotification(error as string);
                    }
                  }}
                  disabled={!currentAgent}
                >
                  Export {currentAgent?.name} Knowledge
                </button>
                <p className="help-text">
                  Exports conversations, agent configuration, and associated documents
                </p>
              </div>
            </div>

            <div className="transfer-section">
              <h2>📥 Import Knowledge</h2>
              <div className="import-strategies">
                <button onClick={() => {
                  // Open file dialog for import
                  // Implementation would call import_agent_knowledge
                }}>Smart Merge</button>
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

        {/* Enhanced System Tab */}
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
                  <div className="stat-number">
                    {agents.reduce((sum, agent) => sum + agent.conversation_count, 0)}
                  </div>
                  <div className="stat-label">Total Conversations</div>
                </div>
                <div className="stat-card">
                  <div className="stat-number">{documents.length}</div>
                  <div className="stat-label">Documents</div>
                </div>
                <div className="stat-card">
                  <div className="stat-number">
                    {agents.reduce((sum, agent) => sum + agent.knowledge_source_count, 0)}
                  </div>
                  <div className="stat-label">Knowledge Sources</div>
                </div>
              </div>
            </div>

            <div className="system-section">
              <h2>🔧 Service Status</h2>
              <div className="service-status">
                <div className="service-item">
                  <span>Ollama Service</span>
                  <span className={getStatusColor(serviceStatus?.ollama.status || 'unknown')}>
                    {serviceStatus?.ollama.status === 'healthy' ? '🟢 Online' : '🔴 Offline'}
                    {serviceStatus?.ollama.response_time_ms && (
                      ` (${serviceStatus.ollama.response_time_ms}ms)`
                    )}
                  </span>
                </div>
                <div className="service-item">
                  <span>ChromaDB</span>
                  <span className={getStatusColor(serviceStatus?.chromadb.status || 'unknown')}>
                    {serviceStatus?.chromadb.status === 'healthy' ? '🟢 Connected' : '🔴 Disconnected'}
                  </span>
                </div>
                <div className="service-item">
                  <span>Local Storage</span>
                  <span className={getStatusColor(serviceStatus?.local_storage.status || 'unknown')}>
                    {serviceStatus?.local_storage.status === 'healthy' ? '🟢 Healthy' : '🔴 Error'}
                  </span>
                </div>
              </div>
            </div>
            
            {/* Agent Management Overview */}
            <div className="system-section">
              <h2>🤖 Agent Overview</h2>
              <div className="space-y-2">
                {agents.map(agent => (
                  <div key={agent.id} className="flex items-center justify-between p-3 bg-gray-800 rounded">
                    <div className="flex items-center gap-3">
                      <span>{getAgentIcon(agent.specialization)}</span>
                      <div>
                        <div className="font-semibold">{agent.name}</div>
                        <div className="text-sm text-gray-400">
                          {agent.specialization} • {agent.conversation_count} chats
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={getAgentStatusBadge(agent.status)}>
                        {agent.status}
                      </span>
                      <div className="flex gap-1">
                        <button
                          onClick={() => openEditAgentModal(agent)}
                          className="text-blue-400 hover:text-blue-300"
                          title="Edit"
                        >
                          ✏️
                        </button>
                        <button
                          onClick={() => duplicateAgent(agent)}
                          className="text-green-400 hover:text-green-300"
                          title="Duplicate"
                        >
                          📋
                        </button>
                        <button
                          onClick={() => openDeleteConfirmModal(agent)}
                          className="text-red-400 hover:text-red-300"
                          title="Delete"
                        >
                          🗑️
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Enhanced Agent Creation/Edit Modal */}
      {(showCreateAgentModal || showEditAgentModal) && (
        <div className="modal-overlay" onClick={() => {
          setShowCreateAgentModal(false);
          setShowEditAgentModal(false);
          resetAgentForm();
        }}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{showEditAgentModal ? '✏️ Edit Agent' : '🤖 Create New AI Agent'}</h3>
            
            <div className="form-group">
              <label>Agent Name *</label>
              <input
                type="text"
                value={formAgent.name || ''}
                onChange={(e) => setFormAgent({...formAgent, name: e.target.value})}
                placeholder="e.g., WorkBot, CodeMaster, ResearchPal"
              />
            </div>

            <div className="form-group">
              <label>Specialization</label>
              <select
                value={formAgent.specialization || 'general'}
                onChange={(e) => setFormAgent({...formAgent, specialization: e.target.value})}
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
                    className={`personality-option ${formAgent.personality === personality ? 'selected' : ''}`}
                    onClick={() => setFormAgent({...formAgent, personality})}
                  >
                    {getPersonalityIcon(personality)} {personality.charAt(0).toUpperCase() + personality.slice(1)}
                  </div>
                ))}
              </div>
            </div>

            <div className="form-group">
              <label>Model Configuration</label>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-sm">Model</label>
                  <select
                    value={formAgent.model_name || 'llama3.1:8b'}
                    onChange={(e) => setFormAgent({...formAgent, model_name: e.target.value})}
                  >
                    <option value="llama3.1:8b">Llama 3.1 8B</option>
                    <option value="llama3.1:70b">Llama 3.1 70B</option>
                    <option value="codellama:7b">CodeLlama 7B</option>
                    <option value="mistral:7b">Mistral 7B</option>
                  </select>
                </div>
                <div>
                  <label className="text-sm">Temperature: {formAgent.temperature}</label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.1"
                    value={formAgent.temperature || 0.7}
                    onChange={(e) => setFormAgent({...formAgent, temperature: parseFloat(e.target.value)})}
                  />
                </div>
              </div>
            </div>

            <div className="form-group">
              <label>Custom Instructions (Optional)</label>
              <textarea
                value={formAgent.instructions || ''}
                onChange={(e) => setFormAgent({...formAgent, instructions: e.target.value})}
                placeholder="Any specific instructions for how this agent should behave..."
                rows={3}
              />
            </div>

            <div className="modal-actions">
              <button 
                className="btn-secondary"
                onClick={() => {
                  setShowCreateAgentModal(false);
                  setShowEditAgentModal(false);
                  resetAgentForm();
                }}
              >
                Cancel
              </button>
              <button 
                className="btn-primary"
                onClick={showEditAgentModal ? updateAgent : createAgent}
                disabled={!formAgent.name?.trim()}
              >
                {showEditAgentModal ? 'Update Agent' : 'Create Agent'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {showDeleteConfirmModal && agentToDelete && (
        <div className="modal-overlay" onClick={() => setShowDeleteConfirmModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>🗑️ Delete Agent</h3>
            <p>
              Are you sure you want to delete <strong>{agentToDelete.name}</strong>?
            </p>
            <p className="text-red-400 text-sm mt-2">
              This will permanently delete the agent and all its conversations. This action cannot be undone.
            </p>
            
            <div className="modal-actions">
              <button 
                className="btn-secondary"
                onClick={() => setShowDeleteConfirmModal(false)}
              >
                Cancel
              </button>
              <button 
                className="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded"
                onClick={deleteAgent}
              >
                Delete Agent
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;