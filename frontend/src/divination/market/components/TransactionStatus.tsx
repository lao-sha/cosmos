// frontend/src/divination/market/components/TransactionStatus.tsx

import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  Modal,
  ActivityIndicator,
  TouchableOpacity,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../theme';
import type { TransactionState } from '../hooks/useChainTransaction';

interface TransactionStatusProps {
  visible: boolean;
  state: TransactionState;
  onClose?: () => void;
  onRetry?: () => void;
  successMessage?: string;
  errorMessage?: string;
}

export const TransactionStatus: React.FC<TransactionStatusProps> = ({
  visible,
  state,
  onClose,
  onRetry,
  successMessage = '交易已确认',
  errorMessage,
}) => {
  const getStatusContent = () => {
    switch (state.status) {
      case 'signing':
        return {
          icon: null,
          title: '签名中',
          desc: '请在钱包中确认签名...',
          showLoader: true,
        };
      case 'broadcasting':
        return {
          icon: null,
          title: '广播中',
          desc: '正在将交易广播到网络...',
          showLoader: true,
        };
      case 'pending':
        return {
          icon: null,
          title: '等待确认',
          desc: '交易已提交，等待区块确认...',
          showLoader: true,
          extra: state.blockHash ? `区块: ${state.blockHash.slice(0, 10)}...` : undefined,
        };
      case 'success':
        return {
          icon: 'checkmark-circle' as const,
          iconColor: THEME.success,
          title: '交易成功',
          desc: successMessage,
          showLoader: false,
          extra: state.txHash ? `交易: ${state.txHash.slice(0, 10)}...` : undefined,
        };
      case 'error':
        return {
          icon: 'close-circle' as const,
          iconColor: THEME.error,
          title: '交易失败',
          desc: errorMessage || state.error || '交易执行失败',
          showLoader: false,
        };
      default:
        return null;
    }
  };

  const content = getStatusContent();
  if (!content) return null;

  const showCloseButton = state.status === 'success' || state.status === 'error';

  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      onRequestClose={onClose}
    >
      <View style={styles.overlay}>
        <View style={styles.container}>
          {content.showLoader ? (
            <ActivityIndicator size="large" color={THEME.primary} style={styles.loader} />
          ) : content.icon ? (
            <Ionicons
              name={content.icon}
              size={48}
              color={content.iconColor}
              style={styles.icon}
            />
          ) : null}

          <Text style={styles.title}>{content.title}</Text>
          <Text style={styles.desc}>{content.desc}</Text>

          {content.extra && (
            <Text style={styles.extra}>{content.extra}</Text>
          )}

          {showCloseButton && (
            <View style={styles.buttons}>
              {state.status === 'error' && onRetry && (
                <TouchableOpacity
                  style={[styles.button, styles.retryButton]}
                  onPress={onRetry}
                >
                  <Text style={styles.retryButtonText}>重试</Text>
                </TouchableOpacity>
              )}
              <TouchableOpacity
                style={[styles.button, styles.closeButton]}
                onPress={onClose}
              >
                <Text style={styles.closeButtonText}>
                  {state.status === 'success' ? '完成' : '关闭'}
                </Text>
              </TouchableOpacity>
            </View>
          )}
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
  },
  container: {
    backgroundColor: THEME.card,
    borderRadius: 16,
    padding: 24,
    width: '80%',
    maxWidth: 320,
    alignItems: 'center',
  },
  loader: {
    marginBottom: 16,
  },
  icon: {
    marginBottom: 16,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 8,
  },
  desc: {
    fontSize: 14,
    color: THEME.textSecondary,
    textAlign: 'center',
    lineHeight: 20,
  },
  extra: {
    fontSize: 12,
    color: THEME.textTertiary,
    marginTop: 8,
    fontFamily: 'monospace',
  },
  buttons: {
    flexDirection: 'row',
    marginTop: 20,
    gap: 12,
  },
  button: {
    paddingHorizontal: 24,
    paddingVertical: 10,
    borderRadius: 8,
    minWidth: 80,
    alignItems: 'center',
  },
  closeButton: {
    backgroundColor: THEME.primary,
  },
  closeButtonText: {
    color: THEME.textInverse,
    fontSize: 14,
    fontWeight: '500',
  },
  retryButton: {
    backgroundColor: THEME.background,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  retryButtonText: {
    color: THEME.text,
    fontSize: 14,
    fontWeight: '500',
  },
});
