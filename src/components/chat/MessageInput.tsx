import React, { useState, useRef } from 'react';

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
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSend = () => {
    if (!inputMessage.trim() || disabled) return;
    onSendMessage(inputMessage);
    setInputMessage('');
  };

  const insertFormatting = (before: string, after: string) => {
    const textarea = textareaRef.current;
    if (!textarea) return;
    
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = inputMessage.substring(start, end);
    const newText = 
      inputMessage.substring(0, start) + 
      before + selectedText + after + 
      inputMessage.substring(end);
    
    setInputMessage(newText);
    
    // Set cursor position
    setTimeout(() => {
      textarea.focus();
      const newPosition = start + before.length + selectedText.length;
      textarea.setSelectionRange(newPosition, newPosition);
    }, 0);
  };

  return (
    <div className="input-container">
      <div className="formatting-toolbar">
        <button
          className="format-btn"
          onClick={() => insertFormatting('**', '**')}
          title="Bold (Ctrl+B)"
          type="button"
        >
          <strong>B</strong>
        </button>
        <button
          className="format-btn"
          onClick={() => insertFormatting('*', '*')}
          title="Italic (Ctrl+I)"
          type="button"
        >
          <em>I</em>
        </button>
        <button
          className="format-btn"
          onClick={() => insertFormatting('`', '`')}
          title="Code"
          type="button"
        >
          {'</>'}
        </button>
        <button
          className="format-btn"
          onClick={() => insertFormatting('\n```\n', '\n```\n')}
          title="Code Block"
          type="button"
        >
          {'{ }'}
        </button>
        <button
          className="format-btn"
          onClick={() => insertFormatting('\n- ', '')}
          title="List"
          type="button"
        >
          ☰
        </button>
        <button
          className="format-btn"
          onClick={() => insertFormatting('> ', '')}
          title="Quote"
          type="button"
        >
          ❝
        </button>
        <button
          className="format-btn"
          onClick={() => insertFormatting('::info:: ', '')}
          title="Info Callout"
          type="button"
        >
          ℹ️
        </button>
        <button
          className="format-btn"
          onClick={() => insertFormatting('==', '==')}
          title="Highlight"
          type="button"
        >
          🖍️
        </button>
        <div className="format-separator" />
        <span className="format-hint">Markdown supported</span>
      </div>
      
      <div className="message-input-wrapper">
        <textarea
          ref={textareaRef}
          value={inputMessage}
          onChange={(e) => setInputMessage(e.target.value)}
          placeholder={`Ask ${agentName} anything... (Markdown supported)`}
          rows={3}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              handleSend();
            }
          }}
          className="message-textarea"
        />
        <button 
          onClick={handleSend} 
          disabled={!inputMessage.trim() || disabled}
          className="send-button"
        >
          {disabled ? '⏳' : 'Send'}
        </button>
      </div>
      
      <div className="input-help">
        <small>
          Shift+Enter for new line • Supports **bold**, *italic*, `code`, and more
        </small>
      </div>
    </div>
  );
};

export default MessageInput;