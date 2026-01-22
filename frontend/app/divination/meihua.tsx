/**
 * 星尘玄鉴 - 梅花易数
 * 参考样式：专业梅花易数起卦风格
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
import { BottomNavBar } from '@/components/BottomNavBar';
import { useDivinationSave } from '@/hooks/useDivinationSave';
import { DivinationType } from '@/services/divination.service';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

// 八卦配置
const TRIGRAMS = [
  { name: '乾', symbol: '☰', element: '金', nature: '天', number: 1 },
  { name: '兑', symbol: '☱', element: '金', nature: '泽', number: 2 },
  { name: '离', symbol: '☲', element: '火', nature: '火', number: 3 },
  { name: '震', symbol: '☳', element: '木', nature: '雷', number: 4 },
  { name: '巽', symbol: '☴', element: '木', nature: '风', number: 5 },
  { name: '坎', symbol: '☵', element: '水', nature: '水', number: 6 },
  { name: '艮', symbol: '☶', element: '土', nature: '山', number: 7 },
  { name: '坤', symbol: '☷', element: '土', nature: '地', number: 8 },
];

// 五行颜色
const WU_XING_COLORS: Record<string, string> = {
  '木': '#2E7D32',
  '火': '#C62828',
  '土': '#F57C00',
  '金': '#F9A825',
  '水': '#1565C0',
};

// 起卦方式
type DivinationMethod = 'time' | 'number' | 'random' | 'manual';

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

// 问事类型
const QUESTION_TYPES = [
  { value: 0, label: '综合运势' },
  { value: 1, label: '事业工作' },
  { value: 2, label: '财运求财' },
  { value: 3, label: '婚姻感情' },
  { value: 4, label: '健康疾病' },
  { value: 5, label: '学业考试' },
  { value: 6, label: '出行远行' },
  { value: 7, label: '寻人寻物' },
];

// 卦象结果
interface HexagramResult {
  id: number;
  name: string;
  question: string;
  questionType: number;
  method: DivinationMethod;
  upperTrigram: number;
  lowerTrigram: number;
  changingLine: number;
  hexagramName: string;
  tiGua: string;
  yongGua: string;
  year: number;
  month: number;
  day: number;
  hour: number;
  createdAt: Date;
}

export default function MeihuaPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<HexagramResult | null>(null);
  const [history, setHistory] = useState<HexagramResult[]>([]);

  // 表单状态
  const [name, setName] = useState('');
  const [question, setQuestion] = useState('');
  const [questionType, setQuestionType] = useState(0);
  const [divinationMethod, setDivinationMethod] = useState<DivinationMethod>('time');
  const [year, setYear] = useState(new Date().getFullYear());
  const [month, setMonth] = useState(new Date().getMonth() + 1);
  const [day, setDay] = useState(new Date().getDate());
  const [hour, setHour] = useState(new Date().getHours());
  const [num1, setNum1] = useState('');
  const [num2, setNum2] = useState('');
  const [manualUpper, setManualUpper] = useState(1);
  const [manualLower, setManualLower] = useState(1);
  const [manualChanging, setManualChanging] = useState(1);

  // 上链保存功能
  const {
    showUnlockDialog,
    showTxStatus,
    txStatus,
    saving,
    saveToChain,
    handleUnlockSuccess,
    setShowUnlockDialog,
    setShowTxStatus,
  } = useDivinationSave({
    divinationType: DivinationType.Meihua,
    historyRoute: '/divination/history',
  });

  // 获取卦名
  const getHexagramName = (upper: number, lower: number) => {
    const names64: Record<string, string> = {
      '11': '乾为天', '12': '天泽履', '13': '天火同人', '14': '天雷无妄',
      '15': '天风姤', '16': '天水讼', '17': '天山遁', '18': '天地否',
      '21': '泽天夬', '22': '兑为泽', '23': '泽火革', '24': '泽雷随',
      '25': '泽风大过', '26': '泽水困', '27': '泽山咸', '28': '泽地萃',
      '31': '火天大有', '32': '火泽睽', '33': '离为火', '34': '火雷噬嗑',
      '35': '火风鼎', '36': '火水未济', '37': '火山旅', '38': '火地晋',
      '41': '雷天大壮', '42': '雷泽归妹', '43': '雷火丰', '44': '震为雷',
      '45': '雷风恒', '46': '雷水解', '47': '雷山小过', '48': '雷地豫',
      '51': '风天小畜', '52': '风泽中孚', '53': '风火家人', '54': '风雷益',
      '55': '巽为风', '56': '风水涣', '57': '风山渐', '58': '风地观',
      '61': '水天需', '62': '水泽节', '63': '水火既济', '64': '水雷屯',
      '65': '水风井', '66': '坎为水', '67': '水山蹇', '68': '水地比',
      '71': '山天大畜', '72': '山泽损', '73': '山火贲', '74': '山雷颐',
      '75': '山风蛊', '76': '山水蒙', '77': '艮为山', '78': '山地剥',
      '81': '地天泰', '82': '地泽临', '83': '地火明夷', '84': '地雷复',
      '85': '地风升', '86': '地水师', '87': '地山谦', '88': '坤为地',
    };
    return names64[`${upper}${lower}`] || `${TRIGRAMS[upper - 1].name}${TRIGRAMS[lower - 1].name}`;
  };

  // 计算梅花卦
  const calculateMeihua = async () => {
    if (divinationMethod === 'number') {
      const n1 = parseInt(num1);
      const n2 = parseInt(num2);
      if (isNaN(n1) || isNaN(n2) || n1 <= 0 || n2 <= 0) {
        Alert.alert('提示', '请输入有效的正整数');
        return;
      }
    }

    setLoading(true);
    await new Promise(resolve => setTimeout(resolve, 800));

    let upper: number, lower: number, changingLine: number;

    switch (divinationMethod) {
      case 'time':
        upper = ((year % 8) + (month % 8) + (day % 8)) % 8 || 8;
        lower = ((year % 8) + (month % 8) + (day % 8) + (Math.floor((hour + 1) / 2) % 8)) % 8 || 8;
        changingLine = ((year + month + day + Math.floor((hour + 1) / 2)) % 6) + 1;
        break;
      case 'number':
        upper = (parseInt(num1) % 8) || 8;
        lower = (parseInt(num2) % 8) || 8;
        changingLine = ((parseInt(num1) + parseInt(num2)) % 6) + 1;
        break;
      case 'random':
        upper = Math.floor(Math.random() * 8) + 1;
        lower = Math.floor(Math.random() * 8) + 1;
        changingLine = Math.floor(Math.random() * 6) + 1;
        break;
      case 'manual':
        upper = manualUpper;
        lower = manualLower;
        changingLine = manualChanging;
        break;
      default:
        upper = 1;
        lower = 1;
        changingLine = 1;
    }

    const hexagramName = getHexagramName(upper, lower);
    const tiGua = changingLine > 3 ? TRIGRAMS[upper - 1].name : TRIGRAMS[lower - 1].name;
    const yongGua = changingLine > 3 ? TRIGRAMS[lower - 1].name : TRIGRAMS[upper - 1].name;

    const hexagramResult: HexagramResult = {
      id: Date.now(),
      name: name || '求测者',
      question: question || '某事',
      questionType,
      method: divinationMethod,
      upperTrigram: upper,
      lowerTrigram: lower,
      changingLine,
      hexagramName,
      tiGua,
      yongGua,
      year,
      month,
      day,
      hour,
      createdAt: new Date(),
    };

    setResult(hexagramResult);
    setHistory(prev => [hexagramResult, ...prev]);
    setLoading(false);
  };

  const handleReset = () => {
    setResult(null);
  };

  // 渲染输入表单
  const renderInputForm = () => (
    <View style={styles.formContainer}>
      {/* 标题 */}
      <View style={styles.headerSection}>
        <Text style={styles.pageTitle}>星尘玄鉴-梅花易数</Text>
        <Text style={styles.pageSubtitle}>邵雍心易秘法 · 万物皆可起卦</Text>
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
              {(['time', 'number', 'random', 'manual'] as DivinationMethod[]).map(m => (
                <Pressable
                  key={m}
                  style={[styles.methodBtn, divinationMethod === m && styles.methodBtnActive]}
                  onPress={() => setDivinationMethod(m)}
                >
                  <Text style={[styles.methodBtnText, divinationMethod === m && styles.methodBtnTextActive]}>
                    {m === 'time' ? '时间' : m === 'number' ? '数字' : m === 'random' ? '随机' : '指定'}
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
              <Text style={styles.formLabel}>起卦日期：</Text>
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
              <Text style={styles.formLabel}>起卦时辰：</Text>
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
            <Text style={styles.formLabel}>起卦数字：</Text>
            <View style={styles.formContent}>
              <Text style={styles.unitText}>上卦数</Text>
              <TextInput
                style={styles.numberInput}
                value={num1}
                onChangeText={setNum1}
                keyboardType="number-pad"
                maxLength={5}
                placeholder="1"
                placeholderTextColor="#999"
              />
              <Text style={styles.unitText}>下卦数</Text>
              <TextInput
                style={styles.numberInput}
                value={num2}
                onChangeText={setNum2}
                keyboardType="number-pad"
                maxLength={5}
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

        {/* 指定起卦 */}
        {divinationMethod === 'manual' && (
          <>
            <View style={styles.formRow}>
              <Text style={styles.formLabel}>上卦：</Text>
              <View style={styles.formContent}>
                <View style={styles.fullPickerWrapper}>
                  <Picker selectedValue={manualUpper} onValueChange={setManualUpper} style={styles.picker}>
                    {TRIGRAMS.map((t, i) => (
                      <Picker.Item key={i} label={`${t.name}卦 (${t.symbol})`} value={i + 1} />
                    ))}
                  </Picker>
                </View>
              </View>
            </View>
            <View style={styles.formRow}>
              <Text style={styles.formLabel}>下卦：</Text>
              <View style={styles.formContent}>
                <View style={styles.fullPickerWrapper}>
                  <Picker selectedValue={manualLower} onValueChange={setManualLower} style={styles.picker}>
                    {TRIGRAMS.map((t, i) => (
                      <Picker.Item key={i} label={`${t.name}卦 (${t.symbol})`} value={i + 1} />
                    ))}
                  </Picker>
                </View>
              </View>
            </View>
            <View style={styles.formRow}>
              <Text style={styles.formLabel}>动爻：</Text>
              <View style={styles.formContent}>
                <View style={styles.smallPickerWrapper}>
                  <Picker selectedValue={manualChanging} onValueChange={setManualChanging} style={styles.picker}>
                    {[1, 2, 3, 4, 5, 6].map(n => (
                      <Picker.Item key={n} label={`第${n}爻`} value={n} />
                    ))}
                  </Picker>
                </View>
              </View>
            </View>
          </>
        )}

        {/* 本地预览按钮 */}
        <Pressable
          style={styles.secondaryButton}
          onPress={calculateMeihua}
          disabled={loading}
        >
          <Text style={styles.secondaryButtonText}>
            {loading ? '计算中...' : '本地预览（不上链）'}
          </Text>
        </Pressable>

        {/* 开始起卦按钮 */}
        <Pressable
          style={[styles.primaryButton, loading && styles.buttonDisabled]}
          onPress={() => saveToChain(result)}
          disabled={loading || saving}
        >
          {loading || saving ? (
            <ActivityIndicator color={THEME_COLOR_LIGHT} />
          ) : (
            <Text style={styles.primaryButtonText}>开始起卦（上链存储）</Text>
          )}
        </Pressable>

        <Text style={styles.tipText}>
          {divinationMethod === 'time' && '以时间年月日时自动起卦'}
          {divinationMethod === 'number' && '输入两个数字起卦'}
          {divinationMethod === 'random' && '使用随机数起卦'}
          {divinationMethod === 'manual' && '直接指定卦象'}
        </Text>
      </View>

      {/* 八卦说明 */}
      <View style={styles.infoCard}>
        <Text style={styles.cardTitle}>八卦</Text>
        <View style={styles.guaGrid}>
          {TRIGRAMS.map((gua, idx) => (
            <View key={idx} style={styles.guaItem}>
              <Text style={styles.guaSymbol}>{gua.symbol}</Text>
              <Text style={styles.guaName}>{gua.name}</Text>
              <Text style={[styles.guaElement, { color: WU_XING_COLORS[gua.element] }]}>
                {gua.element}·{gua.nature}
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

    const upper = TRIGRAMS[result.upperTrigram - 1];
    const lower = TRIGRAMS[result.lowerTrigram - 1];

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
              {result.method === 'time' ? '时间起卦' : result.method === 'number' ? '数字起卦' : result.method === 'random' ? '随机起卦' : '指定卦象'}
            </Text>
          </View>
        </View>

        {/* 卦名卡片 */}
        <View style={styles.hexagramCard}>
          <Text style={styles.hexagramTitle}>{result.hexagramName}</Text>
          <View style={styles.hexagramTags}>
            <View style={[styles.hexagramTag, { backgroundColor: '#E74C3C20' }]}>
              <Text style={[styles.hexagramTagText, { color: '#E74C3C' }]}>动爻：第{result.changingLine}爻</Text>
            </View>
          </View>
        </View>

        {/* 上下卦显示 */}
        <View style={styles.panCard}>
          <Text style={styles.cardTitle}>卦象组成</Text>
          <View style={styles.hexagramDisplay}>
            <View style={styles.trigramCard}>
              <Text style={styles.trigramLabel}>上卦</Text>
              <Text style={styles.trigramSymbol}>{upper.symbol}</Text>
              <Text style={styles.trigramName}>{upper.name}卦</Text>
              <Text style={[styles.trigramElement, { color: WU_XING_COLORS[upper.element] }]}>
                {upper.element} · {upper.nature}
              </Text>
            </View>
            <View style={styles.hexagramConnector}>
              <Text style={styles.connectorText}>+</Text>
            </View>
            <View style={styles.trigramCard}>
              <Text style={styles.trigramLabel}>下卦</Text>
              <Text style={styles.trigramSymbol}>{lower.symbol}</Text>
              <Text style={styles.trigramName}>{lower.name}卦</Text>
              <Text style={[styles.trigramElement, { color: WU_XING_COLORS[lower.element] }]}>
                {lower.element} · {lower.nature}
              </Text>
            </View>
          </View>
        </View>

        {/* 体用分析 */}
        <View style={styles.analysisCard}>
          <Text style={styles.cardTitle}>体用分析</Text>
          <View style={styles.analysisRow}>
            <View style={styles.analysisItem}>
              <Text style={styles.analysisLabel}>体卦</Text>
              <View style={[styles.analysisBadge, { backgroundColor: '#667EEA20' }]}>
                <Text style={[styles.analysisValue, { color: '#667EEA' }]}>{result.tiGua}</Text>
              </View>
            </View>
            <View style={styles.analysisItem}>
              <Text style={styles.analysisLabel}>用卦</Text>
              <View style={[styles.analysisBadge, { backgroundColor: '#F39C1220' }]}>
                <Text style={[styles.analysisValue, { color: '#F39C12' }]}>{result.yongGua}</Text>
              </View>
            </View>
          </View>
          <View style={styles.analysisTip}>
            <Text style={styles.analysisTipText}>
              体用关系：体为自己，用为所问之事。用生体吉，体克用吉。
            </Text>
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
        <Pressable style={styles.navItem} onPress={() => router.push('/divination/history' as any)}>
          <Ionicons name="albums-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>我的记录</Text>
        </Pressable>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => Alert.alert('说明', '梅花易数是宋代邵雍所创的易学占卜术，以时间、数字等快速起卦')}>
          <Ionicons name="help-circle-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>说明</Text>
        </Pressable>
      </View>

      {/* 内容区 */}
      <ScrollView style={styles.content} contentContainerStyle={styles.scrollContent} showsVerticalScrollIndicator={false}>
        {result ? renderResult() : renderInputForm()}
      </ScrollView>

      {/* 解锁钱包对话框 */}
      <UnlockWalletDialog
        visible={showUnlockDialog}
        onClose={() => setShowUnlockDialog(false)}
        onSuccess={(password) => handleUnlockSuccess(password, result)}
      />

      {/* 交易状态对话框 */}
      <TransactionStatusDialog
        visible={showTxStatus}
        status={txStatus}
        onClose={() => setShowTxStatus(false)}
      />

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="divination" />
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
    width: 60,
    height: 36,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    paddingHorizontal: 8,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FFF',
    textAlign: 'center',
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
  smallPickerWrapper: {
    width: 100,
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
  guaGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  guaItem: {
    width: '23%',
    alignItems: 'center',
    paddingVertical: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
  },
  guaSymbol: {
    fontSize: 20,
    color: '#333',
  },
  guaName: {
    fontSize: 12,
    color: '#333',
    fontWeight: '500',
    marginTop: 2,
  },
  guaElement: {
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
  hexagramCard: {
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
  hexagramTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: THEME_COLOR,
    marginBottom: 12,
  },
  hexagramTags: {
    flexDirection: 'row',
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
  hexagramDisplay: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  trigramCard: {
    flex: 1,
    alignItems: 'center',
    paddingVertical: 16,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    backgroundColor: '#FAFAFA',
  },
  trigramLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 8,
  },
  trigramSymbol: {
    fontSize: 36,
    color: '#333',
    marginBottom: 4,
  },
  trigramName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginBottom: 4,
  },
  trigramElement: {
    fontSize: 12,
  },
  hexagramConnector: {
    width: 40,
    alignItems: 'center',
  },
  connectorText: {
    fontSize: 24,
    color: '#CCC',
  },
  analysisCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  analysisRow: {
    flexDirection: 'row',
    gap: 16,
    marginBottom: 12,
  },
  analysisItem: {
    flex: 1,
    alignItems: 'center',
  },
  analysisLabel: {
    fontSize: 14,
    color: '#999',
    marginBottom: 8,
  },
  analysisBadge: {
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  analysisValue: {
    fontSize: 18,
    fontWeight: '600',
  },
  analysisTip: {
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  analysisTipText: {
    fontSize: 12,
    color: '#999',
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
