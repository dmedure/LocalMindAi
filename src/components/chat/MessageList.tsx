import React from 'react';
import { Message } from '../../types/message';
import MessageItem from './MessageItem';
import { EnhancedTypingIndicator } from '../common/RichMessageRender';

interface MessageListProps {
  messages: Message[];
  isLoading: boolean;
  agentName?: string;  // Added this missing prop
}

const MessageList: React.FC<MessageListProps> = ({ messages, isLoading, agentName }) => {
  return (
    <div className="messages">
      {messages.map((message) => (
        <MessageItem 
          key={message.id} 
          message={message} 
          agentName={agentName}
        />
      ))}
      {isLoading && agentName && <EnhancedTypingIndicator agentName={agentName} />}
    </div>
  );
};

export default MessageList;