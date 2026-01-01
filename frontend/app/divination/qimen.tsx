/**
 * 星尘玄鉴 - 奇门遁甲排盘
 * 参考样式：专业奇门遁甲排盘风格
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

// 九宫
const JIU_GONG = ['坎一宫', '坤二宫', '震三宫', '巽四宫', '中五宫', '乾六宫', '兑七宫', '艮八宫', '离九宫'];
const JIU_GONG_SHORT = ['坎', '坤', '震', '巽', '中', '乾', '兑', '艮', '离'];
const JIU_GONG_FANGWEI = ['北', '西南', '东', '东南', '中', '西北', '西', '东北', '南'];

// 八门
const BA_MEN = ['休门', '生门', '伤门', '杜门', '景门', '死门', '惊门', '开门'];
const BA_MEN_JI_XIONG: Record<string, number> = {
  '休门': 1, '生门': 1, '开门': 1, // 吉
  '伤门': -1, '杜门': 0, '景门': 0, '死门': -1, '惊门': -1, // 凶/平
};

// 九星
const JIU_XING = ['天蓬', '天任', '天冲', '天辅', '天英', '天芮', '天柱', '天心', '天禽'];
const JIU_XING_JI_XIONG: Record<string, number> = {
  '天蓬': -1, '天芮': -1, '天柱': -1, // 凶
  '天任': 1, '天冲': 1, '天辅': 1, '天心': 1, // 吉
  '天英': 0, '天禽': 0, // 平
};

// 八神
const BA_SHEN = ['值符', '腾蛇', '太阴', '六合', '白虎', '玄武', '九地', '九天'];
const BA_SHEN_JI_XIONG: Record<string, number> = {
  '值符': 1, '太阴': 1, '六合': 1, '九天': 1, // 吉
  '腾蛇': -1, '白虎': -1, '玄武': -1, // 凶
  '九地': 0, // 平
};

// 三奇六仪
const SAN_QI = ['乙', '丙', '丁']; // 日奇、月奇、星奇
const LIU_YI = ['戊', '己', '庚', '辛', '壬', '癸'];

// 起局方式
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
  { value: 10, label: '合作交易' },
  { value: 11, label: '祈福求神' },
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

// 局数选项
const JU_OPTIONS = [
  { value: 'yang1', label: '阳一局' },
  { value: 'yang2', label: '阳二局' },
  { value: 'yang3', label: '阳三局' },
  { value: 'yang4', label: '阳四局' },
  { value: 'yang5', label: '阳五局' },
  { value: 'yang6', label: '阳六局' },
  { value: 'yang7', label: '阳七局' },
  { value: 'yang8', label: '阳八局' },
  { value: 'yang9', label: '阳九局' },
  { value: 'yin1', label: '阴一局' },
  { value: 'yin2', label: '阴二局' },
  { value: 'yin3', label: '阴三局' },
  { value: 'yin4', label: '阴四局' },
  { value: 'yin5', label: '阴五局' },
  { value: 'yin6', label: '阴六局' },
  { value: 'yin7', label: '阴七局' },
  { value: 'yin8', label: '阴八局' },
  { value: 'yin9', label: '阴九局' },
];

// 性别类型
type Gender = 'male' | 'female';

// 排法类型
type PaiMethod = 'zhuanpan' | 'feigong';

// 宫位信息
interface GongWei {
  gong: number;
  tianPanGan: string;
  diPanGan: string;
  men: string;
  xing: string;
  shen: string;
  isKong: boolean;
  isMa: boolean;
}

// 奇门结果
interface QimenResult {
  id: number;
  name: string;
  gender: Gender;
  birthYear: number;
  question: string;
  questionType: number;
  divinationMethod: DivinationMethod;
  year: number;
  month: number;
  day: number;
  hour: number;
  juShu: string;
  isYangDun: boolean;
  zhiFu: string;
  zhiShi: string;
  xunShou: string;
  jieQi: string;
  gongWeis: GongWei[];
  createdAt: Date;
}

export default function QimenPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<QimenResult | null>(null);
  const [history, setHistory] = useState<QimenResult[]>([]);

  // 表单状态
  const [name, setName] = useState('');
  const [gender, setGender] = useState<Gender>('male');
  const [birthYear, setBirthYear] = useState(1990);
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
  const [numberYangDun, setNumberYangDun] = useState(true);
  const [manualJu, setManualJu] = useState('yang1');
  const [paiMethod, setPaiMethod] = useState<PaiMethod>('zhuanpan');

  // 节气列表
  const JIE_QI = ['小寒', '大寒', '立春', '雨水', '惊蛰', '春分', '清明', '谷雨',
    '立夏', '小满', '芒种', '夏至', '小暑', '大暑', '立秋', '处暑',
    '白露', '秋分', '寒露', '霜降', '立冬', '小雪', '大雪', '冬至'];

  // 计算奇门盘
  const calculateQimen = async () => {
    setLoading(true);
    await new Promise(resolve => setTimeout(resolve, 800));

    // 简化计算
    const shiChen = Math.floor(((hour + 1) % 24) / 2);
    const juNum = (month % 9) + 1;
    const isYang = month <= 6;
    const juShu = isYang ? `阳${['一', '二', '三', '四', '五', '六', '七', '八', '九'][juNum - 1]}局` : `阴${['一', '二', '三', '四', '五', '六', '七', '八', '九'][juNum - 1]}局`;

    // 生成九宫信息
    const gongWeis: GongWei[] = [];
    for (let i = 0; i < 9; i++) {
      const qiYiPool = [...SAN_QI, ...LIU_YI];
      gongWeis.push({
        gong: i,
        tianPanGan: qiYiPool[(i + hour) % 9],
        diPanGan: qiYiPool[(i + 4) % 9],
        men: BA_MEN[i % 8],
        xing: JIU_XING[i],
        shen: BA_SHEN[i % 8],
        isKong: i === 4,
        isMa: i === 2,
      });
    }

    const qimenResult: QimenResult = {
      id: Date.now(),
      name: name || '求测者',
      gender,
      birthYear,
      question: question || '某事',
      questionType,
      divinationMethod,
      year,
      month,
      day,
      hour,
      juShu,
      isYangDun: isYang,
      zhiFu: JIU_XING[juNum - 1],
      zhiShi: BA_MEN[(juNum - 1) % 8],
      xunShou: TIAN_GAN[(year + month + day) % 10],
      jieQi: JIE_QI[(month * 2 - 2 + Math.floor(day / 16)) % 24],
      gongWeis,
      createdAt: new Date(),
    };

    setResult(qimenResult);
    setHistory(prev => [qimenResult, ...prev]);
    setLoading(false);
  };

  const handleReset = () => {
    setResult(null);
  };

  // 获取吉凶颜色
  const getJiXiongColor = (value: number) => {
    if (value > 0) return '#27AE60';
    if (value < 0) return '#E74C3C';
    return '#F39C12';
  };

  // 渲染九宫格单元
  const renderGongWeiCell = (gongWei: GongWei) => {
    const isCenter = gongWei.gong === 4;
    const menJiXiong = BA_MEN_JI_XIONG[gongWei.men] || 0;
    const xingJiXiong = JIU_XING_JI_XIONG[gongWei.xing] || 0;
    const shenJiXiong = BA_SHEN_JI_XIONG[gongWei.shen] || 0;

    return (
      <View style={styles.gongWeiCell} key={gongWei.gong}>
        {/* 宫位标题 */}
        <View style={styles.gongWeiHeader}>
          <Text style={styles.gongWeiName}>{JIU_GONG_SHORT[gongWei.gong]}</Text>
          <Text style={styles.gongWeiFangwei}>{JIU_GONG_FANGWEI[gongWei.gong]}</Text>
        </View>

        {isCenter ? (
          <View style={styles.centerGong}>
            <Text style={styles.centerText}>中宫</Text>
          </View>
        ) : (
          <>
            {/* 天盘+地盘干 */}
            <View style={styles.ganRow}>
              <Text style={styles.ganText}>{gongWei.tianPanGan}+{gongWei.diPanGan}</Text>
            </View>
            {/* 八门 */}
            <View style={[styles.tagRow, { borderColor: getJiXiongColor(menJiXiong) }]}>
              <Text style={[styles.tagText, { color: getJiXiongColor(menJiXiong) }]}>
                {gongWei.men}{menJiXiong > 0 ? '吉' : menJiXiong < 0 ? '凶' : ''}
              </Text>
            </View>
            {/* 九星 */}
            <View style={[styles.tagRow, { borderColor: getJiXiongColor(xingJiXiong) }]}>
              <Text style={[styles.tagText, { color: getJiXiongColor(xingJiXiong) }]}>{gongWei.xing}</Text>
            </View>
            {/* 八神 */}
            <View style={[styles.tagRow, { borderColor: getJiXiongColor(shenJiXiong) }]}>
              <Text style={[styles.tagText, { color: getJiXiongColor(shenJiXiong) }]}>{gongWei.shen}</Text>
            </View>
            {/* 特殊标记 */}
            <View style={styles.markRow}>
              {gongWei.isKong && <Text style={styles.markText}>空</Text>}
              {gongWei.isMa && <Text style={[styles.markText, { color: THEME_COLOR }]}>马</Text>}
            </View>
          </>
        )}
      </View>
    );
  };

  // 渲染输入表单
  const renderInputForm = () => (
    <View style={styles.formContainer}>
      {/* 标题 */}
      <View style={styles.headerSection}>
        <Text style={styles.pageTitle}>星尘玄鉴-奇门遁甲</Text>
        <Text style={styles.pageSubtitle}>帝王之术 · 天地人神</Text>
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

        {/* 出生年份 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>出生年份：</Text>
          <View style={styles.formContent}>
            <View style={styles.smallPickerWrapper}>
              <Picker
                selectedValue={birthYear}
                onValueChange={setBirthYear}
                style={styles.picker}
              >
                {Array.from({ length: 100 }, (_, i) => 1950 + i).map(y => (
                  <Picker.Item key={y} label={`${y}`} value={y} />
                ))}
              </Picker>
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

        {/* 起局方式 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>起局方式：</Text>
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

        {/* 公历时间起局 */}
        {divinationMethod === 'solar' && (
          <>
            <View style={styles.formRow}>
              <Text style={styles.formLabel}>起局日期：</Text>
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
              <Text style={styles.formLabel}>起局时辰：</Text>
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

        {/* 数字起局 */}
        {divinationMethod === 'number' && (
          <>
            <View style={styles.formRow}>
              <Text style={styles.formLabel}>起局数字：</Text>
              <View style={styles.formContent}>
                <Text style={styles.unitText}>上卦数</Text>
                <TextInput
                  style={styles.numberInput}
                  value={upperNumber}
                  onChangeText={setUpperNumber}
                  keyboardType="number-pad"
                  maxLength={3}
                />
                <Text style={styles.unitText}>下卦数</Text>
                <TextInput
                  style={styles.numberInput}
                  value={lowerNumber}
                  onChangeText={setLowerNumber}
                  keyboardType="number-pad"
                  maxLength={3}
                />
              </View>
            </View>
            <View style={styles.formRow}>
              <Text style={styles.formLabel}>遁法选择：</Text>
              <View style={styles.formContent}>
                <View style={styles.genderButtons}>
                  <Pressable
                    style={[styles.genderBtn, numberYangDun && styles.genderBtnActive]}
                    onPress={() => setNumberYangDun(true)}
                  >
                    <Text style={[styles.genderBtnText, numberYangDun && styles.genderBtnTextActive]}>阳遁</Text>
                  </Pressable>
                  <Pressable
                    style={[styles.genderBtn, !numberYangDun && styles.genderBtnActive]}
                    onPress={() => setNumberYangDun(false)}
                  >
                    <Text style={[styles.genderBtnText, !numberYangDun && styles.genderBtnTextActive]}>阴遁</Text>
                  </Pressable>
                </View>
              </View>
            </View>
          </>
        )}

        {/* 指定起局 */}
        {divinationMethod === 'manual' && (
          <View style={styles.formRow}>
            <Text style={styles.formLabel}>指定局数：</Text>
            <View style={styles.formContent}>
              <View style={styles.fullPickerWrapper}>
                <Picker selectedValue={manualJu} onValueChange={setManualJu} style={styles.picker}>
                  {JU_OPTIONS.map(opt => (
                    <Picker.Item key={opt.value} label={opt.label} value={opt.value} />
                  ))}
                </Picker>
              </View>
            </View>
          </View>
        )}

        {/* 随机起局说明 */}
        {divinationMethod === 'random' && (
          <View style={styles.formRow}>
            <Text style={styles.formLabel}></Text>
            <View style={styles.formContent}>
              <Text style={styles.tipTextSmall}>使用区块链随机数自动生成奇门遁甲盘</Text>
            </View>
          </View>
        )}

        {/* 排盘方法 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>排盘方法：</Text>
          <View style={styles.formContent}>
            <View style={styles.genderButtons}>
              <Pressable
                style={[styles.genderBtn, paiMethod === 'zhuanpan' && styles.genderBtnActive]}
                onPress={() => setPaiMethod('zhuanpan')}
              >
                <Text style={[styles.genderBtnText, paiMethod === 'zhuanpan' && styles.genderBtnTextActive]}>转盘</Text>
              </Pressable>
              <Pressable
                style={[styles.genderBtn, paiMethod === 'feigong' && styles.genderBtnActive]}
                onPress={() => setPaiMethod('feigong')}
              >
                <Text style={[styles.genderBtnText, paiMethod === 'feigong' && styles.genderBtnTextActive]}>飞宫</Text>
              </Pressable>
            </View>
          </View>
        </View>

        {/* 本地预览按钮 */}
        {divinationMethod === 'solar' && (
          <Pressable
            style={styles.secondaryButton}
            onPress={calculateQimen}
            disabled={loading}
          >
            <Text style={styles.secondaryButtonText}>
              {loading ? '计算中...' : '本地预览（不上链）'}
            </Text>
          </Pressable>
        )}

        {/* 开始排盘按钮 */}
        <Pressable
          style={[styles.primaryButton, loading && styles.buttonDisabled]}
          onPress={calculateQimen}
          disabled={loading}
        >
          {loading ? (
            <ActivityIndicator color={THEME_COLOR_LIGHT} />
          ) : (
            <Text style={styles.primaryButtonText}>开始排盘（上链存储）</Text>
          )}
        </Pressable>

        <Text style={styles.tipText}>
          {divinationMethod === 'solar' && '输入公历日期自动排盘'}
          {divinationMethod === 'random' && '使用链上随机数起局'}
          {divinationMethod === 'number' && '输入数字起局'}
          {divinationMethod === 'manual' && '直接指定局数'}
        </Text>
      </View>
    </View>
  );

  // 渲染结果
  const renderResult = () => {
    if (!result) return null;

    // 九宫格布局顺序（上南下北，左东右西）
    // 巽四 离九 坤二
    // 震三 中五 兑七
    // 艮八 坎一 乾六
    const layoutOrder = [3, 8, 1, 2, 4, 6, 7, 0, 5];

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
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>节气</Text>
            <Text style={styles.infoValue}>{result.jieQi}</Text>
          </View>
        </View>

        {/* 局数信息 */}
        <View style={styles.juCard}>
          <Text style={styles.juTitle}>{result.juShu}</Text>
          <View style={styles.juTags}>
            <View style={[styles.juTag, { backgroundColor: '#E74C3C20' }]}>
              <Text style={[styles.juTagText, { color: '#E74C3C' }]}>值符：{result.zhiFu}</Text>
            </View>
            <View style={[styles.juTag, { backgroundColor: '#3498DB20' }]}>
              <Text style={[styles.juTagText, { color: '#3498DB' }]}>值使：{result.zhiShi}</Text>
            </View>
            <View style={[styles.juTag, { backgroundColor: '#95A5A620' }]}>
              <Text style={[styles.juTagText, { color: '#666' }]}>旬首：{result.xunShou}</Text>
            </View>
          </View>
        </View>

        {/* 九宫格 */}
        <View style={styles.panCard}>
          <Text style={styles.cardTitle}>九宫排盘</Text>
          <View style={styles.gridContainer}>
            {[0, 1, 2].map(row => (
              <View key={row} style={styles.gridRow}>
                {[0, 1, 2].map(col => {
                  const idx = row * 3 + col;
                  const gongIdx = layoutOrder[idx];
                  return renderGongWeiCell(result.gongWeis[gongIdx]);
                })}
              </View>
            ))}
          </View>
          <Text style={styles.gridNote}>上南下北 · 左东右西</Text>
        </View>

        {/* 三奇六仪说明 */}
        <View style={styles.qiyiCard}>
          <Text style={styles.cardTitle}>三奇六仪</Text>
          <View style={styles.qiyiRow}>
            <Text style={styles.qiyiLabel}>三奇：</Text>
            <View style={styles.qiyiTags}>
              {SAN_QI.map((qi, i) => (
                <View key={i} style={[styles.qiyiTag, { backgroundColor: '#27AE6020' }]}>
                  <Text style={[styles.qiyiTagText, { color: '#27AE60' }]}>
                    {qi}{i === 0 ? '(日奇)' : i === 1 ? '(月奇)' : '(星奇)'}
                  </Text>
                </View>
              ))}
            </View>
          </View>
          <View style={styles.qiyiRow}>
            <Text style={styles.qiyiLabel}>六仪：</Text>
            <View style={styles.qiyiTags}>
              {LIU_YI.map((yi, i) => (
                <View key={i} style={[styles.qiyiTag, { backgroundColor: '#95A5A620' }]}>
                  <Text style={[styles.qiyiTagText, { color: '#666' }]}>{yi}</Text>
                </View>
              ))}
            </View>
          </View>
        </View>

        {/* 操作按钮 */}
        <View style={styles.actionButtons}>
          <Pressable
            style={styles.aiButton}
            onPress={() => Alert.alert('提示', 'AI解读功能即将上线')}
          >
            <Text style={styles.aiButtonText}>AI智能解盘</Text>
          </Pressable>
          <Pressable
            style={styles.detailButton}
            onPress={() => Alert.alert('提示', '详情页面即将上线')}
          >
            <Text style={styles.detailButtonText}>查看详细解卦 →</Text>
          </Pressable>
        </View>

        <Pressable style={styles.resetButton} onPress={handleReset}>
          <Text style={styles.resetButtonText}>重新排盘</Text>
        </Pressable>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.navItem} onPress={() => router.push('/divination/qimen-list' as any)}>
          <Ionicons name="albums-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>我的排盘</Text>
        </Pressable>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => Alert.alert('说明', '奇门遁甲是中国古代最高层次的预测学，号称"帝王之学"')}>
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
  smallPickerWrapper: {
    width: 90,
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
  resultContainer: {
    gap: 12,
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
  juCard: {
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
  juTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: THEME_COLOR,
    marginBottom: 12,
  },
  juTags: {
    flexDirection: 'row',
    gap: 8,
    flexWrap: 'wrap',
    justifyContent: 'center',
  },
  juTag: {
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 4,
  },
  juTagText: {
    fontSize: 12,
    fontWeight: '500',
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
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    textAlign: 'center',
    marginBottom: 16,
  },
  gridContainer: {
    gap: 4,
  },
  gridRow: {
    flexDirection: 'row',
    gap: 4,
  },
  gongWeiCell: {
    flex: 1,
    minHeight: 100,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    borderRadius: 4,
    padding: 6,
    backgroundColor: '#FAFAFA',
  },
  gongWeiHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 4,
  },
  gongWeiName: {
    fontSize: 10,
    fontWeight: 'bold',
    color: '#333',
  },
  gongWeiFangwei: {
    fontSize: 9,
    color: '#999',
  },
  centerGong: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  centerText: {
    fontSize: 12,
    color: '#999',
  },
  ganRow: {
    marginBottom: 2,
  },
  ganText: {
    fontSize: 10,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  tagRow: {
    borderWidth: 1,
    borderRadius: 2,
    paddingHorizontal: 4,
    paddingVertical: 1,
    marginBottom: 2,
    alignSelf: 'flex-start',
  },
  tagText: {
    fontSize: 9,
    fontWeight: '500',
  },
  markRow: {
    flexDirection: 'row',
    gap: 4,
    marginTop: 2,
  },
  markText: {
    fontSize: 8,
    color: '#9B59B6',
    fontWeight: '500',
  },
  gridNote: {
    fontSize: 10,
    color: '#999',
    textAlign: 'center',
    marginTop: 8,
  },
  qiyiCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  qiyiRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  qiyiLabel: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
    width: 50,
  },
  qiyiTags: {
    flex: 1,
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 6,
  },
  qiyiTag: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  qiyiTagText: {
    fontSize: 11,
    fontWeight: '500',
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
