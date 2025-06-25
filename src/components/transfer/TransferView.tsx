import React, { useState } from 'react';
import { Agent } from '../../types/agent';

interface TransferViewProps {
  currentAgent: Agent;
}

const TransferView: React.FC<TransferViewProps> = ({ currentAgent }) => {
  const [domain, setDomain] = useState('');

  return (
    <div className="transfer-container">
      <div className="transfer-section">
        <h2>📤 Export Agent Knowledge</h2>
        <p>Export your agent's knowledge for backup or sharing</p>
        <div className="transfer-actions">
          <button className="primary-action">
            Export {currentAgent.name} Knowledge
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
            value={domain}
            onChange={(e) => setDomain(e.target.value)}
          />
          <button className="primary-action">Create Specialized Agent</button>
        </div>
        <p className="help-text">
          Create a new agent specialized in a specific domain from your current agent's knowledge
        </p>
      </div>
    </div>
  );
};

export default TransferView;