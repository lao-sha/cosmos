// ShareMall 服务层 - 与链上交互

import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';
import type {
  Shop,
  ShopFundInfo,
  Product,
  MallOrder,
  ShopTokenConfig,
  MemberInfo,
  TradeOrder,
  OrderBookDepth,
  MarketSummary,
  TwapInfo,
  PriceLevel,
  Review,
  Proposal,
} from '@/src/types/sharemall';

type AnyCodec = any;

export class ShareMallService {
  private api: ApiPromise | null = null;

  private getApi(): ApiPromise {
    if (!this.api) {
      const { api } = useChainStore.getState();
      if (!api) {
        throw new Error('Chain not connected');
      }
      this.api = api;
    }
    return this.api;
  }

  // ==================== 店铺查询 ====================

  async getShop(shopId: number): Promise<Shop | null> {
    try {
      const api = this.getApi();
      const shop = await api.query.sharemallShop.shops(shopId) as AnyCodec;
      if (shop?.isSome) {
        const data = shop.unwrap();
        return {
          id: data.id.toNumber(),
          owner: data.owner.toString(),
          customerService: data.customerService?.toString(),
          name: new TextDecoder().decode(data.name.toU8a()),
          logoCid: data.logoCid?.toHuman(),
          descriptionCid: data.descriptionCid?.toHuman(),
          status: data.status.toString(),
          rating: data.rating.toNumber(),
          ratingCount: data.ratingCount.toNumber(),
          totalSales: data.totalSales.toString(),
          totalOrders: data.totalOrders.toNumber(),
          productCount: data.productCount.toNumber(),
          createdAt: data.createdAt.toNumber(),
        };
      }
      return null;
    } catch (error) {
      console.error('Failed to get shop:', error);
      return null;
    }
  }

  async getShopList(limit: number = 20): Promise<Shop[]> {
    try {
      const api = this.getApi();
      const entries = await api.query.sharemallShop.shops.entries();
      const shops: Shop[] = [];
      
      for (const [, value] of entries.slice(0, limit)) {
        const data = (value as AnyCodec).unwrap();
        if (data.status.toString() === 'Active') {
          shops.push({
            id: data.id.toNumber(),
            owner: data.owner.toString(),
            name: new TextDecoder().decode(data.name.toU8a()),
            logoCid: data.logoCid?.toHuman(),
            status: data.status.toString(),
            rating: data.rating.toNumber(),
            ratingCount: data.ratingCount.toNumber(),
            totalSales: data.totalSales.toString(),
            totalOrders: data.totalOrders.toNumber(),
            productCount: data.productCount.toNumber(),
            createdAt: data.createdAt.toNumber(),
          });
        }
      }
      return shops.sort((a, b) => b.rating - a.rating);
    } catch (error) {
      console.error('Failed to get shop list:', error);
      return [];
    }
  }

  async getShopFundInfo(shopId: number): Promise<ShopFundInfo | null> {
    try {
      const api = this.getApi();
      const balance = await api.query.sharemallShop.shopFundBalance(shopId) as AnyCodec;
      const health = await api.query.sharemallShop.fundHealth(shopId) as AnyCodec;
      
      return {
        balance: balance.toString(),
        health: health?.toString() || 'Healthy',
        warningThreshold: '200000000000', // 20 COS
        minBalance: '100000000000',       // 10 COS
      };
    } catch (error) {
      console.error('Failed to get shop fund info:', error);
      return null;
    }
  }

  // ==================== 商品查询 ====================

  async getProduct(productId: number): Promise<Product | null> {
    try {
      const api = this.getApi();
      const product = await api.query.sharemallProduct.products(productId) as AnyCodec;
      if (product?.isSome) {
        const data = product.unwrap();
        return {
          id: data.id.toNumber(),
          shopId: data.shopId.toNumber(),
          nameCid: data.nameCid.toHuman(),
          imagesCid: data.imagesCid.toHuman(),
          detailCid: data.detailCid.toHuman(),
          price: data.price.toString(),
          stock: data.stock.toNumber(),
          soldCount: data.soldCount.toNumber(),
          status: data.status.toString(),
          category: data.category.toString(),
          createdAt: data.createdAt.toNumber(),
          updatedAt: data.updatedAt.toNumber(),
        };
      }
      return null;
    } catch (error) {
      console.error('Failed to get product:', error);
      return null;
    }
  }

  async getShopProducts(shopId: number): Promise<Product[]> {
    try {
      const api = this.getApi();
      const productIds = await api.query.sharemallProduct.shopProducts(shopId) as AnyCodec;
      const products: Product[] = [];
      
      for (const id of productIds) {
        const product = await this.getProduct(id.toNumber());
        if (product && product.status === 'OnSale') {
          products.push(product);
        }
      }
      return products;
    } catch (error) {
      console.error('Failed to get shop products:', error);
      return [];
    }
  }

