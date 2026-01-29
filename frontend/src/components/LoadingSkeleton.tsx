import { useEffect, useRef } from 'react';
import { Animated, StyleSheet, View, ViewStyle } from 'react-native';

interface LoadingSkeletonProps {
  width?: number | string;
  height?: number;
  borderRadius?: number;
  style?: ViewStyle;
}

export function LoadingSkeleton({
  width = '100%',
  height = 16,
  borderRadius = 4,
  style,
}: LoadingSkeletonProps) {
  const opacity = useRef(new Animated.Value(0.3)).current;

  useEffect(() => {
    const animation = Animated.loop(
      Animated.sequence([
        Animated.timing(opacity, {
          toValue: 0.7,
          duration: 800,
          useNativeDriver: true,
        }),
        Animated.timing(opacity, {
          toValue: 0.3,
          duration: 800,
          useNativeDriver: true,
        }),
      ])
    );
    animation.start();
    return () => animation.stop();
  }, [opacity]);

  return (
    <Animated.View
      style={[
        styles.skeleton,
        {
          width,
          height,
          borderRadius,
          opacity,
        },
        style,
      ]}
    />
  );
}

interface CardSkeletonProps {
  lines?: number;
  showAvatar?: boolean;
}

export function CardSkeleton({ lines = 3, showAvatar = true }: CardSkeletonProps) {
  return (
    <View style={styles.card}>
      <View style={styles.cardHeader}>
        {showAvatar && (
          <LoadingSkeleton width={48} height={48} borderRadius={24} />
        )}
        <View style={styles.headerContent}>
          <LoadingSkeleton width="60%" height={16} />
          <LoadingSkeleton width="40%" height={12} style={{ marginTop: 8 }} />
        </View>
      </View>
      <View style={styles.cardBody}>
        {Array.from({ length: lines }).map((_, index) => (
          <LoadingSkeleton
            key={index}
            width={index === lines - 1 ? '70%' : '100%'}
            height={14}
            style={{ marginTop: index > 0 ? 10 : 0 }}
          />
        ))}
      </View>
    </View>
  );
}

export function ListSkeleton({ count = 3 }: { count?: number }) {
  return (
    <View style={styles.list}>
      {Array.from({ length: count }).map((_, index) => (
        <CardSkeleton key={index} />
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  skeleton: {
    backgroundColor: '#e5e7eb',
  },
  card: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  cardHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  headerContent: {
    flex: 1,
    marginLeft: 12,
  },
  cardBody: {
    gap: 10,
  },
  list: {
    padding: 16,
  },
});
