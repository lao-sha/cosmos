// Cross-platform storage abstraction for Expo (web + native)
import { Platform } from 'react-native';

interface StorageInterface {
  getItem(key: string): Promise<string | null>;
  setItem(key: string, value: string): Promise<void>;
  removeItem(key: string): Promise<void>;
  multiRemove(keys: string[]): Promise<void>;
}

// Web implementation using localStorage
const webStorage: StorageInterface = {
  async getItem(key: string): Promise<string | null> {
    try {
      return localStorage.getItem(key);
    } catch {
      return null;
    }
  },
  async setItem(key: string, value: string): Promise<void> {
    localStorage.setItem(key, value);
  },
  async removeItem(key: string): Promise<void> {
    localStorage.removeItem(key);
  },
  async multiRemove(keys: string[]): Promise<void> {
    keys.forEach(key => localStorage.removeItem(key));
  },
};

// Native implementation using AsyncStorage
let nativeStorage: StorageInterface | null = null;

const getNativeStorage = async (): Promise<StorageInterface> => {
  if (!nativeStorage) {
    const AsyncStorage = (await import('@react-native-async-storage/async-storage')).default;
    nativeStorage = {
      getItem: (key) => AsyncStorage.getItem(key),
      setItem: (key, value) => AsyncStorage.setItem(key, value),
      removeItem: (key) => AsyncStorage.removeItem(key),
      multiRemove: (keys) => AsyncStorage.multiRemove(keys),
    };
  }
  return nativeStorage;
};

// Unified storage API
export const storage: StorageInterface = Platform.OS === 'web'
  ? webStorage
  : {
      async getItem(key: string): Promise<string | null> {
        const s = await getNativeStorage();
        return s.getItem(key);
      },
      async setItem(key: string, value: string): Promise<void> {
        const s = await getNativeStorage();
        return s.setItem(key, value);
      },
      async removeItem(key: string): Promise<void> {
        const s = await getNativeStorage();
        return s.removeItem(key);
      },
      async multiRemove(keys: string[]): Promise<void> {
        const s = await getNativeStorage();
        return s.multiRemove(keys);
      },
    };

export default storage;
