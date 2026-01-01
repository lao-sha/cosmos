/**
 * 星尘玄鉴 - 首页
 * 参考样式：专业占卜入口风格
 * 主题色：金棕色 #B2955D
 */

import { View, Text, StyleSheet, ScrollView, Pressable, Alert } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { RealtimeClock, FourPillarsInfo } from '../../src/components/RealtimeClock';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

// 占卜模块配置
const DIVINATION_MODULES = [
  {
    id: 'bazi',
    name: '八字排盘',
    subtitle: '四柱推命',
    description: '年月日时四柱八字，推算一生命运格局',
    category: '命理',
    color: '#E74C3C',
    route: '/divination/bazi',
  },
  {
    id: 'ziwei',
    name: '紫微斗数',
    subtitle: '东方星命',
    description: '十四主星十二宫位，详解人生各个领域',
    category: '命理',
    color: '#9B59B6',
    route: '/divination/ziwei',
  },
  {
    id: 'qimen',
    name: '奇门遁甲',
    subtitle: '帝王之术',
    description: '天地人神四盘合一，预测决策最高法门',
    category: '术数',
    color: '#3498DB',
    route: '/divination/qimen',
  },
  {
    id: 'daliuren',
    name: '大六壬',
    subtitle: '三式之首',
    description: '天地盘四课三传，精准预测事物发展',
    category: '术数',
    color: '#1ABC9C',
    route: '/divination/daliuren',
  },
  {
    id: 'liuyao',
    name: '六爻排盘',
    subtitle: '纳甲筮法',
    description: '铜钱摇卦六亲六神，断事精准详实',
    category: '占卜',
    color: '#F39C12',
    route: '/divination/liuyao',
  },
  {
    id: 'meihua',
    name: '梅花易数',
    subtitle: '邵雍心易',
    description: '万物皆可起卦，体用生克速断吉凶',
    category: '占卜',
    color: '#E91E63',
    route: '/divination/meihua',
  },
  {
    id: 'tarot',
    name: '塔罗占卜',
    subtitle: '西方神秘',
    description: '大阿尔卡纳22张牌，指引人生方向',
    category: '占卜',
    color: '#673AB7',
    route: '/divination/tarot',
  },
  {
    id: 'xiaoliuren',
    name: '小六壬',
    subtitle: '马前课法',
    description: '月日时三步掐算，快速得出吉凶',
    category: '速断',
    color: '#00BCD4',
    route: '/divination/xiaoliuren',
  },
];

// 分类配置
const CATEGORIES = ['命理', '术数', '占卜', '速断'];

