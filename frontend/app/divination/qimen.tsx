import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  Platform,
  Pressable,
  ScrollView,
  StyleSheet,
  Switch,
  Text,
  TextInput,
  View
} from 'react-native';

const QUESTION_TYPES = [
  { id: 0, name: '综合', icon: '🔮' },
  { id: 1, name: '事业', icon: '💼' },
  { id: 2, name: '财运', icon: '💰' },
  { id: 3, name: '婚姻', icon: '💑' },
  { id: 4, name: '健康', icon: '🏥' },
  { id: 5, name: '学业', icon: '📚' },
  { id: 6, name: '出行', icon: '✈️' },
  { id: 7, name: '官司', icon: '⚖️' },
  { id: 8, name: '寻物', icon: '🔍' },
  { id: 9, name: '投资', icon: '📈' },
  { id: 10, name: '生意', icon: '🏪' },
  { id: 11, name: '祈福', icon: '🙏' },
];

type Gender = 'male' | 'female';
type PanMethod = 'zhuan' | 'fei';

interface QimenResult {
  chartId: number;
  dunType: string;
  juNumber: number;
  zhiFuXing: string;
  zhiShiMen: string;
  yearGanzhi: string;
  monthGanzhi: string;
  dayGanzhi: string;
  hourGanzhi: string;
  jieQi: string;
  sanYuan: string;
}

