/**
 * 星尘玄鉴 - 塔罗占卜
 * 参考样式：专业塔罗排盘风格
 * 主题色：金棕色 #B2955D
 */

import { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  TextInput,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { Picker } from '@react-native-picker/picker';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

// 大阿尔卡纳牌
const MAJOR_ARCANA = [
  { id: 0, name: '愚者', meaning: '新开始、冒险、纯真', reversed: '鲁莽、冒失、不成熟', element: '风' },
  { id: 1, name: '魔术师', meaning: '创造力、技巧、意志力', reversed: '欺骗、操纵、浪费才能', element: '风' },
  { id: 2, name: '女祭司', meaning: '直觉、神秘、潜意识', reversed: '隐藏、欺骗、表面', element: '水' },
  { id: 3, name: '女皇', meaning: '丰收、母性、创造', reversed: '依赖、空虚、缺乏成长', element: '地' },
  { id: 4, name: '皇帝', meaning: '权威、稳定、领导', reversed: '暴君、固执、缺乏纪律', element: '火' },
  { id: 5, name: '教皇', meaning: '传统、信仰、指导', reversed: '挑战权威、打破常规', element: '地' },
  { id: 6, name: '恋人', meaning: '爱情、选择、和谐', reversed: '不和谐、失衡、错误选择', element: '风' },
  { id: 7, name: '战车', meaning: '胜利、意志、决心', reversed: '失败、缺乏方向、攻击性', element: '水' },
  { id: 8, name: '力量', meaning: '勇气、耐心、内在力量', reversed: '软弱、自我怀疑、缺乏勇气', element: '火' },
  { id: 9, name: '隐士', meaning: '内省、指引、智慧', reversed: '孤立、偏执、撤退', element: '地' },
  { id: 10, name: '命运之轮', meaning: '变化、机遇、命运', reversed: '厄运、抵制变化、失控', element: '火' },
  { id: 11, name: '正义', meaning: '公平、真相、因果', reversed: '不公、偏见、逃避责任', element: '风' },
  { id: 12, name: '倒吊人', meaning: '牺牲、释放、新视角', reversed: '拖延、抵抗、无谓牺牲', element: '水' },
  { id: 13, name: '死神', meaning: '结束、转变、新生', reversed: '抗拒变化、停滞', element: '水' },
  { id: 14, name: '节制', meaning: '平衡、耐心、适度', reversed: '失衡、过度、缺乏远见', element: '火' },
  { id: 15, name: '恶魔', meaning: '束缚、诱惑、物质', reversed: '解脱、释放、面对恐惧', element: '地' },
  { id: 16, name: '塔', meaning: '剧变、觉醒、启示', reversed: '逃避灾难、恐惧改变', element: '火' },
  { id: 17, name: '星星', meaning: '希望、灵感、宁静', reversed: '绝望、失望、缺乏信心', element: '风' },
  { id: 18, name: '月亮', meaning: '幻觉、潜意识、恐惧', reversed: '释放恐惧、解开困惑', element: '水' },
  { id: 19, name: '太阳', meaning: '快乐、成功、活力', reversed: '消极、抑郁、缺乏活力', element: '火' },
  { id: 20, name: '审判', meaning: '反思、召唤、复活', reversed: '自我怀疑、拒绝改变', element: '火' },
  { id: 21, name: '世界', meaning: '完成、整合、成就', reversed: '未完成、缺乏闭环', element: '地' },
];

// 牌阵类型
type SpreadType = 'single' | 'three' | 'celtic';

const SPREAD_CONFIG = {
  single: { name: '单牌占卜', count: 1, positions: ['指引'], desc: '快速获得答案' },
  three: { name: '时间之流', count: 3, positions: ['过去', '现在', '未来'], desc: '看清时间脉络' },
  celtic: { name: '五星阵', count: 5, positions: ['现状', '阻碍', '潜意识', '过去', '未来'], desc: '深入探索问题' },
};

// 问事类型
const QUESTION_TYPES = [
  { value: 0, label: '综合运势' },
  { value: 1, label: '事业工作' },
  { value: 2, label: '财运求财' },
  { value: 3, label: '婚姻感情' },
  { value: 4, label: '人际关系' },
  { value: 5, label: '学业考试' },
  { value: 6, label: '健康疾病' },
  { value: 7, label: '心灵指引' },
];

// 抽牌结果
interface DrawnCard {
  card: typeof MAJOR_ARCANA[0];
  reversed: boolean;
  position: string;
}

// 塔罗结果
interface TarotResult {
  id: number;
  name: string;
  question: string;
  questionType: number;
  spreadType: SpreadType;
  drawnCards: DrawnCard[];
  createdAt: Date;
}

export default function TarotPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<TarotResult | null>(null);
  const [history, setHistory] = useState<TarotResult[]>([]);

  // 表单状态
  const [name, setName] = useState('');
  const [question, setQuestion] = useState('');
  const [questionType, setQuestionType] = useState(0);
  const [spreadType, setSpreadType] = useState<SpreadType>('single');

  // 抽牌
  const drawCards = async () => {
    setLoading(true);
    await new Promise(resolve => setTimeout(resolve, 1000));

    const config = SPREAD_CONFIG[spreadType];
    const shuffled = [...MAJOR_ARCANA].sort(() => Math.random() - 0.5);
    const cards: DrawnCard[] = shuffled.slice(0, config.count).map((card, i) => ({
      card,
      reversed: Math.random() > 0.5,
      position: config.positions[i],
    }));

    const tarotResult: TarotResult = {
      id: Date.now(),
      name: name || '求测者',
      question: question || '某事',
      questionType,
      spreadType,
      drawnCards: cards,
      createdAt: new Date(),
    };

    setResult(tarotResult);
    setHistory(prev => [tarotResult, ...prev]);
    setLoading(false);
  };

  const handleReset = () => {
    setResult(null);
  };

  // 获取元素颜色
  const getElementColor = (element: string) => {
    const colors: Record<string, string> = {
      '风': '#3498DB',
      '火': '#E74C3C',
      '水': '#9B59B6',
      '地': '#27AE60',
    };
    return colors[element] || '#666';
  };

  // 渲染输入表单
  const renderInputForm = () => (
    <View style={styles.formContainer}>
      {/* 标题 */}
      <View style={styles.headerSection}>
        <Text style={styles.pageTitle}>星尘玄鉴-塔罗占卜</Text>
        <Text style={styles.pageSubtitle}>大阿尔卡纳 · 命运指引</Text>
      </View>

      {/* 表单卡片 */}
      <View style={styles.formCard}>
        {/* 求测者 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>求测者：</Text>
          <View style={styles.formContent}>
            <TextInput
              style={styles.fullInput}
              value={name}
              onChangeText={setName}
              placeholder="求测者姓名"
              placeholderTextColor="#999"
            />
          </View>
        </View>

        {/* 占问事宜 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>占问事宜：</Text>
          <View style={styles.formContent}>
            <TextInput
              style={styles.fullInput}
              value={question}
              onChangeText={setQuestion}
              placeholder="想要探索的问题"
              placeholderTextColor="#999"
            />
          </View>
        </View>

        {/* 问事类型 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>问事类型：</Text>
          <View style={styles.formContent}>
            <View style={styles.fullPickerWrapper}>
              <Picker
                selectedValue={questionType}
                onValueChange={setQuestionType}
                style={styles.picker}
              >
                {QUESTION_TYPES.map(qt => (
                  <Picker.Item key={qt.value} label={qt.label} value={qt.value} />
                ))}
              </Picker>
            </View>
          </View>
        </View>

        {/* 牌阵选择 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>牌阵选择：</Text>
          <View style={styles.formContent}>
            <View style={styles.spreadButtons}>
              {(Object.entries(SPREAD_CONFIG) as [SpreadType, typeof SPREAD_CONFIG['single']][]).map(([key, config]) => (
                <Pressable
                  key={key}
                  style={[styles.spreadBtn, spreadType === key && styles.spreadBtnActive]}
                  onPress={() => setSpreadType(key)}
                >
                  <Text style={[styles.spreadBtnText, spreadType === key && styles.spreadBtnTextActive]}>
                    {config.name}
                  </Text>
                  <Text style={[styles.spreadBtnCount, spreadType === key && styles.spreadBtnCountActive]}>
                    {config.count}张
                  </Text>
                </Pressable>
              ))}
            </View>
          </View>
        </View>

        {/* 牌阵说明 */}
        <View style={styles.spreadInfo}>
          <Text style={styles.spreadInfoTitle}>{SPREAD_CONFIG[spreadType].name}</Text>
          <Text style={styles.spreadInfoDesc}>{SPREAD_CONFIG[spreadType].desc}</Text>
          <View style={styles.spreadPositions}>
            {SPREAD_CONFIG[spreadType].positions.map((pos, idx) => (
              <View key={idx} style={styles.positionTag}>
                <Text style={styles.positionText}>{pos}</Text>
              </View>
            ))}
          </View>
        </View>

        {/* 本地预览按钮 */}
        <Pressable
          style={styles.secondaryButton}
          onPress={drawCards}
          disabled={loading}
        >
          <Text style={styles.secondaryButtonText}>
            {loading ? '抽牌中...' : '本地预览（不上链）'}
          </Text>
        </Pressable>

        {/* 开始抽牌按钮 */}
        <Pressable
          style={[styles.primaryButton, loading && styles.buttonDisabled]}
          onPress={drawCards}
          disabled={loading}
        >
          {loading ? (
            <ActivityIndicator color={THEME_COLOR_LIGHT} />
          ) : (
            <Text style={styles.primaryButtonText}>开始抽牌（上链存储）</Text>
          )}
        </Pressable>

        <Text style={styles.tipText}>
          静心冥想，让塔罗为你指引方向
        </Text>
      </View>

      {/* 大阿尔卡纳说明 */}
      <View style={styles.infoCard}>
        <Text style={styles.cardTitle}>大阿尔卡纳 · 22张</Text>
        <View style={styles.arcanaGrid}>
          {MAJOR_ARCANA.slice(0, 11).map((card) => (
            <View key={card.id} style={styles.arcanaItem}>
              <Text style={styles.arcanaNumber}>{card.id}</Text>
              <Text style={styles.arcanaName}>{card.name}</Text>
              <Text style={[styles.arcanaElement, { color: getElementColor(card.element) }]}>
                {card.element}
              </Text>
            </View>
          ))}
        </View>
        <View style={styles.arcanaGrid}>
          {MAJOR_ARCANA.slice(11).map((card) => (
            <View key={card.id} style={styles.arcanaItem}>
              <Text style={styles.arcanaNumber}>{card.id}</Text>
              <Text style={styles.arcanaName}>{card.name}</Text>
              <Text style={[styles.arcanaElement, { color: getElementColor(card.element) }]}>
                {card.element}
              </Text>
            </View>
          ))}
        </View>
      </View>
    </View>
  );

  // 渲染结果
  const renderResult = () => {
    if (!result) return null;

    return (
      <View style={styles.resultContainer}>
        {/* 基本信息卡片 */}
        <View style={styles.infoCard}>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>求测者</Text>
            <Text style={styles.infoValue}>{result.name}</Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>占问</Text>
            <Text style={styles.infoValue}>{result.question}</Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>牌阵</Text>
            <Text style={styles.infoValue}>{SPREAD_CONFIG[result.spreadType].name}</Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>时间</Text>
            <Text style={styles.infoValue}>{result.createdAt.toLocaleString('zh-CN')}</Text>
          </View>
        </View>

        {/* 牌面展示 */}
        <View style={styles.cardsCard}>
          <Text style={styles.cardTitle}>牌面</Text>
          <View style={styles.cardsContainer}>
            {result.drawnCards.map((drawn, index) => (
              <View key={index} style={styles.cardWrapper}>
                <Text style={styles.cardPosition}>{drawn.position}</Text>
                <View style={[
                  styles.cardFace,
                  { borderColor: getElementColor(drawn.card.element) }
                ]}>
                  <Text style={styles.cardNumber}>{drawn.card.id}</Text>
                  <Text style={[styles.cardName, drawn.reversed && styles.cardNameReversed]}>
                    {drawn.card.name}
                  </Text>
                  <Text style={[styles.cardElement, { color: getElementColor(drawn.card.element) }]}>
                    {drawn.card.element}
                  </Text>
                  {drawn.reversed && (
                    <View style={styles.reversedBadge}>
                      <Text style={styles.reversedText}>逆位</Text>
                    </View>
                  )}
                </View>
              </View>
            ))}
          </View>
        </View>

        {/* 牌义解读 */}
        <View style={styles.meaningCard}>
          <Text style={styles.cardTitle}>牌义解读</Text>
          {result.drawnCards.map((drawn, index) => (
            <View key={index} style={styles.meaningItem}>
              <View style={styles.meaningHeader}>
                <View style={styles.meaningTitleRow}>
                  <Text style={styles.meaningCardName}>{drawn.card.name}</Text>
                  <View style={[
                    styles.meaningStatusBadge,
                    drawn.reversed && styles.meaningStatusBadgeReversed
                  ]}>
                    <Text style={[
                      styles.meaningStatusText,
                      drawn.reversed && styles.meaningStatusTextReversed
                    ]}>
                      {drawn.reversed ? '逆位' : '正位'}
                    </Text>
                  </View>
                </View>
                <Text style={styles.meaningPosition}>{drawn.position}</Text>
              </View>
              <Text style={styles.meaningText}>
                {drawn.reversed ? drawn.card.reversed : drawn.card.meaning}
              </Text>
              {index < result.drawnCards.length - 1 && <View style={styles.meaningDivider} />}
            </View>
          ))}
        </View>

        {/* 操作按钮 */}
        <View style={styles.actionButtons}>
          <Pressable
            style={styles.aiButton}
            onPress={() => Alert.alert('提示', 'AI解读功能即将上线')}
          >
            <Text style={styles.aiButtonText}>AI深度解读</Text>
          </Pressable>
          <Pressable
            style={styles.detailButton}
            onPress={() => Alert.alert('提示', '详细解读功能即将上线')}
          >
            <Text style={styles.detailButtonText}>详细解读 →</Text>
          </Pressable>
        </View>

        <Pressable style={styles.resetButton} onPress={handleReset}>
          <Text style={styles.resetButtonText}>重新占卜</Text>
        </Pressable>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.navItem} onPress={() => router.push('/divination/tarot-list' as any)}>
          <Ionicons name="albums-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>我的占卜</Text>
        </Pressable>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => Alert.alert('说明', '塔罗牌是西方古老的占卜工具，大阿尔卡纳22张牌代表人生重大主题')}>
          <Ionicons name="help-circle-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>说明</Text>
        </Pressable>
      </View>

      {/* 内容区 */}
      <ScrollView style={styles.content} contentContainerStyle={styles.scrollContent}>
        {result ? renderResult() : renderInputForm()}
      </ScrollView>

      {/* 底部导航 - 全局统一 */}
      <View style={styles.bottomNav}>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/' as any)}>
          <Text style={styles.bottomNavIcon}>🏠</Text>
          <Text style={styles.bottomNavLabel}>首页</Text>
        </Pressable>
        <Pressable style={[styles.bottomNavItem, styles.bottomNavItemActive]} onPress={() => router.push('/divination' as any)}>
          <Text style={styles.bottomNavIcon}>🧭</Text>
          <Text style={[styles.bottomNavLabel, styles.bottomNavLabelActive]}>占卜</Text>
        </Pressable>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/chat' as any)}>
          <Text style={styles.bottomNavIcon}>💬</Text>
          <Text style={styles.bottomNavLabel}>消息</Text>
        </Pressable>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/profile' as any)}>
          <Text style={styles.bottomNavIcon}>👤</Text>
          <Text style={styles.bottomNavLabel}>我的</Text>
        </Pressable>
      </View>
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
  navBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 20,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  navItem: {
    alignItems: 'center',
    gap: 2,
  },
  navItemText: {
    fontSize: 10,
    color: '#999',
  },
  backButton: {
    padding: 4,
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 12,
    paddingBottom: 100,
  },
  formContainer: {},
  headerSection: {
    alignItems: 'center',
    marginBottom: 16,
  },
  pageTitle: {
    fontSize: 20,
    fontWeight: '500',
    color: '#333',
  },
  pageSubtitle: {
    fontSize: 14,
    color: '#CCC',
    marginTop: 4,
  },
  formCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
    marginBottom: 12,
  },
  formRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  formLabel: {
    width: 70,
    fontSize: 14,
    color: '#8B6914',
    textAlign: 'right',
    paddingRight: 8,
  },
  formContent: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  fullInput: {
    flex: 1,
    height: 36,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    paddingHorizontal: 12,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FFF',
  },
  fullPickerWrapper: {
    flex: 1,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
    height: 36,
    justifyContent: 'center',
  },
  picker: {
    height: 36,
  },
  spreadButtons: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  spreadBtn: {
    flex: 1,
    minWidth: 80,
    paddingVertical: 10,
    paddingHorizontal: 8,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
    alignItems: 'center',
  },
  spreadBtnActive: {
    backgroundColor: THEME_COLOR,
    borderColor: THEME_COLOR,
  },
  spreadBtnText: {
    fontSize: 13,
    color: '#666',
    fontWeight: '500',
  },
  spreadBtnTextActive: {
    color: '#FFF',
  },
  spreadBtnCount: {
    fontSize: 11,
    color: '#999',
    marginTop: 2,
  },
  spreadBtnCountActive: {
    color: 'rgba(255,255,255,0.8)',
  },
  spreadInfo: {
    backgroundColor: '#FAFAFA',
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
    padding: 12,
    marginBottom: 16,
    alignItems: 'center',
  },
  spreadInfoTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: THEME_COLOR,
    marginBottom: 4,
  },
  spreadInfoDesc: {
    fontSize: 12,
    color: '#999',
    marginBottom: 8,
  },
  spreadPositions: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    justifyContent: 'center',
  },
  positionTag: {
    paddingHorizontal: 10,
    paddingVertical: 4,
    backgroundColor: '#667EEA20',
    borderRadius: 4,
  },
  positionText: {
    fontSize: 11,
    color: '#667EEA',
    fontWeight: '500',
  },
  secondaryButton: {
    height: 48,
    borderWidth: 1,
    borderColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 8,
  },
  secondaryButtonText: {
    fontSize: 16,
    fontWeight: '500',
    color: THEME_COLOR,
  },
  primaryButton: {
    height: 48,
    backgroundColor: '#000',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 12,
  },
  primaryButtonText: {
    fontSize: 16,
    fontWeight: '500',
    color: THEME_COLOR_LIGHT,
  },
  buttonDisabled: {
    opacity: 0.7,
  },
  tipText: {
    fontSize: 11,
    color: '#999',
    textAlign: 'center',
  },
  infoCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
    marginBottom: 12,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    textAlign: 'center',
    marginBottom: 16,
  },
  arcanaGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 6,
    justifyContent: 'center',
    marginBottom: 8,
  },
  arcanaItem: {
    width: '17%',
    alignItems: 'center',
    paddingVertical: 6,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
  },
  arcanaNumber: {
    fontSize: 11,
    color: '#999',
  },
  arcanaName: {
    fontSize: 11,
    color: '#333',
    fontWeight: '500',
  },
  arcanaElement: {
    fontSize: 9,
    marginTop: 2,
  },
  resultContainer: {
    gap: 12,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
  },
  infoLabel: {
    fontSize: 14,
    color: '#999',
  },
  infoValue: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  infoDivider: {
    height: 1,
    backgroundColor: '#F0F0F0',
  },
  cardsCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  cardsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'center',
    gap: 16,
  },
  cardWrapper: {
    alignItems: 'center',
    minWidth: 80,
  },
  cardPosition: {
    fontSize: 12,
    color: '#999',
    marginBottom: 8,
  },
  cardFace: {
    width: 80,
    height: 120,
    borderWidth: 2,
    borderRadius: 8,
    backgroundColor: '#FAFAFA',
    justifyContent: 'center',
    alignItems: 'center',
    position: 'relative',
  },
  cardNumber: {
    fontSize: 18,
    fontWeight: 'bold',
    color: THEME_COLOR,
    marginBottom: 4,
  },
  cardName: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  cardNameReversed: {
    color: '#E74C3C',
  },
  cardElement: {
    fontSize: 11,
  },
  reversedBadge: {
    position: 'absolute',
    top: 4,
    right: 4,
    backgroundColor: '#E74C3C',
    paddingHorizontal: 4,
    paddingVertical: 2,
    borderRadius: 2,
  },
  reversedText: {
    fontSize: 9,
    color: '#FFF',
    fontWeight: 'bold',
  },
  meaningCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  meaningItem: {},
  meaningHeader: {
    marginBottom: 8,
  },
  meaningTitleRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
    marginBottom: 2,
  },
  meaningCardName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  meaningStatusBadge: {
    backgroundColor: '#27AE6020',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
  },
  meaningStatusBadgeReversed: {
    backgroundColor: '#E74C3C20',
  },
  meaningStatusText: {
    fontSize: 11,
    fontWeight: '600',
    color: '#27AE60',
  },
  meaningStatusTextReversed: {
    color: '#E74C3C',
  },
  meaningPosition: {
    fontSize: 12,
    color: '#999',
  },
  meaningText: {
    fontSize: 14,
    color: '#666',
    lineHeight: 22,
  },
  meaningDivider: {
    height: 1,
    backgroundColor: '#F0F0F0',
    marginVertical: 12,
  },
  actionButtons: {
    gap: 8,
  },
  aiButton: {
    height: 48,
    backgroundColor: THEME_COLOR,
    justifyContent: 'center',
    alignItems: 'center',
    borderRadius: 4,
  },
  aiButtonText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#FFF',
  },
  detailButton: {
    height: 48,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    justifyContent: 'center',
    alignItems: 'center',
    borderRadius: 4,
    backgroundColor: '#FFF',
  },
  detailButtonText: {
    fontSize: 16,
    color: '#333',
  },
  resetButton: {
    height: 44,
    justifyContent: 'center',
    alignItems: 'center',
  },
  resetButtonText: {
    fontSize: 15,
    color: '#666',
  },
  bottomNav: {
    position: 'absolute',
    bottom: 0,
    left: '50%',
    transform: [{ translateX: -207 }],
    width: 414,
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'center',
    backgroundColor: '#FFF',
    paddingTop: 8,
    paddingBottom: 8,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  bottomNavItem: {
    alignItems: 'center',
    paddingVertical: 4,
    flex: 1,
  },
  bottomNavItemActive: {},
  bottomNavIcon: {
    fontSize: 22,
    marginBottom: 2,
  },
  bottomNavLabel: {
    fontSize: 12,
    color: '#999',
    fontWeight: '500',
  },
  bottomNavLabelActive: {
    color: THEME_COLOR,
  },
});
