# Design Document: 成为占卜师

## Overview

本设计文档描述"成为占卜师"功能的前端实现方案，基于 `pallet-divination-market` 链上模块，为用户提供完整的占卜师注册、管理和运营体验。

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Frontend (React Native)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  Pages                                                                      │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │ /diviner     │ │ /diviner/    │ │ /diviner/    │ │ /diviner/    │       │
│  │ /register    │ │ dashboard    │ │ packages     │ │ orders       │       │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘       │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │ /diviner/    │ │ /diviner/    │ │ /diviner/    │ │ /diviner/    │       │
│  │ reviews      │ │ earnings     │ │ profile      │ │ [providerId] │       │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘       │
├─────────────────────────────────────────────────────────────────────────────┤
│  Components                                                                 │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │ Registration │ │ Dashboard    │ │ Package      │ │ Order        │       │
│  │ Form         │ │ Stats        │ │ Card         │ │ Card         │       │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘       │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │ Tier         │ │ Review       │ │ Earnings     │ │ Specialty    │       │
│  │ Badge        │ │ Card         │ │ Chart        │ │ Selector     │       │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘       │
├─────────────────────────────────────────────────────────────────────────────┤
│  Store (Zustand)                                                            │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ divinerStore: provider, packages, orders, reviews, earnings      │      │
│  └──────────────────────────────────────────────────────────────────┘      │
├─────────────────────────────────────────────────────────────────────────────┤
│  Services                                                                   │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ DivinerService: API calls to pallet-divination-market            │      │
│  └──────────────────────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Substrate Runtime                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  pallet-divination-market                                                   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │ Providers    │ │ Packages     │ │ Orders       │ │ Reviews      │       │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘       │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. 类型定义

```typescript
// types/diviner.ts

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

/** 订单状态 */
export enum OrderStatus {
  PendingPayment = 'PendingPayment',
  Paid = 'Paid',
  Accepted = 'Accepted',
  Completed = 'Completed',
  Reviewed = 'Reviewed',
  Cancelled = 'Cancelled',
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
```

### 2. 核心组件

#### 2.1 注册表单组件

```typescript
// components/RegistrationForm.tsx
interface RegistrationFormProps {
  onSubmit: (data: RegistrationData) => Promise<void>;
  loading: boolean;
}

interface RegistrationData {
  name: string;
  bio: string;
  specialties: number;
  supportedTypes: number;
}
```

#### 2.2 等级徽章组件

```typescript
// components/TierBadge.tsx
interface TierBadgeProps {
  tier: ProviderTier;
  size?: 'small' | 'medium' | 'large';
}

const TIER_CONFIG = {
  [ProviderTier.Novice]: { label: '新手', color: '#8E8E93', icon: '🌱' },
  [ProviderTier.Certified]: { label: '认证', color: '#4CD964', icon: '✓' },
  [ProviderTier.Senior]: { label: '资深', color: '#007AFF', icon: '⭐' },
  [ProviderTier.Expert]: { label: '专家', color: '#5856D6', icon: '💎' },
  [ProviderTier.Master]: { label: '大师', color: '#B2955D', icon: '👑' },
};
```

#### 2.3 套餐卡片组件

```typescript
// components/PackageCard.tsx
interface PackageCardProps {
  package: ServicePackage;
  onEdit?: () => void;
  onToggle?: () => void;
  onDelete?: () => void;
  editable?: boolean;
}
```

#### 2.4 订单卡片组件

```typescript
// components/DivinerOrderCard.tsx
interface DivinerOrderCardProps {
  order: Order;
  onAccept?: () => void;
  onReject?: () => void;
  onSubmitAnswer?: () => void;
  onViewDetail?: () => void;
}
```

#### 2.5 仪表盘统计组件

```typescript
// components/DashboardStats.tsx
interface DashboardStatsProps {
  provider: Provider;
  pendingOrders: number;
  todayEarnings: bigint;
  monthlyEarnings: bigint;
}
```

### 3. 页面结构

```
frontend/app/diviner/
├── index.tsx           # 占卜师入口/注册引导
├── register.tsx        # 注册表单页
├── dashboard.tsx       # 仪表盘
├── packages/
│   ├── index.tsx       # 套餐列表
│   ├── create.tsx      # 创建套餐
│   └── [id].tsx        # 编辑套餐
├── orders/
│   ├── index.tsx       # 订单列表
│   └── [id].tsx        # 订单详情/处理
├── reviews.tsx         # 评价管理
├── earnings.tsx        # 收益管理
├── profile.tsx         # 资料编辑
└── [providerId].tsx    # 公开资料页
```

## Data Models

### 链上存储映射

| 链上存储 | 前端类型 | 说明 |
|---------|---------|------|
| `Providers<AccountId>` | `Provider` | 占卜师信息 |
| `Packages<AccountId, u32>` | `ServicePackage` | 服务套餐 |
| `Orders<u64>` | `Order` | 订单详情 |
| `Reviews<u64>` | `Review` | 评价详情 |
| `ProviderBalances<AccountId>` | `bigint` | 可提现余额 |
| `CustomerOrders<AccountId>` | `number[]` | 客户订单索引 |
| `ProviderOrders<AccountId>` | `number[]` | 占卜师订单索引 |

### 本地状态 (Zustand Store)

