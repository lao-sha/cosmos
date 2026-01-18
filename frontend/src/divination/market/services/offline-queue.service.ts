// frontend/src/divination/market/services/offline-queue.service.ts

import storage from '@/lib/storage';

const OFFLINE_QUEUE_KEY = 'market_offline_queue';

export interface OfflineAction {
  id: string;
  type: 'createOrder' | 'submitReview' | 'followUp';
  payload: any;
  createdAt: number;
  retryCount: number;
  maxRetries: number;
}

/**
 * 离线队列服务
 * 用于在网络断开时缓存操作，网络恢复后自动重试
 */
class OfflineQueueService {
  private queue: OfflineAction[] = [];
  private isProcessing = false;

  constructor() {
    this.loadQueue();
  }

  /**
   * 从本地存储加载队列
   */
  private async loadQueue() {
    try {
      const data = await storage.getItem(OFFLINE_QUEUE_KEY);
      if (data) {
        this.queue = JSON.parse(data);
      }
    } catch (err) {
      console.error('Load offline queue error:', err);
    }
  }

  /**
   * 保存队列到本地存储
   */
  private async saveQueue() {
    try {
      await storage.setItem(OFFLINE_QUEUE_KEY, JSON.stringify(this.queue));
    } catch (err) {
      console.error('Save offline queue error:', err);
    }
  }

  /**
   * 添加操作到队列
   */
  async addAction(
    type: OfflineAction['type'],
    payload: any,
    maxRetries: number = 3
  ): Promise<string> {
    const action: OfflineAction = {
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type,
      payload,
      createdAt: Date.now(),
      retryCount: 0,
      maxRetries,
    };

    this.queue.push(action);
    await this.saveQueue();

    return action.id;
  }

  /**
   * 从队列中移除操作
   */
  async removeAction(id: string) {
    this.queue = this.queue.filter((a) => a.id !== id);
    await this.saveQueue();
  }

  /**
   * 获取队列中的所有操作
   */
  getQueue(): OfflineAction[] {
    return [...this.queue];
  }

  /**
   * 获取队列长度
   */
  getQueueLength(): number {
    return this.queue.length;
  }

  /**
   * 处理队列中的操作
   */
  async processQueue(
    executor: (action: OfflineAction) => Promise<boolean>
  ): Promise<{ success: number; failed: number }> {
    if (this.isProcessing) {
      return { success: 0, failed: 0 };
    }

    this.isProcessing = true;
    let success = 0;
    let failed = 0;

    const actionsToProcess = [...this.queue];

    for (const action of actionsToProcess) {
      try {
        const result = await executor(action);

        if (result) {
          await this.removeAction(action.id);
          success++;
        } else {
          action.retryCount++;
          if (action.retryCount >= action.maxRetries) {
            await this.removeAction(action.id);
            failed++;
          } else {
            await this.saveQueue();
          }
        }
      } catch (err) {
        console.error('Process action error:', err);
        action.retryCount++;
        if (action.retryCount >= action.maxRetries) {
          await this.removeAction(action.id);
          failed++;
        } else {
          await this.saveQueue();
        }
      }
    }

    this.isProcessing = false;
    return { success, failed };
  }

  /**
   * 清空队列
   */
  async clearQueue() {
    this.queue = [];
    await storage.removeItem(OFFLINE_QUEUE_KEY);
  }
}

export const offlineQueueService = new OfflineQueueService();
