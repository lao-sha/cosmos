// ShareMall 类型定义

// ==================== 店铺相关 ====================

export type ShopStatus = 'Pending' | 'Active' | 'Suspended' | 'Banned' | 'Closed';

export interface Shop {
  id: number;
  owner: string;
  customerService?: string;
  name: string;
  logoCid?: string;
  descriptionCid?: string;
  status: ShopStatus;
  rating: number;
  ratingCount: number;
  totalSales: string;
  totalOrders: number;
  productCount: number;
  createdAt: number;
}

export interface ShopFundInfo {
  balance: string;
  health: 'Healthy' | 'Warning' | 'Critical' | 'Depleted';
  warningThreshold: string;
  minBalance: string;
}

// ==================== 商品相关 ====================

export type ProductStatus = 'Draft' | 'OnSale' | 'SoldOut' | 'OffShelf';
export type ProductCategory = 'Digital' | 'Physical' | 'Service' | 'Other';

export interface Product {
  id: number;
  shopId: number;
  nameCid: string;
  imagesCid: string;
  detailCid: string;
  price: string;
  stock: number;
  soldCount: number;
  status: ProductStatus;
  category: ProductCategory;
  createdAt: number;
  updatedAt: number;
}

export interface ProductDetail {
  name: string;
  description: string;
  images: string[];
}

// ==================== 订单相关 ====================

export type OrderStatus = 
  | 'Created' 
  | 'Paid' 
  | 'Shipped' 
  | 'Completed' 
  | 'Cancelled' 
  | 'Disputed' 
  | 'Refunded' 
  | 'Expired';

export interface MallOrder {
  id: number;
  shopId: number;
  productId: number;
  buyer: string;
  seller: string;
  quantity: number;
  unitPrice: string;
  totalAmount: string;
  platformFee: string;
  productCategory: ProductCategory;
  requiresShipping: boolean;
  shippingCid?: string;
  trackingCid?: string;
  status: OrderStatus;
  createdAt: number;
  paidAt?: number;
  shippedAt?: number;
  completedAt?: number;
  escrowId: number;
}

// ==================== 代币相关 ====================

export interface ShopTokenConfig {
  enabled: boolean;
  rewardRate: number;      // 基点
  exchangeRate: number;    // 基点
  minRedeem: string;
  maxRedeemPerOrder: string;
  transferable: boolean;
  createdAt: number;
}

// ==================== 会员相关 ====================

export type MemberLevel = 'Normal' | 'Silver' | 'Gold' | 'Platinum' | 'Diamond';

export interface MemberInfo {
  shopId: number;
  account: string;
  level: MemberLevel;
  totalSpent: string;
  referrer?: string;
  referralCount: number;
  totalCommission: string;
  joinedAt: number;
}

// ==================== P2P 交易市场 ====================

export type OrderSide = 'Buy' | 'Sell';
export type PaymentChannel = 'COS' | 'USDT';
export type TradeOrderStatus = 'Open' | 'PartiallyFilled' | 'Filled' | 'Cancelled' | 'Expired';

export interface TradeOrder {
  orderId: number;
  shopId: number;
  maker: string;
  side: OrderSide;
  orderType: 'Limit' | 'Market';
  channel: PaymentChannel;
  tokenAmount: string;
  filledAmount: string;
  price: string;
  usdtPrice?: number;
  tronAddress?: string;
  status: TradeOrderStatus;
  createdAt: number;
  expiresAt: number;
}

export interface PriceLevel {
  price: string;
  totalAmount: string;
  orderCount: number;
}

export interface OrderBookDepth {
  asks: PriceLevel[];  // 卖单
  bids: PriceLevel[];  // 买单
}

export interface MarketSummary {
  lastPrice: string;
  high24h: string;
  low24h: string;
  volume24h: string;
  priceChange24h: string;
}

export interface TwapInfo {
  oneHour?: string;
  oneDay?: string;
  oneWeek?: string;
}

// ==================== 治理相关 ====================

export type ProposalStatus = 
  | 'Created' 
  | 'Voting' 
  | 'Passed' 
  | 'Failed' 
  | 'Queued' 
  | 'Executed' 
  | 'Cancelled' 
  | 'Expired';

export interface Proposal {
  id: number;
  shopId: number;
  proposer: string;
  titleCid: string;
  descriptionCid: string;
  status: ProposalStatus;
  forVotes: string;
  againstVotes: string;
  abstainVotes: string;
  startBlock: number;
  endBlock: number;
  createdAt: number;
}

// ==================== 评价相关 ====================

export interface Review {
  orderId: number;
  reviewer: string;
  rating: number;  // 1-5
  contentCid?: string;
  createdAt: number;
}

// ==================== 购物车 ====================

export interface CartItem {
  productId: number;
  shopId: number;
  quantity: number;
  product?: Product;
  productDetail?: ProductDetail;
}

// ==================== 返佣相关 ====================

export interface CommissionRecord {
  id: number;
  shopId: number;
  recipient: string;
  amount: string;
  source: string;
  orderId: number;
  createdAt: number;
}
