/**
 * 服务套餐卡片组件
 */

import React from 'react';
import { View, Text, StyleSheet, Pressable, Switch } from 'react-native';
import { ServicePackage, DIVINATION_TYPE_CONFIG, SERVICE_TYPE_CONFIG } from '../types';

const THEME_COLOR = '#B2955D';

interface PackageCardProps {
  package: ServicePackage;
  onEdit?: () => void;
  onToggle?: (isActive: boolean) => void;
  onDelete?: () => void;
  onSelect?: () => void;
  editable?: boolean;
}

export function PackageCard({ package: pkg, onEdit, onToggle, onDelete, onSelect, editable = false }: PackageCardProps) {
  const divType = DIVINATION_TYPE_CONFIG[pkg.divinationType];
  const svcType = SERVICE_TYPE_CONFIG[pkg.serviceType];
  const priceDisplay = (Number(pkg.price) / 1e10).toFixed(2);

  return (
    <View style={[styles.container, !pkg.isActive && styles.containerInactive]}>
      <View style={styles.header}>
        <View style={styles.titleRow}>
          <Text style={styles.name}>{pkg.name}</Text>
          {editable && (
            <Switch
              value={pkg.isActive}
              onValueChange={onToggle}
              trackColor={{ false: '#E8E8E8', true: `${THEME_COLOR}80` }}
              thumbColor={pkg.isActive ? THEME_COLOR : '#FFF'}
            />
          )}
        </View>
        <View style={styles.tags}>
          <View style={styles.tag}>
            <Text style={styles.tagText}>{divType?.icon} {divType?.label}</Text>
          </View>
          <View style={styles.tag}>
            <Text style={styles.tagText}>{svcType?.icon} {svcType?.label}</Text>
          </View>
        </View>
      </View>

      <Text style={styles.description} numberOfLines={2}>{pkg.description}</Text>

      <View style={styles.infoRow}>
        <View style={styles.infoItem}>
          <Text style={styles.infoLabel}>价格</Text>
          <Text style={styles.priceValue}>{priceDisplay} DUST</Text>
        </View>
        <View style={styles.infoItem}>
          <Text style={styles.infoLabel}>追问次数</Text>
          <Text style={styles.infoValue}>{pkg.followUpCount} 次</Text>
        </View>
        <View style={styles.infoItem}>
          <Text style={styles.infoLabel}>销量</Text>
          <Text style={styles.infoValue}>{pkg.salesCount}</Text>
        </View>
      </View>

      {pkg.urgentAvailable && (
        <View style={styles.urgentTag}>
          <Text style={styles.urgentText}>⚡ 支持加急 +{pkg.urgentSurcharge / 100}%</Text>
        </View>
      )}

      {editable && (
        <View style={styles.actions}>
          <Pressable style={styles.editBtn} onPress={onEdit}>
            <Text style={styles.editBtnText}>编辑</Text>
          </Pressable>
          <Pressable style={styles.deleteBtn} onPress={onDelete}>
            <Text style={styles.deleteBtnText}>删除</Text>
          </Pressable>
        </View>
      )}

      {!editable && onSelect && (
        <Pressable style={styles.selectBtn} onPress={onSelect}>
          <Text style={styles.selectBtnText}>选择此套餐</Text>
        </Pressable>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  containerInactive: {
    opacity: 0.6,
  },
  header: {
    marginBottom: 12,
  },
  titleRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  name: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000',
    flex: 1,
  },
  tags: {
    flexDirection: 'row',
    gap: 8,
  },
  tag: {
    backgroundColor: '#F5F5F7',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  tagText: {
    fontSize: 12,
    color: '#666',
  },
  description: {
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
    marginBottom: 12,
  },
  infoRow: {
    flexDirection: 'row',
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
    paddingTop: 12,
  },
  infoItem: {
    flex: 1,
    alignItems: 'center',
  },
  infoLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 4,
  },
  infoValue: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  priceValue: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '600',
  },
  urgentTag: {
    marginTop: 12,
    backgroundColor: '#FFF9F0',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
    alignSelf: 'flex-start',
  },
  urgentText: {
    fontSize: 12,
    color: '#FF9500',
  },
  actions: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  editBtn: {
    flex: 1,
    height: 36,
    borderWidth: 1,
    borderColor: THEME_COLOR,
    borderRadius: 4,
    justifyContent: 'center',
    alignItems: 'center',
  },
  editBtnText: {
    fontSize: 14,
    color: THEME_COLOR,
  },
  deleteBtn: {
    flex: 1,
    height: 36,
    borderWidth: 1,
    borderColor: '#FF3B30',
    borderRadius: 4,
    justifyContent: 'center',
    alignItems: 'center',
  },
  deleteBtnText: {
    fontSize: 14,
    color: '#FF3B30',
  },
  selectBtn: {
    height: 44,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 12,
  },
  selectBtnText: {
    fontSize: 16,
    color: '#FFF',
    fontWeight: '600',
  },
});
