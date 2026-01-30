import React, { useState } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, TextInput } from 'react-native';
import { Search, Filter, TrendingUp, Clock, Gavel } from 'lucide-react-native';

interface Listing {
  id: number;
  petName: string;
  element: string;
  rarity: string;
  level: number;
  price: number;
  seller: string;
  type: 'fixed' | 'auction';
  highestBid?: number;
  endsAt?: string;
}

const RARITY_COLORS: Record<string, string> = {
  common: '#888',
  rare: '#4ECDC4',
  epic: '#BB8FCE',
  legendary: '#F7DC6F',
  mythic: '#FF6B6B',
};

export default function MarketplaceScreen() {
  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState<'all' | 'fixed' | 'auction'>('all');

  const listings: Listing[] = [
    {
      id: 1,
      petName: '烈焰之心',
      element: 'fire',
      rarity: 'legendary',
      level: 35,
      price: 500,
      seller: '5Grw...aEF5',
      type: 'fixed',
    },
    {
      id: 2,
      petName: '深海守护者',
      element: 'water',
      rarity: 'epic',
      level: 28,
      price: 150,
      seller: '5FHn...eVkZ',
      type: 'auction',
      highestBid: 180,
      endsAt: '2h 30m',
    },
    {
      id: 3,
      petName: '暗影猎手',
      element: 'shadow',
      rarity: 'rare',
      level: 20,
      price: 80,
      seller: '5DAn...kLmP',
      type: 'fixed',
    },
    {
      id: 4,
      petName: '光明使者',
      element: 'light',
      rarity: 'mythic',
      level: 50,
      price: 2000,
      seller: '5Grw...aEF5',
      type: 'auction',
      highestBid: 2500,
      endsAt: '5h 15m',
    },
  ];

  const filteredListings = listings.filter((listing) => {
    if (activeTab !== 'all' && listing.type !== activeTab) return false;
    if (searchQuery && !listing.petName.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false;
    }
    return true;
  });

  const marketStats = {
    totalVolume: '125,430 COS',
    totalTrades: 1842,
    activeListings: 156,
  };

  return (
    <View style={styles.container}>
      {/* 市场统计 */}
      <View style={styles.statsBar}>
        <View style={styles.statItem}>
          <TrendingUp size={16} color="#4ECDC4" />
          <Text style={styles.statValue}>{marketStats.totalVolume}</Text>
          <Text style={styles.statLabel}>总交易量</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{marketStats.totalTrades}</Text>
          <Text style={styles.statLabel}>交易次数</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>{marketStats.activeListings}</Text>
          <Text style={styles.statLabel}>在售</Text>
        </View>
      </View>

      {/* 搜索栏 */}
      <View style={styles.searchContainer}>
        <View style={styles.searchBar}>
          <Search size={20} color="#666" />
          <TextInput
            style={styles.searchInput}
            placeholder="搜索宠物..."
            placeholderTextColor="#666"
            value={searchQuery}
            onChangeText={setSearchQuery}
          />
        </View>
        <TouchableOpacity style={styles.filterButton}>
          <Filter size={20} color="#fff" />
        </TouchableOpacity>
      </View>

      {/* 标签页 */}
      <View style={styles.tabs}>
        {(['all', 'fixed', 'auction'] as const).map((tab) => (
          <TouchableOpacity
            key={tab}
            style={[styles.tab, activeTab === tab && styles.tabActive]}
            onPress={() => setActiveTab(tab)}
          >
            <Text style={[styles.tabText, activeTab === tab && styles.tabTextActive]}>
              {tab === 'all' ? '全部' : tab === 'fixed' ? '固定价格' : '拍卖'}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {/* 列表 */}
      <ScrollView style={styles.listContainer}>
        {filteredListings.map((listing) => (
          <TouchableOpacity key={listing.id} style={styles.listingCard}>
            <View style={styles.listingLeft}>
              <View style={[styles.petAvatar, { borderColor: RARITY_COLORS[listing.rarity] }]}>
                <Text style={styles.petEmoji}>🐱</Text>
              </View>
              <View style={styles.listingInfo}>
                <Text style={styles.petName}>{listing.petName}</Text>
                <View style={styles.petMeta}>
                  <Text style={[styles.rarityBadge, { color: RARITY_COLORS[listing.rarity] }]}>
                    {listing.rarity.toUpperCase()}
                  </Text>
                  <Text style={styles.levelText}>Lv.{listing.level}</Text>
                </View>
                <Text style={styles.sellerText}>卖家: {listing.seller}</Text>
              </View>
            </View>
            <View style={styles.listingRight}>
              {listing.type === 'auction' ? (
                <>
                  <View style={styles.auctionBadge}>
                    <Gavel size={12} color="#F7DC6F" />
                    <Text style={styles.auctionText}>拍卖</Text>
                  </View>
                  <Text style={styles.priceText}>{listing.highestBid} COS</Text>
                  <View style={styles.timeContainer}>
                    <Clock size={12} color="#888" />
                    <Text style={styles.timeText}>{listing.endsAt}</Text>
                  </View>
                </>
              ) : (
                <>
                  <Text style={styles.priceText}>{listing.price} COS</Text>
                  <TouchableOpacity style={styles.buyButton}>
                    <Text style={styles.buyButtonText}>购买</Text>
                  </TouchableOpacity>
                </>
              )}
            </View>
          </TouchableOpacity>
        ))}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f0f1a',
  },
  statsBar: {
    flexDirection: 'row',
    backgroundColor: '#1a1a2e',
    padding: 16,
    justifyContent: 'space-around',
  },
  statItem: {
    alignItems: 'center',
  },
  statValue: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
    marginTop: 4,
  },
  statLabel: {
    fontSize: 10,
    color: '#888',
    marginTop: 2,
  },
  searchContainer: {
    flexDirection: 'row',
    padding: 16,
    paddingBottom: 8,
  },
  searchBar: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#1a1a2e',
    borderRadius: 12,
    paddingHorizontal: 12,
    marginRight: 8,
  },
  searchInput: {
    flex: 1,
    padding: 12,
    color: '#fff',
    fontSize: 16,
  },
  filterButton: {
    backgroundColor: '#333',
    borderRadius: 12,
    padding: 14,
  },
  tabs: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    marginBottom: 8,
  },
  tab: {
    paddingVertical: 8,
    paddingHorizontal: 16,
    marginRight: 8,
    borderRadius: 20,
    backgroundColor: '#1a1a2e',
  },
  tabActive: {
    backgroundColor: '#4ECDC4',
  },
  tabText: {
    color: '#888',
    fontSize: 14,
  },
  tabTextActive: {
    color: '#fff',
    fontWeight: 'bold',
  },
  listContainer: {
    flex: 1,
    padding: 16,
  },
  listingCard: {
    backgroundColor: '#1a1a2e',
    borderRadius: 16,
    padding: 16,
    marginBottom: 12,
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  listingLeft: {
    flexDirection: 'row',
    flex: 1,
  },
  petAvatar: {
    width: 56,
    height: 56,
    borderRadius: 28,
    backgroundColor: '#252540',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 2,
    marginRight: 12,
  },
  petEmoji: {
    fontSize: 28,
  },
  listingInfo: {
    flex: 1,
  },
  petName: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 4,
  },
  petMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 4,
  },
  rarityBadge: {
    fontSize: 10,
    fontWeight: 'bold',
    marginRight: 8,
  },
  levelText: {
    fontSize: 12,
    color: '#888',
  },
  sellerText: {
    fontSize: 11,
    color: '#666',
  },
  listingRight: {
    alignItems: 'flex-end',
    justifyContent: 'center',
  },
  auctionBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F7DC6F20',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
    marginBottom: 4,
  },
  auctionText: {
    fontSize: 10,
    color: '#F7DC6F',
    marginLeft: 4,
  },
  priceText: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#4ECDC4',
    marginBottom: 4,
  },
  timeContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  timeText: {
    fontSize: 12,
    color: '#888',
    marginLeft: 4,
  },
  buyButton: {
    backgroundColor: '#FF6B6B',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 8,
  },
  buyButtonText: {
    color: '#fff',
    fontWeight: 'bold',
    fontSize: 14,
  },
});
