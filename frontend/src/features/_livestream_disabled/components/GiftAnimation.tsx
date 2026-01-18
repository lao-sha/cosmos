// frontend/src/features/livestream/components/GiftAnimation.tsx

import React, { useEffect, useRef } from 'react';
import { View, Text, StyleSheet, Animated, Dimensions, Image } from 'react-native';
import type { GiftAnimationItem } from '../types';

const { width: SCREEN_WIDTH, height: SCREEN_HEIGHT } = Dimensions.get('window');
const IPFS_GATEWAY = process.env.EXPO_PUBLIC_IPFS_GATEWAY || 'https://ipfs.io/ipfs/';

interface GiftAnimationProps {
  item: GiftAnimationItem;
  giftIconCid?: string;
  onComplete: () => void;
}

export function GiftAnimation({ item, giftIconCid, onComplete }: GiftAnimationProps) {
  const translateY = useRef(new Animated.Value(100)).current;
  const opacity = useRef(new Animated.Value(0)).current;
  const scale = useRef(new Animated.Value(0.5)).current;

  useEffect(() => {
    // 入场动画
    Animated.parallel([
      Animated.timing(translateY, {
        toValue: 0,
        duration: 300,
        useNativeDriver: true,
      }),
      Animated.timing(opacity, {
        toValue: 1,
        duration: 300,
        useNativeDriver: true,
      }),
      Animated.spring(scale, {
        toValue: 1,
        friction: 5,
        useNativeDriver: true,
      }),
    ]).start(() => {
      // 停留后退出
      setTimeout(() => {
        Animated.parallel([
          Animated.timing(translateY, {
            toValue: -50,
            duration: 300,
            useNativeDriver: true,
          }),
          Animated.timing(opacity, {
            toValue: 0,
            duration: 300,
            useNativeDriver: true,
          }),
        ]).start(onComplete);
      }, item.isFullScreen ? 2500 : 1200);
    });
  }, []);

  if (item.isFullScreen) {
    return (
      <Animated.View
        style={[
          styles.fullScreenContainer,
          {
            opacity,
            transform: [{ scale }],
          },
        ]}
      >
        <View style={styles.fullScreenContent}>
          {giftIconCid && (
            <Image
              source={{ uri: `${IPFS_GATEWAY}${giftIconCid}` }}
              style={styles.fullScreenIcon}
            />
          )}
          <Text style={styles.fullScreenText}>
            <Text style={styles.senderName}>{item.senderName}</Text>
            {' 送出 '}
            <Text style={styles.giftName}>{item.giftName}</Text>
            {item.quantity > 1 && (
              <Text style={styles.quantity}> x{item.quantity}</Text>
            )}
          </Text>
        </View>
      </Animated.View>
    );
  }

  return (
    <Animated.View
      style={[
        styles.normalContainer,
        {
          opacity,
          transform: [{ translateY }, { scale }],
        },
      ]}
    >
      <View style={styles.normalContent}>
        {giftIconCid && (
          <Image
            source={{ uri: `${IPFS_GATEWAY}${giftIconCid}` }}
            style={styles.normalIcon}
          />
        )}
        <View style={styles.textContainer}>
          <Text style={styles.normalSender}>{item.senderName}</Text>
          <Text style={styles.normalGift}>
            送出 {item.giftName}
            {item.quantity > 1 && ` x${item.quantity}`}
          </Text>
        </View>
      </View>
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  fullScreenContainer: {
    ...StyleSheet.absoluteFillObject,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: 'rgba(0,0,0,0.5)',
    zIndex: 1000,
  },
  fullScreenContent: {
    alignItems: 'center',
  },
  fullScreenIcon: {
    width: 120,
    height: 120,
    marginBottom: 20,
  },
  fullScreenText: {
    color: '#FFF',
    fontSize: 20,
    textAlign: 'center',
  },
  senderName: {
    color: '#FF4757',
    fontWeight: 'bold',
  },
  giftName: {
    color: '#FFD700',
    fontWeight: 'bold',
  },
  quantity: {
    color: '#FF4757',
    fontWeight: 'bold',
  },
  normalContainer: {
    position: 'absolute',
    left: 12,
    bottom: 200,
    zIndex: 100,
  },
  normalContent: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(0,0,0,0.7)',
    borderRadius: 24,
    paddingVertical: 8,
    paddingHorizontal: 12,
    paddingRight: 16,
  },
  normalIcon: {
    width: 40,
    height: 40,
    marginRight: 10,
  },
  textContainer: {
    flexDirection: 'column',
  },
  normalSender: {
    color: '#FF4757',
    fontSize: 13,
    fontWeight: '600',
  },
  normalGift: {
    color: '#FFD700',
    fontSize: 12,
    marginTop: 2,
  },
});
