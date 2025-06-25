import React from 'react';
import { Agent } from '../../types/agent';
import { ServiceStatus } from '../../types/service';
import AgentDropdown from '../agents/AgentDropdown';

interface HeaderProps {
  agents: Agent[];
  currentAgent: Agent | null;
  ollamaStatus: ServiceStatus;
  chromaStatus: ServiceStatus;
  onAgentSelect: (agent: Agent) => void;
  onCreateAgent: () => void;
}

const Header: React.FC<HeaderProps> = ({
  agents,
  currentAgent,
  ollamaStatus,
  chromaStatus,
  onAgentSelect,
  onCreateAgent
}) => {
  return (
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
            <AgentDropdown
              agents={agents}
              currentAgent={currentAgent}
              onAgentSelect={onAgentSelect}
              onCreateAgent={onCreateAgent}
            />
            <button 
              className="create-agent-btn"
              onClick={onCreateAgent}
            >
              + New Agent
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default Header;