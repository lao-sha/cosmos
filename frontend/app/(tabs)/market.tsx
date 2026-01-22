/**
 * 星尘玄鉴 - 占卜市场页面
 * 主题色：金棕色 #B2955D
 */

import { View, Text, StyleSheet, ScrollView, Pressable, ActivityIndicator } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useState } from 'react';

const THEME_COLOR = '#B2955D';
const THEME_BG = '#F5F5F7';

// 模拟服务商数据
const MOCK_PROVIDERS = [
  {
    id: '1',
    name: '玄机子',
    avatar: '🧙',
    specialty: ['八字', '紫微'],
    rating: 4.9,
    orders: 1280,
    price: '50 DUST',
    online: true,
  },
  {
    id: '2',
    name: '易道人',
    avatar: '👴',
    specialty: ['六爻', '梅花'],
    rating: 4.8,
    orders: 856,
    price: '30 DUST',
    online: true,
  },
  {
    id: '3',
    name: '星月师',
    avatar: '🌙',
    specialty: ['塔罗', '占星'],
    rating: 4.7,
    orders: 623,
    price: '40 DUST',
    online: false,
  },
  {
    id: '4',
    name: '天机阁',
    avatar: '🏛️',
    specialty: ['奇门', '大六壬'],
    rating: 4.9,
    orders: 2100,
    price: '80 DUST',
    online: true,
  },
];

// 分类
const CATEGORIES = [
  { id: 'all', name: '全部', icon: 'apps' },
  { id: 'bazi', name: '八字', icon: 'calendar' },
  { id: 'liuyao', name: '六爻', icon: 'dice' },
  { id: 'tarot', name: '塔罗', icon: 'card' },
  { id: 'qimen', name: '奇门', icon: 'compass' },
];

