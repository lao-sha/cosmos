// frontend/src/divination/market/components/SpecialtyTags.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../theme';
import { getSpecialtiesFromBitmap } from '../utils/market.utils';

interface SpecialtyTagsProps {
  specialties: number; // 位图
  maxDisplay?: number;
  size?: 'small' | 'medium';
  showIcon?: boolean;
}

export const SpecialtyTags: React.FC<SpecialtyTagsProps> = ({
  specialties,
  maxDisplay = 4,
  size = 'small',
  showIcon = false,
}) => {
  const specialtyList = getSpecialtiesFromBitmap(specialties);
  const displayList = specialtyList.slice(0, maxDisplay);
  const remainCount = specialtyList.length - maxDisplay;

  const sizeStyles = {
    small: { paddingH: 6, paddingV: 2, fontSize: 11, iconSize: 10, gap: 4 },
    medium: { paddingH: 8, paddingV: 3, fontSize: 12, iconSize: 12, gap: 6 },
  };

  const s = sizeStyles[size];

  return (
    <View style={[styles.container, { gap: s.gap }]}>
      {displayList.map((specialty) => (
        <View
          key={specialty.bit}
          style={[
            styles.tag,
            {
              backgroundColor: specialty.color + '15',
              paddingHorizontal: s.paddingH,
              paddingVertical: s.paddingV,
            },
          ]}
        >
          {showIcon && (
            <Ionicons
              name={specialty.icon as any}
              size={s.iconSize}
              color={specialty.color}
              style={styles.icon}
            />
          )}
          <Text style={[styles.text, { fontSize: s.fontSize, color: specialty.color }]}>
            {specialty.name}
          </Text>
        </View>
      ))}
      {remainCount > 0 && (
        <View
          style={[
            styles.tag,
            {
              backgroundColor: THEME.border,
              paddingHorizontal: s.paddingH,
              paddingVertical: s.paddingV,
            },
          ]}
        >
          <Text style={[styles.text, { fontSize: s.fontSize, color: THEME.textTertiary }]}>
            +{remainCount}
          </Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  tag: {
    flexDirection: 'row',
    alignItems: 'center',
    borderRadius: 4,
  },
  icon: {
    marginRight: 2,
  },
  text: {
    fontWeight: '500',
  },
});
