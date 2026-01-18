/**
 * 星尘玄鉴 - 万年历页面
 * 农历、节气、黄历宜忌
 * 主题色：金棕色 #B2955D
 */

import { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { BottomNavBar } from '@/components/BottomNavBar';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_BG = '#F5F5F7';

// 天干地支
const TIAN_GAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];
const SHENG_XIAO = ['鼠', '牛', '虎', '兔', '龙', '蛇', '马', '羊', '猴', '鸡', '狗', '猪'];

// 农历月份
const LUNAR_MONTHS = ['正', '二', '三', '四', '五', '六', '七', '八', '九', '十', '冬', '腊'];
const LUNAR_DAYS = ['初一', '初二', '初三', '初四', '初五', '初六', '初七', '初八', '初九', '初十',
  '十一', '十二', '十三', '十四', '十五', '十六', '十七', '十八', '十九', '二十',
  '廿一', '廿二', '廿三', '廿四', '廿五', '廿六', '廿七', '廿八', '廿九', '三十'];

// 星期
const WEEKDAYS = ['日', '一', '二', '三', '四', '五', '六'];

// 宜忌示例数据
const YI_JI_DATA = {
  yi: ['祭祀', '祈福', '求嗣', '开光', '出行', '解除', '纳采', '冠笄'],
  ji: ['嫁娶', '动土', '破土', '安葬', '开市', '入宅', '移徙'],
};

// 节气数据
const JIE_QI = ['小寒', '大寒', '立春', '雨水', '惊蛰', '春分', '清明', '谷雨',
  '立夏', '小满', '芒种', '夏至', '小暑', '大暑', '立秋', '处暑',
  '白露', '秋分', '寒露', '霜降', '立冬', '小雪', '大雪', '冬至'];

