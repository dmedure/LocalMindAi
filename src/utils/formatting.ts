export function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function getAgentIcon(specialization: string): string {
  const icons: Record<string, string> = {
    work: '💼',
    coding: '💻',
    research: '🔬',
    writing: '📝',
    personal: '👤',
    creative: '🎨',
    technical: '🔧',
    general: '🤖'
  };
  return icons[specialization] || '🤖';
}

export function getPersonalityIcon(personality: string): string {
  const icons: Record<string, string> = {
    professional: '💼',
    friendly: '😊',
    analytical: '🔍',
    creative: '🎨',
    concise: '⚡',
    detailed: '📝'
  };
  return icons[personality] || '😊';
}

export function getAgentIntroduction(agent: { specialization: string }): string {
  const introductions: Record<string, string> = {
    work: `I'm your professional assistant, specialized in handling work tasks, project management, and business communications. I have access to your work documents and can help you stay organized and productive.`,
    coding: `I'm your coding companion! I specialize in programming, code review, debugging, and technical documentation. I can help you with any development challenges you're facing.`,
    research: `I'm your research assistant, focused on academic work, data analysis, and in-depth investigation. I can help you find information, analyze data, and organize your research.`,
    writing: `I'm your writing partner! I specialize in content creation, editing, brainstorming, and helping you communicate effectively through written word.`,
    personal: `I'm your personal assistant, here to help with daily tasks, organization, scheduling, and anything else you need to manage your personal life.`,
    creative: `I'm your creative companion! I specialize in brainstorming, artistic projects, design thinking, and helping you explore your creative potential.`,
    technical: `I'm your technical support specialist, focused on troubleshooting, system administration, and helping you with technical challenges.`,
    general: `I'm your general assistant, ready to help with a wide variety of tasks and questions. I adapt to your needs and learn from our conversations.`
  };

  return introductions[agent.specialization] || introductions.general;
}