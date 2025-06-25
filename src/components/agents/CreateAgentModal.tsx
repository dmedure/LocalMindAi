import React, { useState } from 'react';
import { Agent } from '../../types/agent';
import { getPersonalityIcon } from '../../utils/formatting';

interface CreateAgentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreateAgent: (agent: Agent) => Promise<void>;
}

const CreateAgentModal: React.FC<CreateAgentModalProps> = ({
  isOpen,
  onClose,
  onCreateAgent
}) => {
  const [name, setName] = useState('');
  const [specialization, setSpecialization] = useState('general');
  const [personality, setPersonality] = useState('');
  const [instructions, setInstructions] = useState('');
  const [isCreating, setIsCreating] = useState(false);

  const handleCreate = async () => {
    if (!name.trim()) {
      alert('Please enter an agent name');
      return;
    }

    setIsCreating(true);
    try {
      const agent: Agent = {
        id: crypto.randomUUID(),
        name: name.trim(),
        specialization,
        personality: personality || 'friendly',
        instructions: instructions.trim() || undefined,
        created_at: new Date().toISOString()
      };

      await onCreateAgent(agent);
      
      // Reset form
      setName('');
      setSpecialization('general');
      setPersonality('');
      setInstructions('');
      onClose();
    } catch (error) {
      alert('Failed to create agent. Please try again.');
    } finally {
      setIsCreating(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h3>🤖 Create New AI Agent</h3>
        
        <div className="form-group">
          <label>Agent Name</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., WorkBot, CodeMaster, ResearchPal"
          />
        </div>

        <div className="form-group">
          <label>Specialization</label>
          <select
            value={specialization}
            onChange={(e) => setSpecialization(e.target.value)}
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
            {['professional', 'friendly', 'analytical', 'creative', 'concise', 'detailed'].map(p => (
              <div
                key={p}
                className={`personality-option ${personality === p ? 'selected' : ''}`}
                onClick={() => setPersonality(p)}
              >
                {getPersonalityIcon(p)} {p.charAt(0).toUpperCase() + p.slice(1)}
              </div>
            ))}
          </div>
        </div>

        <div className="form-group">
          <label>Custom Instructions (Optional)</label>
          <textarea
            value={instructions}
            onChange={(e) => setInstructions(e.target.value)}
            placeholder="Any specific instructions for how this agent should behave..."
            rows={3}
          />
        </div>

        <div className="modal-actions">
          <button 
            className="btn-secondary"
            onClick={onClose}
            disabled={isCreating}
          >
            Cancel
          </button>
          <button 
            className="btn-primary"
            onClick={handleCreate}
            disabled={isCreating}
          >
            {isCreating ? 'Creating...' : 'Create Agent'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default CreateAgentModal;