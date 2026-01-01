/**
 * 实时时钟干支组件 (React Native 版本)
 * RealtimeClock - 显示模拟时钟、四柱干支、农历公历
 */

import React, { useState, useEffect, useCallback } from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import Svg, { Circle, Line } from 'react-native-svg';

// 天干
const TIANGAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];

// 地支
const DIZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

// 农历月份
const LUNAR_MONTHS = ['正月', '二月', '三月', '四月', '五月', '六月', '七月', '八月', '九月', '十月', '冬月', '腊月'];

// 农历日期
const LUNAR_DAYS = [
  '初一', '初二', '初三', '初四', '初五', '初六', '初七', '初八', '初九', '初十',
  '十一', '十二', '十三', '十四', '十五', '十六', '十七', '十八', '十九', '二十',
  '廿一', '廿二', '廿三', '廿四', '廿五', '廿六', '廿七', '廿八', '廿九', '三十'
];

// 时辰名称
const SHICHEN_NAMES = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

// 四柱信息
export interface FourPillarsInfo {
  yearGan: number;
  yearZhi: number;
  monthGan: number;
  monthZhi: number;
  dayGan: number;
  dayZhi: number;
  hourGan: number;
  hourZhi: number;
}

// 农历信息
interface LunarInfo {
  year: number;
  month: number;
  day: number;
  isLeap: boolean;
}

interface RealtimeClockProps {
  onStartDivination?: (timestamp: Date, fourPillars: FourPillarsInfo) => void;
  buttonText?: string;
}

/**
 * 根据小时获取时辰索引 (0-11)
 */
function getShichenIndex(hour: number): number {
  return Math.floor(((hour + 1) % 24) / 2);
}

/**
 * 计算时辰天干
 */
function getHourGan(dayGan: number, hourZhi: number): number {
  const startGan = [0, 2, 4, 6, 8, 0, 2, 4, 6, 8];
  return (startGan[dayGan] + hourZhi) % 10;
}

/**
 * 简化的农历计算
 */
function calculateLunarDate(date: Date): LunarInfo {
  const year = date.getFullYear();
  const month = date.getMonth() + 1;
  const day = date.getDate();

  let lunarMonth = month - 1;
  let lunarYear = year;

  if (lunarMonth <= 0) {
    lunarMonth = 12 + lunarMonth;
    lunarYear = year - 1;
  }

  return {
    year: lunarYear,
    month: lunarMonth || 11,
    day: day > 30 ? 30 : day,
    isLeap: false,
  };
}

/**
 * 计算年柱干支
 */
function calculateYearPillar(year: number): { gan: number; zhi: number } {
  const offset = year - 1984;
  const gan = ((offset % 10) + 10) % 10;
  const zhi = ((offset % 12) + 12) % 12;
  return { gan, zhi };
}

/**
 * 计算月柱干支
 */
function calculateMonthPillar(year: number, month: number, yearGan: number): { gan: number; zhi: number } {
  const monthZhi = (month + 1) % 12;
  const startGan = [2, 4, 6, 8, 0, 2, 4, 6, 8, 0];
  const monthGan = (startGan[yearGan] + month - 1) % 10;
  return { gan: monthGan, zhi: monthZhi };
}

/**
 * 计算日柱干支
 */
function calculateDayPillar(date: Date): { gan: number; zhi: number } {
  const baseDate = new Date(2000, 0, 1);
  const diffDays = Math.floor((date.getTime() - baseDate.getTime()) / (24 * 60 * 60 * 1000));
  const gan = ((diffDays % 10) + 0 + 10) % 10;
  const zhi = ((diffDays % 12) + 6 + 12) % 12;
  return { gan, zhi };
}

/**
 * 计算完整四柱
 */
