import React, { useState } from 'react';

interface MessageInputProps {
  agentName: string;
  onSendMessage: (content: string) => void;
  disabled: boolean;
}

const MessageInput: React.FC<MessageInputProps> = ({
  agentName,
  onSendMessage,
  disabled
}) => {
  const [inputMessage, setInputMessage] = useState('');

  const handleSend = () => {
    if (!inputMessage.trim() || disabled) return;
    onSendMessage(inputMessage);
    setInputMessage('');
  };

  return (
    <div className="input-container">
      <textarea
        value={inputMessage}
        onChange={(e) => setInputMessage(e.target.value)}
        placeholder={`Ask ${agentName} anything...`}
        rows={2}
        onKeyPress={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSend();
          }
        }}
      />
      <button 
        onClick={handleSend} 
        disabled={!inputMessage.trim() || disabled}
      >
        Send
      </button>
    </div>
  );
};

export default MessageInput;