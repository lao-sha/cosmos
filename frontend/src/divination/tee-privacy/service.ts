/**
 * # TEE 隐私计算服务
 *
 * 本模块提供 TEE Privacy 系统的前端服务接口。
 *
 * ## 功能概述
 *
 * 1. **节点管理**: 查询 TEE 节点状态、获取 Enclave 公钥
 * 2. **计算请求**: 提交加密计算请求、查询请求状态
 * 3. **数据加密**: 使用 Enclave 公钥加密敏感数据
 * 4. **结果获取**: 获取计算结果并解密
 *
 * ## 使用示例
 *
 * ```typescript
 * import { TeePrivacyService, ComputeTypeId } from '@/divination/tee-privacy';
 *
 * // 创建服务实例
 * const teeService = new TeePrivacyService(api);
 *
 * // 获取活跃节点
 * const nodes = await teeService.getActiveNodes();
 *
 * // 提交加密计算请求
 * const requestId = await teeService.submitComputeRequest(
 *   account,
 *   ComputeTypeId.BaZi,
 *   encryptedInput,
 *   signer
 * );
 *
 * // 查询请求状态
 * const status = await teeService.getRequestStatus(requestId);
 * ```
 */

import type { ApiPromise } from '@polkadot/api';
import type { KeyringPair } from '@polkadot/keyring/types';
import {
  TeeNodeInfo,
  TeeNodeStatus,
  TeeType,
  NodeStakeInfo,
  ComputeRequestInfo,
  ComputeResultInfo,
  RequestStatus,
  BatchRequestItem,
  BatchResultItem,
  AttestationVerifyResult,
  AuditLogEntry,
  ComputeTypeId,
} from './types';

// ============================================================================
// 辅助函数
// ============================================================================

/**
 * 将 Uint8Array 转换为 hex 字符串
 */
function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * 将 hex 字符串转换为 Uint8Array
 */
function fromHex(hex: string): Uint8Array {
  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(cleanHex.substr(i * 2, 2), 16);
  }
  return bytes;
}

// ============================================================================
// TEE Privacy 服务
// ============================================================================

/**
 * TEE 隐私计算服务
 *
 * 提供与 pallet-tee-privacy 交互的完整功能。
 */
export class TeePrivacyService {
  private api: ApiPromise;

  constructor(api: ApiPromise) {
    this.api = api;
  }

  // ==========================================================================
  // 节点查询
  // ==========================================================================

