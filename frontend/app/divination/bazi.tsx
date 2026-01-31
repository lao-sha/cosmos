import { useTransaction } from '@/src/hooks/useTransaction';
import { chainService, FullBaziChartForApi } from '@/src/services/chain';
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
    year: '1990',
    month: '8',
    day: '15',
    hour: '14',
    minute: '30',
  });
  const [isLeapMonth, setIsLeapMonth] = useState(false);
  const [gender, setGender] = useState<Gender | null>(null);
  const [zishiMode, setZishiMode] = useState<ZiShiMode>('modern');
  const [chartName, setChartName] = useState('');
  const [useTrueSolarTime, setUseTrueSolarTime] = useState(false);
  const [longitude, setLongitude] = useState('116.40');
  
  const [result, setResult] = useState<BaziResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isFreeMode, setIsFreeMode] = useState(false); // 是否为免费排盘模式
  const [chainResult, setChainResult] = useState<{
    chartId?: string;
    blockHash?: string;
    txHash?: string;
    birthTime?: any;
  } | null>(null);
  const [fullChartData, setFullChartData] = useState<FullBaziChartForApi | null>(null);
  const [isQueryingChart, setIsQueryingChart] = useState(false);

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
    if (!gender) {
      showAlert('提示', '请选择性别');
      return;
    }
    
    setIsLoading(true);
    setFullChartData(null);
    setChainResult(null);
    
    try {
      // 检查是否连接到链
      if (isConnected) {
        // 使用链端 Runtime API 进行临时排盘（免费、不存储）
        const y = parseInt(birthDate.year);
        const m = parseInt(birthDate.month);
        const d = parseInt(birthDate.day);
        const h = parseInt(birthDate.hour);
        const min = parseInt(birthDate.minute) || 0;
        
        const chainResult = await chainService.calculateBaziTemp(
          calendarType,
          {
            year: y,
            month: m,
            day: d,
            hour: h,
            minute: min,
            isLeapMonth: calendarType === 'lunar' ? isLeapMonth : undefined,
            longitude: useTrueSolarTime ? parseFloat(longitude) : undefined,
          },
          gender,
          zishiMode
        );
        
        if (chainResult) {
          // 使用链端返回的真实数据
          setFullChartData(chainResult);
          setResult({
            yearPillar: `${chainResult.sizhu?.yearZhu?.ganzhi?.gan || ''}${chainResult.sizhu?.yearZhu?.ganzhi?.zhi || ''}`,
            monthPillar: `${chainResult.sizhu?.monthZhu?.ganzhi?.gan || ''}${chainResult.sizhu?.monthZhu?.ganzhi?.zhi || ''}`,
            dayPillar: `${chainResult.sizhu?.dayZhu?.ganzhi?.gan || ''}${chainResult.sizhu?.dayZhu?.ganzhi?.zhi || ''}`,
            hourPillar: `${chainResult.sizhu?.hourZhu?.ganzhi?.gan || ''}${chainResult.sizhu?.hourZhu?.ganzhi?.zhi || ''}`,
          });
          console.log('链端临时排盘成功:', chainResult);
        } else {
          // 链端返回空，回退到本地计算
          console.warn('链端临时排盘返回空，使用本地计算');
          const localResult = calculateLocalBazi();
          setResult(localResult);
        }
      } else {
        // 未连接链，使用本地计算
        console.log('未连接区块链，使用本地模拟计算');
        const localResult = calculateLocalBazi();
        setResult(localResult);
      }
      setIsFreeMode(true); // 标记为免费排盘模式
    } catch (error) {
      console.error('排盘失败:', error);
      // 出错时回退到本地计算
      try {
        const localResult = calculateLocalBazi();
        setResult(localResult);
        setIsFreeMode(true); // 标记为免费排盘模式
        console.log('链端排盘失败，已回退到本地计算');
      } catch (localError) {
        showAlert('错误', '排盘计算失败，请重试');
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleCalculateAndSave = async () => {
    if (!validateInput()) return;
    
    if (!gender) {
      showAlert('提示', '请选择性别');
      return;
    }
    if (!isLoggedIn || !address) {
      showAlert('提示', '请先登录钱包才能保存到链上');
      return;
    }
    if (!isConnected) {
      showAlert('提示', '请先连接区块链网络');
      return;
    }
    
    setIsLoading(true);
    setFullChartData(null);
    setChainResult(null);
    
    try {
      // 1. 先本地计算排盘（用于快速显示）
      const localResult = calculateLocalBazi();
      setResult(localResult);
      
      // 2. 准备链上参数
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
      
      console.log('调用链端 createBaziChart 交易:', { input, gender: genderParam, zishi_mode: zishiModeParam, longitude: longitudeParam });
      
      // 3. 调用链端保存
      const txResult = await createBaziChart({
        name: chartName || undefined,
        input,
        gender: genderParam,
        zishi_mode: zishiModeParam,
        longitude: longitudeParam,
      });
      
      console.log('链端交易结果:', txResult);
      
      if (txResult?.success) {
        // 解析链端返回的事件
        const baziEvent = txResult.events?.find(
          (e: any) => e.event?.method === 'BaziChartCreated'
        );
        
        let chartId = '';
        let birthTime = null;
        
        if (baziEvent?.event?.data) {
          const data = baziEvent.event.data;
          chartId = data.chartId || data.chart_id || data[1] || '';
          birthTime = data.birthTime || data.birth_time || data[2] || null;
        }
        
        console.log('解析事件数据:', { chartId, birthTime, baziEvent });
        
        setChainResult({
          chartId: chartId?.toString(),
          blockHash: txResult.blockHash,
          txHash: txResult.txHash,
          birthTime,
        });
        
        // 查询链端返回的完整命盘数据
        if (chartId) {
          setIsQueryingChart(true);
          try {
            const fullChart = await chainService.getBaziChart(parseInt(chartId.toString()));
            if (fullChart) {
              setFullChartData(fullChart);
              console.log('链端返回的完整命盘数据:', JSON.stringify(fullChart, null, 2));
            }
          } catch (queryError) {
            console.error('查询命盘数据失败:', queryError);
          } finally {
            setIsQueryingChart(false);
          }
        }
        
        setIsFreeMode(false); // 标记为链上保存模式
        showAlert('保存成功', `八字命盘已保存到区块链\n命盘ID: ${chartId}\n区块: ${txResult.blockHash?.slice(0, 16)}...`);
      } else {
        // 交易失败
        console.error('链端交易失败:', txResult?.error);
        showAlert('保存失败', txResult?.error || '交易执行失败，请检查钱包余额或网络连接');
      }
    } catch (error: any) {
      console.error('排盘保存失败:', error);
      showAlert('错误', error?.message || '排盘保存失败，请重试');
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
      privacy_mode: 'PublicEncrypted',
    });
    
    if (txResult?.success) {
      showAlert('保存成功', '八字命盘已加密保存到区块链');
    }
  };

  const handleReset = () => {
    setBirthDate({ year: '1990', month: '8', day: '15', hour: '14', minute: '30' });
    setGender(null);
    setResult(null);
    setChartName('');
    setIsLeapMonth(false);
    setChainResult(null);
    setIsFreeMode(false);
    setFullChartData(null);
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
          <View style={styles.actionButtons}>
            <Pressable 
              style={[styles.freeCalculateButton, isLoading && styles.buttonDisabled]} 
              onPress={handleCalculate}
              disabled={isLoading}
            >
              <Text style={styles.freeCalculateButtonText}>
                {isLoading ? '排盘中...' : '🔮 免费排盘'}
              </Text>
              <Text style={styles.freeCalculateHint}>不保存 · 无需GAS</Text>
            </Pressable>
            <Pressable 
              style={[styles.chainCalculateButton, (isLoading || isTxLoading) && styles.buttonDisabled]} 
              onPress={handleCalculateAndSave}
              disabled={isLoading || isTxLoading}
            >
              <Text style={styles.chainCalculateButtonText}>
                {isTxLoading ? '保存中...' : '💾 排盘并保存'}
              </Text>
              <Text style={styles.chainCalculateHint}>链上存储 · 加密保护</Text>
            </Pressable>
          </View>
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

            {/* 免费排盘模式下不显示操作按钮 */}
            {!isFreeMode && (
              <>
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
              </>
            )}

            {chainResult && (
              <View style={styles.chainResultCard}>
                <Text style={styles.chainResultTitle}>⛓️ 链上存储信息</Text>
                <View style={styles.chainResultRow}>
                  <Text style={styles.chainResultLabel}>命盘 ID:</Text>
                  <Text style={styles.chainResultValue}>{chainResult.chartId || '-'}</Text>
                </View>
                <View style={styles.chainResultRow}>
                  <Text style={styles.chainResultLabel}>区块哈希:</Text>
                  <Text style={styles.chainResultValue} numberOfLines={1}>
                    {chainResult.blockHash ? `${chainResult.blockHash.slice(0, 20)}...` : '-'}
                  </Text>
                </View>
                <View style={styles.chainResultRow}>
                  <Text style={styles.chainResultLabel}>交易哈希:</Text>
                  <Text style={styles.chainResultValue} numberOfLines={1}>
                    {chainResult.txHash ? `${chainResult.txHash.slice(0, 20)}...` : '-'}
                  </Text>
                </View>
                <View style={styles.chainResultRow}>
                  <Text style={styles.chainResultLabel}>隐私模式:</Text>
                  <Text style={[styles.chainResultValue, { color: '#22c55e' }]}>PublicEncrypted</Text>
                </View>
                <Text style={styles.chainResultHint}>
                  ✅ 数据已加密存储，只有您可以解密查看完整结果
                </Text>
              </View>
            )}

            {isQueryingChart && (
              <View style={styles.queryingCard}>
                <Text style={styles.queryingText}>🔄 正在查询链端命盘数据...</Text>
              </View>
            )}

            {fullChartData && (
              <View style={styles.fullChartCard}>
                <Text style={styles.fullChartTitle}>📊 链端返回的完整命盘数据 (FullBaziChartForApi)</Text>
                
                {/* 1. 基本信息 */}
                <View style={styles.fullChartSection}>
                  <Text style={styles.fullChartSectionTitle}>1️⃣ 基本信息</Text>
                  <View style={styles.fullChartRow}>
                    <Text style={styles.fullChartLabel}>性别 (gender):</Text>
                    <Text style={styles.fullChartValue}>{fullChartData.gender}</Text>
                  </View>
                  <View style={styles.fullChartRow}>
                    <Text style={styles.fullChartLabel}>出生年份 (birthYear):</Text>
                    <Text style={styles.fullChartValue}>{fullChartData.birthYear}</Text>
                  </View>
                  <View style={styles.fullChartRow}>
                    <Text style={styles.fullChartLabel}>日历类型 (inputCalendarType):</Text>
                    <Text style={styles.fullChartValue}>{fullChartData.inputCalendarType}</Text>
                  </View>
                </View>

                {/* 2. 四柱信息 (sizhu) */}
                <View style={styles.fullChartSection}>
                  <Text style={styles.fullChartSectionTitle}>2️⃣ 四柱信息 (sizhu: SiZhuForApi)</Text>
                  <View style={styles.fullChartRow}>
                    <Text style={styles.fullChartLabel}>日主 (rizhu):</Text>
                    <Text style={[styles.fullChartValue, { color: '#dc2626', fontWeight: 'bold', fontSize: 16 }]}>
                      {fullChartData.sizhu?.rizhu}
                    </Text>
                  </View>
                  
                  {/* 四柱详细卡片 */}
                  <View style={styles.siZhuGrid}>
                    {['yearZhu', 'monthZhu', 'dayZhu', 'hourZhu'].map((zhuKey, idx) => {
                      const zhu = (fullChartData.sizhu as any)?.[zhuKey];
                      const labels = ['年柱', '月柱', '日柱', '时柱'];
                      const colors = ['#dc2626', '#f59e0b', '#22c55e', '#3b82f6'];
                      return (
                        <View key={zhuKey} style={[styles.zhuCardFull, { borderLeftColor: colors[idx] }]}>
                          <Text style={[styles.zhuLabelFull, { color: colors[idx] }]}>{labels[idx]} ({zhuKey})</Text>
                          <Text style={styles.zhuGanzhiFull}>
                            <Text style={{ color: '#dc2626' }}>{zhu?.ganzhi?.gan}</Text>
                            <Text style={{ color: '#2563eb' }}>{zhu?.ganzhi?.zhi}</Text>
                          </Text>
                          <View style={styles.zhuDetailRow}>
                            <Text style={styles.zhuDetailLabel}>天干十神:</Text>
                            <Text style={styles.zhuDetailValue}>{zhu?.tianganShishen}</Text>
                          </View>
                          <View style={styles.zhuDetailRow}>
                            <Text style={styles.zhuDetailLabel}>地支本气:</Text>
                            <Text style={styles.zhuDetailValue}>{zhu?.dizhiBenqiShishen}</Text>
                          </View>
                          <View style={styles.zhuDetailRow}>
                            <Text style={styles.zhuDetailLabel}>自坐:</Text>
                            <Text style={styles.zhuDetailValue}>{zhu?.zizuo}</Text>
                          </View>
                          <View style={styles.zhuDetailRow}>
                            <Text style={styles.zhuDetailLabel}>纳音:</Text>
                            <Text style={styles.zhuDetailValue}>{zhu?.nayin}</Text>
                          </View>
                          <View style={styles.zhuDetailRow}>
                            <Text style={styles.zhuDetailLabel}>十二长生:</Text>
                            <Text style={styles.zhuDetailValue}>{zhu?.changsheng}</Text>
                          </View>
                          
                          {/* 藏干列表 */}
                          {zhu?.cangganList && zhu.cangganList.length > 0 && (
                            <View style={styles.cangganSection}>
                              <Text style={styles.cangganTitle}>藏干 (cangganList):</Text>
                              {zhu.cangganList.map((cg: any, cgIdx: number) => (
                                <View key={cgIdx} style={styles.cangganItem}>
                                  <Text style={styles.cangganGan}>{cg.gan}</Text>
                                  <Text style={styles.cangganInfo}>
                                    {cg.shishen} · {cg.cangganType} · 权重{cg.weight}
                                  </Text>
                                </View>
                              ))}
                            </View>
                          )}
                        </View>
                      );
                    })}
                  </View>
                </View>

                {/* 3. 命盘分析 (analysis) */}
                <View style={styles.fullChartSection}>
                  <Text style={styles.fullChartSectionTitle}>3️⃣ 命盘分析 (analysis: AnalysisForApi)</Text>
                  <View style={styles.analysisGrid}>
                    <View style={styles.analysisItem}>
                      <Text style={styles.analysisLabel}>格局 (geJu)</Text>
                      <Text style={styles.analysisValue}>{fullChartData.analysis?.geJu}</Text>
                    </View>
                    <View style={styles.analysisItem}>
                      <Text style={styles.analysisLabel}>强弱 (qiangRuo)</Text>
                      <Text style={styles.analysisValue}>{fullChartData.analysis?.qiangRuo}</Text>
                    </View>
                    <View style={[styles.analysisItem, { backgroundColor: '#dcfce7' }]}>
                      <Text style={styles.analysisLabel}>用神 (yongShen)</Text>
                      <Text style={[styles.analysisValue, { color: '#16a34a' }]}>{fullChartData.analysis?.yongShen}</Text>
                    </View>
                    <View style={styles.analysisItem}>
                      <Text style={styles.analysisLabel}>用神类型 (yongShenType)</Text>
                      <Text style={styles.analysisValue}>{fullChartData.analysis?.yongShenType}</Text>
                    </View>
                    <View style={[styles.analysisItem, { backgroundColor: '#dbeafe' }]}>
                      <Text style={styles.analysisLabel}>喜神 (xiShen)</Text>
                      <Text style={[styles.analysisValue, { color: '#2563eb' }]}>{fullChartData.analysis?.xiShen}</Text>
                    </View>
                    <View style={[styles.analysisItem, { backgroundColor: '#fee2e2' }]}>
                      <Text style={styles.analysisLabel}>忌神 (jiShen)</Text>
                      <Text style={[styles.analysisValue, { color: '#dc2626' }]}>{fullChartData.analysis?.jiShen}</Text>
                    </View>
                  </View>
                  <View style={styles.scoreCard}>
                    <Text style={styles.scoreLabel}>综合评分 (score)</Text>
                    <Text style={styles.scoreValue}>{fullChartData.analysis?.score}/100</Text>
                  </View>
                </View>

                {/* 4. 空亡信息 (kongwang) */}
                <View style={styles.fullChartSection}>
                  <Text style={styles.fullChartSectionTitle}>4️⃣ 空亡信息 (kongwang: KongWangInfo)</Text>
                  <View style={styles.kongwangCard}>
                    <Text style={styles.kongwangText}>
                      {fullChartData.kongwang ? JSON.stringify(fullChartData.kongwang, null, 2) : '无数据'}
                    </Text>
                  </View>
                </View>

                {/* 5. 星运信息 (xingyun) */}
                <View style={styles.fullChartSection}>
                  <Text style={styles.fullChartSectionTitle}>5️⃣ 星运信息 (xingyun: XingYunInfo)</Text>
                  <View style={styles.xingyunCard}>
                    <Text style={styles.xingyunText}>
                      {fullChartData.xingyun ? JSON.stringify(fullChartData.xingyun, null, 2) : '无数据'}
                    </Text>
                  </View>
                </View>

                {/* 6. 神煞列表 (shenshaList) */}
                <View style={styles.fullChartSection}>
                  <Text style={styles.fullChartSectionTitle}>6️⃣ 神煞列表 (shenshaList: ShenShaEntry[])</Text>
                  {fullChartData.shenshaList && fullChartData.shenshaList.length > 0 ? (
                    <View style={styles.shenshaContainer}>
                      {fullChartData.shenshaList.map((shensha: any, idx: number) => (
                        <View key={idx} style={styles.shenshaItem}>
                          <Text style={styles.shenshaName}>
                            {typeof shensha === 'string' ? shensha : JSON.stringify(shensha)}
                          </Text>
                        </View>
                      ))}
                    </View>
                  ) : (
                    <Text style={styles.noDataText}>无神煞数据</Text>
                  )}
                </View>

                {/* 7. 五行强度 (wuxingStrength) */}
                <View style={styles.fullChartSection}>
                  <Text style={styles.fullChartSectionTitle}>7️⃣ 五行强度 (wuxingStrength: WuXingStrength)</Text>
                  <View style={styles.wuxingCard}>
                    <Text style={styles.wuxingText}>
                      {JSON.stringify(fullChartData.wuxingStrength, null, 2)}
                    </Text>
                  </View>
                </View>

                {/* 8. 起运信息 (qiyun) */}
                <View style={styles.fullChartSection}>
                  <Text style={styles.fullChartSectionTitle}>8️⃣ 起运信息 (qiyun: QiYunForApi)</Text>
                  <View style={styles.qiyunGrid}>
                    <View style={styles.qiyunItem}>
                      <Text style={styles.qiyunLabel}>起运年龄</Text>
                      <Text style={styles.qiyunValue}>
                        {fullChartData.qiyun?.ageYears}岁{fullChartData.qiyun?.ageMonths}月{fullChartData.qiyun?.ageDays}天
                      </Text>
                    </View>
                    <View style={styles.qiyunItem}>
                      <Text style={styles.qiyunLabel}>大运方向 (isShun)</Text>
                      <Text style={[styles.qiyunValue, { color: fullChartData.qiyun?.isShun ? '#22c55e' : '#ef4444' }]}>
                        {fullChartData.qiyun?.isShun ? '顺排 ↑' : '逆排 ↓'}
                      </Text>
                    </View>
                    <View style={styles.qiyunItem}>
                      <Text style={styles.qiyunLabel}>交运时间</Text>
                      <Text style={styles.qiyunValue}>
                        {fullChartData.qiyun?.jiaoyunYear}年{fullChartData.qiyun?.jiaoyunMonth}月{fullChartData.qiyun?.jiaoyunDay}日
                      </Text>
                    </View>
                  </View>
                </View>

                {/* 9. 大运列表 (dayunList) */}
                {fullChartData.dayunList && fullChartData.dayunList.length > 0 && (
                  <View style={styles.fullChartSection}>
                    <Text style={styles.fullChartSectionTitle}>
                      9️⃣ 大运列表 (dayunList: DaYunForApi[]) - 共{fullChartData.dayunList.length}步
                    </Text>
                    <ScrollView horizontal showsHorizontalScrollIndicator={false}>
                      <View style={styles.dayunContainer}>
                        {fullChartData.dayunList.map((dayun, idx) => (
                          <View key={idx} style={styles.dayunCardFull}>
                            <Text style={styles.dayunIndex}>第{idx + 1}运</Text>
                            <Text style={styles.dayunGanzhiFull}>
                              <Text style={{ color: '#dc2626' }}>{dayun.ganzhi?.gan}</Text>
                              <Text style={{ color: '#2563eb' }}>{dayun.ganzhi?.zhi}</Text>
                            </Text>
                            <Text style={styles.dayunAgeFull}>{dayun.startAge}-{dayun.endAge}岁</Text>
                            <Text style={styles.dayunYearFull}>{dayun.startYear}-{dayun.endYear}年</Text>
                            <View style={styles.dayunDetailRow}>
                              <Text style={styles.dayunDetailLabel}>天干十神:</Text>
                              <Text style={styles.dayunDetailValue}>{dayun.tianganShishen}</Text>
                            </View>
                            <View style={styles.dayunDetailRow}>
                              <Text style={styles.dayunDetailLabel}>地支本气:</Text>
                              <Text style={styles.dayunDetailValue}>{dayun.dizhiBenqiShishen}</Text>
                            </View>
                            <View style={styles.dayunDetailRow}>
                              <Text style={styles.dayunDetailLabel}>十二长生:</Text>
                              <Text style={styles.dayunDetailValue}>{dayun.changsheng}</Text>
                            </View>
                            <Text style={styles.dayunLiunianCount}>
                              流年: {dayun.liunianList?.length || 0}年
                            </Text>
                          </View>
                        ))}
                      </View>
                    </ScrollView>
                  </View>
                )}

                {/* 10. 原始 JSON 数据 */}
                <View style={styles.fullChartSection}>
                  <Text style={styles.fullChartSectionTitle}>🔟 原始 JSON 数据</Text>
                  <ScrollView style={styles.jsonScrollView} nestedScrollEnabled>
                    <Text style={styles.jsonText}>
                      {JSON.stringify(fullChartData, null, 2)}
                    </Text>
                  </ScrollView>
                </View>
              </View>
            )}
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
  actionButtons: {
    flexDirection: 'row',
    marginHorizontal: 16,
    marginVertical: 16,
    gap: 12,
  },
  freeCalculateButton: {
    flex: 1,
    backgroundColor: '#f3f4f6',
    padding: 16,
    borderRadius: 16,
    alignItems: 'center',
    borderWidth: 2,
    borderColor: '#6D28D9',
  },
  freeCalculateButtonText: {
    color: '#6D28D9',
    fontSize: 16,
    fontWeight: '600',
  },
  freeCalculateHint: {
    color: '#9ca3af',
    fontSize: 11,
    marginTop: 4,
  },
  chainCalculateButton: {
    flex: 1,
    backgroundColor: '#6D28D9',
    padding: 16,
    borderRadius: 16,
    alignItems: 'center',
  },
  chainCalculateButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  chainCalculateHint: {
    color: 'rgba(255,255,255,0.7)',
    fontSize: 11,
    marginTop: 4,
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
  chainResultCard: {
    marginTop: 16,
    padding: 16,
    backgroundColor: '#f0fdf4',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#86efac',
  },
  chainResultTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#166534',
    marginBottom: 12,
  },
  chainResultRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 6,
    borderBottomWidth: 1,
    borderBottomColor: '#dcfce7',
  },
  chainResultLabel: {
    fontSize: 13,
    color: '#4b5563',
  },
  chainResultValue: {
    fontSize: 13,
    fontWeight: '500',
    color: '#1f2937',
    flex: 1,
    textAlign: 'right',
    marginLeft: 8,
  },
  chainResultHint: {
    fontSize: 12,
    color: '#16a34a',
    marginTop: 12,
    textAlign: 'center',
  },
  queryingCard: {
    marginTop: 16,
    padding: 16,
    backgroundColor: '#fef3c7',
    borderRadius: 12,
    alignItems: 'center',
  },
  queryingText: {
    fontSize: 14,
    color: '#92400e',
  },
  fullChartCard: {
    marginTop: 16,
    padding: 16,
    backgroundColor: '#eff6ff',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#93c5fd',
  },
  fullChartTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1e40af',
    marginBottom: 16,
  },
  fullChartSection: {
    marginBottom: 16,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#dbeafe',
  },
  fullChartSectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#3b82f6',
    marginBottom: 8,
  },
  fullChartRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 4,
  },
  fullChartLabel: {
    fontSize: 13,
    color: '#6b7280',
  },
  fullChartValue: {
    fontSize: 13,
    fontWeight: '500',
    color: '#1f2937',
  },
  zhuCard: {
    backgroundColor: '#fff',
    padding: 10,
    borderRadius: 8,
    marginVertical: 4,
  },
  zhuLabel: {
    fontSize: 12,
    color: '#6b7280',
    marginBottom: 4,
  },
  zhuGanzhi: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 4,
  },
  zhuDetail: {
    fontSize: 11,
    color: '#9ca3af',
  },
  siZhuGrid: {
    marginTop: 8,
  },
  zhuCardFull: {
    backgroundColor: '#fff',
    padding: 12,
    borderRadius: 8,
    marginVertical: 6,
    borderLeftWidth: 4,
  },
  zhuLabelFull: {
    fontSize: 13,
    fontWeight: '600',
    marginBottom: 6,
  },
  zhuGanzhiFull: {
    fontSize: 28,
    fontWeight: 'bold',
    marginBottom: 8,
  },
  zhuDetailRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 2,
  },
  zhuDetailLabel: {
    fontSize: 11,
    color: '#6b7280',
  },
  zhuDetailValue: {
    fontSize: 11,
    color: '#1f2937',
    fontWeight: '500',
  },
  cangganSection: {
    marginTop: 8,
    paddingTop: 8,
    borderTopWidth: 1,
    borderTopColor: '#e5e7eb',
  },
  cangganTitle: {
    fontSize: 11,
    color: '#6b7280',
    marginBottom: 4,
  },
  cangganItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 2,
  },
  cangganGan: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#dc2626',
    marginRight: 8,
    width: 20,
  },
  cangganInfo: {
    fontSize: 10,
    color: '#9ca3af',
  },
  analysisGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  analysisItem: {
    backgroundColor: '#fff',
    padding: 10,
    borderRadius: 8,
    width: '48%',
  },
  analysisLabel: {
    fontSize: 10,
    color: '#6b7280',
    marginBottom: 4,
  },
  analysisValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
  },
  scoreCard: {
    marginTop: 12,
    backgroundColor: '#fef3c7',
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  scoreLabel: {
    fontSize: 12,
    color: '#92400e',
    marginBottom: 4,
  },
  scoreValue: {
    fontSize: 32,
    fontWeight: 'bold',
    color: '#f59e0b',
  },
  kongwangCard: {
    backgroundColor: '#fff',
    padding: 12,
    borderRadius: 8,
  },
  kongwangText: {
    fontSize: 11,
    color: '#6b7280',
    fontFamily: 'monospace',
  },
  xingyunCard: {
    backgroundColor: '#fff',
    padding: 12,
    borderRadius: 8,
  },
  xingyunText: {
    fontSize: 11,
    color: '#6b7280',
    fontFamily: 'monospace',
  },
  shenshaContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 6,
  },
  shenshaItem: {
    backgroundColor: '#fff',
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 16,
    borderWidth: 1,
    borderColor: '#e5e7eb',
  },
  shenshaName: {
    fontSize: 11,
    color: '#4b5563',
  },
  noDataText: {
    fontSize: 12,
    color: '#9ca3af',
    fontStyle: 'italic',
  },
  wuxingCard: {
    backgroundColor: '#fff',
    padding: 12,
    borderRadius: 8,
  },
  qiyunGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  qiyunItem: {
    backgroundColor: '#fff',
    padding: 12,
    borderRadius: 8,
    flex: 1,
    minWidth: '30%',
  },
  qiyunLabel: {
    fontSize: 10,
    color: '#6b7280',
    marginBottom: 4,
  },
  qiyunValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
  },
  dayunCardFull: {
    backgroundColor: '#fff',
    padding: 12,
    borderRadius: 10,
    alignItems: 'center',
    minWidth: 100,
    marginRight: 8,
  },
  dayunIndex: {
    fontSize: 10,
    color: '#9ca3af',
    marginBottom: 4,
  },
  dayunGanzhiFull: {
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 4,
  },
  dayunAgeFull: {
    fontSize: 12,
    color: '#1f2937',
    fontWeight: '600',
  },
  dayunYearFull: {
    fontSize: 10,
    color: '#6b7280',
    marginBottom: 6,
  },
  dayunDetailRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    width: '100%',
    paddingVertical: 2,
  },
  dayunDetailLabel: {
    fontSize: 9,
    color: '#9ca3af',
  },
  dayunDetailValue: {
    fontSize: 9,
    color: '#4b5563',
  },
  dayunLiunianCount: {
    fontSize: 9,
    color: '#3b82f6',
    marginTop: 4,
  },
  jsonScrollView: {
    maxHeight: 300,
    backgroundColor: '#1f2937',
    borderRadius: 8,
    padding: 12,
  },
  jsonText: {
    fontSize: 10,
    color: '#a5f3fc',
    fontFamily: 'monospace',
  },
  dayunContainer: {
    flexDirection: 'row',
    gap: 8,
  },
  dayunCard: {
    backgroundColor: '#fff',
    padding: 10,
    borderRadius: 8,
    alignItems: 'center',
    minWidth: 70,
  },
  dayunGanzhi: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#1f2937',
  },
  dayunAge: {
    fontSize: 11,
    color: '#6b7280',
    marginTop: 4,
  },
  dayunYear: {
    fontSize: 10,
    color: '#9ca3af',
  },
  dayunShishen: {
    fontSize: 10,
    color: '#3b82f6',
    marginTop: 2,
  },
  wuxingText: {
    fontSize: 11,
    color: '#6b7280',
    fontFamily: 'monospace',
    backgroundColor: '#fff',
    padding: 8,
    borderRadius: 6,
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
