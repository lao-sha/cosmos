# Divination Market 前端页面设计方案

> pallet-divination-market 占卜服务市场前端实现指南

## 目录

1. [设计概述](#设计概述)
2. [主题色系](#主题色系)
3. [页面结构](#页面结构)
4. [市场首页](#1-市场首页)
5. [提供者详情页](#2-提供者详情页)
6. [创建订单页](#3-创建订单页)
7. [订单详情页](#4-订单详情页)
8. [提供者工作台](#5-提供者工作台)
9. [评价页面](#6-评价页面)
10. [通用组件](#7-通用组件)
11. [API 集成](#8-api-集成)

---

## 设计概述

基于现有项目的金棕色主题和 React Native + Expo Router 技术栈，设计占卜服务市场的完整前端页面。

### 技术栈

- React Native 0.81+
- Expo Router 6.x
- @polkadot/api 16.x
- Zustand (状态管理)
- TypeScript

### 设计原则

- 保持与现有占卜页面风格一致
- 移动端优先，最大宽度 414px
- 卡片式布局，圆角 8-12px
- 清晰的状态反馈和交互提示

---

## 主题色系

```typescript
// frontend/src/divination/market/theme.ts

export const THEME = {
  // 主色调
  primary: '#B2955D',        // 主色-金棕
  primaryLight: '#F7D3A1',   // 浅金
  primaryDark: '#8B6914',    // 深金
  
  // 背景色
  background: '#F5F5F7',     // 页面背景
  card: '#FFFFFF',           // 卡片背景
  
  // 文字色
  text: '#333333',           // 主文字
  textSecondary: '#666666',  // 次要文字
  textTertiary: '#999999',   // 辅助文字
  textInverse: '#FFFFFF',    // 反色文字
  
  // 边框色
  border: '#E8E8E8',         // 默认边框
  borderLight: '#F0F0F0',    // 浅边框
  
  // 状态色
  success: '#52C41A',        // 成功
  warning: '#FAAD14',        // 警告
  error: '#FF4D4F',          // 错误
  info: '#1890FF',           // 信息
  
  // 等级色
  tier: {
    novice: '#999999',       // 新手-灰
    certified: '#52C41A',    // 认证-绿
    senior: '#1890FF',       // 资深-蓝
    expert: '#722ED1',       // 专家-紫
    master: '#EB2F96',       // 大师-粉
  },
  
  // 订单状态色
  orderStatus: {
    pending: '#FAAD14',      // 待处理-黄
    paid: '#1890FF',         // 已支付-蓝
    accepted: '#722ED1',     // 已接单-紫
    completed: '#52C41A',    // 已完成-绿
    cancelled: '#999999',    // 已取消-灰
    reviewed: '#EB2F96',     // 已评价-粉
  },
  
  // 占卜类型色
  divinationType: {
    meihua: '#E91E63',       // 梅花易数
    bazi: '#E74C3C',         // 八字
    liuyao: '#F39C12',       // 六爻
    qimen: '#3498DB',        // 奇门
    ziwei: '#9B59B6',        // 紫微
    tarot: '#673AB7',        // 塔罗
    daliuren: '#1ABC9C',     // 大六壬
  },
};

// 阴影样式
export const SHADOWS = {
  small: {
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.04,
    shadowRadius: 4,
    elevation: 2,
  },
  medium: {
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 3,
  },
  large: {
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.08,
    shadowRadius: 12,
    elevation: 4,
  },
};
```

---

## 页面结构

```
frontend/app/market/
├── index.tsx                 # 市场首页（服务提供者列表）
├── search.tsx                # 搜索页面
├── provider/
│   ├── [id].tsx              # 提供者详情页
│   ├── register.tsx          # 注册成为提供者
│   ├── dashboard.tsx         # 提供者工作台
│   ├── packages.tsx          # 套餐管理
│   ├── earnings.tsx          # 收益管理
│   └── settings.tsx          # 提供者设置
├── order/
│   ├── create.tsx            # 创建订单
│   ├── [id].tsx              # 订单详情
│   ├── list.tsx              # 我的订单列表
│   └── submit-answer.tsx     # 提交解读（提供者）
└── review/
    ├── create.tsx            # 提交评价
    └── list.tsx              # 评价列表

frontend/src/divination/market/
├── components/               # 市场相关组件
│   ├── ProviderCard.tsx      # 提供者卡片
│   ├── PackageCard.tsx       # 套餐卡片
│   ├── OrderCard.tsx         # 订单卡片
│   ├── ReviewCard.tsx        # 评价卡片
│   ├── TierBadge.tsx         # 等级徽章
│   ├── RatingStars.tsx       # 评分星星
│   ├── OrderTimeline.tsx     # 订单时间线
│   ├── PriceDisplay.tsx      # 价格显示
│   └── SpecialtyTags.tsx     # 擅长领域标签
├── hooks/                    # 自定义 Hooks
│   ├── useMarketApi.ts       # 市场 API 调用
│   ├── useProvider.ts        # 提供者数据
│   ├── useOrders.ts          # 订单数据
│   └── useReviews.ts         # 评价数据
├── stores/                   # 状态管理
│   └── market.store.ts       # 市场状态
├── types/                    # 类型定义
│   └── market.types.ts       # 市场相关类型
├── utils/                    # 工具函数
│   └── market.utils.ts       # 格式化等工具
├── constants/                # 常量定义
│   └── market.constants.ts   # 配置常量
└── FRONTEND_DESIGN.md        # 本文档
```

---

## 1. 市场首页

### 页面线框图

```
┌─────────────────────────────────────┐
│  占卜服务市场                    🔍  │  ← 顶部标题栏
├─────────────────────────────────────┤
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐   │
│  │全部 │ │梅花 │ │八字 │ │六爻 │→  │  ← 占卜类型筛选（横向滚动）
│  └─────┘ └─────┘ └─────┘ └─────┘   │
├─────────────────────────────────────┤
│  排序: 综合 ▼    筛选 ▼             │  ← 排序筛选栏
├─────────────────────────────────────┤
│  ┌─────────────────────────────────┐│
│  │ ┌────┐                          ││
│  │ │ 👤 │ 张三大师        ⭐ 4.8   ││  ← 提供者卡片
│  │ └────┘ 🏅专家 · 156单完成       ││
│  │                                 ││
│  │ 梅花易数 · 八字命理             ││  ← 支持的占卜类型
│  │                                 ││
│  │ 从业20年，专注事业财运分析...   ││  ← 简介
│  │                                 ││
│  │ ┌──────────┐ ┌──────────┐      ││
│  │ │文字解卦  │ │语音解卦  │      ││  ← 套餐预览
│  │ │ 10 DUST  │ │ 30 DUST  │      ││
│  │ └──────────┘ └──────────┘      ││
│  └─────────────────────────────────┘│
│                                     │
│  ┌─────────────────────────────────┐│
│  │ ... 更多提供者卡片              ││
│  └─────────────────────────────────┘│
│                                     │
├─────────────────────────────────────┤
│  ┌─────────────────────────────────┐│
│  │  💼 成为服务提供者，开启副业    ││  ← 底部入口卡片
│  │  [立即入驻]                     ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

### 核心组件代码

```tsx
// frontend/src/divination/market/components/ProviderCard.tsx

import React from 'react';
import { View, Text, StyleSheet, Pressable, ScrollView } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '../theme';
import { TierBadge } from './TierBadge';
import { RatingStars } from './RatingStars';
import { Provider, ServicePackage } from '../types/market.types';
import { formatBalance, getDivinationTypeNames } from '../utils/market.utils';

interface ProviderCardProps {
  provider: Provider;
  packages: ServicePackage[];
}

export const ProviderCard: React.FC<ProviderCardProps> = ({ provider, packages }) => {
  const router = useRouter();

  const handlePress = () => {
    router.push(`/market/provider/${provider.account}`);
  };

  return (
    <Pressable style={styles.card} onPress={handlePress}>
      {/* 头部区域 */}
      <View style={styles.header}>
        <View style={styles.avatar}>
          <Text style={styles.avatarText}>
            {provider.name.charAt(0)}
          </Text>
        </View>
        <View style={styles.headerInfo}>
          <View style={styles.nameRow}>
            <Text style={styles.name}>{provider.name}</Text>
            <TierBadge tier={provider.tier} />
          </View>
          <Text style={styles.stats}>
            {provider.completedOrders}单完成
          </Text>
        </View>
        <View style={styles.ratingBox}>
          <Ionicons name="star" size={14} color={THEME.warning} />
          <Text style={styles.ratingText}>
            {(provider.totalRating / provider.ratingCount / 100).toFixed(1)}
          </Text>
        </View>
      </View>

      {/* 支持的占卜类型 */}
      <View style={styles.typeTags}>
        {getDivinationTypeNames(provider.supportedTypes).map((type, index) => (
          <View key={index} style={styles.typeTag}>
            <Text style={styles.typeTagText}>{type}</Text>
          </View>
        ))}
      </View>

      {/* 简介 */}
      <Text style={styles.bio} numberOfLines={2}>
        {provider.bio}
      </Text>

      {/* 套餐预览 */}
      <ScrollView 
        horizontal 
        showsHorizontalScrollIndicator={false}
        style={styles.packagesScroll}
      >
        <View style={styles.packagesRow}>
          {packages.slice(0, 3).map((pkg) => (
            <View key={pkg.id} style={styles.packageMini}>
              <Text style={styles.packageName} numberOfLines={1}>
                {pkg.name}
              </Text>
              <Text style={styles.packagePrice}>
                {formatBalance(pkg.price)} DUST
              </Text>
            </View>
          ))}
          {packages.length > 3 && (
            <View style={styles.morePackages}>
              <Text style={styles.moreText}>+{packages.length - 3}</Text>
            </View>
          )}
        </View>
      </ScrollView>
    </Pressable>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    ...SHADOWS.medium,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  avatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: THEME.primary + '20',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    fontSize: 20,
    fontWeight: '600',
    color: THEME.primary,
  },
  headerInfo: {
    flex: 1,
    marginLeft: 12,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  name: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.text,
  },
  stats: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 2,
  },
  ratingBox: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    paddingHorizontal: 8,
    paddingVertical: 4,
    backgroundColor: THEME.warning + '15',
    borderRadius: 4,
  },
  ratingText: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.warning,
  },
  typeTags: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 6,
    marginBottom: 10,
  },
  typeTag: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    backgroundColor: THEME.primary + '15',
    borderRadius: 4,
  },
  typeTagText: {
    fontSize: 12,
    color: THEME.primary,
  },
  bio: {
    fontSize: 14,
    color: THEME.textSecondary,
    lineHeight: 20,
    marginBottom: 12,
  },
  packagesScroll: {
    marginHorizontal: -16,
    paddingHorizontal: 16,
  },
  packagesRow: {
    flexDirection: 'row',
    gap: 8,
  },
  packageMini: {
    minWidth: 90,
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderWidth: 1,
    borderColor: THEME.border,
    borderRadius: 6,
    alignItems: 'center',
  },
  packageName: {
    fontSize: 12,
    color: THEME.text,
    marginBottom: 2,
  },
  packagePrice: {
    fontSize: 13,
    fontWeight: '600',
    color: THEME.primary,
  },
  morePackages: {
    width: 40,
    justifyContent: 'center',
    alignItems: 'center',
  },
  moreText: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
});
```

### 市场首页完整代码

```tsx
// frontend/app/market/index.tsx

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { ProviderCard } from '@/divination/market/components/ProviderCard';
import { BottomNavBar } from '@/components/BottomNavBar';
import { useMarketApi } from '@/divination/market/hooks/useMarketApi';
import { DIVINATION_TYPES } from '@/divination/market/constants/market.constants';

// 筛选类型
type FilterType = 'all' | number;
type SortType = 'comprehensive' | 'rating' | 'orders' | 'price';

export default function MarketIndexPage() {
  const router = useRouter();
  const { getProviders, loading, error } = useMarketApi();
  
  const [providers, setProviders] = useState([]);
  const [refreshing, setRefreshing] = useState(false);
  const [filterType, setFilterType] = useState<FilterType>('all');
  const [sortType, setSortType] = useState<SortType>('comprehensive');

  useEffect(() => {
    loadProviders();
  }, [filterType, sortType]);

  const loadProviders = async () => {
    const data = await getProviders({ filterType, sortType });
    setProviders(data);
  };

  const onRefresh = async () => {
    setRefreshing(true);
    await loadProviders();
    setRefreshing(false);
  };

  return (
    <View style={styles.container}>
      {/* 顶部标题栏 */}
      <View style={styles.header}>
        <Text style={styles.title}>占卜服务市场</Text>
        <Pressable 
          style={styles.searchButton}
          onPress={() => router.push('/market/search')}
        >
          <Ionicons name="search-outline" size={22} color={THEME.text} />
        </Pressable>
      </View>

      {/* 占卜类型筛选 */}
      <ScrollView 
        horizontal 
        showsHorizontalScrollIndicator={false}
        style={styles.filterScroll}
        contentContainerStyle={styles.filterContent}
      >
        <Pressable
          style={[
            styles.filterTag,
            filterType === 'all' && styles.filterTagActive,
          ]}
          onPress={() => setFilterType('all')}
        >
          <Text style={[
            styles.filterText,
            filterType === 'all' && styles.filterTextActive,
          ]}>
            全部
          </Text>
        </Pressable>
        {DIVINATION_TYPES.map((type) => (
          <Pressable
            key={type.id}
            style={[
              styles.filterTag,
              filterType === type.id && styles.filterTagActive,
            ]}
            onPress={() => setFilterType(type.id)}
          >
            <Text style={[
              styles.filterText,
              filterType === type.id && styles.filterTextActive,
            ]}>
              {type.name}
            </Text>
          </Pressable>
        ))}
      </ScrollView>

      {/* 排序栏 */}
      <View style={styles.sortBar}>
        <Pressable style={styles.sortItem}>
          <Text style={styles.sortText}>综合排序</Text>
          <Ionicons name="chevron-down" size={16} color={THEME.textSecondary} />
        </Pressable>
        <Pressable style={styles.sortItem}>
          <Text style={styles.sortText}>筛选</Text>
          <Ionicons name="options-outline" size={16} color={THEME.textSecondary} />
        </Pressable>
      </View>

      {/* 提供者列表 */}
      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
      >
        {loading && !refreshing ? (
          <View style={styles.loadingBox}>
            <ActivityIndicator size="large" color={THEME.primary} />
          </View>
        ) : providers.length === 0 ? (
          <View style={styles.emptyBox}>
            <Ionicons name="people-outline" size={48} color={THEME.textTertiary} />
            <Text style={styles.emptyText}>暂无服务提供者</Text>
          </View>
        ) : (
          providers.map((provider) => (
            <ProviderCard
              key={provider.account}
              provider={provider}
              packages={provider.packages}
            />
          ))
        )}

        {/* 入驻入口 */}
        <Pressable
          style={styles.joinCard}
          onPress={() => router.push('/market/provider/register')}
        >
          <View style={styles.joinIcon}>
            <Ionicons name="briefcase-outline" size={24} color={THEME.primary} />
          </View>
          <View style={styles.joinInfo}>
            <Text style={styles.joinTitle}>成为服务提供者</Text>
            <Text style={styles.joinDesc}>开启您的占卜服务副业</Text>
          </View>
          <Ionicons name="chevron-forward" size={20} color={THEME.textTertiary} />
        </Pressable>
      </ScrollView>

      <BottomNavBar activeTab="market" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 56,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: THEME.card,
    borderBottomWidth: 1,
    borderBottomColor: THEME.borderLight,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: THEME.text,
  },
  searchButton: {
    padding: 8,
  },
  filterScroll: {
    backgroundColor: THEME.card,
    borderBottomWidth: 1,
    borderBottomColor: THEME.borderLight,
  },
  filterContent: {
    paddingHorizontal: 12,
    paddingVertical: 12,
    gap: 8,
    flexDirection: 'row',
  },
  filterTag: {
    paddingHorizontal: 14,
    paddingVertical: 6,
    borderRadius: 16,
    backgroundColor: THEME.background,
  },
  filterTagActive: {
    backgroundColor: THEME.primary,
  },
  filterText: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  filterTextActive: {
    color: THEME.textInverse,
    fontWeight: '500',
  },
  sortBar: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    paddingVertical: 10,
    backgroundColor: THEME.card,
    borderBottomWidth: 1,
    borderBottomColor: THEME.borderLight,
    gap: 24,
  },
  sortItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  sortText: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 12,
    paddingBottom: 100,
  },
  loadingBox: {
    paddingVertical: 60,
    alignItems: 'center',
  },
  emptyBox: {
    paddingVertical: 60,
    alignItems: 'center',
    gap: 12,
  },
  emptyText: {
    fontSize: 14,
    color: THEME.textTertiary,
  },
  joinCard: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginTop: 8,
    ...SHADOWS.small,
  },
  joinIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: THEME.primary + '15',
    justifyContent: 'center',
    alignItems: 'center',
  },
  joinInfo: {
    flex: 1,
    marginLeft: 12,
  },
  joinTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
  },
  joinDesc: {
    fontSize: 13,
    color: THEME.textTertiary,
    marginTop: 2,
  },
});
```

---

## 2. 提供者详情页

### 页面线框图

```
┌─────────────────────────────────────┐
│  ←  提供者详情                      │
├─────────────────────────────────────┤
│           ┌─────────┐               │
│           │   👤    │               │
│           └─────────┘               │
│          张三大师                   │
│    ⭐ 4.8 · 🏅 专家 · 📦 156单      │
│                                     │
│    从业20年，专注事业财运分析       │
│    擅长八字命理、梅花易数...        │
├─────────────────────────────────────┤
│  擅长领域                           │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐       │
│  │事业│ │财运│ │感情│ │健康│       │
│  └────┘ └────┘ └────┘ └────┘       │
├─────────────────────────────────────┤
│  服务套餐                           │
│  ┌─────────────────────────────────┐│
│  │ 📝 详细文字解卦                 ││
│  │ 梅花易数 · 文字解读             ││
│  │ ✓ 3次追问  ✓ 支持加急(+50%)    ││
│  │                                 ││
│  │ 💰 10 DUST        [立即预约]    ││
│  └─────────────────────────────────┘│
│  ┌─────────────────────────────────┐│
│  │ 🎤 语音深度解读                 ││
│  │ 八字命理 · 语音解读 · 15分钟    ││
│  │ ✓ 5次追问                       ││
│  │                                 ││
│  │ 💰 30 DUST        [立即预约]    ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  用户评价 (156)              查看全部│
│  ┌─────────────────────────────────┐│
│  │ 匿名用户 · ⭐⭐⭐⭐⭐ · 1天前   ││
│  │ 准确度:5 态度:5 响应:5          ││
│  │ 解读很准确，态度也很好，推荐！  ││
│  │ ↳ 提供者回复：感谢您的认可...   ││
│  └─────────────────────────────────┘│
│  ┌─────────────────────────────────┐│
│  │ 用户A · ⭐⭐⭐⭐☆ · 3天前       ││
│  │ ...                             ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

### 套餐卡片组件

```tsx
// frontend/src/divination/market/components/PackageCard.tsx

import React from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '../theme';
import { ServicePackage } from '../types/market.types';
import { formatBalance, getServiceTypeName, getDivinationTypeName } from '../utils/market.utils';

// 服务类型图标映射
const SERVICE_TYPE_ICONS: Record<number, string> = {
  0: 'document-text-outline',  // TextReading
  1: 'mic-outline',            // VoiceReading
  2: 'videocam-outline',       // VideoReading
  3: 'chatbubbles-outline',    // LiveConsultation
};

interface PackageCardProps {
  package: ServicePackage;
  onOrder: () => void;
}

export const PackageCard: React.FC<PackageCardProps> = ({ package: pkg, onOrder }) => {
  return (
    <View style={styles.card}>
      {/* 套餐头部 */}
      <View style={styles.header}>
        <View style={styles.iconBox}>
          <Ionicons
            name={SERVICE_TYPE_ICONS[pkg.serviceType] || 'help-outline'}
            size={24}
            color={THEME.primary}
          />
        </View>
        <View style={styles.headerInfo}>
          <Text style={styles.name}>{pkg.name}</Text>
          <View style={styles.metaRow}>
            <Text style={styles.metaText}>
              {getDivinationTypeName(pkg.divinationType)}
            </Text>
            <Text style={styles.metaDot}>·</Text>
            <Text style={styles.metaText}>
              {getServiceTypeName(pkg.serviceType)}
            </Text>
            {pkg.duration > 0 && (
              <>
                <Text style={styles.metaDot}>·</Text>
                <Text style={styles.metaText}>{pkg.duration}分钟</Text>
              </>
            )}
          </View>
        </View>
      </View>

      {/* 套餐描述 */}
      <Text style={styles.description} numberOfLines={2}>
        {pkg.description}
      </Text>

      {/* 套餐特性标签 */}
      <View style={styles.features}>
        {pkg.followUpCount > 0 && (
          <View style={styles.featureTag}>
            <Ionicons name="chatbubble-outline" size={12} color={THEME.textSecondary} />
            <Text style={styles.featureText}>{pkg.followUpCount}次追问</Text>
          </View>
        )}
        {pkg.urgentAvailable && (
          <View style={[styles.featureTag, styles.urgentTag]}>
            <Ionicons name="flash-outline" size={12} color={THEME.warning} />
            <Text style={[styles.featureText, { color: THEME.warning }]}>
              加急+{pkg.urgentSurcharge / 100}%
            </Text>
          </View>
        )}
      </View>

      {/* 底部：价格和按钮 */}
      <View style={styles.footer}>
        <View style={styles.priceBox}>
          <Text style={styles.priceValue}>{formatBalance(pkg.price)}</Text>
          <Text style={styles.priceUnit}>DUST</Text>
        </View>
        <Pressable style={styles.orderButton} onPress={onOrder}>
          <Text style={styles.orderButtonText}>立即预约</Text>
        </Pressable>
      </View>

      {/* 销量标签 */}
      {pkg.salesCount > 0 && (
        <View style={styles.salesBadge}>
          <Text style={styles.salesText}>已售 {pkg.salesCount}</Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    position: 'relative',
    ...SHADOWS.medium,
  },
  header: {
    flexDirection: 'row',
    marginBottom: 12,
  },
  iconBox: {
    width: 44,
    height: 44,
    borderRadius: 10,
    backgroundColor: THEME.primary + '15',
    justifyContent: 'center',
    alignItems: 'center',
  },
  headerInfo: {
    flex: 1,
    marginLeft: 12,
    justifyContent: 'center',
  },
  name: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 4,
  },
  metaRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  metaText: {
    fontSize: 13,
    color: THEME.textTertiary,
  },
  metaDot: {
    fontSize: 13,
    color: THEME.textTertiary,
    marginHorizontal: 4,
  },
  description: {
    fontSize: 14,
    color: THEME.textSecondary,
    lineHeight: 20,
    marginBottom: 12,
  },
  features: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    marginBottom: 16,
  },
  featureTag: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    paddingHorizontal: 8,
    paddingVertical: 4,
    backgroundColor: THEME.background,
    borderRadius: 4,
  },
  urgentTag: {
    backgroundColor: THEME.warning + '15',
  },
  featureText: {
    fontSize: 12,
    color: THEME.textSecondary,
  },
  footer: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  priceBox: {
    flexDirection: 'row',
    alignItems: 'baseline',
    gap: 4,
  },
  priceValue: {
    fontSize: 22,
    fontWeight: '700',
    color: THEME.primary,
  },
  priceUnit: {
    fontSize: 14,
    color: THEME.primary,
  },
  orderButton: {
    paddingHorizontal: 20,
    paddingVertical: 10,
    backgroundColor: THEME.primary,
    borderRadius: 6,
  },
  orderButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.textInverse,
  },
  salesBadge: {
    position: 'absolute',
    top: 12,
    right: 12,
  },
  salesText: {
    fontSize: 11,
    color: THEME.textTertiary,
  },
});
```

