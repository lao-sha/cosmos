// frontend/src/features/livestream/components/GiftPanel.tsx

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  FlatList,
  Image,
  Modal,
  ActivityIndicator,
} from 'react-native';
import { useLivestreamStore } from '@/stores/livestream.store';
import type { Gift } from '../types';

const IPFS_GATEWAY = process.env.EXPO_PUBLIC_IPFS_GATEWAY || 'https://ipfs.io/ipfs/';

interface GiftPanelProps {
  visible: boolean;
  onClose: () => void;
  roomId: number;
  balance: string;
  onSendGift: (giftId: number, quantity: number) => Promise<void>;
}

export function GiftPanel({ visible, onClose, roomId, balance, onSendGift }: GiftPanelProps) {
  const { gifts } = useLivestreamStore();
  const [selectedGift, setSelectedGift] = useState<Gift | null>(null);
  const [quantity, setQuantity] = useState(1);
  const [isSending, setIsSending] = useState(false);

  const handleSend = async () => {
    if (!selectedGift) return;

    setIsSending(true);
    try {
      await onSendGift(selectedGift.id, quantity);
      onClose();
    } catch (error) {
      console.error('发送礼物失败:', error);
    } finally {
      setIsSending(false);
    }
  };

  const totalPrice = selectedGift
    ? (Number(selectedGift.price) * quantity).toString()
    : '0';

  const renderGift = ({ item }: { item: Gift }) => (
    <TouchableOpacity
      style={[
        styles.giftItem,
        selectedGift?.id === item.id && styles.giftItemSelected,
      ]}
      onPress={() => setSelectedGift(item)}
    >
      <Image
        source={{ uri: `${IPFS_GATEWAY}${item.iconCid}` }}
        style={styles.giftIcon}
      />
      <Text style={styles.giftName}>{item.name}</Text>
      <Text style={styles.giftPrice}>{item.price} DUST</Text>
    </TouchableOpacity>
  );

  return (
    <Modal
      visible={visible}
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <View style={styles.overlay}>
        <View style={styles.container}>
          <View style={styles.header}>
            <Text style={styles.title}>礼物</Text>
            <TouchableOpacity onPress={onClose}>
              <Text style={styles.closeBtn}>✕</Text>
            </TouchableOpacity>
          </View>

          <FlatList
            data={gifts}
            renderItem={renderGift}
            keyExtractor={(item) => item.id.toString()}
            numColumns={4}
            style={styles.giftList}
            ListEmptyComponent={
              <View style={styles.emptyContainer}>
                <Text style={styles.emptyText}>暂无礼物</Text>
              </View>
            }
          />

          <View style={styles.footer}>
            <View style={styles.quantityRow}>
              <Text style={styles.label}>数量:</Text>
              <TouchableOpacity
                style={styles.quantityBtn}
                onPress={() => setQuantity(Math.max(1, quantity - 1))}
              >
                <Text style={styles.quantityBtnText}>-</Text>
              </TouchableOpacity>
              <Text style={styles.quantity}>{quantity}</Text>
              <TouchableOpacity
                style={styles.quantityBtn}
                onPress={() => setQuantity(quantity + 1)}
              >
                <Text style={styles.quantityBtnText}>+</Text>
              </TouchableOpacity>
              <Text style={styles.balance}>余额: {balance} DUST</Text>
            </View>

            <TouchableOpacity
              style={[styles.sendBtn, !selectedGift && styles.sendBtnDisabled]}
              onPress={handleSend}
              disabled={!selectedGift || isSending}
            >
              {isSending ? (
                <ActivityIndicator color="#FFF" />
              ) : (
                <Text style={styles.sendBtnText}>
                  发送礼物 ({totalPrice} DUST)
                </Text>
              )}
            </TouchableOpacity>
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
    justifyContent: 'flex-end',
  },
  container: {
    backgroundColor: '#1A1A2E',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    paddingBottom: 34,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  title: {
    color: '#FFF',
    fontSize: 18,
    fontWeight: 'bold',
  },
  closeBtn: {
    color: '#999',
    fontSize: 20,
  },
  giftList: {
    padding: 8,
    maxHeight: 300,
  },
  giftItem: {
    flex: 1,
    alignItems: 'center',
    padding: 12,
    margin: 4,
    borderRadius: 8,
    backgroundColor: '#252540',
  },
  giftItemSelected: {
    backgroundColor: '#FF4757',
  },
  giftIcon: {
    width: 48,
    height: 48,
    marginBottom: 8,
  },
  giftName: {
    color: '#FFF',
    fontSize: 12,
    marginBottom: 4,
  },
  giftPrice: {
    color: '#FFD700',
    fontSize: 11,
  },
  footer: {
    padding: 16,
    borderTopWidth: 1,
    borderTopColor: '#333',
  },
  quantityRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  label: {
    color: '#FFF',
    marginRight: 12,
  },
  quantityBtn: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#333',
    justifyContent: 'center',
    alignItems: 'center',
  },
  quantityBtnText: {
    color: '#FFF',
    fontSize: 18,
  },
  quantity: {
    color: '#FFF',
    fontSize: 18,
    marginHorizontal: 16,
  },
  balance: {
    color: '#999',
    marginLeft: 'auto',
  },
  sendBtn: {
    backgroundColor: '#FF4757',
    borderRadius: 24,
    paddingVertical: 14,
    alignItems: 'center',
  },
  sendBtnDisabled: {
    backgroundColor: '#555',
  },
  sendBtnText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: 'bold',
  },
  emptyContainer: {
    padding: 40,
    alignItems: 'center',
  },
  emptyText: {
    color: '#666',
    fontSize: 14,
  },
});
