// frontend/src/features/livestream/hooks/useGifts.ts

import { useState, useCallback } from 'react';
import { useLivestreamStore } from '@/stores/livestream.store';
import { useWalletStore } from '@/stores/wallet.store';
import { getLivestreamService } from '../services/livestream.service';
import type { Gift, GiftAnimationItem } from '../types';

interface UseGiftsOptions {
  maxAnimations?: number;
  fullScreenThreshold?: number;
}

export function useGifts(options: UseGiftsOptions = {}) {
  const { maxAnimations = 5, fullScreenThreshold = 100 } = options;

  const { currentWallet } = useWalletStore();
  const { gifts, setGifts, addGiftRecord } = useLivestreamStore();

  const [isSending, setIsSending] = useState(false);
  const [animations, setAnimations] = useState<GiftAnimationItem[]>([]);
  const [error, setError] = useState<string | null>(null);

  // 加载礼物列表
  const loadGifts = useCallback(async () => {
    try {
      const service = getLivestreamService();
      const giftList = await service.getGifts();
      setGifts(giftList);
    } catch (err) {
      console.error('加载礼物列表失败:', err);
      setError('加载礼物列表失败');
    }
  }, [setGifts]);

  // 发送礼物
  const sendGift = useCallback(
    async (roomId: number, giftId: number, quantity: number): Promise<boolean> => {
      if (!currentWallet?.address) return false;

      setIsSending(true);
      setError(null);

      try {
        const service = getLivestreamService();
        await service.sendGift(roomId, giftId, quantity);

        // 记录礼物
        const gift = gifts.find((g) => g.id === giftId);
        if (gift) {
          addGiftRecord({
            sender: currentWallet.address,
            senderName: currentWallet.name,
            receiver: '', // 从 room 获取
            roomId,
            giftId,
            giftName: gift.name,
            quantity,
            totalValue: (Number(gift.price) * quantity).toString(),
            timestamp: Date.now(),
          });
        }

        return true;
      } catch (err) {
        console.error('发送礼物失败:', err);
        setError('发送礼物失败');
        return false;
      } finally {
        setIsSending(false);
      }
    },
    [currentWallet, gifts, addGiftRecord]
  );

  // 添加礼物动画
  const addAnimation = useCallback(
    (giftId: number, senderName: string, quantity: number) => {
      const gift = gifts.find((g) => g.id === giftId);
      if (!gift) return;

      const isFullScreen = Number(gift.price) >= fullScreenThreshold;

      const newAnimation: GiftAnimationItem = {
        id: `anim-${Date.now()}-${Math.random()}`,
        giftId,
        giftName: gift.name,
        senderName,
        quantity,
        isFullScreen,
      };

      setAnimations((prev) => {
        // 限制动画数量
        const updated = [...prev, newAnimation];
        if (updated.length > maxAnimations) {
          return updated.slice(-maxAnimations);
        }
        return updated;
      });
    },
    [gifts, fullScreenThreshold, maxAnimations]
  );

  // 移除动画
  const removeAnimation = useCallback((id: string) => {
    setAnimations((prev) => prev.filter((a) => a.id !== id));
  }, []);

  // 获取礼物图标 URL
  const getGiftIconUrl = useCallback(
    (giftId: number): string | undefined => {
      const gift = gifts.find((g) => g.id === giftId);
      if (!gift?.iconCid) return undefined;

      const gateway =
        process.env.EXPO_PUBLIC_IPFS_GATEWAY || 'https://ipfs.io/ipfs/';
      return `${gateway}${gift.iconCid}`;
    },
    [gifts]
  );

  // 计算礼物总价
  const calculateTotal = useCallback(
    (giftId: number, quantity: number): string => {
      const gift = gifts.find((g) => g.id === giftId);
      if (!gift) return '0';
      return (Number(gift.price) * quantity).toString();
    },
    [gifts]
  );

  return {
    gifts,
    isSending,
    animations,
    error,
    loadGifts,
    sendGift,
    addAnimation,
    removeAnimation,
    getGiftIconUrl,
    calculateTotal,
  };
}
