/**
 * Matchmaking 服务
 * 从链上获取相亲档案数据
 */

import { useChainStore } from '@/src/stores/chain';
import { ApiPromise } from '@polkadot/api';

export interface MatchProfile {
  id: string;
  address: string;
  nickname: string;
  age: number | null;
  gender: 'male' | 'female';
  location: string;
  zodiac: string;
  baziScore?: number;
  bio: string;
  isVerified: boolean;
  photos: string[];
  completeness: number;
  status: string;
  createdAt: number;
}

class MatchmakingService {
  private getApi(): ApiPromise {
    const { api } = useChainStore.getState();
    if (!api) {
      throw new Error('Chain not connected');
    }
    return api;
  }

  async getProfile(address: string): Promise<MatchProfile | null> {
    try {
      const api = this.getApi();
      const result = await (api.query as any).matchmakingProfile.profiles(address);
      
      if (result && result.isSome) {
        const profile = result.unwrap();
        return this.parseProfile(address, profile);
      }
      return null;
    } catch (error) {
      console.error('获取档案失败:', error);
      return null;
    }
  }

  async getAllProfiles(): Promise<MatchProfile[]> {
    try {
      const api = this.getApi();
      const profiles: MatchProfile[] = [];
      
      // 获取所有档案
      const entries = await (api.query as any).matchmakingProfile.profiles.entries();
      console.log('[Matchmaking] 获取到档案数:', entries.length);
      
      for (const [key, value] of entries) {
        if (value && value.isSome) {
          const address = key.args[0].toString();
          const profile = value.unwrap();
          const parsed = this.parseProfile(address, profile);
          if (parsed) {
            profiles.push(parsed);
          }
        }
      }
      
      console.log('[Matchmaking] 解析成功档案数:', profiles.length);
      return profiles;
    } catch (error) {
      console.error('获取所有档案失败:', error);
      return [];
    }
  }

  async getProfileCount(): Promise<number> {
    try {
      const api = this.getApi();
      const count = await (api.query as any).matchmakingProfile.profileCount();
      return count ? count.toNumber() : 0;
    } catch (error) {
      console.error('获取档案数量失败:', error);
      return 0;
    }
  }

  async getProfilesByGender(gender: 'Male' | 'Female'): Promise<MatchProfile[]> {
    try {
      const api = this.getApi();
      const profiles: MatchProfile[] = [];
      
      // 通过性别索引获取
      const entries = await (api.query as any).matchmakingProfile.genderIndex.entries(gender);
      
      for (const [key] of entries) {
        const address = key.args[1].toString();
        const profile = await this.getProfile(address);
        if (profile) {
          profiles.push(profile);
        }
      }
      
      return profiles;
    } catch (error) {
      console.error('按性别获取档案失败:', error);
      // 降级到获取所有档案并过滤
      const allProfiles = await this.getAllProfiles();
      const genderLower = gender.toLowerCase() as 'male' | 'female';
      return allProfiles.filter(p => p.gender === genderLower);
    }
  }

  private parseProfile(address: string, profile: any): MatchProfile | null {
    try {
      const nickname = profile.nickname && profile.nickname.length > 0
        ? new TextDecoder().decode(new Uint8Array(profile.nickname))
        : '未设置';
      
      const gender = profile.gender?.toString()?.toLowerCase() === 'female' ? 'female' : 'male';
      
      const location = profile.currentLocation && profile.currentLocation.length > 0
        ? new TextDecoder().decode(new Uint8Array(profile.currentLocation))
        : '未设置';
      
      const bio = profile.bio && profile.bio.length > 0
        ? new TextDecoder().decode(new Uint8Array(profile.bio))
        : '';
      
      // 解析照片 CID
      const photos: string[] = [];
      if (profile.photoCids && profile.photoCids.length > 0) {
        for (const cid of profile.photoCids) {
          if (cid && cid.length > 0) {
            photos.push(new TextDecoder().decode(new Uint8Array(cid)));
          }
        }
      }
      
      // 头像
      if (profile.avatarCid && profile.avatarCid.isSome) {
        const avatarCid = new TextDecoder().decode(new Uint8Array(profile.avatarCid.unwrap()));
        if (avatarCid && !photos.includes(avatarCid)) {
          photos.unshift(avatarCid);
        }
      }
      
      return {
        id: address.slice(0, 8),
        address,
        nickname,
        age: profile.age ? (profile.age.isSome ? profile.age.unwrap().toNumber() : null) : null,
        gender,
        location,
        zodiac: this.getZodiacFromBirthDate(profile.birthDate),
        bio,
        isVerified: profile.verified || false,
        photos,
        completeness: profile.completeness ? profile.completeness.toNumber() : 0,
        status: profile.status?.toString() || 'Unknown',
        createdAt: profile.createdAt ? profile.createdAt.toNumber() : 0,
      };
    } catch (error) {
      console.error('解析档案失败:', error);
      return null;
    }
  }

  private getZodiacFromBirthDate(birthDate: any): string {
    if (!birthDate || !birthDate.isSome) {
      return '未知';
    }
    
    try {
      const date = birthDate.unwrap();
      const month = date.month?.toNumber() || 1;
      const day = date.day?.toNumber() || 1;
      
      // 星座计算
      const zodiacSigns = [
        { name: '摩羯座', start: [1, 1], end: [1, 19] },
        { name: '水瓶座', start: [1, 20], end: [2, 18] },
        { name: '双鱼座', start: [2, 19], end: [3, 20] },
        { name: '白羊座', start: [3, 21], end: [4, 19] },
        { name: '金牛座', start: [4, 20], end: [5, 20] },
        { name: '双子座', start: [5, 21], end: [6, 21] },
        { name: '巨蟹座', start: [6, 22], end: [7, 22] },
        { name: '狮子座', start: [7, 23], end: [8, 22] },
        { name: '处女座', start: [8, 23], end: [9, 22] },
        { name: '天秤座', start: [9, 23], end: [10, 23] },
        { name: '天蝎座', start: [10, 24], end: [11, 22] },
        { name: '射手座', start: [11, 23], end: [12, 21] },
        { name: '摩羯座', start: [12, 22], end: [12, 31] },
      ];
      
      for (const sign of zodiacSigns) {
        const [startMonth, startDay] = sign.start;
        const [endMonth, endDay] = sign.end;
        
        if (
          (month === startMonth && day >= startDay) ||
          (month === endMonth && day <= endDay)
        ) {
          return sign.name;
        }
      }
      
      return '未知';
    } catch {
      return '未知';
    }
  }
}

export const matchmakingService = new MatchmakingService();
