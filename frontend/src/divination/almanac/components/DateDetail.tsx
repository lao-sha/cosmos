/**
 * 万年历 React 组件
 * DateDetail - 日期详情面板
 */

import React from 'react';
import {
  AlmanacInfo,
  formatLunarDate,
  getFourPillars,
  getSuitableNames,
  getAvoidNames,
  getFortuneName,
  getFortuneColor,
  getConflictInfo,
  getSolarTermName,
  getJianchuName,
  WUXING,
  ZODIAC,
  CONSTELLATIONS,
} from '../index';

// 数据来源名称
const SOURCE_NAMES = ['OCW自动更新', '手动录入', '本地计算'];

interface DateDetailProps {
  date: Date;
  info: AlmanacInfo;
  onClose?: () => void;
}

export const DateDetail: React.FC<DateDetailProps> = ({ date, info, onClose }) => {
  const lunarDate = formatLunarDate(info);
  const fourPillars = getFourPillars(info);
  const suitableList = getSuitableNames(info.suitable);
  const avoidList = getAvoidNames(info.avoid);
  const fortuneName = getFortuneName(info.fortuneLevel);
  const fortuneColor = getFortuneColor(info.fortuneLevel);
  const solarTerm = getSolarTermName(info.solarTerm);
  const jianchu = getJianchuName(info.jianchu);
  const constellation = CONSTELLATIONS[info.constellation];
  const wuxing = WUXING[info.wuxing];

  const formatDateTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString('zh-CN');
  };

  return (
    <div className="date-detail">
      {/* 头部 */}
      <div className="detail-header">
        <div className="date-info">
          <span className="calendar-icon">📅</span>
          <span className="gregorian">
            {date.getFullYear()}年{date.getMonth() + 1}月{date.getDate()}日
          </span>
          <span className="lunar">农历{lunarDate}</span>
        </div>
        {onClose && (
          <button className="close-btn" onClick={onClose}>✕</button>
        )}
      </div>

      {/* 主体内容 */}
      <div className="detail-body">
        {/* 左侧：干支信息 */}
        <div className="left-column">
          <div className="section">
            <h3 className="section-title">【干支信息】</h3>
            <div className="ganzhi-list">
              <div className="ganzhi-item">
                <span className="label">年柱:</span>
                <span className="value">{fourPillars.year}</span>
                <span className="wuxing">({WUXING[info.yearTiangan % 5]}{ZODIAC[info.yearDizhi]})</span>
              </div>
              <div className="ganzhi-item">
                <span className="label">月柱:</span>
                <span className="value">{fourPillars.month}</span>
                <span className="wuxing">({WUXING[info.monthTiangan % 5]}{ZODIAC[info.monthDizhi]})</span>
              </div>
              <div className="ganzhi-item">
                <span className="label">日柱:</span>
                <span className="value">{fourPillars.day}</span>
                <span className="wuxing">({WUXING[info.dayTiangan % 5]}{ZODIAC[info.dayDizhi]})</span>
              </div>
              <div className="ganzhi-item">
                <span className="label">时柱:</span>
                <span className="value">{fourPillars.hour}</span>
                <span className="wuxing">(子时)</span>
              </div>
            </div>
          </div>

          <div className="section">
            <h3 className="section-title">【五行纳音】</h3>
            <div className="single-value">{wuxing}</div>
          </div>

          <div className="section">
            <h3 className="section-title">【神煞方位】</h3>
            <div className="info-grid">
              <div className="info-item">
                <span className="label">冲:</span>
                <span className="value">{ZODIAC[info.conflictZodiac]}</span>
              </div>
              <div className="info-item">
                <span className="label">煞:</span>
                <span className="value">{['东', '南', '西', '北'][info.shaDirection]}方</span>
              </div>
              <div className="info-item">
                <span className="label">建除:</span>
                <span className="value">{jianchu}</span>
              </div>
              <div className="info-item">
                <span className="label">二十八宿:</span>
                <span className="value">{constellation}</span>
              </div>
            </div>
          </div>

          <div className="section">
            <h3 className="section-title">【吉凶评级】</h3>
            <div className="fortune-display" style={{ color: fortuneColor }}>
              {'⭐'.repeat(5 - info.fortuneLevel)} {fortuneName}
            </div>
          </div>
        </div>

        {/* 右侧：宜忌和其他 */}
        <div className="right-column">
          <div className="section">
            <h3 className="section-title suitable-title">【宜】</h3>
            <div className="activity-list suitable">
              {suitableList.map((item, i) => (
                <span key={i} className="activity-item">✅ {item}</span>
              ))}
              {suitableList.length === 0 && <span className="empty">诸事不宜</span>}
            </div>
          </div>

          <div className="section">
            <h3 className="section-title avoid-title">【忌】</h3>
            <div className="activity-list avoid">
              {avoidList.map((item, i) => (
                <span key={i} className="activity-item">❌ {item}</span>
              ))}
              {avoidList.length === 0 && <span className="empty">百无禁忌</span>}
            </div>
          </div>

          <div className="section">
            <h3 className="section-title">【节气节日】</h3>
            <div className="festival-info">
              <div className="info-row">
                <span className="icon">🌿</span>
                <span>{solarTerm || '无节气'}</span>
              </div>
              <div className="info-row">
                <span className="icon">🎊</span>
                <span>{info.festivals > 0 ? '有节日' : '无'}</span>
              </div>
            </div>
          </div>

          <div className="section">
            <h3 className="section-title">【数据来源】</h3>
            <div className="source-info">
              <div className="info-row">
                <span className="icon">📡</span>
                <span>{SOURCE_NAMES[info.source]}</span>
              </div>
              <div className="info-row">
                <span className="icon">🕐</span>
                <span>{formatDateTime(info.updatedAt)}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <style jsx>{`
        .date-detail {
          background: #fff;
          border: 1px solid #d2691e;
          border-radius: 12px;
          overflow: hidden;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .detail-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px 20px;
          background: linear-gradient(135deg, #8b0000, #a52a2a);
          color: white;
        }

        .date-info {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .calendar-icon {
          font-size: 24px;
        }

        .gregorian {
          font-size: 18px;
          font-weight: 600;
        }

        .lunar {
          font-size: 14px;
          opacity: 0.9;
        }

        .close-btn {
          background: rgba(255, 255, 255, 0.2);
          border: none;
          color: white;
          width: 32px;
          height: 32px;
          border-radius: 50%;
          cursor: pointer;
          font-size: 16px;
          transition: background 0.2s;
        }

        .close-btn:hover {
          background: rgba(255, 255, 255, 0.3);
        }

        .detail-body {
          display: flex;
          padding: 20px;
          gap: 24px;
        }

        .left-column,
        .right-column {
          flex: 1;
        }

        .section {
          margin-bottom: 20px;
        }

        .section-title {
          font-size: 14px;
          color: #8b0000;
          margin: 0 0 12px 0;
          font-weight: 600;
        }

        .suitable-title {
          color: #228b22;
        }

        .avoid-title {
          color: #dc143c;
        }

        .ganzhi-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .ganzhi-item {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .ganzhi-item .label {
          color: #696969;
          min-width: 50px;
        }

        .ganzhi-item .value {
          font-weight: 600;
          color: #2f4f4f;
        }

        .ganzhi-item .wuxing {
          font-size: 12px;
          color: #888;
        }

        .single-value {
          font-size: 18px;
          font-weight: 600;
          color: #daa520;
        }

        .info-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 8px;
        }

        .info-item {
          display: flex;
          gap: 6px;
        }

        .info-item .label {
          color: #696969;
        }

        .info-item .value {
          color: #2f4f4f;
          font-weight: 500;
        }

        .fortune-display {
          font-size: 18px;
          font-weight: 600;
        }

        .activity-list {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        }

        .activity-item {
          font-size: 13px;
          padding: 4px 8px;
          border-radius: 4px;
        }

        .suitable .activity-item {
          background: rgba(34, 139, 34, 0.1);
          color: #228b22;
        }

        .avoid .activity-item {
          background: rgba(220, 20, 60, 0.1);
          color: #dc143c;
        }

        .empty {
          color: #888;
          font-style: italic;
        }

        .festival-info,
        .source-info {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .info-row {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 14px;
          color: #696969;
        }

        .info-row .icon {
          font-size: 16px;
        }

        @media (max-width: 768px) {
          .detail-body {
            flex-direction: column;
          }
        }
      `}</style>
    </div>
  );
};

export default DateDetail;