export default function QimenScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { createQimenChart, isLoading: isTxLoading, status: txStatus, result: txResult } = useTransaction();
  
  const [useCurrentTime, setUseCurrentTime] = useState(true);
  const [divinationTime, setDivinationTime] = useState({
    year: '',
    month: '',
    day: '',
    hour: '',
  });
  const [question, setQuestion] = useState('');
  const [questionType, setQuestionType] = useState<number>(0);
  const [panMethod, setPanMethod] = useState<PanMethod>('zhuan');
  const [gender, setGender] = useState<Gender | null>(null);
  const [chartName, setChartName] = useState('');
  const [isPublic, setIsPublic] = useState(false);
  
  const [result, setResult] = useState<QimenResult | null>(null);

  const showAlert = (title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      const { Alert } = require('react-native');
      Alert.alert(title, message);
    }
  };

  const getCurrentTime = () => {
    const now = new Date();
    return {
      year: now.getFullYear(),
      month: now.getMonth() + 1,
      day: now.getDate(),
      hour: now.getHours(),
    };
  };

  const validateInput = (): boolean => {
    if (!useCurrentTime) {
      const { year, month, day, hour } = divinationTime;
      if (!year || !month || !day || !hour) {
        showAlert('提示', '请填写完整的起局时间');
        return false;
      }
      
      const y = parseInt(year);
      const m = parseInt(month);
      const d = parseInt(day);
      const h = parseInt(hour);
      
      if (y < 1901 || y > 2100) {
        showAlert('提示', '年份范围：1901-2100');
        return false;
      }
      if (m < 1 || m > 12) {
        showAlert('提示', '月份范围：1-12');
        return false;
      }
      if (d < 1 || d > 31) {
        showAlert('提示', '日期范围：1-31');
        return false;
      }
      if (h < 0 || h > 23) {
        showAlert('提示', '小时范围：0-23');
        return false;
      }
    }
    return true;
  };

  const generateQuestionHash = (q: string): number[] => {
    const hash = new Array(32).fill(0);
    if (q) {
      for (let i = 0; i < q.length; i++) {
        hash[i % 32] = (hash[i % 32] + q.charCodeAt(i)) % 256;
      }
    }
    return hash;
  };

  const handleCalculate = async () => {
    if (!validateInput()) return;
    
    if (!isLoggedIn || !address) {
      showAlert('提示', '请先登录钱包后再起局');
      return;
    }
    if (!isConnected) {
      showAlert('提示', '请先连接区块链网络');
      return;
    }
    
    const time = useCurrentTime ? getCurrentTime() : {
      year: parseInt(divinationTime.year),
      month: parseInt(divinationTime.month),
      day: parseInt(divinationTime.day),
      hour: parseInt(divinationTime.hour),
    };
    
    const questionHash = generateQuestionHash(question);
    
    const txResult = await createQimenChart({
      solar_year: time.year,
      solar_month: time.month,
      solar_day: time.day,
      hour: time.hour,
      question_hash: questionHash,
      is_public: isPublic,
      name: chartName || undefined,
      gender: gender === 'male' ? 0 : gender === 'female' ? 1 : undefined,
      question: question || undefined,
      question_type: questionType,
      pan_method: panMethod === 'fei' ? 1 : 0,
    });
    
    if (txResult?.success) {
      // 从链上事件中解析排盘结果
      const events = txResult.events || [];
      let chartId = 0;
      let dunType = '阳遁';
      let juNumber = 1;
      
      for (const event of events) {
        if (event?.event?.section === 'divinationQimen' && event?.event?.method === 'ChartCreated') {
          const data = event.event.data;
          if (data) {
            chartId = parseInt(data.chart_id || data.chartId || '0');
            dunType = data.dun_type === 'Yin' || data.dunType === 'Yin' ? '阴遁' : '阳遁';
            juNumber = parseInt(data.ju_number || data.juNumber || '1');
          }
        }
      }
      
      setResult({
        chartId,
        dunType,
        juNumber,
        zhiFuXing: '值符',
        zhiShiMen: '值使',
        yearGanzhi: '-',
        monthGanzhi: '-',
        dayGanzhi: '-',
        hourGanzhi: '-',
        jieQi: '-',
        sanYuan: '-',
      });
      
      console.log('奇门起局成功:', txResult);
    }
  };

  const handleReset = () => {
    setDivinationTime({ year: '', month: '', day: '', hour: '' });
    setQuestion('');
    setQuestionType(0);
    setResult(null);
    setChartName('');
    setGender(null);
  };

  const handleFindMaster = () => {
    router.push('/divination/masters/qimen' as any);
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>🧭 奇门遁甲</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.introCard}>
          <Text style={styles.introTitle}>奇门遁甲</Text>
          <Text style={styles.introDesc}>
            帝王之术，趋吉避凶。奇门遁甲以时空为盘，天地人神四盘合一，推演事物发展变化。
          </Text>
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>⏰ 起局时间</Text>
          
          <View style={styles.optionRow}>
            <View style={styles.optionInfo}>
              <Text style={styles.optionLabel}>使用当前时间</Text>
              <Text style={styles.optionHint}>自动获取当前时辰起局</Text>
            </View>
            <Switch
              value={useCurrentTime}
              onValueChange={setUseCurrentTime}
              trackColor={{ false: '#e5e7eb', true: '#a78bfa' }}
              thumbColor={useCurrentTime ? '#6D28D9' : '#f4f3f4'}
            />
          </View>

          {!useCurrentTime && (
            <View style={styles.dateInputs}>
              <View style={styles.dateInputWrapper}>
                <TextInput
                  style={styles.dateInput}
                  placeholder="2024"
                  placeholderTextColor="#9ca3af"
                  value={divinationTime.year}
                  onChangeText={(v) => setDivinationTime({ ...divinationTime, year: v.replace(/[^0-9]/g, '') })}
                  keyboardType="number-pad"
                  maxLength={4}
                />
                <Text style={styles.dateLabel}>年</Text>
              </View>
              <View style={styles.dateInputWrapper}>
                <TextInput
                  style={styles.dateInput}
                  placeholder="01"
                  placeholderTextColor="#9ca3af"
                  value={divinationTime.month}
                  onChangeText={(v) => setDivinationTime({ ...divinationTime, month: v.replace(/[^0-9]/g, '') })}
                  keyboardType="number-pad"
                  maxLength={2}
                />
                <Text style={styles.dateLabel}>月</Text>
              </View>
              <View style={styles.dateInputWrapper}>
                <TextInput
                  style={styles.dateInput}
                  placeholder="01"
                  placeholderTextColor="#9ca3af"
                  value={divinationTime.day}
                  onChangeText={(v) => setDivinationTime({ ...divinationTime, day: v.replace(/[^0-9]/g, '') })}
                  keyboardType="number-pad"
                  maxLength={2}
                />
                <Text style={styles.dateLabel}>日</Text>
              </View>
              <View style={styles.dateInputWrapper}>
                <TextInput
                  style={styles.dateInput}
                  placeholder="12"
                  placeholderTextColor="#9ca3af"
                  value={divinationTime.hour}
                  onChangeText={(v) => setDivinationTime({ ...divinationTime, hour: v.replace(/[^0-9]/g, '') })}
                  keyboardType="number-pad"
                  maxLength={2}
                />
                <Text style={styles.dateLabel}>时</Text>
              </View>
            </View>
          )}
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>❓ 占问事宜</Text>
          <TextInput
            style={styles.questionInput}
            placeholder="请输入您想占问的事情..."
            placeholderTextColor="#9ca3af"
            value={question}
            onChangeText={setQuestion}
            multiline
            numberOfLines={3}
            maxLength={128}
          />
          <Text style={styles.charCount}>{question.length}/128</Text>
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>📋 问事类型</Text>
          <Text style={styles.cardHint}>选择问事类型以获得更精准的解读</Text>
          <View style={styles.questionTypeGrid}>
            {QUESTION_TYPES.map((type) => (
              <Pressable
                key={type.id}
                style={[
                  styles.questionTypeButton,
                  questionType === type.id && styles.questionTypeButtonSelected
                ]}
                onPress={() => setQuestionType(type.id)}
              >
                <Text style={styles.questionTypeIcon}>{type.icon}</Text>
                <Text style={[
                  styles.questionTypeText,
                  questionType === type.id && styles.questionTypeTextSelected
                ]}>
                  {type.name}
                </Text>
              </Pressable>
            ))}
          </View>
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>⚙️ 高级选项</Text>
          
          <View style={styles.optionRow}>
            <View style={styles.optionInfo}>
              <Text style={styles.optionLabel}>排盘方法</Text>
              <Text style={styles.optionHint}>
                {panMethod === 'zhuan' ? '转盘：天盘随时辰转动' : '飞盘：天盘按九宫飞布'}
              </Text>
            </View>
            <View style={styles.panMethodButtons}>
              <Pressable
                style={[styles.panMethodButton, panMethod === 'zhuan' && styles.panMethodButtonSelected]}
                onPress={() => setPanMethod('zhuan')}
              >
                <Text style={[styles.panMethodText, panMethod === 'zhuan' && styles.panMethodTextSelected]}>转盘</Text>
              </Pressable>
              <Pressable
                style={[styles.panMethodButton, panMethod === 'fei' && styles.panMethodButtonSelected]}
                onPress={() => setPanMethod('fei')}
              >
                <Text style={[styles.panMethodText, panMethod === 'fei' && styles.panMethodTextSelected]}>飞盘</Text>
              </Pressable>
            </View>
          </View>

          <View style={styles.optionRow}>
            <View style={styles.optionInfo}>
              <Text style={styles.optionLabel}>公开排盘</Text>
              <Text style={styles.optionHint}>公开后其他用户可查看</Text>
            </View>
            <Switch
              value={isPublic}
              onValueChange={setIsPublic}
              trackColor={{ false: '#e5e7eb', true: '#a78bfa' }}
              thumbColor={isPublic ? '#6D28D9' : '#f4f3f4'}
            />
          </View>

          <View style={styles.optionRow}>
            <View style={styles.optionInfo}>
              <Text style={styles.optionLabel}>命主性别</Text>
              <Text style={styles.optionHint}>可选，用于年命分析</Text>
            </View>
            <View style={styles.genderButtons}>
              <Pressable
                style={[styles.genderButton, gender === 'male' && styles.genderButtonSelected]}
                onPress={() => setGender(gender === 'male' ? null : 'male')}
              >
                <Text style={[styles.genderText, gender === 'male' && styles.genderTextSelected]}>男</Text>
              </Pressable>
              <Pressable
                style={[styles.genderButton, gender === 'female' && styles.genderButtonSelected]}
                onPress={() => setGender(gender === 'female' ? null : 'female')}
              >
                <Text style={[styles.genderText, gender === 'female' && styles.genderTextSelected]}>女</Text>
              </Pressable>
            </View>
          </View>

          <View style={styles.optionRow}>
            <View style={styles.optionInfo}>
              <Text style={styles.optionLabel}>排盘名称</Text>
              <Text style={styles.optionHint}>可选，方便识别</Text>
            </View>
          </View>
          <TextInput
            style={styles.nameInput}
            placeholder="如：求职占、出行占..."
            placeholderTextColor="#9ca3af"
            value={chartName}
            onChangeText={setChartName}
            maxLength={32}
          />
        </View>

        {!result ? (
          <Pressable 
            style={[styles.calculateButton, isTxLoading && styles.buttonDisabled]} 
            onPress={handleCalculate}
            disabled={isTxLoading}
          >
            <Text style={styles.calculateButtonText}>
              {isTxLoading ? (
                txStatus === 'signing' ? '签名中...' : 
                txStatus === 'broadcasting' ? '广播中...' : 
                txStatus === 'inBlock' ? '入块中...' : '起局中...'
              ) : '🧭 起局占断（链上）'}
            </Text>
          </Pressable>
        ) : (
          <View style={styles.resultCard}>
            <Text style={styles.resultTitle}>奇门排盘</Text>
            <Text style={styles.chartIdText}>排盘ID: #{result.chartId}</Text>
            
            <View style={styles.resultInfo}>
              <View style={styles.resultRow}>
                <Text style={styles.resultLabel}>阴阳遁</Text>
                <Text style={[styles.resultValue, result.dunType === '阳遁' ? styles.yangDun : styles.yinDun]}>
                  {result.dunType}
                </Text>
              </View>
              <View style={styles.resultRow}>
                <Text style={styles.resultLabel}>局数</Text>
                <Text style={styles.resultValue}>{result.juNumber}局</Text>
              </View>
              <View style={styles.resultRow}>
                <Text style={styles.resultLabel}>值符</Text>
                <Text style={styles.resultValue}>{result.zhiFuXing}</Text>
              </View>
              <View style={styles.resultRow}>
                <Text style={styles.resultLabel}>值使</Text>
                <Text style={styles.resultValue}>{result.zhiShiMen}</Text>
              </View>
            </View>

            <View style={styles.successBadge}>
              <Text style={styles.successBadgeText}>✓ 已保存到链上</Text>
            </View>

            <Text style={styles.resultHint}>
              💡 奇门遁甲需要专业大师结合用神、格局详细解读
            </Text>

            <View style={styles.resultActions}>
              <Pressable style={styles.resetButton} onPress={handleReset}>
                <Text style={styles.resetButtonText}>重新起局</Text>
              </Pressable>
              <Pressable style={styles.findMasterButton} onPress={handleFindMaster}>
                <Text style={styles.findMasterButtonText}>🔍 找大师解盘</Text>
              </Pressable>
            </View>
          </View>
        )}

        <View style={styles.tipsCard}>
          <Text style={styles.tipsTitle}>💡 起局须知</Text>
          <Text style={styles.tipsText}>
            • 奇门遁甲以时辰起局，时间精确很重要{'\n'}
            • 问事要明确具体，心诚则灵{'\n'}
            • 转盘法为主流，飞盘法较少用{'\n'}
            • 同一事不宜反复起局{'\n'}
            • 保存到链可永久存储排盘记录
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
    backgroundColor: '#059669',
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
    marginBottom: 8,
  },
  cardHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 12,
  },
  optionRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  optionInfo: {
    flex: 1,
  },
  optionLabel: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1f2937',
  },
  optionHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  dateInputs: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 16,
  },
  dateInputWrapper: {
    alignItems: 'center',
    flex: 1,
    marginHorizontal: 4,
  },
  dateInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 10,
    padding: 12,
    fontSize: 16,
    fontWeight: '600',
    color: '#059669',
    textAlign: 'center',
    width: '100%',
  },
  dateLabel: {
    fontSize: 11,
    color: '#9ca3af',
    marginTop: 4,
  },
  questionInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    padding: 14,
    fontSize: 14,
    color: '#1f2937',
    minHeight: 80,
    textAlignVertical: 'top',
  },
  charCount: {
    fontSize: 12,
    color: '#9ca3af',
    textAlign: 'right',
    marginTop: 4,
  },
  questionTypeGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  questionTypeButton: {
    width: '23%',
    paddingVertical: 10,
    borderRadius: 10,
    backgroundColor: '#f9fafb',
    alignItems: 'center',
  },
  questionTypeButtonSelected: {
    backgroundColor: '#059669',
  },
  questionTypeIcon: {
    fontSize: 20,
    marginBottom: 4,
  },
  questionTypeText: {
    fontSize: 12,
    color: '#6b7280',
  },
  questionTypeTextSelected: {
    color: '#fff',
  },
  panMethodButtons: {
    flexDirection: 'row',
    backgroundColor: '#f3f4f6',
    borderRadius: 8,
    padding: 2,
  },
  panMethodButton: {
    paddingHorizontal: 14,
    paddingVertical: 6,
    borderRadius: 6,
  },
  panMethodButtonSelected: {
    backgroundColor: '#059669',
  },
  panMethodText: {
    fontSize: 13,
    color: '#6b7280',
  },
  panMethodTextSelected: {
    color: '#fff',
  },
  genderButtons: {
    flexDirection: 'row',
    backgroundColor: '#f3f4f6',
    borderRadius: 8,
    padding: 2,
  },
  genderButton: {
    paddingHorizontal: 14,
    paddingVertical: 6,
    borderRadius: 6,
  },
  genderButtonSelected: {
    backgroundColor: '#059669',
  },
  genderText: {
    fontSize: 13,
    color: '#6b7280',
  },
  genderTextSelected: {
    color: '#fff',
  },
  nameInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 10,
    padding: 12,
    fontSize: 14,
    color: '#1f2937',
    marginTop: 8,
  },
  calculateButton: {
    backgroundColor: '#059669',
    marginHorizontal: 16,
    marginVertical: 16,
    padding: 18,
    borderRadius: 16,
    alignItems: 'center',
  },
  calculateButtonText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: '600',
  },
  buttonDisabled: {
    opacity: 0.6,
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
    marginBottom: 4,
  },
  chartIdText: {
    fontSize: 12,
    color: '#9ca3af',
    textAlign: 'center',
    marginBottom: 16,
  },
  successBadge: {
    backgroundColor: '#dcfce7',
    paddingVertical: 8,
    paddingHorizontal: 16,
    borderRadius: 20,
    alignSelf: 'center',
    marginBottom: 16,
  },
  successBadgeText: {
    color: '#16a34a',
    fontSize: 14,
    fontWeight: '600',
  },
  resultInfo: {
    backgroundColor: '#f0fdf4',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  resultRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 6,
  },
  resultLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  resultValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#059669',
  },
  yangDun: {
    color: '#dc2626',
  },
  yinDun: {
    color: '#2563eb',
  },
  pillarsDisplay: {
    flexDirection: 'row',
    justifyContent: 'center',
    marginBottom: 16,
  },
  pillarBox: {
    alignItems: 'center',
    marginHorizontal: 6,
    padding: 10,
    backgroundColor: '#f9fafb',
    borderRadius: 10,
    minWidth: 60,
  },
  pillarLabel: {
    fontSize: 10,
    color: '#9ca3af',
    marginBottom: 4,
  },
  pillarGan: {
    fontSize: 22,
    fontWeight: 'bold',
    color: '#dc2626',
  },
  pillarZhi: {
    fontSize: 22,
    fontWeight: 'bold',
    color: '#2563eb',
  },
  resultHint: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    marginBottom: 16,
  },
  resultActions: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 12,
  },
  resetButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#059669',
    alignItems: 'center',
  },
  resetButtonText: {
    color: '#059669',
    fontSize: 15,
    fontWeight: '600',
  },
  saveButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    backgroundColor: '#22c55e',
    alignItems: 'center',
  },
  saveButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
  findMasterButton: {
    padding: 14,
    borderRadius: 12,
    backgroundColor: '#059669',
    alignItems: 'center',
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
