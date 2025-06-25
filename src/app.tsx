import React, { useState, useEffect } from 'react';
import './App.css';

// Components
import Header from './components/layout/Header';
import Navigation, { ViewType } from './components/layout/Navigation';
import WelcomeScreen from './components/WelcomeScreen';
import ChatContainer from './components/chat/ChatContainer';
import CreateAgentModal from './components/agents/CreateAgentModal';

// Views - Make sure these files exist!
import DocumentsView from './components/documents/DocumentsView';
import TransferView from './components/transfer/TransferView';
import SystemView from './components/system/SystemView';

// Hooks
import { useAgents } from './hooks/useAgents';
import { useMessages } from './hooks/useMessages';
import { useServiceStatus } from './hooks/useServiceStatus';

// Types
import { Agent } from './types/agent';

function App() {
  const [currentView, setCurrentView] = useState<ViewType>('welcome');
  const [showCreateAgentModal, setShowCreateAgentModal] = useState(false);
  
  const { agents, currentAgent, setCurrentAgent, createAgent } = useAgents();
  const { messages, sendMessage, isLoading: isMessageLoading } = useMessages(currentAgent?.id);
  const { ollamaStatus, chromaStatus } = useServiceStatus();

  // Determine if we should show welcome screen
  useEffect(() => {
    if (agents.length === 0) {
      setCurrentView('welcome');
    } else if (currentView === 'welcome' && agents.length > 0) {
      setCurrentView('chat');
    }
  }, [agents.length, currentView]);

  const handleCreateAgent = async (agent: Agent) => {
    try {
      await createAgent(agent);
      setCurrentView('chat');
    } catch (error) {
      console.error('Failed to create agent:', error);
    }
  };

  const handleAgentSelect = (agent: Agent) => {
    setCurrentAgent(agent);
    setCurrentView('chat');
  };

  const renderMainContent = () => {
    if (currentView === 'welcome' || !currentAgent) {
      return <WelcomeScreen onCreateAgent={() => setShowCreateAgentModal(true)} />;
    }

    switch (currentView) {
      case 'chat':
        return (
          <ChatContainer
            agent={currentAgent}
            messages={messages}
            isLoading={isMessageLoading}
            onSendMessage={sendMessage}
          />
        );
      case 'documents':
        return <DocumentsView />;
      case 'transfer':
        return <TransferView currentAgent={currentAgent} />;
      case 'system':
        return <SystemView agents={agents} messages={messages} />;
      default:
        return null;
    }
  };

  return (
    <div className="app">
      <Header
        agents={agents}
        currentAgent={currentAgent}
        ollamaStatus={ollamaStatus}
        chromaStatus={chromaStatus}
        onAgentSelect={handleAgentSelect}
        onCreateAgent={() => setShowCreateAgentModal(true)}
      />

      {agents.length > 0 && currentAgent && (
        <Navigation
          currentView={currentView}
          onViewChange={setCurrentView}
        />
      )}

      <div className="main-content">
        {renderMainContent()}
      </div>

      <CreateAgentModal
        isOpen={showCreateAgentModal}
        onClose={() => setShowCreateAgentModal(false)}
        onCreateAgent={handleCreateAgent}
      />
    </div>
  );
}

export default App;