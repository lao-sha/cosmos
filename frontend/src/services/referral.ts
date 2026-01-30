import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';

export interface ReferralInfo {
  sponsor: string | null;
  sponsorCode: string | null;
  myCode: string | null;
  downlines: string[];
  downlineCount: number;
  referralChain: string[];
}

class ReferralService {
  private getApi(): ApiPromise {
    const { api } = useChainStore.getState();
    if (!api) {
      throw new Error('Chain not connected');
    }
    return api;
  }

  async getSponsor(address: string): Promise<string | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).referral.sponsors(address);
      if (result.isSome) {
        return result.unwrap().toString();
      }
      return null;
    } catch (error) {
      console.error('获取上线失败:', error);
      return null;
    }
  }

  async getMyCode(address: string): Promise<string | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).referral.codeByAccount(address);
      if (result.isSome) {
        const codeBytes = result.unwrap();
        return new TextDecoder().decode(new Uint8Array(codeBytes));
      }
      return null;
    } catch (error) {
      console.error('获取推荐码失败:', error);
      return null;
    }
  }

  async getDownlines(address: string): Promise<string[]> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).referral.downlines(address);
      if (result && result.length > 0) {
        return result.map((item: any) => item.toString());
      }
      return [];
    } catch (error) {
      console.error('获取下线列表失败:', error);
      return [];
    }
  }

  async getReferralChain(address: string): Promise<string[]> {
    try {
      const chain: string[] = [];
      let current = address;
      
      for (let i = 0; i < 15; i++) {
        const sponsor = await this.getSponsor(current);
        if (sponsor) {
          chain.push(sponsor);
          current = sponsor;
        } else {
          break;
        }
      }
      
      return chain;
    } catch (error) {
      console.error('获取推荐链失败:', error);
      return [];
    }
  }

  async getReferralInfo(address: string): Promise<ReferralInfo> {
    const [sponsor, myCode, downlines, referralChain] = await Promise.all([
      this.getSponsor(address),
      this.getMyCode(address),
      this.getDownlines(address),
      this.getReferralChain(address),
    ]);

    return {
      sponsor,
      sponsorCode: null,
      myCode,
      downlines,
      downlineCount: downlines.length,
      referralChain,
    };
  }

  async getAccountByCode(code: string): Promise<string | null> {
    try {
      const api = this.getApi();
      const codeBytes = new TextEncoder().encode(code);
      const result = await (api.query as any).referral.accountByCode(codeBytes);
      if (result.isSome) {
        return result.unwrap().toString();
      }
      return null;
    } catch (error) {
      console.error('通过推荐码查找账户失败:', error);
      return null;
    }
  }
}

export const referralService = new ReferralService();
