/**
 * 星尘玄鉴 - 占卜入口页
 * 参考样式：专业占卜入口风格
 * 主题色：金棕色 #B2955D
 */

import { View, Text, StyleSheet, Pressable, ScrollView } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

// 占卜类型配置
const DIVINATION_TYPES = [
  {
    id: 'bazi',
    name: '八字排盘',
    subtitle: '四柱推命',
    desc: '年月日时四柱八字，推算一生命运格局',
    category: '命理',
    route: '/divination/bazi',
    color: '#E74C3C',
  },
  {
    id: 'ziwei',
    name: '紫微斗数',
    subtitle: '东方星命',
    desc: '十四主星十二宫位，详解人生各个领域',
    category: '命理',
    route: '/divination/ziwei',
    color: '#9B59B6',
  },
  {
    id: 'qimen',
    name: '奇门遁甲',
    subtitle: '帝王之术',
    desc: '天地人神四盘合一，预测决策最高法门',
    category: '术数',
    route: '/divination/qimen',
    color: '#3498DB',
  },
  {
    id: 'daliuren',
    name: '大六壬',
    subtitle: '三式之首',
    desc: '天地盘四课三传，精准预测事物发展',
    category: '术数',
    route: '/divination/daliuren',
    color: '#1ABC9C',
  },
  {
    id: 'liuyao',
    name: '六爻排盘',
    subtitle: '纳甲筮法',
    desc: '铜钱摇卦六亲六神，断事精准详实',
    category: '占卜',
    route: '/divination/liuyao',
    color: '#F39C12',
  },
  {
    id: 'meihua',
    name: '梅花易数',
    subtitle: '邵雍心易',
    desc: '万物皆可起卦，体用生克速断吉凶',
    category: '占卜',
    route: '/divination/meihua',
    color: '#E91E63',
  },
  {
    id: 'tarot',
    name: '塔罗占卜',
    subtitle: '西方神秘',
    desc: '大阿尔卡纳22张牌，指引人生方向',
    category: '占卜',
    route: '/divination/tarot',
    color: '#673AB7',
  },
  {
    id: 'xiaoliuren',
    name: '小六壬',
    subtitle: '马前课法',
    desc: '月日时三步掐算，快速得出吉凶',
    category: '速断',
    route: '/divination/xiaoliuren',
    color: '#00BCD4',
  },
];

// 分类配置
const CATEGORIES = [
  { id: 'all', name: '全部' },
  { id: '命理', name: '命理' },
  { id: '术数', name: '术数' },
  { id: '占卜', name: '占卜' },
  { id: '速断', name: '速断' },
];

