/**
 * # TEE 隐私计算服务 - 类型定义
 *
 * 本模块定义了 TEE Privacy 系统的前端类型。
 */

// ============================================================================
// TEE 平台类型
// ============================================================================

/** TEE 平台类型 */
export enum TeeType {
  /** Intel SGX (EPID 认证) */
  IntelSgx = 0,
  /** Intel SGX (DCAP 认证) */
  IntelSgxDcap = 1,
  /** ARM TrustZone */
  ArmTrustZone = 2,
  /** AMD SEV */
  AmdSev = 3,
  /** RISC-V Keystone */
  RiscVKeystone = 4,
}

// ============================================================================
// 节点状态
// ============================================================================

/** TEE 节点状态 */
export enum TeeNodeStatus {
  /** 待验证 */
  Pending = 0,
  /** 活跃 */
  Active = 1,
  /** 暂停 */
  Suspended = 2,
  /** 已注销 */
  Deregistered = 3,
}

// ============================================================================
// 请求状态
// ============================================================================

/** 计算请求状态 */
export enum RequestStatus {
  /** 待处理 */
  Pending = 0,
  /** 处理中 */
  Processing = 1,
  /** 已完成 */
  Completed = 2,
  /** 已失败 */
  Failed = 3,
  /** 已超时 */
  Timeout = 4,
}

// ============================================================================
// 失败原因
// ============================================================================

/** 计算失败原因 */
export enum FailureReason {
  /** 未知错误 */
  Unknown = 0,
  /** 输入数据无效 */
  InvalidInput = 1,
  /** 解密失败 */
  DecryptionFailed = 2,
  /** 计算错误 */
  ComputationError = 3,
  /** 加密失败 */
  EncryptionFailed = 4,
  /** 证明生成失败 */
  ProofGenerationFailed = 5,
  /** 节点故障 */
  NodeFailure = 6,
  /** 超时 */
  Timeout = 7,
}

// ============================================================================
// 计算类型
// ============================================================================

/** 计算类型 ID */
export enum ComputeTypeId {
  /** 八字命理 */
  BaZi = 0,
  /** 梅花易数 */
  MeiHua = 1,
  /** 奇门遁甲 */
  QiMen = 2,
  /** 六爻占卜 */
  LiuYao = 3,
  /** 紫微斗数 */
  ZiWei = 4,
  /** 塔罗占卜 */
  Tarot = 5,
  /** 大六壬 */
  DaLiuRen = 6,
  /** 小六壬 */
  XiaoLiuRen = 7,
}

// ============================================================================
// TEE 节点信息
// ============================================================================

/** TEE 节点信息 */
export interface TeeNodeInfo {
  /** 节点账户地址 (hex) */
  account: string;
  /** Enclave 公钥 (hex, 32 bytes) */
  enclavePubkey: string;
  /** TEE 类型 */
  teeType: TeeType;
  /** 节点状态 */
  status: TeeNodeStatus;
  /** 注册时间 (Unix timestamp) */
  registeredAt: number;
  /** MRENCLAVE (hex, 32 bytes) */
  mrEnclave: string;
  /** 认证时间 (Unix timestamp) */
  attestationTimestamp: number;
}

/** 节点质押信息 */
export interface NodeStakeInfo {
  /** 质押金额 */
  amount: bigint;
  /** 解锁区块 (如果正在解除质押) */
  unlockAt: number | null;
  /** 是否正在解除质押 */
  isUnbonding: boolean;
}

/** 节点统计信息 */
export interface NodeStatistics {
  /** 完成的请求总数 */
  completedRequests: number;
  /** 失败的请求总数 */
  failedRequests: number;
  /** 超时的请求总数 */
  timeoutRequests: number;
  /** 被惩罚次数 */
  slashCount: number;
  /** 累计获得奖励 */
  totalRewards: bigint;
  /** 累计被惩罚金额 */
  totalSlashed: bigint;
  /** 平均处理时间 (区块数) */
  avgProcessingBlocks: number;
  /** 最后活跃区块 */
  lastActiveBlock: number;
}

// ============================================================================
// 计算请求
// ============================================================================

/** 计算请求信息 */
export interface ComputeRequestInfo {
  /** 请求 ID */
  id: number;
  /** 请求者账户 */
  requester: string;
  /** 计算类型 ID */
  computeTypeId: ComputeTypeId;
  /** 加密输入数据哈希 (hex, 32 bytes) */
  inputHash: string;
  /** 分配的节点账户 */
  assignedNode: string | null;
  /** 创建区块 */
  createdAt: number;
  /** 超时区块 */
  timeoutAt: number;
  /** 状态 */
  status: RequestStatus;
  /** 故障转移次数 */
  failoverCount: number;
  /** 失败原因 */
  failureReason: FailureReason | null;
}