### 评价卡片组件

```tsx
// frontend/src/divination/market/components/ReviewCard.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../theme';
import { Review } from '../types/market.types';
import { formatTimeAgo } from '../utils/market.utils';

interface ReviewCardProps {
  review: Review;
}

export const ReviewCard: React.FC<ReviewCardProps> = ({ review }) => {
  const renderStars = (rating: number) => {
    return (
      <View style={styles.starsRow}>
        {[1, 2, 3, 4, 5].map((star) => (
          <Ionicons
            key={star}
            name={star <= rating ? 'star' : 'star-outline'}
            size={12}
            color={star <= rating ? THEME.warning : '#DDD'}
          />
        ))}
      </View>
    );
  };

  return (
    <View style={styles.card}>
      {/* 评价头部 */}
      <View style={styles.header}>
        <Text style={styles.userName}>
          {review.isAnonymous ? '匿名用户' : review.customerName || '用户'}
        </Text>
        {renderStars(review.overallRating)}
        <Text style={styles.time}>{formatTimeAgo(review.createdAt)}</Text>
      </View>

      {/* 多维度评分 */}
      <View style={styles.ratings}>
        <Text style={styles.ratingItem}>准确度:{review.accuracyRating}</Text>
        <Text style={styles.ratingItem}>态度:{review.attitudeRating}</Text>
        <Text style={styles.ratingItem}>响应:{review.responseRating}</Text>
      </View>

      {/* 评价内容 */}
      {review.content && (
        <Text style={styles.content}>{review.content}</Text>
      )}

      {/* 提供者回复 */}
      {review.reply && (
        <View style={styles.replyBox}>
          <Text style={styles.replyLabel}>↳ 提供者回复：</Text>
          <Text style={styles.replyContent}>{review.reply}</Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: THEME.card,
    borderRadius: 8,
    padding: 14,
    marginBottom: 10,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 8,
  },
  userName: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME.text,
  },
  starsRow: {
    flexDirection: 'row',
    gap: 2,
  },
  time: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginLeft: 'auto',
  },
  ratings: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 8,
  },
  ratingItem: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  content: {
    fontSize: 14,
    color: THEME.textSecondary,
    lineHeight: 20,
  },
  replyBox: {
    marginTop: 10,
    paddingTop: 10,
    borderTopWidth: 1,
    borderTopColor: THEME.borderLight,
  },
  replyLabel: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginBottom: 4,
  },
  replyContent: {
    fontSize: 13,
    color: THEME.textSecondary,
    lineHeight: 18,
  },
});
```

---

## 3. 创建订单页

### 页面线框图

```
┌─────────────────────────────────────┐
│  ←  确认订单                        │
├─────────────────────────────────────┤
│  服务提供者                         │
│  ┌─────────────────────────────────┐│
│  │ 👤 张三大师 · 🏅专家            ││
│  │ ⭐ 4.8 · 156单完成              ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  服务套餐                           │
│  ┌─────────────────────────────────┐│
│  │ 📝 详细文字解卦                 ││
│  │ 梅花易数 · 文字解读             ││
│  │ 3次追问 · 支持加急              ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  关联卦象 *                         │
│  ┌─────────────────────────────────┐│
│  │ ☰ 天泽履                        ││
│  │ 2024-01-15 14:30 起卦           ││
│  │ 问事：事业发展                  ││
│  │                    [更换卦象]   ││
│  └─────────────────────────────────┘│
│  或 [选择已有卦象]  [现在起卦]      │
├─────────────────────────────────────┤
│  问题描述 *                         │
│  ┌─────────────────────────────────┐│
│  │ 请详细描述您想咨询的问题，      ││
│  │ 包括背景信息、具体疑问等...     ││
│  │                                 ││
│  │                                 ││
│  │                         0/500   ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  加急服务                           │
│  ┌─────────────────────────────────┐│
│  │ ⚡ 加急处理                [○]  ││
│  │ 预计2小时内响应，加价50%        ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  费用明细                           │
│  ┌─────────────────────────────────┐│
│  │ 套餐价格              10 DUST   ││
│  │ 加急费用               5 DUST   ││
│  │ ─────────────────────────────   ││
│  │ 合计支付              15 DUST   ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  ┌─────────────────────────────────┐│
│  │      确认支付 15 DUST           ││
│  └─────────────────────────────────┘│
│  支付后金额将托管至平台，完成后结算  │
└─────────────────────────────────────┘
```

### 创建订单页面代码

```tsx
// frontend/app/market/order/create.tsx

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  TextInput,
  Switch,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { TierBadge } from '@/divination/market/components/TierBadge';
import { useMarketApi } from '@/divination/market/hooks/useMarketApi';
import { formatBalance } from '@/divination/market/utils/market.utils';

export default function CreateOrderPage() {
  const router = useRouter();
  const params = useLocalSearchParams();
  const { providerId, packageId } = params;
  
  const { getProvider, getPackage, createOrder, loading } = useMarketApi();
  
  const [provider, setProvider] = useState(null);
  const [pkg, setPkg] = useState(null);
  const [selectedHexagram, setSelectedHexagram] = useState(null);
  const [question, setQuestion] = useState('');
  const [isUrgent, setIsUrgent] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    loadData();
  }, [providerId, packageId]);

  const loadData = async () => {
    const [providerData, packageData] = await Promise.all([
      getProvider(providerId as string),
      getPackage(providerId as string, Number(packageId)),
    ]);
    setProvider(providerData);
    setPkg(packageData);
  };

  // 计算费用
  const calculateTotal = () => {
    if (!pkg) return 0n;
    let total = pkg.price;
    if (isUrgent && pkg.urgentAvailable) {
      total = total + (pkg.price * BigInt(pkg.urgentSurcharge)) / 10000n;
    }
    return total;
  };

  // 选择卦象
  const handleSelectHexagram = () => {
    router.push({
      pathname: '/market/order/select-hexagram',
      params: { divinationType: pkg?.divinationType },
    });
  };

  // 提交订单
  const handleSubmit = async () => {
    if (!selectedHexagram) {
      Alert.alert('提示', '请选择关联的卦象');
      return;
    }
    if (!question.trim()) {
      Alert.alert('提示', '请描述您的问题');
      return;
    }
    if (question.length < 10) {
      Alert.alert('提示', '问题描述至少10个字');
      return;
    }

    Alert.alert(
      '确认支付',
      `确认支付 ${formatBalance(calculateTotal())} DUST？`,
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确认',
          onPress: async () => {
            setSubmitting(true);
            try {
              const orderId = await createOrder({
                provider: providerId as string,
                divinationType: pkg.divinationType,
                hexagramId: selectedHexagram.id,
                packageId: Number(packageId),
                question,
                isUrgent,
              });
              
              Alert.alert('成功', '订单创建成功', [
                {
                  text: '查看订单',
                  onPress: () => router.replace(`/market/order/${orderId}`),
                },
              ]);
            } catch (error: any) {
              Alert.alert('错误', error.message || '创建订单失败');
            } finally {
              setSubmitting(false);
            }
          },
        },
      ]
    );
  };

  if (loading || !provider || !pkg) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color={THEME.primary} />
      </View>
    );
  }

  const total = calculateTotal();

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color={THEME.text} />
        </Pressable>
        <Text style={styles.headerTitle}>确认订单</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
      >
        {/* 服务提供者信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>服务提供者</Text>
          <View style={styles.providerCard}>
            <View style={styles.providerAvatar}>
              <Text style={styles.avatarText}>{provider.name.charAt(0)}</Text>
            </View>
            <View style={styles.providerInfo}>
              <View style={styles.providerNameRow}>
                <Text style={styles.providerName}>{provider.name}</Text>
                <TierBadge tier={provider.tier} />
              </View>
              <Text style={styles.providerStats}>
                ⭐ {(provider.totalRating / provider.ratingCount / 100).toFixed(1)} · {provider.completedOrders}单完成
              </Text>
            </View>
          </View>
        </View>

        {/* 服务套餐信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>服务套餐</Text>
          <View style={styles.packageCard}>
            <Text style={styles.packageName}>{pkg.name}</Text>
            <Text style={styles.packageMeta}>
              {pkg.divinationTypeName} · {pkg.serviceTypeName}
              {pkg.duration > 0 && ` · ${pkg.duration}分钟`}
            </Text>
            <Text style={styles.packageFeatures}>
              {pkg.followUpCount > 0 && `${pkg.followUpCount}次追问`}
              {pkg.urgentAvailable && ' · 支持加急'}
            </Text>
          </View>
        </View>

        {/* 关联卦象 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>
            关联卦象 <Text style={styles.required}>*</Text>
          </Text>
          {selectedHexagram ? (
            <Pressable style={styles.hexagramCard} onPress={handleSelectHexagram}>
              <View style={styles.hexagramInfo}>
                <Text style={styles.hexagramSymbol}>{selectedHexagram.symbol}</Text>
                <View>
                  <Text style={styles.hexagramName}>{selectedHexagram.name}</Text>
                  <Text style={styles.hexagramTime}>{selectedHexagram.createdAt}</Text>
                </View>
              </View>
              <Text style={styles.changeText}>更换</Text>
            </Pressable>
          ) : (
            <View style={styles.selectButtons}>
              <Pressable style={styles.selectButton} onPress={handleSelectHexagram}>
                <Ionicons name="albums-outline" size={20} color={THEME.primary} />
                <Text style={styles.selectButtonText}>选择已有卦象</Text>
              </Pressable>
              <Pressable
                style={styles.selectButton}
                onPress={() => router.push(`/divination/${pkg.divinationTypeRoute}`)}
              >
                <Ionicons name="add-circle-outline" size={20} color={THEME.primary} />
                <Text style={styles.selectButtonText}>现在起卦</Text>
              </Pressable>
            </View>
          )}
        </View>

        {/* 问题描述 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>
            问题描述 <Text style={styles.required}>*</Text>
          </Text>
          <View style={styles.inputWrapper}>
            <TextInput
              style={styles.questionInput}
              value={question}
              onChangeText={setQuestion}
              placeholder="请详细描述您想咨询的问题，包括背景信息、具体疑问等..."
              placeholderTextColor={THEME.textTertiary}
              multiline
              numberOfLines={5}
              maxLength={500}
              textAlignVertical="top"
            />
            <Text style={styles.charCount}>{question.length}/500</Text>
          </View>
        </View>

        {/* 加急服务 */}
        {pkg.urgentAvailable && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>加急服务</Text>
            <View style={styles.urgentCard}>
              <View style={styles.urgentInfo}>
                <View style={styles.urgentHeader}>
                  <Ionicons name="flash" size={18} color={THEME.warning} />
                  <Text style={styles.urgentTitle}>加急处理</Text>
                </View>
                <Text style={styles.urgentDesc}>
                  预计2小时内响应，加价{pkg.urgentSurcharge / 100}%
                </Text>
              </View>
              <Switch
                value={isUrgent}
                onValueChange={setIsUrgent}
                trackColor={{ false: '#E8E8E8', true: THEME.warning + '60' }}
                thumbColor={isUrgent ? THEME.warning : '#FFF'}
              />
            </View>
          </View>
        )}

        {/* 费用明细 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>费用明细</Text>
          <View style={styles.feeCard}>
            <View style={styles.feeRow}>
              <Text style={styles.feeLabel}>套餐价格</Text>
              <Text style={styles.feeValue}>{formatBalance(pkg.price)} DUST</Text>
            </View>
            {isUrgent && (
              <View style={styles.feeRow}>
                <Text style={styles.feeLabel}>加急费用</Text>
                <Text style={styles.feeValue}>
                  +{formatBalance((pkg.price * BigInt(pkg.urgentSurcharge)) / 10000n)} DUST
                </Text>
              </View>
            )}
            <View style={styles.feeDivider} />
            <View style={styles.feeRow}>
              <Text style={styles.totalLabel}>合计支付</Text>
              <Text style={styles.totalValue}>{formatBalance(total)} DUST</Text>
            </View>
          </View>
        </View>
      </ScrollView>

      {/* 底部支付按钮 */}
      <View style={styles.footer}>
        <Pressable
          style={[styles.submitButton, submitting && styles.submitButtonDisabled]}
          onPress={handleSubmit}
          disabled={submitting}
        >
          {submitting ? (
            <ActivityIndicator color={THEME.textInverse} />
          ) : (
            <Text style={styles.submitButtonText}>
              确认支付 {formatBalance(total)} DUST
            </Text>
          )}
        </Pressable>
        <Text style={styles.footerTip}>
          支付后金额将托管至平台，服务完成后结算给提供者
        </Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: THEME.background,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: THEME.card,
    borderBottomWidth: 1,
    borderBottomColor: THEME.borderLight,
  },
  backButton: {
    padding: 4,
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME.text,
  },
  placeholder: {
    width: 32,
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 16,
    paddingBottom: 24,
  },
  section: {
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 10,
  },
  required: {
    color: THEME.error,
  },
  providerCard: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    ...SHADOWS.small,
  },
  providerAvatar: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: THEME.primary + '20',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.primary,
  },
  providerInfo: {
    flex: 1,
    marginLeft: 12,
  },
  providerNameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  providerName: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
  },
  providerStats: {
    fontSize: 13,
    color: THEME.textTertiary,
    marginTop: 2,
  },
  packageCard: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    ...SHADOWS.small,
  },
  packageName: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 4,
  },
  packageMeta: {
    fontSize: 13,
    color: THEME.textSecondary,
    marginBottom: 2,
  },
  packageFeatures: {
    fontSize: 13,
    color: THEME.textTertiary,
  },
  hexagramCard: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    ...SHADOWS.small,
  },
  hexagramInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  hexagramSymbol: {
    fontSize: 28,
    color: THEME.text,
  },
  hexagramName: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
  },
  hexagramTime: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 2,
  },
  changeText: {
    fontSize: 14,
    color: THEME.primary,
  },
  selectButtons: {
    flexDirection: 'row',
    gap: 12,
  },
  selectButton: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 6,
    backgroundColor: THEME.card,
    borderRadius: 10,
    paddingVertical: 14,
    borderWidth: 1,
    borderColor: THEME.primary + '40',
    borderStyle: 'dashed',
  },
  selectButtonText: {
    fontSize: 14,
    color: THEME.primary,
  },
  inputWrapper: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    ...SHADOWS.small,
  },
  questionInput: {
    padding: 14,
    fontSize: 14,
    color: THEME.text,
    minHeight: 120,
    lineHeight: 20,
  },
  charCount: {
    fontSize: 12,
    color: THEME.textTertiary,
    textAlign: 'right',
    paddingRight: 14,
    paddingBottom: 10,
  },
  urgentCard: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    ...SHADOWS.small,
  },
  urgentInfo: {
    flex: 1,
  },
  urgentHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  urgentTitle: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
  },
  urgentDesc: {
    fontSize: 13,
    color: THEME.textTertiary,
    marginTop: 4,
  },
  feeCard: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    ...SHADOWS.small,
  },
  feeRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 6,
  },
  feeLabel: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  feeValue: {
    fontSize: 14,
    color: THEME.text,
  },
  feeDivider: {
    height: 1,
    backgroundColor: THEME.borderLight,
    marginVertical: 8,
  },
  totalLabel: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
  },
  totalValue: {
    fontSize: 18,
    fontWeight: '700',
    color: THEME.primary,
  },
  footer: {
    padding: 16,
    paddingBottom: 32,
    backgroundColor: THEME.card,
    borderTopWidth: 1,
    borderTopColor: THEME.borderLight,
  },
  submitButton: {
    height: 50,
    backgroundColor: THEME.primary,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  submitButtonDisabled: {
    opacity: 0.6,
  },
  submitButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.textInverse,
  },
  footerTip: {
    fontSize: 12,
    color: THEME.textTertiary,
    textAlign: 'center',
    marginTop: 10,
  },
});
```

---

## 4. 订单详情页

### 页面线框图

```
┌─────────────────────────────────────┐
│  ←  订单详情              #12345    │
├─────────────────────────────────────┤
│                                     │
│  ○─────●─────○─────○─────○          │
│  待支付  已支付  已接单  已完成  已评价│  ← 状态时间线
│                                     │
├─────────────────────────────────────┤
│  订单信息                           │
│  ┌─────────────────────────────────┐│
│  │ 订单编号        #12345          ││
│  │ 创建时间        2024-01-15 14:30││
│  │ 订单状态        已支付          ││
│  │ 支付金额        15 DUST         ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  服务信息                           │
│  ┌─────────────────────────────────┐│
│  │ 提供者          张三大师        ││
│  │ 服务套餐        详细文字解卦    ││
│  │ 占卜类型        梅花易数        ││
│  │ 是否加急        是              ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  关联卦象                           │
│  ┌─────────────────────────────────┐│
│  │ ☰ 天泽履                        ││
│  │ 上卦：乾(天) 下卦：兑(泽)       ││
│  │ [查看卦象详情]                  ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  我的问题                           │
│  ┌─────────────────────────────────┐│
│  │ 想问一下最近的事业发展方向，    ││
│  │ 目前在考虑是否跳槽...           ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  大师解读                           │  ← 完成后显示
│  ┌─────────────────────────────────┐│
│  │ 根据卦象分析，您近期事业运势    ││
│  │ 整体向好，但需注意...           ││
│  │                                 ││
│  │ [查看完整解读]                  ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  追问记录 (1/3)                     │
│  ┌─────────────────────────────────┐│
│  │ 👤 我：那财运方面呢？           ││
│  │ 🎓 大师：财运方面建议...        ││
│  └─────────────────────────────────┘│
│  ┌─────────────────────────────────┐│
│  │ 输入追问内容...        [发送]   ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  [        提交评价        ]         │  ← 完成后显示
│  或                                 │
│  [        取消订单        ]         │  ← 待接单时显示
└─────────────────────────────────────┘
```

### 订单时间线组件

```tsx
// frontend/src/divination/market/components/OrderTimeline.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../theme';
import { OrderStatus } from '../types/market.types';

interface TimelineStep {
  status: OrderStatus;
  label: string;
  icon: string;
}

const TIMELINE_STEPS: TimelineStep[] = [
  { status: 'PendingPayment', label: '待支付', icon: 'card-outline' },
  { status: 'Paid', label: '已支付', icon: 'checkmark-circle-outline' },
  { status: 'Accepted', label: '已接单', icon: 'hand-left-outline' },
  { status: 'Completed', label: '已完成', icon: 'document-text-outline' },
  { status: 'Reviewed', label: '已评价', icon: 'star-outline' },
];

interface OrderTimelineProps {
  currentStatus: OrderStatus;
  isCancelled?: boolean;
}

export const OrderTimeline: React.FC<OrderTimelineProps> = ({
  currentStatus,
  isCancelled = false,
}) => {
  const currentIndex = TIMELINE_STEPS.findIndex((s) => s.status === currentStatus);

  if (isCancelled) {
    return (
      <View style={styles.cancelledContainer}>
        <View style={styles.cancelledIcon}>
          <Ionicons name="close-circle" size={32} color={THEME.error} />
        </View>
        <Text style={styles.cancelledText}>订单已取消</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.timeline}>
        {TIMELINE_STEPS.map((step, index) => {
          const isCompleted = index < currentIndex;
          const isCurrent = index === currentIndex;
          const isPending = index > currentIndex;

          return (
            <React.Fragment key={step.status}>
              {/* 节点 */}
              <View style={styles.stepContainer}>
                <View
                  style={[
                    styles.dot,
                    isCompleted && styles.dotCompleted,
                    isCurrent && styles.dotCurrent,
                    isPending && styles.dotPending,
                  ]}
                >
                  <Ionicons
                    name={step.icon as any}
                    size={14}
                    color={isPending ? THEME.textTertiary : THEME.textInverse}
                  />
                </View>
                <Text
                  style={[
                    styles.label,
                    isCompleted && styles.labelCompleted,
                    isCurrent && styles.labelCurrent,
                    isPending && styles.labelPending,
                  ]}
                >
                  {step.label}
                </Text>
              </View>

              {/* 连接线 */}
              {index < TIMELINE_STEPS.length - 1 && (
                <View
                  style={[
                    styles.line,
                    index < currentIndex && styles.lineCompleted,
                  ]}
                />
              )}
            </React.Fragment>
          );
        })}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    paddingVertical: 20,
    paddingHorizontal: 16,
    backgroundColor: THEME.card,
  },
  timeline: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    justifyContent: 'space-between',
  },
  stepContainer: {
    alignItems: 'center',
    width: 50,
  },
  dot: {
    width: 28,
    height: 28,
    borderRadius: 14,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 6,
  },
  dotCompleted: {
    backgroundColor: THEME.success,
  },
  dotCurrent: {
    backgroundColor: THEME.primary,
  },
  dotPending: {
    backgroundColor: THEME.border,
  },
  label: {
    fontSize: 11,
    textAlign: 'center',
  },
  labelCompleted: {
    color: THEME.success,
  },
  labelCurrent: {
    color: THEME.primary,
    fontWeight: '600',
  },
  labelPending: {
    color: THEME.textTertiary,
  },
  line: {
    flex: 1,
    height: 2,
    backgroundColor: THEME.border,
    marginTop: 13,
    marginHorizontal: -8,
  },
  lineCompleted: {
    backgroundColor: THEME.success,
  },
  cancelledContainer: {
    alignItems: 'center',
    paddingVertical: 24,
    backgroundColor: THEME.card,
  },
  cancelledIcon: {
    marginBottom: 8,
  },
  cancelledText: {
    fontSize: 15,
    color: THEME.error,
    fontWeight: '500',
  },
});
```

