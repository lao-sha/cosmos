import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  Alert,
  Platform,
  Pressable,
  ScrollView,
  StyleSheet,
  Switch,
  Text,
  TextInput,
  View
} from 'react-native';

const TIANGAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
const DIZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

type CalendarType = 'solar' | 'lunar';
type ZiShiMode = 'modern' | 'traditional';
type Gender = 'male' | 'female';

interface BaziResult {
  yearPillar: string;
  monthPillar: string;
  dayPillar: string;
  hourPillar: string;
}

export default function BaziScreen() {
  const router = useRouter();
  const { isLoggedIn, address } = useAuthStore();
  const { isConnected } = useChainStore();
  const { createBaziChart, isLoading: isTxLoading, status: txStatus } = useTransaction();
  
  const [calendarType, setCalendarType] = useState<CalendarType>('solar');
  const [birthDate, setBirthDate] = useState({
    year: '',
    month: '',
    day: '',
    hour: '',
    minute: '',
  });
  const [isLeapMonth, setIsLeapMonth] = useState(false);
  const [gender, setGender] = useState<Gender | null>(null);
  const [zishiMode, setZishiMode] = useState<ZiShiMode>('modern');
  const [chartName, setChartName] = useState('');
  const [useTrueSolarTime, setUseTrueSolarTime] = useState(false);
  const [longitude, setLongitude] = useState('116.40');
  
  const [result, setResult] = useState<BaziResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const showAlert = (title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      Alert.alert(title, message);
    }
  };

  const validateInput = (): boolean => {
    const { year, month, day, hour } = birthDate;
    if (!year || !month || !day || !hour) {
      showAlert('提示', '请填写完整的出生时间');
      return false;
    }
    if (!gender) {
      showAlert('提示', '请选择性别');
      return false;
    }
    
    const y = parseInt(year);
    const m = parseInt(month);
    const d = parseInt(day);
    const h = parseInt(hour);
    
    if (y < 1900 || y > 2100) {
      showAlert('提示', '年份范围：1900-2100');
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
    
    return true;
  };

  const calculateLocalBazi = () => {
    const y = parseInt(birthDate.year);
    const m = parseInt(birthDate.month);
    const d = parseInt(birthDate.day);
    const h = parseInt(birthDate.hour);

    const yearGan = TIANGAN[(y - 4) % 10];
    const yearZhi = DIZHI[(y - 4) % 12];
    
    const monthGan = TIANGAN[((y - 4) % 5 * 2 + m) % 10];
    const monthZhi = DIZHI[(m + 1) % 12];
    
    const dayGan = TIANGAN[(d + y * 5 + Math.floor((y - 1) / 4)) % 10];
    const dayZhi = DIZHI[(d + y * 5 + Math.floor((y - 1) / 4) + 2) % 12];
    
    const hourZhiIndex = Math.floor((h + 1) / 2) % 12;
    const hourGan = TIANGAN[(TIANGAN.indexOf(dayGan) % 5 * 2 + hourZhiIndex) % 10];
    const hourZhi = DIZHI[hourZhiIndex];

    return {
      yearPillar: yearGan + yearZhi,
      monthPillar: monthGan + monthZhi,
      dayPillar: dayGan + dayZhi,
      hourPillar: hourGan + hourZhi,
    };
  };

  const handleCalculate = async () => {
    if (!validateInput()) return;
    
    setIsLoading(true);
    
    try {
      const localResult = calculateLocalBazi();
      setResult(localResult);
    } catch (error) {
      console.error('排盘失败:', error);
      showAlert('错误', '排盘计算失败，请重试');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSaveToChain = async () => {
    if (!result) {
      showAlert('提示', '请先进行排盘');
      return;
    }
    if (!isLoggedIn || !address) {
      showAlert('提示', '请先登录钱包');
      return;
    }
    if (!isConnected) {
      showAlert('提示', '请先连接区块链网络');
      return;
    }
    
    const y = parseInt(birthDate.year);
    const m = parseInt(birthDate.month);
    const d = parseInt(birthDate.day);
    const h = parseInt(birthDate.hour);
    const min = parseInt(birthDate.minute) || 0;
    
    const input = calendarType === 'solar' 
      ? { Solar: { year: y, month: m, day: d, hour: h, minute: min } }
      : { Lunar: { year: y, month: m, day: d, hour: h, minute: min, is_leap_month: isLeapMonth } };
    
    const genderParam = gender === 'male' ? 'Male' : 'Female';
    const zishiModeParam = zishiMode === 'modern' ? 'Modern' : 'Traditional';
    const longitudeParam = useTrueSolarTime ? Math.round(parseFloat(longitude) * 100000) : undefined;
    
    const txResult = await createBaziChart({
      name: chartName || undefined,
      input,
      gender: genderParam,
      zishi_mode: zishiModeParam,
      longitude: longitudeParam,
    });
    
    if (txResult?.success) {
      console.log('八字命盘创建成功:', txResult);
    }
  };

  const handleReset = () => {
    setBirthDate({ year: '', month: '', day: '', hour: '', minute: '' });
    setGender(null);
    setResult(null);
    setChartName('');
    setIsLeapMonth(false);
  };

  const handleFindMaster = () => {
    router.push('/divination/masters/bazi' as any);
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>📅 八字命理</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.introCard}>
          <Text style={styles.introTitle}>八字命理</Text>
          <Text style={styles.introDesc}>
            四柱推命，知命改运。八字命理以出生年月日时的天干地支组成四柱八字，推算人生命运轨迹。
          </Text>
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>📅 日历类型</Text>
          <View style={styles.calendarTypeButtons}>
            <Pressable
              style={[styles.calendarTypeButton, calendarType === 'solar' && styles.calendarTypeButtonSelected]}
              onPress={() => setCalendarType('solar')}
            >
              <Text style={[styles.calendarTypeText, calendarType === 'solar' && styles.calendarTypeTextSelected]}>
                公历（阳历）
              </Text>
            </Pressable>
            <Pressable
              style={[styles.calendarTypeButton, calendarType === 'lunar' && styles.calendarTypeButtonSelected]}
              onPress={() => setCalendarType('lunar')}
            >
              <Text style={[styles.calendarTypeText, calendarType === 'lunar' && styles.calendarTypeTextSelected]}>
                农历（阴历）
              </Text>
            </Pressable>
          </View>
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>📅 出生时间</Text>
          <Text style={styles.cardHint}>
            {calendarType === 'solar' ? '请输入公历出生时间' : '请输入农历出生时间'}
          </Text>
          
          <View style={styles.dateInputs}>
            <View style={styles.dateInputWrapper}>
              <TextInput
                style={styles.dateInput}
                placeholder="1990"
                placeholderTextColor="#9ca3af"
                value={birthDate.year}
                onChangeText={(v) => setBirthDate({ ...birthDate, year: v.replace(/[^0-9]/g, '') })}
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
                value={birthDate.month}
                onChangeText={(v) => setBirthDate({ ...birthDate, month: v.replace(/[^0-9]/g, '') })}
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
                value={birthDate.day}
                onChangeText={(v) => setBirthDate({ ...birthDate, day: v.replace(/[^0-9]/g, '') })}
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
                value={birthDate.hour}
                onChangeText={(v) => setBirthDate({ ...birthDate, hour: v.replace(/[^0-9]/g, '') })}
                keyboardType="number-pad"
                maxLength={2}
              />
              <Text style={styles.dateLabel}>时</Text>
            </View>
            <View style={styles.dateInputWrapper}>
              <TextInput
                style={styles.dateInput}
                placeholder="00"
                placeholderTextColor="#9ca3af"
                value={birthDate.minute}
                onChangeText={(v) => setBirthDate({ ...birthDate, minute: v.replace(/[^0-9]/g, '') })}
                keyboardType="number-pad"
                maxLength={2}
              />
              <Text style={styles.dateLabel}>分</Text>
            </View>
          </View>

          {calendarType === 'lunar' && (
            <View style={styles.leapMonthRow}>
              <Text style={styles.leapMonthLabel}>闰月</Text>
              <Switch
                value={isLeapMonth}
                onValueChange={setIsLeapMonth}
                trackColor={{ false: '#e5e7eb', true: '#a78bfa' }}
                thumbColor={isLeapMonth ? '#6D28D9' : '#f4f3f4'}
              />
            </View>
          )}
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>👤 性别</Text>
          <Text style={styles.cardHint}>性别影响大运顺逆计算</Text>
          <View style={styles.genderButtons}>
            <Pressable
              style={[styles.genderButton, gender === 'male' && styles.genderButtonSelected]}
              onPress={() => setGender('male')}
            >
              <Text style={[styles.genderButtonText, gender === 'male' && styles.genderButtonTextSelected]}>
                ♂ 男
              </Text>
            </Pressable>
            <Pressable
              style={[styles.genderButton, gender === 'female' && styles.genderButtonSelected]}
              onPress={() => setGender('female')}
            >
              <Text style={[styles.genderButtonText, gender === 'female' && styles.genderButtonTextSelected]}>
                ♀ 女
              </Text>
            </Pressable>
          </View>
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>⚙️ 高级选项</Text>
          
          <View style={styles.optionRow}>
            <View style={styles.optionInfo}>
              <Text style={styles.optionLabel}>子时模式</Text>
              <Text style={styles.optionHint}>
                {zishiMode === 'modern' ? '现代派：23:00-23:59属当日' : '传统派：23:00-23:59属次日'}
              </Text>
            </View>
            <View style={styles.zishiModeButtons}>
              <Pressable
                style={[styles.zishiModeButton, zishiMode === 'modern' && styles.zishiModeButtonSelected]}
                onPress={() => setZishiMode('modern')}
              >
                <Text style={[styles.zishiModeText, zishiMode === 'modern' && styles.zishiModeTextSelected]}>现代</Text>
              </Pressable>
              <Pressable
                style={[styles.zishiModeButton, zishiMode === 'traditional' && styles.zishiModeButtonSelected]}
                onPress={() => setZishiMode('traditional')}
              >
                <Text style={[styles.zishiModeText, zishiMode === 'traditional' && styles.zishiModeTextSelected]}>传统</Text>
              </Pressable>
            </View>
          </View>

          <View style={styles.optionRow}>
            <View style={styles.optionInfo}>
              <Text style={styles.optionLabel}>真太阳时</Text>
              <Text style={styles.optionHint}>根据出生地经度修正时辰</Text>
            </View>
            <Switch
              value={useTrueSolarTime}
              onValueChange={setUseTrueSolarTime}
              trackColor={{ false: '#e5e7eb', true: '#a78bfa' }}
              thumbColor={useTrueSolarTime ? '#6D28D9' : '#f4f3f4'}
            />
          </View>

          {useTrueSolarTime && (
            <View style={styles.longitudeRow}>
              <Text style={styles.longitudeLabel}>出生地经度</Text>
              <View style={styles.longitudeInputWrapper}>
                <TextInput
                  style={styles.longitudeInput}
                  placeholder="116.40"
                  placeholderTextColor="#9ca3af"
                  value={longitude}
                  onChangeText={setLongitude}
                  keyboardType="decimal-pad"
                />
                <Text style={styles.longitudeUnit}>°E</Text>
              </View>
            </View>
          )}

          <View style={styles.optionRow}>
            <View style={styles.optionInfo}>
              <Text style={styles.optionLabel}>命盘名称</Text>
              <Text style={styles.optionHint}>可选，方便识别多个命盘</Text>
            </View>
          </View>
          <TextInput
            style={styles.nameInput}
            placeholder="如：本人、父亲、母亲..."
            placeholderTextColor="#9ca3af"
            value={chartName}
            onChangeText={setChartName}
            maxLength={32}
          />
        </View>

        {!result ? (
          <Pressable 
            style={[styles.calculateButton, isLoading && styles.buttonDisabled]} 
            onPress={handleCalculate}
            disabled={isLoading}
          >
            <Text style={styles.calculateButtonText}>
              {isLoading ? '排盘中...' : '🔮 排盘分析'}
            </Text>
          </Pressable>
        ) : (
          <View style={styles.resultCard}>
            <Text style={styles.resultTitle}>八字排盘</Text>
            
            <View style={styles.pillarsDisplay}>
              {[
                { label: '年柱', value: result.yearPillar },
                { label: '月柱', value: result.monthPillar },
                { label: '日柱', value: result.dayPillar },
                { label: '时柱', value: result.hourPillar },
              ].map((pillar, index) => (
                <View key={index} style={styles.pillarBox}>
                  <Text style={styles.pillarLabel}>{pillar.label}</Text>
                  <Text style={styles.pillarGan}>{pillar.value[0]}</Text>
                  <Text style={styles.pillarZhi}>{pillar.value[1]}</Text>
                </View>
              ))}
            </View>

            <Text style={styles.resultHint}>
              💡 八字命盘需要专业大师结合大运流年详细解读
            </Text>

            <View style={styles.resultActions}>
              <Pressable style={styles.resetButton} onPress={handleReset}>
                <Text style={styles.resetButtonText}>重新排盘</Text>
              </Pressable>
              <Pressable 
                style={[styles.saveButton, isTxLoading && styles.buttonDisabled]} 
                onPress={handleSaveToChain}
                disabled={isTxLoading}
              >
                <Text style={styles.saveButtonText}>
                  {isTxLoading ? (txStatus === 'signing' ? '签名中...' : txStatus === 'broadcasting' ? '广播中...' : '保存中...') : '💾 保存到链'}
                </Text>
              </Pressable>
            </View>

            <Pressable style={styles.findMasterButton} onPress={handleFindMaster}>
              <Text style={styles.findMasterButtonText}>🔍 找大师解盘</Text>
            </Pressable>
          </View>
        )}

        <View style={styles.tipsCard}>
          <Text style={styles.tipsTitle}>💡 排盘须知</Text>
          <Text style={styles.tipsText}>
            • 公历/农历：系统会自动转换计算{'\n'}
            • 出生时辰：尽量精确到分钟{'\n'}
            • 子时模式：影响23:00后的日柱判断{'\n'}
            • 真太阳时：西部地区建议开启{'\n'}
            • 保存到链：永久存储，可用于合婚匹配
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
    marginBottom: 8,
  },
  cardHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 16,
  },
  calendarTypeButtons: {
    flexDirection: 'row',
    gap: 8,
  },
  calendarTypeButton: {
    flex: 1,
    padding: 12,
    borderRadius: 10,
    backgroundColor: '#f9fafb',
    alignItems: 'center',
  },
  calendarTypeButtonSelected: {
    backgroundColor: '#6D28D9',
  },
  calendarTypeText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#6b7280',
  },
  calendarTypeTextSelected: {
    color: '#fff',
  },
  dateInputs: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  dateInputWrapper: {
    alignItems: 'center',
    flex: 1,
    marginHorizontal: 2,
  },
  dateInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 10,
    padding: 10,
    fontSize: 16,
    fontWeight: '600',
    color: '#6D28D9',
    textAlign: 'center',
    width: '100%',
  },
  dateLabel: {
    fontSize: 11,
    color: '#9ca3af',
    marginTop: 4,
  },
  leapMonthRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginTop: 16,
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#f3f4f6',
  },
  leapMonthLabel: {
    fontSize: 14,
    color: '#4b5563',
  },
  genderButtons: {
    flexDirection: 'row',
    gap: 8,
  },
  genderButton: {
    flex: 1,
    padding: 16,
    borderRadius: 12,
    backgroundColor: '#f9fafb',
    alignItems: 'center',
  },
  genderButtonSelected: {
    backgroundColor: '#6D28D9',
  },
  genderButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#6b7280',
  },
  genderButtonTextSelected: {
    color: '#fff',
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
  zishiModeButtons: {
    flexDirection: 'row',
    backgroundColor: '#f3f4f6',
    borderRadius: 8,
    padding: 2,
  },
  zishiModeButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 6,
  },
  zishiModeButtonSelected: {
    backgroundColor: '#6D28D9',
  },
  zishiModeText: {
    fontSize: 13,
    color: '#6b7280',
  },
  zishiModeTextSelected: {
    color: '#fff',
  },
  longitudeRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  longitudeLabel: {
    fontSize: 14,
    color: '#4b5563',
  },
  longitudeInputWrapper: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  longitudeInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 8,
    fontSize: 14,
    color: '#6D28D9',
    width: 80,
    textAlign: 'center',
  },
  longitudeUnit: {
    fontSize: 14,
    color: '#6b7280',
    marginLeft: 4,
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
    backgroundColor: '#6D28D9',
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
    marginBottom: 20,
  },
  pillarsDisplay: {
    flexDirection: 'row',
    justifyContent: 'center',
    marginBottom: 20,
  },
  pillarBox: {
    alignItems: 'center',
    marginHorizontal: 6,
    padding: 12,
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    minWidth: 65,
  },
  pillarLabel: {
    fontSize: 11,
    color: '#9ca3af',
    marginBottom: 6,
  },
  pillarGan: {
    fontSize: 26,
    fontWeight: 'bold',
    color: '#dc2626',
    marginBottom: 2,
  },
  pillarZhi: {
    fontSize: 26,
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
    borderColor: '#6D28D9',
    alignItems: 'center',
  },
  resetButtonText: {
    color: '#6D28D9',
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
    backgroundColor: '#6D28D9',
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
