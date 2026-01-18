// frontend/src/divination/market/types/market.types.ts

// 占卜类型枚举
export enum DivinationType {
  Meihua = 0,
  Bazi = 1,
  Liuyao = 2,
  Qimen = 3,
  Ziwei = 4,
  Tarot = 5,
  Daliuren = 6,
}

// 服务类型枚举
export enum ServiceType {
  TextReading = 0,
  VoiceReading = 1,
  VideoReading = 2,
  LiveConsultation = 3,
}

// 订单状态枚举
export type OrderStatus =
  | 'PendingPayment'
  | 'Paid'
  | 'Accepted'
  | 'Completed'
  | 'Cancelled'
  | 'Reviewed';

// 提供者状态枚举
export type ProviderStatus =
  | 'Pending'
  | 'Active'
  | 'Paused'
  | 'Banned'
  | 'Deactivated';

// 服务提供者
export interface Provider {
  account: string;
  name: string;
  bio: string;
  avatarCid?: string;
  specialties: number;        // 擅长领域位图
  supportedTypes: number;     // 支持的占卜类型位图
  tier: number;               // 等级 0-4
  status: ProviderStatus;
  totalRating: number;        // 总评分（累计）
  ratingCount: number;        // 评价数量
  completedOrders: number;    // 完成订单数
  acceptsUrgent: boolean;     // 是否接受加急
  registeredAt: number;       // 注册区块
  packages?: ServicePackage[]; // 套餐列表（前端扩展）
}

// 服务套餐
export interface ServicePackage {
  id: number;
  divinationType: DivinationType;
  serviceType: ServiceType;
  name: string;
  description: string;
  price: bigint;
  duration: number;           // 时长（分钟）
  followUpCount: number;      // 追问次数
  urgentAvailable: boolean;   // 是否支持加急
  urgentSurcharge: number;    // 加急加价（基点）
  isActive: boolean;
  salesCount: number;
  // 前端扩展字段
  divinationTypeName?: string;
  serviceTypeName?: string;
  divinationTypeRoute?: string;
}

// 订单
export interface Order {
  id: number;
  customer: string;
  provider: string;
  divinationType: DivinationType;
  hexagramId: number;
  packageId: number;
  questionCid: string;
  answerCid?: string;
  status: OrderStatus;
  amount: bigint;
  platformFee: bigint;
  isUrgent: boolean;
  createdAt: number;
  acceptedAt?: number;
  completedAt?: number;
  // 前端扩展字段
  providerName?: string;
  packageName?: string;
  divinationTypeName?: string;
  serviceTypeName?: string;
  question?: string;          // 解密后的内容
  answer?: string;            // 解密后的内容
  followUps?: FollowUp[];
}

// 追问
export interface FollowUp {
  questionCid: string;
  question?: string;          // 解析后的内容
  answerCid?: string;
  answer?: string;            // 解析后的内容
  createdAt: number;
  answeredAt?: number;
}

// 评价
export interface Review {
  orderId: number;
  customer: string;
  customerName?: string;
  overallRating: number;
  accuracyRating: number;
  attitudeRating: number;
  responseRating: number;
  contentCid?: string;
  content?: string;           // 解析后的内容
  isAnonymous: boolean;
  replyCid?: string;
  reply?: string;             // 解析后的内容
  createdAt: number;
  repliedAt?: number;
}

// 提现记录
export interface WithdrawalRequest {
  id: number;
  provider: string;
  amount: bigint;
  status: 'Pending' | 'Completed' | 'Failed';
  createdAt: number;
  completedAt?: number;
}

// 提供者工作台数据
export interface ProviderDashboard {
  provider: Provider;
  balance: bigint;
  todayStats: {
    newOrders: number;
    completed: number;
    earnings: bigint;
  };
  pendingOrders: Order[];
  activeOrders: Order[];
}

// 卦象数据（用于订单关联）
export interface Hexagram {
  id: number;
  type: DivinationType;
  name: string;
  symbol?: string;
  question?: string;
  createdAt: number;
}

// 筛选参数
export interface ProviderFilterParams {
  filterType?: 'all' | number;
  sortType?: 'comprehensive' | 'rating' | 'orders' | 'price';
  keyword?: string;
}

// 创建订单参数
export interface CreateOrderParams {
  provider: string;
  divinationType: DivinationType;
  hexagramId: number;
  packageId: number;
  question: string;
  isUrgent: boolean;
  questionCid?: string;
  encryptionSessionId?: number;
}

// 提交评价参数
export interface SubmitReviewParams {
  orderId: number;
  overallRating: number;
  accuracyRating: number;
  attitudeRating: number;
  responseRating: number;
  content?: string;
  isAnonymous: boolean;
}
