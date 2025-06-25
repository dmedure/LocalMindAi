export interface Message {
  id: string;
  content: string;
  sender: 'user' | 'agent';
  timestamp: string;
  agent_id: string;
}