import React from 'react';
import { Agent } from '../../types/agent';
import { Message } from '../../types/message';
import { useServiceStatus } from '../../hooks/useServiceStatus';

interface SystemViewProps {
  agents: Agent[];
  messages: Message[];
}

const SystemView: React.FC<SystemViewProps> = ({ agents, messages }) => {
  const { ollamaStatus, chromaStatus } = useServiceStatus();

  return (
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
            <div className="stat-number">0</div>
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
  );
};

export default SystemView;