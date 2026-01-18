/**
 * 占卜师模块类型定义
 * 基于 pallet-divination-market
 */

/** 占卜师状态 */
export enum ProviderStatus {
  Pending = 'Pending',
  Active = 'Active',
  Paused = 'Paused',
  Banned = 'Banned',
  Deactivated = 'Deactivated',
}

/** 占卜师等级 */
export enum ProviderTier {
  Novice = 0,      // 新手 20%
  Certified = 1,   // 认证 15%
  Senior = 2,      // 资深 12%
  Expert = 3,      // 专家 10%
  Master = 4,      // 大师 8%
}

/** 占卜类型 */
export enum DivinationType {
  Meihua = 0,      // 梅花易数
  Bazi = 1,        // 八字命理
  Liuyao = 2,      // 六爻
  Qimen = 3,       // 奇门遁甲
  Ziwei = 4,       // 紫微斗数
  Tarot = 5,       // 塔罗牌
  Daliuren = 6,    // 大六壬
}

/** 服务类型 */
export enum ServiceType {
  TextReading = 0,      // 文字解卦
  VoiceReading = 1,     // 语音解卦
  VideoReading = 2,     // 视频解卦
  LiveConsultation = 3, // 实时咨询
}

/** 擅长领域位图 */
export enum Specialty {
  Career = 1 << 0,       // 事业运势
  Relationship = 1 << 1, // 感情婚姻
  Wealth = 1 << 2,       // 财运投资
  Health = 1 << 3,       // 健康养生
  Education = 1 << 4,    // 学业考试
  Travel = 1 << 5,       // 出行旅游
  Legal = 1 << 6,        // 官司诉讼
  Finding = 1 << 7,      // 寻人寻物
  FengShui = 1 << 8,     // 风水堪舆
  DateSelection = 1 << 9, // 择日选时
}

/** 订单状态 */
export enum OrderStatus {
  PendingPayment = 'PendingPayment',
  Paid = 'Paid',
  Accepted = 'Accepted',
  Completed = 'Completed',
  Reviewed = 'Reviewed',
  Cancelled = 'Cancelled',
}

/** 占卜师信息 */
export interface Provider {
  account: string;
  name: string;
  bio: string;
  avatarCid?: string;
  specialties: number;
  supportedTypes: number;
  status: ProviderStatus;
  tier: ProviderTier;
  totalOrders: number;
  completedOrders: number;
  totalEarnings: bigint;
  averageRating: number;
  ratingCount: number;
  acceptsUrgent: boolean;
  registeredAt: number;
}

/** 服务套餐 */
export interface ServicePackage {
  id: number;
  providerId: string;
  divinationType: DivinationType;
  serviceType: ServiceType;
  name: string;
  description: string;
  price: bigint;
  duration: number;
  followUpCount: number;
  urgentAvailable: boolean;
  urgentSurcharge: number;
  isActive: boolean;
  salesCount: number;
}

/** 订单 */
export interface Order {
  id: number;
  customer: string;
  provider: string;
  packageId: number;
  divinationType: DivinationType;
  divinationResultId?: number;
  questionCid: string;
  answerCid?: string;
  totalAmount: bigint;
  platformFee: bigint;
  providerEarnings: bigint;
  isUrgent: boolean;
  status: OrderStatus;
  createdAt: number;
  acceptedAt?: number;
  completedAt?: number;
  followUpsUsed: number;
  followUpsTotal: number;
}

/** 追问记录 */
export interface FollowUp {
  questionCid: string;
  answerCid?: string;
  createdAt: number;
  answeredAt?: number;
}

/** 评价 */
export interface Review {
  orderId: number;
  customer: string;
  provider: string;
  overallRating: number;
  accuracyRating: number;
  attitudeRating: number;
  responseRating: number;
  contentCid?: string;
  isAnonymous: boolean;
  replyCid?: string;
  createdAt: number;
}

/** 提现记录 */
export interface WithdrawalRecord {
  id: number;
  provider: string;
  amount: bigint;
  createdAt: number;
  completedAt?: number;
}

/** 注册数据 */
export interface RegistrationData {
  name: string;
  bio: string;
  specialties: number;
  supportedTypes: number;
}

