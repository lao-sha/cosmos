// frontend/src/divination/market/hooks/useReviews.ts

import { useState, useCallback } from 'react';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { useWalletStore } from '@/stores/wallet.store';
import { Review, SubmitReviewParams } from '../types/market.types';
import { uploadToIpfs, fetchFromIpfs } from '../services/ipfs.service';

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

export function useReviews() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { address } = useWalletStore();

  /**
   * 获取评价详情
   */
  const getReview = useCallback(async (orderId: number): Promise<Review | null> => {
    try {
      setLoading(true);
      setError(null);

      const api = await getApi();

      // @ts-ignore
      const result = await api.query.divinationMarket.reviews(orderId);
      if (result.isNone) {
        return null;
      }

      const data = result.toJSON() as any;

      const review: Review = {
        orderId,
        customer: data.customer || '',
        overallRating: data.overallRating || 0,
        accuracyRating: data.accuracyRating || 0,
        attitudeRating: data.attitudeRating || 0,
        responseRating: data.responseRating || 0,
        contentCid: data.contentCid,
        isAnonymous: data.isAnonymous || false,
        replyCid: data.replyCid,
        createdAt: data.createdAt || 0,
        repliedAt: data.repliedAt,
      };

      // 尝试获取评价内容
      if (review.contentCid) {
        try {
          review.content = await fetchFromIpfs(review.contentCid);
        } catch (e) {
          console.warn('Failed to fetch review content:', e);
        }
      }

      // 尝试获取回复内容
      if (review.replyCid) {
        try {
          review.reply = await fetchFromIpfs(review.replyCid);
        } catch (e) {
          console.warn('Failed to fetch review reply:', e);
        }
      }

      return review;
    } catch (err) {
      const message = err instanceof Error ? err.message : '获取评价详情失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 获取我的评价列表（作为客户）
   */
  const getMyReviews = useCallback(async (): Promise<Review[]> => {
    if (!address) {
      throw new Error('请先连接钱包');
    }

    try {
      setLoading(true);
      setError(null);

      const api = await getApi();

      // @ts-ignore
      const entries = await api.query.divinationMarket.reviews.entries();

      const reviews: Review[] = [];
      for (const [key, value] of entries) {
        const orderId = key.args[0].toNumber();
        const data = value.toJSON() as any;

        if (data.customer !== address) continue;

        reviews.push({
          orderId,
          customer: data.customer || '',
          overallRating: data.overallRating || 0,
          accuracyRating: data.accuracyRating || 0,
          attitudeRating: data.attitudeRating || 0,
          responseRating: data.responseRating || 0,
          contentCid: data.contentCid,
          isAnonymous: data.isAnonymous || false,
          replyCid: data.replyCid,
          createdAt: data.createdAt || 0,
          repliedAt: data.repliedAt,
        });
      }

      // 按时间倒序
      reviews.sort((a, b) => b.createdAt - a.createdAt);

      return reviews;
    } catch (err) {
      const message = err instanceof Error ? err.message : '获取评价列表失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [address]);

  /**
   * 准备提交评价（预览，不发送交易）
   */
  const prepareSubmitReview = useCallback(
    async (
      params: SubmitReviewParams
    ): Promise<SubmitReviewParams & { contentCid?: string }> => {
      try {
        setLoading(true);
        setError(null);

        let contentCid: string | undefined;

        // 上传评价内容到 IPFS
        if (params.content) {
          const result = await uploadToIpfs(params.content);
          contentCid = result.cid;
        }

        return {
          ...params,
          contentCid,
        };
      } catch (err) {
        const message = err instanceof Error ? err.message : '准备评价失败';
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  /**
   * 准备回复评价（预览，不发送交易）
   */
  const prepareReplyReview = useCallback(
    async (orderId: number, reply: string): Promise<{ orderId: number; replyCid: string }> => {
      try {
        setLoading(true);
        setError(null);

        const result = await uploadToIpfs(reply);

        return {
          orderId,
          replyCid: result.cid,
        };
      } catch (err) {
        const message = err instanceof Error ? err.message : '准备回复失败';
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  return {
    loading,
    error,
    getReview,
    getMyReviews,
    prepareSubmitReview,
    prepareReplyReview,
  };
}
