// frontend/src/divination/market/hooks/useOrders.ts

import { useState, useCallback } from 'react';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { useWalletStore } from '@/stores/wallet.store';
import {
  Order,
  OrderStatus,
  CreateOrderParams,
  FollowUp,
} from '../types/market.types';
import {
  getDivinationTypeName,
  getServiceTypeName,
  getDivinationTypeRoute,
} from '../utils/market.utils';
import { uploadToIpfs, fetchFromIpfs } from '../services/ipfs.service';
import {
  encryptWithSharedKey,
  decryptWithSharedKey,
} from '../services/e2e-encryption.service';

// API 端点
const WS_ENDPOINT = process.env.EXPO_PUBLIC_WS_ENDPOINT || 'ws://127.0.0.1:9944';

let apiInstance: ApiPromise | null = null;

async function getApi(): Promise<ApiPromise> {
  if (apiInstance && apiInstance.isConnected) {
    return apiInstance;
  }

  const provider = new WsProvider(WS_ENDPOINT);
  apiInstance = await ApiPromise.create({ provider });
  return apiInstance;
}

export function useOrders() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { address } = useWalletStore();

  /**
   * 获取我的订单列表（作为客户）
   */
  const getMyOrders = useCallback(
    async (statusFilter?: OrderStatus): Promise<Order[]> => {
      if (!address) {
        throw new Error('请先连接钱包');
      }

      try {
        setLoading(true);
        setError(null);

        const api = await getApi();

        // @ts-ignore
        const entries = await api.query.divinationMarket.orders.entries();

        const orders: Order[] = [];
        for (const [key, value] of entries) {
          const orderId = key.args[0].toNumber();
          const data = value.toJSON() as any;

          if (data.customer !== address) continue;
          if (statusFilter && data.status !== statusFilter) continue;

          orders.push({
            id: orderId,
            customer: data.customer,
            provider: data.provider,
            divinationType: data.divinationType || 0,
            hexagramId: data.hexagramId || 0,
            packageId: data.packageId || 0,
            questionCid: data.questionCid || '',
            answerCid: data.answerCid,
            status: data.status || 'PendingPayment',
            amount: BigInt(data.amount || 0),
            platformFee: BigInt(data.platformFee || 0),
            isUrgent: data.isUrgent || false,
            createdAt: data.createdAt || 0,
            acceptedAt: data.acceptedAt,
            completedAt: data.completedAt,
            divinationTypeName: getDivinationTypeName(data.divinationType || 0),
          });
        }

        // 按时间倒序
        orders.sort((a, b) => b.createdAt - a.createdAt);

        return orders;
      } catch (err) {
        const message = err instanceof Error ? err.message : '获取订单列表失败';
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [address]
  );

  /**
   * 获取收到的订单列表（作为提供者）
   */
  const getReceivedOrders = useCallback(
    async (statusFilter?: OrderStatus): Promise<Order[]> => {
      if (!address) {
        throw new Error('请先连接钱包');
      }

      try {
        setLoading(true);
        setError(null);

        const api = await getApi();

        // @ts-ignore
        const entries = await api.query.divinationMarket.orders.entries();

        const orders: Order[] = [];
        for (const [key, value] of entries) {
          const orderId = key.args[0].toNumber();
          const data = value.toJSON() as any;

          if (data.provider !== address) continue;
          if (statusFilter && data.status !== statusFilter) continue;

          orders.push({
            id: orderId,
            customer: data.customer,
            provider: data.provider,
            divinationType: data.divinationType || 0,
            hexagramId: data.hexagramId || 0,
            packageId: data.packageId || 0,
            questionCid: data.questionCid || '',
            answerCid: data.answerCid,
            status: data.status || 'PendingPayment',
            amount: BigInt(data.amount || 0),
            platformFee: BigInt(data.platformFee || 0),
            isUrgent: data.isUrgent || false,
            createdAt: data.createdAt || 0,
            acceptedAt: data.acceptedAt,
            completedAt: data.completedAt,
            divinationTypeName: getDivinationTypeName(data.divinationType || 0),
          });
        }

        // 按时间倒序
        orders.sort((a, b) => b.createdAt - a.createdAt);

        return orders;
      } catch (err) {
        const message = err instanceof Error ? err.message : '获取订单列表失败';
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [address]
  );

  /**
   * 获取订单详情
   */
  const getOrder = useCallback(async (orderId: number): Promise<Order | null> => {
    try {
      setLoading(true);
      setError(null);

      const api = await getApi();

      // @ts-ignore
      const result = await api.query.divinationMarket.orders(orderId);
      if (result.isNone) {
        return null;
      }

      const data = result.toJSON() as any;

      const order: Order = {
        id: orderId,
        customer: data.customer,
        provider: data.provider,
        divinationType: data.divinationType || 0,
        hexagramId: data.hexagramId || 0,
        packageId: data.packageId || 0,
        questionCid: data.questionCid || '',
        answerCid: data.answerCid,
        status: data.status || 'PendingPayment',
        amount: BigInt(data.amount || 0),
        platformFee: BigInt(data.platformFee || 0),
        isUrgent: data.isUrgent || false,
        createdAt: data.createdAt || 0,
        acceptedAt: data.acceptedAt,
        completedAt: data.completedAt,
        divinationTypeName: getDivinationTypeName(data.divinationType || 0),
      };

      // 获取追问列表
      // @ts-ignore
      const followUpsResult = await api.query.divinationMarket.orderFollowUps(orderId);
      if (!followUpsResult.isNone) {
        const followUpsData = followUpsResult.toJSON() as any[];
        order.followUps = followUpsData.map((f) => ({
          questionCid: f.questionCid || '',
          answerCid: f.answerCid,
          createdAt: f.createdAt || 0,
          answeredAt: f.answeredAt,
        }));
      }

      return order;
    } catch (err) {
      const message = err instanceof Error ? err.message : '获取订单详情失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 创建订单（预览，不发送交易）
   */
  const previewCreateOrder = useCallback(
    async (params: CreateOrderParams) => {
      try {
        setLoading(true);
        setError(null);

        // 上传问题到 IPFS
        const { cid: questionCid } = await uploadToIpfs(params.question);

        return {
          ...params,
          questionCid,
        };
      } catch (err) {
        const message = err instanceof Error ? err.message : '准备订单失败';
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  /**
   * 解密订单内容
   */
  const decryptOrderContent = useCallback(
    async (
      order: Order,
      sharedKey: Uint8Array
    ): Promise<{ question?: string; answer?: string; followUps?: FollowUp[] }> => {
      try {
        const result: { question?: string; answer?: string; followUps?: FollowUp[] } = {};

        // 解密问题
        if (order.questionCid) {
          const encryptedQuestion = await fetchFromIpfs(order.questionCid);
          result.question = await decryptWithSharedKey(encryptedQuestion, sharedKey);
        }

        // 解密答案
        if (order.answerCid) {
          const encryptedAnswer = await fetchFromIpfs(order.answerCid);
          result.answer = await decryptWithSharedKey(encryptedAnswer, sharedKey);
        }

        // 解密追问
        if (order.followUps && order.followUps.length > 0) {
          result.followUps = await Promise.all(
            order.followUps.map(async (f) => {
              const followUp: FollowUp = { ...f };

              if (f.questionCid) {
                const encQ = await fetchFromIpfs(f.questionCid);
                followUp.question = await decryptWithSharedKey(encQ, sharedKey);
              }

              if (f.answerCid) {
                const encA = await fetchFromIpfs(f.answerCid);
                followUp.answer = await decryptWithSharedKey(encA, sharedKey);
              }

              return followUp;
            })
          );
        }

        return result;
      } catch (err) {
        console.error('Decrypt order content error:', err);
        throw err;
      }
    },
    []
  );

  return {
    loading,
    error,
    getMyOrders,
    getReceivedOrders,
    getOrder,
    previewCreateOrder,
    decryptOrderContent,
  };
}