```typescript
interface DivinerStore {
  // 状态
  provider: Provider | null;
  packages: ServicePackage[];
  orders: Order[];
  reviews: Review[];
  balance: bigint;
  loading: boolean;
  
  // 操作
  fetchProvider: () => Promise<void>;
  registerProvider: (data: RegistrationData) => Promise<void>;
  updateProvider: (data: Partial<Provider>) => Promise<void>;
  pauseProvider: () => Promise<void>;
  resumeProvider: () => Promise<void>;
  
  fetchPackages: () => Promise<void>;
  createPackage: (data: CreatePackageData) => Promise<void>;
  updatePackage: (id: number, data: Partial<ServicePackage>) => Promise<void>;
  removePackage: (id: number) => Promise<void>;
  
  fetchOrders: (status?: OrderStatus) => Promise<void>;
  acceptOrder: (orderId: number) => Promise<void>;
  rejectOrder: (orderId: number) => Promise<void>;
  submitAnswer: (orderId: number, answerCid: string) => Promise<void>;
  answerFollowUp: (orderId: number, index: number, answerCid: string) => Promise<void>;
  
  fetchReviews: () => Promise<void>;
  replyReview: (orderId: number, replyCid: string) => Promise<void>;
  
  requestWithdrawal: (amount: bigint) => Promise<void>;
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Registration validation
*For any* registration submission, if name length is outside 1-64 chars OR bio length is outside 1-256 chars OR specialties is 0 OR supportedTypes is 0, the registration SHALL be rejected with appropriate error.
**Validates: Requirements 1.3, 1.4**

### Property 2: Deposit locking
*For any* successful registration, the user's balance SHALL decrease by exactly the minimum deposit amount (100 DUST).
**Validates: Requirements 1.5**

### Property 3: Status transitions
*For any* provider status change, the transition SHALL follow the valid state machine: Pending→Active, Active↔Paused, Active→Deactivated, Active→Banned.
**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

### Property 4: Package limit enforcement
*For any* diviner, the total number of active packages SHALL never exceed 10.
**Validates: Requirements 3.3**

### Property 5: Price validation
*For any* package creation or update, if price < minimum (1 DUST), the operation SHALL be rejected.
**Validates: Requirements 3.1**

### Property 6: Order acceptance timeout
*For any* order in Paid status, if 2 hours pass without acceptance, the order SHALL be auto-cancelled and customer refunded.
**Validates: Requirements 4.4**

### Property 7: Earnings calculation
*For any* completed order, provider earnings SHALL equal total_amount - (total_amount × platform_fee_rate / 10000).
**Validates: Requirements 4.6**

### Property 8: Follow-up limit
*For any* order, the number of follow-ups submitted SHALL never exceed the package's followUpCount.
**Validates: Requirements 5.3**

### Property 9: Rating bounds
*For any* review submission, all rating values (overall, accuracy, attitude, response) SHALL be between 1 and 5 inclusive.
**Validates: Requirements 6.2**

### Property 10: Tier auto-upgrade
*For any* provider meeting tier upgrade criteria (order count AND rating threshold), the tier SHALL be automatically upgraded.
**Validates: Requirements 6.4, 8.3**

### Property 11: Withdrawal validation
*For any* withdrawal request, if amount > available balance, the request SHALL be rejected.
**Validates: Requirements 7.4**

### Property 12: Platform fee by tier
*For any* completed order, the platform fee rate SHALL match the provider's tier: Novice=20%, Certified=15%, Senior=12%, Expert=10%, Master=8%.
**Validates: Requirements 8.2**

## Error Handling

| 错误场景 | 处理方式 | 用户提示 |
|---------|---------|---------|
| 余额不足注册 | 阻止提交 | "DUST 余额不足，需要至少 100 DUST 作为保证金" |
| 重复注册 | 阻止提交 | "您已经是占卜师，无需重复注册" |
| 套餐数量超限 | 阻止创建 | "套餐数量已达上限（10个），请删除旧套餐后再创建" |
| 订单超时 | 自动取消 | "订单已超时自动取消" |
| 提现金额超限 | 阻止提交 | "提现金额超过可用余额" |
| 网络错误 | 重试机制 | "网络连接失败，请重试" |
| 链上交易失败 | 显示错误 | "交易失败：{具体错误}" |

## Testing Strategy

### 单元测试
- 组件渲染测试
- 表单验证逻辑测试
- 状态计算函数测试
- 金额格式化测试

### 属性测试
- 使用 fast-check 进行属性测试
- 每个属性测试运行 100+ 次迭代
- 测试边界条件和随机输入

### 集成测试
- 注册流程端到端测试
- 订单处理流程测试
- 提现流程测试

## UI/UX Design

### 主题色
- 主色：#B2955D (金棕色)
- 背景：#F5F5F7
- 成功：#4CD964
- 警告：#FF9500
- 错误：#FF3B30

### 页面流程

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  入口页面   │────►│  注册表单   │────►│  等待审核   │
│  /diviner   │     │  /register  │     │  Pending    │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                                               ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  收益管理   │◄───►│  仪表盘     │◄────│  审核通过   │
│  /earnings  │     │  /dashboard │     │  Active     │
└─────────────┘     └──────┬──────┘     └─────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
       ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
       │  套餐管理   │ │  订单管理   │ │  评价管理   │
       │  /packages  │ │  /orders    │ │  /reviews   │
       └─────────────┘ └─────────────┘ └─────────────┘
```

### 关键界面

1. **注册页面**: 分步表单，清晰的字段说明，实时验证
2. **仪表盘**: 卡片式布局，关键指标突出，快捷操作入口
3. **订单列表**: 状态筛选，时间排序，批量操作
4. **套餐管理**: 网格展示，拖拽排序，快速编辑
5. **收益页面**: 图表展示，明细列表，提现入口
