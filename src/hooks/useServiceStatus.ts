import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { ServiceStatus } from '../types/service';

export function useServiceStatus() {
  const [ollamaStatus, setOllamaStatus] = useState<ServiceStatus>('unknown');
  const [chromaStatus, setChromaStatus] = useState<ServiceStatus>('unknown');

  const checkServiceStatus = async () => {
    try {
      const status = await invoke<{ollama: boolean, chromadb: boolean}>('check_service_status');
      setOllamaStatus(status.ollama ? 'online' : 'offline');
      setChromaStatus(status.chromadb ? 'online' : 'offline');
    } catch (error) {
      console.error('Failed to check service status:', error);
    }
  };

  useEffect(() => {
    checkServiceStatus();
    const interval = setInterval(checkServiceStatus, 30000); // Check every 30 seconds
    return () => clearInterval(interval);
  }, []);

  return { ollamaStatus, chromaStatus };
}