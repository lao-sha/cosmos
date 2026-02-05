# COSMOS UI 颜色规范

> 版本：1.0 | 适用：React Native + Expo

---

## 一、品牌色

### 主色调 (Primary)

| 名称 | HEX | RGB | 用途 |
|------|-----|-----|------|
| **Primary** | `#6366F1` | 99, 102, 241 | 主按钮、链接、选中态 |
| **Primary Light** | `#818CF8` | 129, 140, 248 | 悬停态、浅色背景 |
| **Primary Dark** | `#4F46E5` | 79, 70, 229 | 按下态、强调 |

### 辅助色 (Secondary)

| 名称 | HEX | RGB | 用途 |
|------|-----|-----|------|
| **Secondary** | `#8B5CF6` | 139, 92, 246 | 次要按钮、标签 |
| **Accent** | `#F59E0B` | 245, 158, 11 | 金币、奖励、VIP |

---

## 二、语义色

### 状态色

| 状态 | HEX | 用途 |
|------|-----|------|
| **Success** | `#10B981` | 成功、完成、上涨 |
| **Warning** | `#F59E0B` | 警告、待处理 |
| **Error** | `#EF4444` | 错误、失败、下跌 |
| **Info** | `#3B82F6` | 提示、信息 |

### 交易色

| 场景 | HEX | 用途 |
|------|-----|------|
| **Buy/涨** | `#10B981` | 买入、价格上涨 |
| **Sell/跌** | `#EF4444` | 卖出、价格下跌 |
| **Pending** | `#F59E0B` | 待确认、处理中 |

---

## 三、中性色

### 亮色模式 (Light)

| 名称 | HEX | 用途 |
|------|-----|------|
| **Background** | `#FFFFFF` | 页面背景 |
| **Surface** | `#F9FAFB` | 卡片背景 |
| **Border** | `#E5E7EB` | 边框、分割线 |
| **Text Primary** | `#111827` | 主文字 |
| **Text Secondary** | `#6B7280` | 次要文字 |
| **Text Tertiary** | `#9CA3AF` | 占位符、禁用 |

### 暗色模式 (Dark)

| 名称 | HEX | 用途 |
|------|-----|------|
| **Background** | `#0F172A` | 页面背景 |
| **Surface** | `#1E293B` | 卡片背景 |
| **Border** | `#334155` | 边框、分割线 |
| **Text Primary** | `#F8FAFC` | 主文字 |
| **Text Secondary** | `#94A3B8` | 次要文字 |
| **Text Tertiary** | `#64748B` | 占位符、禁用 |

---

## 四、功能模块色

### 钱包模块

| 元素 | 亮色 | 暗色 |
|------|------|------|
| COS 代币 | `#6366F1` | `#818CF8` |
| USDT | `#10B981` | `#34D399` |
| 余额卡片 | 渐变 `#6366F1` → `#8B5CF6` | 同 |

### 交易模块

| 元素 | 色值 |
|------|------|
| 买入按钮 | `#10B981` |
| 卖出按钮 | `#EF4444` |
| 订单进行中 | `#F59E0B` |
| 订单完成 | `#10B981` |
| 订单取消 | `#6B7280` |

### 聊天模块

| 元素 | 色值 |
|------|------|
| 我的消息气泡 | `#6366F1` |
| 对方消息气泡 | `#F3F4F6` (亮) / `#374151` (暗) |
| 已读标记 | `#10B981` |
| 未读角标 | `#EF4444` |

### 婚恋模块

| 元素 | 色值 |
|------|------|
| 喜欢 | `#EC4899` (粉红) |
| 超级喜欢 | `#3B82F6` (蓝) |
| 配对成功 | 渐变 `#EC4899` → `#F59E0B` |

### 占卜/玄学模块 🆕

| 元素 | 色值 | 说明 |
|------|------|------|
| 主色调 | `#7C3AED` | 紫色（神秘感）|
| 辅助色 | `#C4B5FD` | 淡紫 |
| 金色点缀 | `#D4AF37` | 古铜金（东方美学）|
| 八字/命理 | `#B45309` | 琥珀色 |
| 吉 | `#DC2626` | 中国红 |
| 凶 | `#1F2937` | 墨黑 |
| 背景 | `#1E1B4B` | 深紫暗色 |

