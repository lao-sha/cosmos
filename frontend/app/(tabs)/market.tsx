import { CATEGORIES, DivinationCategory, DivinationType, Provider, useProviders } from '@/src/hooks/useMarket';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    FlatList,
    Pressable,
    StyleSheet,
    Text,
    View
} from 'react-native';

export default function MarketScreen() {
  const router = useRouter();
  const [selectedCategory, setSelectedCategory] = useState<DivinationType | null>(null);
  const { providers, loading, error, categories } = useProviders(selectedCategory);

  const handleCategoryPress = (category: DivinationCategory) => {
    setSelectedCategory(selectedCategory === category.id ? null : category.id);
  };

  const handleProviderPress = (provider: Provider) => {
    router.push(`/market/provider/${provider.id}`);
  };

  const filteredProviders = providers;

  const renderCategory = ({ item }: { item: DivinationCategory }) => (
    <Pressable
      style={({ pressed }) => [
        styles.categoryCard,
        selectedCategory === item.id && styles.categoryCardSelected,
        pressed && styles.categoryCardPressed,
      ]}
      onPress={() => handleCategoryPress(item)}
    >
      <Text style={styles.categoryIcon}>{item.icon}</Text>
      <Text style={[
        styles.categoryName,
        selectedCategory === item.id && styles.categoryNameSelected
      ]}>{item.name}</Text>
    </Pressable>
  );

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
      <View style={styles.providerSpecialties}>
        {item.specialties.map(s => {
          const cat = CATEGORIES.find(c => c.id === s);
          return cat ? (
            <View key={s} style={styles.specialtyTag}>
              <Text style={styles.specialtyText}>{cat.icon} {cat.name}</Text>
            </View>
          ) : null;
        })}
      </View>
    </Pressable>
  );

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.headerTitle}>占卜市场</Text>
        <Text style={styles.headerSubtitle}>寻找适合你的占卜师</Text>
      </View>

      <View style={styles.categoriesSection}>
        <FlatList
          data={CATEGORIES}
          renderItem={renderCategory}
          keyExtractor={(item) => item.id}
          horizontal
          showsHorizontalScrollIndicator={false}
          contentContainerStyle={styles.categoriesList}
        />
      </View>

      <View style={styles.providersSection}>
        <Text style={styles.sectionTitle}>
          {selectedCategory 
            ? `${CATEGORIES.find(c => c.id === selectedCategory)?.name}大师`
            : '推荐大师'}
        </Text>
        <FlatList
          data={filteredProviders}
          renderItem={renderProvider}
          keyExtractor={(item) => item.id}
          contentContainerStyle={styles.providersList}
          ListEmptyComponent={
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyText}>暂无该类型的占卜师</Text>
            </View>
          }
        />
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
    paddingBottom: 20,
    paddingHorizontal: 20,
  },
  headerTitle: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#fff',
  },
  headerSubtitle: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
    marginTop: 4,
  },
  categoriesSection: {
    backgroundColor: '#fff',
    paddingVertical: 16,
  },
  categoriesList: {
    paddingHorizontal: 12,
  },
  categoryCard: {
    alignItems: 'center',
    backgroundColor: '#f3f4f6',
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderRadius: 12,
    marginHorizontal: 4,
    minWidth: 80,
  },
  categoryCardSelected: {
    backgroundColor: '#6D28D9',
  },
  categoryCardPressed: {
    opacity: 0.8,
  },
  categoryIcon: {
    fontSize: 24,
    marginBottom: 4,
  },
  categoryName: {
    fontSize: 12,
    color: '#374151',
    fontWeight: '500',
  },
  categoryNameSelected: {
    color: '#fff',
  },
  providersSection: {
    flex: 1,
    paddingTop: 16,
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
  providerSpecialties: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginTop: 12,
  },
  specialtyTag: {
    backgroundColor: '#f3f4f6',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
    marginRight: 8,
    marginBottom: 4,
  },
  specialtyText: {
    fontSize: 12,
    color: '#6b7280',
  },
  emptyContainer: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  emptyText: {
    fontSize: 14,
    color: '#9ca3af',
  },
});
