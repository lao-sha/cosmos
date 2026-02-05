import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
} from 'react-native';
import { useRouter } from 'expo-router';
import { LinearGradient } from 'expo-linear-gradient';
import { Send, QrCode, History } from 'lucide-react-native';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import { useBalance, formatBalance } from '@/hooks/useBalance';
import { Card, Button } from '@/components/ui';
import { Gradients, Shadows } from '@/constants/colors';

export default function WalletScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address, isConnected, name } = useWalletStore();
  const { data: balance, isLoading } = useBalance();

  if (!isConnected) {
    return (
      <View style={[styles.container, { backgroundColor: colors.background }]}>
        <View style={styles.emptyState}>
          <Text style={[styles.emptyTitle, { color: colors.textPrimary }]}>
            欢迎使用 COSMOS
          </Text>
          <Text style={[styles.emptySubtitle, { color: colors.textSecondary }]}>
            创建或导入钱包开始使用
          </Text>
          <View style={styles.buttonGroup}>
            <Button
              title="创建钱包"
              onPress={() => router.push('/wallet/create')}
              style={styles.button}
            />
            <Button
              title="导入钱包"
              onPress={() => router.push('/wallet/import')}
              variant="outline"
              style={styles.button}
            />
          </View>
        </View>
      </View>
    );
  }

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      {/* Balance Card */}
      <LinearGradient
        colors={Gradients.primary as [string, string]}
        start={{ x: 0, y: 0 }}
        end={{ x: 1, y: 1 }}
        style={[styles.balanceCard, Shadows.lg]}
      >
        <Text style={styles.walletName}>{name || '我的钱包'}</Text>
        <Text style={styles.balanceLabel}>总资产 (COS)</Text>
        <Text style={styles.balanceValue}>
          {isLoading ? '...' : formatBalance(balance || '0')}
        </Text>
        <Text style={styles.addressText} numberOfLines={1}>
          {address?.slice(0, 8)}...{address?.slice(-8)}
        </Text>
      </LinearGradient>

      {/* Quick Actions */}
      <View style={styles.actions}>
        <TouchableOpacity style={styles.actionItem}>
          <View style={[styles.actionIcon, { backgroundColor: colors.surface }]}>
            <Send size={24} color={colors.primary} />
          </View>
          <Text style={[styles.actionText, { color: colors.textPrimary }]}>
            转账
          </Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.actionItem}>
          <View style={[styles.actionIcon, { backgroundColor: colors.surface }]}>
            <QrCode size={24} color={colors.primary} />
          </View>
          <Text style={[styles.actionText, { color: colors.textPrimary }]}>
            收款
          </Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.actionItem}>
          <View style={[styles.actionIcon, { backgroundColor: colors.surface }]}>
            <History size={24} color={colors.primary} />
          </View>
          <Text style={[styles.actionText, { color: colors.textPrimary }]}>
            记录
          </Text>
        </TouchableOpacity>
      </View>

      {/* Assets */}
      <Card style={styles.assetsCard}>
        <Text style={[styles.sectionTitle, { color: colors.textPrimary }]}>
          资产
        </Text>
        <View style={styles.assetItem}>
          <View style={styles.assetInfo}>
            <View style={[styles.assetIcon, { backgroundColor: colors.primary }]}>
              <Text style={styles.assetIconText}>C</Text>
            </View>
            <View>
              <Text style={[styles.assetName, { color: colors.textPrimary }]}>
                COS
              </Text>
              <Text style={[styles.assetSubtitle, { color: colors.textSecondary }]}>
                Cosmos Token
              </Text>
            </View>
          </View>
          <Text style={[styles.assetBalance, { color: colors.textPrimary }]}>
            {isLoading ? '...' : formatBalance(balance || '0')}
          </Text>
        </View>
      </Card>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  emptyState: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
  },
  emptyTitle: {
    fontSize: 28,
    fontWeight: '700',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 16,
    marginBottom: 32,
  },
  buttonGroup: {
    width: '100%',
    gap: 12,
  },
  button: {
    width: '100%',
  },
  balanceCard: {
    borderRadius: 20,
    padding: 24,
    marginBottom: 24,
  },
  walletName: {
    color: 'rgba(255,255,255,0.8)',
    fontSize: 14,
    marginBottom: 16,
  },
  balanceLabel: {
    color: 'rgba(255,255,255,0.7)',
    fontSize: 14,
  },
  balanceValue: {
    color: '#FFFFFF',
    fontSize: 36,
    fontWeight: '700',
    marginVertical: 4,
  },
  addressText: {
    color: 'rgba(255,255,255,0.6)',
    fontSize: 12,
    marginTop: 12,
  },
  actions: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    marginBottom: 24,
  },
  actionItem: {
    alignItems: 'center',
  },
  actionIcon: {
    width: 56,
    height: 56,
    borderRadius: 16,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 8,
  },
  actionText: {
    fontSize: 14,
  },
  assetsCard: {
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 16,
  },
  assetItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  assetInfo: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  assetIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  assetIconText: {
    color: '#FFFFFF',
    fontSize: 18,
    fontWeight: '700',
  },
  assetName: {
    fontSize: 16,
    fontWeight: '600',
  },
  assetSubtitle: {
    fontSize: 12,
  },
  assetBalance: {
    fontSize: 16,
    fontWeight: '600',
  },
});
