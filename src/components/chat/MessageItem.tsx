import React from 'react';
import { Message } from '../../types/message';
import { formatTime } from '../../utils/formatting';
import { RichMessageRenderer } from '../common/RichMessageRender';

interface MessageItemProps {
  message: Message;
  agentName?: string;  // Make sure this prop is included
}

const MessageItem: React.FC<MessageItemProps> = ({ message, agentName }) => {
  const handleCodeCopy = (code: string) => {
    // You can add a toast notification here if you implement one
    console.log('Code copied to clipboard');
  };

  return (
    <RichMessageRenderer
      content={message.content}
      sender={message.sender as 'user' | 'agent'}
      agentName={message.sender === 'agent' ? agentName : undefined}
      onCodeCopy={handleCodeCopy}
    />
  );
};

export default MessageItem;