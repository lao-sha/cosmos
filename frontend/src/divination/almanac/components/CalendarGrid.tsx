/**
 * 万年历 React 组件
 * CalendarGrid - 月历网格组件
 */

import React, { useMemo } from 'react';
import {
  AlmanacInfo,
  getFortuneColor,
  getSolarTermName,
  LUNAR_DAYS,
  getDaysInMonth,
  getFirstDayOfMonth,
} from '../index';

interface CalendarGridProps {
  year: number;
  month: number;
  monthData: Map<number, AlmanacInfo>;
  selectedDay: number | null;
  onDaySelect: (day: number) => void;
  onMonthChange: (year: number, month: number) => void;
  loading?: boolean;
}

export const CalendarGrid: React.FC<CalendarGridProps> = ({
  year,
  month,
  monthData,
  selectedDay,
  onDaySelect,
  onMonthChange,
  loading = false,
}) => {
  const weekdays = ['一', '二', '三', '四', '五', '六', '日'];

  // 计算日历格子
  const calendarDays = useMemo(() => {
    const daysInMonth = getDaysInMonth(year, month);
    const firstDay = getFirstDayOfMonth(year, month);
    // 调整为周一开始 (0=周一, 6=周日)
    const startOffset = firstDay === 0 ? 6 : firstDay - 1;

    const days: Array<{ day: number; isCurrentMonth: boolean }> = [];

    // 上月占位
    const prevMonthDays = getDaysInMonth(year, month === 1 ? 12 : month - 1);
    for (let i = startOffset - 1; i >= 0; i--) {
      days.push({ day: prevMonthDays - i, isCurrentMonth: false });
    }

    // 当月日期
    for (let i = 1; i <= daysInMonth; i++) {
      days.push({ day: i, isCurrentMonth: true });
    }

    // 下月占位
    const remaining = 42 - days.length;
    for (let i = 1; i <= remaining; i++) {
      days.push({ day: i, isCurrentMonth: false });
    }

    return days;
  }, [year, month]);

  // 月份切换
  const handlePrevMonth = () => {
    if (month === 1) {
      onMonthChange(year - 1, 12);
    } else {
      onMonthChange(year, month - 1);
    }
  };

  const handleNextMonth = () => {
    if (month === 12) {
      onMonthChange(year + 1, 1);
    } else {
      onMonthChange(year, month + 1);
    }
  };

  const handleToday = () => {
    const today = new Date();
    onMonthChange(today.getFullYear(), today.getMonth() + 1);
  };

  // 渲染日期单元格
  const renderDayCell = (dayInfo: { day: number; isCurrentMonth: boolean }, index: number) => {
    const { day, isCurrentMonth } = dayInfo;

    if (!isCurrentMonth) {
      return (
        <div key={index} className="day-cell disabled">
          <span className="day-number">{day}</span>
        </div>
      );
    }

    const almanacInfo = monthData.get(day);
    const isSelected = selectedDay === day;
    const isToday =
      year === new Date().getFullYear() &&
      month === new Date().getMonth() + 1 &&
      day === new Date().getDate();

    const lunarDay = almanacInfo ? LUNAR_DAYS[almanacInfo.lunarDay - 1] : '';
    const fortuneColor = almanacInfo ? getFortuneColor(almanacInfo.fortuneLevel) : '#ccc';
    const solarTerm = almanacInfo ? getSolarTermName(almanacInfo.solarTerm) : '';

    return (
      <div
        key={index}
        className={`day-cell ${isSelected ? 'selected' : ''} ${isToday ? 'today' : ''}`}
        onClick={() => onDaySelect(day)}
      >
        <span className="day-number">{day}</span>
        <span className="lunar-day">{solarTerm || lunarDay}</span>
        {almanacInfo && (
          <span
            className="fortune-dot"
            style={{ backgroundColor: fortuneColor }}
            title={solarTerm ? `节气: ${solarTerm}` : undefined}
          />
        )}
        {solarTerm && <span className="solar-term-indicator">🌿</span>}
      </div>
    );
  };

  return (
    <div className="calendar-grid">
      {/* 头部导航 */}
      <div className="calendar-header">
        <button className="nav-btn" onClick={handlePrevMonth}>◀</button>
        <div className="current-month">
          <span className="year">{year}年</span>
          <span className="month">{month}月</span>
        </div>
        <button className="nav-btn" onClick={handleNextMonth}>▶</button>
        <button className="today-btn" onClick={handleToday}>今日</button>
      </div>

      {/* 星期标题 */}
      <div className="weekday-row">
        {weekdays.map((day, i) => (
          <div key={i} className={`weekday ${i >= 5 ? 'weekend' : ''}`}>
            {day}
          </div>
        ))}
      </div>

      {/* 日历网格 */}
      <div className={`days-grid ${loading ? 'loading' : ''}`}>
        {loading && (
          <div className="loading-overlay">
            <div className="spinner" />
            <span>加载中...</span>
          </div>
        )}
        {calendarDays.map((dayInfo, index) => renderDayCell(dayInfo, index))}
      </div>

      {/* 图例 */}
      <div className="legend">
        <div className="legend-item">
          <span className="dot" style={{ backgroundColor: '#228B22' }} /> 大吉/吉
        </div>
        <div className="legend-item">
          <span className="dot" style={{ backgroundColor: '#FFD700' }} /> 平
        </div>
        <div className="legend-item">
          <span className="dot" style={{ backgroundColor: '#DC143C' }} /> 凶/大凶
        </div>
        <div className="legend-item">
          <span className="icon">🌿</span> 节气
        </div>
      </div>

      <style jsx>{`
        .calendar-grid {
          background: #fff;
          border: 1px solid #e0e0e0;
          border-radius: 12px;
          padding: 16px;
          min-width: 400px;
        }

        .calendar-header {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 12px;
          margin-bottom: 16px;
        }

        .nav-btn {
          background: #f5f5f5;
          border: 1px solid #ddd;
          border-radius: 6px;
          padding: 8px 12px;
          cursor: pointer;
          font-size: 14px;
          transition: all 0.2s;
        }

        .nav-btn:hover {
          background: #e8e8e8;
        }

        .today-btn {
          background: #8b0000;
          color: white;
          border: none;
          border-radius: 6px;
          padding: 8px 16px;
          cursor: pointer;
          font-size: 14px;
          margin-left: 16px;
          transition: all 0.2s;
        }

        .today-btn:hover {
          background: #a52a2a;
        }

        .current-month {
          font-size: 20px;
          font-weight: 600;
          color: #2f4f4f;
          min-width: 120px;
          text-align: center;
        }

        .current-month .year {
          margin-right: 8px;
        }

        .weekday-row {
          display: grid;
          grid-template-columns: repeat(7, 1fr);
          gap: 2px;
          margin-bottom: 8px;
        }

        .weekday {
          text-align: center;
          padding: 8px;
          font-weight: 600;
          color: #696969;
          font-size: 14px;
        }

        .weekday.weekend {
          color: #8b0000;
        }

        .days-grid {
          display: grid;
          grid-template-columns: repeat(7, 1fr);
          gap: 2px;
          position: relative;
        }

        .days-grid.loading {
          opacity: 0.5;
          pointer-events: none;
        }

        .loading-overlay {
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 8px;
          z-index: 10;
        }

        .spinner {
          width: 32px;
          height: 32px;
          border: 3px solid #f3f3f3;
          border-top: 3px solid #8b0000;
          border-radius: 50%;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }

        .day-cell {
          aspect-ratio: 1;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 4px;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s;
          position: relative;
          background: #fafafa;
          min-height: 60px;
        }

        .day-cell:hover {
          background: #f0e68c;
        }

        .day-cell.disabled {
          opacity: 0.3;
          cursor: default;
          background: transparent;
        }

        .day-cell.disabled:hover {
          background: transparent;
        }

        .day-cell.selected {
          background: #daa520;
          color: white;
        }

        .day-cell.today {
          border: 2px solid #8b0000;
        }

        .day-number {
          font-size: 16px;
          font-weight: 600;
        }

        .lunar-day {
          font-size: 11px;
          color: #696969;
          margin-top: 2px;
        }

        .day-cell.selected .lunar-day {
          color: rgba(255, 255, 255, 0.9);
        }

        .fortune-dot {
          width: 6px;
          height: 6px;
          border-radius: 50%;
          position: absolute;
          bottom: 4px;
          left: 50%;
          transform: translateX(-50%);
        }

        .solar-term-indicator {
          position: absolute;
          top: 2px;
          right: 2px;
          font-size: 10px;
        }

        .legend {
          display: flex;
          justify-content: center;
          gap: 20px;
          margin-top: 16px;
          padding-top: 12px;
          border-top: 1px solid #e0e0e0;
        }

        .legend-item {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 12px;
          color: #696969;
        }

        .legend-item .dot {
          width: 10px;
          height: 10px;
          border-radius: 50%;
        }

        .legend-item .icon {
          font-size: 12px;
        }
      `}</style>
    </div>
  );
};

export default CalendarGrid;
