/**
 * useBlockTime Hook
 * 获取当前区块高度和出块时间，用于将区块高度转换为实际时间
 */

import { useState, useEffect } from 'react';
import { getApi } from '@/api';

interface BlockTimeState {
  /** 当前区块高度 */
  currentBlock: number;
  /** 出块时间（毫秒） */
  blockTime: number;
  /** 是否已加载 */
  isLoaded: boolean;
}

/**
 * 获取当前区块高度和出块时间
 */
export function useBlockTime(): BlockTimeState {
  const [state, setState] = useState<BlockTimeState>({
    currentBlock: 0,
    blockTime: 6000, // 默认 6 秒
    isLoaded: false,
  });

  useEffect(() => {
    let unsubscribe: (() => void) | null = null;
    let mounted = true;

    const init = async () => {
      try {
        const api = await getApi();

        // 获取出块时间（从链配置）
        const expectedBlockTime =
          (api.consts.babe as any)?.expectedBlockTime ||
          (api.consts.timestamp as any)?.minimumPeriod?.muln(2);
        const blockTime = expectedBlockTime
          ? expectedBlockTime.toNumber()
          : 6000;

        // 订阅最新区块
        const unsub = await api.rpc.chain.subscribeNewHeads((header) => {
          if (mounted) {
            setState({
              currentBlock: header.number.toNumber(),
              blockTime,
              isLoaded: true,
            });
          }
        });

        unsubscribe = unsub as unknown as () => void;
      } catch (error) {
        console.error('Failed to subscribe to block headers:', error);
        if (mounted) {
          setState((prev) => ({ ...prev, isLoaded: true }));
        }
      }
    };

    init();

    return () => {
      mounted = false;
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, []);

  return state;
}

/**
 * 区块时间工具函数
 */
export const BlockTimeUtils = {
  /**
   * 将区块高度差转换为毫秒
   */
  blocksToMs(blocks: number, blockTime: number = 6000): number {
    return blocks * blockTime;
  },

  /**
   * 将毫秒转换为区块数
   */
  msToBlocks(ms: number, blockTime: number = 6000): number {
    return Math.floor(ms / blockTime);
  },

  /**
   * 计算目标区块对应的时间戳
   */
  blockToTimestamp(
    targetBlock: number,
    currentBlock: number,
    blockTime: number = 6000
  ): number {
    const blockDiff = currentBlock - targetBlock;
    return Date.now() - blockDiff * blockTime;
  },

  /**
   * 格式化区块时间为相对时间
   */
  formatRelative(
    targetBlock: number,
    currentBlock: number,
    blockTime: number = 6000
  ): string {
    const timeDiff = (currentBlock - targetBlock) * blockTime;

    if (timeDiff < 0) return '刚刚';
    if (timeDiff < 60000) return '刚刚';
    if (timeDiff < 3600000) return `${Math.floor(timeDiff / 60000)}分钟前`;
    if (timeDiff < 86400000) return `${Math.floor(timeDiff / 3600000)}小时前`;
    if (timeDiff < 604800000) return `${Math.floor(timeDiff / 86400000)}天前`;

    const date = new Date(Date.now() - timeDiff);
    return `${date.getMonth() + 1}/${date.getDate()}`;
  },

  /**
   * 格式化区块时间为 HH:mm
   */
  formatTime(
    targetBlock: number,
    currentBlock: number,
    blockTime: number = 6000
  ): string {
    const timestamp = BlockTimeUtils.blockToTimestamp(
      targetBlock,
      currentBlock,
      blockTime
    );
    const date = new Date(timestamp);
    return `${date.getHours().toString().padStart(2, '0')}:${date
      .getMinutes()
      .toString()
      .padStart(2, '0')}`;
  },

  /**
   * 格式化区块时间为完整日期时间
   */
  formatDateTime(
    targetBlock: number,
    currentBlock: number,
    blockTime: number = 6000
  ): string {
    const timestamp = BlockTimeUtils.blockToTimestamp(
      targetBlock,
      currentBlock,
      blockTime
    );
    const date = new Date(timestamp);
    return `${date.getFullYear()}/${date.getMonth() + 1}/${date.getDate()} ${date
      .getHours()
      .toString()
      .padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  },
};