### 追问组件

```tsx
// frontend/src/divination/market/components/FollowUpSection.tsx

import React, { useState } from 'react';
import { View, Text, StyleSheet, TextInput, Pressable, Alert } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '../theme';
import { FollowUp } from '../types/market.types';

interface FollowUpSectionProps {
  followUps: FollowUp[];
  maxFollowUps: number;
  canSubmit: boolean;
  onSubmit: (content: string) => Promise<void>;
}

export const FollowUpSection: React.FC<FollowUpSectionProps> = ({
  followUps,
  maxFollowUps,
  canSubmit,
  onSubmit,
}) => {
  const [content, setContent] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const remainingCount = maxFollowUps - followUps.length;

  const handleSubmit = async () => {
    if (!content.trim()) {
      Alert.alert('提示', '请输入追问内容');
      return;
    }

    setSubmitting(true);
    try {
      await onSubmit(content);
      setContent('');
    } catch (error: any) {
      Alert.alert('错误', error.message);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>追问记录</Text>
        <Text style={styles.count}>
          ({followUps.length}/{maxFollowUps})
        </Text>
      </View>

      {/* 追问列表 */}
      {followUps.map((followUp, index) => (
        <View key={index} style={styles.followUpItem}>
          {/* 用户追问 */}
          <View style={styles.messageRow}>
            <View style={[styles.avatar, styles.userAvatar]}>
              <Ionicons name="person" size={14} color={THEME.primary} />
            </View>
            <View style={[styles.messageBubble, styles.userBubble]}>
              <Text style={styles.messageText}>{followUp.question}</Text>
            </View>
          </View>

          {/* 提供者回复 */}
          {followUp.answer ? (
            <View style={[styles.messageRow, styles.answerRow]}>
              <View style={[styles.avatar, styles.providerAvatar]}>
                <Ionicons name="school" size={14} color={THEME.success} />
              </View>
              <View style={[styles.messageBubble, styles.providerBubble]}>
                <Text style={styles.messageText}>{followUp.answer}</Text>
              </View>
            </View>
          ) : (
            <View style={styles.waitingRow}>
              <Text style={styles.waitingText}>等待回复中...</Text>
            </View>
          )}
        </View>
      ))}

      {/* 追问输入框 */}
      {canSubmit && remainingCount > 0 && (
        <View style={styles.inputContainer}>
          <TextInput
            style={styles.input}
            value={content}
            onChangeText={setContent}
            placeholder={`还可追问${remainingCount}次...`}
            placeholderTextColor={THEME.textTertiary}
            multiline
            maxLength={200}
          />
          <Pressable
            style={[styles.sendButton, !content.trim() && styles.sendButtonDisabled]}
            onPress={handleSubmit}
            disabled={!content.trim() || submitting}
          >
            <Ionicons
              name="send"
              size={18}
              color={content.trim() ? THEME.textInverse : THEME.textTertiary}
            />
          </Pressable>
        </View>
      )}

      {remainingCount === 0 && (
        <Text style={styles.noMoreText}>追问次数已用完</Text>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    ...SHADOWS.small,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  title: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
  },
  count: {
    fontSize: 13,
    color: THEME.textTertiary,
    marginLeft: 4,
  },
  followUpItem: {
    marginBottom: 16,
  },
  messageRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginBottom: 8,
  },
  answerRow: {
    marginLeft: 20,
  },
  avatar: {
    width: 28,
    height: 28,
    borderRadius: 14,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 8,
  },
  userAvatar: {
    backgroundColor: THEME.primary + '20',
  },
  providerAvatar: {
    backgroundColor: THEME.success + '20',
  },
  messageBubble: {
    flex: 1,
    padding: 10,
    borderRadius: 8,
  },
  userBubble: {
    backgroundColor: THEME.primary + '10',
  },
  providerBubble: {
    backgroundColor: THEME.background,
  },
  messageText: {
    fontSize: 14,
    color: THEME.text,
    lineHeight: 20,
  },
  waitingRow: {
    marginLeft: 36,
  },
  waitingText: {
    fontSize: 13,
    color: THEME.textTertiary,
    fontStyle: 'italic',
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    borderTopWidth: 1,
    borderTopColor: THEME.borderLight,
    paddingTop: 12,
    marginTop: 4,
  },
  input: {
    flex: 1,
    minHeight: 36,
    maxHeight: 80,
    backgroundColor: THEME.background,
    borderRadius: 18,
    paddingHorizontal: 14,
    paddingVertical: 8,
    fontSize: 14,
    color: THEME.text,
  },
  sendButton: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: THEME.primary,
    justifyContent: 'center',
    alignItems: 'center',
    marginLeft: 8,
  },
  sendButtonDisabled: {
    backgroundColor: THEME.border,
  },
  noMoreText: {
    fontSize: 13,
    color: THEME.textTertiary,
    textAlign: 'center',
    marginTop: 8,
  },
});
```

---

## 5. 提供者工作台

### 页面线框图

```
┌─────────────────────────────────────┐
│  我的工作台                    ⚙️   │
├─────────────────────────────────────┤
│  ┌─────────────────────────────────┐│
│  │ 💰 可提现余额                   ││
│  │                                 ││
│  │    156.80 DUST                  ││
│  │                                 ││
│  │    [提现]      [收益明细]       ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  今日数据                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐
│  │  新订单  │ │  已完成  │ │  收入    │
│  │    3     │ │    5     │ │ 45 DUST  │
│  └──────────┘ └──────────┘ └──────────┘
├─────────────────────────────────────┤
│  待处理订单 (3)              查看全部│
│  ┌─────────────────────────────────┐│
│  │ 🔴 #12346                       ││
│  │ 梅花易数 · 文字解卦             ││
│  │ 2小时前 · ⚡加急                ││
│  │                                 ││
│  │ [拒绝]              [接单]      ││
│  └─────────────────────────────────┘│
│  ┌─────────────────────────────────┐│
│  │ 🟡 #12345                       ││
│  │ 八字命理 · 语音解读             ││
│  │ 5小时前                         ││
│  │                                 ││
│  │ [拒绝]              [接单]      ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  进行中订单 (2)              查看全部│
│  ┌─────────────────────────────────┐│
│  │ 🟢 #12340                       ││
│  │ 梅花易数 · 文字解卦             ││
│  │ 等待提交解读                    ││
│  │                                 ││
│  │ [查看详情]        [提交解读]    ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  ┌─────────────────────────────────┐│
│  │  接单状态：接单中 ✓             ││
│  │  [暂停接单]                     ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

### 工作台页面代码

```tsx
// frontend/app/market/provider/dashboard.tsx

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  RefreshControl,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { useMarketApi } from '@/divination/market/hooks/useMarketApi';
import { formatBalance, formatTimeAgo } from '@/divination/market/utils/market.utils';
import { BottomNavBar } from '@/components/BottomNavBar';

export default function ProviderDashboardPage() {
  const router = useRouter();
  const {
    getProviderDashboard,
    acceptOrder,
    rejectOrder,
    pauseProvider,
    resumeProvider,
    requestWithdrawal,
  } = useMarketApi();

  const [dashboard, setDashboard] = useState(null);
  const [refreshing, setRefreshing] = useState(false);

  useEffect(() => {
    loadDashboard();
  }, []);

  const loadDashboard = async () => {
    const data = await getProviderDashboard();
    setDashboard(data);
  };

  const onRefresh = async () => {
    setRefreshing(true);
    await loadDashboard();
    setRefreshing(false);
  };

  // 接单
  const handleAccept = async (orderId: number) => {
    Alert.alert('确认接单', '确认接受此订单？', [
      { text: '取消', style: 'cancel' },
      {
        text: '确认',
        onPress: async () => {
          try {
            await acceptOrder(orderId);
            Alert.alert('成功', '已接单');
            loadDashboard();
          } catch (error: any) {
            Alert.alert('错误', error.message);
          }
        },
      },
    ]);
  };

  // 拒绝订单
  const handleReject = async (orderId: number) => {
    Alert.alert('确认拒绝', '拒绝后订单将自动退款给用户', [
      { text: '取消', style: 'cancel' },
      {
        text: '确认拒绝',
        style: 'destructive',
        onPress: async () => {
          try {
            await rejectOrder(orderId);
            Alert.alert('成功', '已拒绝订单');
            loadDashboard();
          } catch (error: any) {
            Alert.alert('错误', error.message);
          }
        },
      },
    ]);
  };

  // 切换接单状态
  const handleToggleStatus = async () => {
    try {
      if (dashboard.provider.status === 'Active') {
        await pauseProvider();
        Alert.alert('成功', '已暂停接单');
      } else {
        await resumeProvider();
        Alert.alert('成功', '已恢复接单');
      }
      loadDashboard();
    } catch (error: any) {
      Alert.alert('错误', error.message);
    }
  };

  // 提现
  const handleWithdraw = () => {
    Alert.prompt(
      '申请提现',
      `可提现余额：${formatBalance(dashboard.balance)} DUST`,
      [
        { text: '取消', style: 'cancel' },
        {
          text: '全部提现',
          onPress: async () => {
            try {
              await requestWithdrawal(dashboard.balance);
              Alert.alert('成功', '提现申请已提交');
              loadDashboard();
            } catch (error: any) {
              Alert.alert('错误', error.message);
            }
          },
        },
      ],
      'plain-text'
    );
  };

  if (!dashboard) {
    return null;
  }

  const isActive = dashboard.provider.status === 'Active';

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.header}>
        <Text style={styles.headerTitle}>我的工作台</Text>
        <Pressable
          style={styles.settingsButton}
          onPress={() => router.push('/market/provider/settings')}
        >
          <Ionicons name="settings-outline" size={22} color={THEME.text} />
        </Pressable>
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }
      >
        {/* 余额卡片 */}
        <View style={styles.balanceCard}>
          <View style={styles.balanceHeader}>
            <Ionicons name="wallet-outline" size={20} color={THEME.primary} />
            <Text style={styles.balanceLabel}>可提现余额</Text>
          </View>
          <Text style={styles.balanceValue}>
            {formatBalance(dashboard.balance)} <Text style={styles.balanceUnit}>DUST</Text>
          </Text>
          <View style={styles.balanceActions}>
            <Pressable style={styles.withdrawButton} onPress={handleWithdraw}>
              <Text style={styles.withdrawButtonText}>提现</Text>
            </Pressable>
            <Pressable
              style={styles.detailButton}
              onPress={() => router.push('/market/provider/earnings')}
            >
              <Text style={styles.detailButtonText}>收益明细</Text>
            </Pressable>
          </View>
        </View>

        {/* 今日数据 */}
        <View style={styles.statsSection}>
          <Text style={styles.sectionTitle}>今日数据</Text>
          <View style={styles.statsRow}>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{dashboard.todayStats.newOrders}</Text>
              <Text style={styles.statLabel}>新订单</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={styles.statValue}>{dashboard.todayStats.completed}</Text>
              <Text style={styles.statLabel}>已完成</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={[styles.statValue, { color: THEME.primary }]}>
                {formatBalance(dashboard.todayStats.earnings)}
              </Text>
              <Text style={styles.statLabel}>收入(DUST)</Text>
            </View>
          </View>
        </View>

        {/* 待处理订单 */}
        <View style={styles.ordersSection}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>
              待处理订单 ({dashboard.pendingOrders.length})
            </Text>
            <Pressable onPress={() => router.push('/market/order/list?status=pending')}>
              <Text style={styles.viewAllText}>查看全部</Text>
            </Pressable>
          </View>

          {dashboard.pendingOrders.slice(0, 3).map((order) => (
            <View key={order.id} style={styles.orderCard}>
              <View style={styles.orderHeader}>
                <View style={[styles.statusDot, styles.pendingDot]} />
                <Text style={styles.orderId}>#{order.id}</Text>
                {order.isUrgent && (
                  <View style={styles.urgentBadge}>
                    <Ionicons name="flash" size={12} color={THEME.warning} />
                    <Text style={styles.urgentText}>加急</Text>
                  </View>
                )}
              </View>
              <Text style={styles.orderType}>
                {order.divinationTypeName} · {order.serviceTypeName}
              </Text>
              <Text style={styles.orderTime}>{formatTimeAgo(order.createdAt)}</Text>
              <View style={styles.orderActions}>
                <Pressable
                  style={styles.rejectButton}
                  onPress={() => handleReject(order.id)}
                >
                  <Text style={styles.rejectButtonText}>拒绝</Text>
                </Pressable>
                <Pressable
                  style={styles.acceptButton}
                  onPress={() => handleAccept(order.id)}
                >
                  <Text style={styles.acceptButtonText}>接单</Text>
                </Pressable>
              </View>
            </View>
          ))}

          {dashboard.pendingOrders.length === 0 && (
            <View style={styles.emptyBox}>
              <Text style={styles.emptyText}>暂无待处理订单</Text>
            </View>
          )}
        </View>

        {/* 进行中订单 */}
        <View style={styles.ordersSection}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>
              进行中订单 ({dashboard.activeOrders.length})
            </Text>
            <Pressable onPress={() => router.push('/market/order/list?status=active')}>
              <Text style={styles.viewAllText}>查看全部</Text>
            </Pressable>
          </View>

          {dashboard.activeOrders.slice(0, 3).map((order) => (
            <View key={order.id} style={styles.orderCard}>
              <View style={styles.orderHeader}>
                <View style={[styles.statusDot, styles.activeDot]} />
                <Text style={styles.orderId}>#{order.id}</Text>
              </View>
              <Text style={styles.orderType}>
                {order.divinationTypeName} · {order.serviceTypeName}
              </Text>
              <Text style={styles.orderStatus}>等待提交解读</Text>
              <View style={styles.orderActions}>
                <Pressable
                  style={styles.viewButton}
                  onPress={() => router.push(`/market/order/${order.id}`)}
                >
                  <Text style={styles.viewButtonText}>查看详情</Text>
                </Pressable>
                <Pressable
                  style={styles.submitButton}
                  onPress={() => router.push(`/market/order/submit-answer?orderId=${order.id}`)}
                >
                  <Text style={styles.submitButtonText}>提交解读</Text>
                </Pressable>
              </View>
            </View>
          ))}
        </View>

        {/* 接单状态 */}
        <View style={styles.statusCard}>
          <View style={styles.statusInfo}>
            <Text style={styles.statusLabel}>接单状态</Text>
            <View style={styles.statusBadge}>
              <View
                style={[
                  styles.statusIndicator,
                  isActive ? styles.activeIndicator : styles.pausedIndicator,
                ]}
              />
              <Text style={styles.statusText}>
                {isActive ? '接单中' : '已暂停'}
              </Text>
            </View>
          </View>
          <Pressable
            style={[
              styles.toggleButton,
              isActive ? styles.pauseButton : styles.resumeButton,
            ]}
            onPress={handleToggleStatus}
          >
            <Text
              style={[
                styles.toggleButtonText,
                isActive ? styles.pauseButtonText : styles.resumeButtonText,
              ]}
            >
              {isActive ? '暂停接单' : '恢复接单'}
            </Text>
          </Pressable>
        </View>
      </ScrollView>

      <BottomNavBar activeTab="market" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 56,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: THEME.card,
    borderBottomWidth: 1,
    borderBottomColor: THEME.borderLight,
  },
  headerTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: THEME.text,
  },
  settingsButton: {
    padding: 4,
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 12,
    paddingBottom: 100,
  },
  balanceCard: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 20,
    marginBottom: 12,
    ...SHADOWS.medium,
  },
  balanceHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    marginBottom: 12,
  },
  balanceLabel: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  balanceValue: {
    fontSize: 32,
    fontWeight: '700',
    color: THEME.text,
    marginBottom: 16,
  },
  balanceUnit: {
    fontSize: 16,
    fontWeight: '400',
    color: THEME.textSecondary,
  },
  balanceActions: {
    flexDirection: 'row',
    gap: 12,
  },
  withdrawButton: {
    flex: 1,
    height: 40,
    backgroundColor: THEME.primary,
    borderRadius: 6,
    justifyContent: 'center',
    alignItems: 'center',
  },
  withdrawButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.textInverse,
  },
  detailButton: {
    flex: 1,
    height: 40,
    borderWidth: 1,
    borderColor: THEME.border,
    borderRadius: 6,
    justifyContent: 'center',
    alignItems: 'center',
  },
  detailButtonText: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  statsSection: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    ...SHADOWS.small,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 12,
  },
  statsRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 24,
    fontWeight: '700',
    color: THEME.text,
    marginBottom: 4,
  },
  statLabel: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  statDivider: {
    width: 1,
    height: 40,
    backgroundColor: THEME.borderLight,
  },
  ordersSection: {
    marginBottom: 12,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 10,
  },
  viewAllText: {
    fontSize: 13,
    color: THEME.primary,
  },
  orderCard: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    marginBottom: 8,
    ...SHADOWS.small,
  },
  orderHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 6,
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  pendingDot: {
    backgroundColor: THEME.warning,
  },
  activeDot: {
    backgroundColor: THEME.success,
  },
  orderId: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.text,
  },
  urgentBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 2,
    paddingHorizontal: 6,
    paddingVertical: 2,
    backgroundColor: THEME.warning + '20',
    borderRadius: 4,
  },
  urgentText: {
    fontSize: 11,
    color: THEME.warning,
  },
  orderType: {
    fontSize: 13,
    color: THEME.textSecondary,
    marginBottom: 4,
  },
  orderTime: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginBottom: 12,
  },
  orderStatus: {
    fontSize: 13,
    color: THEME.info,
    marginBottom: 12,
  },
  orderActions: {
    flexDirection: 'row',
    gap: 10,
  },
  rejectButton: {
    flex: 1,
    height: 36,
    borderWidth: 1,
    borderColor: THEME.border,
    borderRadius: 6,
    justifyContent: 'center',
    alignItems: 'center',
  },
  rejectButtonText: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  acceptButton: {
    flex: 1,
    height: 36,
    backgroundColor: THEME.primary,
    borderRadius: 6,
    justifyContent: 'center',
    alignItems: 'center',
  },
  acceptButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.textInverse,
  },
  viewButton: {
    flex: 1,
    height: 36,
    borderWidth: 1,
    borderColor: THEME.border,
    borderRadius: 6,
    justifyContent: 'center',
    alignItems: 'center',
  },
  viewButtonText: {
    fontSize: 14,
    color: THEME.textSecondary,
  },
  submitButton: {
    flex: 1,
    height: 36,
    backgroundColor: THEME.success,
    borderRadius: 6,
    justifyContent: 'center',
    alignItems: 'center',
  },
  submitButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.textInverse,
  },
  emptyBox: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 24,
    alignItems: 'center',
    ...SHADOWS.small,
  },
  emptyText: {
    fontSize: 14,
    color: THEME.textTertiary,
  },
  statusCard: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 16,
    ...SHADOWS.small,
  },
  statusInfo: {
    gap: 4,
  },
  statusLabel: {
    fontSize: 13,
    color: THEME.textTertiary,
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  statusIndicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  activeIndicator: {
    backgroundColor: THEME.success,
  },
  pausedIndicator: {
    backgroundColor: THEME.textTertiary,
  },
  statusText: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
  },
  toggleButton: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
  },
  pauseButton: {
    backgroundColor: THEME.warning + '20',
  },
  resumeButton: {
    backgroundColor: THEME.success + '20',
  },
  toggleButtonText: {
    fontSize: 14,
    fontWeight: '500',
  },
  pauseButtonText: {
    color: THEME.warning,
  },
  resumeButtonText: {
    color: THEME.success,
  },
});
```

---

## 6. 评价页面

### 页面线框图

```
┌─────────────────────────────────────┐
│  ←  提交评价                        │
├─────────────────────────────────────┤
│  订单信息                           │
│  ┌─────────────────────────────────┐│
│  │ #12345 · 详细文字解卦           ││
│  │ 张三大师 · 梅花易数             ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  服务评价                           │
│  ┌─────────────────────────────────┐│
│  │ 总体评分                        ││
│  │ ☆ ☆ ☆ ☆ ☆                      ││
│  │                                 ││
│  │ 准确度                          ││
│  │ ☆ ☆ ☆ ☆ ☆                      ││
│  │                                 ││
│  │ 服务态度                        ││
│  │ ☆ ☆ ☆ ☆ ☆                      ││
│  │                                 ││
│  │ 响应速度                        ││
│  │ ☆ ☆ ☆ ☆ ☆                      ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  评价内容（选填）                   │
│  ┌─────────────────────────────────┐│
│  │ 分享您的服务体验，帮助其他      ││
│  │ 用户做出选择...                 ││
│  │                                 ││
│  │                         0/300   ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  ┌─────────────────────────────────┐│
│  │ 🔒 匿名评价                [○]  ││
│  │ 开启后将隐藏您的用户名          ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  [        提交评价        ]         │
└─────────────────────────────────────┘
```

### 评价页面代码

```tsx
// frontend/app/market/review/create.tsx

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  TextInput,
  Switch,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { useMarketApi } from '@/divination/market/hooks/useMarketApi';

