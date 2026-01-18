// frontend/src/features/livestream/hooks/useDanmaku.ts

import { useCallback, useRef, useState } from 'react';
import { Animated, Dimensions } from 'react-native';

const { width: SCREEN_WIDTH } = Dimensions.get('window');

// 性能配置
const MAX_DANMAKU_ON_SCREEN = 50;
const MAX_DANMAKU_PER_SECOND = 10;
const TRACK_COUNT = 12;

interface DanmakuItem {
  id: number;
  content: string;
  color: string;
  translateX: Animated.Value;
  opacity: Animated.Value;
  track: number;
  active: boolean;
}

interface UseDanmakuOptions {
  maxOnScreen?: number;
  maxPerSecond?: number;
  trackCount?: number;
  duration?: number;
}

export function useDanmaku(options: UseDanmakuOptions = {}) {
  const {
    maxOnScreen = MAX_DANMAKU_ON_SCREEN,
    maxPerSecond = MAX_DANMAKU_PER_SECOND,
    trackCount = TRACK_COUNT,
    duration = 8000,
  } = options;

  const poolRef = useRef<DanmakuItem[]>([]);
  const activeCountRef = useRef(0);
  const lastSecondCountRef = useRef(0);
  const lastSecondTimeRef = useRef(Date.now());
  const trackLastUseRef = useRef<number[]>(new Array(trackCount).fill(0));
  const pendingQueueRef = useRef<{ content: string; color: string }[]>([]);
  const idCounterRef = useRef(0);

  const [, forceUpdate] = useState(0);

  // 初始化对象池
  const initPool = useCallback(() => {
    if (poolRef.current.length > 0) return;

    poolRef.current = Array.from({ length: maxOnScreen + 10 }, (_, i) => ({
      id: i,
      content: '',
      color: '#FFFFFF',
      translateX: new Animated.Value(SCREEN_WIDTH),
      opacity: new Animated.Value(0),
      track: 0,
      active: false,
    }));
  }, [maxOnScreen]);

  // 获取最佳轨道
  const getBestTrack = useCallback((): number => {
    const now = Date.now();
    let bestTrack = 0;
    let oldestTime = trackLastUseRef.current[0];

    for (let i = 1; i < trackCount; i++) {
      if (trackLastUseRef.current[i] < oldestTime) {
        oldestTime = trackLastUseRef.current[i];
        bestTrack = i;
      }
    }

    trackLastUseRef.current[bestTrack] = now;
    return bestTrack;
  }, [trackCount]);

  // 从对象池获取
  const acquireFromPool = useCallback((): DanmakuItem | null => {
    const item = poolRef.current.find((d) => !d.active);
    if (item) {
      item.active = true;
      activeCountRef.current++;
    }
    return item || null;
  }, []);

  // 归还到对象池
  const releaseToPool = useCallback((item: DanmakuItem) => {
    item.active = false;
    item.translateX.setValue(SCREEN_WIDTH);
    item.opacity.setValue(0);
    activeCountRef.current--;

    // 处理等待队列
    if (pendingQueueRef.current.length > 0) {
      const pending = pendingQueueRef.current.shift()!;
      setTimeout(() => addDanmaku(pending.content, pending.color), 50);
    }
  }, []);

  // 添加弹幕
  const addDanmaku = useCallback(
    (content: string, color: string = '#FFFFFF') => {
      initPool();

      const now = Date.now();

      // 重置每秒计数器
      if (now - lastSecondTimeRef.current > 1000) {
        lastSecondCountRef.current = 0;
        lastSecondTimeRef.current = now;
      }

      // 限流检查
      if (lastSecondCountRef.current >= maxPerSecond) {
        if (pendingQueueRef.current.length < 20) {
          pendingQueueRef.current.push({ content, color });
        }
        return;
      }

      // 屏幕弹幕数量检查
      if (activeCountRef.current >= maxOnScreen) {
        return;
      }

      const item = acquireFromPool();
      if (!item) return;

      lastSecondCountRef.current++;

      // 配置弹幕
      item.content = content;
      item.color = color;
      item.track = getBestTrack();
      item.translateX.setValue(SCREEN_WIDTH);
      item.opacity.setValue(1);

      forceUpdate((n) => n + 1);

      // 动画
      const animDuration = duration + Math.random() * 2000;
      Animated.timing(item.translateX, {
        toValue: -content.length * 20,
        duration: animDuration,
        useNativeDriver: true,
      }).start(() => {
        releaseToPool(item);
        forceUpdate((n) => n + 1);
      });
    },
    [
      initPool,
      maxPerSecond,
      maxOnScreen,
      acquireFromPool,
      getBestTrack,
      duration,
      releaseToPool,
    ]
  );

  // 清空所有弹幕
  const clearAll = useCallback(() => {
    poolRef.current.forEach((item) => {
      if (item.active) {
        item.active = false;
        item.translateX.setValue(SCREEN_WIDTH);
        item.opacity.setValue(0);
      }
    });
    activeCountRef.current = 0;
    pendingQueueRef.current = [];
    forceUpdate((n) => n + 1);
  }, []);

  // 获取活跃弹幕
  const getActiveDanmakus = useCallback(() => {
    return poolRef.current.filter((d) => d.active);
  }, []);

  return {
    addDanmaku,
    clearAll,
    getActiveDanmakus,
    activeCount: activeCountRef.current,
  };
}
