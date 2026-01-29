import { StyleSheet, Text, View } from 'react-native';

interface RatingStarsProps {
  rating: number;
  maxRating?: number;
  size?: 'small' | 'medium' | 'large';
  showValue?: boolean;
  showCount?: boolean;
  count?: number;
}

const SIZES = {
  small: { star: 12, value: 12, count: 11 },
  medium: { star: 16, value: 14, count: 12 },
  large: { star: 20, value: 16, count: 14 },
};

export function RatingStars({
  rating,
  maxRating = 5,
  size = 'medium',
  showValue = true,
  showCount = false,
  count = 0,
}: RatingStarsProps) {
  const dimensions = SIZES[size];
  const fullStars = Math.floor(rating);
  const hasHalf = rating - fullStars >= 0.5;
  const emptyStars = maxRating - fullStars - (hasHalf ? 1 : 0);

  const renderStars = () => {
    const stars: string[] = [];
    
    for (let i = 0; i < fullStars; i++) {
      stars.push('★');
    }
    if (hasHalf) {
      stars.push('★');
    }
    for (let i = 0; i < emptyStars; i++) {
      stars.push('☆');
    }
    
    return stars.join('');
  };

  return (
    <View style={styles.container}>
      <Text style={[styles.stars, { fontSize: dimensions.star }]}>
        {renderStars()}
      </Text>
      
      {showValue && (
        <Text style={[styles.value, { fontSize: dimensions.value }]}>
          {rating.toFixed(1)}
        </Text>
      )}
      
      {showCount && count > 0 && (
        <Text style={[styles.count, { fontSize: dimensions.count }]}>
          ({count})
        </Text>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  stars: {
    color: '#f59e0b',
  },
  value: {
    color: '#f59e0b',
    fontWeight: '600',
  },
  count: {
    color: '#9ca3af',
  },
});