export default function DivinationPage() {
  const router = useRouter();

  return (
    <View style={styles.container}>
      {/* 顶部标题区 */}
      <View style={styles.header}>
        <Text style={styles.title}>星尘玄鉴</Text>
        <Text style={styles.subtitle}>传统术数 · 链上存证</Text>
      </View>

      {/* 内容区 */}
      <ScrollView style={styles.content} contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
        {/* 功能说明 */}
        <View style={styles.introCard}>
          <View style={styles.introRow}>
            <View style={styles.introItem}>
              <Ionicons name="shield-checkmark-outline" size={24} color={THEME_COLOR} />
              <Text style={styles.introTitle}>链上存证</Text>
              <Text style={styles.introDesc}>结果永久保存</Text>
            </View>
            <View style={styles.introDivider} />
            <View style={styles.introItem}>
              <Ionicons name="lock-closed-outline" size={24} color={THEME_COLOR} />
              <Text style={styles.introTitle}>隐私加密</Text>
              <Text style={styles.introDesc}>数据安全可控</Text>
            </View>
            <View style={styles.introDivider} />
            <View style={styles.introItem}>
              <Ionicons name="sparkles-outline" size={24} color={THEME_COLOR} />
              <Text style={styles.introTitle}>AI解读</Text>
              <Text style={styles.introDesc}>智能分析结果</Text>
            </View>
          </View>
        </View>

        {/* 分类标签 */}
        <View style={styles.categorySection}>
          <Text style={styles.sectionTitle}>占卜类型</Text>
          <ScrollView horizontal showsHorizontalScrollIndicator={false} style={styles.categoryScroll}>
            <View style={styles.categoryRow}>
              {CATEGORIES.map((cat) => (
                <View
                  key={cat.id}
                  style={[styles.categoryTag, cat.id === 'all' && styles.categoryTagActive]}
                >
                  <Text style={[styles.categoryText, cat.id === 'all' && styles.categoryTextActive]}>
                    {cat.name}
                  </Text>
                </View>
              ))}
            </View>
          </ScrollView>
        </View>

        {/* 占卜类型列表 */}
        <View style={styles.typesSection}>
          {DIVINATION_TYPES.map((type) => (
            <Pressable
              key={type.id}
              style={styles.typeCard}
              onPress={() => router.push(type.route as any)}
            >
              <View style={styles.typeHeader}>
                <View style={[styles.typeBadge, { backgroundColor: type.color + '20' }]}>
                  <Text style={[styles.typeBadgeText, { color: type.color }]}>{type.category}</Text>
                </View>
                <Ionicons name="chevron-forward" size={20} color="#CCC" />
              </View>
              <View style={styles.typeContent}>
                <View style={styles.typeTitleRow}>
                  <Text style={styles.typeName}>{type.name}</Text>
                  <Text style={styles.typeSubtitle}>{type.subtitle}</Text>
                </View>
                <Text style={styles.typeDesc}>{type.desc}</Text>
              </View>
              <View style={[styles.typeAccent, { backgroundColor: type.color }]} />
            </Pressable>
          ))}
        </View>

        {/* 底部说明 */}
        <View style={styles.footerSection}>
          <Text style={styles.footerText}>
            所有占卜结果将通过区块链技术永久存储
          </Text>
          <Text style={styles.footerText}>
            仅供娱乐参考，请理性对待
          </Text>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  header: {
    paddingTop: 60,
    paddingHorizontal: 20,
    paddingBottom: 20,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  title: {
    fontSize: 28,
    fontWeight: '600',
    color: '#333',
  },
  subtitle: {
    fontSize: 14,
    color: THEME_COLOR,
    marginTop: 4,
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 12,
    paddingBottom: 100,
  },
  introCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 20,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
  },
  introRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  introItem: {
    flex: 1,
    alignItems: 'center',
  },
  introDivider: {
    width: 1,
    height: 50,
    backgroundColor: '#F0F0F0',
  },
  introTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginTop: 8,
  },
  introDesc: {
    fontSize: 11,
    color: '#999',
    marginTop: 2,
  },
  categorySection: {
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 12,
    paddingHorizontal: 4,
  },
  categoryScroll: {
    marginHorizontal: -12,
    paddingHorizontal: 12,
  },
  categoryRow: {
    flexDirection: 'row',
    gap: 8,
  },
  categoryTag: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    backgroundColor: '#FFF',
  },
  categoryTagActive: {
    backgroundColor: THEME_COLOR,
    borderColor: THEME_COLOR,
  },
  categoryText: {
    fontSize: 14,
    color: '#666',
  },
  categoryTextActive: {
    color: '#FFF',
  },
  typesSection: {
    gap: 12,
  },
  typeCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    position: 'relative',
    overflow: 'hidden',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
  },
  typeHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  typeBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 4,
  },
  typeBadgeText: {
    fontSize: 12,
    fontWeight: '600',
  },
  typeContent: {},
  typeTitleRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
    gap: 8,
    marginBottom: 6,
  },
  typeName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  typeSubtitle: {
    fontSize: 13,
    color: '#999',
  },
  typeDesc: {
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
  },
  typeAccent: {
    position: 'absolute',
    left: 0,
    top: 0,
    bottom: 0,
    width: 4,
  },
  footerSection: {
    marginTop: 24,
    alignItems: 'center',
    paddingVertical: 16,
  },
  footerText: {
    fontSize: 12,
    color: '#BBB',
    marginBottom: 4,
  },
});
