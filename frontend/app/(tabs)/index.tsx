import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';

interface Feature {
  icon: string;
  title: string;
  description: string;
  route: string;
}

const FEATURES: Feature[] = [
  { icon: '🔮', title: '玄学占卜', description: '梅花、八字、六爻等多种占卜', route: '/(tabs)/market' },
  { icon: '💬', title: '即时聊天', description: '端到端加密的安全通讯', route: '/(tabs)/chat' },
  { icon: '💎', title: 'OTC交易', description: '安全便捷的场外交易', route: '/trading/otc' },
  { icon: '💕', title: '缘分匹配', description: '基于八字的智能配对', route: '/matchmaking' },
];

export default function HomeScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();

  const handleFeaturePress = (feature: Feature) => {
    router.push(feature.route as any);
  };

  return (
    <ScrollView style={styles.container}>
      <View style={styles.hero}>
        <Text style={styles.heroTitle}>✨ Cosmos</Text>
        <Text style={styles.heroSubtitle}>融合传统文化与 Web3 技术的去中心化社交平台</Text>
        
        <View style={styles.statusBar}>
          <View style={styles.statusItem}>
            <View style={[styles.statusDot, isLoggedIn && styles.statusDotActive]} />
            <Text style={styles.statusText}>
              {isLoggedIn ? '钱包已连接' : '未登录'}
            </Text>
          </View>
          <View style={styles.statusItem}>
            <View style={[styles.statusDot, isConnected && styles.statusDotActive]} />
            <Text style={styles.statusText}>
              {isConnected ? '链已连接' : '链未连接'}
            </Text>
          </View>
        </View>
      </View>

      <View style={styles.featuresSection}>
        <Text style={styles.sectionTitle}>核心功能</Text>
        <View style={styles.featuresGrid}>
          {FEATURES.map((feature, index) => (
            <Pressable
              key={index}
              style={({ pressed }) => [styles.featureCard, pressed && styles.featureCardPressed]}
              onPress={() => handleFeaturePress(feature)}
            >
              <Text style={styles.featureIcon}>{feature.icon}</Text>
              <Text style={styles.featureTitle}>{feature.title}</Text>
              <Text style={styles.featureDesc}>{feature.description}</Text>
            </Pressable>
          ))}
        </View>
      </View>

      <View style={styles.actionsContainer}>
        <Text style={styles.sectionTitle}>快速开始</Text>
        <View style={styles.actionsRow}>
          <Pressable 
            style={({ pressed }) => [styles.actionCard, styles.actionPurple, pressed && styles.actionPressed]}
            onPress={() => router.push('/(tabs)/market')}
          >
            <Text style={styles.actionIcon}>🔮</Text>
            <Text style={styles.actionTitle}>开始占卜</Text>
          </Pressable>
          
          <Pressable 
            style={({ pressed }) => [styles.actionCard, styles.actionGold, pressed && styles.actionPressed]}
            onPress={() => router.push('/wallet')}
          >
            <Text style={styles.actionIcon}>👛</Text>
            <Text style={styles.actionTitle}>我的钱包</Text>
          </Pressable>
        </View>
        <View style={styles.actionsRow}>
          <Pressable 
            style={({ pressed }) => [styles.actionCard, styles.actionPurple, pressed && styles.actionPressed]}
            onPress={() => router.push('/membership')}
          >
            <Text style={styles.actionIcon}>⭐</Text>
            <Text style={styles.actionTitle}>会员中心</Text>
          </Pressable>
          
          <Pressable 
            style={({ pressed }) => [styles.actionCard, styles.actionGold, pressed && styles.actionPressed]}
            onPress={() => router.push('/settings')}
          >
            <Text style={styles.actionIcon}>⚙️</Text>
            <Text style={styles.actionTitle}>系统设置</Text>
          </Pressable>
        </View>
      </View>

      <View style={styles.infoCard}>
        <Text style={styles.infoTitle}>🌟 关于 Cosmos</Text>
        <Text style={styles.infoText}>
          Cosmos 是一个基于 Substrate 区块链的去中心化应用，
          融合了传统玄学文化与现代 Web3 技术。
          所有数据加密存储在 IPFS，链上只保存元数据，
          确保您的隐私安全。
        </Text>
      </View>

      <View style={styles.footer}>
        <Text style={styles.footerText}>Powered by Polkadot SDK</Text>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  statusBar: {
    flexDirection: 'row',
    marginTop: 16,
    gap: 16,
  },
  statusItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#6b7280',
    marginRight: 6,
  },
  statusDotActive: {
    backgroundColor: '#22c55e',
  },
  statusText: {
    fontSize: 12,
    color: 'rgba(255,255,255,0.8)',
  },
  featuresSection: {
    padding: 20,
  },
  featuresGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 12,
  },
  featureCard: {
    width: '47%',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
  },
  featureCardPressed: {
    backgroundColor: '#f9fafb',
  },
  featureIcon: {
    fontSize: 32,
    marginBottom: 8,
  },
  featureTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 4,
  },
  featureDesc: {
    fontSize: 11,
    color: '#6b7280',
    textAlign: 'center',
  },
  actionPressed: {
    opacity: 0.8,
  },
  infoCard: {
    backgroundColor: '#fff',
    margin: 20,
    marginTop: 0,
    borderRadius: 12,
    padding: 16,
  },
  infoTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 13,
    color: '#6b7280',
    lineHeight: 20,
  },
  footer: {
    alignItems: 'center',
    paddingVertical: 20,
    paddingBottom: 40,
  },
  footerText: {
    fontSize: 12,
    color: '#9ca3af',
  },
  hero: {
    backgroundColor: '#6D28D9',
    paddingVertical: 60,
    paddingHorizontal: 20,
    alignItems: 'center',
  },
  heroTitle: {
    fontSize: 36,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 12,
  },
  heroSubtitle: {
    fontSize: 16,
    color: 'rgba(255,255,255,0.85)',
    textAlign: 'center',
  },
  card: {
    backgroundColor: '#fff',
    margin: 20,
    borderRadius: 16,
    padding: 20,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 8,
    elevation: 3,
  },
  cardTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 12,
  },
  cardDescription: {
    fontSize: 15,
    color: '#6b7280',
    lineHeight: 24,
  },
  actionsContainer: {
    paddingHorizontal: 20,
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#374151',
    marginBottom: 16,
  },
  actionsRow: {
    flexDirection: 'row',
    gap: 12,
  },
  actionCard: {
    flex: 1,
    borderRadius: 16,
    padding: 20,
    alignItems: 'center',
  },
  actionPurple: {
    backgroundColor: 'rgba(109, 40, 217, 0.1)',
    borderWidth: 1,
    borderColor: 'rgba(109, 40, 217, 0.2)',
  },
  actionGold: {
    backgroundColor: 'rgba(212, 175, 55, 0.1)',
    borderWidth: 1,
    borderColor: 'rgba(212, 175, 55, 0.2)',
  },
  actionIcon: {
    fontSize: 32,
    marginBottom: 8,
  },
  actionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#374151',
  },
  hintContainer: {
    paddingHorizontal: 20,
    paddingBottom: 40,
  },
  hintText: {
    fontSize: 14,
    color: '#9ca3af',
    textAlign: 'center',
    lineHeight: 22,
  },
});
