/**
 * # TEE 隐私计算服务模块
 *
 * 本模块提供 TEE Privacy 系统的前端集成接口。
 *
 * ## 功能概述
 *
 * - **类型定义**: TEE 节点、请求、结果等完整类型
 * - **服务接口**: 与 pallet-tee-privacy 交互的服务类
 *
 * ## 快速开始
 *
 * ```typescript
 * import {
 *   TeePrivacyService,
 *   ComputeTypeId,
 *   TeeNodeStatus,
 *   RequestStatus,
 * } from '@/divination/tee-privacy';
 *
 * // 创建服务实例
 * const teeService = new TeePrivacyService(api);
 *
 * // 获取活跃节点
 * const nodes = await teeService.getActiveNodes();
 * console.log(`活跃节点数: ${nodes.length}`);
 *
 * // 提交计算请求
 * const txHash = await teeService.submitComputeRequest(
 *   ComputeTypeId.BaZi,
 *   inputHash,
 *   null, // 自动分配节点
 *   signer
 * );
 *
 * // 查询请求状态
 * const status = await teeService.getRequestStatus(requestId);
 * if (status?.status === RequestStatus.Completed) {
 *   console.log('计算已完成');
 * }
 * ```
 *
 * ## 加密流程
 *
 * 1. 获取 TEE 节点的 Enclave 公钥
 * 2. 使用 ECDH + AES-256-GCM 加密敏感数据
 * 3. 提交加密数据哈希到链上
 * 4. TEE 节点在 Enclave 内解密并计算
 * 5. 获取加密结果并解密
 *
 * @module tee-privacy
 */

// 导出所有类型
export {
  // 枚举类型
  TeeType,
  TeeNodeStatus,
  RequestStatus,
  FailureReason,
  ComputeTypeId,
  AuditEventType,
  // 接口类型
  type TeeNodeInfo,
  type NodeStakeInfo,
  type NodeStatistics,
  type ComputeRequestInfo,
  type ComputeResultInfo,
  type BatchRequestItem,
  type BatchResultItem,
  type EncryptedData,
  type TeeAttestation,
  type AttestationVerifyResult,
  type AuditLogEntry,
} from './types';

// 导出服务类
export { TeePrivacyService } from './service';