// 评分输入组件
interface RatingInputProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
}

const RatingInput: React.FC<RatingInputProps> = ({ label, value, onChange }) => (
  <View style={styles.ratingRow}>
    <Text style={styles.ratingLabel}>{label}</Text>
    <View style={styles.starsRow}>
      {[1, 2, 3, 4, 5].map((star) => (
        <Pressable
          key={star}
          style={styles.starButton}
          onPress={() => onChange(star)}
        >
          <Ionicons
            name={star <= value ? 'star' : 'star-outline'}
            size={28}
            color={star <= value ? THEME.warning : '#DDD'}
          />
        </Pressable>
      ))}
    </View>
    <Text style={styles.ratingValue}>{value}分</Text>
  </View>
);

export default function CreateReviewPage() {
  const router = useRouter();
  const params = useLocalSearchParams();
  const { orderId } = params;

  const { getOrder, submitReview, loading } = useMarketApi();

  const [order, setOrder] = useState(null);
  const [overallRating, setOverallRating] = useState(5);
  const [accuracyRating, setAccuracyRating] = useState(5);
  const [attitudeRating, setAttitudeRating] = useState(5);
  const [responseRating, setResponseRating] = useState(5);
  const [content, setContent] = useState('');
  const [isAnonymous, setIsAnonymous] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    loadOrder();
  }, [orderId]);

  const loadOrder = async () => {
    const data = await getOrder(Number(orderId));
    setOrder(data);
  };

  const handleSubmit = async () => {
    if (overallRating === 0) {
      Alert.alert('提示', '请选择总体评分');
      return;
    }

    Alert.alert('确认提交', '评价提交后不可修改，确认提交？', [
      { text: '取消', style: 'cancel' },
      {
        text: '确认',
        onPress: async () => {
          setSubmitting(true);
          try {
            await submitReview({
              orderId: Number(orderId),
              overallRating,
              accuracyRating,
              attitudeRating,
              responseRating,
              content: content.trim() || undefined,
              isAnonymous,
            });

            Alert.alert('成功', '评价提交成功', [
              {
                text: '确定',
                onPress: () => router.back(),
              },
            ]);
          } catch (error: any) {
            Alert.alert('错误', error.message || '提交失败');
          } finally {
            setSubmitting(false);
          }
        },
      },
    ]);
  };

  if (loading || !order) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color={THEME.primary} />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color={THEME.text} />
        </Pressable>
        <Text style={styles.headerTitle}>提交评价</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
      >
        {/* 订单信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>订单信息</Text>
          <View style={styles.orderCard}>
            <Text style={styles.orderTitle}>
              #{order.id} · {order.packageName}
            </Text>
            <Text style={styles.orderMeta}>
              {order.providerName} · {order.divinationTypeName}
            </Text>
          </View>
        </View>

        {/* 评分区域 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>服务评价</Text>
          <View style={styles.ratingsCard}>
            <RatingInput
              label="总体评分"
              value={overallRating}
              onChange={setOverallRating}
            />
            <View style={styles.ratingDivider} />
            <RatingInput
              label="准确度"
              value={accuracyRating}
              onChange={setAccuracyRating}
            />
            <View style={styles.ratingDivider} />
            <RatingInput
              label="服务态度"
              value={attitudeRating}
              onChange={setAttitudeRating}
            />
            <View style={styles.ratingDivider} />
            <RatingInput
              label="响应速度"
              value={responseRating}
              onChange={setResponseRating}
            />
          </View>
        </View>

        {/* 评价内容 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>评价内容（选填）</Text>
          <View style={styles.inputWrapper}>
            <TextInput
              style={styles.contentInput}
              value={content}
              onChangeText={setContent}
              placeholder="分享您的服务体验，帮助其他用户做出选择..."
              placeholderTextColor={THEME.textTertiary}
              multiline
              numberOfLines={4}
              maxLength={300}
              textAlignVertical="top"
            />
            <Text style={styles.charCount}>{content.length}/300</Text>
          </View>
        </View>

        {/* 匿名选项 */}
        <View style={styles.section}>
          <View style={styles.anonymousCard}>
            <View style={styles.anonymousInfo}>
              <View style={styles.anonymousHeader}>
                <Ionicons name="eye-off-outline" size={18} color={THEME.textSecondary} />
                <Text style={styles.anonymousTitle}>匿名评价</Text>
              </View>
              <Text style={styles.anonymousDesc}>开启后将隐藏您的用户名</Text>
            </View>
            <Switch
              value={isAnonymous}
              onValueChange={setIsAnonymous}
              trackColor={{ false: '#E8E8E8', true: THEME.primary + '60' }}
              thumbColor={isAnonymous ? THEME.primary : '#FFF'}
            />
          </View>
        </View>
      </ScrollView>

      {/* 底部提交按钮 */}
      <View style={styles.footer}>
        <Pressable
          style={[styles.submitButton, submitting && styles.submitButtonDisabled]}
          onPress={handleSubmit}
          disabled={submitting}
        >
          {submitting ? (
            <ActivityIndicator color={THEME.textInverse} />
          ) : (
            <Text style={styles.submitButtonText}>提交评价</Text>
          )}
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: THEME.background,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: THEME.card,
    borderBottomWidth: 1,
    borderBottomColor: THEME.borderLight,
  },
  backButton: {
    padding: 4,
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME.text,
  },
  placeholder: {
    width: 32,
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 16,
    paddingBottom: 24,
  },
  section: {
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 10,
  },
  orderCard: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    ...SHADOWS.small,
  },
  orderTitle: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
    marginBottom: 4,
  },
  orderMeta: {
    fontSize: 13,
    color: THEME.textTertiary,
  },
  ratingsCard: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 16,
    ...SHADOWS.small,
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 8,
  },
  ratingLabel: {
    width: 70,
    fontSize: 14,
    color: THEME.textSecondary,
  },
  starsRow: {
    flex: 1,
    flexDirection: 'row',
    gap: 4,
  },
  starButton: {
    padding: 2,
  },
  ratingValue: {
    width: 36,
    fontSize: 14,
    color: THEME.warning,
    textAlign: 'right',
  },
  ratingDivider: {
    height: 1,
    backgroundColor: THEME.borderLight,
    marginVertical: 4,
  },
  inputWrapper: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    ...SHADOWS.small,
  },
  contentInput: {
    padding: 14,
    fontSize: 14,
    color: THEME.text,
    minHeight: 100,
    lineHeight: 20,
  },
  charCount: {
    fontSize: 12,
    color: THEME.textTertiary,
    textAlign: 'right',
    paddingRight: 14,
    paddingBottom: 10,
  },
  anonymousCard: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    ...SHADOWS.small,
  },
  anonymousInfo: {
    flex: 1,
  },
  anonymousHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  anonymousTitle: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
  },
  anonymousDesc: {
    fontSize: 13,
    color: THEME.textTertiary,
    marginTop: 4,
  },
  footer: {
    padding: 16,
    paddingBottom: 32,
    backgroundColor: THEME.card,
    borderTopWidth: 1,
    borderTopColor: THEME.borderLight,
  },
  submitButton: {
    height: 50,
    backgroundColor: THEME.primary,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  submitButtonDisabled: {
    opacity: 0.6,
  },
  submitButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.textInverse,
  },
});
```

---

## 7. 通用组件

### 等级徽章组件

```tsx
// frontend/src/divination/market/components/TierBadge.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../theme';

// 等级配置
const TIER_CONFIG: Record<number, { name: string; color: string; icon: string }> = {
  0: { name: '新手', color: THEME.tier.novice, icon: 'leaf-outline' },
  1: { name: '认证', color: THEME.tier.certified, icon: 'checkmark-circle-outline' },
  2: { name: '资深', color: THEME.tier.senior, icon: 'ribbon-outline' },
  3: { name: '专家', color: THEME.tier.expert, icon: 'diamond-outline' },
  4: { name: '大师', color: THEME.tier.master, icon: 'trophy-outline' },
};

interface TierBadgeProps {
  tier: number;
  size?: 'small' | 'medium' | 'large';
  showIcon?: boolean;
}

export const TierBadge: React.FC<TierBadgeProps> = ({
  tier,
  size = 'small',
  showIcon = true,
}) => {
  const config = TIER_CONFIG[tier] || TIER_CONFIG[0];

  const sizeStyles = {
    small: { paddingH: 6, paddingV: 2, fontSize: 11, iconSize: 10 },
    medium: { paddingH: 8, paddingV: 3, fontSize: 12, iconSize: 12 },
    large: { paddingH: 10, paddingV: 4, fontSize: 14, iconSize: 14 },
  };

  const s = sizeStyles[size];

  return (
    <View
      style={[
        styles.badge,
        {
          backgroundColor: config.color + '20',
          paddingHorizontal: s.paddingH,
          paddingVertical: s.paddingV,
        },
      ]}
    >
      {showIcon && (
        <Ionicons
          name={config.icon as any}
          size={s.iconSize}
          color={config.color}
          style={styles.icon}
        />
      )}
      <Text style={[styles.text, { color: config.color, fontSize: s.fontSize }]}>
        {config.name}
      </Text>
    </View>
  );
};

const styles = StyleSheet.create({
  badge: {
    flexDirection: 'row',
    alignItems: 'center',
    borderRadius: 4,
  },
  icon: {
    marginRight: 3,
  },
  text: {
    fontWeight: '500',
  },
});
```

### 价格显示组件

```tsx
// frontend/src/divination/market/components/PriceDisplay.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { THEME } from '../theme';
import { formatBalance } from '../utils/market.utils';

interface PriceDisplayProps {
  amount: bigint;
  size?: 'small' | 'medium' | 'large';
  showUnit?: boolean;
  color?: string;
}

export const PriceDisplay: React.FC<PriceDisplayProps> = ({
  amount,
  size = 'medium',
  showUnit = true,
  color = THEME.primary,
}) => {
  const sizeStyles = {
    small: { valueSize: 14, unitSize: 10 },
    medium: { valueSize: 18, unitSize: 12 },
    large: { valueSize: 28, unitSize: 16 },
  };

  const s = sizeStyles[size];

  return (
    <View style={styles.container}>
      <Text style={[styles.value, { fontSize: s.valueSize, color }]}>
        {formatBalance(amount)}
      </Text>
      {showUnit && (
        <Text style={[styles.unit, { fontSize: s.unitSize, color }]}>
          DUST
        </Text>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'baseline',
    gap: 4,
  },
  value: {
    fontWeight: '700',
  },
  unit: {
    fontWeight: '400',
  },
});
```

### 擅长领域标签组件

```tsx
// frontend/src/divination/market/components/SpecialtyTags.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { THEME } from '../theme';

// 擅长领域配置（对应位图）
const SPECIALTIES: Record<number, { name: string; color: string }> = {
  0: { name: '事业', color: '#3498DB' },
  1: { name: '感情', color: '#E91E63' },
  2: { name: '财运', color: '#F39C12' },
  3: { name: '健康', color: '#2ECC71' },
  4: { name: '学业', color: '#9B59B6' },
  5: { name: '出行', color: '#1ABC9C' },
  6: { name: '官司', color: '#E74C3C' },
  7: { name: '寻物', color: '#34495E' },
  8: { name: '风水', color: '#8E44AD' },
  9: { name: '择日', color: '#D35400' },
};

interface SpecialtyTagsProps {
  bitmap: number;
  maxShow?: number;
}

export const SpecialtyTags: React.FC<SpecialtyTagsProps> = ({
  bitmap,
  maxShow = 5,
}) => {
  // 解析位图获取擅长领域
  const specialties: { name: string; color: string }[] = [];
  for (let i = 0; i < 10; i++) {
    if (bitmap & (1 << i)) {
      specialties.push(SPECIALTIES[i]);
    }
  }

  const displayTags = specialties.slice(0, maxShow);
  const remaining = specialties.length - maxShow;

  return (
    <View style={styles.container}>
      {displayTags.map((specialty, index) => (
        <View
          key={index}
          style={[styles.tag, { backgroundColor: specialty.color + '15' }]}
        >
          <Text style={[styles.tagText, { color: specialty.color }]}>
            {specialty.name}
          </Text>
        </View>
      ))}
      {remaining > 0 && (
        <View style={styles.moreTag}>
          <Text style={styles.moreText}>+{remaining}</Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 6,
  },
  tag: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 4,
  },
  tagText: {
    fontSize: 12,
    fontWeight: '500',
  },
  moreTag: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    backgroundColor: THEME.background,
    borderRadius: 4,
  },
  moreText: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
});
```

---

## 8. API 集成

### 类型定义

```typescript
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
```

### API Hook

```typescript
// frontend/src/divination/market/hooks/useMarketApi.ts

import { useState, useCallback } from 'react';
import { useWalletStore } from '@/stores/wallet.store';
import { useApiConnection } from '@/api';
import {
  Provider,
  ServicePackage,
  Order,
  Review,
  DivinationType,
} from '../types/market.types';

export const useMarketApi = () => {
  const { api } = useApiConnection();
  const { activeAccount, signAndSend } = useWalletStore();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 获取提供者列表
  const getProviders = useCallback(
    async (params?: {
      filterType?: 'all' | number;
      sortType?: string;
    }): Promise<Provider[]> => {
      setLoading(true);
      try {
        const entries = await api.query.divinationMarket.providers.entries();
        let providers = entries
          .map(([key, value]) => ({
            account: key.args[0].toString(),
            ...value.toHuman(),
          }))
          .filter((p) => p.status === 'Active');

        // 筛选占卜类型
        if (params?.filterType && params.filterType !== 'all') {
          providers = providers.filter(
            (p) => p.supportedTypes & (1 << params.filterType)
          );
        }

        // 获取每个提供者的套餐
        for (const provider of providers) {
          const packages = await api.query.divinationMarket.packages.entries(
            provider.account
          );
          provider.packages = packages
            .map(([, v]) => v.toHuman())
            .filter((p) => p.isActive);
        }

        return providers;
      } catch (err: any) {
        setError(err.message);
        return [];
      } finally {
        setLoading(false);
      }
    },
    [api]
  );

  // 获取单个提供者
  const getProvider = useCallback(
    async (account: string): Promise<Provider | null> => {
      setLoading(true);
      try {
        const provider = await api.query.divinationMarket.providers(account);
        if (provider.isNone) return null;
        return { account, ...provider.unwrap().toHuman() };
      } catch (err: any) {
        setError(err.message);
        return null;
      } finally {
        setLoading(false);
      }
    },
    [api]
  );

  // 获取套餐
  const getPackage = useCallback(
    async (provider: string, packageId: number): Promise<ServicePackage | null> => {
      try {
        const pkg = await api.query.divinationMarket.packages(provider, packageId);
        if (pkg.isNone) return null;
        return { id: packageId, ...pkg.unwrap().toHuman() };
      } catch (err: any) {
        setError(err.message);
        return null;
      }
    },
    [api]
  );

  // 创建订单
  const createOrder = useCallback(
    async (params: {
      provider: string;
      divinationType: DivinationType;
      hexagramId: number;
      packageId: number;
      question: string;
      isUrgent: boolean;
    }): Promise<number> => {
      // 上传问题到 IPFS
      const questionCid = await uploadToIPFS(params.question);

      const tx = api.tx.divinationMarket.createOrder(
        params.provider,
        params.divinationType,
        params.hexagramId,
        params.packageId,
        questionCid,
        params.isUrgent
      );

      const result = await signAndSend(tx);
      
      // 从事件中获取订单 ID
      const orderCreatedEvent = result.events.find(
        (e) => e.event.method === 'OrderCreated'
      );
      return orderCreatedEvent?.event.data[0].toNumber();
    },
    [api, signAndSend]
  );

  // 接受订单
  const acceptOrder = useCallback(
    async (orderId: number): Promise<void> => {
      const tx = api.tx.divinationMarket.acceptOrder(orderId);
      await signAndSend(tx);
    },
    [api, signAndSend]
  );

  // 拒绝订单
  const rejectOrder = useCallback(
    async (orderId: number): Promise<void> => {
      const tx = api.tx.divinationMarket.rejectOrder(orderId);
      await signAndSend(tx);
    },
    [api, signAndSend]
  );

  // 提交解读
  const submitAnswer = useCallback(
    async (orderId: number, answer: string): Promise<void> => {
      const answerCid = await uploadToIPFS(answer);
      const tx = api.tx.divinationMarket.submitAnswer(orderId, answerCid);
      await signAndSend(tx);
    },
    [api, signAndSend]
  );

  // 提交追问
  const submitFollowUp = useCallback(
    async (orderId: number, question: string): Promise<void> => {
      const questionCid = await uploadToIPFS(question);
      const tx = api.tx.divinationMarket.submitFollowUp(orderId, questionCid);
      await signAndSend(tx);
    },
    [api, signAndSend]
  );

  // 提交评价
  const submitReview = useCallback(
    async (params: {
      orderId: number;
      overallRating: number;
      accuracyRating: number;
      attitudeRating: number;
      responseRating: number;
      content?: string;
      isAnonymous: boolean;
    }): Promise<void> => {
      const contentCid = params.content
        ? await uploadToIPFS(params.content)
        : null;

      const tx = api.tx.divinationMarket.submitReview(
        params.orderId,
        params.overallRating,
        params.accuracyRating,
        params.attitudeRating,
        params.responseRating,
        contentCid,
        params.isAnonymous
      );
      await signAndSend(tx);
    },
    [api, signAndSend]
  );

  // 申请提现
  const requestWithdrawal = useCallback(
    async (amount: bigint): Promise<void> => {
      const tx = api.tx.divinationMarket.requestWithdrawal(amount);
      await signAndSend(tx);
    },
    [api, signAndSend]
  );

  // 暂停接单
  const pauseProvider = useCallback(async (): Promise<void> => {
    const tx = api.tx.divinationMarket.pauseProvider();
    await signAndSend(tx);
  }, [api, signAndSend]);

  // 恢复接单
  const resumeProvider = useCallback(async (): Promise<void> => {
    const tx = api.tx.divinationMarket.resumeProvider();
    await signAndSend(tx);
  }, [api, signAndSend]);

  return {
    loading,
    error,
    getProviders,
    getProvider,
    getPackage,
    createOrder,
    acceptOrder,
    rejectOrder,
    submitAnswer,
    submitFollowUp,
    submitReview,
    requestWithdrawal,
    pauseProvider,
    resumeProvider,
  };
};

// IPFS 上传辅助函数
async function uploadToIPFS(content: string): Promise<string> {
  // 实现 IPFS 上传逻辑
  // 返回 CID
  return 'Qm...';
}
```

### 工具函数

```typescript
// frontend/src/divination/market/utils/market.utils.ts

import { DivinationType, ServiceType } from '../types/market.types';

// 格式化余额（假设 12 位小数）
export const formatBalance = (amount: bigint): string => {
  const divisor = 10n ** 12n;
  const whole = amount / divisor;
  const fraction = amount % divisor;
  
  if (fraction === 0n) {
    return whole.toString();
  }
  
  const fractionStr = fraction.toString().padStart(12, '0').replace(/0+$/, '');
  return `${whole}.${fractionStr.slice(0, 2)}`;
};

// 占卜类型名称
const DIVINATION_TYPE_NAMES: Record<DivinationType, string> = {
  [DivinationType.Meihua]: '梅花易数',
  [DivinationType.Bazi]: '八字命理',
  [DivinationType.Liuyao]: '六爻',
  [DivinationType.Qimen]: '奇门遁甲',
  [DivinationType.Ziwei]: '紫微斗数',
  [DivinationType.Tarot]: '塔罗牌',
  [DivinationType.Daliuren]: '大六壬',
};

export const getDivinationTypeName = (type: DivinationType): string => {
  return DIVINATION_TYPE_NAMES[type] || '未知';
};

// 从位图获取占卜类型名称列表
export const getDivinationTypeNames = (bitmap: number): string[] => {
  const names: string[] = [];
  for (let i = 0; i < 7; i++) {
    if (bitmap & (1 << i)) {
      names.push(DIVINATION_TYPE_NAMES[i as DivinationType]);
    }
  }
  return names;
};

// 服务类型名称
const SERVICE_TYPE_NAMES: Record<ServiceType, string> = {
  [ServiceType.TextReading]: '文字解读',
  [ServiceType.VoiceReading]: '语音解读',
  [ServiceType.VideoReading]: '视频解读',
  [ServiceType.LiveConsultation]: '实时咨询',
};

export const getServiceTypeName = (type: ServiceType): string => {
  return SERVICE_TYPE_NAMES[type] || '未知';
};

// 格式化时间（相对时间）
export const formatTimeAgo = (timestamp: number): string => {
  const now = Date.now();
  const diff = now - timestamp;
  
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);
  
  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes}分钟前`;
  if (hours < 24) return `${hours}小时前`;
  if (days < 30) return `${days}天前`;
  
  const date = new Date(timestamp);
  return `${date.getMonth() + 1}月${date.getDate()}日`;
};

// 格式化日期时间
export const formatDateTime = (timestamp: number): string => {
  const date = new Date(timestamp);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')} ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;
};
```

---

## 常量定义

