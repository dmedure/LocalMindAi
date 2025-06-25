import React, { useState } from 'react';
import { Agent } from '../../types/agent';
import { getAgentIcon } from '../../utils/formatting';

interface AgentDropdownProps {
  agents: Agent[];
  currentAgent: Agent | null;
  onAgentSelect: (agent: Agent) => void;
  onCreateAgent: () => void;
}

const AgentDropdown: React.FC<AgentDropdownProps> = ({
  agents,
  currentAgent,
  onAgentSelect,
  onCreateAgent
}) => {
  const [showDropdown, setShowDropdown] = useState(false);

  const handleAgentSelect = (agent: Agent) => {
    onAgentSelect(agent);
    setShowDropdown(false);
  };

  return (
    <div className="agent-menu">
      <div 
        className="current-agent" 
        onClick={() => setShowDropdown(!showDropdown)}
      >
        {currentAgent ? currentAgent.name : 'Select Agent'} ▼
      </div>
      {showDropdown && (
        <div className="agent-dropdown">
          {agents.map(agent => (
            <div 
              key={agent.id}
              className="agent-option"
              onClick={() => handleAgentSelect(agent)}
            >
              {getAgentIcon(agent.specialization)} {agent.name}
              <span className="agent-spec">({agent.specialization})</span>
            </div>
          ))}
          <hr />
          <div 
            className="agent-option"
            onClick={() => {
              setShowDropdown(false);
              onCreateAgent();
            }}
          >
            ➕ Create New Agent
          </div>
        </div>
      )}
    </div>
  );
};

export default AgentDropdown;