function calculateFourPillars(date: Date): FourPillarsInfo {
  const year = date.getFullYear();
  const month = date.getMonth() + 1;
  const hour = date.getHours();

  const yearPillar = calculateYearPillar(year);
  const monthPillar = calculateMonthPillar(year, month, yearPillar.gan);
  const dayPillar = calculateDayPillar(date);
  const hourZhi = getShichenIndex(hour);
  const hourGan = getHourGan(dayPillar.gan, hourZhi);

  return {
    yearGan: yearPillar.gan,
    yearZhi: yearPillar.zhi,
    monthGan: monthPillar.gan,
    monthZhi: monthPillar.zhi,
    dayGan: dayPillar.gan,
    dayZhi: dayPillar.zhi,
    hourGan,
    hourZhi,
  };
}

/**
 * SVG 模拟时钟组件
 */
const AnalogClock: React.FC<{ date: Date; size?: number }> = ({ date, size = 90 }) => {
  const hours = date.getHours();
  const minutes = date.getMinutes();
  const seconds = date.getSeconds();

  const center = size / 2;
  const radius = size / 2 - 8;

  const secondAngle = (seconds / 60) * 360 - 90;
  const minuteAngle = ((minutes + seconds / 60) / 60) * 360 - 90;
  const hourAngle = ((hours % 12 + minutes / 60) / 12) * 360 - 90;

  const getEndPoint = (angle: number, length: number) => {
    const rad = (angle * Math.PI) / 180;
    return {
      x: center + length * Math.cos(rad),
      y: center + length * Math.sin(rad),
    };
  };

  const hourEnd = getEndPoint(hourAngle, radius * 0.5);
  const minuteEnd = getEndPoint(minuteAngle, radius * 0.7);
  const secondEnd = getEndPoint(secondAngle, radius * 0.85);

  const ticks = [];
  for (let i = 0; i < 12; i++) {
    const angle = (i / 12) * 360 - 90;
    const rad = (angle * Math.PI) / 180;
    const innerR = radius * 0.85;
    const outerR = radius * 0.95;
    ticks.push({
      x1: center + innerR * Math.cos(rad),
      y1: center + innerR * Math.sin(rad),
      x2: center + outerR * Math.cos(rad),
      y2: center + outerR * Math.sin(rad),
      isMain: i % 3 === 0,
    });
  }

  return (
    <Svg width={size} height={size}>
      <Circle
        cx={center}
        cy={center}
        r={radius}
        fill="#faf8f5"
        stroke="#d2b48c"
        strokeWidth={2}
      />
      {ticks.map((tick, i) => (
        <Line
          key={i}
          x1={tick.x1}
          y1={tick.y1}
          x2={tick.x2}
          y2={tick.y2}
          stroke="#8b7355"
          strokeWidth={tick.isMain ? 2 : 1}
        />
      ))}
      <Line
        x1={center}
        y1={center}
        x2={hourEnd.x}
        y2={hourEnd.y}
        stroke="#4a4a4a"
        strokeWidth={3}
        strokeLinecap="round"
      />
      <Line
        x1={center}
        y1={center}
        x2={minuteEnd.x}
        y2={minuteEnd.y}
        stroke="#4a4a4a"
        strokeWidth={2}
        strokeLinecap="round"
      />
      <Line
        x1={center}
        y1={center}
        x2={secondEnd.x}
        y2={secondEnd.y}
        stroke="#8b0000"
        strokeWidth={1}
        strokeLinecap="round"
      />
      <Circle cx={center} cy={center} r={3} fill="#4a4a4a" />
    </Svg>
  );
};

/**
 * 实时时钟干支组件
 */
