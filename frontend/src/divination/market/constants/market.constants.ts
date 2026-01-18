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
  { level: 0, name: '新手', icon: 'leaf-outline', minOrders: 0, minRating: 0, feeRate: 2000, color: '#999999' },
  { level: 1, name: '认证', icon: 'checkmark-circle-outline', minOrders: 10, minRating: 350, feeRate: 1500, color: '#52C41A' },
  { level: 2, name: '资深', icon: 'ribbon-outline', minOrders: 50, minRating: 400, feeRate: 1200, color: '#1890FF' },
  { level: 3, name: '专家', icon: 'diamond-outline', minOrders: 200, minRating: 450, feeRate: 1000, color: '#722ED1' },
  { level: 4, name: '大师', icon: 'trophy-outline', minOrders: 500, minRating: 480, feeRate: 800, color: '#EB2F96' },
];

// 擅长领域配置
export const SPECIALTIES = [
  { bit: 0, name: '事业', icon: 'briefcase-outline', color: '#3498DB' },
  { bit: 1, name: '感情', icon: 'heart-outline', color: '#E91E63' },
  { bit: 2, name: '财运', icon: 'cash-outline', color: '#F39C12' },
  { bit: 3, name: '健康', icon: 'fitness-outline', color: '#2ECC71' },
  { bit: 4, name: '学业', icon: 'school-outline', color: '#9B59B6' },
  { bit: 5, name: '出行', icon: 'airplane-outline', color: '#1ABC9C' },
  { bit: 6, name: '官司', icon: 'hammer-outline', color: '#E74C3C' },
  { bit: 7, name: '寻物', icon: 'search-outline', color: '#34495E' },
  { bit: 8, name: '风水', icon: 'home-outline', color: '#8E44AD' },
  { bit: 9, name: '择日', icon: 'calendar-outline', color: '#D35400' },
];

// 订单状态配置
export const ORDER_STATUS_CONFIG = {
  PendingPayment: { name: '待支付', color: '#FAAD14', icon: 'card-outline' },
  Paid: { name: '已支付', color: '#1890FF', icon: 'checkmark-circle-outline' },
  Accepted: { name: '已接单', color: '#722ED1', icon: 'hand-left-outline' },
  Completed: { name: '已完成', color: '#52C41A', icon: 'document-text-outline' },
  Cancelled: { name: '已取消', color: '#999999', icon: 'close-circle-outline' },
  Reviewed: { name: '已评价', color: '#EB2F96', icon: 'star-outline' },
};

// 排序选项
export const SORT_OPTIONS = [
  { value: 'comprehensive', label: '综合排序' },
  { value: 'rating', label: '评分最高' },
  { value: 'orders', label: '订单最多' },
  { value: 'price', label: '价格最低' },
];

// 搜索历史相关
export const SEARCH_HISTORY_KEY = 'market_search_history';
export const MAX_HISTORY_ITEMS = 10;

// 热门搜索词
export const HOT_KEYWORDS = ['事业', '感情', '财运', '八字', '梅花', '塔罗'];

// Token 精度
export const TOKEN_DECIMALS = 12;
export const TOKEN_SYMBOL = 'DUST';
