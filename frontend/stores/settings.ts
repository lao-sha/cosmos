import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';

type Network = 'local' | 'testnet' | 'mainnet';
type Language = 'zh' | 'en';

interface SettingsState {
  network: Network;
  language: Language;
  biometricEnabled: boolean;
  notificationsEnabled: boolean;

  setNetwork: (network: Network) => void;
  setLanguage: (language: Language) => void;
  setBiometricEnabled: (enabled: boolean) => void;
  setNotificationsEnabled: (enabled: boolean) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      network: 'local',
      language: 'zh',
      biometricEnabled: false,
      notificationsEnabled: true,

      setNetwork: (network) => set({ network }),
      setLanguage: (language) => set({ language }),
      setBiometricEnabled: (enabled) => set({ biometricEnabled: enabled }),
      setNotificationsEnabled: (enabled) => set({ notificationsEnabled: enabled }),
    }),
    {
      name: 'settings-storage',
      storage: createJSONStorage(() => AsyncStorage),
    }
  )
);