  async getHotProducts(limit: number = 10): Promise<Product[]> {
    try {
      const api = this.getApi();
      const entries = await api.query.sharemallProduct.products.entries();
      const products: Product[] = [];
      
      for (const [, value] of entries) {
        if ((value as AnyCodec).isSome) {
          const data = (value as AnyCodec).unwrap();
          if (data.status.toString() === 'OnSale') {
            products.push({
              id: data.id.toNumber(),
              shopId: data.shopId.toNumber(),
              nameCid: data.nameCid.toHuman(),
              imagesCid: data.imagesCid.toHuman(),
              detailCid: data.detailCid.toHuman(),
              price: data.price.toString(),
              stock: data.stock.toNumber(),
              soldCount: data.soldCount.toNumber(),
              status: data.status.toString(),
              category: data.category.toString(),
              createdAt: data.createdAt.toNumber(),
              updatedAt: data.updatedAt.toNumber(),
            });
          }
        }
      }
      return products.sort((a, b) => b.soldCount - a.soldCount).slice(0, limit);
    } catch (error) {
      console.error('Failed to get hot products:', error);
      return [];
    }
  }

  // ==================== 订单查询 ====================

  async getOrder(orderId: number): Promise<MallOrder | null> {
    try {
      const api = this.getApi();
      const order = await api.query.sharemallOrder.orders(orderId) as AnyCodec;
      if (order?.isSome) {
        const data = order.unwrap();
        return {
          id: data.id.toNumber(),
          shopId: data.shopId.toNumber(),
          productId: data.productId.toNumber(),
          buyer: data.buyer.toString(),
          seller: data.seller.toString(),
          quantity: data.quantity.toNumber(),
          unitPrice: data.unitPrice.toString(),
          totalAmount: data.totalAmount.toString(),
          platformFee: data.platformFee.toString(),
          productCategory: data.productCategory.toString(),
          requiresShipping: data.requiresShipping.isTrue,
          shippingCid: data.shippingCid?.toHuman(),
          trackingCid: data.trackingCid?.toHuman(),
          status: data.status.toString(),
          createdAt: data.createdAt.toNumber(),
          paidAt: data.paidAt?.toNumber(),
          shippedAt: data.shippedAt?.toNumber(),
          completedAt: data.completedAt?.toNumber(),
          escrowId: data.escrowId.toNumber(),
        };
      }
      return null;
    } catch (error) {
      console.error('Failed to get order:', error);
      return null;
    }
  }

  async getBuyerOrders(account: string): Promise<MallOrder[]> {
    try {
      const api = this.getApi();
      const orderIds = await api.query.sharemallOrder.buyerOrders(account) as AnyCodec;
      const orders: MallOrder[] = [];
      
      for (const id of orderIds) {
        const order = await this.getOrder(id.toNumber());
        if (order) {
          orders.push(order);
        }
      }
      return orders.sort((a, b) => b.createdAt - a.createdAt);
    } catch (error) {
      console.error('Failed to get buyer orders:', error);
      return [];
    }
  }

  async getShopOrders(shopId: number): Promise<MallOrder[]> {
    try {
      const api = this.getApi();
      const orderIds = await api.query.sharemallOrder.shopOrders(shopId) as AnyCodec;
      const orders: MallOrder[] = [];
      
      for (const id of orderIds) {
        const order = await this.getOrder(id.toNumber());
        if (order) {
          orders.push(order);
        }
      }
      return orders.sort((a, b) => b.createdAt - a.createdAt);
    } catch (error) {
      console.error('Failed to get shop orders:', error);
      return [];
    }
  }

  // ==================== 代币查询 ====================

  async getTokenConfig(shopId: number): Promise<ShopTokenConfig | null> {
    try {
      const api = this.getApi();
      const config = await api.query.sharemallToken.shopTokenConfigs(shopId) as AnyCodec;
      if (config?.isSome) {
        const data = config.unwrap();
        return {
          enabled: data.enabled.isTrue,
          rewardRate: data.rewardRate.toNumber(),
          exchangeRate: data.exchangeRate.toNumber(),
          minRedeem: data.minRedeem.toString(),
          maxRedeemPerOrder: data.maxRedeemPerOrder.toString(),
          transferable: data.transferable.isTrue,
          createdAt: data.createdAt.toNumber(),
        };
      }
      return null;
    } catch (error) {
      console.error('Failed to get token config:', error);
      return null;
    }
  }

  async getTokenBalance(shopId: number, account: string): Promise<string> {
    try {
      const api = this.getApi();
      const balance = await api.query.sharemallToken.tokenBalances([shopId, account]) as AnyCodec;
      return balance.toString();
    } catch (error) {
      console.error('Failed to get token balance:', error);
      return '0';
    }
  }

