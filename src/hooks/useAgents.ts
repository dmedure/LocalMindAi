import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Agent } from '../types/agent';

export function useAgents() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [currentAgent, setCurrentAgent] = useState<Agent | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadAgents = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const agentList = await invoke<Agent[]>('get_agents');
      setAgents(agentList);
      
      if (agentList.length > 0 && !currentAgent) {
        setCurrentAgent(agentList[0]);
      }
      
      return agentList;
    } catch (err) {
      setError('Failed to load agents');
      console.error('Failed to load agents:', err);
      return [];
    } finally {
      setIsLoading(false);
    }
  }, [currentAgent]);

  const createAgent = useCallback(async (agent: Agent) => {
    try {
      await invoke('create_agent', { agent });
      const agentList = await loadAgents();
      const newAgent = agentList.find(a => a.id === agent.id);
      if (newAgent) {
        setCurrentAgent(newAgent);
      }
      return true;
    } catch (err) {
      console.error('Failed to create agent:', err);
      throw err;
    }
  }, [loadAgents]);

  useEffect(() => {
    loadAgents();
  }, []);

  return {
    agents,
    currentAgent,
    setCurrentAgent,
    createAgent,
    loadAgents,
    isLoading,
    error
  };
}