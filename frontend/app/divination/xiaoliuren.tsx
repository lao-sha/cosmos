/**
 * 星尘玄鉴 - 小六壬排盘
 * 参考样式：专业小六壬排盘风格
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

// 六宫
const GONGS = [
  {
    name: '大安',
    element: '木',
    meaning: '稳定平安、贵人相助',
    direction: '东方',
    verse: '大安事事昌，求谋在东方。失物去不远，宅舍保安康。',
    nature: '吉',
    color: '#27AE60',
  },
  {
    name: '留连',
    element: '土',
    meaning: '拖延阻滞、需要等待',
    direction: '中央',
    verse: '留连事难成，求谋日未明。官事只宜缓，去者未回程。',
    nature: '凶',
    color: '#F39C12',
  },
  {
    name: '速喜',
    element: '火',
    meaning: '快速喜事、好消息来',
    direction: '南方',
    verse: '速喜喜来临，求财向南行。失物申未午，逢人路上寻。',
    nature: '吉',
    color: '#E74C3C',
  },
  {
    name: '赤口',
    element: '金',
    meaning: '口舌是非、小心争执',
    direction: '西方',
    verse: '赤口主口舌，官非切要防。失物速速讨，行人有惊慌。',
    nature: '凶',
    color: '#95A5A6',
  },
  {
    name: '小吉',
    element: '水',
    meaning: '小吉顺利、适合行动',
    direction: '北方',
    verse: '小吉最吉昌，路上好商量。阴人来报喜，失物在坤方。',
    nature: '吉',
    color: '#3498DB',
  },
  {
    name: '空亡',
    element: '土',
    meaning: '空虚不成、需要谨慎',
    direction: '中央',
    verse: '空亡事不成，阴人多乖张。求财无利益，行人有灾殃。',
    nature: '凶',
    color: '#7F8C8D',
  },
];

// 问事类型
const QUESTION_TYPES = [
  { value: 0, label: '综合运势' },
  { value: 1, label: '事业工作' },
  { value: 2, label: '财运求财' },
  { value: 3, label: '婚姻感情' },
  { value: 4, label: '健康疾病' },
  { value: 5, label: '出行远行' },
  { value: 6, label: '寻人寻物' },
  { value: 7, label: '日常决策' },
];

// 起卦方式
type DivinationMethod = 'time' | 'number' | 'random';

// 时辰选项
const SHICHEN_OPTIONS = [
  { value: 0, label: '0-子' },
  { value: 1, label: '1-丑' },
  { value: 2, label: '2-丑' },
  { value: 3, label: '3-寅' },
  { value: 4, label: '4-寅' },
  { value: 5, label: '5-卯' },
  { value: 6, label: '6-卯' },
  { value: 7, label: '7-辰' },
  { value: 8, label: '8-辰' },
  { value: 9, label: '9-巳' },
  { value: 10, label: '10-巳' },
  { value: 11, label: '11-午' },
  { value: 12, label: '12-午' },
  { value: 13, label: '13-未' },
  { value: 14, label: '14-未' },
  { value: 15, label: '15-申' },
  { value: 16, label: '16-申' },
  { value: 17, label: '17-酉' },
  { value: 18, label: '18-酉' },
  { value: 19, label: '19-戌' },
  { value: 20, label: '20-戌' },
  { value: 21, label: '21-亥' },
  { value: 22, label: '22-亥' },
  { value: 23, label: '23-子' },
];

// 小六壬结果
interface XiaoliurenResult {
  id: number;
  name: string;
  question: string;
  questionType: number;
  method: DivinationMethod;
  month: number;
  day: number;
  hour: number;
  monthGong: typeof GONGS[0];
  dayGong: typeof GONGS[0];
  hourGong: typeof GONGS[0];
  createdAt: Date;
}

export default function XiaoliurenPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<XiaoliurenResult | null>(null);
  const [history, setHistory] = useState<XiaoliurenResult[]>([]);

  // 表单状态
  const [name, setName] = useState('');
  const [question, setQuestion] = useState('');
  const [questionType, setQuestionType] = useState(0);
  const [divinationMethod, setDivinationMethod] = useState<DivinationMethod>('time');
  const [month, setMonth] = useState(new Date().getMonth() + 1);
  const [day, setDay] = useState(new Date().getDate());
  const [hour, setHour] = useState(new Date().getHours());
  const [num1, setNum1] = useState('');
  const [num2, setNum2] = useState('');
  const [num3, setNum3] = useState('');

  // 掐算
  const calculate = async () => {
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

    let m: number, d: number, h: number;

    switch (divinationMethod) {
      case 'time':
        m = month;
        d = day;
        h = Math.floor((hour + 1) / 2) % 12 + 1;
        break;
      case 'number':
        m = parseInt(num1);
        d = parseInt(num2);
        h = parseInt(num3);
        break;
      case 'random':
        m = Math.floor(Math.random() * 12) + 1;
        d = Math.floor(Math.random() * 30) + 1;
        h = Math.floor(Math.random() * 12) + 1;
        break;
      default:
        m = month;
        d = day;
        h = Math.floor((hour + 1) / 2) % 12 + 1;
    }

    // 小六壬掐算
    const monthIndex = (m - 1) % 6;
    const dayIndex = (monthIndex + d - 1) % 6;
    const hourIndex = (dayIndex + h - 1) % 6;

    const xiaoliurenResult: XiaoliurenResult = {
      id: Date.now(),
      name: name || '求测者',
      question: question || '某事',
      questionType,
      method: divinationMethod,
      month: m,
      day: d,
      hour: h,
      monthGong: GONGS[monthIndex],
      dayGong: GONGS[dayIndex],
      hourGong: GONGS[hourIndex],
      createdAt: new Date(),
    };

    setResult(xiaoliurenResult);
    setHistory(prev => [xiaoliurenResult, ...prev]);
    setLoading(false);
  };

  const handleReset = () => {
    setResult(null);
  };

  // 获取吉凶颜色
  const getNatureColor = (nature: string) => {
    return nature === '吉' ? '#27AE60' : '#E74C3C';
  };

  // 渲染输入表单
  const renderInputForm = () => (
    <View style={styles.formContainer}>
      {/* 标题 */}
      <View style={styles.headerSection}>
        <Text style={styles.pageTitle}>星尘玄鉴-小六壬</Text>
        <Text style={styles.pageSubtitle}>诸葛马前课 · 简便速断</Text>
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
              {(['time', 'number', 'random'] as DivinationMethod[]).map(m => (
                <Pressable
                  key={m}
                  style={[styles.methodBtn, divinationMethod === m && styles.methodBtnActive]}
                  onPress={() => setDivinationMethod(m)}
                >
                  <Text style={[styles.methodBtnText, divinationMethod === m && styles.methodBtnTextActive]}>
                    {m === 'time' ? '时间' : m === 'number' ? '数字' : '随机'}
                  </Text>
                </Pressable>
              ))}
            </View>
          </View>
        </View>

        {/* 时间起卦 */}
        {divinationMethod === 'time' && (
          <>
            <View style={styles.formRow}>
              <Text style={styles.formLabel}>月日数：</Text>
              <View style={styles.formContent}>
                <View style={styles.pickerWrapper}>
                  <Picker selectedValue={month} onValueChange={setMonth} style={styles.picker}>
                    {Array.from({ length: 12 }, (_, i) => i + 1).map(m => (
                      <Picker.Item key={m} label={`${m}月`} value={m} />
                    ))}
                  </Picker>
                </View>
                <View style={styles.pickerWrapper}>
                  <Picker selectedValue={day} onValueChange={setDay} style={styles.picker}>
                    {Array.from({ length: 31 }, (_, i) => i + 1).map(d => (
                      <Picker.Item key={d} label={`${d}日`} value={d} />
                    ))}
                  </Picker>
                </View>
              </View>
            </View>
            <View style={styles.formRow}>
              <Text style={styles.formLabel}>时辰数：</Text>
              <View style={styles.formContent}>
                <View style={styles.hourPickerWrapper}>
                  <Picker selectedValue={hour} onValueChange={setHour} style={styles.picker}>
                    {SHICHEN_OPTIONS.map(opt => (
                      <Picker.Item key={opt.value} label={opt.label} value={opt.value} />
                    ))}
                  </Picker>
                </View>
                <Text style={styles.unitText}>时</Text>
              </View>
            </View>
          </>
        )}

        {/* 数字起卦 */}
        {divinationMethod === 'number' && (
          <View style={styles.formRow}>
            <Text style={styles.formLabel}>三数：</Text>
            <View style={styles.formContent}>
              <Text style={styles.unitText}>月</Text>
              <TextInput
                style={styles.numberInput}
                value={num1}
                onChangeText={setNum1}
                keyboardType="number-pad"
                maxLength={3}
                placeholder="1"
                placeholderTextColor="#999"
              />
              <Text style={styles.unitText}>日</Text>
              <TextInput
                style={styles.numberInput}
                value={num2}
                onChangeText={setNum2}
                keyboardType="number-pad"
                maxLength={3}
                placeholder="1"
                placeholderTextColor="#999"
              />
              <Text style={styles.unitText}>时</Text>
              <TextInput
                style={styles.numberInput}
                value={num3}
                onChangeText={setNum3}
                keyboardType="number-pad"
                maxLength={3}
                placeholder="1"
                placeholderTextColor="#999"
              />
            </View>
          </View>
        )}

        {/* 随机起卦说明 */}
        {divinationMethod === 'random' && (
          <View style={styles.formRow}>
            <Text style={styles.formLabel}></Text>
            <View style={styles.formContent}>
              <Text style={styles.tipTextSmall}>心诚则灵，使用随机数自动起卦</Text>
            </View>
          </View>
        )}

        {/* 掐算方法说明 */}
        <View style={styles.methodInfo}>
          <Text style={styles.methodInfoTitle}>掐算三步</Text>
          <View style={styles.stepsRow}>
            <View style={styles.stepItem}>
              <Text style={styles.stepNum}>1</Text>
              <Text style={styles.stepText}>起月</Text>
            </View>
            <Text style={styles.stepArrow}>→</Text>
            <View style={styles.stepItem}>
              <Text style={styles.stepNum}>2</Text>
              <Text style={styles.stepText}>起日</Text>
            </View>
            <Text style={styles.stepArrow}>→</Text>
            <View style={styles.stepItem}>
              <Text style={styles.stepNum}>3</Text>
              <Text style={styles.stepText}>起时</Text>
            </View>
          </View>
        </View>

        {/* 本地预览按钮 */}
        <Pressable
          style={styles.secondaryButton}
          onPress={calculate}
          disabled={loading}
        >
          <Text style={styles.secondaryButtonText}>
            {loading ? '计算中...' : '本地预览（不上链）'}
          </Text>
        </Pressable>

        {/* 开始掐算按钮 */}
        <Pressable
          style={[styles.primaryButton, loading && styles.buttonDisabled]}
          onPress={calculate}
          disabled={loading}
        >
          {loading ? (
            <ActivityIndicator color={THEME_COLOR_LIGHT} />
          ) : (
            <Text style={styles.primaryButtonText}>开始掐算（上链存储）</Text>
          )}
        </Pressable>

        <Text style={styles.tipText}>
          {divinationMethod === 'time' && '以月日时自动起卦'}
          {divinationMethod === 'number' && '输入三个数字起卦'}
          {divinationMethod === 'random' && '使用随机数起卦'}
        </Text>
      </View>

      {/* 六宫说明 */}
      <View style={styles.infoCard}>
        <Text style={styles.cardTitle}>六宫</Text>
        <View style={styles.gongsGrid}>
          {GONGS.map((gong, idx) => (
            <View key={idx} style={[styles.gongItem, { borderColor: gong.color }]}>
              <Text style={[styles.gongName, { color: gong.color }]}>{gong.name}</Text>
              <Text style={styles.gongElement}>{gong.element}</Text>
              <View style={[styles.gongNatureBadge, { backgroundColor: getNatureColor(gong.nature) + '20' }]}>
                <Text style={[styles.gongNatureText, { color: getNatureColor(gong.nature) }]}>
                  {gong.nature}
                </Text>
              </View>
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
              {result.method === 'time' ? '时间起卦' : result.method === 'number' ? '数字起卦' : '随机起卦'}
            </Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>时间</Text>
            <Text style={styles.infoValue}>{result.createdAt.toLocaleString('zh-CN')}</Text>
          </View>
        </View>

        {/* 月日时数 */}
        <View style={styles.numbersCard}>
          <View style={styles.numbersRow}>
            <View style={styles.numberItem}>
              <Text style={styles.numberLabel}>月数</Text>
              <Text style={styles.numberValue}>{result.month}</Text>
            </View>
            <View style={styles.numberDivider} />
            <View style={styles.numberItem}>
              <Text style={styles.numberLabel}>日数</Text>
              <Text style={styles.numberValue}>{result.day}</Text>
            </View>
            <View style={styles.numberDivider} />
            <View style={styles.numberItem}>
              <Text style={styles.numberLabel}>时数</Text>
              <Text style={styles.numberValue}>{result.hour}</Text>
            </View>
          </View>
        </View>

        {/* 三步落宫 */}
        <View style={styles.stepsCard}>
          <Text style={styles.cardTitle}>三步落宫</Text>
          <View style={styles.stepsContainer}>
            {[
              { label: '月数', value: result.month, gong: result.monthGong },
              { label: '日数', value: result.day, gong: result.dayGong },
              { label: '时数', value: result.hour, gong: result.hourGong },
            ].map((step, index) => (
              <View key={index} style={styles.stepCardItem}>
                <Text style={styles.stepCardLabel}>{step.label}</Text>
                <Text style={styles.stepCardValue}>{step.value}</Text>
                <View style={[styles.stepGongBadge, { borderColor: step.gong.color }]}>
                  <Text style={[styles.stepGongName, { color: step.gong.color }]}>{step.gong.name}</Text>
                  <Text style={[styles.stepGongNature, { color: getNatureColor(step.gong.nature) }]}>
                    {step.gong.nature}
                  </Text>
                </View>
                {index < 2 && <Text style={styles.stepCardArrow}>↓</Text>}
              </View>
            ))}
          </View>
        </View>

        {/* 最终结果 */}
        <View style={[styles.resultCard, { borderColor: result.hourGong.color }]}>
          <Text style={styles.resultLabel}>最终落宫</Text>
          <Text style={[styles.resultName, { color: result.hourGong.color }]}>{result.hourGong.name}</Text>
          <View style={[styles.resultNatureBadge, { backgroundColor: getNatureColor(result.hourGong.nature) + '20' }]}>
            <Text style={[styles.resultNatureText, { color: getNatureColor(result.hourGong.nature) }]}>
              {result.hourGong.nature}
            </Text>
          </View>
          <Text style={styles.resultMeaning}>{result.hourGong.meaning}</Text>
          <View style={styles.resultMeta}>
            <View style={styles.resultMetaItem}>
              <Text style={styles.resultMetaLabel}>五行</Text>
              <Text style={styles.resultMetaValue}>{result.hourGong.element}</Text>
            </View>
            <View style={styles.resultMetaDivider} />
            <View style={styles.resultMetaItem}>
              <Text style={styles.resultMetaLabel}>方位</Text>
              <Text style={styles.resultMetaValue}>{result.hourGong.direction}</Text>
            </View>
          </View>
        </View>

        {/* 断语卡片 */}
        <View style={styles.verseCard}>
          <Text style={styles.cardTitle}>断语</Text>
          <Text style={styles.verseText}>{result.hourGong.verse}</Text>
        </View>

        {/* 操作按钮 */}
        <View style={styles.actionButtons}>
          <Pressable
            style={styles.aiButton}
            onPress={() => Alert.alert('提示', 'AI解读功能即将上线')}
          >
            <Text style={styles.aiButtonText}>AI智能解读</Text>
          </Pressable>
          <Pressable
            style={styles.detailButton}
            onPress={() => Alert.alert('提示', '详细解析功能即将上线')}
          >
            <Text style={styles.detailButtonText}>详细解析 →</Text>
          </Pressable>
        </View>

        <Pressable style={styles.resetButton} onPress={handleReset}>
          <Text style={styles.resetButtonText}>重新掐算</Text>
        </Pressable>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.navItem} onPress={() => router.push('/divination/xiaoliuren-list' as any)}>
          <Ionicons name="albums-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>我的记录</Text>
        </Pressable>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => Alert.alert('说明', '小六壬是诸葛亮马前课的简化版，用月日时三步掐算，快速得出吉凶')}>
          <Ionicons name="help-circle-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>说明</Text>
        </Pressable>
      </View>

      {/* 内容区 */}
      <ScrollView style={styles.content} contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
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
    width: 50,
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
  pickerWrapper: {
    flex: 1,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
    height: 36,
    justifyContent: 'center',
  },
  hourPickerWrapper: {
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
    paddingHorizontal: 16,
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
  unitText: {
    fontSize: 14,
    color: '#666',
  },
  tipTextSmall: {
    fontSize: 12,
    color: '#999',
  },
  methodInfo: {
    backgroundColor: '#FAFAFA',
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
    padding: 12,
    marginBottom: 16,
    alignItems: 'center',
  },
  methodInfoTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME_COLOR,
    marginBottom: 12,
  },
  stepsRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
  },
  stepItem: {
    alignItems: 'center',
    backgroundColor: '#FFF',
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
    paddingVertical: 8,
    paddingHorizontal: 16,
  },
  stepNum: {
    fontSize: 18,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  stepText: {
    fontSize: 12,
    color: '#666',
    marginTop: 2,
  },
  stepArrow: {
    fontSize: 18,
    color: '#CCC',
    marginHorizontal: 8,
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
  gongsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    justifyContent: 'center',
  },
  gongItem: {
    width: '31%',
    alignItems: 'center',
    paddingVertical: 12,
    borderWidth: 2,
    borderRadius: 4,
    backgroundColor: '#FAFAFA',
  },
  gongName: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 4,
  },
  gongElement: {
    fontSize: 12,
    color: '#666',
    marginBottom: 4,
  },
  gongNatureBadge: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
  },
  gongNatureText: {
    fontSize: 11,
    fontWeight: '600',
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
  numbersCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  numbersRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  numberItem: {
    flex: 1,
    alignItems: 'center',
  },
  numberLabel: {
    fontSize: 13,
    color: '#999',
    marginBottom: 4,
  },
  numberValue: {
    fontSize: 28,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  numberDivider: {
    width: 1,
    height: 40,
    backgroundColor: '#E8E8E8',
  },
  stepsCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  stepsContainer: {
    alignItems: 'center',
    gap: 8,
  },
  stepCardItem: {
    width: '100%',
    backgroundColor: '#FAFAFA',
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
    padding: 12,
    alignItems: 'center',
    position: 'relative',
  },
  stepCardLabel: {
    fontSize: 12,
    color: '#999',
  },
  stepCardValue: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#333',
    marginVertical: 4,
  },
  stepGongBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    borderWidth: 1,
    borderRadius: 4,
    paddingHorizontal: 12,
    paddingVertical: 4,
    gap: 8,
  },
  stepGongName: {
    fontSize: 14,
    fontWeight: '600',
  },
  stepGongNature: {
    fontSize: 12,
    fontWeight: '500',
  },
  stepCardArrow: {
    position: 'absolute',
    bottom: -18,
    fontSize: 18,
    color: '#CCC',
  },
  resultCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    borderWidth: 2,
    padding: 24,
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  resultLabel: {
    fontSize: 14,
    color: '#999',
    marginBottom: 4,
  },
  resultName: {
    fontSize: 32,
    fontWeight: 'bold',
    marginBottom: 12,
  },
  resultNatureBadge: {
    paddingHorizontal: 20,
    paddingVertical: 6,
    borderRadius: 4,
    marginBottom: 16,
  },
  resultNatureText: {
    fontSize: 16,
    fontWeight: '600',
  },
  resultMeaning: {
    fontSize: 15,
    color: '#666',
    textAlign: 'center',
    marginBottom: 16,
  },
  resultMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 24,
  },
  resultMetaItem: {
    alignItems: 'center',
  },
  resultMetaLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 2,
  },
  resultMetaValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
  },
  resultMetaDivider: {
    width: 1,
    height: 30,
    backgroundColor: '#E8E8E8',
  },
  verseCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  verseText: {
    fontSize: 15,
    color: '#666',
    lineHeight: 26,
    textAlign: 'center',
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
