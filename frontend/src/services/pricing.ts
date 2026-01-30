import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';

export interface MarketStats {
  otcPrice: number;
  bridgePrice: number;
  weightedPrice: number;
  simpleAvgPrice: number;
  otcVolume: string;
  bridgeVolume: string;
  totalVolume: string;
  otcOrderCount: number;
  bridgeSwapCount: number;
}

export interface PriceAggregateData {
  totalCos: string;
  totalUsdt: string;
  orderCount: number;
}

export interface ExchangeRateData {
  cnyRate: number;
  updatedAt: number;
}

class PricingService {
  private getApi(): ApiPromise {
    const { api } = useChainStore.getState();
    if (!api) {
      throw new Error('Chain not connected');
    }
    return api;
  }

  async getMarketStats(): Promise<MarketStats | null> {
    try {
      const api = this.getApi();
      
      const [otcAgg, bridgeAgg, defaultPrice, coldStartExited] = await Promise.all([
        (api.query as any).tradingPricing.otcPriceAggregate(),
        (api.query as any).tradingPricing.bridgePriceAggregate(),
        (api.query as any).tradingPricing.defaultPrice(),
        (api.query as any).tradingPricing.coldStartExited(),
      ]);

      const otcTotalCos = BigInt(otcAgg.totalCos.toString());
      const otcTotalUsdt = BigInt(otcAgg.totalUsdt.toString());
      const bridgeTotalCos = BigInt(bridgeAgg.totalCos.toString());
      const bridgeTotalUsdt = BigInt(bridgeAgg.totalUsdt.toString());

      const otcPrice = otcTotalCos > 0n 
        ? Number((otcTotalUsdt * BigInt(1e12)) / otcTotalCos)
        : 0;
      const bridgePrice = bridgeTotalCos > 0n
        ? Number((bridgeTotalUsdt * BigInt(1e12)) / bridgeTotalCos)
        : 0;

      const totalCos = otcTotalCos + bridgeTotalCos;
      const totalUsdt = otcTotalUsdt + bridgeTotalUsdt;

      let weightedPrice = 0;
      let simpleAvgPrice = 0;

      if (!coldStartExited.isTrue) {
        weightedPrice = defaultPrice.toNumber();
        simpleAvgPrice = defaultPrice.toNumber();
      } else {
        if (totalCos > 0n) {
          weightedPrice = Number((totalUsdt * BigInt(1e12)) / totalCos);
        }
        if (otcPrice > 0 && bridgePrice > 0) {
          simpleAvgPrice = Math.floor((otcPrice + bridgePrice) / 2);
        } else if (otcPrice > 0) {
          simpleAvgPrice = otcPrice;
        } else if (bridgePrice > 0) {
          simpleAvgPrice = bridgePrice;
        } else {
          simpleAvgPrice = defaultPrice.toNumber();
        }
      }

      return {
        otcPrice,
        bridgePrice,
        weightedPrice,
        simpleAvgPrice,
        otcVolume: otcTotalCos.toString(),
        bridgeVolume: bridgeTotalCos.toString(),
        totalVolume: totalCos.toString(),
        otcOrderCount: otcAgg.orderCount.toNumber(),
        bridgeSwapCount: bridgeAgg.orderCount.toNumber(),
      };
    } catch (error) {
      console.error('获取市场统计失败:', error);
      return null;
    }
  }

  async getCosPrice(): Promise<number | null> {
    try {
      const api = this.getApi();
      
      const [otcAgg, bridgeAgg, defaultPrice, coldStartExited] = await Promise.all([
        (api.query as any).tradingPricing.otcPriceAggregate(),
        (api.query as any).tradingPricing.bridgePriceAggregate(),
        (api.query as any).tradingPricing.defaultPrice(),
        (api.query as any).tradingPricing.coldStartExited(),
      ]);

      if (!coldStartExited.isTrue) {
        return defaultPrice.toNumber();
      }

      const otcTotalCos = BigInt(otcAgg.totalCos.toString());
      const otcTotalUsdt = BigInt(otcAgg.totalUsdt.toString());
      const bridgeTotalCos = BigInt(bridgeAgg.totalCos.toString());
      const bridgeTotalUsdt = BigInt(bridgeAgg.totalUsdt.toString());

      const totalCos = otcTotalCos + bridgeTotalCos;
      const totalUsdt = otcTotalUsdt + bridgeTotalUsdt;

      if (totalCos > 0n) {
        return Number((totalUsdt * BigInt(1e12)) / totalCos);
      }

      return defaultPrice.toNumber();
    } catch (error) {
      console.error('获取 COS 价格失败:', error);
      return null;
    }
  }

  async getDefaultPrice(): Promise<number> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingPricing.defaultPrice();
      return result.toNumber();
    } catch (error) {
      console.error('获取默认价格失败:', error);
      return 1;
    }
  }

  async isColdStartExited(): Promise<boolean> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingPricing.coldStartExited();
      return result.isTrue;
    } catch (error) {
      console.error('获取冷启动状态失败:', error);
      return false;
    }
  }

  async getCnyUsdtRate(): Promise<ExchangeRateData | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).tradingPricing.cnyUsdtRate();
      return {
        cnyRate: result.cnyRate.toNumber(),
        updatedAt: result.updatedAt.toNumber(),
      };
    } catch (error) {
      console.error('获取 CNY/USDT 汇率失败:', error);
      return null;
    }
  }

  formatPrice(priceUsdt: number): string {
    const price = priceUsdt / 1e6;
    if (price < 0.0001) {
      return `$${price.toExponential(2)}`;
    }
    return `$${price.toFixed(6)}`;
  }

  formatPriceSimple(priceUsdt: number): string {
    const price = priceUsdt / 1e6;
    if (price < 0.0001) {
      return `$${price.toFixed(6)}`;
    }
    return `$${price.toFixed(4)}`;
  }

  formatVolume(volumeCos: string): string {
    const volume = BigInt(volumeCos);
    const cos = Number(volume / BigInt(1e12));
    if (cos >= 1e9) {
      return `${(cos / 1e9).toFixed(2)}B COS`;
    }
    if (cos >= 1e6) {
      return `${(cos / 1e6).toFixed(2)}M COS`;
    }
    if (cos >= 1e3) {
      return `${(cos / 1e3).toFixed(2)}K COS`;
    }
    return `${cos.toFixed(2)} COS`;
  }

  usdtToCny(usdtAmount: number, cnyRate: number): number {
    return Math.floor((usdtAmount * cnyRate) / 1e6);
  }

  cnyToUsdt(cnyAmount: number, cnyRate: number): number {
    if (cnyRate === 0) return 0;
    return Math.floor((cnyAmount * 1e6) / cnyRate);
  }
}

export const pricingService = new PricingService();
