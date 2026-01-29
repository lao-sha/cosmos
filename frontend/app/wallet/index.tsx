import { AccountHeader, AccountSelector } from '@/src/components/AccountSelector';
import { AddressDisplay } from '@/src/components/AddressDisplay';
import { BalanceDisplay } from '@/src/components/BalanceDisplay';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useCurrentAccount, useWalletStore } from '@/src/stores/wallet';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import { Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';

export default function WalletScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { accounts } = useWalletStore();
  const currentAccount = useCurrentAccount();
  const [showAccountSelector, setShowAccountSelector] = useState(false);

  // 使用当前账户地址，如果没有则使用 auth store 的地址
  const displayAddress = currentAccount?.address || address || '';

  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>钱包</Text>
          <View style={styles.headerRight} />
        </View>

        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>👛</Text>
          <Text style={styles.emptyTitle}>还没有钱包</Text>
          <Text style={styles.emptyDesc}>创建或导入钱包以开始使用</Text>

          <View style={styles.emptyActions}>
            <Pressable
              style={styles.primaryButton}
              onPress={() => router.push('/wallet/create')}
            >
              <Text style={styles.primaryButtonText}>创建新钱包</Text>
            </Pressable>
            <Pressable
              style={styles.secondaryButton}
              onPress={() => router.push('/wallet/import')}
            >
              <Text style={styles.secondaryButtonText}>导入已有钱包</Text>
            </Pressable>
          </View>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        {accounts.length > 0 ? (
          <AccountHeader onPress={() => setShowAccountSelector(true)} />
        ) : (
          <Text style={styles.headerTitle}>钱包</Text>
        )}
        <View style={styles.headerRight} />
      </View>

      <AccountSelector
        visible={showAccountSelector}
        onClose={() => setShowAccountSelector(false)}
      />

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.balanceCard}>
          <View style={styles.networkStatus}>
            <View
              style={[
                styles.statusDot,
                { backgroundColor: isConnected ? '#22c55e' : '#ef4444' },
              ]}
            />
            <Text style={styles.networkText}>
              {isConnected ? '已连接' : '未连接'}
            </Text>
          </View>

          <BalanceDisplay />

          <View style={styles.addressSection}>
            <AddressDisplay address={displayAddress} size="small" />
          </View>
        </View>

        <View style={styles.quickActions}>
          <Pressable
            style={styles.actionItem}
            onPress={() => router.push('/wallet/transfer')}
          >
            <View style={[styles.actionIcon, { backgroundColor: '#dbeafe' }]}>
              <Text style={styles.actionEmoji}>↗️</Text>
            </View>
            <Text style={styles.actionLabel}>转账</Text>
          </Pressable>

          <Pressable
            style={styles.actionItem}
            onPress={() => router.push('/wallet/receive' as any)}
          >
            <View style={[styles.actionIcon, { backgroundColor: '#dcfce7' }]}>
              <Text style={styles.actionEmoji}>↙️</Text>
            </View>
            <Text style={styles.actionLabel}>收款</Text>
          </Pressable>

          <Pressable
            style={styles.actionItem}
            onPress={() => router.push('/wallet/history')}
          >
            <View style={[styles.actionIcon, { backgroundColor: '#fef3c7' }]}>
              <Text style={styles.actionEmoji}>📜</Text>
            </View>
            <Text style={styles.actionLabel}>记录</Text>
          </Pressable>

          <Pressable
            style={styles.actionItem}
            onPress={() => router.push('/trading/otc')}
          >
            <View style={[styles.actionIcon, { backgroundColor: '#ede9fe' }]}>
              <Text style={styles.actionEmoji}>💱</Text>
            </View>
            <Text style={styles.actionLabel}>OTC</Text>
          </Pressable>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>钱包管理</Text>

          <Pressable
            style={styles.menuItem}
            onPress={() => router.push('/wallet/accounts' as any)}
          >
            <View style={styles.menuLeft}>
              <Text style={styles.menuIcon}>�</Text>
              <View>
                <Text style={styles.menuLabel}>账户管理</Text>
                <Text style={styles.menuDesc}>管理多个账户，创建或导入</Text>
              </View>
            </View>
            <View style={styles.menuRight}>
              <Text style={styles.accountCount}>{accounts.length}个</Text>
              <Text style={styles.menuArrow}>›</Text>
            </View>
          </Pressable>

          <Pressable
            style={styles.menuItem}
            onPress={() => router.push('/wallet/backup')}
          >
            <View style={styles.menuLeft}>
              <Text style={styles.menuIcon}>�</Text>
              <View>
                <Text style={styles.menuLabel}>备份助记词</Text>
                <Text style={styles.menuDesc}>查看并备份你的助记词</Text>
              </View>
            </View>
            <Text style={styles.menuArrow}>›</Text>
          </Pressable>

          <Pressable 
            style={styles.menuItem}
            onPress={() => router.push('/settings/security')}
          >
            <View style={styles.menuLeft}>
              <Text style={styles.menuIcon}>🛡️</Text>
              <View>
                <Text style={styles.menuLabel}>安全设置</Text>
                <Text style={styles.menuDesc}>交易密码、生物识别</Text>
              </View>
            </View>
            <Text style={styles.menuArrow}>›</Text>
          </Pressable>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>资产概览</Text>
          <View style={styles.assetCard}>
            <View style={styles.assetRow}>
              <View style={styles.assetInfo}>
                <Text style={styles.assetSymbol}>STAR</Text>
                <Text style={styles.assetName}>Cosmos Token</Text>
              </View>
              <View style={styles.assetAmount}>
                <Text style={styles.assetBalance}>0.00</Text>
                <Text style={styles.assetValue}>≈ ¥0.00</Text>
              </View>
            </View>
          </View>
        </View>

        <View style={styles.bottomPadding} />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#6D28D9',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#fff',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#fff',
  },
  headerRight: {
    width: 50,
  },
  content: {
    flex: 1,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  emptyIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  emptyDesc: {
    fontSize: 14,
    color: '#6b7280',
    marginBottom: 32,
  },
  emptyActions: {
    width: '100%',
    gap: 12,
  },
  primaryButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  primaryButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  secondaryButton: {
    backgroundColor: '#f3f4f6',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  secondaryButtonText: {
    color: '#4b5563',
    fontSize: 16,
    fontWeight: '600',
  },
  balanceCard: {
    backgroundColor: '#6D28D9',
    marginHorizontal: 16,
    marginTop: 16,
    borderRadius: 20,
    padding: 20,
  },
  networkStatus: {
    flexDirection: 'row',
    alignItems: 'center',
    alignSelf: 'flex-start',
    backgroundColor: 'rgba(255,255,255,0.2)',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
    marginBottom: 16,
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 6,
  },
  networkText: {
    fontSize: 12,
    color: '#fff',
  },
  addressSection: {
    marginTop: 16,
    backgroundColor: 'rgba(255,255,255,0.1)',
    borderRadius: 12,
    padding: 12,
  },
  quickActions: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: -20,
    borderRadius: 16,
    paddingVertical: 20,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 8,
    elevation: 4,
  },
  actionItem: {
    alignItems: 'center',
  },
  actionIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 8,
  },
  actionEmoji: {
    fontSize: 22,
  },
  actionLabel: {
    fontSize: 13,
    color: '#4b5563',
  },
  section: {
    marginTop: 24,
    paddingHorizontal: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    padding: 16,
    borderRadius: 12,
    marginBottom: 8,
  },
  menuLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  menuIcon: {
    fontSize: 24,
  },
  menuLabel: {
    fontSize: 15,
    fontWeight: '500',
    color: '#1f2937',
  },
  menuDesc: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  menuRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  accountCount: {
    fontSize: 13,
    color: '#6D28D9',
    fontWeight: '500',
  },
  menuArrow: {
    fontSize: 20,
    color: '#d1d5db',
  },
  assetCard: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
  },
  assetRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  assetInfo: {},
  assetSymbol: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  assetName: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  assetAmount: {
    alignItems: 'flex-end',
  },
  assetBalance: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  assetValue: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  bottomPadding: {
    height: 32,
  },
});
