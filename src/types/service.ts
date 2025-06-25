export type ServiceStatus = 'online' | 'offline' | 'unknown';

export interface ServiceHealth {
  ollama: ServiceStatus;
  chromadb: ServiceStatus;
}