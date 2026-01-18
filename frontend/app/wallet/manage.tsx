/**
 * 星尘玄鉴 - 钱包管理页面
 * 支持多钱包管理
 * 主题色：金棕色 #B2955D
 */

import { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  Alert,
  ActivityIndicator,
  RefreshControl,
  Platform,
  Modal,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWalletStore } from '@/stores';
import { WalletSwitcher } from '@/components/WalletSwitcher';
import { BottomNavBar } from '@/components/BottomNavBar';
import * as Clipboard from 'expo-clipboard';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_BG = '#f5f5f5';

export default function WalletManagePage() {
  const router = useRouter();
  const {
    address,
    accounts,
    loadingAccounts,
    loadAllAccounts,
    switchWallet,
    deleteWalletByAddress,
  } = useWalletStore();

  const [refreshing, setRefreshing] = useState(false);
  const [switcherVisible, setSwitcherVisible] = useState(false);
  const [deleteModalVisible, setDeleteModalVisible] = useState(false);
  const [walletToDelete, setWalletToDelete] = useState<string | null>(null);

  useEffect(() => {
    loadAllAccounts();
  }, []);

  // 获取当前钱包信息
  const currentWallet = accounts.find(acc => acc.address === address);
  const walletAlias = currentWallet?.alias || `钱包 ${address?.slice(0, 6) || ''}`;
  const balance = currentWallet?.balance || '0.0000';

  // 刷新
  const handleRefresh = async () => {
    setRefreshing(true);
    await loadAllAccounts();
    setRefreshing(false);
  };

  // 复制地址
  const handleCopyAddress = async (addr?: string) => {
    const addrToCopy = addr || address;
    if (addrToCopy) {
      await Clipboard.setStringAsync(addrToCopy);
      if (Platform.OS === 'web') {
        // Web 平台使用自定义提示
      } else {
        Alert.alert('成功', '地址已复制到剪贴板');
      }
    }
  };

  // 格式化地址显示
  const formatAddress = (addr: string, long = false) => {
    if (!addr) return '未连接';
    if (long) {
      return `${addr.slice(0, 20)}...${addr.slice(-10)}`;
    }
    return `${addr.slice(0, 8)}...${addr.slice(-8)}`;
  };

  // 切换到某个钱包
  const handleSwitchTo = async (walletAddress: string) => {
    if (walletAddress === address) return;
    try {
      await switchWallet(walletAddress);
    } catch (error) {
      console.error('切换钱包失败:', error);
    }
  };

  // 确认删除钱包
  const handleConfirmDelete = async () => {
    if (!walletToDelete) return;
    try {
      await deleteWalletByAddress(walletToDelete);
      setDeleteModalVisible(false);
      setWalletToDelete(null);
    } catch (error) {
      console.error('删除钱包失败:', error);
    }
  };

  // 显示删除确认
  const showDeleteConfirm = (walletAddress: string) => {
    setWalletToDelete(walletAddress);
    setDeleteModalVisible(true);
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>我的钱包</Text>
        <View style={styles.navRight}>
          <Pressable
            style={styles.switchBtn}
            onPress={() => setSwitcherVisible(true)}
          >
            <Ionicons name="swap-horizontal" size={16} color="#1890ff" />
            <Text style={styles.switchBtnText}>切换</Text>
          </Pressable>
          <View style={styles.networkBadge}>
            <Text style={styles.networkText}>Mainnet</Text>
          </View>
        </View>
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.contentContainer}
        showsVerticalScrollIndicator={false}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={handleRefresh}
            tintColor={THEME_COLOR}
          />
        }
      >
        {/* 钱包卡片 */}
        <View style={styles.walletCard}>
          {/* 设置图标 */}
          <Pressable
            style={styles.settingsBtn}
            onPress={() => Alert.alert('提示', '钱包设置功能即将上线')}
          >
            <Ionicons name="settings-outline" size={20} color="#FFF" />
          </Pressable>

          {/* 钱包名称和余额 */}
          <View style={styles.walletInfo}>
            <Text style={styles.walletAlias}>{walletAlias}</Text>
            <View style={styles.balanceRow}>
              <Text style={styles.balanceValue}>{balance}</Text>
              <Text style={styles.balanceUnit}>DUST</Text>
            </View>
          </View>

          {/* 钱包地址 */}
          <Pressable style={styles.addressRow} onPress={() => handleCopyAddress()}>
            <Text style={styles.addressText}>{formatAddress(address || '', true)}</Text>
            <Ionicons name="copy-outline" size={16} color="rgba(255,255,255,0.8)" />
          </Pressable>
        </View>

        {/* 创建/导入钱包按钮 */}
        <View style={styles.actionButtons}>
          <Pressable style={styles.actionButton} onPress={() => router.push('/auth/create')}>
            <View style={[styles.actionIcon, { backgroundColor: '#e6f7ff' }]}>
              <Ionicons name="add-circle-outline" size={24} color="#1890ff" />
            </View>
            <Text style={styles.actionText}>创建钱包</Text>
          </Pressable>

          <Pressable style={styles.actionButton} onPress={() => router.push('/auth/import')}>
            <View style={[styles.actionIcon, { backgroundColor: '#fff7e6' }]}>
              <Ionicons name="download-outline" size={24} color="#fa8c16" />
            </View>
            <Text style={styles.actionText}>导入钱包</Text>
          </Pressable>
        </View>

        {/* 资产列表标题 */}
        <View style={styles.assetHeader}>
          <Text style={styles.assetTitle}>DUST 资产</Text>
          <Pressable onPress={handleRefresh}>
            <Ionicons
              name="refresh-outline"
              size={20}
              color={loadingAccounts ? THEME_COLOR : '#8c8c8c'}
            />
          </Pressable>
        </View>

        {/* 账户资产列表 */}
        <View style={styles.assetList}>
          {loadingAccounts && accounts.length === 0 ? (
            <View style={styles.emptyState}>
              <ActivityIndicator size="large" color={THEME_COLOR} />
              <Text style={styles.emptyText}>加载中...</Text>
            </View>
          ) : accounts.length === 0 ? (
            <View style={styles.emptyState}>
              <Ionicons name="wallet-outline" size={48} color="#CCC" />
              <Text style={styles.emptyText}>暂无账户</Text>
            </View>
          ) : (
            accounts.map((acc, index) => (
              <Pressable
                key={acc.address}
                style={[
                  styles.assetItem,
                  acc.isCurrentAccount && styles.assetItemCurrent,
                  index < accounts.length - 1 && styles.assetItemBorder,
                ]}
                onPress={() => handleSwitchTo(acc.address)}
              >
                {/* 左侧：图标 + 账户信息 */}
                <View style={styles.assetLeft}>
                  <View style={styles.assetIcon}>
                    <Text style={styles.assetIconText}>🪙</Text>
                  </View>
                  <View style={styles.assetInfo}>
                    <View style={styles.assetNameRow}>
                      <Text style={styles.assetName}>{acc.alias}</Text>
                      {acc.isCurrentAccount && (
                        <View style={styles.currentBadge}>
                          <Text style={styles.currentBadgeText}>当前</Text>
                        </View>
                      )}
                    </View>
                    <Text style={styles.assetAddress}>{formatAddress(acc.address)}</Text>
                  </View>
                </View>

                {/* 右侧：余额 + 操作 */}
                <View style={styles.assetRight}>
                  <View style={styles.assetBalanceCol}>
                    <Text style={styles.assetBalance}>{acc.balance}</Text>
                    <Text style={styles.assetUnit}>DUST</Text>
                  </View>
                  {!acc.isCurrentAccount && (
                    <Pressable
                      style={styles.deleteBtn}
                      onPress={() => showDeleteConfirm(acc.address)}
                    >
                      <Ionicons name="trash-outline" size={18} color="#ff4d4f" />
                    </Pressable>
                  )}
                </View>
              </Pressable>
            ))
          )}
        </View>

        {/* 钱包数量提示 */}
        {accounts.length > 0 && (
          <View style={styles.walletCountHint}>
            <Ionicons name="information-circle-outline" size={16} color="#999" />
            <Text style={styles.walletCountText}>
              共 {accounts.length} 个钱包，点击可切换
            </Text>
          </View>
        )}

        {/* 功能列表 */}
        <View style={styles.menuSection}>
          <Text style={styles.menuSectionTitle}>钱包功能</Text>
          <View style={styles.menuList}>
            <Pressable style={styles.menuItem} onPress={() => router.push('/wallet/transfer')}>
              <View style={styles.menuLeft}>
                <View style={[styles.menuIcon, { backgroundColor: THEME_COLOR + '20' }]}>
                  <Ionicons name="send-outline" size={20} color={THEME_COLOR} />
                </View>
                <Text style={styles.menuTitle}>转账</Text>
              </View>
              <Ionicons name="chevron-forward" size={18} color="#bfbfbf" />
            </Pressable>

            <Pressable
              style={styles.menuItem}
              onPress={() => Alert.alert('收款', '收款功能即将上线')}
            >
              <View style={styles.menuLeft}>
                <View style={[styles.menuIcon, { backgroundColor: '#52c41a20' }]}>
                  <Ionicons name="qr-code-outline" size={20} color="#52c41a" />
                </View>
                <Text style={styles.menuTitle}>收款</Text>
              </View>
              <Ionicons name="chevron-forward" size={18} color="#bfbfbf" />
            </Pressable>

            <Pressable style={styles.menuItem} onPress={() => router.push('/wallet/transactions')}>
              <View style={styles.menuLeft}>
                <View style={[styles.menuIcon, { backgroundColor: '#1890ff20' }]}>
                  <Ionicons name="time-outline" size={20} color="#1890ff" />
                </View>
                <Text style={styles.menuTitle}>交易记录</Text>
              </View>
              <Ionicons name="chevron-forward" size={18} color="#bfbfbf" />
            </Pressable>

            <Pressable
              style={styles.menuItem}
              onPress={() => router.push('/wallet/export-mnemonic')}
            >
              <View style={styles.menuLeft}>
                <View style={[styles.menuIcon, { backgroundColor: '#fa8c1620' }]}>
                  <Ionicons name="key-outline" size={20} color="#fa8c16" />
                </View>
                <Text style={styles.menuTitle}>导出助记词</Text>
              </View>
              <Ionicons name="chevron-forward" size={18} color="#bfbfbf" />
            </Pressable>
          </View>
        </View>

        {/* 水印 */}
        <View style={styles.watermark}>
          <Text style={styles.watermarkText}>https://www.dustapps.net</Text>
        </View>
      </ScrollView>

      {/* 钱包切换弹窗 */}
      <WalletSwitcher
        visible={switcherVisible}
        onClose={() => setSwitcherVisible(false)}
        onCreateNew={() => router.push('/auth/create')}
        onImport={() => router.push('/auth/import')}
      />

      {/* 删除确认弹窗 */}
      <Modal visible={deleteModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalIcon}>
              <Ionicons name="warning" size={48} color="#ff4d4f" />
            </View>
            <Text style={styles.modalTitle}>删除钱包</Text>
            <Text style={styles.modalText}>
              确定要删除此钱包吗？{'\n'}此操作无法撤销，请确保已备份助记词。
            </Text>
            <View style={styles.modalButtons}>
              <Pressable
                style={styles.modalCancelBtn}
                onPress={() => {
                  setDeleteModalVisible(false);
                  setWalletToDelete(null);
                }}
              >
                <Text style={styles.modalCancelText}>取消</Text>
              </Pressable>
              <Pressable style={styles.modalDeleteBtn} onPress={handleConfirmDelete}>
                <Text style={styles.modalDeleteText}>删除</Text>
              </Pressable>
            </View>
          </View>
        </View>
      </Modal>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  content: {
    flex: 1,
  },
  contentContainer: {
    paddingBottom: Platform.select({ ios: 100, android: 76, default: 76 }),
  },
  navBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: Platform.select({ ios: 50, android: 40, default: 20 }),
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backButton: {
    padding: 4,
  },
  navTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  navRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  switchBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    backgroundColor: '#e6f7ff',
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: '#1890ff',
  },
  switchBtnText: {
    fontSize: 12,
    fontWeight: '500',
    color: '#1890ff',
  },
  networkBadge: {
    backgroundColor: '#f0f0f0',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 4,
  },
  networkText: {
    fontSize: 12,
    color: '#666',
  },
  // 钱包卡片
  walletCard: {
    margin: 16,
    padding: 20,
    borderRadius: 16,
    backgroundColor: THEME_COLOR,
    position: 'relative',
  },
  settingsBtn: {
    position: 'absolute',
    top: 16,
    right: 16,
    padding: 4,
  },
  walletInfo: {
    marginBottom: 16,
  },
  walletAlias: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
    marginBottom: 8,
  },
  balanceRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
  },
  balanceValue: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#FFF',
  },
  balanceUnit: {
    fontSize: 16,
    color: '#FFF',
    marginLeft: 8,
  },
  addressRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  addressText: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.9)',
  },
  // 操作按钮
  actionButtons: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    gap: 16,
    marginBottom: 24,
  },
  actionButton: {
    flex: 1,
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
    gap: 8,
  },
  actionIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    justifyContent: 'center',
    alignItems: 'center',
  },
  actionText: {
    fontSize: 14,
    color: '#262626',
  },
  // 资产列表
  assetHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    marginBottom: 12,
  },
  assetTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  assetList: {
    marginHorizontal: 16,
    backgroundColor: '#FFF',
    borderRadius: 12,
    overflow: 'hidden',
  },
  assetItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: 16,
  },
  assetItemCurrent: {
    backgroundColor: '#f0f7ff',
  },
  assetItemBorder: {
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  assetLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
    flex: 1,
  },
  assetIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
  },
  assetIconText: {
    fontSize: 20,
  },
  assetInfo: {
    flex: 1,
  },
  assetNameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 4,
  },
  assetName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  currentBadge: {
    backgroundColor: '#e6f7ff',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
    borderWidth: 1,
    borderColor: '#91d5ff',
  },
  currentBadgeText: {
    fontSize: 10,
    color: '#1890ff',
  },
  assetAddress: {
    fontSize: 12,
    color: '#999',
  },
  assetRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  assetBalanceCol: {
    alignItems: 'flex-end',
  },
  assetBalance: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  assetUnit: {
    fontSize: 12,
    color: '#999',
  },
  deleteBtn: {
    padding: 8,
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
  walletCountHint: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 6,
    paddingVertical: 12,
  },
  walletCountText: {
    fontSize: 12,
    color: '#999',
  },
  // 功能菜单
  menuSection: {
    marginTop: 8,
    paddingHorizontal: 16,
  },
  menuSectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
  },
  menuList: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    overflow: 'hidden',
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  menuLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  menuIcon: {
    width: 36,
    height: 36,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  menuTitle: {
    fontSize: 15,
    color: '#333',
  },
  // 水印
  watermark: {
    alignItems: 'center',
    paddingVertical: 24,
    marginTop: 20,
  },
  watermarkText: {
    fontSize: 12,
    color: '#999',
  },
  // 弹窗
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modalContent: {
    backgroundColor: '#FFF',
    borderRadius: 16,
    padding: 24,
    width: '100%',
    maxWidth: 320,
    alignItems: 'center',
  },
  modalIcon: {
    marginBottom: 16,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  modalText: {
    fontSize: 14,
    color: '#666',
    textAlign: 'center',
    lineHeight: 22,
    marginBottom: 20,
  },
  modalButtons: {
    flexDirection: 'row',
    gap: 12,
    width: '100%',
  },
  modalCancelBtn: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#d9d9d9',
    alignItems: 'center',
  },
  modalCancelText: {
    fontSize: 15,
    color: '#666',
  },
  modalDeleteBtn: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    backgroundColor: '#ff4d4f',
    alignItems: 'center',
  },
  modalDeleteText: {
    fontSize: 15,
    color: '#FFF',
    fontWeight: '600',
  },
});
