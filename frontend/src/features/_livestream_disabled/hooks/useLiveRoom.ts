// frontend/src/features/livestream/hooks/useLiveRoom.ts

import { useState, useCallback, useEffect } from 'react';
import { useLivestreamStore } from '@/stores/livestream.store';
import { useWalletStore } from '@/stores/wallet.store';
import {
  getLivestreamService,
  initLivestreamService,
} from '../services/livestream.service';
import type { LiveRoom, RoomFilter } from '../types';

export function useLiveRoom() {
  const { currentWallet } = useWalletStore();
  const {
    rooms,
    isLoadingRooms,
    roomFilter,
    currentRoom,
    setRooms,
    setIsLoadingRooms,
    setRoomFilter,
    setCurrentRoom,
  } = useLivestreamStore();

  const [error, setError] = useState<string | null>(null);

  // 加载直播间列表
  const loadRooms = useCallback(
    async (filter?: RoomFilter) => {
      if (!currentWallet?.address) return;

      setIsLoadingRooms(true);
      setError(null);

      try {
        const service = initLivestreamService(currentWallet.address);
        await service.init();

        const filterValue = filter || roomFilter;
        const data = await service.getLiveRooms(
          filterValue === 'all' ? undefined : filterValue
        );
        setRooms(data);
      } catch (err) {
        console.error('加载直播列表失败:', err);
        setError('加载直播列表失败');
      } finally {
        setIsLoadingRooms(false);
      }
    },
    [currentWallet?.address, roomFilter, setRooms, setIsLoadingRooms]
  );

  // 获取直播间详情
  const getRoomInfo = useCallback(
    async (roomId: number): Promise<LiveRoom | null> => {
      if (!currentWallet?.address) return null;

      try {
        const service = initLivestreamService(currentWallet.address);
        await service.init();

        const room = await service.getRoomInfo(roomId);
        setCurrentRoom(room);
        return room;
      } catch (err) {
        console.error('获取直播间详情失败:', err);
        setError('获取直播间详情失败');
        return null;
      }
    },
    [currentWallet?.address, setCurrentRoom]
  );

  // 检查门票
  const checkTicket = useCallback(
    async (roomId: number): Promise<boolean> => {
      if (!currentWallet?.address) return false;

      try {
        const service = getLivestreamService();
        return await service.checkTicket(roomId);
      } catch (err) {
        console.error('检查门票失败:', err);
        return false;
      }
    },
    [currentWallet?.address]
  );

  // 购买门票
  const buyTicket = useCallback(
    async (roomId: number): Promise<boolean> => {
      if (!currentWallet?.address) return false;

      try {
        const service = getLivestreamService();
        await service.buyTicket(roomId);
        return true;
      } catch (err) {
        console.error('购买门票失败:', err);
        setError('购买门票失败');
        return false;
      }
    },
    [currentWallet?.address]
  );

  // 切换筛选
  const changeFilter = useCallback(
    (filter: RoomFilter) => {
      setRoomFilter(filter);
    },
    [setRoomFilter]
  );

  // 刷新列表
  const refresh = useCallback(() => {
    return loadRooms();
  }, [loadRooms]);

  return {
    rooms,
    isLoadingRooms,
    roomFilter,
    currentRoom,
    error,
    loadRooms,
    getRoomInfo,
    checkTicket,
    buyTicket,
    changeFilter,
    refresh,
  };
}
