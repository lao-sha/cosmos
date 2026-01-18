// frontend/src/divination/market/components/Avatar.tsx

import React from 'react';
import { View, Image, Text, StyleSheet } from 'react-native';
import { THEME } from '../theme';

interface AvatarProps {
  uri?: string;
  name?: string;
  size?: number;
  borderColor?: string;
  borderWidth?: number;
}

export const Avatar: React.FC<AvatarProps> = ({
  uri,
  name,
  size = 48,
  borderColor,
  borderWidth = 0,
}) => {
  const initial = name ? name.charAt(0).toUpperCase() : '?';

  const containerStyle = {
    width: size,
    height: size,
    borderRadius: size / 2,
    borderColor: borderColor || THEME.border,
    borderWidth,
  };

  if (uri) {
    return (
      <Image
        source={{ uri }}
        style={[styles.image, containerStyle]}
      />
    );
  }

  return (
    <View style={[styles.placeholder, containerStyle, { backgroundColor: THEME.primaryLight }]}>
      <Text style={[styles.initial, { fontSize: size * 0.4, color: THEME.primary }]}>
        {initial}
      </Text>
    </View>
  );
};

const styles = StyleSheet.create({
  image: {
    resizeMode: 'cover',
  },
  placeholder: {
    justifyContent: 'center',
    alignItems: 'center',
  },
  initial: {
    fontWeight: '600',
  },
});
