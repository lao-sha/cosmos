import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  Alert,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View
} from 'react-native';

const MAJOR_ARCANA = [
  { name: '愚者', meaning: '新开始、冒险、无限可能' },
  { name: '魔术师', meaning: '创造力、技能、意志力' },
  { name: '女祭司', meaning: '直觉、神秘、内在智慧' },
  { name: '女皇', meaning: '丰收、母性、创造' },
  { name: '皇帝', meaning: '权威、稳定、领导' },
  { name: '教皇', meaning: '传统、信仰、指导' },
  { name: '恋人', meaning: '爱情、选择、和谐' },
  { name: '战车', meaning: '胜利、意志、决心' },
  { name: '力量', meaning: '勇气、耐心、内在力量' },
  { name: '隐士', meaning: '内省、寻找、智慧' },
  { name: '命运之轮', meaning: '转变、机遇、命运' },
  { name: '正义', meaning: '公平、真相、因果' },
  { name: '倒吊人', meaning: '牺牲、等待、新视角' },
  { name: '死神', meaning: '结束、转变、重生' },
  { name: '节制', meaning: '平衡、耐心、调和' },
  { name: '恶魔', meaning: '束缚、诱惑、物质' },
  { name: '塔', meaning: '突变、觉醒、释放' },
  { name: '星星', meaning: '希望、灵感、宁静' },
  { name: '月亮', meaning: '幻觉、直觉、潜意识' },
  { name: '太阳', meaning: '成功、快乐、活力' },
  { name: '审判', meaning: '觉醒、重生、召唤' },
  { name: '世界', meaning: '完成、整合、成就' },
];

const SPREADS = [
  { id: 'single', name: '单牌占卜', count: 1, description: '快速获得指引' },
  { id: 'three', name: '三牌阵', count: 3, description: '过去-现在-未来' },
  { id: 'celtic', name: '凯尔特十字', count: 10, description: '深度分析' },
];

