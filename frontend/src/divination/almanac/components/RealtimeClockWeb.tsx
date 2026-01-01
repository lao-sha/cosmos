/**
 * 实时时钟干支组件 (Web版本)
 * RealtimeClockWeb - 显示模拟时钟、四柱干支、农历公历
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  TIANGAN,
  DIZHI,
  LUNAR_MONTHS,
  LUNAR_DAYS,
} from '../almanac';

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
const AnalogClock: React.FC<{ date: Date; size?: number }> = ({ date, size = 100 }) => {
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
    <svg width={size} height={size}>
      {/* 表盘背景 */}
      <circle
        cx={center}
        cy={center}
        r={radius}
        fill="#faf8f5"
        stroke="#d2b48c"
        strokeWidth={2}
      />

      {/* 刻度 */}
      {ticks.map((tick, i) => (
        <line
          key={i}
          x1={tick.x1}
          y1={tick.y1}
          x2={tick.x2}
          y2={tick.y2}
          stroke="#8b7355"
          strokeWidth={tick.isMain ? 2 : 1}
        />
      ))}

      {/* 时针 */}
      <line
        x1={center}
        y1={center}
        x2={hourEnd.x}
        y2={hourEnd.y}
        stroke="#4a4a4a"
        strokeWidth={3}
        strokeLinecap="round"
      />

      {/* 分针 */}
      <line
        x1={center}
        y1={center}
        x2={minuteEnd.x}
        y2={minuteEnd.y}
        stroke="#4a4a4a"
        strokeWidth={2}
        strokeLinecap="round"
      />

      {/* 秒针 */}
      <line
        x1={center}
        y1={center}
        x2={secondEnd.x}
        y2={secondEnd.y}
        stroke="#8b0000"
        strokeWidth={1}
        strokeLinecap="round"
      />

      {/* 中心点 */}
      <circle cx={center} cy={center} r={3} fill="#4a4a4a" />
    </svg>
  );
};

/**
 * 实时时钟干支组件
 */
export const RealtimeClockWeb: React.FC<RealtimeClockProps> = ({
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
    <div className="realtime-clock">
      {/* 左侧：模拟时钟 */}
      <div className="clock-section">
        <AnalogClock date={currentTime} size={90} />
      </div>

      {/* 中间：四柱干支 */}
      <div className="pillars-section">
        {/* 天干行 */}
        <div className="ganzhi-row">
          <span className="gan-text">{TIANGAN[fourPillars.yearGan]}</span>
          <span className="gan-text">{TIANGAN[fourPillars.monthGan]}</span>
          <span className="gan-text">{TIANGAN[fourPillars.dayGan]}</span>
          <span className="gan-text">{TIANGAN[fourPillars.hourGan]}</span>
        </div>

        {/* 地支行 */}
        <div className="ganzhi-row">
          <span className="zhi-text">{DIZHI[fourPillars.yearZhi]}</span>
          <span className="zhi-text">{DIZHI[fourPillars.monthZhi]}</span>
          <span className="zhi-text">{DIZHI[fourPillars.dayZhi]}</span>
          <span className="zhi-text">{DIZHI[fourPillars.hourZhi]}</span>
        </div>

        {/* 日期信息 */}
        <div className="date-info">
          <div className="lunar-text">农历: {formatLunar()}</div>
          <div className="gregorian-text">公历: {formatGregorian(currentTime)}</div>
        </div>
      </div>

      {/* 右侧：起局按钮 */}
      <button
        className="start-button"
        onClick={handleStartDivination}
      >
        {buttonText.split('').map((char, i) => (
          <span key={i}>{char}</span>
        ))}
      </button>

      <style jsx>{`
        .realtime-clock {
          display: flex;
          align-items: center;
          background: #faf8f5;
          border-radius: 8px;
          padding: 12px;
          border: 1px solid #e8e0d5;
          gap: 12px;
        }

        .clock-section {
          flex-shrink: 0;
        }

        .pillars-section {
          flex: 1;
        }

        .ganzhi-row {
          display: flex;
          gap: 4px;
          margin-bottom: 2px;
        }

        .gan-text {
          font-size: 20px;
          font-weight: 600;
          color: #4a4a4a;
          width: 28px;
          text-align: center;
        }

        .zhi-text {
          font-size: 16px;
          color: #696969;
          width: 28px;
          text-align: center;
        }

        .date-info {
          margin-top: 8px;
        }

        .lunar-text {
          font-size: 12px;
          color: #696969;
        }

        .gregorian-text {
          font-size: 12px;
          color: #999;
        }

        .start-button {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          background: #f5f0e8;
          padding: 8px 6px;
          border-radius: 4px;
          border: 1px solid #d2b48c;
          cursor: pointer;
          transition: all 0.2s;
          font-size: 14px;
          color: #8b7355;
          line-height: 1.4;
          min-height: 80px;
        }

        .start-button:hover {
          background: #ebe5db;
          border-color: #c4a67c;
        }

        .start-button:active {
          transform: scale(0.98);
        }

        .start-button span {
          display: block;
        }
      `}</style>
    </div>
  );
};

export default RealtimeClockWeb;