```typescript
// frontend/src/divination/market/constants/market.constants.ts

import { DivinationType, ServiceType } from '../types/market.types';

// 占卜类型配置
export const DIVINATION_TYPES = [
  { id: DivinationType.Meihua, name: '梅花易数', route: 'meihua', color: '#E91E63' },
  { id: DivinationType.Bazi, name: '八字命理', route: 'bazi', color: '#E74C3C' },
  { id: DivinationType.Liuyao, name: '六爻', route: 'liuyao', color: '#F39C12' },
  { id: DivinationType.Qimen, name: '奇门遁甲', route: 'qimen', color: '#3498DB' },
  { id: DivinationType.Ziwei, name: '紫微斗数', route: 'ziwei', color: '#9B59B6' },
  { id: DivinationType.Tarot, name: '塔罗牌', route: 'tarot', color: '#673AB7' },
  { id: DivinationType.Daliuren, name: '大六壬', route: 'daliuren', color: '#1ABC9C' },
];

// 服务类型配置
export const SERVICE_TYPES = [
  { id: ServiceType.TextReading, name: '文字解读', icon: 'document-text-outline' },
  { id: ServiceType.VoiceReading, name: '语音解读', icon: 'mic-outline' },
  { id: ServiceType.VideoReading, name: '视频解读', icon: 'videocam-outline' },
  { id: ServiceType.LiveConsultation, name: '实时咨询', icon: 'chatbubbles-outline' },
];

// 等级配置
export const TIER_CONFIG = [
  { level: 0, name: '新手', minOrders: 0, minRating: 0, feeRate: 2000 },
  { level: 1, name: '认证', minOrders: 10, minRating: 350, feeRate: 1500 },
  { level: 2, name: '资深', minOrders: 50, minRating: 400, feeRate: 1200 },
  { level: 3, name: '专家', minOrders: 200, minRating: 450, feeRate: 1000 },
  { level: 4, name: '大师', minOrders: 500, minRating: 480, feeRate: 800 },
];

// 擅长领域配置
export const SPECIALTIES = [
  { bit: 0, name: '事业', icon: 'briefcase-outline' },
  { bit: 1, name: '感情', icon: 'heart-outline' },
  { bit: 2, name: '财运', icon: 'cash-outline' },
  { bit: 3, name: '健康', icon: 'fitness-outline' },
  { bit: 4, name: '学业', icon: 'school-outline' },
  { bit: 5, name: '出行', icon: 'airplane-outline' },
  { bit: 6, name: '官司', icon: 'hammer-outline' },
  { bit: 7, name: '寻物', icon: 'search-outline' },
  { bit: 8, name: '风水', icon: 'home-outline' },
  { bit: 9, name: '择日', icon: 'calendar-outline' },
];
```

---

## 9. IPFS 集成实现

### IPFS 服务配置

```typescript
// frontend/src/divination/market/services/ipfs.service.ts

import { create as createIPFSClient, IPFSHTTPClient } from 'ipfs-http-client';
import * as FileSystem from 'expo-file-system';
import { Buffer } from 'buffer';

// IPFS 网关配置
const IPFS_CONFIG = {
  // 主节点（Pinata）
  primary: {
    host: 'api.pinata.cloud',
    port: 443,
    protocol: 'https',
    headers: {
      pinata_api_key: process.env.PINATA_API_KEY || '',
      pinata_secret_api_key: process.env.PINATA_SECRET_KEY || '',
    },
  },
  // 备用节点（Infura）
  fallback: {
    host: 'ipfs.infura.io',
    port: 5001,
    protocol: 'https',
    headers: {
      authorization: `Basic ${Buffer.from(
        `${process.env.INFURA_PROJECT_ID}:${process.env.INFURA_SECRET}`
      ).toString('base64')}`,
    },
  },
  // 公共网关（只读）
  gateways: [
    'https://gateway.pinata.cloud/ipfs/',
    'https://ipfs.io/ipfs/',
    'https://cloudflare-ipfs.com/ipfs/',
    'https://dweb.link/ipfs/',
  ],
  // 上传重试配置
  upload: {
    maxRetries: 3,
    retryDelay: 1000,
    timeout: 30000,
  },
};

// 加密配置（用于敏感内容）
interface EncryptionConfig {
  enabled: boolean;
  algorithm: 'AES-GCM';
  keyLength: 256;
}

class IPFSService {
  private primaryClient: IPFSHTTPClient | null = null;
  private fallbackClient: IPFSHTTPClient | null = null;
  private currentGatewayIndex = 0;

  constructor() {
    this.initClients();
  }

  private initClients() {
    try {
      this.primaryClient = createIPFSClient(IPFS_CONFIG.primary);
    } catch (error) {
      console.warn('Failed to init primary IPFS client:', error);
    }

    try {
      this.fallbackClient = createIPFSClient(IPFS_CONFIG.fallback);
    } catch (error) {
      console.warn('Failed to init fallback IPFS client:', error);
    }
  }

  /**
   * 上传文本内容到 IPFS
   * @param content 文本内容
   * @param options 上传选项
   * @returns CID 字符串
   */
  async uploadText(
    content: string,
    options?: {
      encrypt?: boolean;
      encryptionKey?: CryptoKey;
      metadata?: Record<string, string>;
    }
  ): Promise<string> {
    let data = content;

    // 加密处理
    if (options?.encrypt && options.encryptionKey) {
      data = await this.encryptContent(content, options.encryptionKey);
    }

    const blob = new Blob([data], { type: 'text/plain' });
    return this.uploadWithRetry(blob, options?.metadata);
  }

  /**
   * 上传文件到 IPFS（支持语音/视频）
   * @param filePath 本地文件路径
   * @param options 上传选项
   * @returns CID 字符串
   */
  async uploadFile(
    filePath: string,
    options?: {
      encrypt?: boolean;
      encryptionKey?: CryptoKey;
      onProgress?: (progress: number) => void;
    }
  ): Promise<string> {
    // 读取文件信息
    const fileInfo = await FileSystem.getInfoAsync(filePath);
    if (!fileInfo.exists) {
      throw new Error('File not found');
    }

    // 大文件分片上传阈值：10MB
    const CHUNK_THRESHOLD = 10 * 1024 * 1024;

    if (fileInfo.size && fileInfo.size > CHUNK_THRESHOLD) {
      return this.uploadLargeFile(filePath, fileInfo.size, options);
    }

    // 小文件直接上传
    const base64Content = await FileSystem.readAsStringAsync(filePath, {
      encoding: FileSystem.EncodingType.Base64,
    });

    let buffer = Buffer.from(base64Content, 'base64');

    // 加密处理
    if (options?.encrypt && options.encryptionKey) {
      const encrypted = await this.encryptBuffer(buffer, options.encryptionKey);
      buffer = Buffer.from(encrypted);
    }

    return this.uploadWithRetry(buffer);
  }

  /**
   * 大文件分片上传
   */
  private async uploadLargeFile(
    filePath: string,
    fileSize: number,
    options?: {
      encrypt?: boolean;
      encryptionKey?: CryptoKey;
      onProgress?: (progress: number) => void;
    }
  ): Promise<string> {
    const CHUNK_SIZE = 1024 * 1024; // 1MB chunks
    const totalChunks = Math.ceil(fileSize / CHUNK_SIZE);
    const chunkCids: string[] = [];

    for (let i = 0; i < totalChunks; i++) {
      const start = i * CHUNK_SIZE;
      const end = Math.min(start + CHUNK_SIZE, fileSize);

      // 读取分片
      const chunk = await FileSystem.readAsStringAsync(filePath, {
        encoding: FileSystem.EncodingType.Base64,
        position: start,
        length: end - start,
      });

      let buffer = Buffer.from(chunk, 'base64');

      // 加密分片
      if (options?.encrypt && options.encryptionKey) {
        const encrypted = await this.encryptBuffer(buffer, options.encryptionKey);
        buffer = Buffer.from(encrypted);
      }

      // 上传分片
      const cid = await this.uploadWithRetry(buffer);
      chunkCids.push(cid);

      // 进度回调
      options?.onProgress?.((i + 1) / totalChunks);
    }

    // 创建文件清单并上传
    const manifest = {
      type: 'chunked-file',
      totalSize: fileSize,
      chunkSize: CHUNK_SIZE,
      chunks: chunkCids,
      encrypted: options?.encrypt || false,
    };

    return this.uploadText(JSON.stringify(manifest));
  }

  /**
   * 带重试的上传逻辑
   */
  private async uploadWithRetry(
    data: Blob | Buffer,
    metadata?: Record<string, string>
  ): Promise<string> {
    const { maxRetries, retryDelay, timeout } = IPFS_CONFIG.upload;
    let lastError: Error | null = null;

    // 尝试主节点
    for (let attempt = 0; attempt < maxRetries; attempt++) {
      try {
        if (this.primaryClient) {
          const result = await Promise.race([
            this.primaryClient.add(data, {
              pin: true,
              cidVersion: 1,
            }),
            new Promise<never>((_, reject) =>
              setTimeout(() => reject(new Error('Upload timeout')), timeout)
            ),
          ]);
          return result.cid.toString();
        }
      } catch (error: any) {
        lastError = error;
        console.warn(`Primary upload attempt ${attempt + 1} failed:`, error.message);
        await this.delay(retryDelay * (attempt + 1));
      }
    }

    // 尝试备用节点
    for (let attempt = 0; attempt < maxRetries; attempt++) {
      try {
        if (this.fallbackClient) {
          const result = await Promise.race([
            this.fallbackClient.add(data, {
              pin: true,
              cidVersion: 1,
            }),
            new Promise<never>((_, reject) =>
              setTimeout(() => reject(new Error('Upload timeout')), timeout)
            ),
          ]);
          return result.cid.toString();
        }
      } catch (error: any) {
        lastError = error;
        console.warn(`Fallback upload attempt ${attempt + 1} failed:`, error.message);
        await this.delay(retryDelay * (attempt + 1));
      }
    }

    throw new Error(`IPFS upload failed after all retries: ${lastError?.message}`);
  }

  /**
   * 从 IPFS 获取内容（带网关故障转移）
   * @param cid Content ID
   * @returns 内容字符串
   */
  async fetchContent(
    cid: string,
    options?: {
      decrypt?: boolean;
      decryptionKey?: CryptoKey;
    }
  ): Promise<string> {
    const { gateways } = IPFS_CONFIG;
    let lastError: Error | null = null;

    // 从当前网关索引开始尝试
    for (let i = 0; i < gateways.length; i++) {
      const gatewayIndex = (this.currentGatewayIndex + i) % gateways.length;
      const gateway = gateways[gatewayIndex];

      try {
        const response = await fetch(`${gateway}${cid}`, {
          method: 'GET',
          headers: {
            'Accept': 'text/plain,application/json,*/*',
          },
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }

        let content = await response.text();

        // 解密处理
        if (options?.decrypt && options.decryptionKey) {
          content = await this.decryptContent(content, options.decryptionKey);
        }

        // 更新当前网关为成功的那个
        this.currentGatewayIndex = gatewayIndex;
        return content;
      } catch (error: any) {
        lastError = error;
        console.warn(`Gateway ${gateway} failed:`, error.message);
      }
    }

    throw new Error(`Failed to fetch from all gateways: ${lastError?.message}`);
  }

  /**
   * 检查 CID 是否有效/可访问
   */
  async checkAvailability(cid: string): Promise<boolean> {
    try {
      const response = await fetch(`${IPFS_CONFIG.gateways[0]}${cid}`, {
        method: 'HEAD',
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * 加密内容（AES-GCM）
   */
  private async encryptContent(content: string, key: CryptoKey): Promise<string> {
    const encoder = new TextEncoder();
    const data = encoder.encode(content);

    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      data
    );

    // 将 IV 和密文合并，然后 base64 编码
    const combined = new Uint8Array(iv.length + encrypted.byteLength);
    combined.set(iv);
    combined.set(new Uint8Array(encrypted), iv.length);

    return Buffer.from(combined).toString('base64');
  }

  /**
   * 加密 Buffer
   */
  private async encryptBuffer(buffer: Buffer, key: CryptoKey): Promise<ArrayBuffer> {
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      buffer
    );

    const combined = new Uint8Array(iv.length + encrypted.byteLength);
    combined.set(iv);
    combined.set(new Uint8Array(encrypted), iv.length);

    return combined.buffer;
  }

  /**
   * 解密内容
   */
  private async decryptContent(encryptedBase64: string, key: CryptoKey): Promise<string> {
    const combined = Buffer.from(encryptedBase64, 'base64');
    const iv = combined.slice(0, 12);
    const ciphertext = combined.slice(12);

    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      key,
      ciphertext
    );

    const decoder = new TextDecoder();
    return decoder.decode(decrypted);
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

// 单例导出
export const ipfsService = new IPFSService();

// 便捷函数导出
export const uploadToIPFS = ipfsService.uploadText.bind(ipfsService);
export const uploadFileToIPFS = ipfsService.uploadFile.bind(ipfsService);
export const fetchFromIPFS = ipfsService.fetchContent.bind(ipfsService);
```

### 加密密钥管理

```typescript
// frontend/src/divination/market/services/encryption.service.ts

import * as SecureStore from 'expo-secure-store';
import { Buffer } from 'buffer';

const KEY_STORAGE_PREFIX = 'stardust_encryption_key_';

export class EncryptionService {
  /**
   * 为订单生成加密密钥
   * 使用提供者公钥和用户私钥派生共享密钥
   */
  async generateOrderKey(
    orderId: number,
    providerPublicKey: string,
    userPrivateKey: string
  ): Promise<CryptoKey> {
    // 使用 ECDH 派生共享密钥
    const providerKey = await this.importPublicKey(providerPublicKey);
    const userKey = await this.importPrivateKey(userPrivateKey);

    const sharedSecret = await crypto.subtle.deriveBits(
      { name: 'ECDH', public: providerKey },
      userKey,
      256
    );

    // 从共享密钥派生 AES 密钥
    const aesKey = await crypto.subtle.importKey(
      'raw',
      sharedSecret,
      { name: 'AES-GCM', length: 256 },
      true,
      ['encrypt', 'decrypt']
    );

    // 存储密钥用于后续解密
    await this.storeKey(orderId, aesKey);

    return aesKey;
  }

  /**
   * 获取订单的加密密钥
   */
  async getOrderKey(orderId: number): Promise<CryptoKey | null> {
    const keyData = await SecureStore.getItemAsync(`${KEY_STORAGE_PREFIX}${orderId}`);
    if (!keyData) return null;

    return crypto.subtle.importKey(
      'raw',
      Buffer.from(keyData, 'base64'),
      { name: 'AES-GCM', length: 256 },
      true,
      ['encrypt', 'decrypt']
    );
  }

  /**
   * 存储密钥
   */
  private async storeKey(orderId: number, key: CryptoKey): Promise<void> {
    const exported = await crypto.subtle.exportKey('raw', key);
    const keyData = Buffer.from(exported).toString('base64');
    await SecureStore.setItemAsync(`${KEY_STORAGE_PREFIX}${orderId}`, keyData);
  }

  private async importPublicKey(pem: string): Promise<CryptoKey> {
    const binaryDer = Buffer.from(pem, 'base64');
    return crypto.subtle.importKey(
      'spki',
      binaryDer,
      { name: 'ECDH', namedCurve: 'P-256' },
      true,
      []
    );
  }

  private async importPrivateKey(pem: string): Promise<CryptoKey> {
    const binaryDer = Buffer.from(pem, 'base64');
    return crypto.subtle.importKey(
      'pkcs8',
      binaryDer,
      { name: 'ECDH', namedCurve: 'P-256' },
      true,
      ['deriveBits']
    );
  }
}

export const encryptionService = new EncryptionService();
```

---

## 10. 缺失的辅助页面

### 搜索页面

```tsx
// frontend/app/market/search.tsx

import React, { useState, useCallback, useRef } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  ScrollView,
  Pressable,
  ActivityIndicator,
  Keyboard,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { ProviderCard } from '@/divination/market/components/ProviderCard';
import { useMarketApi } from '@/divination/market/hooks/useMarketApi';
import { useDebounce } from '@/hooks/useDebounce';
import AsyncStorage from '@react-native-async-storage/async-storage';

const SEARCH_HISTORY_KEY = 'market_search_history';
const MAX_HISTORY_ITEMS = 10;

// 热门搜索词
const HOT_KEYWORDS = ['事业', '感情', '财运', '八字', '梅花', '塔罗'];

export default function MarketSearchPage() {
  const router = useRouter();
  const { searchProviders, loading } = useMarketApi();
  const inputRef = useRef<TextInput>(null);

  const [keyword, setKeyword] = useState('');
  const [searchHistory, setSearchHistory] = useState<string[]>([]);
  const [results, setResults] = useState([]);
  const [hasSearched, setHasSearched] = useState(false);

  // 加载搜索历史
  React.useEffect(() => {
    loadHistory();
    setTimeout(() => inputRef.current?.focus(), 100);
  }, []);

  const loadHistory = async () => {
    try {
      const history = await AsyncStorage.getItem(SEARCH_HISTORY_KEY);
      if (history) {
        setSearchHistory(JSON.parse(history));
      }
    } catch (error) {
      console.error('Failed to load search history:', error);
    }
  };

  const saveHistory = async (newKeyword: string) => {
    try {
      const updated = [
        newKeyword,
        ...searchHistory.filter((k) => k !== newKeyword),
      ].slice(0, MAX_HISTORY_ITEMS);

      setSearchHistory(updated);
      await AsyncStorage.setItem(SEARCH_HISTORY_KEY, JSON.stringify(updated));
    } catch (error) {
      console.error('Failed to save search history:', error);
    }
  };

  const clearHistory = async () => {
    setSearchHistory([]);
    await AsyncStorage.removeItem(SEARCH_HISTORY_KEY);
  };

  // 防抖搜索
  const debouncedSearch = useDebounce(async (searchKeyword: string) => {
    if (!searchKeyword.trim()) {
      setResults([]);
      setHasSearched(false);
      return;
    }

    const data = await searchProviders(searchKeyword);
    setResults(data);
    setHasSearched(true);
  }, 300);

  const handleSearch = useCallback((text: string) => {
    setKeyword(text);
    debouncedSearch(text);
  }, [debouncedSearch]);

  const handleSubmit = () => {
    Keyboard.dismiss();
    if (keyword.trim()) {
      saveHistory(keyword.trim());
    }
  };

  const handleKeywordClick = (kw: string) => {
    setKeyword(kw);
    saveHistory(kw);
    debouncedSearch(kw);
  };

  return (
    <View style={styles.container}>
      {/* 搜索栏 */}
      <View style={styles.searchBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color={THEME.text} />
        </Pressable>
        <View style={styles.inputWrapper}>
          <Ionicons name="search-outline" size={18} color={THEME.textTertiary} />
          <TextInput
            ref={inputRef}
            style={styles.input}
            value={keyword}
            onChangeText={handleSearch}
            onSubmitEditing={handleSubmit}
            placeholder="搜索大师、擅长领域..."
            placeholderTextColor={THEME.textTertiary}
            returnKeyType="search"
            autoCorrect={false}
          />
          {keyword.length > 0 && (
            <Pressable onPress={() => handleSearch('')}>
              <Ionicons name="close-circle" size={18} color={THEME.textTertiary} />
            </Pressable>
          )}
        </View>
        <Pressable style={styles.searchButton} onPress={handleSubmit}>
          <Text style={styles.searchButtonText}>搜索</Text>
        </Pressable>
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
        keyboardShouldPersistTaps="handled"
      >
        {/* 未搜索时显示历史和热门 */}
        {!hasSearched && (
          <>
            {searchHistory.length > 0 && (
              <View style={styles.section}>
                <View style={styles.sectionHeader}>
                  <Text style={styles.sectionTitle}>搜索历史</Text>
                  <Pressable onPress={clearHistory}>
                    <Ionicons name="trash-outline" size={18} color={THEME.textTertiary} />
                  </Pressable>
                </View>
                <View style={styles.tagsContainer}>
                  {searchHistory.map((item, index) => (
                    <Pressable
                      key={index}
                      style={styles.historyTag}
                      onPress={() => handleKeywordClick(item)}
                    >
                      <Ionicons name="time-outline" size={14} color={THEME.textTertiary} />
                      <Text style={styles.historyTagText}>{item}</Text>
                    </Pressable>
                  ))}
                </View>
              </View>
            )}

            <View style={styles.section}>
              <Text style={styles.sectionTitle}>热门搜索</Text>
              <View style={styles.tagsContainer}>
                {HOT_KEYWORDS.map((item, index) => (
                  <Pressable
                    key={index}
                    style={styles.hotTag}
                    onPress={() => handleKeywordClick(item)}
                  >
                    <Ionicons name="flame-outline" size={14} color={THEME.warning} />
                    <Text style={styles.hotTagText}>{item}</Text>
                  </Pressable>
                ))}
              </View>
            </View>
          </>
        )}

        {/* 搜索结果 */}
        {hasSearched && (
          <>
            {loading ? (
              <View style={styles.loadingBox}>
                <ActivityIndicator size="large" color={THEME.primary} />
              </View>
            ) : results.length === 0 ? (
              <View style={styles.emptyBox}>
                <Ionicons name="search-outline" size={48} color={THEME.textTertiary} />
                <Text style={styles.emptyText}>未找到相关服务提供者</Text>
              </View>
            ) : (
              <>
                <Text style={styles.resultCount}>找到 {results.length} 位相关大师</Text>
                {results.map((provider) => (
                  <ProviderCard
                    key={provider.account}
                    provider={provider}
                    packages={provider.packages}
                  />
                ))}
              </>
            )}
          </>
        )}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: THEME.background, maxWidth: 414, width: '100%', alignSelf: 'center' },
  searchBar: { flexDirection: 'row', alignItems: 'center', paddingTop: 50, paddingHorizontal: 12, paddingBottom: 12, backgroundColor: THEME.card, borderBottomWidth: 1, borderBottomColor: THEME.borderLight, gap: 8 },
  backButton: { padding: 4 },
  inputWrapper: { flex: 1, flexDirection: 'row', alignItems: 'center', backgroundColor: THEME.background, borderRadius: 8, paddingHorizontal: 12, height: 40, gap: 8 },
  input: { flex: 1, fontSize: 15, color: THEME.text },
  searchButton: { paddingHorizontal: 12, paddingVertical: 8 },
  searchButtonText: { fontSize: 15, color: THEME.primary, fontWeight: '500' },
  content: { flex: 1 },
  scrollContent: { padding: 16, paddingBottom: 40 },
  section: { marginBottom: 24 },
  sectionHeader: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 },
  sectionTitle: { fontSize: 15, fontWeight: '600', color: THEME.text },
  tagsContainer: { flexDirection: 'row', flexWrap: 'wrap', gap: 10 },
  historyTag: { flexDirection: 'row', alignItems: 'center', gap: 4, paddingHorizontal: 12, paddingVertical: 6, backgroundColor: THEME.card, borderRadius: 16, ...SHADOWS.small },
  historyTagText: { fontSize: 13, color: THEME.textSecondary },
  hotTag: { flexDirection: 'row', alignItems: 'center', gap: 4, paddingHorizontal: 12, paddingVertical: 6, backgroundColor: THEME.warning + '10', borderRadius: 16 },
  hotTagText: { fontSize: 13, color: THEME.warning },
  loadingBox: { paddingVertical: 60, alignItems: 'center' },
  emptyBox: { paddingVertical: 60, alignItems: 'center', gap: 8 },
  emptyText: { fontSize: 15, color: THEME.textSecondary },
  resultCount: { fontSize: 13, color: THEME.textTertiary, marginBottom: 12 },
});
```

