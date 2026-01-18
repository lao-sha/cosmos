/**
 * 星尘玄鉴 - 八字排盘
 * 参考样式：专业八字排盘风格
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
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { divinationService, DivinationType } from '@/services/divination.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';
import { getCurrentSignerAddress } from '@/lib/signer';

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

// 八字结果
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

  // 上链相关状态
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');
  const [saving, setSaving] = useState(false);

  // 免费试算（调用 Runtime API，不保存到链上）
  const calculateBaziTemp = async () => {
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

      // 从链端返回的数据中提取四柱信息
      const siZhu = {
        year: {
          gan: chartData.sizhu.yearZhu.tianganIndex,
          zhi: chartData.sizhu.yearZhu.dizhiIndex
        },
        month: {
          gan: chartData.sizhu.monthZhu.tianganIndex,
          zhi: chartData.sizhu.monthZhu.dizhiIndex
        },
        day: {
          gan: chartData.sizhu.dayZhu.tianganIndex,
          zhi: chartData.sizhu.dayZhu.dizhiIndex
        },
        hour: {
          gan: chartData.sizhu.hourZhu.tianganIndex,
          zhi: chartData.sizhu.hourZhu.dizhiIndex
        },
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
  const createBaziChart = async () => {
    // 检查钱包是否解锁
    if (!isSignerUnlocked()) {
      setShowUnlockDialog(true);
      return;
    }

    setLoading(true);
    setShowTxStatus(true);
    setTxStatus('准备上链...');

    try {
      // 调用服务创建八字命盘
      const chartId = await divinationService.createBaziChart(
        name || null,
        birthYear,
        birthMonth,
        birthDay,
        birthHour,
        0, // minute，暂时设为 0
        gender,
        calendarType,
        (status) => {
          setTxStatus(status);
        }
      );

      setTxStatus('创建成功！');

      // 创建成功后，调用免费试算获取结果显示
      setTimeout(async () => {
        setShowTxStatus(false);

        // 获取八字数据用于显示
        try {
          const chartData = await divinationService.calculateBaziTemp(
            birthYear,
            birthMonth,
            birthDay,
            birthHour,
            0,
            gender,
            calendarType
          );

          // 从链端返回的数据中提取四柱信息
          const siZhu = {
            year: {
              gan: chartData.sizhu.yearZhu.tianganIndex,
              zhi: chartData.sizhu.yearZhu.dizhiIndex
            },
            month: {
              gan: chartData.sizhu.monthZhu.tianganIndex,
              zhi: chartData.sizhu.monthZhu.dizhiIndex
            },
            day: {
              gan: chartData.sizhu.dayZhu.tianganIndex,
              zhi: chartData.sizhu.dayZhu.dizhiIndex
            },
            hour: {
              gan: chartData.sizhu.hourZhu.tianganIndex,
              zhi: chartData.sizhu.hourZhu.dizhiIndex
            },
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
            id: chartId,
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
          };

          setResult(baziResult);
          setHistory(prev => [baziResult, ...prev]);

          Alert.alert(
            '创建成功',
            `八字已保存到链上\\n命盘ID: ${chartId}`,
            [
              {
                text: '查看历史',
                onPress: () => router.push('/divination/bazi-list' as any),
              },
              { text: '确定' },
            ]
          );
        } catch (error) {
          console.error('获取八字数据失败:', error);
        }
      }, 1500);
    } catch (error: any) {
      console.error('创建八字失败:', error);
      setTxStatus('创建失败');
      setTimeout(() => {
        setShowTxStatus(false);
        Alert.alert('创建失败', error.message || '未知错误');
      }, 1500);
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    setResult(null);
  };

  // 保存到链上
  const handleSaveToChain = async () => {
    if (!result) {
      Alert.alert('提示', '请先进行八字排盘');
      return;
    }

    try {
      // 检查钱包是否解锁
      if (!isSignerUnlocked()) {
        setShowUnlockDialog(true);
        return;
      }

      setSaving(true);
      setShowTxStatus(true);
      setTxStatus('准备上链...');

      // 调用服务保存到链上
      const recordId = await divinationService.storeDivinationResult(
        DivinationType.Bazi,
        result,
        (status) => {
          setTxStatus(status);
        }
      );

      setTxStatus('保存成功！');

      setTimeout(() => {
        setShowTxStatus(false);
        Alert.alert(
          '保存成功',
          `八字已保存到链上\n记录ID: ${recordId}`,
          [
            {
              text: '查看历史',
              onPress: () => router.push('/divination/bazi-list' as any),
            },
            { text: '确定' },
          ]
        );
      }, 1500);
    } catch (error: any) {
      console.error('保存到链上失败:', error);
      setTxStatus('保存失败');
      setTimeout(() => {
        setShowTxStatus(false);
        Alert.alert('保存失败', error.message || '未知错误');
      }, 1500);
    } finally {
      setSaving(false);
    }
  };

  // 处理钱包解锁
  const handleUnlockSuccess = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      // 解锁成功后继续创建八字
      setTimeout(() => {
        createBaziChart();
      }, 300);
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
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
          style={[styles.primaryButton, loading && styles.buttonDisabled]}
          onPress={createBaziChart}
          disabled={loading}
        >
          {loading ? (
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

    return (
      <View style={styles.resultContainer}>
        {/* 基本信息卡片 */}
        <View style={styles.infoCard}>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>姓名</Text>
            <Text style={styles.infoValue}>{result.name}</Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>公历</Text>
            <Text style={styles.infoValue}>
              {result.birthYear}年{result.birthMonth}月{result.birthDay}日 {result.birthHour}时
            </Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>性别</Text>
            <Text style={styles.infoValue}>{result.gender === 'male' ? '男' : '女'}</Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>生肖</Text>
            <Text style={styles.infoValue}>属{result.shengxiao}</Text>
          </View>
        </View>

        {/* 四柱 */}
        {renderSiZhu()}

        {/* 五行分布 */}
        {renderWuXing()}

        {/* 操作按钮 */}
        <View style={styles.actionButtons}>
          <Pressable
            style={styles.saveButton}
            onPress={handleSaveToChain}
            disabled={saving}
          >
            {saving ? (
              <ActivityIndicator color="#FFF" />
            ) : (
              <>
                <Ionicons name="cloud-upload-outline" size={18} color="#FFF" />
                <Text style={styles.saveButtonText}>保存到链上</Text>
              </>
            )}
          </Pressable>
          <Pressable
            style={styles.aiButton}
            onPress={() => Alert.alert('提示', 'AI解读功能即将上线')}
          >
            <Text style={styles.aiButtonText}>🤖 AI智能解盘</Text>
          </Pressable>
          <Pressable
            style={styles.detailButton}
            onPress={() => router.push({
              pathname: '/divination/bazi-detail',
              params: { data: JSON.stringify(result) }
            } as any)}
          >
            <Text style={styles.detailButtonText}>查看命盘详情 →</Text>
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
});
