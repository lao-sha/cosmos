// frontend/src/divination/market/services/chain.service.ts

import { getApi } from '@/api';
import { Keyring } from '@polkadot/keyring';
import { retrieveEncryptedMnemonic, createKeyPairFromMnemonic } from '@/lib';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { SubmittableExtrinsic } from '@polkadot/api/types';
import type { ISubmittableResult } from '@polkadot/types/types';
import type { ApiPromise } from '@polkadot/api';
import type { DivinationType, Provider, ServicePackage, Order, Review, FollowUp } from '../types';

/**
 * 交易状态回调
 */
export interface TransactionCallbacks {
  onBroadcast?: () => void;
  onInBlock?: (blockHash: string) => void;
  onFinalized?: (blockHash: string) => void;
  onError?: (error: Error) => void;
}

/**
 * 交易结果
 */
export interface TransactionResult {
  success: boolean;
  blockHash?: string;
  txHash?: string;
  error?: string;
  events?: any[];
}

/**
 * 获取签名者
 */
export async function getSigner(password: string, address?: string): Promise<KeyringPair> {
  const mnemonic = await retrieveEncryptedMnemonic(password, address);
  return createKeyPairFromMnemonic(mnemonic);
}

/**
 * 签名并发送交易
 */
async function signAndSend(
  tx: SubmittableExtrinsic<'promise'>,
  signer: KeyringPair,
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  return new Promise((resolve, reject) => {
    let txHash: string | undefined;

    tx.signAndSend(signer, ({ status, events, dispatchError }: ISubmittableResult) => {
      if (status.isBroadcast) {
        txHash = tx.hash.toHex();
        callbacks?.onBroadcast?.();
      }

      if (status.isInBlock) {
        const blockHash = status.asInBlock.toHex();
        callbacks?.onInBlock?.(blockHash);
      }

      if (status.isFinalized) {
        const blockHash = status.asFinalized.toHex();

        // 检查是否有 dispatch 错误
        if (dispatchError) {
          let errorMessage = 'Transaction failed';

          if (dispatchError.isModule) {
            const api = tx.registry;
            try {
              const decoded = api.findMetaError(dispatchError.asModule);
              errorMessage = `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
            } catch {
              errorMessage = dispatchError.toString();
            }
          } else {
            errorMessage = dispatchError.toString();
          }

          const error = new Error(errorMessage);
          callbacks?.onError?.(error);
          resolve({
            success: false,
            blockHash,
            txHash,
            error: errorMessage,
            events: events.map((e) => e.toHuman()),
          });
          return;
        }

        callbacks?.onFinalized?.(blockHash);
        resolve({
          success: true,
          blockHash,
          txHash,
          events: events.map((e) => e.toHuman()),
        });
      }
    }).catch((error: Error) => {
      callbacks?.onError?.(error);
      reject(error);
    });
  });
}

// ==================== Provider 相关交易 ====================

/**
 * 注册成为解卦师
 */
export async function registerProvider(
  signer: KeyringPair,
  params: {
    name: string;
    bio: string;
    divinationTypes: DivinationType[];
    specialties: number[];
    avatarCid?: string;
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  // 计算占卜类型的 bitmap
  const typesBitmap = params.divinationTypes.reduce(
    (acc, type) => acc | (1 << type),
    0
  );

  // 计算专长的 bitmap
  const specialtiesBitmap = params.specialties.reduce(
    (acc, spec) => acc | (1 << spec),
    0
  );

  const tx = (api.tx as any).divinationMarket.registerProvider(
    params.name,
    params.bio,
    typesBitmap,
    specialtiesBitmap,
    params.avatarCid || null
  );

  return signAndSend(tx, signer, callbacks);
}

/**
 * 更新解卦师资料
 */
export async function updateProviderProfile(
  signer: KeyringPair,
  params: {
    name?: string;
    bio?: string;
    avatarCid?: string;
    specialties?: number[];
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const specialtiesBitmap = params.specialties
    ? params.specialties.reduce((acc, spec) => acc | (1 << spec), 0)
    : null;

  const tx = (api.tx as any).divinationMarket.updateProviderProfile(
    params.name || null,
    params.bio || null,
    params.avatarCid || null,
    specialtiesBitmap
  );

  return signAndSend(tx, signer, callbacks);
}

/**
 * 创建服务套餐
 */
export async function createPackage(
  signer: KeyringPair,
  params: {
    name: string;
    description: string;
    divinationType: DivinationType;
    price: bigint;
    deliveryDays: number;
    maxFollowUps: number;
    isActive?: boolean;
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.createPackage(
    params.name,
    params.description,
    params.divinationType,
    params.price,
    params.deliveryDays,
    params.maxFollowUps,
    params.isActive ?? true
  );

  return signAndSend(tx, signer, callbacks);
}

/**
 * 更新服务套餐
 */
export async function updatePackage(
  signer: KeyringPair,
  params: {
    packageId: number;
    name?: string;
    description?: string;
    price?: bigint;
    deliveryDays?: number;
    maxFollowUps?: number;
    isActive?: boolean;
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.updatePackage(
    params.packageId,
    params.name || null,
    params.description || null,
    params.price || null,
    params.deliveryDays || null,
    params.maxFollowUps || null,
    params.isActive ?? null
  );

  return signAndSend(tx, signer, callbacks);
}

/**
 * 删除服务套餐
 */
export async function deletePackage(
  signer: KeyringPair,
  packageId: number,
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.deletePackage(packageId);

  return signAndSend(tx, signer, callbacks);
}

// ==================== Order 相关交易 ====================

/**
 * 创建订单
 */
export async function createOrder(
  signer: KeyringPair,
  params: {
    providerId: string;
    packageId: number;
    questionCid: string; // IPFS CID 存储加密问题
    hexagramData?: string; // 卦象数据
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.createOrder(
    params.providerId,
    params.packageId,
    params.questionCid,
    params.hexagramData || null
  );

  return signAndSend(tx, signer, callbacks);
}

/**
 * 接受订单（解卦师）
 */
export async function acceptOrder(
  signer: KeyringPair,
  orderId: number,
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.acceptOrder(orderId);

  return signAndSend(tx, signer, callbacks);
}

/**
 * 拒绝订单（解卦师）
 */
export async function rejectOrder(
  signer: KeyringPair,
  orderId: number,
  reason?: string,
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.rejectOrder(orderId, reason || null);

  return signAndSend(tx, signer, callbacks);
}

/**
 * 完成订单（提交解卦结果）
 */
export async function completeOrder(
  signer: KeyringPair,
  params: {
    orderId: number;
    resultCid: string; // IPFS CID 存储加密结果
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.completeOrder(
    params.orderId,
    params.resultCid
  );

  return signAndSend(tx, signer, callbacks);
}

/**
 * 取消订单（求卦者）
 */
export async function cancelOrder(
  signer: KeyringPair,
  orderId: number,
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.cancelOrder(orderId);

  return signAndSend(tx, signer, callbacks);
}

/**
 * 申请退款
 */
export async function requestRefund(
  signer: KeyringPair,
  orderId: number,
  reason: string,
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.requestRefund(orderId, reason);

  return signAndSend(tx, signer, callbacks);
}

/**
 * 提交追问
 */
export async function submitFollowUp(
  signer: KeyringPair,
  params: {
    orderId: number;
    questionCid: string;
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.submitFollowUp(
    params.orderId,
    params.questionCid
  );

  return signAndSend(tx, signer, callbacks);
}

/**
 * 回复追问（解卦师）
 */
export async function replyFollowUp(
  signer: KeyringPair,
  params: {
    orderId: number;
    followUpIndex: number;
    answerCid: string;
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.replyFollowUp(
    params.orderId,
    params.followUpIndex,
    params.answerCid
  );

  return signAndSend(tx, signer, callbacks);
}

// ==================== Review 相关交易 ====================

/**
 * 提交评价
 */
export async function submitReview(
  signer: KeyringPair,
  params: {
    orderId: number;
    ratings: {
      accuracy: number; // 1-5
      attitude: number; // 1-5
      speed: number; // 1-5
      value: number; // 1-5
    };
    contentCid?: string; // 加密评价内容的 IPFS CID
    isAnonymous?: boolean;
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.submitReview(
    params.orderId,
    params.ratings.accuracy,
    params.ratings.attitude,
    params.ratings.speed,
    params.ratings.value,
    params.contentCid || null,
    params.isAnonymous ?? false
  );

  return signAndSend(tx, signer, callbacks);
}

/**
 * 回复评价（解卦师）
 */
export async function replyReview(
  signer: KeyringPair,
  params: {
    reviewId: number;
    replyCid: string;
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.replyReview(
    params.reviewId,
    params.replyCid
  );

  return signAndSend(tx, signer, callbacks);
}

// ==================== 资金相关交易 ====================

/**
 * 提现（解卦师提取已完成订单的收入）
 */
export async function withdraw(
  signer: KeyringPair,
  amount?: bigint, // 不传则提取全部
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = amount
    ? (api.tx as any).divinationMarket.withdraw(amount)
    : (api.tx as any).divinationMarket.withdrawAll();

  return signAndSend(tx, signer, callbacks);
}

/**
 * 打赏（额外打赏解卦师）
 */
export async function tip(
  signer: KeyringPair,
  params: {
    providerId: string;
    amount: bigint;
    orderId?: number;
  },
  callbacks?: TransactionCallbacks
): Promise<TransactionResult> {
  const api = await getApi();

  const tx = (api.tx as any).divinationMarket.tip(
    params.providerId,
    params.amount,
    params.orderId || null
  );

  return signAndSend(tx, signer, callbacks);
}

// ==================== 查询方法 ====================

/**
 * 查询解卦师信息
 */
export async function queryProvider(address: string): Promise<Provider | null> {
  const api = await getApi();

  const result = await (api.query as any).divinationMarket.providers(address);

  if (result.isNone) {
    return null;
  }

  const data = result.unwrap();
  return {
    id: address,
    address,
    name: data.name.toHuman(),
    bio: data.bio.toHuman(),
    avatarCid: data.avatarCid.toHuman() || undefined,
    divinationTypes: parseBitmapToArray(data.divinationTypes.toNumber()),
    specialties: parseBitmapToArray(data.specialties.toNumber()),
    tier: data.tier.toNumber(),
    totalOrders: data.totalOrders.toNumber(),
    completedOrders: data.completedOrders.toNumber(),
    averageRating: data.averageRating.toNumber() / 100,
    totalEarnings: BigInt(data.totalEarnings.toString()),
    availableBalance: BigInt(data.availableBalance.toString()),
    isActive: data.isActive.toHuman(),
    registeredAt: data.registeredAt.toNumber(),
  };
}

/**
 * 查询套餐列表
 */
export async function queryPackages(providerId: string): Promise<ServicePackage[]> {
  const api = await getApi();

  const result = await (api.query as any).divinationMarket.providerPackages(providerId);

  if (!result || result.length === 0) {
    return [];
  }

  return result.map((pkg: any, index: number) => ({
    id: index,
    providerId,
    name: pkg.name.toHuman(),
    description: pkg.description.toHuman(),
    divinationType: pkg.divinationType.toNumber(),
    price: BigInt(pkg.price.toString()),
    deliveryDays: pkg.deliveryDays.toNumber(),
    maxFollowUps: pkg.maxFollowUps.toNumber(),
    isActive: pkg.isActive.toHuman(),
    totalOrders: pkg.totalOrders?.toNumber() || 0,
    createdAt: pkg.createdAt?.toNumber() || 0,
  }));
}

/**
 * 查询订单详情
 */
export async function queryOrder(orderId: number): Promise<Order | null> {
  const api = await getApi();

  const result = await (api.query as any).divinationMarket.orders(orderId);

  if (result.isNone) {
    return null;
  }

  const data = result.unwrap();
  return {
    id: orderId,
    clientId: data.clientId.toHuman(),
    providerId: data.providerId.toHuman(),
    packageId: data.packageId.toNumber(),
    status: data.status.toNumber(),
    questionCid: data.questionCid.toHuman(),
    resultCid: data.resultCid?.toHuman() || undefined,
    hexagramData: data.hexagramData?.toHuman() || undefined,
    price: BigInt(data.price.toString()),
    createdAt: data.createdAt.toNumber(),
    acceptedAt: data.acceptedAt?.toNumber() || undefined,
    completedAt: data.completedAt?.toNumber() || undefined,
    deliveryDeadline: data.deliveryDeadline?.toNumber() || undefined,
    followUps: data.followUps?.map((fu: any) => ({
      questionCid: fu.questionCid.toHuman(),
      answerCid: fu.answerCid?.toHuman() || undefined,
      createdAt: fu.createdAt.toNumber(),
      answeredAt: fu.answeredAt?.toNumber() || undefined,
    })) || [],
  };
}

/**
 * 查询评价详情
 */
export async function queryReview(reviewId: number): Promise<Review | null> {
  const api = await getApi();

  const result = await (api.query as any).divinationMarket.reviews(reviewId);

  if (result.isNone) {
    return null;
  }

  const data = result.unwrap();
  return {
    id: reviewId,
    orderId: data.orderId.toNumber(),
    clientId: data.clientId.toHuman(),
    providerId: data.providerId.toHuman(),
    ratings: {
      accuracy: data.accuracy.toNumber(),
      attitude: data.attitude.toNumber(),
      speed: data.speed.toNumber(),
      value: data.value.toNumber(),
    },
    contentCid: data.contentCid?.toHuman() || undefined,
    replyCid: data.replyCid?.toHuman() || undefined,
    isAnonymous: data.isAnonymous.toHuman(),
    createdAt: data.createdAt.toNumber(),
    repliedAt: data.repliedAt?.toNumber() || undefined,
  };
}

/**
 * 查询用户订单列表
 */
export async function queryUserOrders(
  address: string,
  role: 'client' | 'provider'
): Promise<number[]> {
  const api = await getApi();

  const result =
    role === 'client'
      ? await (api.query as any).divinationMarket.clientOrders(address)
      : await (api.query as any).divinationMarket.providerOrders(address);

  return result?.map((id: any) => id.toNumber()) || [];
}

/**
 * 查询解卦师评价列表
 */
export async function queryProviderReviews(providerId: string): Promise<number[]> {
  const api = await getApi();

  const result = await (api.query as any).divinationMarket.providerReviews(providerId);

  return result?.map((id: any) => id.toNumber()) || [];
}

// ==================== 工具函数 ====================

/**
 * 解析 bitmap 为数组
 */
function parseBitmapToArray(bitmap: number): number[] {
  const result: number[] = [];
  for (let i = 0; i < 32; i++) {
    if (bitmap & (1 << i)) {
      result.push(i);
    }
  }
  return result;
}

/**
 * 估算交易费用
 */
export async function estimateFee(
  tx: SubmittableExtrinsic<'promise'>,
  address: string
): Promise<bigint> {
  const api = await getApi();
  const info = await tx.paymentInfo(address);
  return BigInt(info.partialFee.toString());
}

/**
 * 获取当前区块高度
 */
export async function getCurrentBlock(): Promise<number> {
  const api = await getApi();
  const header = await api.rpc.chain.getHeader();
  return header.number.toNumber();
}

/**
 * 订阅区块
 */
export async function subscribeBlocks(
  callback: (blockNumber: number) => void
): Promise<() => void> {
  const api = await getApi();
  const unsub = await api.rpc.chain.subscribeNewHeads((header) => {
    callback(header.number.toNumber());
  });
  return unsub;
}