  /**
   * 获取所有活跃 TEE 节点
   *
   * @returns 活跃节点信息列表
   */
  async getActiveNodes(): Promise<TeeNodeInfo[]> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getActiveNodes();
      return result.map(this.parseNodeInfo);
    } catch (error) {
      console.error('Failed to get active nodes:', error);
      return [];
    }
  }

  /**
   * 获取指定节点信息
   *
   * @param account 节点账户地址
   * @returns 节点信息，如果不存在返回 null
   */
  async getNodeInfo(account: string): Promise<TeeNodeInfo | null> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getNodeInfo(account);
      if (result.isNone) {
        return null;
      }
      return this.parseNodeInfo(result.unwrap());
    } catch (error) {
      console.error('Failed to get node info:', error);
      return null;
    }
  }

  /**
   * 获取节点 Enclave 公钥
   *
   * @param account 节点账户地址
   * @returns Enclave 公钥 (hex)，如果不存在返回 null
   */
  async getEnclavePubkey(account: string): Promise<string | null> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getEnclavePubkey(account);
      if (result.isNone) {
        return null;
      }
      return toHex(result.unwrap());
    } catch (error) {
      console.error('Failed to get enclave pubkey:', error);
      return null;
    }
  }

  /**
   * 检查节点是否活跃
   *
   * @param account 节点账户地址
   * @returns 是否活跃
   */
  async isNodeActive(account: string): Promise<boolean> {
    try {
      return await (this.api.call as any).teePrivacyApi.isNodeActive(account);
    } catch (error) {
      console.error('Failed to check node active:', error);
      return false;
    }
  }

  /**
   * 获取活跃节点数量
   */
  async getActiveNodeCount(): Promise<number> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getActiveNodeCount();
      return result.toNumber();
    } catch (error) {
      console.error('Failed to get active node count:', error);
      return 0;
    }
  }

  /**
   * 获取节点总数
   */
  async getNodeCount(): Promise<number> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getNodeCount();
      return result.toNumber();
    } catch (error) {
      console.error('Failed to get node count:', error);
      return 0;
    }
  }

  // ==========================================================================
  // 请求查询
  // ==========================================================================

  /**
   * 获取请求状态
   *
   * @param requestId 请求 ID
   * @returns 请求状态信息，如果不存在返回 null
   */
  async getRequestStatus(requestId: number): Promise<ComputeRequestInfo | null> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getRequestStatus(requestId);
      if (result.isNone) {
        return null;
      }
      return this.parseRequestInfo(result.unwrap());
    } catch (error) {
      console.error('Failed to get request status:', error);
      return null;
    }
  }

  /**
   * 获取用户的所有待处理请求
   *
   * @param account 用户账户地址
   * @returns 待处理请求 ID 列表
   */
  async getUserPendingRequests(account: string): Promise<number[]> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getUserPendingRequests(account);
      return result.map((id: any) => id.toNumber());
    } catch (error) {
      console.error('Failed to get user pending requests:', error);
      return [];
    }
  }

  /**
   * 获取节点当前处理的请求
   *
   * @param node 节点账户地址
   * @returns 请求 ID，如果没有返回 null
   */
  async getNodeCurrentRequest(node: string): Promise<number | null> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getNodeCurrentRequest(node);
      if (result.isNone) {
        return null;
      }
      return result.unwrap().toNumber();
    } catch (error) {
      console.error('Failed to get node current request:', error);
      return null;
    }
  }

  /**
   * 获取下一个请求 ID
   */
  async getNextRequestId(): Promise<number> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getNextRequestId();
      return result.toNumber();
    } catch (error) {
      console.error('Failed to get next request id:', error);
      return 0;
    }
  }

  /**
   * 获取待处理请求数量
   */
  async getPendingRequestCount(): Promise<number> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getPendingRequestCount();
      return result.toNumber();
    } catch (error) {
      console.error('Failed to get pending request count:', error);
      return 0;
    }
  }

  // ==========================================================================
  // 认证验证
  // ==========================================================================

  /**
   * 验证认证报告
   *
   * @param mrEnclave MRENCLAVE 值 (hex)
   * @param mrSigner MRSIGNER 值 (hex)
   * @param timestamp 认证时间戳
   * @returns 验证结果
   */
  async verifyAttestation(
    mrEnclave: string,
    mrSigner: string,
    timestamp: number
  ): Promise<AttestationVerifyResult> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.verifyAttestation(
        fromHex(mrEnclave),
        fromHex(mrSigner),
        timestamp
      );
      return {
        isValid: result.isValid.isTrue,
        mrEnclaveMatch: result.mrEnclaveMatch.isTrue,
        isExpired: result.isExpired.isTrue,
        errorMessage: result.errorMessage.isSome
          ? new TextDecoder().decode(result.errorMessage.unwrap())
          : null,
      };
    } catch (error) {
      console.error('Failed to verify attestation:', error);
      return {
        isValid: false,
        mrEnclaveMatch: false,
        isExpired: true,
        errorMessage: 'Verification failed',
      };
    }
  }

  /**
   * 获取允许的 MRENCLAVE 列表
   */
  async getAllowedMrEnclaves(): Promise<string[]> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getAllowedMrEnclaves();
      return result.map((item: Uint8Array) => toHex(item));
    } catch (error) {
      console.error('Failed to get allowed mr enclaves:', error);
      return [];
    }
  }

  /**
   * 获取允许的 MRSIGNER 列表
   */
  async getAllowedMrSigners(): Promise<string[]> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getAllowedMrSigners();
      return result.map((item: Uint8Array) => toHex(item));
    } catch (error) {
      console.error('Failed to get allowed mr signers:', error);
      return [];
    }
  }

  // ==========================================================================
  // 经济激励查询
  // ==========================================================================

  /**
   * 获取节点质押信息
   *
   * @param account 节点账户地址
   * @returns 质押信息，如果未质押返回 null
   */
  async getNodeStake(account: string): Promise<NodeStakeInfo | null> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getNodeStake(account);
      if (result.isNone) {
        return null;
      }
      const [amount, unlockAt, isUnbonding] = result.unwrap();
      return {
        amount: BigInt(amount.toString()),
        unlockAt: unlockAt.isSome ? unlockAt.unwrap().toNumber() : null,
        isUnbonding: isUnbonding.isTrue,
      };
    } catch (error) {
      console.error('Failed to get node stake:', error);
      return null;
    }
  }

  /**
   * 获取最低质押要求
   */
  async getMinimumStake(): Promise<bigint> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getMinimumStake();
      return BigInt(result.toString());
    } catch (error) {
      console.error('Failed to get minimum stake:', error);
      return BigInt(0);
    }
  }

  /**
   * 获取累计惩罚金额
   */
  async getTotalSlashed(): Promise<bigint> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getTotalSlashed();
      return BigInt(result.toString());
    } catch (error) {
      console.error('Failed to get total slashed:', error);
      return BigInt(0);
    }
  }

  /**
   * 获取奖励池余额
   */
  async getRewardPool(): Promise<bigint> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getRewardPool();
      return BigInt(result.toString());
    } catch (error) {
      console.error('Failed to get reward pool:', error);
      return BigInt(0);
    }
  }

  // ==========================================================================
  // 审计日志查询
  // ==========================================================================

  /**
   * 获取审计日志是否启用
   */
  async isAuditEnabled(): Promise<boolean> {
    try {
      return await (this.api.call as any).teePrivacyApi.isAuditEnabled();
    } catch (error) {
      console.error('Failed to check audit enabled:', error);
      return false;
    }
  }

  /**
   * 获取账户的审计日志数量
   *
   * @param account 账户地址
   */
  async getAccountAuditLogCount(account: string): Promise<number> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getAccountAuditLogCount(account);
      return result.toNumber();
    } catch (error) {
      console.error('Failed to get account audit log count:', error);
      return 0;
    }
  }

  /**
   * 获取下一个审计日志 ID
   */
  async getNextAuditLogId(): Promise<number> {
    try {
      const result = await (this.api.call as any).teePrivacyApi.getNextAuditLogId();
      return result.toNumber();
    } catch (error) {
      console.error('Failed to get next audit log id:', error);
      return 0;
    }
  }

  // ==========================================================================
  // 交易提交
  // ==========================================================================

  /**
   * 提交计算请求
   *
   * @param computeTypeId 计算类型 ID
   * @param inputHash 加密输入数据哈希 (hex)
   * @param assignedNode 指定节点账户 (可选)
   * @param signer 签名者
   * @returns 交易哈希
   */
  async submitComputeRequest(
    computeTypeId: ComputeTypeId,
    inputHash: string,
    assignedNode: string | null,
    signer: KeyringPair
  ): Promise<string> {
    const tx = this.api.tx.teePrivacy.submitComputeRequest(
      computeTypeId,
      fromHex(inputHash),
      assignedNode
    );

    return new Promise((resolve, reject) => {
      tx.signAndSend(signer, ({ status, dispatchError }) => {
        if (status.isInBlock || status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            resolve(status.hash.toHex());
          }
        }
      }).catch(reject);
    });
  }

  /**
   * 批量提交计算请求
   *
   * @param requests 批量请求项列表
   * @param signer 签名者
   * @returns 交易哈希
   */
  async submitBatchComputeRequests(
    requests: BatchRequestItem[],
    signer: KeyringPair
  ): Promise<string> {
    const tx = this.api.tx.teePrivacy.submitBatchComputeRequests(
      requests.map((r) => ({
        compute_type_id: r.computeTypeId,
        input_hash: fromHex(r.inputHash),
      }))
    );

    return new Promise((resolve, reject) => {
      tx.signAndSend(signer, ({ status, dispatchError }) => {
        if (status.isInBlock || status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            resolve(status.hash.toHex());
          }
        }
      }).catch(reject);
    });
  }

  /**
   * 取消计算请求
   *
   * @param requestId 请求 ID
   * @param signer 签名者
   * @returns 交易哈希
   */
  async cancelComputeRequest(requestId: number, signer: KeyringPair): Promise<string> {
    const tx = this.api.tx.teePrivacy.cancelComputeRequest(requestId);

    return new Promise((resolve, reject) => {
      tx.signAndSend(signer, ({ status, dispatchError }) => {
        if (status.isInBlock || status.isFinalized) {
          if (dispatchError) {
            reject(new Error(dispatchError.toString()));
          } else {
            resolve(status.hash.toHex());
          }
        }
      }).catch(reject);
    });
  }

  // ==========================================================================
  // 私有辅助方法
  // ==========================================================================

  private parseNodeInfo(raw: any): TeeNodeInfo {
    return {
      account: raw.account.toString(),
      enclavePubkey: toHex(raw.enclavePubkey),
      teeType: raw.teeType.toNumber() as TeeType,
      status: raw.status.toNumber() as TeeNodeStatus,
      registeredAt: raw.registeredAt.toNumber(),
      mrEnclave: toHex(raw.mrEnclave),
      attestationTimestamp: raw.attestationTimestamp.toNumber(),
    };
  }

  private parseRequestInfo(raw: any): ComputeRequestInfo {
    return {
      id: raw.requestId.toNumber(),
      requester: raw.requester.toString(),
      computeTypeId: raw.computeTypeId.toNumber() as ComputeTypeId,
      inputHash: toHex(raw.inputHash),
      assignedNode: raw.assignedNode.isSome ? raw.assignedNode.unwrap().toString() : null,
      createdAt: raw.createdAt.toNumber(),
      timeoutAt: raw.timeoutAt.toNumber(),
      status: raw.status.toNumber() as RequestStatus,
      failoverCount: raw.failoverCount.toNumber(),
      failureReason: raw.failureReason.isSome
        ? raw.failureReason.unwrap().toNumber()
        : null,
    };
  }
}
