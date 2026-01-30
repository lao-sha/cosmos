/**
 * Meowstar Universe - AI 陪伴 React Hooks
 */

import { useState, useCallback, useEffect } from 'react';
import {
  getAICompanionService,
  ChatMessage,
  PetEmotion,
  PetPersonality,
  AIResponse,
} from '../services/aiCompanion';
import type { Pet } from '../services/meowstar';

export function useAIChat(pet: Pet | null) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentEmotion, setCurrentEmotion] = useState<PetEmotion>({
    primary: 'neutral',
    intensity: 50,
  });
  const [personality, setPersonality] = useState<PetPersonality | null>(null);

  // 初始化
  useEffect(() => {
    if (pet) {
      const service = getAICompanionService();
      const history = service.getConversationHistory(pet.id);
      setMessages(history);
      setCurrentEmotion(service.getEmotion(pet.id));
      setPersonality(service.generatePersonality(pet));
    }
  }, [pet?.id]);

  // 发送消息
  const sendMessage = useCallback(
    async (content: string): Promise<AIResponse | null> => {
      if (!pet || !personality) return null;

      setIsLoading(true);
      setError(null);

      try {
        const service = getAICompanionService();
        const response = await service.chat(pet, content, personality);

        // 更新消息列表
        const userMsg: ChatMessage = {
          id: `${Date.now()}-user`,
          role: 'user',
          content,
          timestamp: Date.now(),
        };

        const assistantMsg: ChatMessage = {
          id: `${Date.now()}-assistant`,
          role: 'assistant',
          content: response.message,
          timestamp: Date.now(),
          emotion: response.emotion,
        };

        setMessages((prev) => [...prev, userMsg, assistantMsg]);
        setCurrentEmotion(response.emotion);

        return response;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : '发送失败';
        setError(errorMessage);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [pet, personality]
  );

  // 清除历史
  const clearHistory = useCallback(() => {
    if (pet) {
      const service = getAICompanionService();
      service.clearConversationHistory(pet.id);
      setMessages([]);
    }
  }, [pet?.id]);

  // 更新情绪
  const triggerEmotionEvent = useCallback(
    (event: string) => {
      if (pet) {
        const service = getAICompanionService();
        const newEmotion = service.updateEmotion(pet.id, event);
        setCurrentEmotion(newEmotion);
        return newEmotion;
      }
      return null;
    },
    [pet?.id]
  );

  return {
    messages,
    isLoading,
    error,
    currentEmotion,
    personality,
    sendMessage,
    clearHistory,
    triggerEmotionEvent,
  };
}

export function usePetEmotion(petId: number | undefined) {
  const [emotion, setEmotion] = useState<PetEmotion>({
    primary: 'neutral',
    intensity: 50,
  });

  useEffect(() => {
    if (petId !== undefined) {
      const service = getAICompanionService();
      setEmotion(service.getEmotion(petId));
    }
  }, [petId]);

  const updateEmotion = useCallback(
    (event: string) => {
      if (petId !== undefined) {
        const service = getAICompanionService();
        const newEmotion = service.updateEmotion(petId, event);
        setEmotion(newEmotion);
        return newEmotion;
      }
      return null;
    },
    [petId]
  );

  return { emotion, updateEmotion };
}

// 情绪显示配置
export const EMOTION_CONFIG: Record<
  string,
  { emoji: string; label: string; color: string }
> = {
  happy: { emoji: '😊', label: '开心', color: '#F7DC6F' },
  excited: { emoji: '🎉', label: '兴奋', color: '#FF6B6B' },
  curious: { emoji: '🤔', label: '好奇', color: '#45B7D1' },
  caring: { emoji: '💕', label: '关心', color: '#FF69B4' },
  playful: { emoji: '😸', label: '调皮', color: '#4ECDC4' },
  sleepy: { emoji: '😴', label: '困倦', color: '#BB8FCE' },
  hungry: { emoji: '🍖', label: '饥饿', color: '#F39C12' },
  sad: { emoji: '😢', label: '难过', color: '#5DADE2' },
  anxious: { emoji: '😰', label: '焦虑', color: '#E74C3C' },
  neutral: { emoji: '😐', label: '平静', color: '#888' },
};
