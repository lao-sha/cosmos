// frontend/src/divination/market/components/RatingStars.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../theme';

interface RatingStarsProps {
  rating: number; // 0-5
  maxRating?: number;
  size?: number;
  showValue?: boolean;
  count?: number; // 评价数量
}

export const RatingStars: React.FC<RatingStarsProps> = ({
  rating,
  maxRating = 5,
  size = 14,
  showValue = true,
  count,
}) => {
  const stars = [];
  const fullStars = Math.floor(rating);
  const hasHalfStar = rating - fullStars >= 0.5;

  for (let i = 0; i < maxRating; i++) {
    if (i < fullStars) {
      stars.push(
        <Ionicons
          key={i}
          name="star"
          size={size}
          color={THEME.primary}
          style={styles.star}
        />
      );
    } else if (i === fullStars && hasHalfStar) {
      stars.push(
        <Ionicons
          key={i}
          name="star-half"
          size={size}
          color={THEME.primary}
          style={styles.star}
        />
      );
    } else {
      stars.push(
        <Ionicons
          key={i}
          name="star-outline"
          size={size}
          color={THEME.primary}
          style={styles.star}
        />
      );
    }
  }

  return (
    <View style={styles.container}>
      <View style={styles.starsContainer}>{stars}</View>
      {showValue && (
        <Text style={[styles.value, { fontSize: size - 2 }]}>
          {rating.toFixed(1)}
        </Text>
      )}
      {count !== undefined && (
        <Text style={[styles.count, { fontSize: size - 3 }]}>({count})</Text>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  starsContainer: {
    flexDirection: 'row',
  },
  star: {
    marginRight: 1,
  },
  value: {
    color: THEME.primary,
    fontWeight: '600',
    marginLeft: 4,
  },
  count: {
    color: THEME.textTertiary,
    marginLeft: 2,
  },
});
