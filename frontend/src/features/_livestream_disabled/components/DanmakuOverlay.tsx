// frontend/src/features/livestream/components/DanmakuOverlay.tsx

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { View, Animated, StyleSheet, Dimensions } from 'react-native';
import { useLivestreamStore } from '@/stores/livestream.store';

const { width: SCREEN_WIDTH } = Dimensions.get('window');

// 性能配置
const MAX_DANMAKU_ON_SCREEN = 50;
const MAX_DANMAKU_PER_SECOND = 10;
const DANMAKU_POOL_SIZE = 60;
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

interface DanmakuOverlayProps {
  roomId: number;
}

export function DanmakuOverlay({ roomId }: DanmakuOverlayProps) {
  // 对象池 - 预创建 Animated.Value 避免频繁 GC
  const poolRef = useRef<DanmakuItem[]>([]);
  const activeCountRef = useRef(0);
  const lastSecondCountRef = useRef(0);
  const lastSecondTimeRef = useRef(Date.now());
  const trackLastUseRef = useRef<number[]>(new Array(TRACK_COUNT).fill(0));
  const pendingQueueRef = useRef<{ content: string; color: string }[]>([]);

  const [, forceUpdate] = useState(0);
  const { messages } = useLivestreamStore();
  const lastMessageIdRef = useRef<string | null>(null);

  // 初始化对象池
  useEffect(() => {
    poolRef.current = Array.from({ length: DANMAKU_POOL_SIZE }, (_, i) => ({
      id: i,
      content: '',
      color: '#FFFFFF',
      translateX: new Animated.Value(SCREEN_WIDTH),
      opacity: new Animated.Value(0),
      track: 0,
      active: false,
    }));
  }, []);

  // 获取最佳轨道 (避免重叠)
  const getBestTrack = useCallback((): number => {
    const now = Date.now();
    let bestTrack = 0;
    let oldestTime = trackLastUseRef.current[0];

    for (let i = 1; i < TRACK_COUNT; i++) {
      if (trackLastUseRef.current[i] < oldestTime) {
        oldestTime = trackLastUseRef.current[i];
        bestTrack = i;
      }
    }

    trackLastUseRef.current[bestTrack] = now;
    return bestTrack;
  }, []);

  // 从对象池获取可用项
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

  // 添加弹幕 (带限流)
  const addDanmaku = useCallback(
    (content: string, color: string) => {
      const now = Date.now();

      // 重置每秒计数器
      if (now - lastSecondTimeRef.current > 1000) {
        lastSecondCountRef.current = 0;
        lastSecondTimeRef.current = now;
      }

      // 限流检查
      if (lastSecondCountRef.current >= MAX_DANMAKU_PER_SECOND) {
        // 加入等待队列 (最多缓存 20 条)
        if (pendingQueueRef.current.length < 20) {
          pendingQueueRef.current.push({ content, color });
        }
        return;
      }

      // 屏幕弹幕数量检查
      if (activeCountRef.current >= MAX_DANMAKU_ON_SCREEN) {
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
      Animated.timing(item.translateX, {
        toValue: -content.length * 20,
        duration: 8000 + Math.random() * 2000,
        useNativeDriver: true,
      }).start(() => {
        releaseToPool(item);
        forceUpdate((n) => n + 1);
      });
    },
    [acquireFromPool, releaseToPool, getBestTrack]
  );

  // 监听弹幕消息
  useEffect(() => {
    const danmakuMessages = messages.filter((m) => m.type === 'danmaku');
    const lastMessage = danmakuMessages[danmakuMessages.length - 1];

    if (lastMessage && lastMessage.id !== lastMessageIdRef.current) {
      lastMessageIdRef.current = lastMessage.id;
      addDanmaku(lastMessage.content, lastMessage.color || '#FFFFFF');
    }
  }, [messages, addDanmaku]);

  const activeDanmakus = poolRef.current.filter((d) => d.active);

  return (
    <View style={StyleSheet.absoluteFill} pointerEvents="none">
      {activeDanmakus.map((d) => (
        <Animated.Text
          key={d.id}
          style={[
            styles.danmaku,
            {
              color: d.color,
              top: d.track * 32 + 20,
              opacity: d.opacity,
              transform: [{ translateX: d.translateX }],
            },
          ]}
        >
          {d.content}
        </Animated.Text>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  danmaku: {
    position: 'absolute',
    fontSize: 16,
    fontWeight: 'bold',
    textShadowColor: '#000',
    textShadowOffset: { width: 1, height: 1 },
    textShadowRadius: 2,
  },
});
