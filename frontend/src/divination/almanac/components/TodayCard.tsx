/**
 * 万年历 React 组件
 * TodayCard - 今日黄历卡片
 */

import React from 'react';
import {
  AlmanacInfo,
  formatLunarDate,
  formatLunarYear,
  getFourPillars,
  getSuitableNames,
  getAvoidNames,
  getFortuneName,
  getFortuneColor,
  getConflictInfo,
  getSolarTermName,
  ZODIAC,
  ZODIAC_EMOJI,
} from '../index';

// 吉凶星级
const FORTUNE_STARS: Record<number, string> = {
  0: '⭐⭐⭐⭐⭐',
  1: '⭐⭐⭐⭐',
  2: '⭐⭐⭐',
  3: '⭐⭐',
  4: '⭐',
};

interface TodayCardProps {
  date: Date;
  info: AlmanacInfo;
  onDateClick?: () => void;
}

export const TodayCard: React.FC<TodayCardProps> = ({ date, info, onDateClick }) => {
  const weekdays = ['日', '一', '二', '三', '四', '五', '六'];
  const weekday = weekdays[date.getDay()];

  const lunarDate = formatLunarDate(info);
  const lunarYear = formatLunarYear(info);
  const fourPillars = getFourPillars(info);
  const suitableList = getSuitableNames(info.suitable).slice(0, 6);
  const avoidList = getAvoidNames(info.avoid).slice(0, 6);
  const conflictInfo = getConflictInfo(info);
  const fortuneName = getFortuneName(info.fortuneLevel);
  const fortuneColor = getFortuneColor(info.fortuneLevel);
  const solarTerm = getSolarTermName(info.solarTerm);

  return (
    <div className="today-card" onClick={onDateClick}>
      {/* 日期头部 */}
      <div className="card-header">
        <div className="gregorian-date">
          {date.getFullYear()}年{date.getMonth() + 1}月{date.getDate()}日 星期{weekday}
        </div>
        {solarTerm && <div className="solar-term-badge">🌿 {solarTerm}</div>}
      </div>

      <div className="card-divider" />

      {/* 农历信息 */}
      <div className="lunar-section">
        <div className="lunar-date">{lunarDate}</div>
        <div className="ganzhi-year">
          {fourPillars.year}年 {fourPillars.month}月 {fourPillars.day}日
        </div>
      </div>

      {/* 生肖图标 */}
      <div className="zodiac-section">
        <div className="zodiac-emoji">{ZODIAC_EMOJI[info.zodiac]}</div>
        <div className="zodiac-name">{ZODIAC[info.zodiac]}年</div>
      </div>

      {/* 宜忌表格 */}
      <div className="suitable-avoid-table">
        <div className="table-column suitable-column">
          <div className="column-header">宜</div>
          <div className="column-content">
            {suitableList.map((item, i) => (
              <span key={i} className="item suitable-item">
                {item}
              </span>
            ))}
          </div>
        </div>
        <div className="table-divider" />
        <div className="table-column avoid-column">
          <div className="column-header">忌</div>
          <div className="column-content">
            {avoidList.map((item, i) => (
              <span key={i} className="item avoid-item">
                {item}
              </span>
            ))}
          </div>
        </div>
      </div>

      {/* 冲煞和吉凶 */}
      <div className="card-footer">
        <div className="conflict-info">{conflictInfo}</div>
        <div className="fortune-info" style={{ color: fortuneColor }}>
          {FORTUNE_STARS[info.fortuneLevel]} {fortuneName}
        </div>
      </div>

      <style jsx>{`
        .today-card {
          background: linear-gradient(145deg, #fff8dc, #faebd7);
          border: 2px solid #d2691e;
          border-radius: 12px;
          padding: 20px;
          width: 320px;
          box-shadow: 0 4px 12px rgba(139, 0, 0, 0.15);
          cursor: pointer;
          transition: transform 0.2s, box-shadow 0.2s;
        }

        .today-card:hover {
          transform: translateY(-2px);
          box-shadow: 0 6px 16px rgba(139, 0, 0, 0.2);
        }

        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .gregorian-date {
          font-size: 16px;
          color: #2f4f4f;
          font-weight: 600;
        }

        .solar-term-badge {
          background: #228b22;
          color: white;
          padding: 4px 8px;
          border-radius: 12px;
          font-size: 12px;
        }

        .card-divider {
          height: 2px;
          background: linear-gradient(to right, transparent, #d2691e, transparent);
          margin: 12px 0;
        }

        .lunar-section {
          text-align: center;
          margin-bottom: 16px;
        }

        .lunar-date {
          font-size: 24px;
          color: #8b0000;
          font-weight: bold;
          margin-bottom: 4px;
        }

        .ganzhi-year {
          font-size: 14px;
          color: #696969;
        }

        .zodiac-section {
          text-align: center;
          margin-bottom: 16px;
        }

        .zodiac-emoji {
          font-size: 48px;
          margin-bottom: 4px;
        }

        .zodiac-name {
          font-size: 14px;
          color: #8b0000;
          font-weight: 500;
        }

        .suitable-avoid-table {
          display: flex;
          border: 1px solid #d2691e;
          border-radius: 8px;
          overflow: hidden;
          margin-bottom: 16px;
        }

        .table-column {
          flex: 1;
          padding: 12px;
        }

        .table-divider {
          width: 1px;
          background: #d2691e;
        }

        .column-header {
          font-size: 14px;
          font-weight: bold;
          text-align: center;
          margin-bottom: 8px;
          padding-bottom: 6px;
          border-bottom: 1px dashed #d2691e;
        }

        .suitable-column .column-header {
          color: #228b22;
        }

        .avoid-column .column-header {
          color: #dc143c;
        }

        .column-content {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
          justify-content: center;
        }

        .item {
          font-size: 13px;
          padding: 2px 6px;
          border-radius: 4px;
        }

        .suitable-item {
          color: #228b22;
          background: rgba(34, 139, 34, 0.1);
        }

        .avoid-item {
          color: #dc143c;
          background: rgba(220, 20, 60, 0.1);
        }

        .card-footer {
          display: flex;
          justify-content: space-between;
          align-items: center;
          font-size: 14px;
        }

        .conflict-info {
          color: #696969;
        }

        .fortune-info {
          font-weight: 600;
        }
      `}</style>
    </div>
  );
};

export default TodayCard;
