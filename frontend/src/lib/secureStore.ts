/**
 * 星尘玄鉴 - 安全存储（Web 兼容版本）
 * Web 使用 localStorage，Native 使用 expo-secure-store
 */

import { Platform } from 'react-native';

// Web 存储实现
const webStorage = {
  async setItemAsync(key: string, value: string): Promise<void> {
    try {
      localStorage.setItem(key, value);
    } catch (error) {
      throw new Error(`Failed to save ${key}`);
    }
  },

  async getItemAsync(key: string): Promise<string | null> {
    try {
      return localStorage.getItem(key);
    } catch (error) {
      return null;
    }
  },

  async deleteItemAsync(key: string): Promise<void> {
    try {
      localStorage.removeItem(key);
    } catch (error) {
      throw new Error(`Failed to delete ${key}`);
    }
  },
};

// 根据平台选择存储实现
let SecureStore: typeof webStorage;

if (Platform.OS === 'web') {
  SecureStore = webStorage;
} else {
  // 动态导入 native 模块
  SecureStore = require('expo-secure-store');
}

export async function setItemAsync(key: string, value: string): Promise<void> {
  return SecureStore.setItemAsync(key, value);
}

export async function getItemAsync(key: string): Promise<string | null> {
  return SecureStore.getItemAsync(key);
}

export async function deleteItemAsync(key: string): Promise<void> {
  return SecureStore.deleteItemAsync(key);
}
