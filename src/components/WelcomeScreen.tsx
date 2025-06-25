import React from 'react';

interface WelcomeScreenProps {
  onCreateAgent: () => void;
}

const WelcomeScreen: React.FC<WelcomeScreenProps> = ({ onCreateAgent }) => {
  return (
    <div className="welcome-screen">
      <h2>Welcome to LocalMind!</h2>
      <p>
        Create your first AI agent to get started. Each agent can have its own personality, 
        specialization, and knowledge base - all running privately on your device.
      </p>
      <button 
        className="get-started-btn"
        onClick={onCreateAgent}
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
  );
};

export default WelcomeScreen;