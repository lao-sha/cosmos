/**
 * 星尘玄鉴 - 八字排盘
 * 参考样式：专业八字排盘风格
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
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { useDivinationSave } from '@/hooks/useDivinationSave';
import { divinationService, DivinationType } from '@/services/divination.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';
import { getCurrentSignerAddress } from '@/lib/signer';
import { initializeApi, isApiInitialized } from '@/lib/api';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

// 天干
const TIAN_GAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
const TIAN_GAN_WUXING = ['木', '木', '火', '火', '土', '土', '金', '金', '水', '水'];

// 地支
const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];
const DI_ZHI_WUXING = ['水', '土', '木', '木', '土', '火', '火', '土', '金', '金', '土', '水'];
const SHENG_XIAO = ['鼠', '牛', '虎', '兔', '龙', '蛇', '马', '羊', '猴', '鸡', '狗', '猪'];

// 五行颜色
const WU_XING_COLORS: Record<string, string> = {
  '木': '#2E7D32',
  '火': '#C62828',
  '土': '#F57C00',
  '金': '#FDD835',
  '水': '#1565C0',
};

// 从干支字符串解析索引 (如 "甲子" -> { gan: 0, zhi: 0 })
const parseGanzhi = (ganzhi: string): { gan: number; zhi: number } => {
  if (!ganzhi || ganzhi.length < 2) return { gan: 0, zhi: 0 };
  const ganChar = ganzhi.charAt(0);
  const zhiChar = ganzhi.charAt(1);
  const gan = TIAN_GAN.indexOf(ganChar);
  const zhi = DI_ZHI.indexOf(zhiChar);
  return { gan: gan >= 0 ? gan : 0, zhi: zhi >= 0 ? zhi : 0 };
};

// 时辰选项
const SHICHEN_OPTIONS = [
  { value: 0, label: '子时 (23-1点)' },
  { value: 1, label: '丑时 (1-3点)' },
  { value: 2, label: '丑时 (1-3点)' },
  { value: 3, label: '寅时 (3-5点)' },
  { value: 4, label: '寅时 (3-5点)' },
  { value: 5, label: '卯时 (5-7点)' },
  { value: 6, label: '卯时 (5-7点)' },
  { value: 7, label: '辰时 (7-9点)' },
  { value: 8, label: '辰时 (7-9点)' },
  { value: 9, label: '巳时 (9-11点)' },
  { value: 10, label: '巳时 (9-11点)' },
  { value: 11, label: '午时 (11-13点)' },
  { value: 12, label: '午时 (11-13点)' },
  { value: 13, label: '未时 (13-15点)' },
  { value: 14, label: '未时 (13-15点)' },
  { value: 15, label: '申时 (15-17点)' },
  { value: 16, label: '申时 (15-17点)' },
  { value: 17, label: '酉时 (17-19点)' },
  { value: 18, label: '酉时 (17-19点)' },
  { value: 19, label: '戌时 (19-21点)' },
  { value: 20, label: '戌时 (19-21点)' },
  { value: 21, label: '亥时 (21-23点)' },
  { value: 22, label: '亥时 (21-23点)' },
  { value: 23, label: '子时 (23-1点)' },
];

// 性别类型
type Gender = 'male' | 'female';
type CalendarType = 'solar' | 'lunar';

// 八字结果（存储完整 API 返回数据）
interface BaziResult {
  id: number;
  name: string;
  birthYear: number;
  birthMonth: number;
  birthDay: number;
  birthHour: number;
  gender: Gender;
  siZhu: {
    year: { gan: number; zhi: number };
    month: { gan: number; zhi: number };
    day: { gan: number; zhi: number };
    hour: { gan: number; zhi: number };
  };
  wuxingCount: Record<string, number>;
  dayMaster: number;
  shengxiao: string;
  createdAt: Date;
  // 完整 API 数据
  chartData?: any;
}

export default function BaziPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<BaziResult | null>(null);
  const [history, setHistory] = useState<BaziResult[]>([]);

  // 表单状态
  const [name, setName] = useState('');
  const [gender, setGender] = useState<Gender>('male');
  const [calendarType, setCalendarType] = useState<CalendarType>('solar');
  const [birthYear, setBirthYear] = useState(1990);
  const [birthMonth, setBirthMonth] = useState(6);
  const [birthDay, setBirthDay] = useState(15);
  const [birthHour, setBirthHour] = useState(12);

  // 使用统一的上链保存 Hook
  const {
    showUnlockDialog,
    showTxStatus,
    txStatus,
    saving,
    saveToChain,
    saveBaziToChain,
    handleUnlockSuccess,
    setShowUnlockDialog,
    setShowTxStatus,
  } = useDivinationSave({
    divinationType: DivinationType.Bazi,
    historyRoute: '/divination/history',
  });

  const [apiReady, setApiReady] = useState(false);

  // 初始化 API
  useEffect(() => {
    const init = async () => {
      try {
        if (!isApiInitialized()) {
          await initializeApi();
        }
        setApiReady(true);
      } catch (error) {
        console.error('API 初始化失败:', error);
        Alert.alert('连接失败', '无法连接到区块链节点，请检查网络');
      }
    };
    init();
  }, []);

  // 免费试算（调用 Runtime API，不保存到链上）
  const calculateBaziTemp = async () => {
    if (!apiReady) {
      Alert.alert('请稍候', '正在连接区块链节点...');
      return;
    }
    setLoading(true);
    try {
      // 调用链端 Runtime API 进行免费计算
      const chartData = await divinationService.calculateBaziTemp(
        birthYear,
        birthMonth,
        birthDay,
        birthHour,
        0, // minute，暂时设为 0
        gender,
        calendarType
      );

      console.log('八字计算结果:', chartData);

      // 从链端返回的数据中提取四柱信息（解析 ganzhi 字符串）
      const siZhu = {
        year: parseGanzhi(chartData.sizhu.yearZhu.ganzhi),
        month: parseGanzhi(chartData.sizhu.monthZhu.ganzhi),
        day: parseGanzhi(chartData.sizhu.dayZhu.ganzhi),
        hour: parseGanzhi(chartData.sizhu.hourZhu.ganzhi),
      };

      // 五行统计
      const wuxingCount: Record<string, number> = { '木': 0, '火': 0, '土': 0, '金': 0, '水': 0 };
      Object.values(siZhu).forEach(zhu => {
        const ganWuxing = TIAN_GAN_WUXING[zhu.gan];
        const zhiWuxing = DI_ZHI_WUXING[zhu.zhi];
        if (ganWuxing) {
          wuxingCount[ganWuxing] = (wuxingCount[ganWuxing] || 0) + 1;
        }
        if (zhiWuxing) {
          wuxingCount[zhiWuxing] = (wuxingCount[zhiWuxing] || 0) + 1;
        }
      });

      const baziResult: BaziResult = {
        id: Date.now(),
        name: name || '求测者',
        birthYear,
        birthMonth,
        birthDay,
        birthHour,
        gender,
        siZhu,
        wuxingCount,
        dayMaster: siZhu.day.gan,
        shengxiao: SHENG_XIAO[siZhu.year.zhi] || '未知',
        createdAt: new Date(),
        chartData, // 保存完整 API 数据
      };

      setResult(baziResult);
      setHistory(prev => [baziResult, ...prev]);
    } catch (error: any) {
      console.error('免费试算失败:', error);
      Alert.alert('计算失败', error.message || '未知错误');
    } finally {
      setLoading(false);
    }
  };

  // 开始排盘（调用 Extrinsic，保存到链上）
  const handleCreateBaziChart = async () => {
    if (!apiReady) {
      Alert.alert('请稍候', '正在连接区块链节点...');
      return;
    }

    await saveBaziToChain({
      name: name || null,
      birthYear,
      birthMonth,
      birthDay,
      birthHour,
      birthMinute: 0,
      gender,
      calendarType,
    });

    // 注意：saveBaziToChain 成功后会跳转或显示提示
    // 如果需要更新本地结果状态，可以在这里添加逻辑
  };

  const handleReset = () => {
    setResult(null);
  };

  // 保存到链上 (保存计算结果)
  const handleSaveToChain = async () => {
    if (!result) {
      Alert.alert('提示', '请先进行八字排盘');
      return;
    }
    await saveToChain(result);
  };

  // 渲染四柱
  const renderSiZhu = () => {
    if (!result) return null;

    const pillars = [
      { label: '年柱', data: result.siZhu.year },
      { label: '月柱', data: result.siZhu.month },
      { label: '日柱', data: result.siZhu.day, isDay: true },
      { label: '时柱', data: result.siZhu.hour },
    ];

    return (
      <View style={styles.siZhuCard}>
        <Text style={styles.cardTitle}>四柱八字</Text>
        <View style={styles.siZhuContainer}>
          {pillars.map((pillar, index) => (
            <View key={index} style={styles.zhuColumn}>
              <Text style={styles.zhuLabel}>{pillar.label}</Text>
              <View style={[
                styles.ganBox,
                { borderColor: WU_XING_COLORS[TIAN_GAN_WUXING[pillar.data.gan] || '木'] || '#999' }
              ]}>
                <Text style={[
                  styles.ganText,
                  { color: WU_XING_COLORS[TIAN_GAN_WUXING[pillar.data.gan] || '木'] || '#999' }
                ]}>
                  {TIAN_GAN[pillar.data.gan] || '?'}
                </Text>
                <Text style={styles.wuxingLabel}>{TIAN_GAN_WUXING[pillar.data.gan] || '?'}</Text>
              </View>
              <View style={[
                styles.zhiBox,
                { borderColor: WU_XING_COLORS[DI_ZHI_WUXING[pillar.data.zhi] || '水'] || '#999' }
              ]}>
                <Text style={[
                  styles.zhiText,
                  { color: WU_XING_COLORS[DI_ZHI_WUXING[pillar.data.zhi] || '水'] || '#999' }
                ]}>
                  {DI_ZHI[pillar.data.zhi] || '?'}
                </Text>
                <Text style={styles.wuxingLabel}>{DI_ZHI_WUXING[pillar.data.zhi] || '?'}</Text>
              </View>
              {pillar.isDay && (
                <View style={styles.dayMasterTag}>
                  <Text style={styles.dayMasterTagText}>日主</Text>
                </View>
              )}
            </View>
          ))}
        </View>
      </View>
    );
  };

  // 渲染五行分布
  const renderWuXing = () => {
    if (!result) return null;

    const total = Object.values(result.wuxingCount).reduce((a, b) => a + b, 0);
    const wuXingList = ['木', '火', '土', '金', '水'];

    return (
      <View style={styles.wuxingCard}>
        <Text style={styles.cardTitle}>五行分布</Text>
        <View style={styles.wuxingBars}>
          {wuXingList.map(wx => {
            const count = result.wuxingCount[wx] || 0;
            const percent = total > 0 ? Math.round((count / total) * 100) : 0;
            return (
              <View key={wx} style={styles.wuxingBarItem}>
                <View style={styles.wuxingBarLabel}>
                  <Text style={[styles.wuxingName, { color: WU_XING_COLORS[wx] }]}>{wx}</Text>
                  <Text style={styles.wuxingPercent}>{count}个 ({percent}%)</Text>
                </View>
                <View style={styles.wuxingBarTrack}>
                  <View style={[
                    styles.wuxingBarFill,
                    { width: `${percent}%`, backgroundColor: WU_XING_COLORS[wx] }
                  ]} />
                </View>
              </View>
            );
          })}
        </View>
        {/* 缺失五行 */}
        {Object.entries(result.wuxingCount)
          .filter(([_, count]) => count === 0)
          .map(([wx]) => (
            <View key={wx} style={styles.wuxingLack}>
              <Text style={styles.wuxingLackText}>⚠️ 八字缺 {wx}</Text>
            </View>
          ))
        }
      </View>
    );
  };

  // 渲染输入表单
  const renderInputForm = () => (
    <View style={styles.formContainer}>
      {/* 标题 */}
      <View style={styles.headerSection}>
        <Text style={styles.pageTitle}>星尘玄鉴-八字排盘</Text>
        <Text style={styles.pageSubtitle}>专业命理分析</Text>
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

        {/* 日期类型 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>日期类型：</Text>
          <View style={styles.formContent}>
            <View style={styles.calendarButtons}>
              <Pressable
                style={[styles.calendarBtn, calendarType === 'solar' && styles.calendarBtnActive]}
                onPress={() => setCalendarType('solar')}
              >
                <Text style={[styles.calendarBtnText, calendarType === 'solar' && styles.calendarBtnTextActive]}>公历</Text>
              </Pressable>
              <Pressable
                style={[styles.calendarBtn, calendarType === 'lunar' && styles.calendarBtnActive]}
                onPress={() => setCalendarType('lunar')}
              >
                <Text style={[styles.calendarBtnText, calendarType === 'lunar' && styles.calendarBtnTextActive]}>农历</Text>
              </Pressable>
            </View>
          </View>
        </View>

        {/* 出生日期 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>出生日期：</Text>
          <View style={styles.datePickerRow}>
            <View style={styles.pickerWrapper}>
              <Picker
                selectedValue={birthYear}
                onValueChange={setBirthYear}
                style={styles.picker}
              >
                {Array.from({ length: 100 }, (_, i) => 1950 + i).map(year => (
                  <Picker.Item key={year} label={`${year}年`} value={year} />
                ))}
              </Picker>
            </View>
            <View style={styles.pickerWrapper}>
              <Picker
                selectedValue={birthMonth}
                onValueChange={setBirthMonth}
                style={styles.picker}
              >
                {Array.from({ length: 12 }, (_, i) => i + 1).map(month => (
                  <Picker.Item key={month} label={`${month}月`} value={month} />
                ))}
              </Picker>
            </View>
            <View style={styles.pickerWrapper}>
              <Picker
                selectedValue={birthDay}
                onValueChange={setBirthDay}
                style={styles.picker}
              >
                {Array.from({ length: 31 }, (_, i) => i + 1).map(day => (
                  <Picker.Item key={day} label={`${day}日`} value={day} />
                ))}
              </Picker>
            </View>
          </View>
        </View>

        {/* 出生时辰 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>出生时辰：</Text>
          <View style={styles.formContent}>
            <View style={styles.hourPickerWrapper}>
              <Picker
                selectedValue={birthHour}
                onValueChange={setBirthHour}
                style={styles.picker}
              >
                {SHICHEN_OPTIONS.map(opt => (
                  <Picker.Item key={opt.value} label={opt.label} value={opt.value} />
                ))}
              </Picker>
            </View>
          </View>
        </View>

        {/* 免费试算按钮 */}
        <Pressable
          style={styles.secondaryButton}
          onPress={calculateBaziTemp}
          disabled={loading}
        >
          <Text style={styles.secondaryButtonText}>
            {loading ? '计算中...' : '免费试算（不保存）'}
          </Text>
        </Pressable>

        {/* 开始排盘按钮 */}
        <Pressable
          style={[styles.primaryButton, (loading || saving) && styles.buttonDisabled]}
          onPress={handleCreateBaziChart}
          disabled={loading || saving}
        >
          {loading || saving ? (
            <ActivityIndicator color={THEME_COLOR_LIGHT} />
          ) : (
            <Text style={styles.primaryButtonText}>开始排盘</Text>
          )}
        </Pressable>

        <Text style={styles.tipText}>
          免费试算：立即查看四柱八字 | 开始排盘：保存到链上并获取完整解盘
        </Text>
      </View>
    </View>
  );

  // 渲染结果
  const renderResult = () => {
    if (!result) return null;
    const cd = result.chartData;

    // 获取四柱数据
    const pillars = cd ? [
      { label: '年柱', zhu: cd.sizhu?.yearZhu },
      { label: '月柱', zhu: cd.sizhu?.monthZhu },
      { label: '日柱', zhu: cd.sizhu?.dayZhu, isDay: true },
      { label: '时柱', zhu: cd.sizhu?.hourZhu },
    ] : [];

    return (
      <View style={styles.resultContainer}>
        {/* 基本信息 */}
        <View style={styles.infoCard}>
          <Text style={styles.infoTitle}>{result.name} - {result.gender === 'male' ? '乾造' : '坤造'}</Text>
          <Text style={styles.infoSubtitle}>
            {result.birthYear}年{result.birthMonth}月{result.birthDay}日 {result.birthHour}时 | 属{result.shengxiao}
          </Text>
        </View>

        {/* 命盘表格 */}
        {cd && (
          <View style={styles.chartTable}>
            {/* 表头 */}
            <View style={styles.tableRow}>
              <View style={[styles.tableCell, styles.tableLabelCell]}><Text style={styles.tableLabelText}>四柱</Text></View>
              {pillars.map((p, i) => (
                <View key={i} style={[styles.tableCell, styles.tableHeaderCell]}>
                  <Text style={[styles.tableHeaderText, p.isDay && { color: THEME_COLOR }]}>{p.label}</Text>
                </View>
              ))}
            </View>

            {/* 十神 */}
            <View style={styles.tableRow}>
              <View style={[styles.tableCell, styles.tableLabelCell]}><Text style={styles.tableLabelText}>十神</Text></View>
              {pillars.map((p, i) => (
                <View key={i} style={styles.tableCell}>
                  <Text style={styles.tableCellText}>{p.isDay ? '日元' : (p.zhu?.tianganShishen || '-')}</Text>
                </View>
              ))}
            </View>

            {/* 天干 */}
            <View style={styles.tableRow}>
              <View style={[styles.tableCell, styles.tableLabelCell]}><Text style={styles.tableLabelText}>{result.gender === 'male' ? '乾造' : '坤造'}</Text></View>
              {pillars.map((p, i) => {
                const gz = p.zhu?.ganzhi || '';
                const gan = gz.charAt(0);
                const ganIdx = TIAN_GAN.indexOf(gan);
                const wuxing = ganIdx >= 0 ? TIAN_GAN_WUXING[ganIdx] : '木';
                const color = WU_XING_COLORS[wuxing] || '#333';
                return (
                  <View key={i} style={styles.tableCell}>
                    <Text style={[styles.ganzhiText, { color }]}>{gan || '-'}</Text>
                  </View>
                );
              })}
            </View>

            {/* 地支 */}
            <View style={styles.tableRow}>
              <View style={[styles.tableCell, styles.tableLabelCell]}><Text style={styles.tableLabelText}></Text></View>
              {pillars.map((p, i) => {
                const gz = p.zhu?.ganzhi || '';
                const zhi = gz.charAt(1);
                const zhiIdx = DI_ZHI.indexOf(zhi);
                const wuxing = zhiIdx >= 0 ? DI_ZHI_WUXING[zhiIdx] : '水';
                const color = WU_XING_COLORS[wuxing] || '#333';
                return (
                  <View key={i} style={styles.tableCell}>
                    <Text style={[styles.ganzhiText, { color }]}>{zhi || '-'}</Text>
                  </View>
                );
              })}
            </View>

            {/* 藏干 */}
            <View style={styles.tableRow}>
              <View style={[styles.tableCell, styles.tableLabelCell]}><Text style={styles.tableLabelText}>藏干</Text></View>
              {pillars.map((p, i) => (
                <View key={i} style={styles.tableCell}>
                  <Text style={styles.tableCellSmall}>
                    {p.zhu?.cangganList?.map((cg: any) => `${cg.gan}${cg.shishen}`).join('\n') || '-'}
                  </Text>
                </View>
              ))}
            </View>

            {/* 纳音 */}
            <View style={styles.tableRow}>
              <View style={[styles.tableCell, styles.tableLabelCell]}><Text style={styles.tableLabelText}>纳音</Text></View>
              {pillars.map((p, i) => (
                <View key={i} style={styles.tableCell}>
                  <Text style={styles.tableCellSmall}>{p.zhu?.nayin || '-'}</Text>
                </View>
              ))}
            </View>

            {/* 地势(长生) */}
            <View style={styles.tableRow}>
              <View style={[styles.tableCell, styles.tableLabelCell]}><Text style={styles.tableLabelText}>地势</Text></View>
              {pillars.map((p, i) => (
                <View key={i} style={styles.tableCell}>
                  <Text style={styles.tableCellText}>{p.zhu?.changsheng || '-'}</Text>
                </View>
              ))}
            </View>

            {/* 自坐 */}
            <View style={styles.tableRow}>
              <View style={[styles.tableCell, styles.tableLabelCell]}><Text style={styles.tableLabelText}>自坐</Text></View>
              {pillars.map((p, i) => (
                <View key={i} style={styles.tableCell}>
                  <Text style={styles.tableCellText}>{p.zhu?.zizuo || '-'}</Text>
                </View>
              ))}
            </View>

            {/* 空亡 */}
            <View style={styles.tableRow}>
              <View style={[styles.tableCell, styles.tableLabelCell]}><Text style={styles.tableLabelText}>空亡</Text></View>
              <View style={styles.tableCell}>
                <Text style={styles.tableCellSmall}>{cd.kongwang?.yearKong ? '空' : '-'}</Text>
              </View>
              <View style={styles.tableCell}>
                <Text style={styles.tableCellSmall}>{cd.kongwang?.monthKong ? '空' : '-'}</Text>
              </View>
              <View style={styles.tableCell}>
                <Text style={styles.tableCellSmall}>{cd.kongwang?.dayKong ? '空' : '-'}</Text>
              </View>
              <View style={styles.tableCell}>
                <Text style={styles.tableCellSmall}>{cd.kongwang?.hourKong ? '空' : '-'}</Text>
              </View>
            </View>
          </View>
        )}

        {/* 神煞（按柱位分组显示） */}
        {cd?.shenshaList && cd.shenshaList.length > 0 && (
          <View style={styles.sectionCard}>
            <Text style={styles.sectionTitle}>神煞</Text>
            <View style={styles.shenshaTable}>
              {/* 表头 */}
              <View style={styles.shenshaTableRow}>
                <View style={[styles.shenshaTableCell, styles.shenshaTableLabelCell]}>
                  <Text style={styles.shenshaTableLabelText}>柱位</Text>
                </View>
                <View style={[styles.shenshaTableCell, styles.shenshaTableHeaderCell]}>
                  <Text style={styles.shenshaTableHeaderText}>年柱</Text>
                </View>
                <View style={[styles.shenshaTableCell, styles.shenshaTableHeaderCell]}>
                  <Text style={styles.shenshaTableHeaderText}>月柱</Text>
                </View>
                <View style={[styles.shenshaTableCell, styles.shenshaTableHeaderCell]}>
                  <Text style={styles.shenshaTableHeaderText}>日柱</Text>
                </View>
                <View style={[styles.shenshaTableCell, styles.shenshaTableHeaderCell]}>
                  <Text style={styles.shenshaTableHeaderText}>时柱</Text>
                </View>
              </View>
              {/* 神煞内容行 */}
              <View style={styles.shenshaTableRow}>
                <View style={[styles.shenshaTableCell, styles.shenshaTableLabelCell]}>
                  <Text style={styles.shenshaTableLabelText}>神煞</Text>
                </View>
                {['Year', 'Month', 'Day', 'Hour'].map((pos) => {
                  const items = cd.shenshaList.filter((ss: any) => ss.position === pos);
                  return (
                    <View key={pos} style={styles.shenshaTableCell}>
                      {items.length > 0 ? items.map((ss: any, idx: number) => (
                        <Text key={idx} style={[
                          styles.shenshaItemText,
                          ss.nature === 'Xiong' ? styles.shenshaItemBad : styles.shenshaItemGood
                        ]}>
                          {ss.shensha}
                        </Text>
                      )) : <Text style={styles.shenshaItemEmpty}>-</Text>}
                    </View>
                  );
                })}
              </View>
            </View>
          </View>
        )}

        {/* 五行强度 */}
        {cd?.wuxingStrength && (
          <View style={styles.sectionCard}>
            <Text style={styles.sectionTitle}>五行强度</Text>
            <View style={styles.wuxingRow}>
              {(['jin', 'mu', 'shui', 'huo', 'tu'] as const).map((wx) => {
                const labels: Record<string, string> = { jin: '金', mu: '木', shui: '水', huo: '火', tu: '土' };
                const label = labels[wx] || '木';
                const val = cd.wuxingStrength?.[wx] || 0;
                return (
                  <View key={wx} style={styles.wuxingItem}>
                    <Text style={[styles.wuxingLabelLarge, { color: WU_XING_COLORS[label] }]}>{label}</Text>
                    <Text style={styles.wuxingValue}>{val}</Text>
                  </View>
                );
              })}
            </View>
          </View>
        )}

        {/* 命盘分析 */}
        {cd?.analysis && (
          <View style={styles.sectionCard}>
            <Text style={styles.sectionTitle}>命盘分析</Text>
            <View style={styles.analysisRow}>
              <Text style={styles.analysisItem}>格局: {cd.analysis.geJu}</Text>
              <Text style={styles.analysisItem}>强弱: {cd.analysis.qiangRuo}</Text>
            </View>
            <View style={styles.analysisRow}>
              <Text style={styles.analysisItem}>用神: {cd.analysis.yongShen}</Text>
              <Text style={styles.analysisItem}>喜神: {cd.analysis.xiShen}</Text>
              <Text style={styles.analysisItem}>忌神: {cd.analysis.jiShen}</Text>
            </View>
            <Text style={styles.scoreText}>综合评分: {cd.analysis.score}/100</Text>
          </View>
        )}

        {/* 起运信息 */}
        {cd?.qiyun && (
          <View style={styles.sectionCard}>
            <Text style={styles.sectionTitle}>起运信息</Text>
            <Text style={styles.qiyunText}>
              出生后{cd.qiyun.ageYears}年{cd.qiyun.ageMonths}月{cd.qiyun.ageDays}日起大运，
              {cd.qiyun.isShun ? '顺排' : '逆排'}，
              {cd.qiyun.jiaoyunYear}年{cd.qiyun.jiaoyunMonth}月{cd.qiyun.jiaoyunDay}日交运
            </Text>
          </View>
        )}

        {/* 大运列表 */}
        {cd?.dayunList && cd.dayunList.length > 0 && (
          <View style={styles.sectionCard}>
            <Text style={styles.sectionTitle}>大运</Text>
            <ScrollView horizontal showsHorizontalScrollIndicator={false}>
              <View style={styles.dayunRow}>
                {cd.dayunList.map((dy: any, i: number) => (
                  <View key={i} style={styles.dayunItem}>
                    <Text style={styles.dayunAge}>{dy.startAge}-{dy.endAge}岁</Text>
                    <Text style={styles.dayunGanzhi}>{dy.ganzhi}</Text>
                    <Text style={styles.dayunShishen}>{dy.tianganShishen}</Text>
                    <Text style={styles.dayunYear}>{dy.startYear}年</Text>
                  </View>
                ))}
              </View>
            </ScrollView>

            {/* 流年表格 */}
            <Text style={[styles.sectionTitle, { marginTop: 16 }]}>流年</Text>
            <ScrollView horizontal showsHorizontalScrollIndicator={false}>
              <View>
                {/* 表头：大运干支 */}
                <View style={styles.liunianHeaderRow}>
                  <View style={styles.liunianLabelCell}><Text style={styles.liunianLabelText}>大运</Text></View>
                  {cd.dayunList.map((dy: any, i: number) => (
                    <View key={i} style={styles.liunianHeaderCell}>
                      <Text style={styles.liunianHeaderText}>{dy.ganzhi}</Text>
                      <Text style={styles.liunianSubText}>{dy.tianganShishen}</Text>
                    </View>
                  ))}
                </View>
                {/* 流年行（每个大运10年） */}
                {Array.from({ length: 10 }).map((_, rowIdx) => (
                  <View key={rowIdx} style={styles.liunianRow}>
                    <View style={styles.liunianLabelCell}>
                      <Text style={styles.liunianLabelText}>流年{rowIdx + 1}</Text>
                    </View>
                    {cd.dayunList.map((dy: any, colIdx: number) => {
                      const ln = dy.liunianList?.[rowIdx];
                      return (
                        <View key={colIdx} style={styles.liunianCell}>
                          {ln ? (
                            <>
                              <Text style={styles.liunianYear}>{ln.year}</Text>
                              <Text style={styles.liunianGanzhi}>{ln.ganzhi}</Text>
                              <Text style={styles.liunianShishen}>{ln.tianganShishen}</Text>
                            </>
                          ) : <Text style={styles.liunianEmpty}>-</Text>}
                        </View>
                      );
                    })}
                  </View>
                ))}
              </View>
            </ScrollView>
          </View>
        )}

        {/* 操作按钮 */}
        <View style={styles.actionButtons}>
          <Pressable style={styles.saveButton} onPress={handleSaveToChain} disabled={saving}>
            {saving ? <ActivityIndicator color="#FFF" /> : (
              <>
                <Ionicons name="cloud-upload-outline" size={18} color="#FFF" />
                <Text style={styles.saveButtonText}>保存到链上</Text>
              </>
            )}
          </Pressable>
          <Pressable style={styles.aiButton} onPress={() => Alert.alert('提示', 'AI解读功能即将上线')}>
            <Text style={styles.aiButtonText}>🤖 AI智能解盘</Text>
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
        <Pressable style={styles.navItem} onPress={() => router.push('/divination/history' as any)}>
          <Ionicons name="albums-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>我的记录</Text>
        </Pressable>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Pressable style={styles.navItem}>
          <Ionicons name="calendar-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>生日</Text>
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
        onSuccess={handleUnlockSuccess}
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
  calendarButtons: {
    flexDirection: 'row',
  },
  calendarBtn: {
    paddingHorizontal: 20,
    paddingVertical: 6,
    borderWidth: 1,
    borderColor: '#D9D9D9',
    backgroundColor: '#FFF',
  },
  calendarBtnActive: {
    backgroundColor: THEME_COLOR,
    borderColor: THEME_COLOR,
  },
  calendarBtnText: {
    fontSize: 14,
    color: '#666',
  },
  calendarBtnTextActive: {
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
  siZhuCard: {
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
  siZhuContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    gap: 8,
  },
  zhuColumn: {
    flex: 1,
    alignItems: 'center',
    gap: 8,
  },
  zhuLabel: {
    fontSize: 12,
    color: '#999',
  },
  ganBox: {
    width: '100%',
    aspectRatio: 1,
    borderWidth: 2,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FAFAFA',
    position: 'relative',
  },
  ganText: {
    fontSize: 28,
    fontWeight: 'bold',
  },
  zhiBox: {
    width: '100%',
    aspectRatio: 1,
    borderWidth: 2,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FAFAFA',
    position: 'relative',
  },
  zhiText: {
    fontSize: 28,
    fontWeight: 'bold',
  },
  wuxingLabel: {
    position: 'absolute',
    bottom: 4,
    right: 6,
    fontSize: 10,
    color: '#999',
  },
  dayMasterTag: {
    backgroundColor: THEME_COLOR,
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
  },
  dayMasterTagText: {
    fontSize: 10,
    color: '#FFF',
  },
  wuxingCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  wuxingBars: {
    gap: 12,
  },
  wuxingBarItem: {
    gap: 6,
  },
  wuxingBarLabel: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  wuxingName: {
    fontSize: 14,
    fontWeight: '600',
  },
  wuxingPercent: {
    fontSize: 12,
    color: '#999',
  },
  wuxingBarTrack: {
    height: 12,
    backgroundColor: '#F0F0F0',
    borderRadius: 6,
    overflow: 'hidden',
  },
  wuxingBarFill: {
    height: '100%',
    borderRadius: 6,
  },
  wuxingLack: {
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
    borderStyle: 'dashed',
  },
  wuxingLackText: {
    fontSize: 13,
    color: '#E74C3C',
  },
  actionButtons: {
    gap: 8,
  },
  saveButton: {
    height: 48,
    backgroundColor: '#4CAF50',
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    borderRadius: 4,
    gap: 8,
  },
  saveButtonText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#FFF',
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
  // 新增：结果页面表格样式
  infoTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    textAlign: 'center',
  },
  infoSubtitle: {
    fontSize: 13,
    color: '#666',
    textAlign: 'center',
    marginTop: 4,
  },
  chartTable: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    marginTop: 12,
    overflow: 'hidden',
    borderWidth: 1,
    borderColor: '#E5E5E5',
  },
  tableRow: {
    flexDirection: 'row',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  tableCell: {
    flex: 1,
    paddingVertical: 8,
    paddingHorizontal: 4,
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 40,
  },
  tableLabelCell: {
    flex: 0.6,
    backgroundColor: '#FDF8E8',
  },
  tableLabelText: {
    fontSize: 12,
    color: '#8B6914',
    fontWeight: '500',
  },
  tableHeaderCell: {
    backgroundColor: '#FAFAFA',
  },
  tableHeaderText: {
    fontSize: 13,
    fontWeight: '600',
    color: '#333',
  },
  tableCellText: {
    fontSize: 12,
    color: '#333',
    textAlign: 'center',
  },
  tableCellSmall: {
    fontSize: 10,
    color: '#666',
    textAlign: 'center',
    lineHeight: 14,
  },
  ganzhiText: {
    fontSize: 24,
    fontWeight: '700',
  },
  sectionCard: {
    backgroundColor: '#FFF',
    borderRadius: 8,
    marginTop: 12,
    padding: 12,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME_COLOR,
    marginBottom: 8,
  },
  shenshaContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 6,
  },
  shenshaTag: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    backgroundColor: '#E8F5E9',
    borderRadius: 4,
  },
  shenshaTagBad: {
    backgroundColor: '#FFEBEE',
  },
  shenshaText: {
    fontSize: 11,
    color: '#333',
  },
  wuxingRow: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  wuxingItem: {
    alignItems: 'center',
  },
  wuxingLabelLarge: {
    fontSize: 16,
    fontWeight: '600',
  },
  wuxingValue: {
    fontSize: 12,
    color: '#666',
    marginTop: 2,
  },
  analysisRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 12,
    marginBottom: 6,
  },
  analysisItem: {
    fontSize: 13,
    color: '#333',
  },
  scoreText: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME_COLOR,
    marginTop: 8,
  },
  qiyunText: {
    fontSize: 13,
    color: '#333',
    lineHeight: 20,
  },
  dayunRow: {
    flexDirection: 'row',
    gap: 8,
  },
  dayunItem: {
    width: 60,
    alignItems: 'center',
    paddingVertical: 8,
    backgroundColor: '#FAFAFA',
    borderRadius: 6,
  },
  dayunAge: {
    fontSize: 10,
    color: '#999',
  },
  dayunGanzhi: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
    marginVertical: 4,
  },
  dayunShishen: {
    fontSize: 10,
    color: THEME_COLOR,
  },
  dayunYear: {
    fontSize: 10,
    color: '#666',
  },
  // 流年样式
  liunianHeaderRow: {
    flexDirection: 'row',
    borderBottomWidth: 1,
    borderBottomColor: '#E5E5E5',
  },
  liunianRow: {
    flexDirection: 'row',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  liunianLabelCell: {
    width: 50,
    paddingVertical: 6,
    paddingHorizontal: 4,
    backgroundColor: '#FDF8E8',
    alignItems: 'center',
    justifyContent: 'center',
  },
  liunianLabelText: {
    fontSize: 10,
    color: '#8B6914',
    fontWeight: '500',
  },
  liunianHeaderCell: {
    width: 55,
    paddingVertical: 6,
    alignItems: 'center',
    backgroundColor: '#FAFAFA',
  },
  liunianHeaderText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#333',
  },
  liunianSubText: {
    fontSize: 9,
    color: THEME_COLOR,
  },
  liunianCell: {
    width: 55,
    paddingVertical: 4,
    alignItems: 'center',
  },
  liunianYear: {
    fontSize: 9,
    color: '#999',
  },
  liunianGanzhi: {
    fontSize: 11,
    fontWeight: '500',
    color: '#333',
  },
  liunianShishen: {
    fontSize: 9,
    color: '#666',
  },
  liunianEmpty: {
    fontSize: 10,
    color: '#CCC',
  },
  // 神煞表格样式
  shenshaTable: {
    borderWidth: 1,
    borderColor: '#E5E5E5',
    borderRadius: 6,
    overflow: 'hidden',
  },
  shenshaTableRow: {
    flexDirection: 'row',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  shenshaTableCell: {
    flex: 1,
    paddingVertical: 8,
    paddingHorizontal: 4,
    alignItems: 'center',
    justifyContent: 'flex-start',
    minHeight: 40,
  },
  shenshaTableLabelCell: {
    flex: 0.6,
    backgroundColor: '#FDF8E8',
    justifyContent: 'center',
  },
  shenshaTableLabelText: {
    fontSize: 11,
    color: '#8B6914',
    fontWeight: '500',
  },
  shenshaTableHeaderCell: {
    backgroundColor: '#FAFAFA',
    justifyContent: 'center',
  },
  shenshaTableHeaderText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#333',
  },
  shenshaItemText: {
    fontSize: 10,
    marginVertical: 1,
  },
  shenshaItemGood: {
    color: '#2E7D32',
  },
  shenshaItemBad: {
    color: '#C62828',
  },
  shenshaItemEmpty: {
    fontSize: 10,
    color: '#CCC',
  },
});
