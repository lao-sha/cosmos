// frontend/src/divination/market/components/LoadingSpinner.tsx

import React from 'react';
import { View, ActivityIndicator, Text, StyleSheet } from 'react-native';
import { THEME } from '../theme';

interface LoadingSpinnerProps {
  text?: string;
  size?: 'small' | 'large';
  color?: string;
  fullScreen?: boolean;
}

export const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({
  text,
  size = 'large',
  color = THEME.primary,
  fullScreen = false,
}) => {
  const content = (
    <>
      <ActivityIndicator size={size} color={color} />
      {text && <Text style={styles.text}>{text}</Text>}
    </>
  );

  if (fullScreen) {
    return (
      <View style={styles.fullScreen}>
        {content}
      </View>
    );
  }

  return (
    <View style={styles.container}>
      {content}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    padding: 24,
    justifyContent: 'center',
    alignItems: 'center',
  },
  fullScreen: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: THEME.background,
  },
  text: {
    marginTop: 12,
    fontSize: 14,
    color: THEME.textSecondary,
  },
});
