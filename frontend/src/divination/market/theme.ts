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
