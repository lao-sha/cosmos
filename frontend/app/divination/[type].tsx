import { CATEGORIES, DivinationType, Provider, useProviders } from '@/src/hooks/useMarket';
import { useLocalSearchParams, useRouter } from 'expo-router';
import {
  FlatList,
  Pressable,
  StyleSheet,
  Text,
  View
} from 'react-native';

export default function DivinationScreen() {
  const router = useRouter();
  const { type } = useLocalSearchParams<{ type: DivinationType }>();
  const category = CATEGORIES.find(c => c.id === type);
  const { providers, loading } = useProviders(type);

  const handleProviderPress = (provider: Provider) => {
    router.push(`/market/provider/${provider.id}`);
  };

  const renderProvider = ({ item }: { item: Provider }) => (
    <Pressable
      style={({ pressed }) => [styles.providerCard, pressed && styles.providerCardPressed]}
      onPress={() => handleProviderPress(item)}
    >
      <View style={styles.providerHeader}>
        <View style={styles.providerAvatar}>
          <Text style={styles.providerAvatarText}>{item.name[0]}</Text>
        </View>
        <View style={styles.providerInfo}>
          <View style={styles.providerNameRow}>
            <Text style={styles.providerName}>{item.name}</Text>
            {item.isOnline && <View style={styles.onlineDot} />}
          </View>
          <View style={styles.providerStats}>
            <Text style={styles.providerRating}>⭐ {item.rating}</Text>
            <Text style={styles.providerOrders}>{item.orderCount}单</Text>
          </View>
        </View>
        <View style={styles.providerPrice}>
          <Text style={styles.priceValue}>¥{item.price}</Text>
          <Text style={styles.priceUnit}>起</Text>
        </View>
      </View>
    </Pressable>
  );

  if (!category) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>未知类型</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyText}>占卜类型不存在</Text>
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
        <Text style={styles.headerTitle}>{category.icon} {category.name}</Text>
        <View style={styles.headerRight} />
      </View>

      <View style={styles.introCard}>
        <Text style={styles.introIcon}>{category.icon}</Text>
        <Text style={styles.introTitle}>{category.name}</Text>
        <Text style={styles.introDesc}>{category.description}</Text>
      </View>

      <View style={styles.providersSection}>
        <Text style={styles.sectionTitle}>{category.name}大师</Text>
        {loading ? (
          <View style={styles.loadingContainer}>
            <Text style={styles.loadingText}>加载中...</Text>
          </View>
        ) : (
          <FlatList
            data={providers}
            renderItem={renderProvider}
            keyExtractor={(item) => item.id}
            contentContainerStyle={styles.providersList}
            ListEmptyComponent={
              <View style={styles.emptyContainer}>
                <Text style={styles.emptyText}>暂无该类型的占卜师</Text>
                <Text style={styles.emptyHint}>敬请期待更多大师入驻</Text>
              </View>
            }
          />
        )}
      </View>
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
  introCard: {
    backgroundColor: '#fff',
    margin: 16,
    padding: 24,
    borderRadius: 16,
    alignItems: 'center',
  },
  introIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  introTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 8,
  },
  introDesc: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
  },
  providersSection: {
    flex: 1,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    paddingHorizontal: 16,
    marginBottom: 12,
  },
  providersList: {
    paddingHorizontal: 12,
    paddingBottom: 20,
  },
  providerCard: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  providerCardPressed: {
    backgroundColor: '#f9fafb',
  },
  providerHeader: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  providerAvatar: {
    width: 50,
    height: 50,
    borderRadius: 25,
    backgroundColor: '#6D28D9',
    alignItems: 'center',
    justifyContent: 'center',
  },
  providerAvatarText: {
    color: '#fff',
    fontSize: 20,
    fontWeight: 'bold',
  },
  providerInfo: {
    flex: 1,
    marginLeft: 12,
  },
  providerNameRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  providerName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  onlineDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#22c55e',
    marginLeft: 6,
  },
  providerStats: {
    flexDirection: 'row',
    marginTop: 4,
  },
  providerRating: {
    fontSize: 13,
    color: '#f59e0b',
    marginRight: 12,
  },
  providerOrders: {
    fontSize: 13,
    color: '#6b7280',
  },
  providerPrice: {
    alignItems: 'flex-end',
  },
  priceValue: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#ef4444',
  },
  priceUnit: {
    fontSize: 12,
    color: '#9ca3af',
  },
  loadingContainer: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  loadingText: {
    fontSize: 14,
    color: '#9ca3af',
  },
  emptyContainer: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  emptyText: {
    fontSize: 14,
    color: '#9ca3af',
  },
  emptyHint: {
    fontSize: 12,
    color: '#d1d5db',
    marginTop: 8,
  },
});