/** 计算结果信息 */
export interface ComputeResultInfo {
  /** 请求 ID */
  requestId: number;
  /** 执行节点账户 */
  executor: string;
  /** 加密输出数据哈希 (hex, 32 bytes) */
  outputHash: string;
  /** Enclave 签名 (hex, 64 bytes) */
  enclaveSignature: string;
  /** 完成时间 (Unix timestamp) */
  completedAt: number;
}

// ============================================================================
// 批处理
// ============================================================================

/** 批处理请求项 */
export interface BatchRequestItem {
  /** 计算类型 ID */
  computeTypeId: ComputeTypeId;
  /** 加密输入数据哈希 (hex, 32 bytes) */
  inputHash: string;
}

/** 批处理结果项 */
export interface BatchResultItem {
  /** 请求 ID */
  requestId: number;
  /** 加密输出数据哈希 (hex, 32 bytes) */
  outputHash: string;
  /** Enclave 签名 (hex, 64 bytes) */
  enclaveSignature: string;
}

// ============================================================================
// 加密数据
// ============================================================================

/** 加密数据结构 (ECDH + AES-256-GCM) */
export interface EncryptedData {
  /** 密文 (base64) */
  ciphertext: string;
  /** 临时公钥 (X25519, hex, 32 bytes) */
  ephemeralPubkey: string;
  /** Nonce (hex, 12 bytes) */
  nonce: string;
  /** 认证标签 (hex, 16 bytes) */
  authTag: string;
}

// ============================================================================
// 认证相关
// ============================================================================

/** 远程认证报告 */
export interface TeeAttestation {
  /** TEE 类型 */
  teeType: TeeType;
  /** MRENCLAVE (hex, 32 bytes) */
  mrEnclave: string;
  /** MRSIGNER (hex, 32 bytes) */
  mrSigner: string;
  /** ISV Product ID */
  isvProdId: number;
  /** ISV SVN (安全版本号) */
  isvSvn: number;
  /** 报告数据 (hex, 64 bytes) */
  reportData: string;
  /** IAS 签名 (base64) */
  iasSignature: string;
  /** 认证时间 (Unix timestamp) */
  timestamp: number;
}

/** 认证验证结果 */
export interface AttestationVerifyResult {
  /** 是否有效 */
  isValid: boolean;
  /** MRENCLAVE 是否匹配 */
  mrEnclaveMatch: boolean;
  /** 是否过期 */
  isExpired: boolean;
  /** 错误信息 */
  errorMessage: string | null;
}

// ============================================================================
// 审计日志
// ============================================================================

/** 审计事件类型 */
export enum AuditEventType {
  /** TEE 节点注册 */
  NodeRegistered = 0,
  /** TEE 节点注销 */
  NodeDeregistered = 1,
  /** 认证更新 */
  AttestationUpdated = 2,
  /** 计算请求提交 */
  ComputeRequestSubmitted = 3,
  /** 计算结果提交 */
  ComputeResultSubmitted = 4,
  /** 请求超时 */
  RequestTimeout = 5,
  /** 节点惩罚 */
  NodeSlashed = 6,
  /** MRENCLAVE 添加 */
  MrEnclaveAdded = 7,
  /** MRENCLAVE 移除 */
  MrEnclaveRemoved = 8,
  /** MRSIGNER 添加 */
  MrSignerAdded = 9,
  /** MRSIGNER 移除 */
  MrSignerRemoved = 10,
  /** 质押 */
  Staked = 11,
  /** 解除质押 */
  Unstaked = 12,
  /** 批量请求提交 */
  BatchRequestsSubmitted = 13,
  /** 批量结果提交 */
  BatchResultsSubmitted = 14,
}

/** 审计日志条目 */
export interface AuditLogEntry {
  /** 日志 ID */
  id: number;
  /** 事件类型 */
  eventType: AuditEventType;
  /** 相关账户 */
  account: string;
  /** 区块号 */
  blockNumber: number;
  /** 时间戳 (Unix timestamp) */
  timestamp: number;
  /** 相关数据哈希 (hex, 32 bytes) */
  dataHash: string;
  /** 操作是否成功 */
  success: boolean;
}
