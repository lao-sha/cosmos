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

  // 计算八字
  const calculateBazi = async () => {
    setLoading(true);
    await new Promise(resolve => setTimeout(resolve, 800));

    // 简化计算
    const yearGanIndex = (birthYear - 4) % 10;
    const yearZhiIndex = (birthYear - 4) % 12;
    const monthGanIndex = (yearGanIndex * 2 + birthMonth) % 10;
    const monthZhiIndex = (birthMonth + 1) % 12;
    const dayNum = Math.floor(new Date(birthYear, birthMonth - 1, birthDay).getTime() / (24 * 60 * 60 * 1000));
    const dayGanIndex = (dayNum + 9) % 10;
    const dayZhiIndex = (dayNum + 1) % 12;
    const hourZhiIndex = Math.floor((birthHour + 1) / 2) % 12;
    const hourGanIndex = (dayGanIndex * 2 + hourZhiIndex) % 10;

    const siZhu = {
      year: { gan: yearGanIndex, zhi: yearZhiIndex },
      month: { gan: monthGanIndex, zhi: monthZhiIndex },
      day: { gan: dayGanIndex, zhi: dayZhiIndex },
      hour: { gan: hourGanIndex, zhi: hourZhiIndex },
    };

    // 五行统计
    const wuxingCount: Record<string, number> = { '木': 0, '火': 0, '土': 0, '金': 0, '水': 0 };
    Object.values(siZhu).forEach(zhu => {
      wuxingCount[TIAN_GAN_WUXING[zhu.gan]]++;
      wuxingCount[DI_ZHI_WUXING[zhu.zhi]]++;
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
      dayMaster: dayGanIndex,
      shengxiao: SHENG_XIAO[yearZhiIndex],
      createdAt: new Date(),
    };

    setResult(baziResult);
    setHistory(prev => [baziResult, ...prev]);
    setLoading(false);
  };

  const handleReset = () => {
    setResult(null);
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
                { borderColor: WU_XING_COLORS[TIAN_GAN_WUXING[pillar.data.gan]] }
              ]}>
                <Text style={[
                  styles.ganText,
                  { color: WU_XING_COLORS[TIAN_GAN_WUXING[pillar.data.gan]] }
                ]}>
                  {TIAN_GAN[pillar.data.gan]}
                </Text>
                <Text style={styles.wuxingLabel}>{TIAN_GAN_WUXING[pillar.data.gan]}</Text>
              </View>
              <View style={[
                styles.zhiBox,
                { borderColor: WU_XING_COLORS[DI_ZHI_WUXING[pillar.data.zhi]] }
              ]}>
                <Text style={[
                  styles.zhiText,
                  { color: WU_XING_COLORS[DI_ZHI_WUXING[pillar.data.zhi]] }
                ]}>
                  {DI_ZHI[pillar.data.zhi]}
                </Text>
                <Text style={styles.wuxingLabel}>{DI_ZHI_WUXING[pillar.data.zhi]}</Text>
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
            const count = result.wuxingCount[wx];
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
          onPress={calculateBazi}
          disabled={loading}
        >
          <Text style={styles.secondaryButtonText}>
            {loading ? '计算中...' : '免费试算（不保存）'}
          </Text>
        </Pressable>

        {/* 开始排盘按钮 */}
        <Pressable
          style={[styles.primaryButton, loading && styles.buttonDisabled]}
          onPress={calculateBazi}
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
            style={styles.aiButton}
            onPress={() => Alert.alert('提示', 'AI解读功能即将上线')}
          >
            <Text style={styles.aiButtonText}>🤖 AI智能解盘</Text>
          </Pressable>
          <Pressable
            style={styles.detailButton}
            onPress={() => Alert.alert('提示', '详情页面即将上线')}
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
        <Pressable style={styles.navItem} onPress={() => router.push('/divination/bazi-list' as any)}>
          <Ionicons name="albums-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>我的命盘</Text>
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