### 会员等级色系 🆕

| 等级 | 色值 | 图标色 |
|------|------|--------|
| 普通 | `#6B7280` | 灰 |
| 青铜 | `#CD7F32` | 铜 |
| 白银 | `#C0C0C0` | 银 |
| 黄金 | `#FFD700` | 金 |
| 铂金 | `#E5E4E2` | 铂 |
| 钻石 | `#B9F2FF` | 蓝白 |
| 至尊 | 渐变 `#FFD700` → `#FF6B6B` | 金红 |

### KYC 认证状态 🆕

| 状态 | 色值 | 图标 |
|------|------|------|
| 未认证 | `#9CA3AF` | 灰色盾牌 |
| 待审核 | `#F59E0B` | 黄色时钟 |
| 已认证 | `#10B981` | 绿色勾选 |
| 已拒绝 | `#EF4444` | 红色叉号 |
| 增强认证 | `#6366F1` | 紫色星标 |

### 争议仲裁模块 🆕

| 状态 | 色值 | 说明 |
|------|------|------|
| 待处理 | `#F59E0B` | 等待响应 |
| 证据收集 | `#3B82F6` | 双方举证 |
| 仲裁中 | `#8B5CF6` | 仲裁员审理 |
| 已裁决 | `#10B981` | 已出结果 |
| 已上诉 | `#EF4444` | 申诉中 |

### 推荐层级色系 🆕

| 层级 | 色值 | 透明度 |
|------|------|--------|
| L1 (直推) | `#6366F1` | 100% |
| L2 | `#818CF8` | 90% |
| L3 | `#A5B4FC` | 80% |
| L4-L5 | `#C7D2FE` | 70% |
| L6-L10 | `#E0E7FF` | 60% |
| L11-L15 | `#EEF2FF` | 50% |

### Entity/商城模块 🆕

| 元素 | 色值 |
|------|------|
| 店铺标识 | `#059669` (绿) |
| 商品标签 | `#0891B2` (青) |
| 促销/折扣 | `#DC2626` (红) |
| 已售罄 | `#9CA3AF` (灰) |
| 新品 | `#F59E0B` (橙) |
| 热卖 | `#EF4444` (红) |

### 治理投票模块 🆕

| 状态 | 色值 |
|------|------|
| 提案中 | `#3B82F6` |
| 投票中 | `#8B5CF6` |
| 已通过 | `#10B981` |
| 已否决 | `#EF4444` |
| 已执行 | `#6B7280` |
| 赞成票 | `#10B981` |
| 反对票 | `#EF4444` |
| 弃权票 | `#F59E0B` |

---

## 五、代码实现

### constants/colors.ts

