import React, { useState } from 'react';
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
      </div>

      <MessageList messages={messages} isLoading={isLoading} />
      <MessageInput
        agentName={agent.name}
        onSendMessage={onSendMessage}
        disabled={isLoading}
      />
    </div>
  );
};

export default ChatContainer;