export default function MarketPage() {
  const router = useRouter();
  const [activeCategory, setActiveCategory] = useState('all');
  const [loading, setLoading] = useState(false);

  return (
    <View style={styles.wrapper}>
    <View style={styles.container}>
      {/* 顶部标题 */}
      <View style={styles.header}>
        <Text style={styles.title}>占卜市场</Text>
        <Text style={styles.subtitle}>找到适合你的占卜师</Text>
      </View>

      {/* 搜索栏 */}
      <View style={styles.searchSection}>
        <View style={styles.searchBar}>
          <Ionicons name="search" size={20} color="#999" />
          <Text style={styles.searchPlaceholder}>搜索占卜师或服务</Text>
        </View>
        <Pressable style={styles.filterButton}>
          <Ionicons name="options-outline" size={20} color={THEME_COLOR} />
        </Pressable>
      </View>

      {/* 分类标签 */}
      <ScrollView 
        horizontal 
        showsHorizontalScrollIndicator={false} 
        style={styles.categoryScroll}
        contentContainerStyle={styles.categoryContent}
      >
        {CATEGORIES.map((cat) => (
          <Pressable
            key={cat.id}
            style={[
              styles.categoryTag,
              activeCategory === cat.id && styles.categoryTagActive
            ]}
            onPress={() => setActiveCategory(cat.id)}
          >
            <Text style={[
              styles.categoryText,
              activeCategory === cat.id && styles.categoryTextActive
            ]}>
              {cat.name}
            </Text>
          </Pressable>
        ))}
      </ScrollView>

      {/* 服务商列表 */}
      <ScrollView 
        style={styles.content} 
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
      >
        {/* 推荐区 */}
        <View style={styles.section}>
          <View style={styles.sectionHeader}>
            <Text style={styles.sectionTitle}>推荐占卜师</Text>
            <Pressable>
              <Text style={styles.seeAll}>查看全部</Text>
            </Pressable>
          </View>

          {loading ? (
            <ActivityIndicator size="large" color={THEME_COLOR} style={styles.loading} />
          ) : (
            <View style={styles.providerList}>
              {MOCK_PROVIDERS.map((provider) => (
                <Pressable 
                  key={provider.id} 
                  style={styles.providerCard}
                  onPress={() => {
                    // TODO: 跳转到服务商详情
                    console.log('Provider:', provider.id);
                  }}
                >
                  <View style={styles.providerHeader}>
                    <View style={styles.avatarContainer}>
                      <Text style={styles.avatar}>{provider.avatar}</Text>
                      {provider.online && <View style={styles.onlineDot} />}
                    </View>
                    <View style={styles.providerInfo}>
                      <Text style={styles.providerName}>{provider.name}</Text>
                      <View style={styles.specialtyRow}>
                        {provider.specialty.map((s, i) => (
                          <View key={i} style={styles.specialtyTag}>
                            <Text style={styles.specialtyText}>{s}</Text>
                          </View>
                        ))}
                      </View>
                    </View>
                    <View style={styles.priceContainer}>
                      <Text style={styles.price}>{provider.price}</Text>
                      <Text style={styles.priceLabel}>起</Text>
                    </View>
                  </View>
                  <View style={styles.providerFooter}>
                    <View style={styles.ratingRow}>
                      <Ionicons name="star" size={14} color="#F5A623" />
                      <Text style={styles.rating}>{provider.rating}</Text>
                      <Text style={styles.orders}>· {provider.orders}单</Text>
                    </View>
                    <Pressable style={styles.consultButton}>
                      <Text style={styles.consultText}>咨询</Text>
                    </Pressable>
                  </View>
                </Pressable>
              ))}
            </View>
          )}
        </View>

        {/* 底部提示 */}
        <View style={styles.footer}>
          <Text style={styles.footerText}>所有交易通过智能合约保障</Text>
          <Text style={styles.footerText}>服务完成后自动结算</Text>
        </View>
      </ScrollView>
    </View>
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
  header: {
    paddingTop: 60,
    paddingHorizontal: 20,
    paddingBottom: 16,
    backgroundColor: '#FFF',
  },
  title: {
    fontSize: 28,
    fontWeight: '600',
    color: '#333',
  },
  subtitle: {
    fontSize: 14,
    color: '#999',
    marginTop: 4,
  },
  searchSection: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
    gap: 12,
  },
  searchBar: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    gap: 8,
  },
  searchPlaceholder: {
    color: '#999',
    fontSize: 15,
  },
  filterButton: {
    width: 44,
    height: 44,
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
  },
  categoryScroll: {
    backgroundColor: '#FFF',
    maxHeight: 56,
  },
  categoryContent: {
    paddingHorizontal: 16,
    paddingVertical: 12,
    gap: 8,
  },
  categoryTag: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    backgroundColor: '#F5F5F7',
    borderRadius: 20,
    marginRight: 8,
  },
  categoryTagActive: {
    backgroundColor: THEME_COLOR,
  },
  categoryText: {
    fontSize: 14,
    color: '#666',
  },
  categoryTextActive: {
    color: '#FFF',
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 16,
    paddingBottom: 100,
  },
  section: {
    marginBottom: 24,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  seeAll: {
    fontSize: 14,
    color: THEME_COLOR,
  },
  loading: {
    paddingVertical: 40,
  },
  providerList: {
    gap: 12,
  },
  providerCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
  },
  providerHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  avatarContainer: {
    position: 'relative',
  },
  avatar: {
    fontSize: 40,
    width: 56,
    height: 56,
    textAlign: 'center',
    lineHeight: 56,
    backgroundColor: '#F5F5F7',
    borderRadius: 28,
    overflow: 'hidden',
  },
  onlineDot: {
    position: 'absolute',
    bottom: 2,
    right: 2,
    width: 12,
    height: 12,
    backgroundColor: '#4CD964',
    borderRadius: 6,
    borderWidth: 2,
    borderColor: '#FFF',
  },
  providerInfo: {
    flex: 1,
    marginLeft: 12,
  },
  providerName: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
    marginBottom: 6,
  },
  specialtyRow: {
    flexDirection: 'row',
    gap: 6,
  },
  specialtyTag: {
    backgroundColor: THEME_COLOR + '15',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
  },
  specialtyText: {
    fontSize: 12,
    color: THEME_COLOR,
  },
  priceContainer: {
    alignItems: 'flex-end',
  },
  price: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  priceLabel: {
    fontSize: 12,
    color: '#999',
  },
  providerFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#F5F5F7',
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  rating: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
  },
  orders: {
    fontSize: 13,
    color: '#999',
  },
  consultButton: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 20,
    paddingVertical: 8,
    borderRadius: 20,
  },
  consultText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#FFF',
  },
  footer: {
    alignItems: 'center',
    paddingVertical: 20,
  },
  footerText: {
    fontSize: 12,
    color: '#BBB',
    marginBottom: 4,
  },
});
