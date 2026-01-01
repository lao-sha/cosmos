/**
 * 星尘玄鉴 - 触觉反馈 Hook
 * 提供统一的触觉反馈接口
 */

import { useCallback } from 'react';
import * as Haptics from 'expo-haptics';

type HapticType = 'light' | 'medium' | 'heavy' | 'success' | 'warning' | 'error' | 'selection';

interface HapticResult {
  vibrate: (type?: HapticType) => Promise<void>;
  isSupported: boolean;
}

/**
 * 触觉反馈 Hook
 */
export function useHaptic(): HapticResult {
  const vibrate = useCallback(async (type: HapticType = 'medium'): Promise<void> => {
    try {
      switch (type) {
        case 'light':
          await Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light);
          break;
        case 'medium':
          await Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium);
          break;
        case 'heavy':
          await Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Heavy);
          break;
        case 'success':
          await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
          break;
        case 'warning':
          await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Warning);
          break;
        case 'error':
          await Haptics.notificationAsync(Haptics.NotificationFeedbackType.Error);
          break;
        case 'selection':
          await Haptics.selectionAsync();
          break;
        default:
          await Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium);
      }
    } catch (error) {
      console.warn('[Haptic] Vibration error:', error);
    }
  }, []);

  return {
    vibrate,
    isSupported: true, // Expo Haptics 保证支持
  };
}
