import React from 'react';

export type ViewType = 'welcome' | 'chat' | 'documents' | 'transfer' | 'system';

interface NavigationProps {
  currentView: ViewType;
  onViewChange: (view: ViewType) => void;
}

const Navigation: React.FC<NavigationProps> = ({ currentView, onViewChange }) => {
  return (
    <div className="tab-navigation">
      <button 
        className={currentView === 'chat' ? 'active' : ''}
        onClick={() => onViewChange('chat')}
      >
        💬 Chat
      </button>
      <button 
        className={currentView === 'documents' ? 'active' : ''}
        onClick={() => onViewChange('documents')}
      >
        📄 Documents
      </button>
      <button 
        className={currentView === 'transfer' ? 'active' : ''}
        onClick={() => onViewChange('transfer')}
      >
        🔄 Transfer
      </button>
      <button 
        className={currentView === 'system' ? 'active' : ''}
        onClick={() => onViewChange('system')}
      >
        ⚙️ System
      </button>
    </div>
  );
};

export default Navigation;