### 提供者注册页面

```tsx
// frontend/app/market/provider/register.tsx

import React, { useState } from 'react';
import {
  View, Text, StyleSheet, ScrollView, Pressable, TextInput, Alert, ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { useMarketApi } from '@/divination/market/hooks/useMarketApi';
import { DIVINATION_TYPES, SPECIALTIES } from '@/divination/market/constants/market.constants';

export default function ProviderRegisterPage() {
  const router = useRouter();
  const { registerProvider, loading } = useMarketApi();

  const [name, setName] = useState('');
  const [bio, setBio] = useState('');
  const [selectedTypes, setSelectedTypes] = useState<number[]>([]);
  const [selectedSpecialties, setSelectedSpecialties] = useState<number[]>([]);
  const [acceptsUrgent, setAcceptsUrgent] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  const toggleType = (typeId: number) => {
    setSelectedTypes((prev) =>
      prev.includes(typeId) ? prev.filter((id) => id !== typeId) : [...prev, typeId]
    );
  };

  const toggleSpecialty = (bit: number) => {
    setSelectedSpecialties((prev) =>
      prev.includes(bit) ? prev.filter((b) => b !== bit) : [...prev, bit]
    );
  };

  const calculateBitmap = (bits: number[]): number => {
    return bits.reduce((acc, bit) => acc | (1 << bit), 0);
  };

  const handleSubmit = async () => {
    if (!name.trim()) { Alert.alert('提示', '请输入显示名称'); return; }
    if (name.length < 2 || name.length > 20) { Alert.alert('提示', '名称长度需在2-20个字符之间'); return; }
    if (!bio.trim()) { Alert.alert('提示', '请填写个人简介'); return; }
    if (bio.length < 20) { Alert.alert('提示', '个人简介至少20个字'); return; }
    if (selectedTypes.length === 0) { Alert.alert('提示', '请选择至少一种擅长的占卜类型'); return; }
    if (selectedSpecialties.length === 0) { Alert.alert('提示', '请选择至少一个擅长领域'); return; }

    Alert.alert('确认注册', '注册成为服务提供者后，您需要设置服务套餐才能开始接单。确认继续？', [
      { text: '取消', style: 'cancel' },
      {
        text: '确认',
        onPress: async () => {
          setSubmitting(true);
          try {
            await registerProvider({
              name: name.trim(),
              bio: bio.trim(),
              supportedTypes: calculateBitmap(selectedTypes),
              specialties: calculateBitmap(selectedSpecialties),
              acceptsUrgent,
            });
            Alert.alert('成功', '注册成功！请设置服务套餐', [
              { text: '去设置', onPress: () => router.replace('/market/provider/packages') },
            ]);
          } catch (error: any) {
            Alert.alert('错误', error.message || '注册失败');
          } finally {
            setSubmitting(false);
          }
        },
      },
    ]);
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color={THEME.text} />
        </Pressable>
        <Text style={styles.headerTitle}>注册成为提供者</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.scrollContent}>
        {/* 提示信息 */}
        <View style={styles.infoCard}>
          <Ionicons name="information-circle-outline" size={20} color={THEME.info} />
          <Text style={styles.infoText}>
            注册后您将成为平台认证的服务提供者，可以设置服务套餐并接受用户订单。
          </Text>
        </View>

        {/* 基本信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>基本信息</Text>
          <View style={styles.formItem}>
            <Text style={styles.label}>显示名称 <Text style={styles.required}>*</Text></Text>
            <TextInput
              style={styles.textInput}
              value={name}
              onChangeText={setName}
              placeholder="您的显示名称，如：张三大师"
              placeholderTextColor={THEME.textTertiary}
              maxLength={20}
            />
          </View>
          <View style={styles.formItem}>
            <Text style={styles.label}>个人简介 <Text style={styles.required}>*</Text></Text>
            <TextInput
              style={[styles.textInput, styles.textArea]}
              value={bio}
              onChangeText={setBio}
              placeholder="介绍您的从业经历、擅长方向、服务特色等..."
              placeholderTextColor={THEME.textTertiary}
              multiline
              maxLength={200}
            />
          </View>
        </View>

        {/* 擅长的占卜类型 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>擅长的占卜类型 <Text style={styles.required}>*</Text></Text>
          <View style={styles.tagsGrid}>
            {DIVINATION_TYPES.map((type) => (
              <Pressable
                key={type.id}
                style={[styles.typeTag, selectedTypes.includes(type.id) && styles.typeTagSelected]}
                onPress={() => toggleType(type.id)}
              >
                {selectedTypes.includes(type.id) && (
                  <Ionicons name="checkmark-circle" size={16} color={type.color} />
                )}
                <Text style={[styles.typeTagText, selectedTypes.includes(type.id) && { color: type.color }]}>
                  {type.name}
                </Text>
              </Pressable>
            ))}
          </View>
        </View>

        {/* 擅长领域 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>擅长领域 <Text style={styles.required}>*</Text></Text>
          <View style={styles.tagsGrid}>
            {SPECIALTIES.map((specialty) => (
              <Pressable
                key={specialty.bit}
                style={[styles.specialtyTag, selectedSpecialties.includes(specialty.bit) && styles.specialtyTagSelected]}
                onPress={() => toggleSpecialty(specialty.bit)}
              >
                <Ionicons
                  name={specialty.icon as any}
                  size={16}
                  color={selectedSpecialties.includes(specialty.bit) ? THEME.primary : THEME.textTertiary}
                />
                <Text style={[styles.specialtyTagText, selectedSpecialties.includes(specialty.bit) && styles.specialtyTagTextSelected]}>
                  {specialty.name}
                </Text>
              </Pressable>
            ))}
          </View>
        </View>

        <Text style={styles.agreementText}>注册即表示您同意《服务提供者协议》和《平台规则》</Text>
      </ScrollView>

      <View style={styles.footer}>
        <Pressable
          style={[styles.submitButton, submitting && styles.submitButtonDisabled]}
          onPress={handleSubmit}
          disabled={submitting}
        >
          {submitting ? (
            <ActivityIndicator color={THEME.textInverse} />
          ) : (
            <Text style={styles.submitButtonText}>提交注册</Text>
          )}
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: THEME.background, maxWidth: 414, width: '100%', alignSelf: 'center' },
  header: { flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', paddingTop: 50, paddingHorizontal: 16, paddingBottom: 12, backgroundColor: THEME.card, borderBottomWidth: 1, borderBottomColor: THEME.borderLight },
  backButton: { padding: 4 },
  headerTitle: { fontSize: 17, fontWeight: '600', color: THEME.text },
  placeholder: { width: 32 },
  content: { flex: 1 },
  scrollContent: { padding: 16, paddingBottom: 24 },
  infoCard: { flexDirection: 'row', alignItems: 'flex-start', gap: 10, backgroundColor: THEME.info + '10', borderRadius: 10, padding: 14, marginBottom: 20 },
  infoText: { flex: 1, fontSize: 14, color: THEME.info, lineHeight: 20 },
  section: { marginBottom: 24 },
  sectionTitle: { fontSize: 15, fontWeight: '600', color: THEME.text, marginBottom: 12 },
  required: { color: THEME.error },
  formItem: { marginBottom: 16 },
  label: { fontSize: 14, color: THEME.textSecondary, marginBottom: 8 },
  textInput: { backgroundColor: THEME.card, borderRadius: 8, paddingHorizontal: 14, paddingVertical: 12, fontSize: 15, color: THEME.text, ...SHADOWS.small },
  textArea: { minHeight: 100, textAlignVertical: 'top' },
  tagsGrid: { flexDirection: 'row', flexWrap: 'wrap', gap: 10 },
  typeTag: { flexDirection: 'row', alignItems: 'center', gap: 4, paddingHorizontal: 14, paddingVertical: 8, backgroundColor: THEME.card, borderRadius: 20, borderWidth: 1, borderColor: THEME.border, ...SHADOWS.small },
  typeTagSelected: { backgroundColor: THEME.primary + '10' },
  typeTagText: { fontSize: 14, color: THEME.textSecondary },
  specialtyTag: { flexDirection: 'row', alignItems: 'center', gap: 4, paddingHorizontal: 12, paddingVertical: 6, backgroundColor: THEME.card, borderRadius: 6, borderWidth: 1, borderColor: THEME.border },
  specialtyTagSelected: { borderColor: THEME.primary, backgroundColor: THEME.primary + '10' },
  specialtyTagText: { fontSize: 13, color: THEME.textSecondary },
  specialtyTagTextSelected: { color: THEME.primary },
  agreementText: { fontSize: 12, color: THEME.textTertiary, textAlign: 'center', marginTop: 8 },
  footer: { padding: 16, paddingBottom: 32, backgroundColor: THEME.card, borderTopWidth: 1, borderTopColor: THEME.borderLight },
  submitButton: { height: 50, backgroundColor: THEME.primary, borderRadius: 8, justifyContent: 'center', alignItems: 'center' },
  submitButtonDisabled: { opacity: 0.6 },
  submitButtonText: { fontSize: 16, fontWeight: '600', color: THEME.textInverse },
});
```

### 选择卦象页面

```tsx
// frontend/app/market/order/select-hexagram.tsx

import React, { useState, useEffect } from 'react';
import { View, Text, StyleSheet, ScrollView, Pressable, ActivityIndicator } from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { useDivinationHistory } from '@/hooks/useDivinationHistory';
import { formatDateTime } from '@/divination/market/utils/market.utils';

export default function SelectHexagramPage() {
  const router = useRouter();
  const params = useLocalSearchParams();
  const { divinationType } = params;
  const { getHistoryByType, loading } = useDivinationHistory();
  const [hexagrams, setHexagrams] = useState([]);

  useEffect(() => {
    loadHexagrams();
  }, [divinationType]);

  const loadHexagrams = async () => {
    const data = await getHistoryByType(Number(divinationType));
    setHexagrams(data);
  };

  const handleSelect = (hexagram: any) => {
    router.back();
    router.setParams({ selectedHexagram: JSON.stringify(hexagram) });
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color={THEME.text} />
        </Pressable>
        <Text style={styles.headerTitle}>选择卦象</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.scrollContent}>
        {loading ? (
          <View style={styles.loadingBox}>
            <ActivityIndicator size="large" color={THEME.primary} />
          </View>
        ) : hexagrams.length === 0 ? (
          <View style={styles.emptyBox}>
            <Ionicons name="albums-outline" size={48} color={THEME.textTertiary} />
            <Text style={styles.emptyText}>暂无卦象记录</Text>
            <Pressable style={styles.emptyButton} onPress={() => router.push('/divination/meihua')}>
              <Text style={styles.emptyButtonText}>去起卦</Text>
            </Pressable>
          </View>
        ) : (
          hexagrams.map((hexagram) => (
            <Pressable key={hexagram.id} style={styles.hexagramCard} onPress={() => handleSelect(hexagram)}>
              <View style={styles.hexagramHeader}>
                <Text style={styles.hexagramSymbol}>{hexagram.symbol || '☰'}</Text>
                <View style={styles.hexagramInfo}>
                  <Text style={styles.hexagramName}>{hexagram.name}</Text>
                  <Text style={styles.hexagramTime}>{formatDateTime(hexagram.createdAt)}</Text>
                </View>
                <Ionicons name="chevron-forward" size={20} color={THEME.textTertiary} />
              </View>
              {hexagram.question && (
                <Text style={styles.hexagramQuestion} numberOfLines={2}>问事：{hexagram.question}</Text>
              )}
            </Pressable>
          ))
        )}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: THEME.background, maxWidth: 414, width: '100%', alignSelf: 'center' },
  header: { flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', paddingTop: 50, paddingHorizontal: 16, paddingBottom: 12, backgroundColor: THEME.card, borderBottomWidth: 1, borderBottomColor: THEME.borderLight },
  backButton: { padding: 4 },
  headerTitle: { fontSize: 17, fontWeight: '600', color: THEME.text },
  placeholder: { width: 32 },
  content: { flex: 1 },
  scrollContent: { padding: 16, paddingBottom: 40 },
  loadingBox: { paddingVertical: 60, alignItems: 'center' },
  emptyBox: { paddingVertical: 60, alignItems: 'center', gap: 12 },
  emptyText: { fontSize: 15, color: THEME.textTertiary },
  emptyButton: { marginTop: 8, paddingHorizontal: 20, paddingVertical: 10, backgroundColor: THEME.primary, borderRadius: 6 },
  emptyButtonText: { fontSize: 14, fontWeight: '500', color: THEME.textInverse },
  hexagramCard: { backgroundColor: THEME.card, borderRadius: 10, padding: 14, marginBottom: 10, ...SHADOWS.small },
  hexagramHeader: { flexDirection: 'row', alignItems: 'center' },
  hexagramSymbol: { fontSize: 32, marginRight: 12 },
  hexagramInfo: { flex: 1 },
  hexagramName: { fontSize: 16, fontWeight: '600', color: THEME.text, marginBottom: 2 },
  hexagramTime: { fontSize: 13, color: THEME.textTertiary },
  hexagramQuestion: { fontSize: 14, color: THEME.textSecondary, marginTop: 10, paddingTop: 10, borderTopWidth: 1, borderTopColor: THEME.borderLight },
});
```

---

## 11. 错误处理和离线支持

### 错误边界组件

```tsx
// frontend/src/divination/market/components/ErrorBoundary.tsx

import React, { Component, ErrorInfo, ReactNode } from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../theme';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    this.props.onError?.(error, errorInfo);
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <View style={styles.container}>
          <Ionicons name="warning-outline" size={64} color={THEME.warning} />
          <Text style={styles.title}>出错了</Text>
          <Text style={styles.message}>{this.state.error?.message || '发生了未知错误'}</Text>
          <Pressable style={styles.retryButton} onPress={this.handleRetry}>
            <Ionicons name="refresh-outline" size={18} color={THEME.textInverse} />
            <Text style={styles.retryText}>重试</Text>
          </Pressable>
        </View>
      );
    }
    return this.props.children;
  }
}

const styles = StyleSheet.create({
  container: { flex: 1, justifyContent: 'center', alignItems: 'center', padding: 24, backgroundColor: THEME.background },
  title: { fontSize: 20, fontWeight: '600', color: THEME.text, marginTop: 16, marginBottom: 8 },
  message: { fontSize: 14, color: THEME.textSecondary, textAlign: 'center', marginBottom: 24 },
  retryButton: { flexDirection: 'row', alignItems: 'center', gap: 6, paddingHorizontal: 20, paddingVertical: 12, backgroundColor: THEME.primary, borderRadius: 8 },
  retryText: { fontSize: 15, fontWeight: '500', color: THEME.textInverse },
});
```

### 网络状态监控

```typescript
// frontend/src/divination/market/hooks/useNetworkStatus.ts

import { useState, useEffect } from 'react';
import NetInfo, { NetInfoState } from '@react-native-community/netinfo';

export interface NetworkStatus {
  isConnected: boolean;
  isInternetReachable: boolean | null;
  type: string;
}

export const useNetworkStatus = () => {
  const [status, setStatus] = useState<NetworkStatus>({
    isConnected: true,
    isInternetReachable: true,
    type: 'unknown',
  });

  useEffect(() => {
    const unsubscribe = NetInfo.addEventListener((state: NetInfoState) => {
      setStatus({
        isConnected: state.isConnected ?? false,
        isInternetReachable: state.isInternetReachable,
        type: state.type,
      });
    });

    NetInfo.fetch().then((state) => {
      setStatus({
        isConnected: state.isConnected ?? false,
        isInternetReachable: state.isInternetReachable,
        type: state.type,
      });
    });

    return () => unsubscribe();
  }, []);

  return status;
};
```

### 离线交易队列

```typescript
// frontend/src/divination/market/services/offline-queue.service.ts

import AsyncStorage from '@react-native-async-storage/async-storage';
import NetInfo from '@react-native-community/netinfo';

const QUEUE_STORAGE_KEY = 'stardust_offline_tx_queue';

export interface QueuedTransaction {
  id: string;
  type: 'createOrder' | 'submitAnswer' | 'submitReview' | 'submitFollowUp';
  data: any;
  createdAt: number;
  retryCount: number;
  maxRetries: number;
  status: 'pending' | 'processing' | 'failed';
  error?: string;
}

class OfflineQueueService {
  private queue: QueuedTransaction[] = [];
  private isProcessing = false;
  private networkSubscription: (() => void) | null = null;

  async init() {
    await this.loadQueue();
    this.networkSubscription = NetInfo.addEventListener((state) => {
      if (state.isConnected && state.isInternetReachable) {
        this.processQueue();
      }
    });
  }

  async destroy() {
    this.networkSubscription?.();
  }

  async enqueue(type: QueuedTransaction['type'], data: any, options?: { maxRetries?: number }): Promise<string> {
    const tx: QueuedTransaction = {
      id: `tx_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type,
      data,
      createdAt: Date.now(),
      retryCount: 0,
      maxRetries: options?.maxRetries ?? 3,
      status: 'pending',
    };

    this.queue.push(tx);
    await this.saveQueue();
    this.processQueue();

    return tx.id;
  }

  getQueueStatus(): { pending: number; processing: number; failed: number } {
    return {
      pending: this.queue.filter((tx) => tx.status === 'pending').length,
      processing: this.queue.filter((tx) => tx.status === 'processing').length,
      failed: this.queue.filter((tx) => tx.status === 'failed').length,
    };
  }

  async retryFailed(id: string): Promise<void> {
    const tx = this.queue.find((t) => t.id === id);
    if (tx && tx.status === 'failed') {
      tx.status = 'pending';
      tx.retryCount = 0;
      tx.error = undefined;
      await this.saveQueue();
      this.processQueue();
    }
  }

  async remove(id: string): Promise<void> {
    this.queue = this.queue.filter((tx) => tx.id !== id);
    await this.saveQueue();
  }

  private async processQueue() {
    if (this.isProcessing) return;
    const netInfo = await NetInfo.fetch();
    if (!netInfo.isConnected || !netInfo.isInternetReachable) return;

    this.isProcessing = true;

    try {
      const pendingTxs = this.queue.filter((tx) => tx.status === 'pending');

      for (const tx of pendingTxs) {
        try {
          tx.status = 'processing';
          await this.saveQueue();
          await this.executeTransaction(tx);
          this.queue = this.queue.filter((t) => t.id !== tx.id);
          await this.saveQueue();
        } catch (error: any) {
          tx.retryCount++;
          tx.error = error.message;
          tx.status = tx.retryCount >= tx.maxRetries ? 'failed' : 'pending';
          await this.saveQueue();
        }
      }
    } finally {
      this.isProcessing = false;
    }
  }

  private async executeTransaction(tx: QueuedTransaction): Promise<void> {
    const { marketApi } = await import('./market-api.service');

    switch (tx.type) {
      case 'createOrder': await marketApi.createOrder(tx.data); break;
      case 'submitAnswer': await marketApi.submitAnswer(tx.data.orderId, tx.data.answer); break;
      case 'submitReview': await marketApi.submitReview(tx.data); break;
      case 'submitFollowUp': await marketApi.submitFollowUp(tx.data.orderId, tx.data.question); break;
      default: throw new Error(`Unknown transaction type: ${tx.type}`);
    }
  }

  private async loadQueue() {
    try {
      const data = await AsyncStorage.getItem(QUEUE_STORAGE_KEY);
      if (data) this.queue = JSON.parse(data);
    } catch (error) {
      console.error('Failed to load offline queue:', error);
    }
  }

  private async saveQueue() {
    try {
      await AsyncStorage.setItem(QUEUE_STORAGE_KEY, JSON.stringify(this.queue));
    } catch (error) {
      console.error('Failed to save offline queue:', error);
    }
  }
}

export const offlineQueueService = new OfflineQueueService();
```

### 离线提示组件

```tsx
// frontend/src/divination/market/components/OfflineBanner.tsx

import React from 'react';
import { View, Text, StyleSheet, Animated } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../theme';
import { useNetworkStatus } from '../hooks/useNetworkStatus';

export const OfflineBanner: React.FC = () => {
  const { isConnected, isInternetReachable } = useNetworkStatus();
  const [visible, setVisible] = React.useState(false);
  const slideAnim = React.useRef(new Animated.Value(-60)).current;

  const isOffline = !isConnected || isInternetReachable === false;

  React.useEffect(() => {
    if (isOffline) {
      setVisible(true);
      Animated.spring(slideAnim, { toValue: 0, useNativeDriver: true, friction: 8 }).start();
    } else {
      Animated.timing(slideAnim, { toValue: -60, duration: 200, useNativeDriver: true }).start(() => setVisible(false));
    }
  }, [isOffline]);

  if (!visible) return null;

  return (
    <Animated.View style={[styles.container, { transform: [{ translateY: slideAnim }] }]}>
      <View style={styles.content}>
        <Ionicons name="cloud-offline-outline" size={18} color={THEME.textInverse} />
        <Text style={styles.text}>当前处于离线状态</Text>
      </View>
      <Text style={styles.hint}>部分功能可能无法使用，操作将在恢复网络后自动提交</Text>
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: { position: 'absolute', top: 0, left: 0, right: 0, backgroundColor: THEME.warning, paddingTop: 50, paddingHorizontal: 16, paddingBottom: 12, zIndex: 1000 },
  content: { flexDirection: 'row', alignItems: 'center', gap: 8 },
  text: { fontSize: 15, fontWeight: '500', color: THEME.textInverse },
  hint: { fontSize: 12, color: THEME.textInverse + 'CC', marginTop: 4 },
});
```

### 全局错误处理 Hook

```typescript
// frontend/src/divination/market/hooks/useErrorHandler.ts

