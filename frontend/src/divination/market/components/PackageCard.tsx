// frontend/src/divination/market/components/PackageCard.tsx

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { ServicePackage } from '../types/market.types';
import { THEME, SHADOWS } from '../theme';
import { DivinationTypeBadge } from './DivinationTypeBadge';
import { PriceDisplay } from './PriceDisplay';
import {
  getServiceTypeName,
  getServiceTypeIcon,
} from '../utils/market.utils';

interface PackageCardProps {
  package_: ServicePackage;
  onSelect?: (pkg: ServicePackage) => void;
  showUrgentPrice?: boolean;
}

export const PackageCard: React.FC<PackageCardProps> = ({
  package_,
  onSelect,
  showUrgentPrice = true,
}) => {
  const serviceTypeName = getServiceTypeName(package_.serviceType);
  const serviceTypeIcon = getServiceTypeIcon(package_.serviceType);

  const handlePress = () => {
    onSelect?.(package_);
  };

  return (
    <TouchableOpacity
      style={[styles.card, SHADOWS.small]}
      onPress={handlePress}
      activeOpacity={0.7}
    >
      {/* 头部 */}
      <View style={styles.header}>
        <View style={styles.headerLeft}>
          <DivinationTypeBadge type={package_.divinationType} size="small" />
          <View style={styles.serviceType}>
            <Ionicons
              name={serviceTypeIcon as any}
              size={12}
              color={THEME.textSecondary}
            />
            <Text style={styles.serviceTypeText}>{serviceTypeName}</Text>
          </View>
        </View>
        {package_.urgentAvailable && showUrgentPrice && (
          <View style={styles.urgentBadge}>
            <Ionicons name="flash" size={10} color={THEME.warning} />
            <Text style={styles.urgentText}>可加急</Text>
          </View>
        )}
      </View>

      {/* 名称和描述 */}
      <Text style={styles.name}>{package_.name}</Text>
      <Text style={styles.description} numberOfLines={2}>
        {package_.description}
      </Text>

      {/* 详情 */}
      <View style={styles.details}>
        <View style={styles.detailItem}>
          <Ionicons name="time-outline" size={14} color={THEME.textTertiary} />
          <Text style={styles.detailText}>{package_.duration}分钟</Text>
        </View>
        <View style={styles.detailItem}>
          <Ionicons name="chatbubble-outline" size={14} color={THEME.textTertiary} />
          <Text style={styles.detailText}>{package_.followUpCount}次追问</Text>
        </View>
        <View style={styles.detailItem}>
          <Ionicons name="cart-outline" size={14} color={THEME.textTertiary} />
          <Text style={styles.detailText}>{package_.salesCount}单</Text>
        </View>
      </View>

      {/* 价格 */}
      <View style={styles.footer}>
        <PriceDisplay amount={package_.price} size="medium" />
        {package_.urgentAvailable && showUrgentPrice && (
          <Text style={styles.urgentPrice}>
            加急 +{package_.urgentSurcharge / 100}%
          </Text>
        )}
      </View>
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    marginBottom: 10,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 10,
  },
  headerLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  serviceType: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 3,
  },
  serviceTypeText: {
    fontSize: 11,
    color: THEME.textSecondary,
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
    fontSize: 10,
    color: THEME.warning,
    fontWeight: '500',
  },
  name: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 6,
  },
  description: {
    fontSize: 13,
    color: THEME.textSecondary,
    lineHeight: 18,
    marginBottom: 10,
  },
  details: {
    flexDirection: 'row',
    gap: 16,
    marginBottom: 12,
  },
  detailItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  detailText: {
    fontSize: 12,
    color: THEME.textTertiary,
  },
  footer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingTop: 10,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: THEME.border,
  },
  urgentPrice: {
    fontSize: 12,
    color: THEME.warning,
  },
});
