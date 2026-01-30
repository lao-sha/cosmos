import { QRAddressDisplay } from '@/src/components/AddressDisplay';
import { useAuthStore } from '@/src/stores/auth';
import { useRouter } from 'expo-router';
import { Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';

export default function ReceiveScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();

  if (!isLoggedIn || !address) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>收款</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.centerContent}>
          <Text style={styles.emptyText}>请先登录</Text>
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
        <Text style={styles.headerTitle}>收款</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.contentContainer}
        showsVerticalScrollIndicator={false}
      >
        <View style={styles.qrCard}>
          <QRAddressDisplay address={address} label="扫码向我转账" />
        </View>

        <View style={styles.infoCard}>
          <Text style={styles.infoTitle}>💡 收款须知</Text>
          <Text style={styles.infoText}>
            • 仅支持接收 COS 代币{'\n'}
            • 请确保发送方使用正确的网络{'\n'}
            • 转账通常在几秒内到账
          </Text>
        </View>

        <View style={styles.tokenInfo}>
          <Text style={styles.tokenTitle}>支持的代币</Text>
          <View style={styles.tokenItem}>
            <View style={styles.tokenIcon}>
              <Text style={styles.tokenEmoji}>⭐</Text>
            </View>
            <View style={styles.tokenDetails}>
              <Text style={styles.tokenSymbol}>COS</Text>
              <Text style={styles.tokenName}>Cosmos Token</Text>
            </View>
          </View>
        </View>
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
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  headerRight: {
    width: 50,
  },
  content: {
    flex: 1,
  },
  contentContainer: {
    padding: 16,
  },
  centerContent: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 16,
    color: '#6b7280',
  },
  qrCard: {
    backgroundColor: '#fff',
    borderRadius: 20,
    padding: 24,
    marginBottom: 16,
  },
  infoCard: {
    backgroundColor: '#eff6ff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  infoTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1e40af',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 13,
    color: '#1e3a8a',
    lineHeight: 20,
  },
  tokenInfo: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
  },
  tokenTitle: {
    fontSize: 14,
    color: '#6b7280',
    marginBottom: 12,
  },
  tokenItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  tokenIcon: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#fef3c7',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  tokenEmoji: {
    fontSize: 20,
  },
  tokenDetails: {},
  tokenSymbol: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  tokenName: {
    fontSize: 13,
    color: '#6b7280',
    marginTop: 2,
  },
});
