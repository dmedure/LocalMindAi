import React from 'react';
import { Message } from '../../types/message';
import MessageItem from './MessageItem';
import TypingIndicator from './TypingIndicator';

interface MessageListProps {
  messages: Message[];
  isLoading: boolean;
}

const MessageList: React.FC<MessageListProps> = ({ messages, isLoading }) => {
  return (
    <div className="messages">
      {messages.map((message) => (
        <MessageItem key={message.id} message={message} />
      ))}
      {isLoading && <TypingIndicator />}
    </div>
  );
};

export default MessageList;