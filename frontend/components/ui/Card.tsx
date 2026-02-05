import React from 'react';
import { View, StyleSheet, ViewStyle } from 'react-native';
import { useColors } from '@/hooks/useColors';
import { Shadows } from '@/constants/colors';

interface CardProps {
  children: React.ReactNode;
  style?: ViewStyle;
  shadow?: 'sm' | 'md' | 'lg' | 'none';
}

export function Card({ children, style, shadow = 'md' }: CardProps) {
  const colors = useColors();

  return (
    <View
      style={[
        styles.card,
        { backgroundColor: colors.surface, borderColor: colors.border },
        shadow !== 'none' && Shadows[shadow],
        style,
      ]}
    >
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    borderRadius: 16,
    padding: 16,
    borderWidth: 1,
  },
});