export default function CalendarPage() {
  const router = useRouter();
  const [currentDate, setCurrentDate] = useState(new Date());
  const [selectedDate, setSelectedDate] = useState(new Date());

  // 获取当前月的日期数组
  const getDaysInMonth = (date: Date) => {
    const year = date.getFullYear();
    const month = date.getMonth();
    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);
    const daysInMonth = lastDay.getDate();
    const startDayOfWeek = firstDay.getDay();

    const days = [];

    // 上月的日期
    const prevMonthLastDay = new Date(year, month, 0).getDate();
    for (let i = startDayOfWeek - 1; i >= 0; i--) {
      days.push({
        day: prevMonthLastDay - i,
        isCurrentMonth: false,
        date: new Date(year, month - 1, prevMonthLastDay - i),
      });
    }

    // 当月的日期
    for (let i = 1; i <= daysInMonth; i++) {
      days.push({
        day: i,
        isCurrentMonth: true,
        date: new Date(year, month, i),
      });
    }

    // 下月的日期
    const remainingDays = 42 - days.length;
    for (let i = 1; i <= remainingDays; i++) {
      days.push({
        day: i,
        isCurrentMonth: false,
        date: new Date(year, month + 1, i),
      });
    }

    return days;
  };

  // 简单的农历计算（示例）
  const getLunarDay = (date: Date) => {
    const day = date.getDate();
    return LUNAR_DAYS[(day - 1) % 30];
  };

  // 获取干支年
  const getGanZhiYear = (date: Date) => {
    const year = date.getFullYear();
    const ganIndex = (year - 4) % 10;
    const zhiIndex = (year - 4) % 12;
    return `${TIAN_GAN[ganIndex]}${DI_ZHI[zhiIndex]}`;
  };

  // 获取生肖
  const getShengXiao = (date: Date) => {
    const year = date.getFullYear();
    return SHENG_XIAO[(year - 4) % 12];
  };

  // 切换月份
  const changeMonth = (delta: number) => {
    const newDate = new Date(currentDate);
    newDate.setMonth(newDate.getMonth() + delta);
    setCurrentDate(newDate);
  };

  // 是否是今天
  const isToday = (date: Date) => {
    const today = new Date();
    return date.getDate() === today.getDate() &&
      date.getMonth() === today.getMonth() &&
      date.getFullYear() === today.getFullYear();
  };

  // 是否是选中日期
  const isSelected = (date: Date) => {
    return date.getDate() === selectedDate.getDate() &&
      date.getMonth() === selectedDate.getMonth() &&
      date.getFullYear() === selectedDate.getFullYear();
  };

  const days = getDaysInMonth(currentDate);

  return (
    <View style={styles.container}>
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>万年历</Text>
        <Pressable onPress={() => {
          setCurrentDate(new Date());
          setSelectedDate(new Date());
        }}>
          <Text style={styles.todayBtn}>今天</Text>
        </Pressable>
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 月份切换 */}
        <View style={styles.monthHeader}>
          <Pressable onPress={() => changeMonth(-1)} style={styles.monthArrow}>
            <Ionicons name="chevron-back" size={24} color={THEME_COLOR} />
          </Pressable>
          <View style={styles.monthInfo}>
            <Text style={styles.monthText}>
              {currentDate.getFullYear()}年{currentDate.getMonth() + 1}月
            </Text>
            <Text style={styles.lunarMonthText}>
              {getGanZhiYear(currentDate)}年 · {getShengXiao(currentDate)}年
            </Text>
          </View>
          <Pressable onPress={() => changeMonth(1)} style={styles.monthArrow}>
            <Ionicons name="chevron-forward" size={24} color={THEME_COLOR} />
          </Pressable>
        </View>

        {/* 星期标题 */}
        <View style={styles.weekHeader}>
          {WEEKDAYS.map((day, index) => (
            <View key={day} style={styles.weekCell}>
              <Text style={[
                styles.weekText,
                (index === 0 || index === 6) && styles.weekendText
              ]}>
                {day}
              </Text>
            </View>
          ))}
        </View>

        {/* 日历网格 */}
        <View style={styles.calendarGrid}>
          {days.map((item, index) => (
            <Pressable
              key={index}
              style={[
                styles.dayCell,
                !item.isCurrentMonth && styles.dayCellOther,
                isSelected(item.date) && styles.dayCellSelected,
                isToday(item.date) && styles.dayCellToday,
              ]}
              onPress={() => setSelectedDate(item.date)}
            >
              <Text style={[
                styles.dayText,
                !item.isCurrentMonth && styles.dayTextOther,
                isSelected(item.date) && styles.dayTextSelected,
                (item.date.getDay() === 0 || item.date.getDay() === 6) && styles.weekendDayText,
              ]}>
                {item.day}
              </Text>
              <Text style={[
                styles.lunarText,
                !item.isCurrentMonth && styles.lunarTextOther,
                isSelected(item.date) && styles.lunarTextSelected,
              ]}>
                {getLunarDay(item.date)}
              </Text>
            </Pressable>
          ))}
        </View>

        {/* 选中日期详情 */}
        <View style={styles.detailCard}>
          <View style={styles.detailHeader}>
            <Text style={styles.detailDate}>
              {selectedDate.getMonth() + 1}月{selectedDate.getDate()}日
            </Text>
            <Text style={styles.detailWeek}>
              星期{WEEKDAYS[selectedDate.getDay()]}
            </Text>
          </View>
          <View style={styles.detailInfo}>
            <View style={styles.detailItem}>
              <Text style={styles.detailLabel}>农历</Text>
              <Text style={styles.detailValue}>
                {LUNAR_MONTHS[selectedDate.getMonth() % 12]}月{getLunarDay(selectedDate)}
              </Text>
            </View>
            <View style={styles.detailItem}>
              <Text style={styles.detailLabel}>干支</Text>
              <Text style={styles.detailValue}>{getGanZhiYear(selectedDate)}年</Text>
            </View>
            <View style={styles.detailItem}>
              <Text style={styles.detailLabel}>生肖</Text>
              <Text style={styles.detailValue}>{getShengXiao(selectedDate)}年</Text>
            </View>
          </View>
        </View>

        {/* 宜忌 */}
        <View style={styles.yijiCard}>
          <View style={styles.yiSection}>
            <View style={styles.yiHeader}>
              <View style={styles.yiIcon}>
                <Text style={styles.yiIconText}>宜</Text>
              </View>
              <Text style={styles.yiTitle}>今日宜</Text>
            </View>
            <View style={styles.yiTags}>
              {YI_JI_DATA.yi.map((item, index) => (
                <View key={index} style={styles.yiTag}>
                  <Text style={styles.yiTagText}>{item}</Text>
                </View>
              ))}
            </View>
          </View>

          <View style={styles.yiDivider} />

          <View style={styles.jiSection}>
            <View style={styles.jiHeader}>
              <View style={styles.jiIcon}>
                <Text style={styles.jiIconText}>忌</Text>
              </View>
              <Text style={styles.jiTitle}>今日忌</Text>
            </View>
            <View style={styles.jiTags}>
              {YI_JI_DATA.ji.map((item, index) => (
                <View key={index} style={styles.jiTag}>
                  <Text style={styles.jiTagText}>{item}</Text>
                </View>
              ))}
            </View>
          </View>
        </View>

        {/* 底部提示 */}
        <View style={styles.footer}>
          <Text style={styles.footerText}>仅供参考，请理性对待</Text>
        </View>

        {/* 底部间距 */}
        <View style={{ height: 80 }} />
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="index" />
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
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backButton: {
    padding: 4,
  },
  navTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  todayBtn: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
  },
  content: {
    flex: 1,
  },
  monthHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 16,
    paddingHorizontal: 16,
    backgroundColor: '#FFF',
  },
  monthArrow: {
    padding: 8,
  },
  monthInfo: {
    alignItems: 'center',
  },
  monthText: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  lunarMonthText: {
    fontSize: 13,
    color: THEME_COLOR,
    marginTop: 4,
  },
  weekHeader: {
    flexDirection: 'row',
    backgroundColor: '#FFF',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  weekCell: {
    flex: 1,
    alignItems: 'center',
  },
  weekText: {
    fontSize: 14,
    color: '#666',
    fontWeight: '500',
  },
  weekendText: {
    color: THEME_COLOR,
  },
  calendarGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    backgroundColor: '#FFF',
    paddingHorizontal: 4,
    paddingBottom: 12,
  },
  dayCell: {
    width: '14.28%',
    aspectRatio: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 4,
  },
  dayCellOther: {
    opacity: 0.4,
  },
  dayCellSelected: {
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
  },
  dayCellToday: {
    borderWidth: 2,
    borderColor: THEME_COLOR,
    borderRadius: 8,
  },
  dayText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#333',
  },
  dayTextOther: {
    color: '#999',
  },
  dayTextSelected: {
    color: '#FFF',
  },
  weekendDayText: {
    color: THEME_COLOR,
  },
  lunarText: {
    fontSize: 10,
    color: '#999',
    marginTop: 2,
  },
  lunarTextOther: {
    color: '#CCC',
  },
  lunarTextSelected: {
    color: 'rgba(255,255,255,0.8)',
  },
  detailCard: {
    backgroundColor: '#FFF',
    margin: 12,
    borderRadius: 12,
    padding: 16,
  },
  detailHeader: {
    flexDirection: 'row',
    alignItems: 'baseline',
    marginBottom: 16,
    gap: 12,
  },
  detailDate: {
    fontSize: 24,
    fontWeight: '600',
    color: '#333',
  },
  detailWeek: {
    fontSize: 15,
    color: '#666',
  },
  detailInfo: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  detailItem: {
    alignItems: 'center',
  },
  detailLabel: {
    fontSize: 12,
    color: '#999',
    marginBottom: 4,
  },
  detailValue: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME_COLOR,
  },
  yijiCard: {
    backgroundColor: '#FFF',
    marginHorizontal: 12,
    borderRadius: 12,
    padding: 16,
  },
  yiSection: {
    marginBottom: 16,
  },
  yiHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
    gap: 8,
  },
  yiIcon: {
    width: 24,
    height: 24,
    borderRadius: 4,
    backgroundColor: '#52c41a',
    justifyContent: 'center',
    alignItems: 'center',
  },
  yiIconText: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#FFF',
  },
  yiTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#333',
  },
  yiTags: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  yiTag: {
    backgroundColor: '#F6FFED',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
    borderWidth: 1,
    borderColor: '#B7EB8F',
  },
  yiTagText: {
    fontSize: 13,
    color: '#52c41a',
  },
  yiDivider: {
    height: 1,
    backgroundColor: '#F0F0F0',
    marginBottom: 16,
  },
  jiSection: {},
  jiHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
    gap: 8,
  },
  jiIcon: {
    width: 24,
    height: 24,
    borderRadius: 4,
    backgroundColor: '#E74C3C',
    justifyContent: 'center',
    alignItems: 'center',
  },
  jiIconText: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#FFF',
  },
  jiTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#333',
  },
  jiTags: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  jiTag: {
    backgroundColor: '#FFF1F0',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
    borderWidth: 1,
    borderColor: '#FFA39E',
  },
  jiTagText: {
    fontSize: 13,
    color: '#E74C3C',
  },
  footer: {
    alignItems: 'center',
    paddingVertical: 24,
  },
  footerText: {
    fontSize: 12,
    color: '#999',
  },
});
