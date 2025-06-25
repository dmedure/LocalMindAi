import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Message } from '../types/message';

export function useMessages(agentId: string | undefined) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const loadMessages = useCallback(async () => {
    if (!agentId) return;
    
    try {
      const agentMessages = await invoke<Message[]>('get_agent_messages', { agentId });
      setMessages(agentMessages);
    } catch (error) {
      console.error('Failed to load messages:', error);
      setMessages([]);
    }
  }, [agentId]);

  const sendMessage = useCallback(async (content: string) => {
    if (!agentId || !content.trim()) return;

    const userMessage: Message = {
      id: crypto.randomUUID(),
      content: content.trim(),
      sender: 'user',
      timestamp: new Date().toISOString(),
      agent_id: agentId
    };

    setMessages(prev => [...prev, userMessage]);
    setIsLoading(true);

    try {
      const response = await invoke<string>('send_message_to_agent', {
        agentId,
        message: content.trim()
      });

      const agentMessage: Message = {
        id: crypto.randomUUID(),
        content: response,
        sender: 'agent',
        timestamp: new Date().toISOString(),
        agent_id: agentId
      };

      setMessages(prev => [...prev, agentMessage]);
    } catch (error) {
      console.error('Failed to send message:', error);
      const errorMessage: Message = {
        id: crypto.randomUUID(),
        content: 'Sorry, I encountered an error processing your message. Please try again.',
        sender: 'agent',
        timestamp: new Date().toISOString(),
        agent_id: agentId
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  }, [agentId]);

  useEffect(() => {
    loadMessages();
  }, [loadMessages]);

  return {
    messages,
    sendMessage,
    isLoading
  };
}