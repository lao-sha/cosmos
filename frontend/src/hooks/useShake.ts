/**
 * 星尘玄鉴 - 摇晃检测 Hook
 * 用于六爻摇卦等功能，支持后台暂停和平台差异化
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { Platform, AppState, type AppStateStatus } from 'react-native';
import { Accelerometer } from 'expo-sensors';
import * as Haptics from 'expo-haptics';

interface ShakeOptions {
  threshold?: number;      // 加速度阈值（平台自适应）
  cooldown?: number;       // 冷却时间 ms
  onShake?: () => void;    // 摇晃回调
}

interface ShakeResult {
  isShaking: boolean;
  shakeCount: number;
  start: () => void;
  stop: () => void;
  reset: () => void;
  isSupported: boolean;
}

// P1-6：平台差异化阈值
const DEFAULT_THRESHOLD = Platform.select({
  ios: 1.2,      // iOS 传感器更灵敏
  android: 1.5,  // Android 阈值稍高
  default: 1.5,
})!;

const UPDATE_INTERVAL = 300; // 300ms 更新间隔（平衡性能和响应）

/**
 * 摇晃检测 Hook
 */
export function useShake(options: ShakeOptions = {}): ShakeResult {
  const {
    threshold = DEFAULT_THRESHOLD,
    cooldown = 800,
    onShake,
  } = options;

  const [isShaking, setIsShaking] = useState(false);
  const [shakeCount, setShakeCount] = useState(0);
  const [isListening, setIsListening] = useState(false);

  const lastShakeRef = useRef(0);
  const subscriptionRef = useRef<ReturnType<typeof Accelerometer.addListener> | null>(null);
  const appStateRef = useRef<AppStateStatus>(AppState.currentState);
  const onShakeRef = useRef(onShake);

  // 更新回调引用
  useEffect(() => {
    onShakeRef.current = onShake;
  }, [onShake]);

  /**
   * 处理摇晃事件
   */
  const handleShake = useCallback(() => {
    const now = Date.now();
    if (now - lastShakeRef.current < cooldown) return;

    lastShakeRef.current = now;
    setIsShaking(true);
    setShakeCount(c => c + 1);

    // 触觉反馈
    Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium).catch((err) => {
      console.warn('[Shake] Haptics error:', err);
    });

    // 触发回调
    onShakeRef.current?.();

    // 重置摇晃状态
    setTimeout(() => setIsShaking(false), 300);
  }, [cooldown]);

  /**
   * 开始监听
   */
  const start = useCallback(() => {
    if (isListening) return;

    try {
      Accelerometer.setUpdateInterval(UPDATE_INTERVAL);

      subscriptionRef.current = Accelerometer.addListener(({ x, y, z }) => {
        const acceleration = Math.sqrt(x * x + y * y + z * z);

        if (acceleration > threshold) {
          handleShake();
        }
      });

      setIsListening(true);
      console.log('[Shake] Started listening');
    } catch (error) {
      console.error('[Shake] Start error:', error);
    }
  }, [isListening, threshold, handleShake]);

  /**
   * 停止监听
   */
  const stop = useCallback(() => {
    subscriptionRef.current?.remove();
    subscriptionRef.current = null;
    setIsListening(false);
    console.log('[Shake] Stopped listening');
  }, []);

  /**
   * 重置计数
   */
  const reset = useCallback(() => {
    setShakeCount(0);
    setIsShaking(false);
    lastShakeRef.current = 0;
  }, []);

  // P1-5：监听应用状态（后台暂停传感器）
  useEffect(() => {
    const subscription = AppState.addEventListener('change', (nextAppState) => {
      if (appStateRef.current === 'active' && nextAppState.match(/inactive|background/)) {
        // 进入后台，暂停监听
        console.log('[Shake] App went to background, stopping');
        stop();
      } else if (appStateRef.current.match(/inactive|background/) && nextAppState === 'active') {
        // 回到前台（如果之前在监听则恢复）
        console.log('[Shake] App came to foreground');
        // 注意：不自动恢复，由页面控制
      }

      appStateRef.current = nextAppState;
    });

    return () => {
      subscription.remove();
    };
  }, [stop]);

  // 组件卸载时清理
  useEffect(() => {
    return () => {
      subscriptionRef.current?.remove();
    };
  }, []);

  return {
    isShaking,
    shakeCount,
    start,
    stop,
    reset,
    isSupported: true, // Expo Sensors 保证支持
  };
}
