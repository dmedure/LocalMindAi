import React from 'react';
import { Agent } from '../../types/agent';
import { Message } from '../../types/message';
import { getAgentIntroduction } from '../../utils/formatting';
import MessageList from './MessageList';
import MessageInput from './MessageInput';

interface ChatContainerProps {
  agent: Agent;
  messages: Message[];
  isLoading: boolean;
  onSendMessage: (content: string) => Promise<void>;
}

const ChatContainer: React.FC<ChatContainerProps> = ({
  agent,
  messages,
  isLoading,
  onSendMessage
}) => {
  const handleSendMessage = async (content: string) => {
    try {
      await onSendMessage(content);
    } catch (error) {
      console.error('Failed to send message:', error);
      // You could add error handling/notification here
    }
  };

  return (
    <div className="chat-container">
      <div className="agent-intro">
        <h3>👋 Hello! I'm {agent.name}</h3>
        <p>{getAgentIntroduction(agent)}</p>
        {agent.instructions && (
          <div className="custom-instructions">
            <strong>Special Focus:</strong> {agent.instructions}
          </div>
        )}
        
        <div className="formatting-preview">
          <details>
            <summary>✨ Rich formatting available</summary>
            <div className="formatting-examples">
              <p><strong>Bold text</strong>, <em>italic text</em>, <code>inline code</code></p>
              <p>📝 Lists, 📊 tables, 💡 callouts, and much more!</p>
            </div>
          </details>
        </div>
      </div>

      <MessageList 
        messages={messages} 
        isLoading={isLoading} 
        agentName={agent.name}
      />
      
      <MessageInput
        agentName={agent.name}
        onSendMessage={handleSendMessage}
        disabled={isLoading}
      />
    </div>
  );
};

export default ChatContainer;