/**
 * 星尘玄鉴 - 钱包切换器组件
 * 用于在多个钱包之间切换
 * 主题色：金棕色 #B2955D
 */

import { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  Modal,
  ScrollView,
  ActivityIndicator,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useWalletStore, type AccountAsset } from '@/stores';

const THEME_COLOR = '#B2955D';

interface WalletSwitcherProps {
  visible: boolean;
  onClose: () => void;
  onCreateNew?: () => void;
  onImport?: () => void;
}

export function WalletSwitcher({
  visible,
  onClose,
  onCreateNew,
  onImport,
}: WalletSwitcherProps) {
  const {
    accounts,
    loadingAccounts,
    address: currentAddress,
    switchWallet,
    loadAllAccounts,
  } = useWalletStore();

  const [switching, setSwitching] = useState(false);

  useEffect(() => {
    if (visible) {
      loadAllAccounts();
    }
  }, [visible]);

  const handleSwitch = async (address: string) => {
    if (address === currentAddress) {
      onClose();
      return;
    }

    try {
      setSwitching(true);
      await switchWallet(address);
      onClose();
    } catch (error) {
      console.error('切换钱包失败:', error);
    } finally {
      setSwitching(false);
    }
  };

  const formatAddress = (addr: string) => {
    return `${addr.slice(0, 8)}...${addr.slice(-8)}`;
  };

  return (
    <Modal visible={visible} transparent animationType="slide">
      <View style={styles.overlay}>
        <View style={styles.container}>
          {/* 头部 */}
          <View style={styles.header}>
            <View style={styles.headerLeft}>
              <Ionicons name="swap-horizontal" size={20} color={THEME_COLOR} />
              <Text style={styles.headerTitle}>切换钱包</Text>
            </View>
            <Pressable onPress={onClose} style={styles.closeBtn}>
              <Ionicons name="close" size={24} color="#666" />
            </Pressable>
          </View>

          {/* 钱包列表 */}
          <ScrollView style={styles.list} showsVerticalScrollIndicator={false}>
            {loadingAccounts ? (
              <View style={styles.loadingState}>
                <ActivityIndicator size="large" color={THEME_COLOR} />
                <Text style={styles.loadingText}>加载中...</Text>
              </View>
            ) : accounts.length === 0 ? (
              <View style={styles.emptyState}>
                <Ionicons name="wallet-outline" size={48} color="#CCC" />
                <Text style={styles.emptyText}>暂无钱包，请创建或导入</Text>
              </View>
            ) : (
              accounts.map((wallet) => {
                const isCurrent = wallet.address === currentAddress;
                return (
                  <Pressable
                    key={wallet.address}
                    style={[
                      styles.walletItem,
                      isCurrent && styles.walletItemCurrent,
                    ]}
                    onPress={() => handleSwitch(wallet.address)}
                    disabled={switching}
                  >
                    {/* 当前钱包标识 */}
                    {isCurrent && (
                      <View style={styles.currentBadge}>
                        <Ionicons name="checkmark-circle" size={20} color="#1890ff" />
                      </View>
                    )}

                    {/* 钱包图标 */}
                    <View style={styles.walletIcon}>
                      <Text style={styles.walletIconText}>🪙</Text>
                    </View>

                    {/* 钱包信息 */}
                    <View style={styles.walletInfo}>
                      <View style={styles.walletNameRow}>
                        <Text style={[
                          styles.walletName,
                          isCurrent && styles.walletNameCurrent,
                        ]}>
                          {wallet.alias}
                        </Text>
                        {isCurrent && (
                          <View style={styles.currentTag}>
                            <Text style={styles.currentTagText}>当前</Text>
                          </View>
                        )}
                      </View>
                      <Text style={styles.walletAddress}>
                        {formatAddress(wallet.address)}
                      </Text>
                      <Text style={styles.walletBalance}>
                        余额: {wallet.balance} DUST
                      </Text>
                    </View>
                  </Pressable>
                );
              })
            )}
          </ScrollView>

          {/* 底部操作 */}
          <View style={styles.footer}>
            <Pressable
              style={styles.footerBtn}
              onPress={() => {
                onClose();
                onCreateNew?.();
              }}
            >
              <Ionicons name="add-circle-outline" size={20} color="#1890ff" />
              <Text style={styles.footerBtnText}>创建新钱包</Text>
            </Pressable>

            <Pressable
              style={[styles.footerBtn, styles.footerBtnSecondary]}
              onPress={() => {
                onClose();
                onImport?.();
              }}
            >
              <Ionicons name="download-outline" size={20} color="#666" />
              <Text style={styles.footerBtnTextSecondary}>导入钱包</Text>
            </Pressable>
          </View>
        </View>
      </View>

      {/* 切换中遮罩 */}
      {switching && (
        <View style={styles.switchingOverlay}>
          <ActivityIndicator size="large" color="#FFF" />
          <Text style={styles.switchingText}>切换中...</Text>
        </View>
      )}
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
    backgroundColor: '#FFF',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    maxHeight: '80%',
    paddingBottom: 34,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  headerLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  closeBtn: {
    padding: 4,
  },
  list: {
    maxHeight: 400,
    paddingHorizontal: 16,
  },
  loadingState: {
    padding: 40,
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#999',
  },
  emptyState: {
    padding: 40,
    alignItems: 'center',
    gap: 12,
  },
  emptyText: {
    fontSize: 14,
    color: '#999',
  },
  walletItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    marginVertical: 6,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#F0F0F0',
    backgroundColor: '#FFF',
    position: 'relative',
  },
  walletItemCurrent: {
    borderColor: '#1890ff',
    borderWidth: 2,
    backgroundColor: '#e6f7ff',
  },
  currentBadge: {
    position: 'absolute',
    top: 12,
    right: 12,
  },
  walletIcon: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  walletIconText: {
    fontSize: 20,
  },
  walletInfo: {
    flex: 1,
  },
  walletNameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 4,
  },
  walletName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  walletNameCurrent: {
    color: '#1890ff',
  },
  currentTag: {
    backgroundColor: '#e6f7ff',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
    borderWidth: 1,
    borderColor: '#91d5ff',
  },
  currentTagText: {
    fontSize: 10,
    color: '#1890ff',
  },
  walletAddress: {
    fontSize: 12,
    color: '#999',
    marginBottom: 4,
  },
  walletBalance: {
    fontSize: 13,
    color: '#666',
  },
  footer: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  footerBtn: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 6,
    paddingVertical: 12,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#1890ff',
    backgroundColor: '#FFF',
  },
  footerBtnSecondary: {
    borderColor: '#d9d9d9',
  },
  footerBtnText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1890ff',
  },
  footerBtnTextSecondary: {
    fontSize: 14,
    fontWeight: '500',
    color: '#666',
  },
  switchingOverlay: {
    ...StyleSheet.absoluteFillObject,
    backgroundColor: 'rgba(0,0,0,0.7)',
    justifyContent: 'center',
    alignItems: 'center',
    gap: 12,
  },
  switchingText: {
    fontSize: 16,
    color: '#FFF',
  },
});
