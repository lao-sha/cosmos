/**
 * 星尘玄鉴 - 六爻排盘
 * 参考样式：专业六爻排盘风格
 * 主题色：金棕色 #B2955D
 */

import { useState, useEffect } from 'react';
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

// 爻值：6=老阴(变), 7=少阳, 8=少阴, 9=老阳(变)
type YaoValue = 6 | 7 | 8 | 9;

// 八卦
const TRIGRAMS = [
  { name: '乾', symbol: '☰', nature: '天' },
  { name: '兑', symbol: '☱', nature: '泽' },
  { name: '离', symbol: '☲', nature: '火' },
  { name: '震', symbol: '☳', nature: '雷' },
  { name: '巽', symbol: '☴', nature: '风' },
  { name: '坎', symbol: '☵', nature: '水' },
  { name: '艮', symbol: '☶', nature: '山' },
  { name: '坤', symbol: '☷', nature: '地' },
];

// 六亲
const LIU_QIN = ['父母', '兄弟', '子孙', '妻财', '官鬼'];

// 六神
const LIU_SHEN = ['青龙', '朱雀', '勾陈', '螣蛇', '白虎', '玄武'];

// 问事类型
const QUESTION_TYPES = [
  { value: 0, label: '综合运势' },
  { value: 1, label: '事业工作' },
  { value: 2, label: '财运求财' },
  { value: 3, label: '婚姻感情' },
  { value: 4, label: '健康疾病' },
  { value: 5, label: '官司诉讼' },
  { value: 6, label: '出行远行' },
  { value: 7, label: '寻人寻物' },
];

// 起卦方式
type DivinationMethod = 'shake' | 'time' | 'number' | 'manual';

// 爻位信息
interface YaoInfo {
  position: string;
  value: YaoValue;
  symbol: string;
  nature: string;
  liuQin: string;
  liuShen: string;
  isChanging: boolean;
}

// 六爻结果
interface LiuyaoResult {
  id: number;
  name: string;
  question: string;
  questionType: number;
  method: DivinationMethod;
  yaos: YaoInfo[];
  benGua: string;
  bianGua: string;
  shiYao: number;
  yingYao: number;
  createdAt: Date;
}