  // ==================== 会员查询 ====================

  async getMemberInfo(shopId: number, account: string): Promise<MemberInfo | null> {
    try {
      const api = this.getApi();
      const member = await api.query.sharemallMember.members([shopId, account]) as AnyCodec;
      if (member?.isSome) {
        const data = member.unwrap();
        return {
          shopId,
          account,
          level: data.level.toString(),
          totalSpent: data.totalSpent.toString(),
          referrer: data.referrer?.toString(),
          referralCount: data.referralCount.toNumber(),
          totalCommission: data.totalCommission.toString(),
          joinedAt: data.joinedAt.toNumber(),
        };
      }
      return null;
    } catch (error) {
      console.error('Failed to get member info:', error);
      return null;
    }
  }

  // ==================== P2P 市场查询 ====================

  async getOrderBookDepth(shopId: number, depth: number = 10): Promise<OrderBookDepth> {
    try {
      const api = this.getApi();
      const result = await (api.rpc as any).sharemallMarket.getOrderBookDepth(shopId, depth);
      
      return {
        asks: result.asks.map((a: any) => ({
          price: a.price.toString(),
          totalAmount: a.totalAmount.toString(),
          orderCount: a.orderCount.toNumber(),
        })),
        bids: result.bids.map((b: any) => ({
          price: b.price.toString(),
          totalAmount: b.totalAmount.toString(),
          orderCount: b.orderCount.toNumber(),
        })),
      };
    } catch (error) {
      console.error('Failed to get order book:', error);
      return { asks: [], bids: [] };
    }
  }

  async getMarketSummary(shopId: number): Promise<MarketSummary | null> {
    try {
      const api = this.getApi();
      const summary = await api.query.sharemallMarket.marketSummaryStorage(shopId) as AnyCodec;
      if (summary?.isSome) {
        const data = summary.unwrap();
        return {
          lastPrice: data.lastPrice.toString(),
          high24h: data.high24h.toString(),
          low24h: data.low24h.toString(),
          volume24h: data.volume24h.toString(),
          priceChange24h: data.priceChange24h.toString(),
        };
      }
      return null;
    } catch (error) {
      console.error('Failed to get market summary:', error);
      return null;
    }
  }

  async getBestPrices(shopId: number): Promise<{ bestAsk?: string; bestBid?: string }> {
    try {
      const api = this.getApi();
      const bestAsk = await api.query.sharemallMarket.bestAsk(shopId) as AnyCodec;
      const bestBid = await api.query.sharemallMarket.bestBid(shopId) as AnyCodec;
      
      return {
        bestAsk: bestAsk?.isSome ? bestAsk.unwrap().toString() : undefined,
        bestBid: bestBid?.isSome ? bestBid.unwrap().toString() : undefined,
      };
    } catch (error) {
      console.error('Failed to get best prices:', error);
      return {};
    }
  }

  async getUserTradeOrders(account: string): Promise<TradeOrder[]> {
    try {
      const api = this.getApi();
      const orderIds = await api.query.sharemallMarket.userOrders(account) as AnyCodec;
      const orders: TradeOrder[] = [];
      
      for (const id of orderIds) {
        const order = await api.query.sharemallMarket.orders(id) as AnyCodec;
        if (order?.isSome) {
          const data = order.unwrap();
          orders.push({
            orderId: data.orderId.toNumber(),
            shopId: data.shopId.toNumber(),
            maker: data.maker.toString(),
            side: data.side.toString(),
            orderType: data.orderType.toString(),
            channel: data.channel.toString(),
            tokenAmount: data.tokenAmount.toString(),
            filledAmount: data.filledAmount.toString(),
            price: data.price.toString(),
            status: data.status.toString(),
            createdAt: data.createdAt.toNumber(),
            expiresAt: data.expiresAt.toNumber(),
          });
        }
      }
      return orders;
    } catch (error) {
      console.error('Failed to get user trade orders:', error);
      return [];
    }
  }

  // ==================== 评价查询 ====================

  async getProductReviews(productId: number): Promise<Review[]> {
    try {
      const api = this.getApi();
      const reviews = await api.query.sharemallReview.productReviews(productId) as AnyCodec;
      return reviews.map((r: any) => ({
        orderId: r.orderId.toNumber(),
        reviewer: r.reviewer.toString(),
        rating: r.rating.toNumber(),
        contentCid: r.contentCid?.toHuman(),
        createdAt: r.createdAt.toNumber(),
      }));
    } catch (error) {
      console.error('Failed to get product reviews:', error);
      return [];
    }
  }
}

export const sharemallService = new ShareMallService();
