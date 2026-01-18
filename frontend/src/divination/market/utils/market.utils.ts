// frontend/src/divination/market/utils/market.utils.ts

import { DivinationType, ServiceType, OrderStatus } from '../types/market.types';
import {
  DIVINATION_TYPES,
  SERVICE_TYPES,
  TIER_CONFIG,
  SPECIALTIES,
  ORDER_STATUS_CONFIG,
  TOKEN_DECIMALS,
} from '../constants/market.constants';

// 格式化余额（假设 12 位小数）
export const formatBalance = (amount: bigint | string | number): string => {
  const amountBigInt = typeof amount === 'bigint' ? amount : BigInt(amount || 0);
  const divisor = 10n ** BigInt(TOKEN_DECIMALS);
  const whole = amountBigInt / divisor;
  const fraction = amountBigInt % divisor;

  if (fraction === 0n) {
    return whole.toString();
  }

  const fractionStr = fraction.toString().padStart(TOKEN_DECIMALS, '0').replace(/0+$/, '');
  return `${whole}.${fractionStr.slice(0, 2)}`;
};

// 解析余额字符串为 bigint
export const parseBalance = (amount: string): bigint => {
  const [whole, fraction = ''] = amount.split('.');
  const fractionPadded = fraction.padEnd(TOKEN_DECIMALS, '0').slice(0, TOKEN_DECIMALS);
  return BigInt(whole || '0') * 10n ** BigInt(TOKEN_DECIMALS) + BigInt(fractionPadded);
};

// 占卜类型名称
export const getDivinationTypeName = (type: DivinationType | number): string => {
  const config = DIVINATION_TYPES.find((t) => t.id === type);
  return config?.name || '未知';
};

// 占卜类型路由
export const getDivinationTypeRoute = (type: DivinationType | number): string => {
  const config = DIVINATION_TYPES.find((t) => t.id === type);
  return config?.route || 'meihua';
};

// 占卜类型颜色
export const getDivinationTypeColor = (type: DivinationType | number): string => {
  const config = DIVINATION_TYPES.find((t) => t.id === type);
  return config?.color || '#999999';
};

// 从位图获取占卜类型名称列表
export const getDivinationTypeNames = (bitmap: number): string[] => {
  const names: string[] = [];
  for (let i = 0; i < DIVINATION_TYPES.length; i++) {
    if (bitmap & (1 << i)) {
      names.push(DIVINATION_TYPES[i].name);
    }
  }
  return names;
};

// 从位图获取占卜类型配置列表
export const getDivinationTypesFromBitmap = (bitmap: number) => {
  return DIVINATION_TYPES.filter((_, i) => bitmap & (1 << i));
};

// 服务类型名称
export const getServiceTypeName = (type: ServiceType | number): string => {
  const config = SERVICE_TYPES.find((t) => t.id === type);
  return config?.name || '未知';
};

// 服务类型图标
export const getServiceTypeIcon = (type: ServiceType | number): string => {
  const config = SERVICE_TYPES.find((t) => t.id === type);
  return config?.icon || 'help-outline';
};

// 等级名称
export const getTierName = (tier: number): string => {
  const config = TIER_CONFIG[tier];
  return config?.name || '新手';
};

// 等级颜色
export const getTierColor = (tier: number): string => {
  const config = TIER_CONFIG[tier];
  return config?.color || '#999999';
};

// 等级图标
export const getTierIcon = (tier: number): string => {
  const config = TIER_CONFIG[tier];
  return config?.icon || 'leaf-outline';
};

// 等级配置
export const getTierConfig = (tier: number) => {
  return TIER_CONFIG[tier] || TIER_CONFIG[0];
};

// 从位图获取擅长领域列表
export const getSpecialtiesFromBitmap = (bitmap: number) => {
  return SPECIALTIES.filter((s) => bitmap & (1 << s.bit));
};

// 从位图获取擅长领域名称列表
export const getSpecialtyNames = (bitmap: number): string[] => {
  return getSpecialtiesFromBitmap(bitmap).map((s) => s.name);
};

// 计算位图
export const calculateBitmap = (bits: number[]): number => {
  return bits.reduce((acc, bit) => acc | (1 << bit), 0);
};

// 订单状态名称
export const getOrderStatusName = (status: OrderStatus): string => {
  return ORDER_STATUS_CONFIG[status]?.name || '未知';
};

// 订单状态颜色
export const getOrderStatusColor = (status: OrderStatus): string => {
  return ORDER_STATUS_CONFIG[status]?.color || '#999999';
};

// 订单状态图标
export const getOrderStatusIcon = (status: OrderStatus): string => {
  return ORDER_STATUS_CONFIG[status]?.icon || 'help-outline';
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
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const mins = String(date.getMinutes()).padStart(2, '0');
  return `${year}-${month}-${day} ${hours}:${mins}`;
};

// 格式化日期
export const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
};

// 计算平均评分
export const calculateAverageRating = (totalRating: number, ratingCount: number): number => {
  if (ratingCount === 0) return 0;
  return totalRating / ratingCount / 100;
};

// 格式化评分显示
export const formatRating = (totalRating: number, ratingCount: number): string => {
  const avg = calculateAverageRating(totalRating, ratingCount);
  return avg.toFixed(1);
};

// 截断地址显示
export const truncateAddress = (address: string, start: number = 6, end: number = 4): string => {
  if (!address || address.length <= start + end) return address;
  return `${address.slice(0, start)}...${address.slice(-end)}`;
};

// 校验名称
export const validateName = (name: string): { valid: boolean; message?: string } => {
  if (!name.trim()) {
    return { valid: false, message: '名称不能为空' };
  }
  if (name.length < 2) {
    return { valid: false, message: '名称至少2个字符' };
  }
  if (name.length > 20) {
    return { valid: false, message: '名称不能超过20个字符' };
  }
  return { valid: true };
};

// 校验简介
export const validateBio = (bio: string): { valid: boolean; message?: string } => {
  if (!bio.trim()) {
    return { valid: false, message: '简介不能为空' };
  }
  if (bio.length < 20) {
    return { valid: false, message: '简介至少20个字符' };
  }
  if (bio.length > 200) {
    return { valid: false, message: '简介不能超过200个字符' };
  }
  return { valid: true };
};

// 校验问题
export const validateQuestion = (question: string): { valid: boolean; message?: string } => {
  if (!question.trim()) {
    return { valid: false, message: '问题描述不能为空' };
  }
  if (question.length < 10) {
    return { valid: false, message: '问题描述至少10个字符' };
  }
  if (question.length > 500) {
    return { valid: false, message: '问题描述不能超过500个字符' };
  }
  return { valid: true };
};
