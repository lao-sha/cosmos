// frontend/src/divination/market/components/ProviderCard.tsx

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import { Provider } from '../types/market.types';
import { THEME, SHADOWS } from '../theme';
import { TierBadge } from './TierBadge';
import { RatingStars } from './RatingStars';
import { SpecialtyTags } from './SpecialtyTags';
import { Avatar } from './Avatar';
import { calculateAverageRating, getDivinationTypeNames } from '../utils/market.utils';
import { getIpfsUrl } from '../services/ipfs.service';

interface ProviderCardProps {
  provider: Provider;
  showPackagePreview?: boolean;
  minPrice?: bigint;
}

export const ProviderCard: React.FC<ProviderCardProps> = ({
  provider,
  showPackagePreview = true,
  minPrice,
}) => {
  const router = useRouter();
  const averageRating = calculateAverageRating(provider.totalRating, provider.ratingCount);
  const divinationTypes = getDivinationTypeNames(provider.supportedTypes);

  const handlePress = () => {
    router.push(`/market/provider/${provider.account}`);
  };

  return (
    <TouchableOpacity
      style={[styles.card, SHADOWS.medium]}
      onPress={handlePress}
      activeOpacity={0.7}
    >
      {/* 头部信息 */}
      <View style={styles.header}>
        <Avatar
          uri={provider.avatarCid ? getIpfsUrl(provider.avatarCid) : undefined}
          name={provider.name}
          size={56}
        />
        <View style={styles.headerInfo}>
          <View style={styles.nameRow}>
            <Text style={styles.name} numberOfLines={1}>
              {provider.name}
            </Text>
            <TierBadge tier={provider.tier} size="small" />
          </View>
          <View style={styles.ratingRow}>
            <RatingStars
              rating={averageRating}
              size={12}
              count={provider.ratingCount}
            />
            <Text style={styles.ordersText}>
              {provider.completedOrders}单
            </Text>
          </View>
        </View>
        {provider.acceptsUrgent && (
          <View style={styles.urgentBadge}>
            <Ionicons name="flash" size={12} color={THEME.warning} />
            <Text style={styles.urgentText}>可加急</Text>
          </View>
        )}
      </View>

      {/* 简介 */}
      <Text style={styles.bio} numberOfLines={2}>
        {provider.bio}
      </Text>

      {/* 擅长领域 */}
      <View style={styles.specialtiesContainer}>
        <SpecialtyTags specialties={provider.specialties} maxDisplay={4} size="small" />
      </View>

      {/* 占卜类型 */}
      <View style={styles.typesContainer}>
        <Text style={styles.typesLabel}>擅长：</Text>
        <Text style={styles.typesText} numberOfLines={1}>
          {divinationTypes.join('、')}
        </Text>
      </View>

      {/* 底部价格 */}
      {showPackagePreview && minPrice !== undefined && (
        <View style={styles.footer}>
          <Text style={styles.priceLabel}>起步价</Text>
          <Text style={styles.price}>
            {(Number(minPrice) / 1e12).toFixed(2)} DUST
          </Text>
        </View>
      )}
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: THEME.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  headerInfo: {
    flex: 1,
    marginLeft: 12,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  name: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.text,
    flexShrink: 1,
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 4,
    gap: 8,
  },
  ordersText: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  urgentBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.warning + '15',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
    gap: 2,
  },
  urgentText: {
    fontSize: 11,
    color: THEME.warning,
    fontWeight: '500',
  },
  bio: {
    fontSize: 13,
    color: THEME.textSecondary,
    lineHeight: 18,
    marginTop: 12,
  },
  specialtiesContainer: {
    marginTop: 12,
  },
  typesContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 10,
  },
  typesLabel: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  typesText: {
    fontSize: 12,
    color: THEME.textSecondary,
    flex: 1,
  },
  footer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: THEME.border,
  },
  priceLabel: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  price: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.primary,
  },
});
