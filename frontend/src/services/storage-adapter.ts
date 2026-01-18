/**
 * 星尘玄鉴 - 存储适配器
 * 为不同平台提供统一的存储接口
 */

import { Platform } from 'react-native';

interface StorageAdapter {
  getItemAsync(key: string, options?: { requireAuthentication?: boolean }): Promise<string | null>;
  setItemAsync(key: string, value: string, options?: { requireAuthentication?: boolean }): Promise<void>;
  deleteItemAsync(key: string): Promise<void>;
}

/**
 * Web 平台使用 localStorage 作为 fallback
 * 注意：这不如原生安全存储安全，仅用于开发测试
 */
class WebStorageAdapter implements StorageAdapter {
  private prefix = 'stardust_secure_';

  async getItemAsync(key: string): Promise<string | null> {
    try {
      return localStorage.getItem(this.prefix + key);
    } catch {
      return null;
    }
  }

  async setItemAsync(key: string, value: string): Promise<void> {
    try {
      localStorage.setItem(this.prefix + key, value);
    } catch (error) {
      console.warn('[WebStorage] Failed to save:', error);
    }
  }

  async deleteItemAsync(key: string): Promise<void> {
    try {
      localStorage.removeItem(this.prefix + key);
    } catch (error) {
      console.warn('[WebStorage] Failed to delete:', error);
    }
  }
}

/**
 * 原生平台使用 expo-secure-store
 */
class NativeStorageAdapter implements StorageAdapter {
  private secureStore: typeof import('expo-secure-store') | null = null;

  private async getSecureStore() {
    if (!this.secureStore) {
      this.secureStore = await import('expo-secure-store');
    }
    return this.secureStore;
  }

  async getItemAsync(key: string, options?: { requireAuthentication?: boolean }): Promise<string | null> {
    const store = await this.getSecureStore();
    return store.getItemAsync(key, options);
  }

  async setItemAsync(key: string, value: string, options?: { requireAuthentication?: boolean }): Promise<void> {
    const store = await this.getSecureStore();
    await store.setItemAsync(key, value, options);
  }

  async deleteItemAsync(key: string): Promise<void> {
    const store = await this.getSecureStore();
    await store.deleteItemAsync(key);
  }
}

/**
 * 根据平台选择适当的存储适配器
 */
export const storageAdapter: StorageAdapter = Platform.OS === 'web'
  ? new WebStorageAdapter()
  : new NativeStorageAdapter();
