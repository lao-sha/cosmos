// frontend/src/features/livestream/hooks/useCoHost.ts

import { useState, useCallback } from 'react';
import { useLivestreamStore } from '@/stores/livestream.store';
import { useWalletStore } from '@/stores/wallet.store';
import { getLivestreamService } from '../services/livestream.service';
import { LiveKitService } from '../services/livekit.service';
import type { CoHost, CoHostRequest, DataChannelMessage } from '../types';

interface UseCoHostOptions {
  roomId: number;
  liveKitService: LiveKitService | null;
}

export function useCoHost({ roomId, liveKitService }: UseCoHostOptions) {
  const { currentWallet } = useWalletStore();
  const {
    coHosts,
    coHostRequests,
    setCoHosts,
    addCoHostRequest,
    removeCoHostRequest,
  } = useLivestreamStore();

  const [isRequesting, setIsRequesting] = useState(false);
  const [isCoHosting, setIsCoHosting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 申请连麦
  const requestCoHost = useCallback(
    async (type: 'audio' | 'video'): Promise<boolean> => {
      if (!currentWallet?.address || !liveKitService) return false;

      setIsRequesting(true);
      setError(null);

      try {
        const message: DataChannelMessage = {
          type: 'cohost_request',
          payload: { type },
          sender: currentWallet.address,
          senderName: currentWallet.name || currentWallet.address.slice(0, 8),
          timestamp: Date.now(),
        };

        await liveKitService.sendData(message);
        return true;
      } catch (err) {
        console.error('申请连麦失败:', err);
        setError('申请连麦失败');
        return false;
      } finally {
        setIsRequesting(false);
      }
    },
    [currentWallet, liveKitService]
  );

  // 接受连麦 (主播)
  const acceptCoHost = useCallback(
    async (address: string): Promise<boolean> => {
      if (!currentWallet?.address) return false;

      try {
        const service = getLivestreamService();
        await service.startCoHost(roomId, address);

        // 发送接受通知
        if (liveKitService) {
          const message: DataChannelMessage = {
            type: 'cohost_accept',
            payload: { address },
            sender: currentWallet.address,
            timestamp: Date.now(),
          };
          await liveKitService.sendData(message);
        }

        removeCoHostRequest(address);
        return true;
      } catch (err) {
        console.error('接受连麦失败:', err);
        setError('接受连麦失败');
        return false;
      }
    },
    [currentWallet, roomId, liveKitService, removeCoHostRequest]
  );

  // 拒绝连麦 (主播)
  const rejectCoHost = useCallback(
    async (address: string): Promise<void> => {
      if (!currentWallet?.address) return;

      // 发送拒绝通知
      if (liveKitService) {
        const message: DataChannelMessage = {
          type: 'cohost_reject',
          payload: { address },
          sender: currentWallet.address,
          timestamp: Date.now(),
        };
        await liveKitService.sendData(message);
      }

      removeCoHostRequest(address);
    },
    [currentWallet, liveKitService, removeCoHostRequest]
  );

  // 结束连麦
  const endCoHost = useCallback(
    async (address?: string): Promise<boolean> => {
      if (!currentWallet?.address) return false;

      try {
        const service = getLivestreamService();
        await service.endCoHost(roomId, address);

        // 发送结束通知
        if (liveKitService) {
          const message: DataChannelMessage = {
            type: 'cohost_end',
            payload: { address: address || currentWallet.address },
            sender: currentWallet.address,
            timestamp: Date.now(),
          };
          await liveKitService.sendData(message);
        }

        setIsCoHosting(false);
        return true;
      } catch (err) {
        console.error('结束连麦失败:', err);
        setError('结束连麦失败');
        return false;
      }
    },
    [currentWallet, roomId, liveKitService]
  );

  // 开始连麦推流 (被接受后)
  const startCoHostStream = useCallback(async (): Promise<boolean> => {
    if (!liveKitService) return false;

    try {
      await liveKitService.enableCameraAndMicrophone();
      setIsCoHosting(true);
      return true;
    } catch (err) {
      console.error('开启连麦推流失败:', err);
      setError('开启连麦推流失败');
      return false;
    }
  }, [liveKitService]);

  // 处理连麦相关消息
  const handleCoHostMessage = useCallback(
    (message: DataChannelMessage) => {
      switch (message.type) {
        case 'cohost_request':
          const requestPayload = message.payload as { type: 'audio' | 'video' };
          addCoHostRequest({
            address: message.sender,
            name: message.senderName,
            type: requestPayload.type,
            timestamp: message.timestamp,
          });
          break;

        case 'cohost_accept':
          const acceptPayload = message.payload as { address: string };
          if (acceptPayload.address === currentWallet?.address) {
            // 自己被接受，开始推流
            startCoHostStream();
          }
          break;

        case 'cohost_reject':
          const rejectPayload = message.payload as { address: string };
          if (rejectPayload.address === currentWallet?.address) {
            setError('连麦申请被拒绝');
          }
          break;

        case 'cohost_end':
          const endPayload = message.payload as { address: string };
          if (endPayload.address === currentWallet?.address) {
            setIsCoHosting(false);
          }
          // 更新连麦者列表
          setCoHosts(coHosts.filter((c) => c.address !== endPayload.address));
          break;
      }
    },
    [currentWallet, addCoHostRequest, startCoHostStream, setCoHosts, coHosts]
  );

  return {
    coHosts,
    coHostRequests,
    isRequesting,
    isCoHosting,
    error,
    requestCoHost,
    acceptCoHost,
    rejectCoHost,
    endCoHost,
    startCoHostStream,
    handleCoHostMessage,
  };
}
