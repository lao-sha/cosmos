// frontend/src/features/livestream/components/TicketModal.tsx

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Modal,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';

interface TicketModalProps {
  visible: boolean;
  onClose: () => void;
  ticketPrice: string;
  balance: string;
  onBuyTicket: () => Promise<void>;
}

export function TicketModal({
  visible,
  onClose,
  ticketPrice,
  balance,
  onBuyTicket,
}: TicketModalProps) {
  const [isBuying, setIsBuying] = useState(false);

  const hasEnoughBalance = Number(balance) >= Number(ticketPrice);

  const handleBuy = async () => {
    setIsBuying(true);
    try {
      await onBuyTicket();
    } catch (error) {
      console.error('购买门票失败:', error);
    } finally {
      setIsBuying(false);
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
          <TouchableOpacity style={styles.closeBtn} onPress={onClose}>
            <Ionicons name="close" size={24} color="#999" />
          </TouchableOpacity>

          <View style={styles.iconContainer}>
            <Ionicons name="ticket" size={48} color="#FFD700" />
          </View>

          <Text style={styles.title}>付费直播</Text>
          <Text style={styles.description}>
            这是一场付费直播，需要购买门票才能观看
          </Text>

          <View style={styles.priceContainer}>
            <Text style={styles.priceLabel}>门票价格</Text>
            <Text style={styles.priceValue}>{ticketPrice} DUST</Text>
          </View>

          <View style={styles.balanceContainer}>
            <Text style={styles.balanceLabel}>当前余额</Text>
            <Text
              style={[
                styles.balanceValue,
                !hasEnoughBalance && styles.balanceInsufficient,
              ]}
            >
              {balance} DUST
            </Text>
          </View>

          {!hasEnoughBalance && (
            <Text style={styles.warningText}>余额不足，请先充值</Text>
          )}

          <TouchableOpacity
            style={[styles.buyBtn, !hasEnoughBalance && styles.buyBtnDisabled]}
            onPress={handleBuy}
            disabled={!hasEnoughBalance || isBuying}
          >
            {isBuying ? (
              <ActivityIndicator color="#FFF" />
            ) : (
              <Text style={styles.buyBtnText}>
                {hasEnoughBalance ? '购买门票' : '余额不足'}
              </Text>
            )}
          </TouchableOpacity>

          <TouchableOpacity style={styles.cancelBtn} onPress={onClose}>
            <Text style={styles.cancelBtnText}>暂不观看</Text>
          </TouchableOpacity>
        </View>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.7)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  container: {
    width: '100%',
    maxWidth: 340,
    backgroundColor: '#1A1A2E',
    borderRadius: 20,
    padding: 24,
    alignItems: 'center',
  },
  closeBtn: {
    position: 'absolute',
    top: 12,
    right: 12,
    padding: 4,
  },
  iconContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: 'rgba(255, 215, 0, 0.1)',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 16,
  },
  title: {
    color: '#FFF',
    fontSize: 20,
    fontWeight: 'bold',
    marginBottom: 8,
  },
  description: {
    color: '#999',
    fontSize: 14,
    textAlign: 'center',
    marginBottom: 24,
  },
  priceContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    width: '100%',
    paddingVertical: 12,
    borderTopWidth: 1,
    borderTopColor: '#333',
  },
  priceLabel: {
    color: '#999',
    fontSize: 14,
  },
  priceValue: {
    color: '#FFD700',
    fontSize: 18,
    fontWeight: 'bold',
  },
  balanceContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    width: '100%',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#333',
    marginBottom: 16,
  },
  balanceLabel: {
    color: '#999',
    fontSize: 14,
  },
  balanceValue: {
    color: '#FFF',
    fontSize: 16,
  },
  balanceInsufficient: {
    color: '#FF4757',
  },
  warningText: {
    color: '#FF4757',
    fontSize: 12,
    marginBottom: 16,
  },
  buyBtn: {
    width: '100%',
    backgroundColor: '#FF4757',
    borderRadius: 24,
    paddingVertical: 14,
    alignItems: 'center',
    marginBottom: 12,
  },
  buyBtnDisabled: {
    backgroundColor: '#555',
  },
  buyBtnText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: 'bold',
  },
  cancelBtn: {
    padding: 12,
  },
  cancelBtnText: {
    color: '#999',
    fontSize: 14,
  },
});
