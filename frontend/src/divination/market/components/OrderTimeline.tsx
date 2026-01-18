// frontend/src/divination/market/components/OrderTimeline.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { Order } from '../types/market.types';
import { THEME } from '../theme';
import { formatDateTime } from '../utils/market.utils';

interface TimelineItem {
  status: string;
  label: string;
  time?: number;
  icon: string;
  color: string;
}

interface OrderTimelineProps {
  order: Order;
}

export const OrderTimeline: React.FC<OrderTimelineProps> = ({ order }) => {
  const getTimelineItems = (): TimelineItem[] => {
    const items: TimelineItem[] = [
      {
        status: 'created',
        label: '订单创建',
        time: order.createdAt,
        icon: 'add-circle-outline',
        color: THEME.success,
      },
    ];

    // 根据状态添加时间点
    if (order.status === 'PendingPayment') {
      items.push({
        status: 'pending_payment',
        label: '等待支付',
        icon: 'card-outline',
        color: THEME.warning,
      });
    } else {
      items.push({
        status: 'paid',
        label: '已支付',
        time: order.createdAt, // 实际应该是支付时间
        icon: 'checkmark-circle-outline',
        color: THEME.success,
      });

      if (order.status === 'Paid') {
        items.push({
          status: 'waiting_accept',
          label: '等待接单',
          icon: 'hourglass-outline',
          color: THEME.warning,
        });
      } else if (order.acceptedAt) {
        items.push({
          status: 'accepted',
          label: '已接单',
          time: order.acceptedAt,
          icon: 'hand-left-outline',
          color: THEME.success,
        });

        if (order.status === 'Accepted') {
          items.push({
            status: 'in_progress',
            label: '解读中',
            icon: 'create-outline',
            color: THEME.info,
          });
        } else if (order.completedAt) {
          items.push({
            status: 'completed',
            label: '已完成',
            time: order.completedAt,
            icon: 'document-text-outline',
            color: THEME.success,
          });

          if (order.status === 'Reviewed') {
            items.push({
              status: 'reviewed',
              label: '已评价',
              icon: 'star-outline',
              color: THEME.success,
            });
          }
        }
      }
    }

    if (order.status === 'Cancelled') {
      items.push({
        status: 'cancelled',
        label: '已取消',
        icon: 'close-circle-outline',
        color: THEME.textTertiary,
      });
    }

    return items;
  };

  const items = getTimelineItems();

  return (
    <View style={styles.container}>
      {items.map((item, index) => {
        const isLast = index === items.length - 1;
        const isCompleted = !!item.time;
        const isCurrent = !item.time && index === items.length - 1;

        return (
          <View key={item.status} style={styles.item}>
            {/* 左侧时间线 */}
            <View style={styles.timeline}>
              <View
                style={[
                  styles.dot,
                  {
                    backgroundColor: isCompleted ? item.color : THEME.border,
                    borderColor: isCurrent ? item.color : 'transparent',
                  },
                  isCurrent && styles.currentDot,
                ]}
              >
                {isCompleted && (
                  <Ionicons name="checkmark" size={10} color={THEME.textInverse} />
                )}
              </View>
              {!isLast && (
                <View
                  style={[
                    styles.line,
                    { backgroundColor: isCompleted ? item.color : THEME.border },
                  ]}
                />
              )}
            </View>

            {/* 右侧内容 */}
            <View style={styles.content}>
              <View style={styles.labelRow}>
                <Ionicons
                  name={item.icon as any}
                  size={16}
                  color={isCompleted || isCurrent ? item.color : THEME.textTertiary}
                />
                <Text
                  style={[
                    styles.label,
                    { color: isCompleted || isCurrent ? THEME.text : THEME.textTertiary },
                  ]}
                >
                  {item.label}
                </Text>
              </View>
              {item.time && (
                <Text style={styles.time}>{formatDateTime(item.time)}</Text>
              )}
            </View>
          </View>
        );
      })}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {},
  item: {
    flexDirection: 'row',
  },
  timeline: {
    width: 24,
    alignItems: 'center',
  },
  dot: {
    width: 18,
    height: 18,
    borderRadius: 9,
    justifyContent: 'center',
    alignItems: 'center',
  },
  currentDot: {
    borderWidth: 2,
    backgroundColor: 'transparent',
  },
  line: {
    width: 2,
    flex: 1,
    minHeight: 24,
    marginVertical: 4,
  },
  content: {
    flex: 1,
    paddingLeft: 10,
    paddingBottom: 20,
  },
  labelRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
  },
  time: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 4,
    marginLeft: 22,
  },
});