import { useCallback } from 'react';
import { Alert } from 'react-native';
import { useNetworkStatus } from './useNetworkStatus';
import { offlineQueueService, QueuedTransaction } from '../services/offline-queue.service';

export enum ErrorType {
  NETWORK = 'NETWORK',
  CHAIN = 'CHAIN',
  IPFS = 'IPFS',
  VALIDATION = 'VALIDATION',
  AUTH = 'AUTH',
  UNKNOWN = 'UNKNOWN',
}

export interface AppError {
  type: ErrorType;
  message: string;
  code?: string;
  originalError?: Error;
  retryable?: boolean;
}

const ERROR_MESSAGES: Record<string, string> = {
  'InsufficientBalance': '余额不足，请充值后重试',
  'OrderNotFound': '订单不存在',
  'Unauthorized': '您没有权限执行此操作',
  'ProviderNotActive': '服务提供者暂停接单',
  'PackageNotFound': '套餐不存在或已下架',
  'network request failed': '网络连接失败，请检查网络设置',
  'timeout': '请求超时，请稍后重试',
};

export const useErrorHandler = () => {
  const { isConnected } = useNetworkStatus();

  const parseError = useCallback((error: any): AppError => {
    if (!isConnected || error.message?.toLowerCase().includes('network')) {
      return { type: ErrorType.NETWORK, message: '网络连接失败', originalError: error, retryable: true };
    }
    if (error.message?.includes('Module') || error.dispatchError) {
      const errorCode = error.dispatchError?.asModule?.error?.toString() || '';
      return { type: ErrorType.CHAIN, message: ERROR_MESSAGES[errorCode] || `交易失败: ${error.message}`, code: errorCode, originalError: error, retryable: false };
    }
    if (error.message?.toLowerCase().includes('ipfs')) {
      return { type: ErrorType.IPFS, message: 'IPFS 服务暂时不可用', originalError: error, retryable: true };
    }
    return { type: ErrorType.UNKNOWN, message: ERROR_MESSAGES[error.message] || error.message || '发生未知错误', originalError: error, retryable: false };
  }, [isConnected]);

  const handleError = useCallback((
    error: any,
    options?: {
      silent?: boolean;
      onRetry?: () => void;
      queueForOffline?: { type: QueuedTransaction['type']; data: any };
    }
  ): AppError => {
    const appError = parseError(error);
    if (options?.silent) { console.error('Silent error:', appError); return appError; }

    if (appError.type === ErrorType.NETWORK && options?.queueForOffline) {
      Alert.alert('网络不可用', '操作已保存，将在网络恢复后自动提交', [{ text: '确定' }]);
      offlineQueueService.enqueue(options.queueForOffline.type, options.queueForOffline.data);
      return appError;
    }

    if (appError.retryable && options?.onRetry) {
      Alert.alert('操作失败', appError.message, [
        { text: '取消', style: 'cancel' },
        { text: '重试', onPress: options.onRetry },
      ]);
    } else {
      Alert.alert('错误', appError.message);
    }

    return appError;
  }, [parseError]);

  const withErrorHandling = useCallback(<T,>(
    fn: () => Promise<T>,
    options?: Parameters<typeof handleError>[1]
  ): Promise<T | null> => {
    return fn().catch((error) => { handleError(error, options); return null; });
  }, [handleError]);

  return { parseError, handleError, withErrorHandling };
};
```

### 页面包装器

```tsx
// frontend/src/divination/market/components/MarketPageWrapper.tsx

import React from 'react';
import { View, StyleSheet } from 'react-native';
import { ErrorBoundary } from './ErrorBoundary';
import { OfflineBanner } from './OfflineBanner';
import { THEME } from '../theme';

interface MarketPageWrapperProps {
  children: React.ReactNode;
  showOfflineBanner?: boolean;
}

export const MarketPageWrapper: React.FC<MarketPageWrapperProps> = ({
  children,
  showOfflineBanner = true,
}) => {
  return (
    <ErrorBoundary>
      <View style={styles.container}>
        {showOfflineBanner && <OfflineBanner />}
        {children}
      </View>
    </ErrorBoundary>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: THEME.background },
});
```

---

## 12. 隐私数据加密方案

### 12.1 数据分类与加密级别

```
┌─────────────────────────────────────────────────────────────────┐
│                        数据隐私分级                              │
├─────────────────────────────────────────────────────────────────┤
│  级别        │  数据类型              │  加密方式                │
├─────────────────────────────────────────────────────────────────┤
│  L0-公开     │  提供者名称、简介      │  无加密，直接上链        │
│              │  套餐信息、价格        │                          │
│              │  评分统计              │                          │
├─────────────────────────────────────────────────────────────────┤
│  L1-半公开   │  评价内容（非匿名）    │  IPFS 存储，CID 上链     │
│              │  提供者回复            │  可选加密                │
├─────────────────────────────────────────────────────────────────┤
│  L2-私密     │  用户问题              │  E2E 加密 + IPFS         │
│              │  大师解读              │  仅订单双方可解密        │
│              │  追问对话              │                          │
├─────────────────────────────────────────────────────────────────┤
│  L3-敏感     │  用户出生信息          │  E2E 加密 + 本地缓存     │
│              │  卦象详细数据          │  支持擦除                │
│              │  语音/视频内容         │                          │
└─────────────────────────────────────────────────────────────────┘
```

### 12.2 端到端加密架构

```
┌─────────────────────────────────────────────────────────────────┐
│                     端到端加密数据流                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   用户 (Customer)                    提供者 (Provider)           │
│   ┌─────────────┐                    ┌─────────────┐             │
│   │ Ed25519     │                    │ Ed25519     │             │
│   │ 签名密钥对   │                    │ 签名密钥对   │             │
│   │ (钱包密钥)   │                    │ (钱包密钥)   │             │
│   └──────┬──────┘                    └──────┬──────┘             │
│          │                                   │                    │
│          ▼                                   ▼                    │
│   ┌─────────────┐                    ┌─────────────┐             │
│   │ X25519      │   ◄── 密钥交换 ──►  │ X25519      │             │
│   │ 加密密钥对   │   (ECDH)            │ 加密密钥对   │             │
│   └──────┬──────┘                    └──────┬──────┘             │
│          │                                   │                    │
│          └───────────┬───────────────────────┘                   │
│                      ▼                                            │
│               ┌─────────────┐                                     │
│               │ 共享密钥     │                                     │
│               │ (每订单唯一) │                                     │
│               └──────┬──────┘                                     │
│                      │                                            │
│                      ▼                                            │
│               ┌─────────────┐                                     │
│               │ AES-256-GCM │                                     │
│               │ 对称加密     │                                     │
│               └──────┬──────┘                                     │
│                      │                                            │
│                      ▼                                            │
│               ┌─────────────┐                                     │
│               │ 加密内容     │ ──► IPFS ──► CID 上链              │
│               └─────────────┘                                     │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

### 12.3 加密密钥派生服务

```typescript
// frontend/src/divination/market/services/e2e-encryption.service.ts

import * as ed from '@noble/ed25519';
import { x25519 } from '@noble/curves/ed25519';
import * as SecureStore from 'expo-secure-store';
import { Buffer } from 'buffer';

const KEY_PREFIX = 'stardust_e2e_';

// 订单加密上下文
interface OrderEncryptionContext {
  orderId: number;
  sharedSecret: Uint8Array;
  aesKey: CryptoKey;
  createdAt: number;
}

export class E2EEncryptionService {
  private keyCache: Map<number, OrderEncryptionContext> = new Map();

  /**
   * 从 Ed25519 签名密钥派生 X25519 加密密钥对
   * 这样用户无需管理额外的密钥
   */
  async deriveEncryptionKeyPair(ed25519PrivateKey: Uint8Array): Promise<{
    publicKey: Uint8Array;
    privateKey: Uint8Array;
  }> {
    // Ed25519 私钥的前 32 字节可以用作 X25519 私钥种子
    const x25519PrivateKey = ed25519PrivateKey.slice(0, 32);
    const x25519PublicKey = x25519.getPublicKey(x25519PrivateKey);

    return {
      publicKey: x25519PublicKey,
      privateKey: x25519PrivateKey,
    };
  }

  /**
   * 为订单创建端到端加密上下文
   * @param orderId 订单ID
   * @param myPrivateKey 我的 X25519 私钥
   * @param theirPublicKey 对方的 X25519 公钥
   */
  async createOrderContext(
    orderId: number,
    myPrivateKey: Uint8Array,
    theirPublicKey: Uint8Array
  ): Promise<OrderEncryptionContext> {
    // 使用 X25519 进行密钥交换，派生共享密钥
    const sharedSecret = x25519.getSharedSecret(myPrivateKey, theirPublicKey);

    // 添加订单特定的盐，确保每个订单的密钥唯一
    const orderSalt = new TextEncoder().encode(`stardust_order_${orderId}`);
    const keyMaterial = new Uint8Array([...sharedSecret, ...orderSalt]);

    // 使用 HKDF 派生 AES 密钥
    const hkdfKey = await crypto.subtle.importKey(
      'raw',
      keyMaterial,
      'HKDF',
      false,
      ['deriveBits', 'deriveKey']
    );

    const aesKey = await crypto.subtle.deriveKey(
      {
        name: 'HKDF',
        hash: 'SHA-256',
        salt: orderSalt,
        info: new TextEncoder().encode('order-e2e-encryption'),
      },
      hkdfKey,
      { name: 'AES-GCM', length: 256 },
      true,
      ['encrypt', 'decrypt']
    );

    const context: OrderEncryptionContext = {
      orderId,
      sharedSecret,
      aesKey,
      createdAt: Date.now(),
    };

    // 缓存上下文
    this.keyCache.set(orderId, context);

    // 持久化存储密钥（用于应用重启后恢复）
    await this.persistContext(context);

    return context;
  }

  /**
   * 获取订单的加密上下文
   */
  async getOrderContext(orderId: number): Promise<OrderEncryptionContext | null> {
    // 先查缓存
    if (this.keyCache.has(orderId)) {
      return this.keyCache.get(orderId)!;
    }

    // 从持久化存储恢复
    return this.loadContext(orderId);
  }

  /**
   * 加密消息（问题/解读/追问）
   */
  async encryptMessage(
    orderId: number,
    plaintext: string,
    messageType: 'question' | 'answer' | 'followup'
  ): Promise<EncryptedPayload> {
    const context = await this.getOrderContext(orderId);
    if (!context) {
      throw new Error('Order encryption context not found');
    }

    // 构建消息结构
    const messageData = {
      type: messageType,
      content: plaintext,
      timestamp: Date.now(),
      nonce: crypto.getRandomValues(new Uint8Array(8)),
    };

    const encoder = new TextEncoder();
    const data = encoder.encode(JSON.stringify(messageData));

    // 生成随机 IV
    const iv = crypto.getRandomValues(new Uint8Array(12));

    // 加密
    const ciphertext = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      context.aesKey,
      data
    );

    // 组装加密载荷
    return {
      version: 1,
      algorithm: 'AES-256-GCM',
      iv: Buffer.from(iv).toString('base64'),
      ciphertext: Buffer.from(ciphertext).toString('base64'),
      authTag: 'included', // GCM 模式的认证标签包含在 ciphertext 中
    };
  }

  /**
   * 解密消息
   */
  async decryptMessage(
    orderId: number,
    payload: EncryptedPayload
  ): Promise<DecryptedMessage> {
    const context = await this.getOrderContext(orderId);
    if (!context) {
      throw new Error('Order encryption context not found');
    }

    if (payload.version !== 1 || payload.algorithm !== 'AES-256-GCM') {
      throw new Error('Unsupported encryption format');
    }

    const iv = Buffer.from(payload.iv, 'base64');
    const ciphertext = Buffer.from(payload.ciphertext, 'base64');

    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      context.aesKey,
      ciphertext
    );

    const decoder = new TextDecoder();
    const messageData = JSON.parse(decoder.decode(decrypted));

    return {
      type: messageData.type,
      content: messageData.content,
      timestamp: messageData.timestamp,
    };
  }

  /**
   * 持久化加密上下文
   */
  private async persistContext(context: OrderEncryptionContext): Promise<void> {
    const exportedKey = await crypto.subtle.exportKey('raw', context.aesKey);
    const stored = {
      orderId: context.orderId,
      aesKeyBase64: Buffer.from(exportedKey).toString('base64'),
      createdAt: context.createdAt,
    };
    await SecureStore.setItemAsync(
      `${KEY_PREFIX}order_${context.orderId}`,
      JSON.stringify(stored)
    );
  }

  /**
   * 从持久化存储加载上下文
   */
  private async loadContext(orderId: number): Promise<OrderEncryptionContext | null> {
    try {
      const stored = await SecureStore.getItemAsync(`${KEY_PREFIX}order_${orderId}`);
      if (!stored) return null;

      const data = JSON.parse(stored);
      const keyBuffer = Buffer.from(data.aesKeyBase64, 'base64');

      const aesKey = await crypto.subtle.importKey(
        'raw',
        keyBuffer,
        { name: 'AES-GCM', length: 256 },
        true,
        ['encrypt', 'decrypt']
      );

      const context: OrderEncryptionContext = {
        orderId: data.orderId,
        sharedSecret: new Uint8Array(), // 不存储共享密钥原文
        aesKey,
        createdAt: data.createdAt,
      };

      this.keyCache.set(orderId, context);
      return context;
    } catch {
      return null;
    }
  }

  /**
   * 删除订单的加密上下文（订单完成后清理）
   */
  async deleteOrderContext(orderId: number): Promise<void> {
    this.keyCache.delete(orderId);
    await SecureStore.deleteItemAsync(`${KEY_PREFIX}order_${orderId}`);
  }

  /**
   * 清理过期的加密上下文（超过90天）
   */
  async cleanupExpiredContexts(): Promise<void> {
    const EXPIRY_MS = 90 * 24 * 60 * 60 * 1000; // 90 days
    const now = Date.now();

    for (const [orderId, context] of this.keyCache.entries()) {
      if (now - context.createdAt > EXPIRY_MS) {
        await this.deleteOrderContext(orderId);
      }
    }
  }
}

// 加密载荷格式
export interface EncryptedPayload {
  version: number;
  algorithm: string;
  iv: string;
  ciphertext: string;
  authTag: string;
}

// 解密后的消息
export interface DecryptedMessage {
  type: 'question' | 'answer' | 'followup';
  content: string;
  timestamp: number;
}

export const e2eEncryptionService = new E2EEncryptionService();
```

### 12.4 公钥注册与交换机制

```typescript
// frontend/src/divination/market/services/key-exchange.service.ts

import { useWalletStore } from '@/stores/wallet.store';
import { useApiConnection } from '@/api';
import { e2eEncryptionService } from './e2e-encryption.service';
import { Buffer } from 'buffer';

/**
 * 公钥交换服务
 *
 * 流程:
 * 1. 用户/提供者注册时，从签名密钥派生加密公钥并上链
 * 2. 创建订单时，用户获取提供者公钥，建立加密上下文
 * 3. 提供者接单时，获取用户公钥，建立加密上下文
 */
export class KeyExchangeService {
  /**
   * 注册加密公钥到链上
   * 在用户首次使用或提供者注册时调用
   */
  async registerEncryptionPublicKey(): Promise<string> {
    const { api } = useApiConnection.getState();
    const { activeAccount, signAndSend } = useWalletStore.getState();

    if (!activeAccount) {
      throw new Error('No active account');
    }

    // 获取签名密钥的私钥部分
    const privateKeyBytes = await this.getSigningPrivateKey();

    // 派生加密密钥对
    const { publicKey } = await e2eEncryptionService.deriveEncryptionKeyPair(
      privateKeyBytes
    );

    // 将公钥编码为 Base64 存储
    const publicKeyBase64 = Buffer.from(publicKey).toString('base64');

    // 调用链上方法注册公钥
    const tx = api.tx.divinationMarket.registerEncryptionKey(publicKeyBase64);
    await signAndSend(tx);

    return publicKeyBase64;
  }

  /**
   * 获取某个账户的加密公钥
   */
  async getEncryptionPublicKey(account: string): Promise<Uint8Array | null> {
    const { api } = useApiConnection.getState();

    const keyData = await api.query.divinationMarket.encryptionKeys(account);
    if (keyData.isNone) {
      return null;
    }

    const publicKeyBase64 = keyData.unwrap().toString();
    return Buffer.from(publicKeyBase64, 'base64');
  }

  /**
   * 为订单建立加密通道
   * @param orderId 订单ID
   * @param counterpartyAccount 对方账户地址
   */
  async establishOrderEncryption(
    orderId: number,
    counterpartyAccount: string
  ): Promise<void> {
    // 获取对方公钥
    const theirPublicKey = await this.getEncryptionPublicKey(counterpartyAccount);
    if (!theirPublicKey) {
      throw new Error('Counterparty encryption key not found');
    }

    // 获取自己的私钥
    const privateKeyBytes = await this.getSigningPrivateKey();
    const { privateKey: myPrivateKey } = await e2eEncryptionService.deriveEncryptionKeyPair(
      privateKeyBytes
    );

    // 创建订单加密上下文
    await e2eEncryptionService.createOrderContext(
      orderId,
      myPrivateKey,
      theirPublicKey
    );
  }

  /**
   * 获取签名密钥的私钥部分
   * 需要根据钱包类型适配
   */
  private async getSigningPrivateKey(): Promise<Uint8Array> {
    const { activeAccount, keystore } = useWalletStore.getState();

    if (!activeAccount) {
      throw new Error('No active account');
    }

    // 从 keystore 获取私钥
    // 注意: 这里需要根据实际的 keystore 实现来适配
    const keypair = await keystore.getKeypair(activeAccount.address);
    if (!keypair) {
      throw new Error('Keypair not found');
    }

    // 返回私钥的前32字节（Ed25519 种子）
    return keypair.secretKey.slice(0, 32);
  }
}

export const keyExchangeService = new KeyExchangeService();
```

### 12.5 加密内容的 IPFS 上传

```typescript
// frontend/src/divination/market/services/encrypted-ipfs.service.ts

import { ipfsService } from './ipfs.service';
import { e2eEncryptionService, EncryptedPayload } from './e2e-encryption.service';

/**
 * 加密 IPFS 服务
 * 封装端到端加密与 IPFS 上传的集成
 */
export class EncryptedIPFSService {
  /**
   * 加密并上传问题内容
   */
  async uploadQuestion(orderId: number, question: string): Promise<string> {
    // 加密问题
    const encrypted = await e2eEncryptionService.encryptMessage(
      orderId,
      question,
      'question'
    );

    // 上传加密后的内容到 IPFS
    const payload = JSON.stringify(encrypted);
    const cid = await ipfsService.uploadText(payload);

    return cid;
  }

  /**
   * 加密并上传解读内容
   */
  async uploadAnswer(orderId: number, answer: string): Promise<string> {
    const encrypted = await e2eEncryptionService.encryptMessage(
      orderId,
      answer,
      'answer'
    );

    const payload = JSON.stringify(encrypted);
    const cid = await ipfsService.uploadText(payload);

    return cid;
  }

  /**
   * 加密并上传追问内容
   */
  async uploadFollowUp(orderId: number, content: string): Promise<string> {
    const encrypted = await e2eEncryptionService.encryptMessage(
      orderId,
      content,
      'followup'
    );

    const payload = JSON.stringify(encrypted);
    const cid = await ipfsService.uploadText(payload);

    return cid;
  }

  /**
   * 下载并解密内容
   */
  async downloadAndDecrypt(
    orderId: number,
    cid: string
  ): Promise<{ type: string; content: string; timestamp: number }> {
    // 从 IPFS 获取加密内容
    const payload = await ipfsService.fetchContent(cid);
    const encrypted: EncryptedPayload = JSON.parse(payload);

    // 解密
    const decrypted = await e2eEncryptionService.decryptMessage(orderId, encrypted);

    return decrypted;
  }

  /**
   * 上传加密的语音/视频文件
   */
  async uploadEncryptedMedia(
    orderId: number,
    filePath: string,
    onProgress?: (progress: number) => void
  ): Promise<string> {
    const context = await e2eEncryptionService.getOrderContext(orderId);
    if (!context) {
      throw new Error('Order encryption context not found');
    }

    // 使用订单密钥加密上传
    return ipfsService.uploadFile(filePath, {
      encrypt: true,
      encryptionKey: context.aesKey,
      onProgress,
    });
  }
}

export const encryptedIPFSService = new EncryptedIPFSService();
```

### 12.6 隐私评价系统

```typescript
// frontend/src/divination/market/services/private-review.service.ts

import { ipfsService } from './ipfs.service';

/**
 * 隐私评价服务
 *
 * 评价隐私策略:
 * 1. 评分数据: 公开上链（用于计算提供者信誉）
 * 2. 评价文字: 可选加密存储到 IPFS
 * 3. 匿名评价: 隐藏评价者身份
 */
export class PrivateReviewService {
  /**
   * 处理评价内容
   * @param content 评价文字
   * @param isAnonymous 是否匿名
   * @param encryptForProvider 是否仅对提供者可见
   */
  async processReviewContent(
    content: string,
    options: {
      isAnonymous: boolean;
      encryptForProvider?: boolean;
      providerPublicKey?: Uint8Array;
    }
  ): Promise<{ cid: string; isEncrypted: boolean }> {
    // 构建评价元数据
    const metadata = {
      content,
      anonymous: options.isAnonymous,
      timestamp: Date.now(),
    };

    // 如果需要对提供者加密（私密反馈）
    if (options.encryptForProvider && options.providerPublicKey) {
      const encryptedContent = await this.encryptForRecipient(
        JSON.stringify(metadata),
        options.providerPublicKey
      );

      const cid = await ipfsService.uploadText(
        JSON.stringify({ encrypted: true, payload: encryptedContent })
      );

      return { cid, isEncrypted: true };
    }

    // 普通上传（公开评价）
    const cid = await ipfsService.uploadText(JSON.stringify(metadata));
    return { cid, isEncrypted: false };
  }

  /**
   * 使用接收者公钥加密内容
   * 采用 ECIES (椭圆曲线集成加密方案)
   */
  private async encryptForRecipient(
    plaintext: string,
    recipientPublicKey: Uint8Array
  ): Promise<string> {
    // 生成临时密钥对
    const ephemeralKeyPair = await crypto.subtle.generateKey(
      { name: 'ECDH', namedCurve: 'P-256' },
      true,
      ['deriveBits']
    );

    // 导出临时公钥（将随密文一起发送）
    const ephemeralPublicKeyRaw = await crypto.subtle.exportKey(
      'raw',
      ephemeralKeyPair.publicKey
    );

    // 导入接收者公钥
    const recipientKey = await crypto.subtle.importKey(
      'raw',
      recipientPublicKey,
      { name: 'ECDH', namedCurve: 'P-256' },
      false,
      []
    );

    // 派生共享密钥
    const sharedBits = await crypto.subtle.deriveBits(
      { name: 'ECDH', public: recipientKey },
      ephemeralKeyPair.privateKey,
      256
    );

    // 从共享密钥派生 AES 密钥
    const aesKey = await crypto.subtle.importKey(
      'raw',
      sharedBits,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt']
    );

    // 加密内容
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encoder = new TextEncoder();
    const ciphertext = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      aesKey,
      encoder.encode(plaintext)
    );

    // 组装 ECIES 密文包
    const result = {
      ephemeralPublicKey: Buffer.from(ephemeralPublicKeyRaw).toString('base64'),
      iv: Buffer.from(iv).toString('base64'),
      ciphertext: Buffer.from(ciphertext).toString('base64'),
    };

    return JSON.stringify(result);
  }
}

export const privateReviewService = new PrivateReviewService();
```