export const RealtimeClock: React.FC<RealtimeClockProps> = ({
  onStartDivination,
  buttonText = '即时起局',
}) => {
  const [currentTime, setCurrentTime] = useState(new Date());
  const [fourPillars, setFourPillars] = useState<FourPillarsInfo>(() =>
    calculateFourPillars(new Date())
  );
  const [lunarDate, setLunarDate] = useState<LunarInfo>(() =>
    calculateLunarDate(new Date())
  );

  useEffect(() => {
    const timer = setInterval(() => {
      const now = new Date();
      setCurrentTime(now);
      setFourPillars(calculateFourPillars(now));
      setLunarDate(calculateLunarDate(now));
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  const handleStartDivination = useCallback(() => {
    onStartDivination?.(currentTime, fourPillars);
  }, [currentTime, fourPillars, onStartDivination]);

  const formatGregorian = (date: Date): string => {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    const seconds = String(date.getSeconds()).padStart(2, '0');
    return `${year}/${month}/${day} ${hours}:${minutes}:${seconds}`;
  };

  const formatLunar = (): string => {
    const monthName = lunarDate.isLeap ? '闰' : '';
    const monthStr = LUNAR_MONTHS[lunarDate.month - 1] || '冬月';
    const dayStr = LUNAR_DAYS[lunarDate.day - 1] || '十三';
    const shichen = SHICHEN_NAMES[fourPillars.hourZhi];
    return `${lunarDate.year}年${monthName}${monthStr}${dayStr} ${shichen}时`;
  };

  return (
    <View style={styles.container}>
      {/* 左侧：模拟时钟 */}
      <View style={styles.clockSection}>
        <AnalogClock date={currentTime} size={90} />
      </View>

      {/* 中间：四柱干支 */}
      <View style={styles.pillarsSection}>
        {/* 天干行 */}
        <View style={styles.ganzhiRow}>
          <Text style={styles.ganText}>{TIANGAN[fourPillars.yearGan]}</Text>
          <Text style={styles.ganText}>{TIANGAN[fourPillars.monthGan]}</Text>
          <Text style={styles.ganText}>{TIANGAN[fourPillars.dayGan]}</Text>
          <Text style={styles.ganText}>{TIANGAN[fourPillars.hourGan]}</Text>
        </View>

        {/* 地支行 */}
        <View style={styles.ganzhiRow}>
          <Text style={styles.zhiText}>{DIZHI[fourPillars.yearZhi]}</Text>
          <Text style={styles.zhiText}>{DIZHI[fourPillars.monthZhi]}</Text>
          <Text style={styles.zhiText}>{DIZHI[fourPillars.dayZhi]}</Text>
          <Text style={styles.zhiText}>{DIZHI[fourPillars.hourZhi]}</Text>
        </View>

        {/* 日期信息 */}
        <View style={styles.dateInfo}>
          <Text style={styles.lunarText}>农历: {formatLunar()}</Text>
          <Text style={styles.gregorianText}>公历: {formatGregorian(currentTime)}</Text>
        </View>
      </View>

      {/* 右侧：起局按钮 */}
      <TouchableOpacity
        style={styles.startButton}
        onPress={handleStartDivination}
        activeOpacity={0.8}
      >
        {buttonText.split('').map((char, i) => (
          <Text key={i} style={styles.buttonChar}>{char}</Text>
        ))}
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#faf8f5',
    borderRadius: 8,
    padding: 12,
    borderWidth: 1,
    borderColor: '#e8e0d5',
  },
  clockSection: {
    marginRight: 12,
  },
  pillarsSection: {
    flex: 1,
  },
  ganzhiRow: {
    flexDirection: 'row',
    marginBottom: 2,
  },
  ganText: {
    fontSize: 20,
    fontWeight: '600',
    color: '#4a4a4a',
    width: 28,
    textAlign: 'center',
  },
  zhiText: {
    fontSize: 16,
    color: '#696969',
    width: 28,
    textAlign: 'center',
  },
  dateInfo: {
    marginTop: 8,
  },
  lunarText: {
    fontSize: 12,
    color: '#696969',
  },
  gregorianText: {
    fontSize: 12,
    color: '#999',
  },
  startButton: {
    backgroundColor: '#f5f0e8',
    paddingVertical: 8,
    paddingHorizontal: 10,
    borderRadius: 4,
    borderWidth: 1,
    borderColor: '#d2b48c',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 80,
  },
  buttonChar: {
    fontSize: 14,
    color: '#8b7355',
    lineHeight: 18,
  },
});

export default RealtimeClock;
