import React, { useState } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, TextInput, Modal, Platform, ActivityIndicator } from 'react-native';
import { useRouter } from 'expo-router';
import { Search, Filter, TrendingUp, Clock, Gavel, X, Check, AlertCircle } from 'lucide-react-native';
import { useMeowstar, MarketListing } from '@/services/meowstar';

// 跨平台 Alert
const showAlert = (title: string, message: string, onOk?: () => void) => {
  if (Platform.OS === 'web') {
    window.alert(`${title}\n\n${message}`);
    onOk?.();
  } else {
    const { Alert } = require('react-native');
    Alert.alert(title, message, [{ text: '确定', onPress: onOk }]);
  }
};

// 格式化剩余时间
const formatTimeLeft = (endsAt: number): string => {
  const now = Date.now();
  const diff = endsAt - now;
  if (diff <= 0) return '已结束';
  
  const hours = Math.floor(diff / 3600000);
  const minutes = Math.floor((diff % 3600000) / 60000);
  
  if (hours >= 24) {
    const days = Math.floor(hours / 24);
    return `${days}天 ${hours % 24}小时`;
  }
  return `${hours}h ${minutes}m`;
};

const RARITY_COLORS: Record<string, string> = {
  common: '#888',
  rare: '#4ECDC4',
  epic: '#BB8FCE',
  legendary: '#F7DC6F',
  mythic: '#FF6B6B',
};

