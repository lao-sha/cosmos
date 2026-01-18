// frontend/src/divination/market/components/OfflineBanner.tsx

import React, { useEffect, useState } from 'react';
import { View, Text, StyleSheet, Animated } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import NetInfo from '@react-native-community/netinfo';
import { THEME } from '../theme';

interface OfflineBannerProps {
  onStatusChange?: (isOnline: boolean) => void;
}

export const OfflineBanner: React.FC<OfflineBannerProps> = ({ onStatusChange }) => {
  const [isOnline, setIsOnline] = useState(true);
  const [showBanner, setShowBanner] = useState(false);
  const slideAnim = useState(new Animated.Value(-50))[0];

  useEffect(() => {
    const unsubscribe = NetInfo.addEventListener((state) => {
      const online = state.isConnected ?? true;
      setIsOnline(online);
      onStatusChange?.(online);

      if (!online) {
        setShowBanner(true);
        Animated.spring(slideAnim, {
          toValue: 0,
          useNativeDriver: true,
          tension: 100,
          friction: 10,
        }).start();
      } else if (showBanner) {
        // 网络恢复时显示提示后消失
        setTimeout(() => {
          Animated.timing(slideAnim, {
            toValue: -50,
            duration: 300,
            useNativeDriver: true,
          }).start(() => setShowBanner(false));
        }, 2000);
      }
    });

    return () => unsubscribe();
  }, [showBanner]);

  if (!showBanner) return null;

  return (
    <Animated.View
      style={[
        styles.container,
        {
          transform: [{ translateY: slideAnim }],
          backgroundColor: isOnline ? THEME.success : THEME.warning,
        },
      ]}
    >
      <Ionicons
        name={isOnline ? 'wifi-outline' : 'cloud-offline-outline'}
        size={16}
        color={THEME.textInverse}
      />
      <Text style={styles.text}>
        {isOnline ? '网络已恢复' : '网络已断开，部分功能可能受限'}
      </Text>
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 8,
    paddingHorizontal: 16,
    gap: 8,
    zIndex: 1000,
  },
  text: {
    fontSize: 13,
    color: THEME.textInverse,
    fontWeight: '500',
  },
});