```typescript
export const Colors = {
  // Primary
  primary: '#6366F1',
  primaryLight: '#818CF8',
  primaryDark: '#4F46E5',
  
  // Secondary
  secondary: '#8B5CF6',
  accent: '#F59E0B',
  
  // Semantic
  success: '#10B981',
  warning: '#F59E0B',
  error: '#EF4444',
  info: '#3B82F6',
  
  // Trading
  trading: {
    buy: '#10B981',
    sell: '#EF4444',
    pending: '#F59E0B',
    completed: '#10B981',
    cancelled: '#6B7280',
  },
  
  // 会员等级
  membership: {
    normal: '#6B7280',
    bronze: '#CD7F32',
    silver: '#C0C0C0',
    gold: '#FFD700',
    platinum: '#E5E4E2',
    diamond: '#B9F2FF',
    supreme: ['#FFD700', '#FF6B6B'], // 渐变
  },
  
  // KYC 认证
  kyc: {
    none: '#9CA3AF',
    pending: '#F59E0B',
    verified: '#10B981',
    rejected: '#EF4444',
    enhanced: '#6366F1',
  },
  
  // 争议仲裁
  dispute: {
    pending: '#F59E0B',
    evidence: '#3B82F6',
    arbitrating: '#8B5CF6',
    resolved: '#10B981',
    appealed: '#EF4444',
  },
  
  // 推荐层级 (15层)
  referral: {
    l1: '#6366F1',
    l2: '#818CF8',
    l3: '#A5B4FC',
    l4: '#C7D2FE',
    l5: '#E0E7FF',
    l6Plus: '#EEF2FF',
  },
  
  // 商城
  shop: {
    store: '#059669',
    product: '#0891B2',
    discount: '#DC2626',
    soldOut: '#9CA3AF',
    newArrival: '#F59E0B',
    hotSale: '#EF4444',
  },
  
  // 治理投票
  governance: {
    proposing: '#3B82F6',
    voting: '#8B5CF6',
    passed: '#10B981',
    rejected: '#EF4444',
    executed: '#6B7280',
    voteYes: '#10B981',
    voteNo: '#EF4444',
    voteAbstain: '#F59E0B',
  },
  
  // 占卜玄学
  divination: {
    primary: '#7C3AED',
    secondary: '#C4B5FD',
    gold: '#D4AF37',
    bazi: '#B45309',
    lucky: '#DC2626',
    unlucky: '#1F2937',
    background: '#1E1B4B',
  },
  
  // 聊天
  chat: {
    myBubble: '#6366F1',
    theirBubbleLight: '#F3F4F6',
    theirBubbleDark: '#374151',
    read: '#10B981',
    unread: '#EF4444',
  },
  
  // 婚恋
  matchmaking: {
    like: '#EC4899',
    superLike: '#3B82F6',
    match: ['#EC4899', '#F59E0B'], // 渐变
  },
  
  // Light Mode
  light: {
    background: '#FFFFFF',
    surface: '#F9FAFB',
    border: '#E5E7EB',
    textPrimary: '#111827',
    textSecondary: '#6B7280',
    textTertiary: '#9CA3AF',
  },
  
  // Dark Mode
  dark: {
    background: '#0F172A',
    surface: '#1E293B',
    border: '#334155',
    textPrimary: '#F8FAFC',
    textSecondary: '#94A3B8',
    textTertiary: '#64748B',
  },
};

// 类型定义
export type MembershipLevel = keyof typeof Colors.membership;
export type KycStatus = keyof typeof Colors.kyc;
export type DisputeStatus = keyof typeof Colors.dispute;
export type GovernanceStatus = keyof typeof Colors.governance;
```

### hooks/useColors.ts

```typescript
import { useColorScheme } from 'react-native';
import { Colors } from '@/constants/colors';

export function useColors() {
  const scheme = useColorScheme();
  const isDark = scheme === 'dark';
  
  return {
    ...Colors,
    ...(isDark ? Colors.dark : Colors.light),
    isDark,
  };
}
```

---

## 六、设计原则

### 对比度

- 文字与背景对比度 ≥ 4.5:1 (WCAG AA)
- 大文字对比度 ≥ 3:1
- 使用工具检查：[WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)

### 一致性

- 同一语义使用同一颜色
- 成功/错误/警告色全局统一
- 交易涨跌色保持一致

### 无障碍

- 不仅靠颜色传达信息
- 配合图标、文字说明
- 支持系统深色模式

---

## 七、渐变色

```typescript
export const Gradients = {
  // 主渐变（钱包卡片）
  primary: ['#6366F1', '#8B5CF6'],
  
  // 金色渐变（VIP、奖励）
  gold: ['#F59E0B', '#FBBF24'],
  
  // 成功渐变
  success: ['#10B981', '#34D399'],
  
  // 婚恋配对
  love: ['#EC4899', '#F59E0B'],
};
```

---

## 八、阴影

```typescript
export const Shadows = {
  sm: {
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 1,
  },
  md: {
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.1,
    shadowRadius: 6,
    elevation: 3,
  },
  lg: {
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 10 },
    shadowOpacity: 0.15,
    shadowRadius: 15,
    elevation: 5,
  },
};
```
