/**
 * 占卜师订单卡片组件
 */

import React from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { Order, OrderStatus, ORDER_STATUS_CONFIG, DIVINATION_TYPE_CONFIG } from '../types';

const THEME_COLOR = '#B2955D';

interface DivinerOrderCardProps {
  order: Order;
  onAccept?: () => void;
  onReject?: () => void;
  onSubmitAnswer?: () => void;
  onViewDetail?: () => void;
}

export function DivinerOrderCard({ order, onAccept, onReject, onSubmitAnswer, onViewDetail }: DivinerOrderCardProps) {
  const statusConfig = ORDER_STATUS_CONFIG[order.status];
  const divType = DIVINATION_TYPE_CONFIG[order.divinationType];
  const priceDisplay = (Number(order.totalAmount) / 1e10).toFixed(2);
  const earningsDisplay = (Number(order.providerEarnings) / 1e10).toFixed(2);

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return `${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`;
  };

  const showActions = order.status === OrderStatus.Paid || order.status === OrderStatus.Accepted;

  return (
    <Pressable style={styles.container} onPress={onViewDetail}>
      <View style={styles.header}>
        <View style={styles.orderInfo}>
          <Text style={styles.orderId}>订单 #{order.id}</Text>
          <View style={[styles.statusBadge, { backgroundColor: `${statusConfig.color}20` }]}>
            <Text style={[styles.statusText, { color: statusConfig.color }]}>{statusConfig.label}</Text>
          </View>
        </View>
        <Text style={styles.time}>{formatTime(order.createdAt)}</Text>
      </View>

      <View style={styles.content}>
        <View style={styles.typeTag}>
          <Text style={styles.typeText}>{divType?.icon} {divType?.label}</Text>
        </View>
        {order.isUrgent && (
          <View style={styles.urgentTag}>
            <Text style={styles.urgentText}>⚡ 加急</Text>
          </View>
        )}
      </View>

      <View style={styles.priceRow}>
        <View style={styles.priceItem}>
          <Text style={styles.priceLabel}>订单金额</Text>
          <Text style={styles.priceValue}>{priceDisplay} DUST</Text>
        </View>
        <View style={styles.priceItem}>
          <Text style={styles.priceLabel}>预计收益</Text>
          <Text style={styles.earningsValue}>{earningsDisplay} DUST</Text>
        </View>
      </View>

      {order.followUpsTotal > 0 && (
        <View style={styles.followUpInfo}>
          <Text style={styles.followUpText}>
            追问：{order.followUpsUsed}/{order.followUpsTotal}
          </Text>
        </View>
      )}

      {showActions && (
        <View style={styles.actions}>
          {order.status === OrderStatus.Paid && (
            <>
              <Pressable style={styles.rejectBtn} onPress={onReject}>
                <Text style={styles.rejectBtnText}>拒绝</Text>
              </Pressable>
              <Pressable style={styles.acceptBtn} onPress={onAccept}>
                <Text style={styles.acceptBtnText}>接单</Text>
              </Pressable>
            </>
          )}
          {order.status === OrderStatus.Accepted && (
            <Pressable style={styles.submitBtn} onPress={onSubmitAnswer}>
              <Text style={styles.submitBtnText}>提交解读</Text>
            </Pressable>
          )}
        </View>
      )}
    </Pressable>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  orderInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  orderId: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000',
  },
  statusBadge: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
  },
  time: {
    fontSize: 12,
    color: '#999',
  },
  content: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 12,
  },
  typeTag: {
    backgroundColor: '#F5F5F7',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  typeText: {
    fontSize: 12,
    color: '#666',
  },
  urgentTag: {
    backgroundColor: '#FFF9F0',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  urgentText: {
    fontSize: 12,
    color: '#FF9500',
  },
  priceRow: {
    flexDirection: 'row',
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
    paddingTop: 12,
  },
  priceItem: {
    flex: 1,
  },
  priceLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 4,
  },
  priceValue: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  earningsValue: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '600',
  },
  followUpInfo: {
    marginTop: 8,
  },
  followUpText: {
    fontSize: 12,
    color: '#666',
  },
  actions: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  rejectBtn: {
    flex: 1,
    height: 40,
    borderWidth: 1,
    borderColor: '#FF3B30',
    borderRadius: 4,
    justifyContent: 'center',
    alignItems: 'center',
  },
  rejectBtnText: {
    fontSize: 14,
    color: '#FF3B30',
  },
  acceptBtn: {
    flex: 1,
    height: 40,
    backgroundColor: THEME_COLOR,
    borderRadius: 4,
    justifyContent: 'center',
    alignItems: 'center',
  },
  acceptBtnText: {
    fontSize: 14,
    color: '#FFF',
    fontWeight: '500',
  },
  submitBtn: {
    flex: 1,
    height: 40,
    backgroundColor: THEME_COLOR,
    borderRadius: 4,
    justifyContent: 'center',
    alignItems: 'center',
  },
  submitBtnText: {
    fontSize: 14,
    color: '#FFF',
    fontWeight: '500',
  },
});
