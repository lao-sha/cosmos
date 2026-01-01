/**
 * 星尘玄鉴 - 大六壬排盘
 * 参考样式：专业大六壬排盘风格
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

// 十天干
const TIAN_GAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];

// 十二地支
const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

// 十二将
const SHI_ER_JIANG = [
  { name: '贵人', nature: '吉', element: '土' },
  { name: '腾蛇', nature: '凶', element: '火' },
  { name: '朱雀', nature: '凶', element: '火' },
  { name: '六合', nature: '吉', element: '木' },
  { name: '勾陈', nature: '平', element: '土' },
  { name: '青龙', nature: '吉', element: '木' },
  { name: '天空', nature: '平', element: '土' },
  { name: '白虎', nature: '凶', element: '金' },
  { name: '太常', nature: '吉', element: '土' },
  { name: '玄武', nature: '凶', element: '水' },
  { name: '太阴', nature: '吉', element: '金' },
  { name: '天后', nature: '吉', element: '水' },
];

// 起课方式
type DivinationMethod = 'solar' | 'number' | 'random' | 'manual';

// 问事类型
const QUESTION_TYPES = [
  { value: 0, label: '综合运势' },
  { value: 1, label: '事业工作' },
  { value: 2, label: '财运求财' },
  { value: 3, label: '婚姻感情' },
  { value: 4, label: '健康疾病' },
  { value: 5, label: '学业考试' },
  { value: 6, label: '出行远行' },
  { value: 7, label: '官司诉讼' },
  { value: 8, label: '寻人寻物' },
  { value: 9, label: '投资理财' },
];

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

// 性别类型
type Gender = 'male' | 'female';

// 四课信息
interface SiKe {
  name: string;
  tianPan: string;
  diPan: string;
  tianJiang: string;
  jiXiong: string;
}

// 三传信息
interface SanChuan {
  name: string;
  desc: string;
  shen: string;
  tianJiang: string;
  jiXiong: string;
}

// 大六壬结果
interface DaliurenResult {
  id: number;
  name: string;
  gender: Gender;
  question: string;
  questionType: number;
  divinationMethod: DivinationMethod;
  year: number;
  month: number;
  day: number;
  hour: number;
  dayGan: string;
  dayZhi: string;
  yueJian: string;
  shiZhi: string;
  siKe: SiKe[];
  sanChuan: SanChuan[];
  tianPan: string[];
  diPan: string[];
  createdAt: Date;
}

export default function DaliurenPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<DaliurenResult | null>(null);
  const [history, setHistory] = useState<DaliurenResult[]>([]);

  // 表单状态
  const [name, setName] = useState('');
  const [gender, setGender] = useState<Gender>('male');
  const [question, setQuestion] = useState('');
  const [questionType, setQuestionType] = useState(0);
  const [divinationMethod, setDivinationMethod] = useState<DivinationMethod>('solar');
  const [year, setYear] = useState(new Date().getFullYear());
  const [month, setMonth] = useState(new Date().getMonth() + 1);
  const [day, setDay] = useState(new Date().getDate());
  const [hour, setHour] = useState(new Date().getHours());
  const [minute, setMinute] = useState(new Date().getMinutes());
  const [upperNumber, setUpperNumber] = useState('1');
  const [lowerNumber, setLowerNumber] = useState('1');

  // 计算大六壬
  const calculateDaliuren = async () => {
    setLoading(true);
    await new Promise(resolve => setTimeout(resolve, 800));

    // 简化计算
    const dayNum = Math.floor(new Date(year, month - 1, day).getTime() / (24 * 60 * 60 * 1000));
    const dayGan = TIAN_GAN[(dayNum + 9) % 10];
    const dayZhi = DI_ZHI[(dayNum + 1) % 12];
    const yueJian = DI_ZHI[(month + 1) % 12];
    const shiZhi = DI_ZHI[Math.floor((hour + 1) / 2) % 12];

    // 模拟天地盘
    const diPan = [...DI_ZHI];
    const offset = (month + hour) % 12;
    const tianPan = diPan.map((_, i) => DI_ZHI[(i + offset) % 12]);

    // 模拟四课
    const siKe: SiKe[] = [
      { name: '初课', tianPan: tianPan[(dayNum + 0) % 12], diPan: diPan[(dayNum + 0) % 12], tianJiang: SHI_ER_JIANG[(dayNum + 0) % 12].name, jiXiong: SHI_ER_JIANG[(dayNum + 0) % 12].nature },
      { name: '二课', tianPan: tianPan[(dayNum + 3) % 12], diPan: diPan[(dayNum + 3) % 12], tianJiang: SHI_ER_JIANG[(dayNum + 3) % 12].name, jiXiong: SHI_ER_JIANG[(dayNum + 3) % 12].nature },
      { name: '三课', tianPan: tianPan[(dayNum + 6) % 12], diPan: diPan[(dayNum + 6) % 12], tianJiang: SHI_ER_JIANG[(dayNum + 6) % 12].name, jiXiong: SHI_ER_JIANG[(dayNum + 6) % 12].nature },
      { name: '四课', tianPan: tianPan[(dayNum + 9) % 12], diPan: diPan[(dayNum + 9) % 12], tianJiang: SHI_ER_JIANG[(dayNum + 9) % 12].name, jiXiong: SHI_ER_JIANG[(dayNum + 9) % 12].nature },
    ];

    // 模拟三传
    const sanChuan: SanChuan[] = [
      { name: '初传', desc: '发端', shen: DI_ZHI[(dayNum + hour) % 12], tianJiang: SHI_ER_JIANG[(dayNum + hour) % 12].name, jiXiong: SHI_ER_JIANG[(dayNum + hour) % 12].nature },
      { name: '中传', desc: '过程', shen: DI_ZHI[(dayNum + hour + 4) % 12], tianJiang: SHI_ER_JIANG[(dayNum + hour + 4) % 12].name, jiXiong: SHI_ER_JIANG[(dayNum + hour + 4) % 12].nature },
      { name: '末传', desc: '结果', shen: DI_ZHI[(dayNum + hour + 8) % 12], tianJiang: SHI_ER_JIANG[(dayNum + hour + 8) % 12].name, jiXiong: SHI_ER_JIANG[(dayNum + hour + 8) % 12].nature },
    ];

    const daliurenResult: DaliurenResult = {
      id: Date.now(),
      name: name || '求测者',
      gender,
      question: question || '某事',
      questionType,
      divinationMethod,
      year,
      month,
      day,
      hour,
      dayGan,
      dayZhi,
      yueJian,
      shiZhi,
      siKe,
      sanChuan,
      tianPan,
      diPan,
      createdAt: new Date(),
    };

    setResult(daliurenResult);
    setHistory(prev => [daliurenResult, ...prev]);
    setLoading(false);
  };

  const handleReset = () => {
    setResult(null);
  };

  // 获取吉凶颜色
  const getJiXiongColor = (value: string) => {
    if (value === '吉') return '#27AE60';
    if (value === '凶') return '#E74C3C';
    return '#F39C12';
  };

  // 渲染输入表单
  const renderInputForm = () => (
    <View style={styles.formContainer}>
      {/* 标题 */}
      <View style={styles.headerSection}>
        <Text style={styles.pageTitle}>星尘玄鉴-大六壬</Text>
        <Text style={styles.pageSubtitle}>三式之首 · 天地盘起课</Text>
      </View>

      {/* 表单卡片 */}
      <View style={styles.formCard}>
        {/* 命主姓名 + 性别 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>命主姓名：</Text>
          <View style={styles.formContent}>
            <TextInput
              style={styles.nameInput}
              value={name}
              onChangeText={setName}
              placeholder="求测者"
              placeholderTextColor="#999"
            />
            <Text style={styles.genderLabel}>性别：</Text>
            <View style={styles.genderButtons}>
              <Pressable
                style={[styles.genderBtn, gender === 'male' && styles.genderBtnActive]}
                onPress={() => setGender('male')}
              >
                <Text style={[styles.genderBtnText, gender === 'male' && styles.genderBtnTextActive]}>男</Text>
              </Pressable>
              <Pressable
                style={[styles.genderBtn, gender === 'female' && styles.genderBtnActive]}
                onPress={() => setGender('female')}
              >
                <Text style={[styles.genderBtnText, gender === 'female' && styles.genderBtnTextActive]}>女</Text>
              </Pressable>
            </View>
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

        {/* 起课方式 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>起课方式：</Text>
          <View style={styles.formContent}>
            <View style={styles.methodButtons}>
              {(['solar', 'random', 'number', 'manual'] as DivinationMethod[]).map(m => (
                <Pressable
                  key={m}
                  style={[styles.methodBtn, divinationMethod === m && styles.methodBtnActive]}
                  onPress={() => setDivinationMethod(m)}
                >
                  <Text style={[styles.methodBtnText, divinationMethod === m && styles.methodBtnTextActive]}>
                    {m === 'solar' ? '公历' : m === 'random' ? '随机' : m === 'number' ? '数字' : '指定'}
                  </Text>
                </Pressable>
              ))}
            </View>
          </View>
        </View>

        {/* 公历时间起课 */}
        {divinationMethod === 'solar' && (
          <>
            <View style={styles.formRow}>
              <Text style={styles.formLabel}>起课日期：</Text>
              <View style={styles.datePickerRow}>
                <View style={styles.pickerWrapper}>
                  <Picker selectedValue={year} onValueChange={setYear} style={styles.picker}>
                    {Array.from({ length: 50 }, (_, i) => 2000 + i).map(y => (
                      <Picker.Item key={y} label={`${y}年`} value={y} />
                    ))}
                  </Picker>
                </View>
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
              <Text style={styles.formLabel}>起课时辰：</Text>
              <View style={styles.formContent}>
                <View style={styles.hourPickerWrapper}>
                  <Picker selectedValue={hour} onValueChange={setHour} style={styles.picker}>
                    {SHICHEN_OPTIONS.map(opt => (
                      <Picker.Item key={opt.value} label={opt.label} value={opt.value} />
                    ))}
                  </Picker>
                </View>
                <Text style={styles.unitText}>时</Text>
                <View style={styles.minutePickerWrapper}>
                  <Picker selectedValue={minute} onValueChange={setMinute} style={styles.picker}>
                    {Array.from({ length: 60 }, (_, i) => (
                      <Picker.Item key={i} label={`${i}`} value={i} />
                    ))}
                  </Picker>
                </View>
                <Text style={styles.unitText}>分</Text>
              </View>
            </View>
          </>
        )}

        {/* 数字起课 */}
        {divinationMethod === 'number' && (
          <View style={styles.formRow}>
            <Text style={styles.formLabel}>起课数字：</Text>
            <View style={styles.formContent}>
              <Text style={styles.unitText}>上数</Text>
              <TextInput
                style={styles.numberInput}
                value={upperNumber}
                onChangeText={setUpperNumber}
                keyboardType="number-pad"
                maxLength={3}
              />
              <Text style={styles.unitText}>下数</Text>
              <TextInput
                style={styles.numberInput}
                value={lowerNumber}
                onChangeText={setLowerNumber}
                keyboardType="number-pad"
                maxLength={3}
              />
            </View>
          </View>
        )}

        {/* 随机起课说明 */}
        {divinationMethod === 'random' && (
          <View style={styles.formRow}>
            <Text style={styles.formLabel}></Text>
            <View style={styles.formContent}>
              <Text style={styles.tipTextSmall}>使用区块链随机数自动生成大六壬课局</Text>
            </View>
          </View>
        )}

        {/* 本地预览按钮 */}
        {divinationMethod === 'solar' && (
          <Pressable
            style={styles.secondaryButton}
            onPress={calculateDaliuren}
            disabled={loading}
          >
            <Text style={styles.secondaryButtonText}>
              {loading ? '计算中...' : '本地预览（不上链）'}
            </Text>
          </Pressable>
        )}

        {/* 开始起课按钮 */}
        <Pressable
          style={[styles.primaryButton, loading && styles.buttonDisabled]}
          onPress={calculateDaliuren}
          disabled={loading}
        >
          {loading ? (
            <ActivityIndicator color={THEME_COLOR_LIGHT} />
          ) : (
            <Text style={styles.primaryButtonText}>开始起课（上链存储）</Text>
          )}
        </Pressable>

        <Text style={styles.tipText}>
          {divinationMethod === 'solar' && '输入公历日期自动起课'}
          {divinationMethod === 'random' && '使用链上随机数起课'}
          {divinationMethod === 'number' && '输入数字起课'}
          {divinationMethod === 'manual' && '直接指定课局'}
        </Text>
      </View>

      {/* 十二将说明 */}
      <View style={styles.infoCard}>
        <Text style={styles.cardTitle}>十二天将</Text>
        <View style={styles.jiangGrid}>
          {SHI_ER_JIANG.map((jiang, idx) => (
            <View key={idx} style={styles.jiangItem}>
              <Text style={styles.jiangName}>{jiang.name}</Text>
              <Text style={[styles.jiangNature, { color: getJiXiongColor(jiang.nature) }]}>
                {jiang.nature}
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
            <Text style={styles.infoLabel}>命主</Text>
            <Text style={styles.infoValue}>{result.name} ({result.gender === 'male' ? '男' : '女'})</Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>占问</Text>
            <Text style={styles.infoValue}>{result.question}</Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>时间</Text>
            <Text style={styles.infoValue}>
              {result.year}年{result.month}月{result.day}日 {DI_ZHI[Math.floor((result.hour + 1) / 2) % 12]}时
            </Text>
          </View>
        </View>

        {/* 日干支信息 */}
        <View style={styles.ganZhiCard}>
          <Text style={styles.ganZhiTitle}>{result.dayGan}{result.dayZhi}日</Text>
          <View style={styles.ganZhiTags}>
            <View style={[styles.ganZhiTag, { backgroundColor: '#E74C3C20' }]}>
              <Text style={[styles.ganZhiTagText, { color: '#E74C3C' }]}>月建：{result.yueJian}</Text>
            </View>
            <View style={[styles.ganZhiTag, { backgroundColor: '#3498DB20' }]}>
              <Text style={[styles.ganZhiTagText, { color: '#3498DB' }]}>时支：{result.shiZhi}</Text>
            </View>
          </View>
        </View>

        {/* 四课 */}
        <View style={styles.siKeCard}>
          <Text style={styles.cardTitle}>四课</Text>
          <View style={styles.siKeContainer}>
            {result.siKe.map((ke, idx) => (
              <View key={idx} style={styles.siKeItem}>
                <Text style={styles.siKeLabel}>{ke.name}</Text>
                <View style={styles.siKePan}>
                  <Text style={styles.siKeTianPan}>{ke.tianPan}</Text>
                  <View style={styles.siKeDivider} />
                  <Text style={styles.siKeDiPan}>{ke.diPan}</Text>
                </View>
                <View style={[styles.siKeJiang, { borderColor: getJiXiongColor(ke.jiXiong) }]}>
                  <Text style={[styles.siKeJiangText, { color: getJiXiongColor(ke.jiXiong) }]}>
                    {ke.tianJiang}
                  </Text>
                </View>
              </View>
            ))}
          </View>
        </View>

        {/* 三传 */}
        <View style={styles.sanChuanCard}>
          <Text style={styles.cardTitle}>三传</Text>
          <View style={styles.sanChuanContainer}>
            {result.sanChuan.map((chuan, idx) => (
              <View key={idx} style={styles.sanChuanItem}>
                <Text style={styles.sanChuanLabel}>{chuan.name}</Text>
                <Text style={styles.sanChuanDesc}>{chuan.desc}</Text>
                <View style={styles.sanChuanShenBox}>
                  <Text style={styles.sanChuanShen}>{chuan.shen}</Text>
                </View>
                <View style={[styles.sanChuanJiang, { borderColor: getJiXiongColor(chuan.jiXiong) }]}>
                  <Text style={[styles.sanChuanJiangText, { color: getJiXiongColor(chuan.jiXiong) }]}>
                    {chuan.tianJiang}
                  </Text>
                </View>
                {idx < 2 && <Text style={styles.sanChuanArrow}>→</Text>}
              </View>
            ))}
          </View>
        </View>

        {/* 天地盘 */}
        <View style={styles.panCard}>
          <Text style={styles.cardTitle}>天地盘</Text>
          <View style={styles.panContainer}>
            <View style={styles.panRow}>
              <View style={styles.panLabelBox}>
                <Text style={styles.panLabelText}>天盘</Text>
              </View>
              <ScrollView horizontal showsHorizontalScrollIndicator={false} style={styles.panScroll}>
                <View style={styles.panItems}>
                  {result.tianPan.map((zhi, i) => (
                    <View key={i} style={styles.panItemTian}>
                      <Text style={styles.panItemText}>{zhi}</Text>
                    </View>
                  ))}
                </View>
              </ScrollView>
            </View>
            <View style={styles.panRow}>
              <View style={[styles.panLabelBox, { backgroundColor: '#F39C1220' }]}>
                <Text style={[styles.panLabelText, { color: '#F39C12' }]}>地盘</Text>
              </View>
              <ScrollView horizontal showsHorizontalScrollIndicator={false} style={styles.panScroll}>
                <View style={styles.panItems}>
                  {result.diPan.map((zhi, i) => (
                    <View key={i} style={styles.panItemDi}>
                      <Text style={styles.panItemText}>{zhi}</Text>
                    </View>
                  ))}
                </View>
              </ScrollView>
            </View>
          </View>
        </View>

        {/* 操作按钮 */}
        <View style={styles.actionButtons}>
          <Pressable
            style={styles.aiButton}
            onPress={() => Alert.alert('提示', 'AI解读功能即将上线')}
          >
            <Text style={styles.aiButtonText}>AI智能解课</Text>
          </Pressable>
          <Pressable
            style={styles.detailButton}
            onPress={() => Alert.alert('提示', '类神取用功能即将上线')}
          >
            <Text style={styles.detailButtonText}>类神取用 →</Text>
          </Pressable>
        </View>

        <Pressable style={styles.resetButton} onPress={handleReset}>
          <Text style={styles.resetButtonText}>重新起课</Text>
        </Pressable>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.navItem} onPress={() => router.push('/divination/daliuren-list' as any)}>
          <Ionicons name="albums-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>我的课局</Text>
        </Pressable>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => Alert.alert('说明', '大六壬是中国古代最高层次的预测学之一，与太乙神数、奇门遁甲并称"三式"')}>
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
  nameInput: {
    flex: 1,
    height: 36,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    paddingHorizontal: 12,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FFF',
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
  genderLabel: {
    fontSize: 14,
    color: '#8B6914',
  },
  genderButtons: {
    flexDirection: 'row',
  },
  genderBtn: {
    paddingHorizontal: 16,
    paddingVertical: 6,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
  },
  genderBtnActive: {
    backgroundColor: THEME_COLOR,
    borderColor: THEME_COLOR,
  },
  genderBtnText: {
    fontSize: 14,
    color: '#666',
  },
  genderBtnTextActive: {
    color: '#FFF',
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
  datePickerRow: {
    flex: 1,
    flexDirection: 'row',
    gap: 4,
  },
  pickerWrapper: {
    flex: 1,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
    height: 36,
    justifyContent: 'center',
  },
  fullPickerWrapper: {
    flex: 1,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
    height: 36,
    justifyContent: 'center',
  },
  hourPickerWrapper: {
    width: 78,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
    height: 36,
    justifyContent: 'center',
  },
  minutePickerWrapper: {
    width: 60,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
    height: 36,
    justifyContent: 'center',
  },
  picker: {
    height: 36,
  },
  unitText: {
    fontSize: 14,
    color: '#666',
  },
  tipTextSmall: {
    fontSize: 12,
    color: '#999',
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
  jiangGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  jiangItem: {
    width: '23%',
    alignItems: 'center',
    paddingVertical: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
  },
  jiangName: {
    fontSize: 13,
    color: '#333',
    fontWeight: '500',
  },
  jiangNature: {
    fontSize: 10,
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
  ganZhiCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  ganZhiTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: THEME_COLOR,
    marginBottom: 12,
  },
  ganZhiTags: {
    flexDirection: 'row',
    gap: 8,
  },
  ganZhiTag: {
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 4,
  },
  ganZhiTagText: {
    fontSize: 12,
    fontWeight: '500',
  },
  siKeCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  siKeContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  siKeItem: {
    flex: 1,
    alignItems: 'center',
    gap: 6,
  },
  siKeLabel: {
    fontSize: 12,
    color: '#999',
  },
  siKePan: {
    alignItems: 'center',
  },
  siKeTianPan: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#667EEA',
  },
  siKeDivider: {
    width: 24,
    height: 1,
    backgroundColor: '#E8E8E8',
    marginVertical: 4,
  },
  siKeDiPan: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#F39C12',
  },
  siKeJiang: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderWidth: 1,
    borderRadius: 4,
  },
  siKeJiangText: {
    fontSize: 11,
    fontWeight: '500',
  },
  sanChuanCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  sanChuanContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  sanChuanItem: {
    flex: 1,
    alignItems: 'center',
    position: 'relative',
    gap: 4,
  },
  sanChuanLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
  },
  sanChuanDesc: {
    fontSize: 11,
    color: '#999',
  },
  sanChuanShenBox: {
    marginVertical: 8,
  },
  sanChuanShen: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#333',
  },
  sanChuanJiang: {
    paddingHorizontal: 10,
    paddingVertical: 5,
    borderWidth: 1,
    borderRadius: 4,
  },
  sanChuanJiangText: {
    fontSize: 12,
    fontWeight: '500',
  },
  sanChuanArrow: {
    position: 'absolute',
    right: -8,
    top: '50%',
    fontSize: 18,
    color: '#CCC',
  },
  panCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  panContainer: {
    gap: 12,
  },
  panRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
  },
  panLabelBox: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    backgroundColor: '#667EEA20',
    borderRadius: 4,
  },
  panLabelText: {
    fontSize: 13,
    fontWeight: '600',
    color: '#667EEA',
  },
  panScroll: {
    flex: 1,
  },
  panItems: {
    flexDirection: 'row',
    gap: 6,
  },
  panItemTian: {
    backgroundColor: '#EEF2FF',
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 4,
  },
  panItemDi: {
    backgroundColor: '#FEF3C7',
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 4,
  },
  panItemText: {
    fontSize: 13,
    fontWeight: '600',
    color: '#333',
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
