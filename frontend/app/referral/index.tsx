import { useReferral } from '@/src/hooks/useReferral';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  ActivityIndicator,
  Platform,
  Pressable,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  View
} from 'react-native';
import * as Clipboard from 'expo-clipboard';

export default function ReferralScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { referralInfo, loading, refresh } = useReferral();
  const [refreshing, setRefreshing] = useState(false);

  const showAlert = (title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      const { Alert } = require('react-native');
      Alert.alert(title, message);
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  const copyToClipboard = async (text: string) => {
    await Clipboard.setStringAsync(text);
    showAlert('已复制', text);
  };

  const shortenAddress = (addr: string) => {
    if (!addr) return '';
    return `${addr.slice(0, 8)}...${addr.slice(-6)}`;
  };

  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>推荐关系</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔗</Text>
          <Text style={styles.emptyTitle}>请先登录</Text>
          <Text style={styles.emptySubtitle}>登录后查看推荐关系</Text>
        </View>
      </View>
    );
  }

  if (!isConnected) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>推荐关系</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔌</Text>
          <Text style={styles.emptyTitle}>未连接网络</Text>
          <Text style={styles.emptySubtitle}>请先连接区块链网络</Text>
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
        <Text style={styles.headerTitle}>推荐关系</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.content}
        refreshControl={
          <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
        }
      >
        {loading && !referralInfo ? (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="large" color="#6D28D9" />
            <Text style={styles.loadingText}>加载中...</Text>
          </View>
        ) : (
          <>
            <View style={styles.card}>
              <Text style={styles.cardTitle}>📋 我的推荐码</Text>
              {referralInfo?.myCode ? (
                <Pressable 
                  style={styles.codeBox}
                  onPress={() => copyToClipboard(referralInfo.myCode!)}
                >
                  <Text style={styles.codeText}>{referralInfo.myCode}</Text>
                  <Text style={styles.copyHint}>点击复制</Text>
                </Pressable>
              ) : (
                <View style={styles.noDataBox}>
                  <Text style={styles.noDataText}>暂无推荐码</Text>
                  <Text style={styles.noDataHint}>成为会员后自动获得</Text>
                </View>
              )}
            </View>

            <View style={styles.card}>
              <Text style={styles.cardTitle}>⬆️ 我的上线</Text>
              {referralInfo?.sponsor ? (
                <Pressable 
                  style={styles.addressBox}
                  onPress={() => copyToClipboard(referralInfo.sponsor!)}
                >
                  <Text style={styles.addressText}>
                    {shortenAddress(referralInfo.sponsor)}
                  </Text>
                  <Text style={styles.copyHint}>点击复制完整地址</Text>
                </Pressable>
              ) : (
                <View style={styles.noDataBox}>
                  <Text style={styles.noDataText}>暂无上线</Text>
                  <Text style={styles.noDataHint}>使用推荐码绑定上线</Text>
                </View>
              )}
            </View>

            <View style={styles.card}>
              <View style={styles.cardHeader}>
                <Text style={styles.cardTitle}>⬇️ 我的下线</Text>
                <View style={styles.countBadge}>
                  <Text style={styles.countText}>
                    {referralInfo?.downlineCount || 0}
                  </Text>
                </View>
              </View>
              
              {referralInfo?.downlines && referralInfo.downlines.length > 0 ? (
                <View style={styles.downlineList}>
                  {referralInfo.downlines.slice(0, 10).map((downline, index) => (
                    <Pressable
                      key={downline}
                      style={styles.downlineItem}
                      onPress={() => copyToClipboard(downline)}
                    >
                      <Text style={styles.downlineIndex}>{index + 1}</Text>
                      <Text style={styles.downlineAddress}>
                        {shortenAddress(downline)}
                      </Text>
                    </Pressable>
                  ))}
                  {referralInfo.downlines.length > 10 && (
                    <Text style={styles.moreText}>
                      还有 {referralInfo.downlines.length - 10} 个下线...
                    </Text>
                  )}
                </View>
              ) : (
                <View style={styles.noDataBox}>
                  <Text style={styles.noDataText}>暂无下线</Text>
                  <Text style={styles.noDataHint}>分享推荐码邀请好友</Text>
                </View>
              )}
            </View>

            {referralInfo?.referralChain && referralInfo.referralChain.length > 0 && (
              <View style={styles.card}>
                <Text style={styles.cardTitle}>🔗 推荐链（上线链）</Text>
                <View style={styles.chainList}>
                  {referralInfo.referralChain.map((addr, index) => (
                    <View key={addr} style={styles.chainItem}>
                      <Text style={styles.chainLevel}>L{index + 1}</Text>
                      <Text style={styles.chainAddress}>
                        {shortenAddress(addr)}
                      </Text>
                    </View>
                  ))}
                </View>
              </View>
            )}

            <View style={styles.tipsCard}>
              <Text style={styles.tipsTitle}>💡 推荐说明</Text>
              <Text style={styles.tipsText}>
                • 分享推荐码邀请好友注册{'\n'}
                • 下线购买会员，您可获得分成{'\n'}
                • 推荐链最多15层，层层有收益{'\n'}
                • 推荐关系一旦绑定不可更改
              </Text>
            </View>

            <View style={styles.bottomPadding} />
          </>
        )}
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
    backgroundColor: '#6D28D9',
    paddingTop: 50,
    paddingBottom: 16,
    paddingHorizontal: 16,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    color: '#fff',
    fontSize: 18,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#fff',
  },
  headerRight: {
    width: 40,
  },
  content: {
    flex: 1,
  },
  loadingContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingTop: 100,
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
  },
  emptyContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  emptyIcon: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 14,
    color: '#9ca3af',
  },
  card: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  cardHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  countBadge: {
    backgroundColor: '#6D28D9',
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
    marginBottom: 12,
  },
  countText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '600',
  },
  codeBox: {
    backgroundColor: '#f0fdf4',
    padding: 16,
    borderRadius: 10,
    alignItems: 'center',
  },
  codeText: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#16a34a',
    letterSpacing: 2,
  },
  copyHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 8,
  },
  addressBox: {
    backgroundColor: '#f3f4f6',
    padding: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  addressText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#6D28D9',
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
  },
  noDataBox: {
    backgroundColor: '#f9fafb',
    padding: 20,
    borderRadius: 10,
    alignItems: 'center',
  },
  noDataText: {
    fontSize: 14,
    color: '#6b7280',
  },
  noDataHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 4,
  },
  downlineList: {
    gap: 8,
  },
  downlineItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f9fafb',
    padding: 12,
    borderRadius: 8,
  },
  downlineIndex: {
    width: 24,
    fontSize: 12,
    fontWeight: '600',
    color: '#6D28D9',
  },
  downlineAddress: {
    flex: 1,
    fontSize: 14,
    color: '#374151',
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
  },
  moreText: {
    textAlign: 'center',
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 8,
  },
  chainList: {
    gap: 8,
  },
  chainItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#faf5ff',
    padding: 10,
    borderRadius: 8,
  },
  chainLevel: {
    width: 32,
    fontSize: 12,
    fontWeight: '600',
    color: '#6D28D9',
  },
  chainAddress: {
    flex: 1,
    fontSize: 13,
    color: '#374151',
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
  },
  tipsCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  tipsTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  tipsText: {
    fontSize: 13,
    color: '#6b7280',
    lineHeight: 22,
  },
  bottomPadding: {
    height: 40,
  },
});
