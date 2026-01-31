import React, { useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  TextInput,
  Platform,
} from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import {
  ArrowLeft,
  Tag,
  Clock,
  TrendingUp,
  AlertCircle,
  Check,
  Flame,
  Shield,
  Zap,
  Heart,
  Star,
  Info,
} from 'lucide-react-native';

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

// 稀有度配置
const RARITY_CONFIG = {
  common: { color: '#888', name: '普通', multiplier: 1 },
  rare: { color: '#4ECDC4', name: '稀有', multiplier: 2 },
  epic: { color: '#BB8FCE', name: '史诗', multiplier: 5 },
  legendary: { color: '#F7DC6F', name: '传说', multiplier: 15 },
  mythic: { color: '#FF6B6B', name: '神话', multiplier: 50 },
};

// 元素配置
const ELEMENT_CONFIG = {
  normal: { icon: Star, color: '#888', name: '普通' },
  fire: { icon: Flame, color: '#FF6B6B', name: '火焰' },
  water: { icon: Star, color: '#45B7D1', name: '水' },
  light: { icon: Star, color: '#F7DC6F', name: '光明' },
  shadow: { icon: Star, color: '#BB8FCE', name: '暗影' },
};

type SaleType = 'fixed' | 'auction';
type Duration = '1d' | '3d' | '7d' | '14d';

