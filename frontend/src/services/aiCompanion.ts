/**
 * Meowstar Universe - AI 陪伴服务
 * 与 AI 后端交互，提供宠物聊天和情感分析功能
 */

import type { Pet } from './meowstar';

// ============ 类型定义 ============

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  emotion?: PetEmotion;
  metadata?: {
    tokensUsed?: number;
    responseTime?: number;
  };
}

export interface PetEmotion {
  primary: EmotionType;
  intensity: number; // 0-100
  secondary?: EmotionType;
}

export type EmotionType =
  | 'happy'
  | 'excited'
  | 'curious'
  | 'caring'
  | 'playful'
  | 'sleepy'
  | 'hungry'
  | 'sad'
  | 'anxious'
  | 'neutral';

export interface PetPersonality {
  extroversion: number;    // 0-100 外向程度
  warmth: number;          // 0-100 温暖程度
  humor: number;           // 0-100 幽默感
  curiosity: number;       // 0-100 好奇心
  responsibility: number;  // 0-100 责任感
}

export interface ConversationContext {
  petId: number;
  petName: string;
  petElement: string;
  petLevel: number;
  personality: PetPersonality;
  recentMessages: ChatMessage[];
  currentEmotion: PetEmotion;
}

export interface AIResponse {
  message: string;
  emotion: PetEmotion;
  suggestedActions?: string[];
  memoryUpdate?: {
    key: string;
    value: string;
  };
}

// ============ 配置 ============

const AI_API_BASE_URL = process.env.EXPO_PUBLIC_AI_API_URL || 'http://localhost:8000';
const AI_API_TIMEOUT = 30000;

// ============ 服务类 ============

export class AICompanionService {
  private apiUrl: string;
  private conversationHistory: Map<number, ChatMessage[]> = new Map();
  private petEmotions: Map<number, PetEmotion> = new Map();

  constructor(apiUrl: string = AI_API_BASE_URL) {
    this.apiUrl = apiUrl;
  }

  /**
   * 发送消息给宠物 AI
   */
  async chat(
    pet: Pet,
    userMessage: string,
    personality: PetPersonality
  ): Promise<AIResponse> {
    const history = this.conversationHistory.get(pet.id) || [];
    const currentEmotion = this.petEmotions.get(pet.id) || {
      primary: 'neutral' as EmotionType,
      intensity: 50,
    };

    const context: ConversationContext = {
      petId: pet.id,
      petName: pet.name,
      petElement: pet.element,
      petLevel: pet.level,
      personality,
      recentMessages: history.slice(-10),
      currentEmotion,
    };

    try {
      const response = await this.callAI('/chat', {
        context,
        message: userMessage,
      });

      // 更新对话历史
      const userMsg: ChatMessage = {
        id: this.generateId(),
        role: 'user',
        content: userMessage,
        timestamp: Date.now(),
      };

      const assistantMsg: ChatMessage = {
        id: this.generateId(),
        role: 'assistant',
        content: response.message,
        timestamp: Date.now(),
        emotion: response.emotion,
      };

      const updatedHistory = [...history, userMsg, assistantMsg].slice(-50);
      this.conversationHistory.set(pet.id, updatedHistory);
      this.petEmotions.set(pet.id, response.emotion);

      return response;
    } catch (error) {
      // 离线模式：使用本地响应生成
      return this.generateLocalResponse(pet, userMessage, personality, currentEmotion);
    }
  }

  /**
   * 获取宠物当前情绪
   */
  getEmotion(petId: number): PetEmotion {
    return this.petEmotions.get(petId) || {
      primary: 'neutral',
      intensity: 50,
    };
  }

  /**
   * 更新宠物情绪（基于事件）
   */
  updateEmotion(petId: number, event: string): PetEmotion {
    const currentEmotion = this.getEmotion(petId);
    let newEmotion: PetEmotion;

    switch (event) {
      case 'battle_win':
        newEmotion = { primary: 'excited', intensity: 90 };
        break;
      case 'battle_lose':
        newEmotion = { primary: 'sad', intensity: 60 };
        break;
      case 'level_up':
        newEmotion = { primary: 'happy', intensity: 85 };
        break;
      case 'evolve':
        newEmotion = { primary: 'excited', intensity: 100 };
        break;
      case 'feed':
        newEmotion = { primary: 'happy', intensity: 70 };
        break;
      case 'play':
        newEmotion = { primary: 'playful', intensity: 80 };
        break;
      case 'rest':
        newEmotion = { primary: 'sleepy', intensity: 60 };
        break;
      default:
        newEmotion = currentEmotion;
    }

    this.petEmotions.set(petId, newEmotion);
    return newEmotion;
  }

  /**
   * 获取对话历史
   */
  getConversationHistory(petId: number): ChatMessage[] {
    return this.conversationHistory.get(petId) || [];
  }

  /**
   * 清除对话历史
   */
  clearConversationHistory(petId: number): void {
    this.conversationHistory.delete(petId);
  }

