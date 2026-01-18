/**
 * 做市商离线警告组件
 */

import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  Modal,
  TouchableOpacity,
} from 'react-native';
import type { Maker } from '@/stores/trading.store';

interface MakerOfflineWarningProps {
  visible: boolean;
  maker: Maker;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * 格式化最后活跃时间
 */
function formatLastActive(timestamp?: number): string {
  if (!timestamp) return '未知';

  const now = Date.now();
  const diff = now - timestamp;

  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 60) {
    return `${minutes} 分钟前`;
  } else if (hours < 24) {
    return `${hours} 小时前`;
  } else {
    return `${days} 天前`;
  }
}

export const MakerOfflineWarning: React.FC<MakerOfflineWarningProps> = ({
  visible,
  maker,
  onConfirm,
  onCancel,
}) => {
  const lastActiveText = formatLastActive(maker.avgResponseTime);

  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      onRequestClose={onCancel}
    >
      <View style={styles.overlay}>
        <View style={styles.container}>
          {/* 警告图标 */}
          <View style={styles.iconContainer}>
            <Text style={styles.icon}>⚠️</Text>
          </View>

          {/* 标题 */}
          <Text style={styles.title}>做市商当前离线</Text>

          {/* 描述 */}
          <Text style={styles.description}>
            该做市商最后活跃于 {lastActiveText}，可能无法及时处理您的订单。
          </Text>

          {/* 做市商信息 */}
          <View style={styles.makerInfo}>
            <View style={styles.makerRow}>
              <Text style={styles.makerLabel}>做市商</Text>
              <Text style={styles.makerValue}>{maker.maskedFullName}</Text>
            </View>
            <View style={styles.makerRow}>
              <Text style={styles.makerLabel}>状态</Text>
              <View style={styles.statusBadge}>
                <View style={styles.statusDot} />
                <Text style={styles.statusText}>离线</Text>
              </View>
            </View>
            <View style={styles.makerRow}>
              <Text style={styles.makerLabel}>评分</Text>
              <Text style={styles.makerValue}>⭐ {maker.rating.toFixed(1)}</Text>
            </View>
          </View>

          {/* 建议 */}
          <View style={styles.suggestionCard}>
            <Text style={styles.suggestionText}>
              建议选择在线的做市商以获得更快的交易体验
            </Text>
          </View>

          {/* 操作按钮 */}
          <View style={styles.actions}>
            <TouchableOpacity
              style={styles.cancelButton}
              onPress={onCancel}
            >
              <Text style={styles.cancelButtonText}>选择其他做市商</Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={styles.confirmButton}
              onPress={onConfirm}
            >
              <Text style={styles.confirmButtonText}>仍然选择</Text>
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </Modal>
  );
};

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  container: {
    backgroundColor: '#FFFFFF',
    borderRadius: 16,
    padding: 24,
    width: '100%',
    maxWidth: 400,
  },
  iconContainer: {
    alignItems: 'center',
    marginBottom: 16,
  },
  icon: {
    fontSize: 48,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
    textAlign: 'center',
    marginBottom: 12,
  },
  description: {
    fontSize: 14,
    color: '#666666',
    textAlign: 'center',
    lineHeight: 20,
    marginBottom: 20,
  },
  makerInfo: {
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  makerRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 6,
  },
  makerLabel: {
    fontSize: 14,
    color: '#666666',
  },
  makerValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#000000',
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#FF9500',
    marginRight: 6,
  },
  statusText: {
    fontSize: 14,
    color: '#FF9500',
    fontWeight: '500',
  },
  suggestionCard: {
    backgroundColor: '#FFF9F0',
    borderRadius: 8,
    padding: 12,
    marginBottom: 20,
  },
  suggestionText: {
    fontSize: 13,
    color: '#B2955D',
    textAlign: 'center',
  },
  actions: {
    flexDirection: 'row',
    gap: 12,
  },
  cancelButton: {
    flex: 1,
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  cancelButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  confirmButton: {
    flex: 1,
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  confirmButtonText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#666666',
  },
});