### 12.7 敏感数据本地加密存储

```typescript
// frontend/src/divination/market/services/secure-local-storage.service.ts

import * as SecureStore from 'expo-secure-store';
import * as Crypto from 'expo-crypto';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { Buffer } from 'buffer';

const MASTER_KEY_ID = 'stardust_master_key';
const ENCRYPTED_STORAGE_PREFIX = 'stardust_encrypted_';

/**
 * 安全本地存储服务
 *
 * 用于存储敏感的本地数据:
 * - 用户出生信息
 * - 卦象详细数据
 * - 订单草稿
 */
export class SecureLocalStorageService {
  private masterKey: CryptoKey | null = null;

  /**
   * 初始化服务，生成或加载主密钥
   */
  async initialize(): Promise<void> {
    const existingKey = await SecureStore.getItemAsync(MASTER_KEY_ID);

    if (existingKey) {
      // 从 SecureStore 加载主密钥
      this.masterKey = await crypto.subtle.importKey(
        'raw',
        Buffer.from(existingKey, 'base64'),
        { name: 'AES-GCM', length: 256 },
        true,
        ['encrypt', 'decrypt']
      );
    } else {
      // 生成新的主密钥
      this.masterKey = await crypto.subtle.generateKey(
        { name: 'AES-GCM', length: 256 },
        true,
        ['encrypt', 'decrypt']
      );

      // 存储到 SecureStore（硬件级安全存储）
      const exportedKey = await crypto.subtle.exportKey('raw', this.masterKey);
      await SecureStore.setItemAsync(
        MASTER_KEY_ID,
        Buffer.from(exportedKey).toString('base64')
      );
    }
  }

  /**
   * 加密存储数据
   */
  async setSecureItem<T>(key: string, value: T): Promise<void> {
    if (!this.masterKey) {
      await this.initialize();
    }

    const plaintext = JSON.stringify(value);
    const encoder = new TextEncoder();
    const data = encoder.encode(plaintext);

    const iv = crypto.getRandomValues(new Uint8Array(12));
    const ciphertext = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      this.masterKey!,
      data
    );

    const stored = {
      iv: Buffer.from(iv).toString('base64'),
      data: Buffer.from(ciphertext).toString('base64'),
    };

    await AsyncStorage.setItem(
      `${ENCRYPTED_STORAGE_PREFIX}${key}`,
      JSON.stringify(stored)
    );
  }

  /**
   * 解密读取数据
   */
  async getSecureItem<T>(key: string): Promise<T | null> {
    if (!this.masterKey) {
      await this.initialize();
    }

    const stored = await AsyncStorage.getItem(`${ENCRYPTED_STORAGE_PREFIX}${key}`);
    if (!stored) return null;

    try {
      const { iv, data } = JSON.parse(stored);
      const ivBuffer = Buffer.from(iv, 'base64');
      const ciphertext = Buffer.from(data, 'base64');

      const plaintext = await crypto.subtle.decrypt(
        { name: 'AES-GCM', iv: ivBuffer },
        this.masterKey!,
        ciphertext
      );

      const decoder = new TextDecoder();
      return JSON.parse(decoder.decode(plaintext)) as T;
    } catch {
      return null;
    }
  }

  /**
   * 删除加密数据
   */
  async removeSecureItem(key: string): Promise<void> {
    await AsyncStorage.removeItem(`${ENCRYPTED_STORAGE_PREFIX}${key}`);
  }

  /**
   * 安全擦除所有加密数据
   * 用于用户注销或隐私清理
   */
  async secureWipeAll(): Promise<void> {
    const allKeys = await AsyncStorage.getAllKeys();
    const encryptedKeys = allKeys.filter((k) =>
      k.startsWith(ENCRYPTED_STORAGE_PREFIX)
    );

    // 删除所有加密数据
    await AsyncStorage.multiRemove(encryptedKeys);

    // 删除主密钥
    await SecureStore.deleteItemAsync(MASTER_KEY_ID);
    this.masterKey = null;
  }

  /**
   * 导出加密数据（用于备份）
   * 返回使用用户密码加密的备份包
   */
  async exportBackup(userPassword: string): Promise<string> {
    // 从密码派生密钥
    const salt = crypto.getRandomValues(new Uint8Array(16));
    const passwordKey = await this.deriveKeyFromPassword(userPassword, salt);

    // 导出主密钥
    const exportedMasterKey = await crypto.subtle.exportKey('raw', this.masterKey!);

    // 加密主密钥
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encryptedMasterKey = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      passwordKey,
      exportedMasterKey
    );

    // 收集所有加密数据
    const allKeys = await AsyncStorage.getAllKeys();
    const encryptedKeys = allKeys.filter((k) =>
      k.startsWith(ENCRYPTED_STORAGE_PREFIX)
    );

    const dataEntries: Record<string, string> = {};
    for (const key of encryptedKeys) {
      const value = await AsyncStorage.getItem(key);
      if (value) {
        dataEntries[key] = value;
      }
    }

    // 组装备份包
    const backup = {
      version: 1,
      salt: Buffer.from(salt).toString('base64'),
      iv: Buffer.from(iv).toString('base64'),
      encryptedMasterKey: Buffer.from(encryptedMasterKey).toString('base64'),
      data: dataEntries,
      createdAt: Date.now(),
    };

    return JSON.stringify(backup);
  }

  /**
   * 从备份恢复
   */
  async restoreFromBackup(backupJson: string, userPassword: string): Promise<void> {
    const backup = JSON.parse(backupJson);

    if (backup.version !== 1) {
      throw new Error('Unsupported backup version');
    }

    // 从密码派生密钥
    const salt = Buffer.from(backup.salt, 'base64');
    const passwordKey = await this.deriveKeyFromPassword(userPassword, salt);

    // 解密主密钥
    const iv = Buffer.from(backup.iv, 'base64');
    const encryptedMasterKey = Buffer.from(backup.encryptedMasterKey, 'base64');

    const decryptedMasterKey = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      passwordKey,
      encryptedMasterKey
    );

    // 导入主密钥
    this.masterKey = await crypto.subtle.importKey(
      'raw',
      decryptedMasterKey,
      { name: 'AES-GCM', length: 256 },
      true,
      ['encrypt', 'decrypt']
    );

    // 存储主密钥
    await SecureStore.setItemAsync(
      MASTER_KEY_ID,
      Buffer.from(decryptedMasterKey).toString('base64')
    );

    // 恢复数据
    for (const [key, value] of Object.entries(backup.data)) {
      await AsyncStorage.setItem(key, value as string);
    }
  }

  private async deriveKeyFromPassword(
    password: string,
    salt: Uint8Array
  ): Promise<CryptoKey> {
    const encoder = new TextEncoder();
    const passwordBuffer = encoder.encode(password);

    const baseKey = await crypto.subtle.importKey(
      'raw',
      passwordBuffer,
      'PBKDF2',
      false,
      ['deriveKey']
    );

    return crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt,
        iterations: 100000,
        hash: 'SHA-256',
      },
      baseKey,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    );
  }
}

export const secureLocalStorage = new SecureLocalStorageService();
```

### 12.8 隐私数据使用组件示例

```tsx
// frontend/src/divination/market/hooks/usePrivateOrder.ts

import { useState, useCallback } from 'react';
import { keyExchangeService } from '../services/key-exchange.service';
import { encryptedIPFSService } from '../services/encrypted-ipfs.service';
import { useMarketApi } from './useMarketApi';

/**
 * 隐私订单 Hook
 * 封装端到端加密的订单操作
 */
export const usePrivateOrder = () => {
  const { createOrder: _createOrder, submitAnswer: _submitAnswer } = useMarketApi();
  const [loading, setLoading] = useState(false);

  /**
   * 创建加密订单
   * 1. 建立与提供者的加密通道
   * 2. 加密问题并上传到 IPFS
   * 3. 提交订单交易
   */
  const createPrivateOrder = useCallback(
    async (params: {
      provider: string;
      divinationType: number;
      hexagramId: number;
      packageId: number;
      question: string;
      isUrgent: boolean;
    }) => {
      setLoading(true);
      try {
        // 1. 首先确保我们已经注册了加密公钥
        await keyExchangeService.registerEncryptionPublicKey();

        // 2. 创建临时订单ID用于加密（实际订单ID由链生成）
        const tempOrderId = Date.now();

        // 3. 建立与提供者的加密通道
        await keyExchangeService.establishOrderEncryption(
          tempOrderId,
          params.provider
        );

        // 4. 加密并上传问题
        const questionCid = await encryptedIPFSService.uploadQuestion(
          tempOrderId,
          params.question
        );

        // 5. 提交订单（链上记录加密后的 CID）
        const orderId = await _createOrder({
          ...params,
          questionCid,
          encryptionSessionId: tempOrderId, // 传递加密会话ID
        });

        // 6. 更新加密上下文到正式订单ID
        // (实际实现中需要将临时上下文迁移到正式订单ID)

        return orderId;
      } finally {
        setLoading(false);
      }
    },
    [_createOrder]
  );

  /**
   * 提交加密解读（提供者使用）
   */
  const submitPrivateAnswer = useCallback(
    async (orderId: number, answer: string) => {
      setLoading(true);
      try {
        // 加密并上传解读
        const answerCid = await encryptedIPFSService.uploadAnswer(orderId, answer);

        // 提交到链上
        await _submitAnswer(orderId, answerCid);
      } finally {
        setLoading(false);
      }
    },
    [_submitAnswer]
  );

  /**
   * 解密订单内容
   */
  const decryptOrderContent = useCallback(
    async (orderId: number, cid: string) => {
      return encryptedIPFSService.downloadAndDecrypt(orderId, cid);
    },
    []
  );

  return {
    loading,
    createPrivateOrder,
    submitPrivateAnswer,
    decryptOrderContent,
  };
};
```

### 12.9 隐私设置页面

```tsx
// frontend/app/market/privacy-settings.tsx

import React, { useState, useEffect } from 'react';
import {
  View, Text, StyleSheet, ScrollView, Pressable, Switch, Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { THEME, SHADOWS } from '@/divination/market/theme';
import { secureLocalStorage } from '@/divination/market/services/secure-local-storage.service';
import * as FileSystem from 'expo-file-system';
import * as Sharing from 'expo-sharing';

export default function PrivacySettingsPage() {
  const router = useRouter();
  const [encryptQuestions, setEncryptQuestions] = useState(true);
  const [anonymousReviews, setAnonymousReviews] = useState(false);
  const [autoDeleteHistory, setAutoDeleteHistory] = useState(false);

  const handleExportData = async () => {
    Alert.prompt(
      '导出数据',
      '请设置备份密码（用于加密您的数据）',
      async (password) => {
        if (!password || password.length < 8) {
          Alert.alert('错误', '密码长度至少8位');
          return;
        }

        try {
          const backup = await secureLocalStorage.exportBackup(password);
          const filePath = `${FileSystem.documentDirectory}stardust_backup_${Date.now()}.json`;
          await FileSystem.writeAsStringAsync(filePath, backup);

          if (await Sharing.isAvailableAsync()) {
            await Sharing.shareAsync(filePath);
          } else {
            Alert.alert('成功', `备份已保存到：${filePath}`);
          }
        } catch (error: any) {
          Alert.alert('错误', error.message);
        }
      },
      'secure-text'
    );
  };

  const handleWipeData = () => {
    Alert.alert(
      '安全擦除',
      '这将永久删除所有本地加密数据，包括：\n• 卦象历史\n• 订单草稿\n• 加密密钥\n\n此操作不可恢复！',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确认擦除',
          style: 'destructive',
          onPress: async () => {
            try {
              await secureLocalStorage.secureWipeAll();
              Alert.alert('成功', '所有数据已安全擦除');
            } catch (error: any) {
              Alert.alert('错误', error.message);
            }
          },
        },
      ]
    );
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color={THEME.text} />
        </Pressable>
        <Text style={styles.headerTitle}>隐私设置</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.scrollContent}>
        {/* 加密设置 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>加密设置</Text>

          <View style={styles.settingCard}>
            <View style={styles.settingRow}>
              <View style={styles.settingInfo}>
                <View style={styles.settingHeader}>
                  <Ionicons name="lock-closed-outline" size={18} color={THEME.primary} />
                  <Text style={styles.settingTitle}>问题内容加密</Text>
                </View>
                <Text style={styles.settingDesc}>
                  使用端到端加密保护您的咨询问题，仅您和服务提供者可以查看
                </Text>
              </View>
              <Switch
                value={encryptQuestions}
                onValueChange={setEncryptQuestions}
                trackColor={{ false: '#E8E8E8', true: THEME.primary + '60' }}
                thumbColor={encryptQuestions ? THEME.primary : '#FFF'}
              />
            </View>
          </View>

          <View style={styles.settingCard}>
            <View style={styles.settingRow}>
              <View style={styles.settingInfo}>
                <View style={styles.settingHeader}>
                  <Ionicons name="eye-off-outline" size={18} color={THEME.primary} />
                  <Text style={styles.settingTitle}>默认匿名评价</Text>
                </View>
                <Text style={styles.settingDesc}>
                  评价时默认隐藏您的用户名
                </Text>
              </View>
              <Switch
                value={anonymousReviews}
                onValueChange={setAnonymousReviews}
                trackColor={{ false: '#E8E8E8', true: THEME.primary + '60' }}
                thumbColor={anonymousReviews ? THEME.primary : '#FFF'}
              />
            </View>
          </View>
        </View>

        {/* 数据管理 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>数据管理</Text>

          <View style={styles.settingCard}>
            <View style={styles.settingRow}>
              <View style={styles.settingInfo}>
                <View style={styles.settingHeader}>
                  <Ionicons name="timer-outline" size={18} color={THEME.warning} />
                  <Text style={styles.settingTitle}>自动清理历史</Text>
                </View>
                <Text style={styles.settingDesc}>
                  90天后自动删除已完成订单的加密密钥
                </Text>
              </View>
              <Switch
                value={autoDeleteHistory}
                onValueChange={setAutoDeleteHistory}
                trackColor={{ false: '#E8E8E8', true: THEME.warning + '60' }}
                thumbColor={autoDeleteHistory ? THEME.warning : '#FFF'}
              />
            </View>
          </View>

          <Pressable style={styles.actionCard} onPress={handleExportData}>
            <View style={styles.actionIcon}>
              <Ionicons name="download-outline" size={22} color={THEME.info} />
            </View>
            <View style={styles.actionInfo}>
              <Text style={styles.actionTitle}>导出加密数据</Text>
              <Text style={styles.actionDesc}>备份您的本地加密数据</Text>
            </View>
            <Ionicons name="chevron-forward" size={20} color={THEME.textTertiary} />
          </Pressable>

          <Pressable style={[styles.actionCard, styles.dangerCard]} onPress={handleWipeData}>
            <View style={[styles.actionIcon, styles.dangerIcon]}>
              <Ionicons name="trash-outline" size={22} color={THEME.error} />
            </View>
            <View style={styles.actionInfo}>
              <Text style={[styles.actionTitle, { color: THEME.error }]}>安全擦除数据</Text>
              <Text style={styles.actionDesc}>永久删除所有本地加密数据</Text>
            </View>
            <Ionicons name="chevron-forward" size={20} color={THEME.textTertiary} />
          </Pressable>
        </View>

        {/* 加密说明 */}
        <View style={styles.infoBox}>
          <Ionicons name="shield-checkmark-outline" size={20} color={THEME.success} />
          <Text style={styles.infoText}>
            您的隐私数据使用 AES-256-GCM 加密算法保护。加密密钥存储在设备安全区域，
            即使应用数据被导出也无法解密。端到端加密确保只有您和服务提供者可以
            查看订单内容，平台无法访问。
          </Text>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: THEME.background, maxWidth: 414, width: '100%', alignSelf: 'center' },
  header: { flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', paddingTop: 50, paddingHorizontal: 16, paddingBottom: 12, backgroundColor: THEME.card, borderBottomWidth: 1, borderBottomColor: THEME.borderLight },
  backButton: { padding: 4 },
  headerTitle: { fontSize: 17, fontWeight: '600', color: THEME.text },
  placeholder: { width: 32 },
  content: { flex: 1 },
  scrollContent: { padding: 16, paddingBottom: 40 },
  section: { marginBottom: 24 },
  sectionTitle: { fontSize: 15, fontWeight: '600', color: THEME.text, marginBottom: 12 },
  settingCard: { backgroundColor: THEME.card, borderRadius: 10, padding: 14, marginBottom: 10, ...SHADOWS.small },
  settingRow: { flexDirection: 'row', alignItems: 'center' },
  settingInfo: { flex: 1, marginRight: 12 },
  settingHeader: { flexDirection: 'row', alignItems: 'center', gap: 8, marginBottom: 4 },
  settingTitle: { fontSize: 15, fontWeight: '500', color: THEME.text },
  settingDesc: { fontSize: 13, color: THEME.textTertiary, lineHeight: 18 },
  actionCard: { flexDirection: 'row', alignItems: 'center', backgroundColor: THEME.card, borderRadius: 10, padding: 14, marginBottom: 10, ...SHADOWS.small },
  dangerCard: { borderWidth: 1, borderColor: THEME.error + '30' },
  actionIcon: { width: 40, height: 40, borderRadius: 20, backgroundColor: THEME.info + '15', justifyContent: 'center', alignItems: 'center' },
  dangerIcon: { backgroundColor: THEME.error + '15' },
  actionInfo: { flex: 1, marginLeft: 12 },
  actionTitle: { fontSize: 15, fontWeight: '500', color: THEME.text },
  actionDesc: { fontSize: 13, color: THEME.textTertiary, marginTop: 2 },
  infoBox: { flexDirection: 'row', alignItems: 'flex-start', gap: 10, backgroundColor: THEME.success + '10', borderRadius: 10, padding: 14 },
  infoText: { flex: 1, fontSize: 13, color: THEME.success, lineHeight: 18 },
});
```

### 12.10 加密架构安全性说明

```
┌─────────────────────────────────────────────────────────────────┐
│                      安全性设计要点                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. 密钥派生                                                     │
│     • 从钱包签名密钥 (Ed25519) 派生加密密钥 (X25519)            │
│     • 用户无需管理额外密钥                                       │
│     • 密钥与账户绑定，账户恢复后密钥自动恢复                     │
│                                                                  │
│  2. 前向保密                                                     │
│     • 每个订单使用独立的会话密钥                                 │
│     • 订单密钥 = HKDF(共享密钥 + 订单ID)                        │
│     • 单个订单密钥泄露不影响其他订单                             │
│                                                                  │
│  3. 密钥存储                                                     │
│     • 主密钥: SecureStore (硬件级安全)                           │
│     • 会话密钥: 内存缓存 + 加密持久化                            │
│     • 支持设备更换时的密钥迁移                                   │
│                                                                  │
│  4. 加密算法选择                                                 │
│     • X25519: 密钥交换（高性能，安全性高）                       │
│     • AES-256-GCM: 对称加密（认证加密，防篡改）                  │
│     • HKDF-SHA256: 密钥派生（标准安全）                          │
│                                                                  │
│  5. 攻击防护                                                     │
│     • 重放攻击: 每条消息包含时间戳和随机 nonce                   │
│     • 中间人攻击: 公钥上链，链下校验                             │
│     • 降级攻击: 强制使用 AES-256-GCM，不支持弱加密               │
│                                                                  │
│  6. 隐私保护                                                     │
│     • 链上仅存储 CID，内容不可见                                 │
│     • IPFS 内容加密，网关无法解密                                │
│     • 匿名评价隐藏用户身份                                       │
│                                                                  │
│  7. 数据生命周期                                                 │
│     • 订单完成后保留密钥90天（用于争议处理）                     │
│     • 用户可主动擦除历史数据                                     │
│     • 支持加密备份和恢复                                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 总结

本设计方案涵盖了 pallet-divination-market 的完整前端实现，包括：

1. **市场首页** - 服务提供者列表、筛选排序
2. **提供者详情** - 个人信息、套餐列表、用户评价
3. **创建订单** - 选择卦象、填写问题、费用计算
4. **订单详情** - 状态时间线、解读内容、追问功能
5. **提供者工作台** - 余额管理、订单处理、状态切换
6. **评价系统** - 多维度评分、匿名评价
7. **IPFS 集成** - 多节点上传、网关故障转移、大文件分片
8. **辅助页面** - 搜索、提供者注册、套餐管理、选择卦象
9. **错误处理** - 错误边界、全局错误处理 Hook
10. **离线支持** - 网络状态监控、离线交易队列、离线提示
11. **隐私加密** - 端到端加密、密钥管理、安全存储

所有组件遵循现有项目的设计风格，使用金棕色主题，保持一致的用户体验。
