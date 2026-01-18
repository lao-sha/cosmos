# Implementation Tasks: 成为占卜师

## Task 1: 创建类型定义
- [ ] 创建 `frontend/src/features/diviner/types.ts`
- [ ] 定义 ProviderStatus, ProviderTier, DivinationType, ServiceType, Specialty 枚举
- [ ] 定义 Provider, ServicePackage, Order, Review 接口

## Task 2: 创建核心组件
- [ ] 创建 `frontend/src/features/diviner/components/TierBadge.tsx` - 等级徽章
- [ ] 创建 `frontend/src/features/diviner/components/StatusBadge.tsx` - 状态徽章
- [ ] 创建 `frontend/src/features/diviner/components/SpecialtySelector.tsx` - 擅长领域选择器
- [ ] 创建 `frontend/src/features/diviner/components/DivinationTypeSelector.tsx` - 占卜类型选择器
- [ ] 创建 `frontend/src/features/diviner/components/PackageCard.tsx` - 套餐卡片
- [ ] 创建 `frontend/src/features/diviner/components/DivinerOrderCard.tsx` - 订单卡片
- [ ] 创建 `frontend/src/features/diviner/components/ReviewCard.tsx` - 评价卡片
- [ ] 创建 `frontend/src/features/diviner/components/DashboardStats.tsx` - 仪表盘统计
- [ ] 创建 `frontend/src/features/diviner/components/index.ts` - 组件导出

## Task 3: 创建页面
- [ ] 创建 `frontend/app/diviner/index.tsx` - 入口/注册引导页
- [ ] 创建 `frontend/app/diviner/register.tsx` - 注册表单页
- [ ] 创建 `frontend/app/diviner/dashboard.tsx` - 仪表盘
- [ ] 创建 `frontend/app/diviner/packages/index.tsx` - 套餐列表
- [ ] 创建 `frontend/app/diviner/packages/create.tsx` - 创建套餐
- [ ] 创建 `frontend/app/diviner/orders/index.tsx` - 订单列表
- [ ] 创建 `frontend/app/diviner/orders/[id].tsx` - 订单详情
- [ ] 创建 `frontend/app/diviner/reviews.tsx` - 评价管理
- [ ] 创建 `frontend/app/diviner/earnings.tsx` - 收益管理
- [ ] 创建 `frontend/app/diviner/profile.tsx` - 资料编辑
- [ ] 创建 `frontend/app/diviner/[providerId].tsx` - 公开资料页

## Task 4: 创建 Store 和 Service
- [ ] 创建 `frontend/src/stores/diviner.store.ts` - Zustand store
- [ ] 创建 `frontend/src/services/diviner.service.ts` - API service

## Task 5: 连接入口
- [ ] 在 profile 页面添加"成为占卜师"入口
