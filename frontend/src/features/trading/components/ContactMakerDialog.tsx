/**
 * 联系做市商对话框
 */

import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  Modal,
  TouchableOpacity,
  Linking,
  Alert,
  Clipboard,
} from 'react-native';
import type { Maker } from '@/stores/trading.store';

interface ContactMakerDialogProps {
  visible: boolean;
  maker: Maker | null;
  orderId?: number;
  onClose: () => void;
}

export const ContactMakerDialog: React.FC<ContactMakerDialogProps> = ({
  visible,
  maker,
  orderId,
  onClose,
}) => {
  if (!maker) return null;

  const handleCopyWechat = () => {
    Clipboard.setString(maker.wechatId);
    Alert.alert('成功', '微信号已复制到剪贴板');
  };

  const handleOpenWechat = async () => {
    // 尝试打开微信
    const wechatUrl = 'weixin://';
    const canOpen = await Linking.canOpenURL(wechatUrl);

    if (canOpen) {
      await Linking.openURL(wechatUrl);
    } else {
      Alert.alert('提示', '请手动打开微信添加做市商');
    }
  };

  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      onRequestClose={onClose}
    >
      <View style={styles.overlay}>
        <View style={styles.container}>
          <Text style={styles.title}>联系做市商</Text>

          {/* 做市商信息 */}
          <View style={styles.makerInfo}>
            <Text style={styles.makerName}>👤 {maker.maskedFullName}</Text>
            <View style={styles.ratingRow}>
              <Text style={styles.rating}>⭐ {maker.rating.toFixed(1)}</Text>
              <Text style={styles.usersServed}>{maker.usersServed} 单</Text>
            </View>
          </View>

          {/* 订单号 */}
          {orderId && (
            <View style={styles.orderInfo}>
              <Text style={styles.orderLabel}>订单号</Text>
              <Text style={styles.orderValue}>#{orderId}</Text>
            </View>
          )}

          {/* 微信联系方式 */}
          <View style={styles.contactSection}>
            <Text style={styles.contactLabel}>微信号</Text>
            <View style={styles.wechatRow}>
              <Text style={styles.wechatId}>{maker.wechatId}</Text>
              <TouchableOpacity
                style={styles.copyButton}
                onPress={handleCopyWechat}
              >
                <Text style={styles.copyButtonText}>复制</Text>
              </TouchableOpacity>
            </View>
          </View>

          {/* 提示信息 */}
          <View style={styles.tipCard}>
            <Text style={styles.tipTitle}>💡 联系提示</Text>
            <Text style={styles.tipText}>• 添加微信时请备注订单号</Text>
            <Text style={styles.tipText}>• 请保持礼貌，耐心沟通</Text>
            <Text style={styles.tipText}>• 如遇问题可申请平台仲裁</Text>
          </View>

          {/* 操作按钮 */}
          <View style={styles.actions}>
            <TouchableOpacity
              style={styles.wechatButton}
              onPress={handleOpenWechat}
            >
              <Text style={styles.wechatButtonText}>打开微信</Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={styles.closeButton}
              onPress={onClose}
            >
              <Text style={styles.closeButtonText}>关闭</Text>
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
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
    textAlign: 'center',
    marginBottom: 20,
  },
  makerInfo: {
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  makerName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  rating: {
    fontSize: 14,
    color: '#666666',
    marginRight: 12,
  },
  usersServed: {
    fontSize: 14,
    color: '#666666',
  },
  orderInfo: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
    marginBottom: 16,
  },
  orderLabel: {
    fontSize: 14,
    color: '#666666',
  },
  orderValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
  },
  contactSection: {
    marginBottom: 16,
  },
  contactLabel: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 8,
  },
  wechatRow: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    padding: 12,
  },
  wechatId: {
    flex: 1,
    fontSize: 16,
    fontWeight: '500',
    color: '#000000',
  },
  copyButton: {
    backgroundColor: '#B2955D',
    borderRadius: 6,
    paddingHorizontal: 12,
    paddingVertical: 6,
  },
  copyButtonText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  tipCard: {
    backgroundColor: '#FFF9F0',
    borderRadius: 12,
    padding: 16,
    marginBottom: 20,
  },
  tipTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  tipText: {
    fontSize: 13,
    color: '#666666',
    marginBottom: 4,
  },
  actions: {
    gap: 12,
  },
  wechatButton: {
    backgroundColor: '#07C160',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  wechatButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  closeButton: {
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  closeButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#666666',
  },
});
