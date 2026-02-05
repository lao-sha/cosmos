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
    supreme: ['#FFD700', '#FF6B6B'],
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

  // 推荐层级
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
    match: ['#EC4899', '#F59E0B'],
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

export const Gradients = {
  primary: ['#6366F1', '#8B5CF6'],
  gold: ['#F59E0B', '#FBBF24'],
  success: ['#10B981', '#34D399'],
  love: ['#EC4899', '#F59E0B'],
};

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

export type MembershipLevel = keyof typeof Colors.membership;
export type KycStatus = keyof typeof Colors.kyc;
export type DisputeStatus = keyof typeof Colors.dispute;
export type GovernanceStatus = keyof typeof Colors.governance;
