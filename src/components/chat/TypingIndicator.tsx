import React from 'react';

const TypingIndicator: React.FC = () => {
  return (
    <div className="message agent">
      <div className="message-content">
        <div className="typing-indicator">
          <span></span>
          <span></span>
          <span></span>
        </div>
      </div>
    </div>
  );
};

export default TypingIndicator;