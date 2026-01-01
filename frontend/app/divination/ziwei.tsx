/**
 * 星尘玄鉴 - 紫微斗数排盘
 * 参考样式：专业紫微斗数排盘风格
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

// 十二宫
const PALACES = [
  '命宫', '兄弟', '夫妻', '子女', '财帛', '疾厄',
  '迁移', '仆役', '官禄', '田宅', '福德', '父母',
];

// 主星（十四主星）
const MAIN_STARS = [
  { name: '紫微', type: '帝星', nature: 'bright' },
  { name: '天机', type: '善星', nature: 'bright' },
  { name: '太阳', type: '贵星', nature: 'bright' },
  { name: '武曲', type: '财星', nature: 'bright' },
  { name: '天同', type: '福星', nature: 'bright' },
  { name: '廉贞', type: '囚星', nature: 'dark' },
  { name: '天府', type: '财库', nature: 'bright' },
  { name: '太阴', type: '财星', nature: 'bright' },
  { name: '贪狼', type: '桃花', nature: 'neutral' },
  { name: '巨门', type: '暗星', nature: 'dark' },
  { name: '天相', type: '印星', nature: 'bright' },
  { name: '天梁', type: '荫星', nature: 'bright' },
  { name: '七杀', type: '将星', nature: 'neutral' },
  { name: '破军', type: '耗星', nature: 'dark' },
];

// 辅星
const LUCKY_STARS = ['左辅', '右弼', '天魁', '天钺', '文昌', '文曲'];
const UNLUCKY_STARS = ['火星', '铃星', '擎羊', '陀罗', '地空', '地劫'];

// 四化
const SIHUA = [
  { name: '化禄', color: '#27AE60' },
  { name: '化权', color: '#E74C3C' },
  { name: '化科', color: '#3498DB' },
  { name: '化忌', color: '#95A5A6' },
];

// 五行局
const WUXING_JU = ['水二局', '木三局', '金四局', '土五局', '火六局'];

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
  { value: 18, label: '17-酉' },
  { value: 19, label: '19-戌' },
  { value: 20, label: '20-戌' },
  { value: 21, label: '21-亥' },
  { value: 22, label: '22-亥' },
  { value: 23, label: '23-子' },
];

// 性别类型
type Gender = 'male' | 'female';
type CalendarType = 'solar' | 'lunar';

// 宫位信息
interface PalaceInfo {
  name: string;
  mainStars: string[];
  luckyStars: string[];
  unluckyStars: string[];
  sihua: string | null;
  isMing: boolean;
  isShen: boolean;
}

// 紫微结果
interface ZiweiResult {
  id: number;
  name: string;
  gender: Gender;
  birthYear: number;
  birthMonth: number;
  birthDay: number;
  birthHour: number;
  calendarType: CalendarType;
  mingGong: string;
  shenGong: string;
  wuxingJu: string;
  mingZhu: string;
  shenZhu: string;
  palaces: PalaceInfo[];
  createdAt: Date;
}

export default function ZiweiPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ZiweiResult | null>(null);
  const [history, setHistory] = useState<ZiweiResult[]>([]);

  // 表单状态
  const [name, setName] = useState('');
  const [gender, setGender] = useState<Gender>('male');
  const [calendarType, setCalendarType] = useState<CalendarType>('lunar');
  const [birthYear, setBirthYear] = useState(1990);
  const [birthMonth, setBirthMonth] = useState(6);
  const [birthDay, setBirthDay] = useState(15);
  const [birthHour, setBirthHour] = useState(12);

  // 计算紫微命盘
  const calculateZiwei = async () => {
    setLoading(true);
    await new Promise(resolve => setTimeout(resolve, 800));

    // 简化计算
    const hourZhi = Math.floor((birthHour + 1) / 2) % 12;
    const mingGongIndex = (birthMonth + hourZhi) % 12;
    const shenGongIndex = (12 - hourZhi + birthMonth) % 12;
    const wuxingJuIndex = (birthYear + birthMonth) % 5;

    const mingZhuStars = ['贪狼', '巨门', '禄存', '文曲', '廉贞', '武曲', '破军', '武曲', '廉贞', '文曲', '禄存', '巨门'];
    const shenZhuStars = ['火星', '天相', '天梁', '天同', '文昌', '天机', '火星', '天相', '天梁', '天同', '文昌', '天机'];

    // 模拟排盘结果
    const palaces: PalaceInfo[] = PALACES.map((palace, i) => {
      const starPool = [...MAIN_STARS];
      const mainStarCount = Math.floor(Math.random() * 3);
      const shuffledStars = starPool.sort(() => Math.random() - 0.5);
      const mainStars = shuffledStars.slice(0, mainStarCount).map(s => s.name);

      const luckyCount = Math.floor(Math.random() * 2);
      const luckyStars = [...LUCKY_STARS].sort(() => Math.random() - 0.5).slice(0, luckyCount);

      const unluckyCount = Math.floor(Math.random() * 2);
      const unluckyStars = [...UNLUCKY_STARS].sort(() => Math.random() - 0.5).slice(0, unluckyCount);

      const sihua = Math.random() > 0.7 && mainStars.length > 0 ? SIHUA[Math.floor(Math.random() * SIHUA.length)].name : null;

      return {
        name: palace,
        mainStars,
        luckyStars,
        unluckyStars,
        sihua,
        isMing: i === 0,
        isShen: i === shenGongIndex,
      };
    });

    const ziweiResult: ZiweiResult = {
      id: Date.now(),
      name: name || '求测者',
      gender,
      birthYear,
      birthMonth,
      birthDay,
      birthHour,
      calendarType,
      mingGong: PALACES[0],
      shenGong: PALACES[shenGongIndex],
      wuxingJu: WUXING_JU[wuxingJuIndex],
      mingZhu: mingZhuStars[mingGongIndex],
      shenZhu: shenZhuStars[shenGongIndex],
      palaces,
      createdAt: new Date(),
    };

    setResult(ziweiResult);
    setHistory(prev => [ziweiResult, ...prev]);
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
        <Text style={styles.pageTitle}>星尘玄鉴-紫微斗数</Text>
        <Text style={styles.pageSubtitle}>东方星命学巅峰 · 十二宫命盘</Text>
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
                style={[styles.calendarBtn, calendarType === 'lunar' && styles.calendarBtnActive]}
                onPress={() => setCalendarType('lunar')}
              >
                <Text style={[styles.calendarBtnText, calendarType === 'lunar' && styles.calendarBtnTextActive]}>农历</Text>
              </Pressable>
              <Pressable
                style={[styles.calendarBtn, calendarType === 'solar' && styles.calendarBtnActive]}
                onPress={() => setCalendarType('solar')}
              >
                <Text style={[styles.calendarBtnText, calendarType === 'solar' && styles.calendarBtnTextActive]}>公历</Text>
              </Pressable>
            </View>
            <Text style={styles.tipTextSmall}>（紫微斗数以农历为准）</Text>
          </View>
        </View>

        {/* 出生日期 */}
        <View style={styles.formRow}>
          <Text style={styles.formLabel}>出生日期：</Text>
          <View style={styles.datePickerRow}>
            <View style={styles.pickerWrapper}>
              <Picker selectedValue={birthYear} onValueChange={setBirthYear} style={styles.picker}>
                {Array.from({ length: 100 }, (_, i) => 1950 + i).map(y => (
                  <Picker.Item key={y} label={`${y}年`} value={y} />
                ))}
              </Picker>
            </View>
            <View style={styles.pickerWrapper}>
              <Picker selectedValue={birthMonth} onValueChange={setBirthMonth} style={styles.picker}>
                {Array.from({ length: 12 }, (_, i) => i + 1).map(m => (
                  <Picker.Item key={m} label={`${m}月`} value={m} />
                ))}
              </Picker>
            </View>
            <View style={styles.pickerWrapper}>
              <Picker selectedValue={birthDay} onValueChange={setBirthDay} style={styles.picker}>
                {Array.from({ length: 30 }, (_, i) => i + 1).map(d => (
                  <Picker.Item key={d} label={`${d}日`} value={d} />
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
              <Picker selectedValue={birthHour} onValueChange={setBirthHour} style={styles.picker}>
                {SHICHEN_OPTIONS.map(opt => (
                  <Picker.Item key={opt.value} label={opt.label} value={opt.value} />
                ))}
              </Picker>
            </View>
            <Text style={styles.unitText}>时</Text>
          </View>
        </View>

        {/* 免费试算按钮 */}
        <Pressable
          style={styles.secondaryButton}
          onPress={calculateZiwei}
          disabled={loading}
        >
          <Text style={styles.secondaryButtonText}>
            {loading ? '计算中...' : '免费试算（不保存）'}
          </Text>
        </Pressable>

        {/* 开始排盘按钮 */}
        <Pressable
          style={[styles.primaryButton, loading && styles.buttonDisabled]}
          onPress={calculateZiwei}
          disabled={loading}
        >
          {loading ? (
            <ActivityIndicator color={THEME_COLOR_LIGHT} />
          ) : (
            <Text style={styles.primaryButtonText}>开始排盘（上链存储）</Text>
          )}
        </Pressable>

        <Text style={styles.tipText}>
          免费试算：立即查看命盘 | 开始排盘：保存到链上并获取完整解盘
        </Text>
      </View>

      {/* 五行局说明 */}
      <View style={styles.infoCard}>
        <Text style={styles.cardTitle}>五行局</Text>
        <View style={styles.wuxingGrid}>
          {WUXING_JU.map((ju, idx) => (
            <View key={idx} style={styles.wuxingItem}>
              <Text style={styles.wuxingName}>{ju}</Text>
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
            <Text style={styles.infoLabel}>姓名</Text>
            <Text style={styles.infoValue}>{result.name} ({result.gender === 'male' ? '男' : '女'})</Text>
          </View>
          <View style={styles.infoDivider} />
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>{result.calendarType === 'lunar' ? '农历' : '公历'}</Text>
            <Text style={styles.infoValue}>
              {result.birthYear}年{result.birthMonth}月{result.birthDay}日 {result.birthHour}时
            </Text>
          </View>
        </View>

        {/* 核心信息 */}
        <View style={styles.coreCard}>
          <Text style={styles.coreTitle}>{result.wuxingJu}</Text>
          <View style={styles.coreTags}>
            <View style={[styles.coreTag, { backgroundColor: '#667EEA20' }]}>
              <Text style={[styles.coreTagText, { color: '#667EEA' }]}>命宫：{result.mingGong}</Text>
            </View>
            <View style={[styles.coreTag, { backgroundColor: '#F093FB20' }]}>
              <Text style={[styles.coreTagText, { color: '#9B59B6' }]}>身宫：{result.shenGong}</Text>
            </View>
          </View>
          <View style={styles.coreTags}>
            <View style={[styles.coreTag, { backgroundColor: '#27AE6020' }]}>
              <Text style={[styles.coreTagText, { color: '#27AE60' }]}>命主：{result.mingZhu}</Text>
            </View>
            <View style={[styles.coreTag, { backgroundColor: '#E74C3C20' }]}>
              <Text style={[styles.coreTagText, { color: '#E74C3C' }]}>身主：{result.shenZhu}</Text>
            </View>
          </View>
        </View>

        {/* 十二宫命盘 */}
        <View style={styles.panCard}>
          <Text style={styles.cardTitle}>十二宫命盘</Text>
          <View style={styles.palaceGrid}>
            {result.palaces.map((palace, idx) => (
              <View
                key={idx}
                style={[
                  styles.palaceCell,
                  palace.isMing && styles.palaceCellMing,
                  palace.isShen && styles.palaceCellShen,
                ]}
              >
                <View style={styles.palaceHeader}>
                  <Text style={[
                    styles.palaceName,
                    palace.isMing && styles.palaceNameMing,
                    palace.isShen && styles.palaceNameShen,
                  ]}>
                    {palace.name}
                  </Text>
                  {palace.isMing && (
                    <View style={styles.palaceBadgeMing}>
                      <Text style={styles.palaceBadgeText}>命</Text>
                    </View>
                  )}
                  {palace.isShen && (
                    <View style={styles.palaceBadgeShen}>
                      <Text style={styles.palaceBadgeText}>身</Text>
                    </View>
                  )}
                </View>
                <View style={styles.starsContainer}>
                  {palace.mainStars.map((star, i) => (
                    <View key={i} style={styles.starRow}>
                      <Text style={styles.mainStar}>{star}</Text>
                      {palace.sihua && i === 0 && (
                        <Text style={[styles.sihuaText, {
                          color: SIHUA.find(s => s.name === palace.sihua)?.color || '#333'
                        }]}>
                          {palace.sihua}
                        </Text>
                      )}
                    </View>
                  ))}
                  {palace.luckyStars.map((star, i) => (
                    <Text key={`lucky-${i}`} style={styles.luckyStar}>{star}</Text>
                  ))}
                  {palace.unluckyStars.map((star, i) => (
                    <Text key={`unlucky-${i}`} style={styles.unluckyStar}>{star}</Text>
                  ))}
                  {palace.mainStars.length === 0 && palace.luckyStars.length === 0 && palace.unluckyStars.length === 0 && (
                    <Text style={styles.noStar}>-</Text>
                  )}
                </View>
              </View>
            ))}
          </View>
        </View>

        {/* 图例 */}
        <View style={styles.legendCard}>
          <View style={styles.legendItem}>
            <View style={[styles.legendDot, { backgroundColor: '#9B59B6' }]} />
            <Text style={styles.legendText}>主星</Text>
          </View>
          <View style={styles.legendItem}>
            <View style={[styles.legendDot, { backgroundColor: '#27AE60' }]} />
            <Text style={styles.legendText}>吉星</Text>
          </View>
          <View style={styles.legendItem}>
            <View style={[styles.legendDot, { backgroundColor: '#E74C3C' }]} />
            <Text style={styles.legendText}>煞星</Text>
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
            onPress={() => Alert.alert('提示', '大限流年功能即将上线')}
          >
            <Text style={styles.detailButtonText}>大限流年 →</Text>
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
        <Pressable style={styles.navItem} onPress={() => router.push('/divination/ziwei-list' as any)}>
          <Ionicons name="albums-outline" size={20} color="#999" />
          <Text style={styles.navItemText}>我的命盘</Text>
        </Pressable>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Pressable style={styles.navItem} onPress={() => Alert.alert('说明', '紫微斗数是中国古代命理学巅峰，以北斗星为主体，通过十二宫位推算人的命运')}>
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
    paddingHorizontal: 16,
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
  unitText: {
    fontSize: 14,
    color: '#666',
  },
  tipTextSmall: {
    fontSize: 11,
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
  wuxingGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    justifyContent: 'center',
  },
  wuxingItem: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
  },
  wuxingName: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
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
  coreCard: {
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
  coreTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: THEME_COLOR,
    marginBottom: 12,
  },
  coreTags: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 8,
  },
  coreTag: {
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 4,
  },
  coreTagText: {
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
  palaceGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 6,
  },
  palaceCell: {
    width: '31.5%',
    minHeight: 90,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 4,
    padding: 8,
    backgroundColor: '#FAFAFA',
  },
  palaceCellMing: {
    borderColor: '#667EEA',
    borderWidth: 2,
  },
  palaceCellShen: {
    borderColor: '#9B59B6',
    borderWidth: 2,
  },
  palaceHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 6,
    gap: 4,
  },
  palaceName: {
    fontSize: 12,
    fontWeight: '600',
    color: '#333',
    flex: 1,
  },
  palaceNameMing: {
    color: '#667EEA',
  },
  palaceNameShen: {
    color: '#9B59B6',
  },
  palaceBadgeMing: {
    backgroundColor: '#667EEA',
    borderRadius: 2,
    paddingHorizontal: 4,
    paddingVertical: 1,
  },
  palaceBadgeShen: {
    backgroundColor: '#9B59B6',
    borderRadius: 2,
    paddingHorizontal: 4,
    paddingVertical: 1,
  },
  palaceBadgeText: {
    fontSize: 8,
    color: '#FFF',
    fontWeight: 'bold',
  },
  starsContainer: {
    gap: 2,
  },
  starRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    flexWrap: 'wrap',
  },
  mainStar: {
    fontSize: 11,
    color: '#9B59B6',
    fontWeight: '500',
  },
  sihuaText: {
    fontSize: 9,
    fontWeight: 'bold',
  },
  luckyStar: {
    fontSize: 10,
    color: '#27AE60',
  },
  unluckyStar: {
    fontSize: 10,
    color: '#E74C3C',
  },
  noStar: {
    fontSize: 11,
    color: '#CCC',
  },
  legendCard: {
    flexDirection: 'row',
    justifyContent: 'center',
    gap: 24,
    backgroundColor: '#FFF',
    borderRadius: 8,
    padding: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  legendItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  legendDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  legendText: {
    fontSize: 12,
    color: '#666',
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