export default function MarketplaceScreen() {
  const router = useRouter();
  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState<'all' | 'fixed' | 'auction'>('all');
  const [selectedListing, setSelectedListing] = useState<MarketListing | null>(null);
  const [showBuyModal, setShowBuyModal] = useState(false);
  const [showBidModal, setShowBidModal] = useState(false);
  const [bidAmount, setBidAmount] = useState('');
  const [isPurchasing, setIsPurchasing] = useState(false);

  // 使用全局状态
  const { marketListings, user, buyPet, placeBid, isLoading } = useMeowstar();

  const filteredListings = marketListings.filter((listing) => {
    if (activeTab !== 'all' && listing.type !== activeTab) return false;
    if (searchQuery && !listing.petName.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false;
    }
    return true;
  });

  const marketStats = {
    totalVolume: '125,430 COS',
    totalTrades: 1842,
    activeListings: marketListings.length,
  };

  // 处理购买
  const handleBuy = (listing: MarketListing) => {
    setSelectedListing(listing);
    setShowBuyModal(true);
  };

  // 处理出价
  const handleBid = (listing: MarketListing) => {
    setSelectedListing(listing);
    setBidAmount(String((listing.highestBid || listing.price) + 10));
    setShowBidModal(true);
  };

  // 确认购买
  const confirmPurchase = async () => {
    if (!selectedListing) return;
    setIsPurchasing(true);
    
    const result = await buyPet(selectedListing.id);
    setIsPurchasing(false);
    setShowBuyModal(false);
    
    showAlert(
      result.success ? '购买成功！' : '购买失败',
      result.message,
      result.success ? () => router.push('/meowstar/pets' as any) : undefined
    );
  };

  // 确认出价
  const confirmBid = async () => {
    if (!selectedListing) return;
    const minBid = (selectedListing.highestBid || selectedListing.price) + 1;
    
    if (!bidAmount || Number(bidAmount) < minBid) {
      showAlert('出价失败', `出价必须高于当前最高价 ${minBid - 1} COS`);
      return;
    }
    
    setIsPurchasing(true);
    
    const result = await placeBid(selectedListing.id, Number(bidAmount));
    setIsPurchasing(false);
    setShowBidModal(false);
    
    showAlert(
      result.success ? '出价成功！' : '出价失败',
      result.message
    );
  };

  if (isLoading) {
    return (
      <View style={[styles.container, { justifyContent: 'center', alignItems: 'center' }]}>
        <ActivityIndicator size="large" color="#4ECDC4" />
        <Text style={{ color: '#888', marginTop: 16 }}>加载中...</Text>
      </View>
    );
  }

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
                  <TouchableOpacity style={styles.bidButton} onPress={() => handleBid(listing)}>
                    <Text style={styles.bidButtonText}>出价</Text>
                  </TouchableOpacity>
                </>
              ) : (
                <>
                  <Text style={styles.priceText}>{listing.price} COS</Text>
                  <TouchableOpacity style={styles.buyButton} onPress={() => handleBuy(listing)}>
                    <Text style={styles.buyButtonText}>购买</Text>
                  </TouchableOpacity>
                </>
              )}
            </View>
          </TouchableOpacity>
        ))}
      </ScrollView>

      {/* 购买确认弹窗 */}
      <Modal
        visible={showBuyModal}
        transparent
        animationType="fade"
        onRequestClose={() => setShowBuyModal(false)}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <TouchableOpacity
              style={styles.modalClose}
              onPress={() => setShowBuyModal(false)}
            >
              <X size={24} color="#888" />
            </TouchableOpacity>
            
            <Text style={styles.modalTitle}>确认购买</Text>
            
            {selectedListing && (
              <>
                <View style={styles.modalPetInfo}>
                  <View style={[styles.modalPetAvatar, { borderColor: RARITY_COLORS[selectedListing.rarity] }]}>
                    <Text style={styles.modalPetEmoji}>🐱</Text>
                  </View>
                  <Text style={styles.modalPetName}>{selectedListing.petName}</Text>
                  <View style={styles.modalPetMeta}>
                    <Text style={[styles.modalRarity, { color: RARITY_COLORS[selectedListing.rarity] }]}>
                      {selectedListing.rarity.toUpperCase()}
                    </Text>
                    <Text style={styles.modalLevel}>Lv.{selectedListing.level}</Text>
                  </View>
                </View>
                
                <View style={styles.modalPriceSection}>
                  <Text style={styles.modalPriceLabel}>购买价格</Text>
                  <Text style={styles.modalPrice}>{selectedListing.price} COS</Text>
                </View>
                
                <View style={styles.modalFeeSection}>
                  <View style={styles.modalFeeRow}>
                    <Text style={styles.modalFeeLabel}>商品价格</Text>
                    <Text style={styles.modalFeeValue}>{selectedListing.price} COS</Text>
                  </View>
                  <View style={styles.modalFeeRow}>
                    <Text style={styles.modalFeeLabel}>手续费 (0%)</Text>
                    <Text style={styles.modalFeeValue}>0 COS</Text>
                  </View>
                  <View style={[styles.modalFeeRow, styles.modalTotalRow]}>
                    <Text style={styles.modalTotalLabel}>总计</Text>
                    <Text style={styles.modalTotalValue}>{selectedListing.price} COS</Text>
                  </View>
                </View>
                
                <View style={styles.modalWarning}>
                  <AlertCircle size={16} color="#F7DC6F" />
                  <Text style={styles.modalWarningText}>购买后将从你的钱包扣除相应金额</Text>
                </View>
                
                <View style={styles.modalActions}>
                  <TouchableOpacity
                    style={styles.modalCancelButton}
                    onPress={() => setShowBuyModal(false)}
                  >
                    <Text style={styles.modalCancelText}>取消</Text>
                  </TouchableOpacity>
                  <TouchableOpacity
                    style={[styles.modalConfirmButton, isPurchasing && styles.modalButtonDisabled]}
                    onPress={confirmPurchase}
                    disabled={isPurchasing}
                  >
                    {isPurchasing ? (
                      <Text style={styles.modalConfirmText}>处理中...</Text>
                    ) : (
                      <>
                        <Check size={18} color="#fff" />
                        <Text style={styles.modalConfirmText}>确认购买</Text>
                      </>
                    )}
                  </TouchableOpacity>
                </View>
              </>
            )}
          </View>
        </View>
      </Modal>

      {/* 出价弹窗 */}
      <Modal
        visible={showBidModal}
        transparent
        animationType="fade"
        onRequestClose={() => setShowBidModal(false)}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <TouchableOpacity
              style={styles.modalClose}
              onPress={() => setShowBidModal(false)}
            >
              <X size={24} color="#888" />
            </TouchableOpacity>
            
            <Text style={styles.modalTitle}>出价竞拍</Text>
            
            {selectedListing && (
              <>
                <View style={styles.modalPetInfo}>
                  <View style={[styles.modalPetAvatar, { borderColor: RARITY_COLORS[selectedListing.rarity] }]}>
                    <Text style={styles.modalPetEmoji}>🐱</Text>
                  </View>
                  <Text style={styles.modalPetName}>{selectedListing.petName}</Text>
                  <View style={styles.modalPetMeta}>
                    <Text style={[styles.modalRarity, { color: RARITY_COLORS[selectedListing.rarity] }]}>
                      {selectedListing.rarity.toUpperCase()}
                    </Text>
                    <Text style={styles.modalLevel}>Lv.{selectedListing.level}</Text>
                  </View>
                </View>
                
                <View style={styles.modalBidInfo}>
                  <View style={styles.bidInfoRow}>
                    <Text style={styles.bidInfoLabel}>起拍价</Text>
                    <Text style={styles.bidInfoValue}>{selectedListing.price} COS</Text>
                  </View>
                  <View style={styles.bidInfoRow}>
                    <Text style={styles.bidInfoLabel}>当前最高出价</Text>
                    <Text style={[styles.bidInfoValue, styles.bidHighest]}>
                      {selectedListing.highestBid || selectedListing.price} COS
                    </Text>
                  </View>
                  <View style={styles.bidInfoRow}>
                    <Text style={styles.bidInfoLabel}>剩余时间</Text>
                    <Text style={styles.bidInfoValue}>{selectedListing.endsAt}</Text>
                  </View>
                </View>
                
                <View style={styles.bidInputSection}>
                  <Text style={styles.bidInputLabel}>你的出价 (COS)</Text>
                  <View style={styles.bidInputWrapper}>
                    <TextInput
                      style={styles.bidInput}
                      value={bidAmount}
                      onChangeText={setBidAmount}
                      keyboardType="numeric"
                      placeholder={`最低 ${(selectedListing.highestBid || selectedListing.price) + 1}`}
                      placeholderTextColor="#666"
                    />
                    <Text style={styles.bidInputSuffix}>COS</Text>
                  </View>
                  <Text style={styles.bidMinHint}>
                    最低出价: {(selectedListing.highestBid || selectedListing.price) + 1} COS
                  </Text>
                </View>
                
                <View style={styles.modalWarning}>
                  <AlertCircle size={16} color="#F7DC6F" />
                  <Text style={styles.modalWarningText}>出价成功后金额将被冻结，如被超越将自动退还</Text>
                </View>
                
                <View style={styles.modalActions}>
                  <TouchableOpacity
                    style={styles.modalCancelButton}
                    onPress={() => setShowBidModal(false)}
                  >
                    <Text style={styles.modalCancelText}>取消</Text>
                  </TouchableOpacity>
                  <TouchableOpacity
                    style={[styles.modalBidButton, isPurchasing && styles.modalButtonDisabled]}
                    onPress={confirmBid}
                    disabled={isPurchasing}
                  >
                    {isPurchasing ? (
                      <Text style={styles.modalConfirmText}>处理中...</Text>
                    ) : (
                      <>
                        <Gavel size={18} color="#fff" />
                        <Text style={styles.modalConfirmText}>确认出价</Text>
                      </>
                    )}
                  </TouchableOpacity>
                </View>
              </>
            )}
          </View>
        </View>
      </Modal>
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
  bidButton: {
    backgroundColor: '#F7DC6F',
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 8,
    marginTop: 8,
  },
  bidButtonText: {
    color: '#000',
    fontWeight: 'bold',
    fontSize: 14,
  },
  // Modal styles
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modalContent: {
    backgroundColor: '#1a1a2e',
    borderRadius: 20,
    padding: 24,
    width: '100%',
    maxWidth: 400,
  },
  modalClose: {
    position: 'absolute',
    top: 16,
    right: 16,
    zIndex: 1,
  },
  modalTitle: {
    fontSize: 22,
    fontWeight: 'bold',
    color: '#fff',
    textAlign: 'center',
    marginBottom: 20,
  },
  modalPetInfo: {
    alignItems: 'center',
    marginBottom: 20,
  },
  modalPetAvatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#252540',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 3,
    marginBottom: 12,
  },
  modalPetEmoji: {
    fontSize: 40,
  },
  modalPetName: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 8,
  },
  modalPetMeta: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  modalRarity: {
    fontSize: 12,
    fontWeight: 'bold',
    marginRight: 12,
  },
  modalLevel: {
    fontSize: 14,
    color: '#888',
  },
  modalPriceSection: {
    alignItems: 'center',
    marginBottom: 20,
  },
  modalPriceLabel: {
    fontSize: 14,
    color: '#888',
    marginBottom: 4,
  },
  modalPrice: {
    fontSize: 32,
    fontWeight: 'bold',
    color: '#4ECDC4',
  },
  modalFeeSection: {
    backgroundColor: '#252540',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  modalFeeRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  modalFeeLabel: {
    fontSize: 14,
    color: '#888',
  },
  modalFeeValue: {
    fontSize: 14,
    color: '#fff',
  },
  modalTotalRow: {
    borderTopWidth: 1,
    borderTopColor: '#333',
    paddingTop: 8,
    marginTop: 4,
    marginBottom: 0,
  },
  modalTotalLabel: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
  },
  modalTotalValue: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#4ECDC4',
  },
  modalWarning: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F7DC6F15',
    padding: 12,
    borderRadius: 8,
    marginBottom: 20,
  },
  modalWarningText: {
    fontSize: 12,
    color: '#F7DC6F',
    marginLeft: 8,
    flex: 1,
  },
  modalActions: {
    flexDirection: 'row',
  },
  modalCancelButton: {
    flex: 1,
    backgroundColor: '#252540',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
    marginRight: 8,
  },
  modalCancelText: {
    fontSize: 16,
    color: '#888',
    fontWeight: '600',
  },
  modalConfirmButton: {
    flex: 1,
    backgroundColor: '#FF6B6B',
    paddingVertical: 14,
    borderRadius: 12,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    marginLeft: 8,
  },
  modalBidButton: {
    flex: 1,
    backgroundColor: '#F7DC6F',
    paddingVertical: 14,
    borderRadius: 12,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    marginLeft: 8,
  },
  modalButtonDisabled: {
    opacity: 0.6,
  },
  modalConfirmText: {
    fontSize: 16,
    color: '#fff',
    fontWeight: 'bold',
    marginLeft: 6,
  },
  modalBidInfo: {
    backgroundColor: '#252540',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  bidInfoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  bidInfoLabel: {
    fontSize: 14,
    color: '#888',
  },
  bidInfoValue: {
    fontSize: 14,
    color: '#fff',
  },
  bidHighest: {
    color: '#F7DC6F',
    fontWeight: 'bold',
  },
  bidInputSection: {
    marginBottom: 16,
  },
  bidInputLabel: {
    fontSize: 14,
    color: '#888',
    marginBottom: 8,
  },
  bidInputWrapper: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#252540',
    borderRadius: 12,
    paddingHorizontal: 16,
  },
  bidInput: {
    flex: 1,
    fontSize: 20,
    color: '#fff',
    paddingVertical: 14,
  },
  bidInputSuffix: {
    fontSize: 14,
    color: '#888',
  },
  bidMinHint: {
    fontSize: 12,
    color: '#666',
    marginTop: 8,
  },
});