  /**
   * 生成宠物性格
   */
  generatePersonality(pet: Pet): PetPersonality {
    // 基于宠物属性生成性格
    const seed = pet.id * 1000 + pet.level;
    const random = this.seededRandom(seed);

    const basePersonality: PetPersonality = {
      extroversion: 50,
      warmth: 50,
      humor: 50,
      curiosity: 50,
      responsibility: 50,
    };

    // 元素影响性格
    switch (pet.element) {
      case 'Fire':
        basePersonality.extroversion += 20;
        basePersonality.humor += 10;
        break;
      case 'Water':
        basePersonality.warmth += 20;
        basePersonality.responsibility += 10;
        break;
      case 'Light':
        basePersonality.warmth += 15;
        basePersonality.curiosity += 15;
        break;
      case 'Shadow':
        basePersonality.curiosity += 20;
        basePersonality.extroversion -= 10;
        break;
    }

    // 稀有度影响
    const rarityBonus = {
      Common: 0,
      Rare: 5,
      Epic: 10,
      Legendary: 15,
      Mythic: 20,
    };
    const bonus = rarityBonus[pet.rarity] || 0;
    basePersonality.curiosity += bonus;

    // 添加随机变化
    Object.keys(basePersonality).forEach((key) => {
      const k = key as keyof PetPersonality;
      basePersonality[k] = Math.min(100, Math.max(0, basePersonality[k] + (random() * 20 - 10)));
    });

    return basePersonality;
  }

  // ============ 私有方法 ============

  private async callAI(endpoint: string, data: any): Promise<AIResponse> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), AI_API_TIMEOUT);

    try {
      const response = await fetch(`${this.apiUrl}${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`AI API error: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      throw error;
    }
  }

  private generateLocalResponse(
    pet: Pet,
    userMessage: string,
    personality: PetPersonality,
    currentEmotion: PetEmotion
  ): AIResponse {
    // 本地响应模板
    const responses = {
      greeting: [
        `喵~ 主人好呀！${pet.name}很高兴见到你！🐱`,
        `嗨嗨！主人今天看起来精神不错呢~ ✨`,
        `喵呜~ ${pet.name}一直在等主人呢！💕`,
      ],
      question: [
        `嗯...让${pet.name}想想喵~ 🤔`,
        `这个问题好有趣呀！${pet.name}觉得...`,
        `喵？主人想知道什么呢~`,
      ],
      emotion_happy: [
        `太棒了喵！${pet.name}也很开心！🎉`,
        `喵喵~ 主人开心${pet.name}也开心！💖`,
        `耶！这真是太好了喵~ ✨`,
      ],
      emotion_sad: [
        `喵...主人不要难过，${pet.name}会一直陪着你的 💕`,
        `${pet.name}给主人一个大大的拥抱喵~ 🤗`,
        `没关系的喵，明天会更好的！`,
      ],
      default: [
        `喵喵~ ${pet.name}明白了！`,
        `原来是这样呀喵~`,
        `嗯嗯，${pet.name}在听呢！`,
        `喵~ 主人说的真有趣！`,
      ],
    };

    // 简单的意图识别
    const lowerMessage = userMessage.toLowerCase();
    let category = 'default';
    let newEmotion: PetEmotion = { primary: 'happy', intensity: 70 };

    if (lowerMessage.includes('你好') || lowerMessage.includes('嗨') || lowerMessage.includes('hi')) {
      category = 'greeting';
      newEmotion = { primary: 'excited', intensity: 80 };
    } else if (lowerMessage.includes('?') || lowerMessage.includes('？') || lowerMessage.includes('什么') || lowerMessage.includes('怎么')) {
      category = 'question';
      newEmotion = { primary: 'curious', intensity: 75 };
    } else if (lowerMessage.includes('开心') || lowerMessage.includes('高兴') || lowerMessage.includes('棒')) {
      category = 'emotion_happy';
      newEmotion = { primary: 'happy', intensity: 90 };
    } else if (lowerMessage.includes('难过') || lowerMessage.includes('伤心') || lowerMessage.includes('累')) {
      category = 'emotion_sad';
      newEmotion = { primary: 'caring', intensity: 85 };
    }

    const categoryResponses = responses[category as keyof typeof responses];
    const randomIndex = Math.floor(Math.random() * categoryResponses.length);

    return {
      message: categoryResponses[randomIndex],
      emotion: newEmotion,
    };
  }

  private generateId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private seededRandom(seed: number): () => number {
    return () => {
      seed = (seed * 9301 + 49297) % 233280;
      return seed / 233280;
    };
  }
}

// ============ 单例导出 ============

let aiCompanionService: AICompanionService | null = null;

export function initAICompanionService(apiUrl?: string): AICompanionService {
  aiCompanionService = new AICompanionService(apiUrl);
  return aiCompanionService;
}

export function getAICompanionService(): AICompanionService {
  if (!aiCompanionService) {
    aiCompanionService = new AICompanionService();
  }
  return aiCompanionService;
}