export default function SellPetScreen() {
  const router = useRouter();
  const params = useLocalSearchParams();
  const petId = params.id || '1';

  // 模拟宠物数据
  const pet = {
    id: Number(petId),
    name: '小火',
    element: 'fire' as const,
    rarity: 'rare' as const,
    level: 15,
    attack: 45,
    defense: 30,
    speed: 50,
    hp: 120,
    evolutionStage: 1,
  };

  // 表单状态
  const [saleType, setSaleType] = useState<SaleType>('fixed');
  const [price, setPrice] = useState('');
  const [minBid, setMinBid] = useState('');
  const [buyNowPrice, setBuyNowPrice] = useState('');
  const [duration, setDuration] = useState<Duration>('3d');
  const [isSubmitting, setIsSubmitting] = useState(false);

  // 计算建议价格
  const calculateSuggestedPrice = () => {
    const basePrice = 10;
    const levelBonus = pet.level * 2;
    const rarityMultiplier = RARITY_CONFIG[pet.rarity].multiplier;
    const evolutionBonus = pet.evolutionStage * 20;
    const statsBonus = Math.floor((pet.attack + pet.defense + pet.speed) / 10);
    
    return Math.floor((basePrice + levelBonus + evolutionBonus + statsBonus) * rarityMultiplier);
  };

  const suggestedPrice = calculateSuggestedPrice();

  // 市场参考价格
  const marketPrices = {
    min: Math.floor(suggestedPrice * 0.7),
    avg: suggestedPrice,
    max: Math.floor(suggestedPrice * 1.5),
  };

  // 手续费计算
  const calculateFee = (amount: number) => {
    return Math.floor(amount * 0.025); // 2.5% 手续费
  };

  // 提交挂单
  const handleSubmit = () => {
    if (saleType === 'fixed' && !price) {
      showAlert('错误', '请输入出售价格');
      return;
    }
    if (saleType === 'auction' && !minBid) {
      showAlert('错误', '请输入起拍价');
      return;
    }

    const salePrice = saleType === 'fixed' ? Number(price) : Number(minBid);
    if (salePrice <= 0) {
      showAlert('错误', '价格必须大于 0');
      return;
    }

    setIsSubmitting(true);

    // 模拟提交
    setTimeout(() => {
      setIsSubmitting(false);
      const fee = calculateFee(salePrice);
      showAlert(
        '挂单成功！',
        `${pet.name} 已成功挂单出售\n\n` +
        `出售方式: ${saleType === 'fixed' ? '一口价' : '拍卖'}\n` +
        `${saleType === 'fixed' ? '价格' : '起拍价'}: ${salePrice} COS\n` +
        `手续费: ${fee} COS (2.5%)\n` +
        `挂单时长: ${duration === '1d' ? '1天' : duration === '3d' ? '3天' : duration === '7d' ? '7天' : '14天'}`,
        () => router.back()
      );
    }, 1500);
  };

  const ElementIcon = ELEMENT_CONFIG[pet.element].icon;

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backButton}>
          <ArrowLeft size={24} color="#fff" />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>出售宠物</Text>
        <View style={{ width: 40 }} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 宠物信息卡片 */}
        <View style={styles.petCard}>
          <View style={styles.petAvatar}>
            <Text style={styles.petEmoji}>🐱</Text>
            <View style={[styles.elementBadge, { backgroundColor: ELEMENT_CONFIG[pet.element].color + '30' }]}>
              <ElementIcon size={14} color={ELEMENT_CONFIG[pet.element].color} />
            </View>
          </View>
          <View style={styles.petInfo}>
            <Text style={styles.petName}>{pet.name}</Text>
            <View style={styles.petBadges}>
              <View style={[styles.rarityBadge, { backgroundColor: RARITY_CONFIG[pet.rarity].color + '30' }]}>
                <Text style={[styles.rarityText, { color: RARITY_CONFIG[pet.rarity].color }]}>
                  {RARITY_CONFIG[pet.rarity].name}
                </Text>
              </View>
              <Text style={styles.levelText}>Lv.{pet.level}</Text>
            </View>
            <View style={styles.petStats}>
              <View style={styles.statItem}>
                <Zap size={12} color="#F7DC6F" />
                <Text style={styles.statText}>{pet.attack}</Text>
              </View>
              <View style={styles.statItem}>
                <Shield size={12} color="#4ECDC4" />
                <Text style={styles.statText}>{pet.defense}</Text>
              </View>
              <View style={styles.statItem}>
                <Heart size={12} color="#FF6B6B" />
                <Text style={styles.statText}>{pet.hp}</Text>
              </View>
            </View>
          </View>
        </View>

        {/* 市场参考价 */}
        <View style={styles.marketCard}>
          <View style={styles.marketHeader}>
            <TrendingUp size={18} color="#4ECDC4" />
            <Text style={styles.marketTitle}>市场参考价</Text>
          </View>
          <View style={styles.marketPrices}>
            <View style={styles.priceItem}>
              <Text style={styles.priceLabel}>最低</Text>
              <Text style={styles.priceValue}>{marketPrices.min} COS</Text>
            </View>
            <View style={styles.priceItem}>
              <Text style={styles.priceLabel}>平均</Text>
              <Text style={[styles.priceValue, styles.avgPrice]}>{marketPrices.avg} COS</Text>
            </View>
            <View style={styles.priceItem}>
              <Text style={styles.priceLabel}>最高</Text>
              <Text style={styles.priceValue}>{marketPrices.max} COS</Text>
            </View>
          </View>
          <TouchableOpacity
            style={styles.suggestButton}
            onPress={() => {
              if (saleType === 'fixed') {
                setPrice(String(suggestedPrice));
              } else {
                setMinBid(String(Math.floor(suggestedPrice * 0.8)));
                setBuyNowPrice(String(Math.floor(suggestedPrice * 1.2)));
              }
            }}
          >
            <Text style={styles.suggestButtonText}>使用建议价格</Text>
          </TouchableOpacity>
        </View>

        {/* 出售方式 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>出售方式</Text>
          <View style={styles.saleTypeContainer}>
            <TouchableOpacity
              style={[styles.saleTypeCard, saleType === 'fixed' && styles.saleTypeCardActive]}
              onPress={() => setSaleType('fixed')}
            >
              <Tag size={24} color={saleType === 'fixed' ? '#4ECDC4' : '#666'} />
              <Text style={[styles.saleTypeTitle, saleType === 'fixed' && styles.saleTypeTitleActive]}>
                一口价
              </Text>
              <Text style={styles.saleTypeDesc}>设定固定价格</Text>
            </TouchableOpacity>
            <TouchableOpacity
              style={[styles.saleTypeCard, saleType === 'auction' && styles.saleTypeCardActive]}
              onPress={() => setSaleType('auction')}
            >
              <TrendingUp size={24} color={saleType === 'auction' ? '#FF6B6B' : '#666'} />
              <Text style={[styles.saleTypeTitle, saleType === 'auction' && styles.saleTypeTitleActive]}>
                拍卖
              </Text>
              <Text style={styles.saleTypeDesc}>竞价出售</Text>
            </TouchableOpacity>
          </View>
        </View>

        {/* 价格设置 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>
            {saleType === 'fixed' ? '出售价格' : '拍卖设置'}
          </Text>
          
          {saleType === 'fixed' ? (
            <View style={styles.inputContainer}>
              <Text style={styles.inputLabel}>价格 (COS)</Text>
              <View style={styles.inputWrapper}>
                <TextInput
                  style={styles.input}
                  value={price}
                  onChangeText={setPrice}
                  placeholder="输入价格"
                  placeholderTextColor="#666"
                  keyboardType="numeric"
                />
                <Text style={styles.inputSuffix}>COS</Text>
              </View>
              {price && Number(price) > 0 && (
                <Text style={styles.feeText}>
                  手续费: {calculateFee(Number(price))} COS (2.5%)
                </Text>
              )}
            </View>
          ) : (
            <>
              <View style={styles.inputContainer}>
                <Text style={styles.inputLabel}>起拍价 (COS)</Text>
                <View style={styles.inputWrapper}>
                  <TextInput
                    style={styles.input}
                    value={minBid}
                    onChangeText={setMinBid}
                    placeholder="输入起拍价"
                    placeholderTextColor="#666"
                    keyboardType="numeric"
                  />
                  <Text style={styles.inputSuffix}>COS</Text>
                </View>
              </View>
              <View style={styles.inputContainer}>
                <Text style={styles.inputLabel}>一口价 (可选)</Text>
                <View style={styles.inputWrapper}>
                  <TextInput
                    style={styles.input}
                    value={buyNowPrice}
                    onChangeText={setBuyNowPrice}
                    placeholder="输入一口价"
                    placeholderTextColor="#666"
                    keyboardType="numeric"
                  />
                  <Text style={styles.inputSuffix}>COS</Text>
                </View>
              </View>
              {minBid && Number(minBid) > 0 && (
                <Text style={styles.feeText}>
                  手续费: 成交价的 2.5%
                </Text>
              )}
            </>
          )}
        </View>

        {/* 挂单时长 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>挂单时长</Text>
          <View style={styles.durationContainer}>
            {(['1d', '3d', '7d', '14d'] as Duration[]).map((d) => (
              <TouchableOpacity
                key={d}
                style={[styles.durationCard, duration === d && styles.durationCardActive]}
                onPress={() => setDuration(d)}
              >
                <Clock size={16} color={duration === d ? '#4ECDC4' : '#666'} />
                <Text style={[styles.durationText, duration === d && styles.durationTextActive]}>
                  {d === '1d' ? '1天' : d === '3d' ? '3天' : d === '7d' ? '7天' : '14天'}
                </Text>
              </TouchableOpacity>
            ))}
          </View>
        </View>

        {/* 注意事项 */}
        <View style={styles.noticeCard}>
          <View style={styles.noticeHeader}>
            <AlertCircle size={18} color="#F7DC6F" />
            <Text style={styles.noticeTitle}>注意事项</Text>
          </View>
          <View style={styles.noticeList}>
            <View style={styles.noticeItem}>
              <Info size={14} color="#888" />
              <Text style={styles.noticeText}>挂单期间宠物将被锁定，无法使用</Text>
            </View>
            <View style={styles.noticeItem}>
              <Info size={14} color="#888" />
              <Text style={styles.noticeText}>成交后收取 2.5% 手续费</Text>
            </View>
            <View style={styles.noticeItem}>
              <Info size={14} color="#888" />
              <Text style={styles.noticeText}>可随时取消挂单（拍卖中有出价时除外）</Text>
            </View>
            <View style={styles.noticeItem}>
              <Info size={14} color="#888" />
              <Text style={styles.noticeText}>挂单到期未成交将自动下架</Text>
            </View>
          </View>
        </View>

        {/* 底部间距 */}
        <View style={{ height: 100 }} />
      </ScrollView>

      {/* 底部提交按钮 */}
      <View style={styles.footer}>
        <View style={styles.footerInfo}>
          <Text style={styles.footerLabel}>预计收入</Text>
          <Text style={styles.footerValue}>
            {saleType === 'fixed' && price
              ? `${Number(price) - calculateFee(Number(price))} COS`
              : saleType === 'auction' && minBid
              ? `≥ ${Number(minBid) - calculateFee(Number(minBid))} COS`
              : '-- COS'}
          </Text>
        </View>
        <TouchableOpacity
          style={[styles.submitButton, isSubmitting && styles.submitButtonDisabled]}
          onPress={handleSubmit}
          disabled={isSubmitting}
        >
          {isSubmitting ? (
            <Text style={styles.submitButtonText}>提交中...</Text>
          ) : (
            <>
              <Check size={20} color="#fff" />
              <Text style={styles.submitButtonText}>确认挂单</Text>
            </>
          )}
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f0f1a',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingTop: Platform.OS === 'web' ? 16 : 50,
    paddingBottom: 16,
    backgroundColor: '#1a1a2e',
  },
  backButton: {
    padding: 8,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  petCard: {
    flexDirection: 'row',
    backgroundColor: '#1a1a2e',
    borderRadius: 16,
    padding: 16,
    marginBottom: 16,
  },
  petAvatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#252540',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 16,
  },
  petEmoji: {
    fontSize: 40,
  },
  elementBadge: {
    position: 'absolute',
    bottom: 0,
    right: 0,
    padding: 4,
    borderRadius: 10,
  },
  petInfo: {
    flex: 1,
    justifyContent: 'center',
  },
  petName: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 8,
  },
  petBadges: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  rarityBadge: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 6,
    marginRight: 8,
  },
  rarityText: {
    fontSize: 12,
    fontWeight: '600',
  },
  levelText: {
    fontSize: 14,
    color: '#888',
  },
  petStats: {
    flexDirection: 'row',
  },
  statItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginRight: 16,
  },
  statText: {
    fontSize: 12,
    color: '#888',
    marginLeft: 4,
  },
  marketCard: {
    backgroundColor: '#1a1a2e',
    borderRadius: 16,
    padding: 16,
    marginBottom: 16,
  },
  marketHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  marketTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
    marginLeft: 8,
  },
  marketPrices: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    marginBottom: 12,
  },
  priceItem: {
    alignItems: 'center',
  },
  priceLabel: {
    fontSize: 12,
    color: '#888',
    marginBottom: 4,
  },
  priceValue: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
  },
  avgPrice: {
    color: '#4ECDC4',
  },
  suggestButton: {
    backgroundColor: '#4ECDC420',
    paddingVertical: 10,
    borderRadius: 8,
    alignItems: 'center',
  },
  suggestButtonText: {
    fontSize: 14,
    color: '#4ECDC4',
    fontWeight: '600',
  },
  section: {
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 12,
  },
  saleTypeContainer: {
    flexDirection: 'row',
  },
  saleTypeCard: {
    flex: 1,
    backgroundColor: '#1a1a2e',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
    marginHorizontal: 4,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  saleTypeCardActive: {
    borderColor: '#4ECDC4',
  },
  saleTypeTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#888',
    marginTop: 8,
  },
  saleTypeTitleActive: {
    color: '#fff',
  },
  saleTypeDesc: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
  },
  inputContainer: {
    marginBottom: 16,
  },
  inputLabel: {
    fontSize: 14,
    color: '#888',
    marginBottom: 8,
  },
  inputWrapper: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#1a1a2e',
    borderRadius: 12,
    paddingHorizontal: 16,
  },
  input: {
    flex: 1,
    fontSize: 18,
    color: '#fff',
    paddingVertical: 14,
  },
  inputSuffix: {
    fontSize: 14,
    color: '#888',
  },
  feeText: {
    fontSize: 12,
    color: '#F7DC6F',
    marginTop: 8,
  },
  durationContainer: {
    flexDirection: 'row',
  },
  durationCard: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#1a1a2e',
    borderRadius: 10,
    paddingVertical: 12,
    marginHorizontal: 4,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  durationCardActive: {
    borderColor: '#4ECDC4',
  },
  durationText: {
    fontSize: 14,
    color: '#888',
    marginLeft: 6,
  },
  durationTextActive: {
    color: '#4ECDC4',
  },
  noticeCard: {
    backgroundColor: '#F7DC6F10',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#F7DC6F30',
  },
  noticeHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#F7DC6F',
    marginLeft: 8,
  },
  noticeList: {},
  noticeItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 12,
    color: '#888',
    marginLeft: 8,
    flex: 1,
  },
  footer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#1a1a2e',
    paddingHorizontal: 16,
    paddingVertical: 12,
    paddingBottom: Platform.OS === 'web' ? 12 : 30,
    borderTopWidth: 1,
    borderTopColor: '#252540',
  },
  footerInfo: {
    flex: 1,
  },
  footerLabel: {
    fontSize: 12,
    color: '#888',
  },
  footerValue: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#4ECDC4',
  },
  submitButton: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FF6B6B',
    paddingHorizontal: 24,
    paddingVertical: 14,
    borderRadius: 12,
  },
  submitButtonDisabled: {
    backgroundColor: '#666',
  },
  submitButtonText: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
    marginLeft: 8,
  },
});