/** 创建套餐数据 */
export interface CreatePackageData {
  divinationType: DivinationType;
  serviceType: ServiceType;
  name: string;
  description: string;
  price: bigint;
  duration: number;
  followUpCount: number;
  urgentAvailable: boolean;
  urgentSurcharge: number;
}

/** 等级配置 */
export const TIER_CONFIG = {
  [ProviderTier.Novice]: { label: '新手', color: '#8E8E93', icon: '🌱', feeRate: 20 },
  [ProviderTier.Certified]: { label: '认证', color: '#4CD964', icon: '✓', feeRate: 15 },
  [ProviderTier.Senior]: { label: '资深', color: '#007AFF', icon: '⭐', feeRate: 12 },
  [ProviderTier.Expert]: { label: '专家', color: '#5856D6', icon: '💎', feeRate: 10 },
  [ProviderTier.Master]: { label: '大师', color: '#B2955D', icon: '👑', feeRate: 8 },
};

/** 状态配置 */
export const STATUS_CONFIG = {
  [ProviderStatus.Pending]: { label: '待审核', color: '#FF9500' },
  [ProviderStatus.Active]: { label: '已激活', color: '#4CD964' },
  [ProviderStatus.Paused]: { label: '已暂停', color: '#8E8E93' },
  [ProviderStatus.Banned]: { label: '已封禁', color: '#FF3B30' },
  [ProviderStatus.Deactivated]: { label: '已注销', color: '#8E8E93' },
};

/** 占卜类型配置 */
export const DIVINATION_TYPE_CONFIG = {
  [DivinationType.Meihua]: { label: '梅花易数', icon: '🌸' },
  [DivinationType.Bazi]: { label: '八字命理', icon: '📅' },
  [DivinationType.Liuyao]: { label: '六爻', icon: '☰' },
  [DivinationType.Qimen]: { label: '奇门遁甲', icon: '🚪' },
  [DivinationType.Ziwei]: { label: '紫微斗数', icon: '⭐' },
  [DivinationType.Tarot]: { label: '塔罗牌', icon: '🃏' },
  [DivinationType.Daliuren]: { label: '大六壬', icon: '🔮' },
};

/** 服务类型配置 */
export const SERVICE_TYPE_CONFIG = {
  [ServiceType.TextReading]: { label: '文字解卦', icon: '📝' },
  [ServiceType.VoiceReading]: { label: '语音解卦', icon: '🎙️' },
  [ServiceType.VideoReading]: { label: '视频解卦', icon: '📹' },
  [ServiceType.LiveConsultation]: { label: '实时咨询', icon: '💬' },
};

/** 擅长领域配置 */
export const SPECIALTY_CONFIG = {
  [Specialty.Career]: { label: '事业运势', icon: '💼' },
  [Specialty.Relationship]: { label: '感情婚姻', icon: '💕' },
  [Specialty.Wealth]: { label: '财运投资', icon: '💰' },
  [Specialty.Health]: { label: '健康养生', icon: '🏥' },
  [Specialty.Education]: { label: '学业考试', icon: '📚' },
  [Specialty.Travel]: { label: '出行旅游', icon: '✈️' },
  [Specialty.Legal]: { label: '官司诉讼', icon: '⚖️' },
  [Specialty.Finding]: { label: '寻人寻物', icon: '🔍' },
  [Specialty.FengShui]: { label: '风水堪舆', icon: '🏠' },
  [Specialty.DateSelection]: { label: '择日选时', icon: '📆' },
};

/** 订单状态配置 */
export const ORDER_STATUS_CONFIG = {
  [OrderStatus.PendingPayment]: { label: '待支付', color: '#FF9500' },
  [OrderStatus.Paid]: { label: '已支付', color: '#007AFF' },
  [OrderStatus.Accepted]: { label: '已接单', color: '#5856D6' },
  [OrderStatus.Completed]: { label: '已完成', color: '#4CD964' },
  [OrderStatus.Reviewed]: { label: '已评价', color: '#4CD964' },
  [OrderStatus.Cancelled]: { label: '已取消', color: '#8E8E93' },
};