export default function LiuyaoPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<LiuyaoResult | null>(null);
  const [history, setHistory] = useState<LiuyaoResult[]>([]);
  const [currentYaos, setCurrentYaos] = useState<YaoValue[]>([]);

  // 表单状态
  const [name, setName] = useState('');
  const [question, setQuestion] = useState('');
  const [questionType, setQuestionType] = useState(0);
  const [divinationMethod, setDivinationMethod] = useState<DivinationMethod>('shake');
  const [num1, setNum1] = useState('');
  const [num2, setNum2] = useState('');
  const [num3, setNum3] = useState('');

  const isComplete = currentYaos.length >= 6;

  // 生成一爻（模拟三枚铜钱）
  const generateYao = (): YaoValue => {
    const coins = [0, 0, 0].map(() => Math.random() > 0.5 ? 3 : 2);
    const sum = coins.reduce((a, b) => a + b, 0) as YaoValue;
    return sum;
  };

  // 摇卦
  const handleShake = () => {
    if (isComplete) return;
    const newYao = generateYao();
    setCurrentYaos(prev => [...prev, newYao]);
  };

  // 获取卦名
  const getGuaName = (upper: number, lower: number): string => {
    const names64: Record<string, string> = {
      '00': '乾为天', '01': '天泽履', '02': '天火同人', '03': '天雷无妄',
      '04': '天风姤', '05': '天水讼', '06': '天山遁', '07': '天地否',
      '10': '泽天夬', '11': '兑为泽', '12': '泽火革', '13': '泽雷随',
      '14': '泽风大过', '15': '泽水困', '16': '泽山咸', '17': '泽地萃',
      '20': '火天大有', '21': '火泽睽', '22': '离为火', '23': '火雷噬嗑',
      '24': '火风鼎', '25': '火水未济', '26': '火山旅', '27': '火地晋',
      '30': '雷天大壮', '31': '雷泽归妹', '32': '雷火丰', '33': '震为雷',
      '34': '雷风恒', '35': '雷水解', '36': '雷山小过', '37': '雷地豫',
      '40': '风天小畜', '41': '风泽中孚', '42': '风火家人', '43': '风雷益',
      '44': '巽为风', '45': '风水涣', '46': '风山渐', '47': '风地观',
      '50': '水天需', '51': '水泽节', '52': '水火既济', '53': '水雷屯',
      '54': '水风井', '55': '坎为水', '56': '水山蹇', '57': '水地比',
      '60': '山天大畜', '61': '山泽损', '62': '山火贲', '63': '山雷颐',
      '64': '山风蛊', '65': '山水蒙', '66': '艮为山', '67': '山地剥',
      '70': '地天泰', '71': '地泽临', '72': '地火明夷', '73': '地雷复',
      '74': '地风升', '75': '地水师', '76': '地山谦', '77': '坤为地',
    };
    return names64[`${upper}${lower}`] || `${TRIGRAMS[upper].name}${TRIGRAMS[lower].name}`;
  };

  // 计算六爻
  const calculateLiuyao = async () => {
    if (divinationMethod === 'shake' && !isComplete) {
      Alert.alert('提示', '请先完成六次摇卦');
      return;
    }
    if (divinationMethod === 'number') {
      const n1 = parseInt(num1);
      const n2 = parseInt(num2);
      const n3 = parseInt(num3);
      if (isNaN(n1) || isNaN(n2) || isNaN(n3) || n1 <= 0 || n2 <= 0 || n3 <= 0) {
        Alert.alert('提示', '请输入有效的正整数');
        return;
      }
    }

    setLoading(true);
    await new Promise(resolve => setTimeout(resolve, 800));

    let yaos: YaoValue[];
    if (divinationMethod === 'shake') {
      yaos = currentYaos;
    } else if (divinationMethod === 'time') {
      const now = new Date();
      const seed = now.getFullYear() + now.getMonth() + now.getDate() + now.getHours();
      yaos = Array(6).fill(0).map((_, i) => {
        const v = ((seed + i * 7) % 4) + 6;
        return v as YaoValue;
      });
    } else if (divinationMethod === 'number') {
      const n1 = parseInt(num1);
      const n2 = parseInt(num2);
      const n3 = parseInt(num3);
      yaos = Array(6).fill(0).map((_, i) => {
        const v = ((n1 + n2 + n3 + i) % 4) + 6;
        return v as YaoValue;
      });
    } else {
      yaos = Array(6).fill(0).map(() => generateYao());
    }

    // 构建爻位信息
    const yaoInfos: YaoInfo[] = yaos.map((yao, i) => {
      const positions = ['初爻', '二爻', '三爻', '四爻', '五爻', '上爻'];
      const isYang = yao === 7 || yao === 9;
      const isChanging = yao === 6 || yao === 9;
      return {
        position: positions[i],
        value: yao,
        symbol: isYang ? '⚊' : '⚋',
        nature: isChanging ? (yao === 9 ? '老阳' : '老阴') : (isYang ? '少阳' : '少阴'),
        liuQin: LIU_QIN[i % 5],
        liuShen: LIU_SHEN[i],
        isChanging,
      };
    });

    // 简化卦象计算
    const lowerNum = (yaos[0] + yaos[1] + yaos[2]) % 8;
    const upperNum = (yaos[3] + yaos[4] + yaos[5]) % 8;
    const benGua = getGuaName(upperNum, lowerNum);

    // 变卦计算
    const changedYaos = yaos.map(y => y === 6 ? 7 : y === 9 ? 8 : y);
    const changedLowerNum = (changedYaos[0] + changedYaos[1] + changedYaos[2]) % 8;
    const changedUpperNum = (changedYaos[3] + changedYaos[4] + changedYaos[5]) % 8;
    const bianGua = getGuaName(changedUpperNum, changedLowerNum);

    const liuyaoResult: LiuyaoResult = {
      id: Date.now(),
      name: name || '求测者',
      question: question || '某事',
      questionType,
      method: divinationMethod,
      yaos: yaoInfos,
      benGua,
      bianGua,
      shiYao: (upperNum + lowerNum) % 6,
      yingYao: ((upperNum + lowerNum) + 3) % 6,
      createdAt: new Date(),
    };

    setResult(liuyaoResult);
    setHistory(prev => [liuyaoResult, ...prev]);
    setLoading(false);
  };

  const handleReset = () => {
    setResult(null);
    setCurrentYaos([]);
  };

  // 获取爻颜色
  const getYaoColor = (yao: YaoInfo) => {
    if (yao.isChanging) return '#E74C3C';
    return '#333';
  };

  // 渲染输入表单
  const renderInputForm = () => (
    <View style={styles.formContainer}>
      {/* 标题 */}
      <View style={styles.headerSection}>
        <Text style={styles.pageTitle}>星尘玄鉴-六爻排盘</Text>
        <Text style={styles.pageSubtitle}>纳甲筮法 · 周易正宗</Text>
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
              placeholder="要问的事情"
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

        {/* 起卦方式 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>起卦方式：</Text>
          <View style={styles.formContent}>
            <View style={styles.methodButtons}>
              {(['shake', 'time', 'number', 'manual'] as DivinationMethod[]).map(m => (
                <Pressable
                  key={m}
                  style={[styles.methodBtn, divinationMethod === m && styles.methodBtnActive]}
                  onPress={() => setDivinationMethod(m)}
                >
                  <Text style={[styles.methodBtnText, divinationMethod === m && styles.methodBtnTextActive]}>
                    {m === 'shake' ? '摇卦' : m === 'time' ? '时间' : m === 'number' ? '数字' : '随机'}
                  </Text>
                </Pressable>
              ))}
            </View>
          </View>
        </View>

        {/* 摇卦区域 */}
        {divinationMethod === 'shake' && (
          <View style={styles.shakeSection}>
            <View style={styles.shakeCounter}>
              <Text style={styles.shakeCounterCurrent}>{currentYaos.length}</Text>
              <Text style={styles.shakeCounterSeparator}>/</Text>
              <Text style={styles.shakeCounterTotal}>6</Text>
              <Text style={styles.shakeCounterUnit}> 爻</Text>
            </View>

            {/* 已摇爻象 */}
            <View style={styles.yaosPreview}>
              {[...currentYaos].reverse().map((yao, idx) => (
                <View key={`yao-${5 - idx}`} style={styles.yaoPreviewItem}>
                  <Text style={styles.yaoPreviewLabel}>
                    {['初爻', '二爻', '三爻', '四爻', '五爻', '上爻'][5 - idx]}
                  </Text>
                  <Text style={[
                    styles.yaoPreviewSymbol,
                    (yao === 6 || yao === 9) && styles.yaoPreviewChanging
                  ]}>
                    {yao === 7 || yao === 9 ? '⚊' : '⚋'}
                    {yao === 6 ? ' 老阴' : yao === 9 ? ' 老阳' : yao === 7 ? ' 少阳' : ' 少阴'}
                  </Text>
                </View>
              ))}
            </View>

            {!isComplete && (
              <Pressable style={styles.shakeButton} onPress={handleShake}>
                <Text style={styles.shakeButtonText}>点击摇卦 ({currentYaos.length}/6)</Text>
              </Pressable>
            )}
          </View>
        )}

        {/* 数字起卦 */}
        {divinationMethod === 'number' && (
          <View style={styles.formRow}>
            <Text style={styles.formLabel}>起卦数字：</Text>
            <View style={styles.formContent}>
              <TextInput
                style={styles.numberInput}
                value={num1}
                onChangeText={setNum1}
                keyboardType="number-pad"
                maxLength={3}
                placeholder="数1"
                placeholderTextColor="#999"
              />
              <TextInput
                style={styles.numberInput}
                value={num2}
                onChangeText={setNum2}
                keyboardType="number-pad"
                maxLength={3}
                placeholder="数2"
                placeholderTextColor="#999"
              />
              <TextInput
                style={styles.numberInput}
                value={num3}
                onChangeText={setNum3}
                keyboardType="number-pad"
                maxLength={3}
                placeholder="数3"
                placeholderTextColor="#999"
              />
            </View>
          </View>
        )}

        {/* 时间/随机起卦说明 */}
        {(divinationMethod === 'time' || divinationMethod === 'manual') && (
          <View style={styles.formRow}>
            <Text style={styles.formLabel}></Text>
            <View style={styles.formContent}>
              <Text style={styles.tipTextSmall}>
                {divinationMethod === 'time' ? '以当前时间自动起卦' : '使用随机数自动起卦'}
              </Text>
            </View>
          </View>
        )}

        {/* 本地预览按钮 */}
        <Pressable
          style={styles.secondaryButton}
          onPress={calculateLiuyao}
          disabled={loading}
        >
          <Text style={styles.secondaryButtonText}>
            {loading ? '计算中...' : '本地预览（不上链）'}
          </Text>
        </Pressable>

        {/* 开始排盘按钮 */}
        <Pressable
          style={[styles.primaryButton, loading && styles.buttonDisabled]}
          onPress={calculateLiuyao}
          disabled={loading}
        >
          {loading ? (
            <ActivityIndicator color={THEME_COLOR_LIGHT} />
          ) : (
            <Text style={styles.primaryButtonText}>开始排盘（上链存储）</Text>
          )}
        </Pressable>

        <Text style={styles.tipText}>
          {divinationMethod === 'shake' && '模拟三枚铜钱起卦'}
          {divinationMethod === 'time' && '以时间年月日时自动起卦'}
          {divinationMethod === 'number' && '输入三个数字起卦'}
          {divinationMethod === 'manual' && '使用随机数起卦'}
        </Text>
      </View>

      {/* 六亲说明 */}
      <View style={styles.infoCard}>
        <Text style={styles.cardTitle}>六亲</Text>
        <View style={styles.liuqinGrid}>
          {LIU_QIN.map((lq, idx) => (
            <View key={idx} style={styles.liuqinItem}>
              <Text style={styles.liuqinName}>{lq}</Text>
            </View>
          ))}
        </View>
      </View>

      {/* 六神说明 */}
      <View style={styles.infoCard}>
        <Text style={styles.cardTitle}>六神</Text>
        <View style={styles.liushenGrid}>
          {LIU_SHEN.map((ls, idx) => (
            <View key={idx} style={styles.liushenItem}>
              <Text style={styles.liushenName}>{ls}</Text>
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
            <Text style={styles.infoLabel}>起卦方式</Text>
            <Text style={styles.infoValue}>
              {result.method === 'shake' ? '摇卦' : result.method === 'time' ? '时间起卦' : result.method === 'number' ? '数字起卦' : '随机起卦'}
            </Text>
          </View>
        </View>

        {/* 卦名卡片 */}
        <View style={styles.hexagramCard}>
          <View style={styles.hexagramRow}>
            <View style={styles.hexagramItem}>
              <Text style={styles.hexagramLabel}>本卦</Text>
              <Text style={styles.hexagramName}>{result.benGua}</Text>
            </View>
            <View style={styles.hexagramArrow}>
              <Text style={styles.hexagramArrowText}>→</Text>
            </View>
            <View style={styles.hexagramItem}>
              <Text style={styles.hexagramLabel}>变卦</Text>
              <Text style={styles.hexagramName}>{result.bianGua}</Text>
            </View>
          </View>
          <View style={styles.hexagramTags}>
            <View style={[styles.hexagramTag, { backgroundColor: '#667EEA20' }]}>
              <Text style={[styles.hexagramTagText, { color: '#667EEA' }]}>
                世爻：{['初爻', '二爻', '三爻', '四爻', '五爻', '上爻'][result.shiYao]}
              </Text>
            </View>
            <View style={[styles.hexagramTag, { backgroundColor: '#F39C1220' }]}>
              <Text style={[styles.hexagramTagText, { color: '#F39C12' }]}>
                应爻：{['初爻', '二爻', '三爻', '四爻', '五爻', '上爻'][result.yingYao]}
              </Text>
            </View>
          </View>
        </View>

        {/* 六爻详情 */}
        <View style={styles.yaosCard}>
          <Text style={styles.cardTitle}>六爻详情</Text>
          <View style={styles.yaosContainer}>
            {[...result.yaos].reverse().map((yao, idx) => (
              <View
                key={idx}
                style={[
                  styles.yaoRow,
                  (5 - idx) === result.shiYao && styles.yaoRowShi,
                  (5 - idx) === result.yingYao && styles.yaoRowYing,
                ]}
              >
                <View style={styles.yaoPosition}>
                  <Text style={styles.yaoPositionText}>{yao.position}</Text>
                  {(5 - idx) === result.shiYao && (
                    <View style={styles.shiYingBadge}>
                      <Text style={styles.shiYingText}>世</Text>
                    </View>
                  )}
                  {(5 - idx) === result.yingYao && (
                    <View style={[styles.shiYingBadge, { backgroundColor: '#F39C12' }]}>
                      <Text style={styles.shiYingText}>应</Text>
                    </View>
                  )}
                </View>
                <Text style={[styles.yaoSymbol, { color: getYaoColor(yao) }]}>
                  {yao.symbol}
                </Text>
                <Text style={[styles.yaoNature, { color: getYaoColor(yao) }]}>
                  {yao.nature}
                </Text>
                <Text style={styles.yaoLiuQin}>{yao.liuQin}</Text>
                <Text style={styles.yaoLiuShen}>{yao.liuShen}</Text>
                {yao.isChanging && (
                  <View style={styles.changingBadge}>
                    <Text style={styles.changingText}>动</Text>
                  </View>
                )}
              </View>
            ))}
          </View>
        </View>

        {/* 操作按钮 */}
        <View style={styles.actionButtons}>
          <Pressable
            style={styles.aiButton}
            onPress={() => Alert.alert('提示', 'AI解读功能即将上线')}
          >
            <Text style={styles.aiButtonText}>AI智能解卦</Text>
          </Pressable>
          <Pressable
            style={styles.detailButton}
            onPress={() => Alert.alert('提示', '详细解卦功能即将上线')}
          >
            <Text style={styles.detailButtonText}>详细解卦 →</Text>
          </Pressable>
        </View>

        <Pressable style={styles.resetButton} onPress={handleReset}>
          <Text style={styles.resetButtonText}>重新起卦</Text>
        </Pressable>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.navItem} onPress={() => router.push('/divination/liuyao-list' as any)}>
          <Ionicons name="albums-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>我的卦象</Text>
        </Pressable>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => Alert.alert('说明', '六爻是中国古代最重要的占卜术之一，以纳甲筮法为核心')}>
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
  numberInput: {
    flex: 1,
    height: 36,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    paddingHorizontal: 8,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FFF',
    textAlign: 'center',
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
  methodButtons: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 0,
  },
  methodBtn: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
  },
  methodBtnActive: {
    backgroundColor: THEME_COLOR,
    borderColor: THEME_COLOR,
  },
  methodBtnText: {
    fontSize: 14,
    color: '#666',
  },
  methodBtnTextActive: {
    color: '#FFF',
  },
  tipTextSmall: {
    fontSize: 12,
    color: '#999',
  },
  shakeSection: {
    alignItems: 'center',
    paddingVertical: 16,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
    marginBottom: 16,
    backgroundColor: '#FAFAFA',
  },
  shakeCounter: {
    flexDirection: 'row',
    alignItems: 'baseline',
    marginBottom: 16,
  },
  shakeCounterCurrent: {
    fontSize: 36,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  shakeCounterSeparator: {
    fontSize: 24,
    color: '#999',
  },
  shakeCounterTotal: {
    fontSize: 24,
    color: '#999',
  },
  shakeCounterUnit: {
    fontSize: 14,
    color: '#999',
  },
  yaosPreview: {
    width: '100%',
    paddingHorizontal: 16,
    gap: 6,
    marginBottom: 16,
  },
  yaoPreviewItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FFF',
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderRadius: 4,
  },
  yaoPreviewLabel: {
    width: 50,
    fontSize: 13,
    color: '#666',
  },
  yaoPreviewSymbol: {
    flex: 1,
    fontSize: 16,
    color: '#333',
  },
  yaoPreviewChanging: {
    color: '#E74C3C',
  },
  shakeButton: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 24,
    paddingVertical: 12,
  },
  shakeButtonText: {
    fontSize: 15,
    fontWeight: '500',
    color: '#FFF',
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
  liuqinGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    justifyContent: 'center',
  },
  liuqinItem: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
  },
  liuqinName: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  liushenGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    justifyContent: 'center',
  },
  liushenItem: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
  },
  liushenName: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
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
  hexagramCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  hexagramRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 16,
  },
  hexagramItem: {
    alignItems: 'center',
  },
  hexagramLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 4,
  },
  hexagramName: {
    fontSize: 20,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  hexagramArrow: {
    paddingHorizontal: 20,
  },
  hexagramArrowText: {
    fontSize: 24,
    color: '#CCC',
  },
  hexagramTags: {
    flexDirection: 'row',
    justifyContent: 'center',
    gap: 8,
  },
  hexagramTag: {
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 4,
  },
  hexagramTagText: {
    fontSize: 12,
    fontWeight: '500',
  },
  yaosCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  yaosContainer: {
    gap: 8,
  },
  yaoRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 10,
    paddingHorizontal: 12,
    backgroundColor: '#FAFAFA',
    borderRadius: 4,
    borderWidth: 1,
    borderColor: '#E8E8E8',
  },
  yaoRowShi: {
    borderColor: '#667EEA',
    borderWidth: 2,
  },
  yaoRowYing: {
    borderColor: '#F39C12',
    borderWidth: 2,
  },
  yaoPosition: {
    width: 60,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  yaoPositionText: {
    fontSize: 13,
    color: '#666',
  },
  shiYingBadge: {
    backgroundColor: '#667EEA',
    borderRadius: 2,
    paddingHorizontal: 4,
    paddingVertical: 1,
  },
  shiYingText: {
    fontSize: 9,
    color: '#FFF',
    fontWeight: 'bold',
  },
  yaoSymbol: {
    width: 30,
    fontSize: 20,
    textAlign: 'center',
  },
  yaoNature: {
    width: 50,
    fontSize: 12,
    textAlign: 'center',
  },
  yaoLiuQin: {
    width: 50,
    fontSize: 12,
    color: '#667EEA',
    textAlign: 'center',
  },
  yaoLiuShen: {
    flex: 1,
    fontSize: 12,
    color: '#9B59B6',
    textAlign: 'center',
  },
  changingBadge: {
    backgroundColor: '#E74C3C',
    borderRadius: 2,
    paddingHorizontal: 6,
    paddingVertical: 2,
  },
  changingText: {
    fontSize: 10,
    color: '#FFF',
    fontWeight: 'bold',
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
