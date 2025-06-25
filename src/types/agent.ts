export interface Agent {
  id: string;
  name: string;
  specialization: string;
  personality: string;
  instructions?: string;
  created_at: string;
}

export type AgentSpecialization = 
  | 'general' 
  | 'work' 
  | 'coding' 
  | 'research' 
  | 'writing' 
  | 'personal' 
  | 'creative' 
  | 'technical';

export type AgentPersonality = 
  | 'professional' 
  | 'friendly' 
  | 'analytical' 
  | 'creative' 
  | 'concise' 
  | 'detailed';