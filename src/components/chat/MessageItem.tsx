import React from 'react';
import { Message } from '../../types/message';
import { formatTime } from '../../utils/formatting';

interface MessageItemProps {
  message: Message;
}

const MessageItem: React.FC<MessageItemProps> = ({ message }) => {
  return (
    <div className={`message ${message.sender}`}>
      <div className="message-content">
        {message.content}
      </div>
      <div className="message-time">
        {formatTime(message.timestamp)}
      </div>
    </div>
  );
};

export default MessageItem;