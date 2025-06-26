import React, { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';
import type { Components } from 'react-markdown';
import 'katex/dist/katex.min.css';

interface RichMessageRendererProps {
  content: string;
  sender: 'user' | 'agent';
  agentName?: string;
  responseTime?: number;
  onCodeCopy?: (code: string) => void;
}

export const RichMessageRenderer: React.FC<RichMessageRendererProps> = ({
  content,
  sender,
  agentName,
  responseTime,
  onCodeCopy
}) => {
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  const copyToClipboard = (code: string) => {
    navigator.clipboard.writeText(code);
    setCopiedCode(code);
    setTimeout(() => setCopiedCode(null), 2000);
    onCodeCopy?.(code);
  };

  const components: Components = {
    // Enhanced code blocks with syntax highlighting and copy button
    code(props) {
      const  { className, children } = props;
      const match = /language-(\w+)/.exec(className || '');
      const language = match ? match[1] : '';
      const codeString = String(children).replace(/\n$/, '');

      if (!language) {
        return (
          <div className="code-block-container">
            <div className="code-block-header">
              <span className="code-language">{language}</span>
              <button
                className="copy-button"
                onClick={() => copyToClipboard(codeString)}
                type="button"
              >
                {copiedCode === codeString ? '✓ Copied!' : '📋 Copy'}
              </button>
            </div>
            <SyntaxHighlighter
              language={language}
              style={vscDarkPlus}
              customStyle={{
                margin: 0,
                borderRadius: '0 0 8px 8px',
                fontSize: '0.875rem',
              }}
            >
              {codeString}
            </SyntaxHighlighter>
          </div>
        );
      }

      return (
        <code className="inline-code">
          {children}
        </code>
      );
    },

    // Enhanced paragraphs with better spacing
    p(props) {
      return <p className="message-paragraph">{props.children}</p>;
    },

    // Styled blockquotes
    blockquote(props) {
      return (
        <blockquote className="message-blockquote">
          <div className="blockquote-bar" />
          <div className="blockquote-content">{props.children}</div>
        </blockquote>
      );
    },

    // Enhanced lists with custom bullets
    ul(props) {
      return <ul className="message-list unordered">{props.children}</ul>;
    },

    ol(props) {
      return <ol className="message-list ordered">{props.children}</ol>;
    },

    li(props) {
      return <li className="message-list-item">{props.children}</li>;
    },

    // Tables with better styling
    table(props) {
      return (
        <div className="table-wrapper">
          <table className="message-table">{props.children}</table>
        </div>
      );
    },

    // Table components
    thead(props) {
      return <thead>{props.children}</thead>;
    },

    tbody(props) {
      return <tbody>{props.children}</tbody>;
    },

    tr(props) {
      return <tr>{props.children}</tr>;
    },

    th(props) {
      return <th>{props.children}</th>;
    },

    td(props) {
      return <td>{props.children}</td>;
    },

    // Links that open in new tab
    a(props) {
      return (
        <a
          href={props.href}
          target="_blank"
          rel="noopener noreferrer"
          className="message-link"
        >
          {props.children} ↗
        </a>
      );
    },

    // Headings with proper styling
    h1(props) {
      return <h1 className="message-heading h1">{props.children}</h1>;
    },

    h2(props) {
      return <h2 className="message-heading h2">{props.children}</h2>;
    },

    h3(props) {
      return <h3 className="message-heading h3">{props.children}</h3>;
    },

    h4(props) {
      return <h4 className="message-heading h4">{props.children}</h4>;
    },

    h5(props) {
      return <h5 className="message-heading h5">{props.children}</h5>;
    },

    h6(props) {
      return <h6 className="message-heading h6">{props.children}</h6>;
    },

    // Horizontal rules with style
    hr() {
      return <hr className="message-divider" />;
    },

    // Enhanced emphasis
    strong(props) {
      return <strong className="message-strong">{props.children}</strong>;
    },

    em(props) {
      return <em className="message-emphasis">{props.children}</em>;
    },

    // Delete and insert for showing changes
    del(props) {
      return <del className="message-deleted">{props.children}</del>;
    },

    ins(props) {
      return <ins className="message-inserted">{props.children}</ins>;
    },
  };

  // Parse special formatting patterns
  const enhancedContent = content
    // Callout boxes: ::info::, ::warning::, ::tip::, ::danger::
    .replace(/::(\w+)::\s*(.+?)(?=\n|$)/g, (match, type, text) => {
      return `<div class="callout callout-${type}"><span class="callout-icon">${getCalloutIcon(type)}</span>${text}</div>`;
    })
    // Keyboard shortcuts: ++ctrl+c++
    .replace(/\+\+(.+?)\+\+/g, '<kbd>$1</kbd>')
    // Highlighting: ==text==
    .replace(/==(.+?)==/g, '<mark>$1</mark>')
    // Task lists enhancement
    .replace(/^\s*[-*] \[x\]/gim, '- ✅')
    .replace(/^\s*[-*] \[ \]/gim, '- ⬜');

  return (
    <div className={`rich-message ${sender}`}>
      {sender === 'agent' && agentName && (
        <div className="message-header">
          <span className="agent-name">{agentName}</span>
          {responseTime && (
            <span className="response-time">{responseTime}ms</span>
          )}
        </div>
      )}
      
      <div className="message-content-wrapper markdown-content">
        <ReactMarkdown
          remarkPlugins={[remarkGfm, remarkMath]}
          rehypePlugins={[rehypeKatex]}
          components={components}
        >
          {enhancedContent}
        </ReactMarkdown>
      </div>
    </div>
  );
};

function getCalloutIcon(type: string): string {
  const icons: Record<string, string> = {
    info: 'ℹ️',
    warning: '⚠️',
    tip: '💡',
    danger: '🚨',
    success: '✅',
    note: '📝',
  };
  return icons[type] || 'ℹ️';
}

// Enhanced typing indicator component
export const EnhancedTypingIndicator: React.FC<{ agentName: string }> = ({ agentName }) => {
  return (
    <div className="typing-indicator-wrapper">
      <div className="agent-typing-label">{agentName} is thinking...</div>
      <div className="typing-dots">
        <span className="dot"></span>
        <span className="dot"></span>
        <span className="dot"></span>
      </div>
    </div>
  );
};

// Export alias for backward compatibility
export const RichMessageFormatting = RichMessageRenderer;