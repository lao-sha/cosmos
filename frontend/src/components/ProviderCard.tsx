import { Pressable, StyleSheet, Text, View } from 'react-native';

export interface ProviderData {
  id: string;
  name: string;
  avatar?: string;
  specialties: string[];
  rating: number;
  completedOrders: number;
  price: string;
  responseTime?: string;
  isOnline?: boolean;
  description?: string;
}

interface ProviderCardProps {
  provider: ProviderData;
  onPress?: () => void;
  compact?: boolean;
}

export function ProviderCard({ provider, onPress, compact = false }: ProviderCardProps) {
  const renderStars = (rating: number) => {
    const stars = [];
    const fullStars = Math.floor(rating);
    const hasHalf = rating - fullStars >= 0.5;
    
    for (let i = 0; i < 5; i++) {
      if (i < fullStars) {
        stars.push('★');
      } else if (i === fullStars && hasHalf) {
        stars.push('☆');
      } else {
        stars.push('☆');
      }
    }
    return stars.join('');
  };

  if (compact) {
    return (
      <Pressable
        style={({ pressed }) => [styles.compactCard, pressed && styles.cardPressed]}
        onPress={onPress}
      >
        <View style={styles.compactAvatar}>
          <Text style={styles.avatarText}>{provider.name[0]}</Text>
          {provider.isOnline && <View style={styles.onlineDot} />}
        </View>
        <View style={styles.compactInfo}>
          <Text style={styles.compactName} numberOfLines={1}>{provider.name}</Text>
          <Text style={styles.compactSpecialty} numberOfLines={1}>
            {provider.specialties.slice(0, 2).join(' · ')}
          </Text>
        </View>
        <Text style={styles.compactPrice}>{provider.price}</Text>
      </Pressable>
    );
  }

  return (
    <Pressable
      style={({ pressed }) => [styles.card, pressed && styles.cardPressed]}
      onPress={onPress}
    >
      <View style={styles.header}>
        <View style={styles.avatarContainer}>
          <View style={styles.avatar}>
            <Text style={styles.avatarText}>{provider.name[0]}</Text>
          </View>
          {provider.isOnline && <View style={styles.onlineIndicator} />}
        </View>
        
        <View style={styles.info}>
          <Text style={styles.name}>{provider.name}</Text>
          <View style={styles.ratingRow}>
            <Text style={styles.stars}>{renderStars(provider.rating)}</Text>
            <Text style={styles.ratingText}>{provider.rating.toFixed(1)}</Text>
            <Text style={styles.ordersText}>({provider.completedOrders}单)</Text>
          </View>
        </View>
        
        <View style={styles.priceContainer}>
          <Text style={styles.priceLabel}>起步价</Text>
          <Text style={styles.price}>{provider.price}</Text>
        </View>
      </View>
      
      <View style={styles.specialties}>
        {provider.specialties.map((specialty, index) => (
          <View key={index} style={styles.specialtyTag}>
            <Text style={styles.specialtyText}>{specialty}</Text>
          </View>
        ))}
      </View>
      
      {provider.description && (
        <Text style={styles.description} numberOfLines={2}>
          {provider.description}
        </Text>
      )}
      
      <View style={styles.footer}>
        {provider.responseTime && (
          <Text style={styles.responseTime}>⏱ 平均响应 {provider.responseTime}</Text>
        )}
      </View>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  card: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.05,
    shadowRadius: 8,
    elevation: 2,
  },
  cardPressed: {
    opacity: 0.9,
    transform: [{ scale: 0.99 }],
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  avatarContainer: {
    position: 'relative',
  },
  avatar: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#6D28D9',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    color: '#fff',
    fontSize: 22,
    fontWeight: '600',
  },
  onlineIndicator: {
    position: 'absolute',
    bottom: 2,
    right: 2,
    width: 14,
    height: 14,
    borderRadius: 7,
    backgroundColor: '#22c55e',
    borderWidth: 2,
    borderColor: '#fff',
  },
  info: {
    flex: 1,
    marginLeft: 12,
  },
  name: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 4,
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  stars: {
    color: '#f59e0b',
    fontSize: 14,
  },
  ratingText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#f59e0b',
    marginLeft: 4,
  },
  ordersText: {
    fontSize: 12,
    color: '#9ca3af',
    marginLeft: 4,
  },
  priceContainer: {
    alignItems: 'flex-end',
  },
  priceLabel: {
    fontSize: 11,
    color: '#9ca3af',
  },
  price: {
    fontSize: 18,
    fontWeight: '700',
    color: '#6D28D9',
  },
  specialties: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginTop: 12,
    gap: 8,
  },
  specialtyTag: {
    backgroundColor: '#f3f4f6',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  specialtyText: {
    fontSize: 12,
    color: '#6b7280',
  },
  description: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
    lineHeight: 20,
  },
  footer: {
    marginTop: 12,
    flexDirection: 'row',
    alignItems: 'center',
  },
  responseTime: {
    fontSize: 12,
    color: '#9ca3af',
  },
  compactCard: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    borderRadius: 10,
    padding: 12,
    marginBottom: 8,
  },
  compactAvatar: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#6D28D9',
    justifyContent: 'center',
    alignItems: 'center',
    position: 'relative',
  },
  onlineDot: {
    position: 'absolute',
    bottom: 0,
    right: 0,
    width: 10,
    height: 10,
    borderRadius: 5,
    backgroundColor: '#22c55e',
    borderWidth: 2,
    borderColor: '#fff',
  },
  compactInfo: {
    flex: 1,
    marginLeft: 10,
  },
  compactName: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1f2937',
  },
  compactSpecialty: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  compactPrice: {
    fontSize: 15,
    fontWeight: '600',
    color: '#6D28D9',
  },
});