export default function TarotScreen() {
  const router = useRouter();
  const [question, setQuestion] = useState('');
  const [selectedSpread, setSelectedSpread] = useState<string | null>(null);
  const [drawnCards, setDrawnCards] = useState<typeof MAJOR_ARCANA>([]);
  const [isReversed, setIsReversed] = useState<boolean[]>([]);

  const handleDraw = () => {
    if (!question.trim()) {
      Alert.alert('提示', '请先输入您想问的问题');
      return;
    }
    if (!selectedSpread) {
      Alert.alert('提示', '请选择牌阵');
      return;
    }

    const spread = SPREADS.find(s => s.id === selectedSpread);
    if (!spread) return;

    const shuffled = [...MAJOR_ARCANA].sort(() => Math.random() - 0.5);
    const cards = shuffled.slice(0, spread.count);
    const reversed = cards.map(() => Math.random() > 0.7);

    setDrawnCards(cards);
    setIsReversed(reversed);
  };

  const handleReset = () => {
    setQuestion('');
    setSelectedSpread(null);
    setDrawnCards([]);
    setIsReversed([]);
  };

  const handleFindMaster = () => {
    router.push('/divination/masters/tarot' as any);
  };

  const getPositionName = (index: number) => {
    if (selectedSpread === 'three') {
      return ['过去', '现在', '未来'][index];
    }
    if (selectedSpread === 'celtic') {
      return ['现状', '挑战', '过去', '未来', '目标', '潜意识', '建议', '外部影响', '希望/恐惧', '结果'][index];
    }
    return '指引';
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>🃏 塔罗占卜</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.introCard}>
          <Text style={styles.introTitle}>塔罗占卜</Text>
          <Text style={styles.introDesc}>
            西方神秘学，直觉引导。塔罗牌通过象征性的图像，连接潜意识，为您提供人生指引。
          </Text>
        </View>

        {drawnCards.length === 0 ? (
          <>
            <View style={styles.inputCard}>
              <Text style={styles.cardTitle}>📝 您的问题</Text>
              <TextInput
                style={styles.questionInput}
                placeholder="请输入您想问的问题..."
                placeholderTextColor="#9ca3af"
                value={question}
                onChangeText={setQuestion}
                multiline
                numberOfLines={3}
              />
            </View>

            <View style={styles.inputCard}>
              <Text style={styles.cardTitle}>🎴 选择牌阵</Text>
              <View style={styles.spreadOptions}>
                {SPREADS.map((spread) => (
                  <Pressable
                    key={spread.id}
                    style={[
                      styles.spreadOption,
                      selectedSpread === spread.id && styles.spreadOptionSelected,
                    ]}
                    onPress={() => setSelectedSpread(spread.id)}
                  >
                    <Text style={[
                      styles.spreadName,
                      selectedSpread === spread.id && styles.spreadNameSelected,
                    ]}>
                      {spread.name}
                    </Text>
                    <Text style={[
                      styles.spreadDesc,
                      selectedSpread === spread.id && styles.spreadDescSelected,
                    ]}>
                      {spread.description}
                    </Text>
                  </Pressable>
                ))}
              </View>
            </View>

            <Pressable style={styles.drawButton} onPress={handleDraw}>
              <Text style={styles.drawButtonText}>🔮 抽牌</Text>
            </Pressable>
          </>
        ) : (
          <View style={styles.resultCard}>
            <Text style={styles.resultTitle}>塔罗牌阵</Text>
            
            <View style={styles.cardsDisplay}>
              {drawnCards.map((card, index) => (
                <View key={index} style={styles.cardItem}>
                  <Text style={styles.cardPosition}>{getPositionName(index)}</Text>
                  <View style={[styles.cardBox, isReversed[index] && styles.cardBoxReversed]}>
                    <Text style={styles.cardEmoji}>🃏</Text>
                    <Text style={styles.cardName}>{card.name}</Text>
                    {isReversed[index] && (
                      <Text style={styles.reversedTag}>逆位</Text>
                    )}
                  </View>
                  <Text style={styles.cardMeaning}>{card.meaning}</Text>
                </View>
              ))}
            </View>

            <Text style={styles.resultHint}>
              💡 塔罗牌解读需要结合牌阵位置和问题背景
            </Text>

            <View style={styles.resultActions}>
              <Pressable style={styles.resetButton} onPress={handleReset}>
                <Text style={styles.resetButtonText}>重新抽牌</Text>
              </Pressable>
              <Pressable style={styles.findMasterButton} onPress={handleFindMaster}>
                <Text style={styles.findMasterButtonText}>找大师解牌</Text>
              </Pressable>
            </View>
          </View>
        )}

        <View style={styles.tipsCard}>
          <Text style={styles.tipsTitle}>💡 占卜须知</Text>
          <Text style={styles.tipsText}>
            • 保持内心平静，专注于问题{'\n'}
            • 问题要具体，避免是非题{'\n'}
            • 同一问题短期内不宜重复占卜{'\n'}
            • 塔罗是指引，决定权在您手中
          </Text>
        </View>

        <View style={styles.bottomPadding} />
      </ScrollView>
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
  content: {
    flex: 1,
  },
  introCard: {
    backgroundColor: '#fff',
    margin: 16,
    marginBottom: 8,
    padding: 20,
    borderRadius: 16,
  },
  introTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 8,
  },
  introDesc: {
    fontSize: 14,
    color: '#6b7280',
    lineHeight: 22,
  },
  inputCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginVertical: 8,
    padding: 20,
    borderRadius: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  questionInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    padding: 16,
    fontSize: 15,
    color: '#1f2937',
    minHeight: 80,
    textAlignVertical: 'top',
  },
  spreadOptions: {
    gap: 8,
  },
  spreadOption: {
    backgroundColor: '#f9fafb',
    padding: 16,
    borderRadius: 12,
    marginBottom: 8,
  },
  spreadOptionSelected: {
    backgroundColor: '#6D28D9',
  },
  spreadName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 4,
  },
  spreadNameSelected: {
    color: '#fff',
  },
  spreadDesc: {
    fontSize: 13,
    color: '#6b7280',
  },
  spreadDescSelected: {
    color: 'rgba(255,255,255,0.8)',
  },
  drawButton: {
    backgroundColor: '#6D28D9',
    marginHorizontal: 16,
    marginVertical: 16,
    padding: 18,
    borderRadius: 16,
    alignItems: 'center',
  },
  drawButtonText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: '600',
  },
  resultCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginVertical: 8,
    padding: 20,
    borderRadius: 16,
  },
  resultTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#1f2937',
    textAlign: 'center',
    marginBottom: 20,
  },
  cardsDisplay: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'center',
    marginBottom: 16,
  },
  cardItem: {
    alignItems: 'center',
    margin: 8,
    width: 100,
  },
  cardPosition: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 8,
  },
  cardBox: {
    width: 80,
    height: 120,
    backgroundColor: '#4c1d95',
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 8,
  },
  cardBoxReversed: {
    transform: [{ rotate: '180deg' }],
  },
  cardEmoji: {
    fontSize: 32,
    marginBottom: 4,
  },
  cardName: {
    fontSize: 12,
    color: '#fff',
    fontWeight: '600',
    textAlign: 'center',
  },
  reversedTag: {
    position: 'absolute',
    bottom: -20,
    fontSize: 10,
    color: '#ef4444',
    fontWeight: '600',
    transform: [{ rotate: '180deg' }],
  },
  cardMeaning: {
    fontSize: 11,
    color: '#6b7280',
    textAlign: 'center',
    lineHeight: 16,
  },
  resultHint: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    marginBottom: 16,
  },
  resultActions: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  resetButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#6D28D9',
    alignItems: 'center',
    marginRight: 8,
  },
  resetButtonText: {
    color: '#6D28D9',
    fontSize: 15,
    fontWeight: '600',
  },
  findMasterButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    backgroundColor: '#6D28D9',
    alignItems: 'center',
    marginLeft: 8,
  },
  findMasterButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
  tipsCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginVertical: 8,
    padding: 20,
    borderRadius: 16,
  },
  tipsTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  tipsText: {
    fontSize: 14,
    color: '#6b7280',
    lineHeight: 24,
  },
  bottomPadding: {
    height: 40,
  },
});