export default function HomePage() {
  const router = useRouter();

  // 处理即时起局
  const handleStartDivination = (timestamp: Date, fourPillars: FourPillarsInfo) => {
    // 可以跳转到具体的占卜页面，携带时间参数
    Alert.alert(
      '即时起局',
      `时间: ${timestamp.toLocaleString()}\n四柱: ${['甲乙丙丁戊己庚辛壬癸'][fourPillars.yearGan]}${['子丑寅卯辰巳午未申酉戌亥'][fourPillars.yearZhi]}年`,
      [
        { text: '奇门遁甲', onPress: () => router.push('/divination/qimen' as any) },
        { text: '大六壬', onPress: () => router.push('/divination/daliuren' as any) },
        { text: '取消', style: 'cancel' },
      ]
    );
  };

  return (
    <View style={styles.container}>
      {/* 顶部标题区 */}
      <View style={styles.header}>
        <Text style={styles.title}>星尘玄鉴</Text>
        <Text style={styles.subtitle}>传统术数 · 链上存证</Text>
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.scrollContent}>
        {/* Banner 区域 */}
        <View style={styles.bannerCard}>
          <Text style={styles.bannerTitle}>探索命运奥秘</Text>
          <Text style={styles.bannerDesc}>
            基于 Substrate 区块链的玄学占卜平台{'\n'}链上记录，不可篡改
          </Text>
          <View style={styles.bannerStats}>
            <View style={styles.statItem}>
              <Text style={styles.statValue}>8</Text>
              <Text style={styles.statLabel}>占卜术数</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={styles.statValue}>64</Text>
              <Text style={styles.statLabel}>周易卦象</Text>
            </View>
            <View style={styles.statDivider} />
            <View style={styles.statItem}>
              <Text style={styles.statValue}>22</Text>
              <Text style={styles.statLabel}>塔罗大牌</Text>
            </View>
          </View>
        </View>

        {/* 实时时钟干支 */}
        <View style={styles.clockCard}>
          <RealtimeClock
            onStartDivination={handleStartDivination}
            buttonText="即时起局"
          />
        </View>

        {/* 功能说明 */}
        <View style={styles.featuresCard}>
          <View style={styles.featuresRow}>
            <View style={styles.featureItem}>
              <Ionicons name="shield-checkmark-outline" size={24} color={THEME_COLOR} />
              <Text style={styles.featureTitle}>链上存证</Text>
              <Text style={styles.featureDesc}>结果永久保存</Text>
            </View>
            <View style={styles.featureDivider} />
            <View style={styles.featureItem}>
              <Ionicons name="lock-closed-outline" size={24} color={THEME_COLOR} />
              <Text style={styles.featureTitle}>隐私加密</Text>
              <Text style={styles.featureDesc}>数据安全可控</Text>
            </View>
            <View style={styles.featureDivider} />
            <View style={styles.featureItem}>
              <Ionicons name="sparkles-outline" size={24} color={THEME_COLOR} />
              <Text style={styles.featureTitle}>AI解读</Text>
              <Text style={styles.featureDesc}>智能分析结果</Text>
            </View>
          </View>
        </View>

        {/* 分类导航 */}
        <View style={styles.categorySection}>
          <Text style={styles.sectionTitle}>占卜术数</Text>
          <View style={styles.categoryRow}>
            {CATEGORIES.map((cat) => (
              <View key={cat} style={styles.categoryTag}>
                <Text style={styles.categoryText}>{cat}</Text>
              </View>
            ))}
          </View>
        </View>

        {/* 占卜模块列表 */}
        <View style={styles.modulesSection}>
          {DIVINATION_MODULES.map((module) => (
            <Pressable
              key={module.id}
              style={styles.moduleCard}
              onPress={() => router.push(module.route as any)}
            >
              <View style={styles.moduleHeader}>
                <View style={[styles.moduleBadge, { backgroundColor: module.color + '20' }]}>
                  <Text style={[styles.moduleBadgeText, { color: module.color }]}>{module.category}</Text>
                </View>
                <Ionicons name="chevron-forward" size={20} color="#CCC" />
              </View>
              <View style={styles.moduleContent}>
                <View style={styles.moduleTitleRow}>
                  <Text style={styles.moduleName}>{module.name}</Text>
                  <Text style={styles.moduleSubtitle}>{module.subtitle}</Text>
                </View>
                <Text style={styles.moduleDesc}>{module.description}</Text>
              </View>
              <View style={[styles.moduleAccent, { backgroundColor: module.color }]} />
            </Pressable>
          ))}
        </View>

        {/* 平台特色 */}
        <View style={styles.platformSection}>
          <Text style={styles.sectionTitle}>平台特色</Text>
          <View style={styles.platformCard}>
            <View style={styles.platformItem}>
              <View style={styles.platformIcon}>
                <Ionicons name="link-outline" size={20} color={THEME_COLOR} />
              </View>
              <View style={styles.platformContent}>
                <Text style={styles.platformTitle}>Substrate 区块链</Text>
                <Text style={styles.platformText}>所有占卜结果永久上链，确保真实不可篡改</Text>
              </View>
            </View>
            <View style={styles.platformDivider} />
            <View style={styles.platformItem}>
              <View style={styles.platformIcon}>
                <Ionicons name="finger-print-outline" size={20} color={THEME_COLOR} />
              </View>
              <View style={styles.platformContent}>
                <Text style={styles.platformTitle}>零知识证明</Text>
                <Text style={styles.platformText}>保护个人隐私信息，仅本人可查看详情</Text>
              </View>
            </View>
            <View style={styles.platformDivider} />
            <View style={styles.platformItem}>
              <View style={styles.platformIcon}>
                <Ionicons name="analytics-outline" size={20} color={THEME_COLOR} />
              </View>
              <View style={styles.platformContent}>
                <Text style={styles.platformTitle}>AI 智能解读</Text>
                <Text style={styles.platformText}>结合传统命理与人工智能，提供专业解析</Text>
              </View>
            </View>
          </View>
        </View>

        {/* 底部说明 */}
        <View style={styles.footerSection}>
          <Text style={styles.footerText}>
            星尘玄鉴 - 传统文化与区块链的完美结合
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
  bannerCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 24,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
    borderLeftWidth: 4,
    borderLeftColor: THEME_COLOR,
  },
  clockCard: {
    marginBottom: 12,
  },
  bannerTitle: {
    fontSize: 22,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  bannerDesc: {
    fontSize: 14,
    color: '#666',
    lineHeight: 22,
    marginBottom: 20,
  },
  bannerStats: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FAFAFA',
    borderRadius: 4,
    padding: 12,
  },
  statItem: {
    flex: 1,
    alignItems: 'center',
  },
  statValue: {
    fontSize: 24,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  statLabel: {
    fontSize: 11,
    color: '#999',
    marginTop: 2,
  },
  statDivider: {
    width: 1,
    height: 30,
    backgroundColor: '#E8E8E8',
  },
  featuresCard: {
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
  featuresRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  featureItem: {
    flex: 1,
    alignItems: 'center',
  },
  featureDivider: {
    width: 1,
    height: 50,
    backgroundColor: '#F0F0F0',
  },
  featureTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginTop: 8,
  },
  featureDesc: {
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
  categoryText: {
    fontSize: 14,
    color: '#666',
  },
  modulesSection: {
    gap: 12,
    marginBottom: 24,
  },
  moduleCard: {
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
  moduleHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  moduleBadge: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 4,
  },
  moduleBadgeText: {
    fontSize: 12,
    fontWeight: '600',
  },
  moduleContent: {},
  moduleTitleRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
    gap: 8,
    marginBottom: 6,
  },
  moduleName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  moduleSubtitle: {
    fontSize: 13,
    color: '#999',
  },
  moduleDesc: {
    fontSize: 14,
    color: '#666',
    lineHeight: 20,
  },
  moduleAccent: {
    position: 'absolute',
    left: 0,
    top: 0,
    bottom: 0,
    width: 4,
  },
  platformSection: {
    marginBottom: 24,
  },
  platformCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
  },
  platformItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    gap: 16,
  },
  platformIcon: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: THEME_COLOR + '15',
    justifyContent: 'center',
    alignItems: 'center',
  },
  platformContent: {
    flex: 1,
  },
  platformTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  platformText: {
    fontSize: 13,
    color: '#999',
  },
  platformDivider: {
    height: 1,
    backgroundColor: '#F0F0F0',
  },
  footerSection: {
    alignItems: 'center',
    paddingVertical: 16,
  },
  footerText: {
    fontSize: 12,
    color: '#BBB',
    marginBottom: 4,
  },
});
