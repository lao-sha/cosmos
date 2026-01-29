import { TX_STATUS_TEXT, TxStatus } from '@/src/hooks/useTransaction';
import { ActivityIndicator, Modal, Pressable, StyleSheet, Text, View } from 'react-native';

interface TransactionModalProps {
  visible: boolean;
  onClose: () => void;
  title: string;
  description?: string;
  amount?: string;
  status: TxStatus | null;
  isLoading: boolean;
  onConfirm: () => void;
  confirmText?: string;
}

const STATUS_ICONS: Record<TxStatus, string> = {
  pending: '⏳',
  signing: '🔐',
  broadcasting: '📡',
  inBlock: '📦',
  finalized: '✅',
  failed: '❌',
};

export function TransactionModal({
  visible,
  onClose,
  title,
  description,
  amount,
  status,
  isLoading,
  onConfirm,
  confirmText = '确认交易',
}: TransactionModalProps) {
  const isProcessing = isLoading && status !== null;
  const isSuccess = status === 'finalized';
  const isFailed = status === 'failed';

  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      onRequestClose={onClose}
    >
      <View style={styles.overlay}>
        <View style={styles.modal}>
          <Text style={styles.title}>{title}</Text>
          
          {description && (
            <Text style={styles.description}>{description}</Text>
          )}
          
          {amount && (
            <View style={styles.amountBox}>
              <Text style={styles.amountLabel}>交易金额</Text>
              <Text style={styles.amount}>{amount}</Text>
            </View>
          )}

          {isProcessing && (
            <View style={styles.statusBox}>
              <Text style={styles.statusIcon}>
                {status ? STATUS_ICONS[status] : '⏳'}
              </Text>
              <Text style={styles.statusText}>
                {status ? TX_STATUS_TEXT[status] : '处理中...'}
              </Text>
              {!isSuccess && !isFailed && (
                <ActivityIndicator size="small" color="#6D28D9" style={styles.spinner} />
              )}
            </View>
          )}

          {isSuccess && (
            <View style={styles.successBox}>
              <Text style={styles.successIcon}>✅</Text>
              <Text style={styles.successText}>交易成功</Text>
            </View>
          )}

          {isFailed && (
            <View style={styles.failedBox}>
              <Text style={styles.failedIcon}>❌</Text>
              <Text style={styles.failedText}>交易失败</Text>
            </View>
          )}

          <View style={styles.buttons}>
            {!isProcessing && !isSuccess && (
              <>
                <Pressable 
                  style={styles.cancelButton} 
                  onPress={onClose}
                >
                  <Text style={styles.cancelText}>取消</Text>
                </Pressable>
                <Pressable 
                  style={[styles.confirmButton, isLoading && styles.buttonDisabled]}
                  onPress={onConfirm}
                  disabled={isLoading}
                >
                  <Text style={styles.confirmText}>{confirmText}</Text>
                </Pressable>
              </>
            )}
            
            {(isSuccess || isFailed) && (
              <Pressable 
                style={styles.closeButton} 
                onPress={onClose}
              >
                <Text style={styles.closeText}>完成</Text>
              </Pressable>
            )}
          </View>
        </View>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modal: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 24,
    width: '100%',
    maxWidth: 340,
  },
  title: {
    fontSize: 18,
    fontWeight: '700',
    color: '#1f2937',
    textAlign: 'center',
    marginBottom: 12,
  },
  description: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    lineHeight: 20,
    marginBottom: 16,
  },
  amountBox: {
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
    marginBottom: 16,
  },
  amountLabel: {
    fontSize: 13,
    color: '#9ca3af',
    marginBottom: 4,
  },
  amount: {
    fontSize: 28,
    fontWeight: '700',
    color: '#6D28D9',
  },
  statusBox: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#faf5ff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    gap: 8,
  },
  statusIcon: {
    fontSize: 20,
  },
  statusText: {
    fontSize: 15,
    color: '#6D28D9',
    fontWeight: '500',
  },
  spinner: {
    marginLeft: 4,
  },
  successBox: {
    alignItems: 'center',
    backgroundColor: '#dcfce7',
    borderRadius: 12,
    padding: 20,
    marginBottom: 16,
  },
  successIcon: {
    fontSize: 40,
    marginBottom: 8,
  },
  successText: {
    fontSize: 16,
    color: '#16a34a',
    fontWeight: '600',
  },
  failedBox: {
    alignItems: 'center',
    backgroundColor: '#fee2e2',
    borderRadius: 12,
    padding: 20,
    marginBottom: 16,
  },
  failedIcon: {
    fontSize: 40,
    marginBottom: 8,
  },
  failedText: {
    fontSize: 16,
    color: '#dc2626',
    fontWeight: '600',
  },
  buttons: {
    flexDirection: 'row',
    gap: 12,
  },
  cancelButton: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  cancelText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#6b7280',
  },
  confirmButton: {
    flex: 1,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  confirmText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  closeButton: {
    flex: 1,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  closeText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
});
