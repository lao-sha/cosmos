/**
 * 万年历 React 组件
 * AuspiciousFinder - 择吉日查询组件
 */

import React, { useState } from 'react';
import {
  AlmanacInfo,
  AuspiciousQuery,
  SuitableItem,
  DateKey,
  SUITABLE_ITEMS,
  ZODIAC,
  FORTUNE_LEVELS,
  formatLunarDate,
  getFourPillars,
  getFortuneName,
  getFortuneColor,
  findAuspiciousDays,
} from '../index';

interface AuspiciousFinderProps {
  onDaySelect?: (date: DateKey, info: AlmanacInfo) => void;
}

export const AuspiciousFinder: React.FC<AuspiciousFinderProps> = ({ onDaySelect }) => {
  const today = new Date();
  const nextMonth = new Date(today.getFullYear(), today.getMonth() + 3, today.getDate());

  // 查询条件状态
  const [startDate, setStartDate] = useState(today.toISOString().split('T')[0]);
  const [endDate, setEndDate] = useState(nextMonth.toISOString().split('T')[0]);
  const [mustSuitable, setMustSuitable] = useState<SuitableItem[]>([]);
  const [mustNotAvoid, setMustNotAvoid] = useState<SuitableItem[]>([]);
  const [fortuneLevels, setFortuneLevels] = useState<number[]>([0, 1]);
  const [avoidZodiacs, setAvoidZodiacs] = useState<number[]>([]);

  // 查询结果
  const [results, setResults] = useState<Array<{ date: DateKey; info: AlmanacInfo }>>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);

  // 切换活动选择
  const toggleActivity = (
    item: SuitableItem,
    list: SuitableItem[],
    setList: (l: SuitableItem[]) => void
  ) => {
    if (list.includes(item)) {
      setList(list.filter(i => i !== item));
    } else {
      setList([...list, item]);
    }
  };

  // 切换吉凶等级
  const toggleFortune = (level: number) => {
    if (fortuneLevels.includes(level)) {
      setFortuneLevels(fortuneLevels.filter(l => l !== level));
    } else {
      setFortuneLevels([...fortuneLevels, level]);
    }
  };

  // 切换生肖
  const toggleZodiac = (zodiac: number) => {
    if (avoidZodiacs.includes(zodiac)) {
      setAvoidZodiacs(avoidZodiacs.filter(z => z !== zodiac));
    } else {
      setAvoidZodiacs([...avoidZodiacs, zodiac]);
    }
  };

  // 执行查询
  const handleSearch = async () => {
    setLoading(true);
    setSearched(true);

    const start = startDate.split('-').map(Number) as DateKey;
    const end = endDate.split('-').map(Number) as DateKey;

    const query: AuspiciousQuery = {
      startDate: start,
      endDate: end,
      mustSuitable,
      mustNotAvoid,
      fortuneLevels,
      avoidZodiacs,
    };

    try {
      const foundDays = await findAuspiciousDays(query);
      setResults(foundDays);
    } catch (error) {
      console.error('查询失败:', error);
      setResults([]);
    }

    setLoading(false);
  };

  // 格式化日期显示
  const formatDate = (dateKey: DateKey) => {
    const [year, month, day] = dateKey;
    return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  };

  return (
    <div className="auspicious-finder">
      <div className="finder-header">
        <span className="icon">🔮</span>
        <h2>择吉日查询</h2>
      </div>

      <div className="finder-body">
        {/* 日期范围 */}
        <div className="form-section">
          <label className="section-label">查询条件:</label>
          <div className="date-range">
            <span>日期范围:</span>
            <input
              type="date"
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
            />
            <span>至</span>
            <input
              type="date"
              value={endDate}
              onChange={(e) => setEndDate(e.target.value)}
            />
          </div>
        </div>

        {/* 必须宜 */}
        <div className="form-section">
          <label className="section-label">必须宜:</label>
          <div className="checkbox-grid">
            {SUITABLE_ITEMS.slice(0, 12).map((name, i) => (
              <label key={i} className="checkbox-item">
                <input
                  type="checkbox"
                  checked={mustSuitable.includes(i as SuitableItem)}
                  onChange={() => toggleActivity(i as SuitableItem, mustSuitable, setMustSuitable)}
                />
                <span>{name}</span>
              </label>
            ))}
          </div>
        </div>

        {/* 不能忌 */}
        <div className="form-section">
          <label className="section-label">不能忌:</label>
          <div className="checkbox-grid">
            {['嫁娶', '动土', '安葬', '破土', '出行', '开市'].map((name, i) => {
              const itemIndex = SUITABLE_ITEMS.indexOf(name as any);
              return (
                <label key={i} className="checkbox-item">
                  <input
                    type="checkbox"
                    checked={mustNotAvoid.includes(itemIndex as SuitableItem)}
                    onChange={() => toggleActivity(itemIndex as SuitableItem, mustNotAvoid, setMustNotAvoid)}
                  />
                  <span>{name}</span>
                </label>
              );
            })}
          </div>
        </div>

        {/* 吉凶要求 */}
        <div className="form-section">
          <label className="section-label">吉凶要求:</label>
          <div className="checkbox-grid">
            {FORTUNE_LEVELS.map((name, i) => (
              <label key={i} className="checkbox-item">
                <input
                  type="checkbox"
                  checked={fortuneLevels.includes(i)}
                  onChange={() => toggleFortune(i)}
                />
                <span style={{ color: getFortuneColor(i) }}>{name}</span>
              </label>
            ))}
          </div>
        </div>

        {/* 冲煞避免 */}
        <div className="form-section">
          <label className="section-label">冲煞避免:</label>
          <div className="checkbox-grid zodiac-grid">
            {ZODIAC.map((name, i) => (
              <label key={i} className="checkbox-item">
                <input
                  type="checkbox"
                  checked={avoidZodiacs.includes(i)}
                  onChange={() => toggleZodiac(i)}
                />
                <span>{name}</span>
              </label>
            ))}
          </div>
        </div>

        {/* 查询按钮 */}
        <button className="search-btn" onClick={handleSearch} disabled={loading}>
          {loading ? '查询中...' : '🔍 查询吉日'}
        </button>
      </div>

      {/* 查询结果 */}
      {searched && (
        <div className="finder-results">
          <div className="results-header">
            查询结果 {!loading && `(共找到 ${results.length} 个吉日)`}
          </div>

          {loading ? (
            <div className="loading">
              <div className="spinner" />
              <span>正在查询...</span>
            </div>
          ) : results.length > 0 ? (
            <div className="results-list">
              {results.map(({ date, info }, i) => (
                <div
                  key={i}
                  className="result-item"
                  onClick={() => onDaySelect?.(date, info)}
                >
                  <div className="result-date">
                    <span className="icon">📅</span>
                    <span className="date">{formatDate(date)}</span>
                    <span className="lunar">({formatLunarDate(info)})</span>
                    <span
                      className="fortune"
                      style={{ color: getFortuneColor(info.fortuneLevel) }}
                    >
                      ⭐{getFortuneName(info.fortuneLevel)}
                    </span>
                  </div>
                  <div className="result-info">
                    <span className="ganzhi">
                      干支: {getFourPillars(info).year}年 {getFourPillars(info).month}月 {getFourPillars(info).day}日
                    </span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="no-results">
              <span className="icon">😔</span>
              <span>未找到符合条件的吉日，请调整查询条件</span>
            </div>
          )}
        </div>
      )}

      <style jsx>{`
        .auspicious-finder {
          background: #fff;
          border: 1px solid #e0e0e0;
          border-radius: 12px;
          overflow: hidden;
        }

        .finder-header {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 16px 20px;
          background: linear-gradient(135deg, #daa520, #cd853f);
          color: white;
        }

        .finder-header .icon {
          font-size: 24px;
        }

        .finder-header h2 {
          margin: 0;
          font-size: 18px;
          font-weight: 600;
        }

        .finder-body {
          padding: 20px;
        }

        .form-section {
          margin-bottom: 20px;
        }

        .section-label {
          display: block;
          font-weight: 600;
          color: #2f4f4f;
          margin-bottom: 10px;
        }

        .date-range {
          display: flex;
          align-items: center;
          gap: 12px;
          flex-wrap: wrap;
        }

        .date-range input {
          padding: 8px 12px;
          border: 1px solid #ddd;
          border-radius: 6px;
          font-size: 14px;
        }

        .checkbox-grid {
          display: flex;
          flex-wrap: wrap;
          gap: 12px;
        }

        .zodiac-grid {
          gap: 8px;
        }

        .checkbox-item {
          display: flex;
          align-items: center;
          gap: 6px;
          cursor: pointer;
          font-size: 14px;
        }

        .checkbox-item input {
          cursor: pointer;
        }

        .search-btn {
          width: 100%;
          padding: 14px;
          background: linear-gradient(135deg, #8b0000, #a52a2a);
          color: white;
          border: none;
          border-radius: 8px;
          font-size: 16px;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
        }

        .search-btn:hover:not(:disabled) {
          background: linear-gradient(135deg, #a52a2a, #cd5c5c);
          transform: translateY(-1px);
        }

        .search-btn:disabled {
          opacity: 0.7;
          cursor: not-allowed;
        }

        .finder-results {
          border-top: 1px solid #e0e0e0;
        }

        .results-header {
          padding: 12px 20px;
          background: #f5f5f5;
          font-weight: 600;
          color: #696969;
        }

        .loading {
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: 40px;
          gap: 12px;
          color: #696969;
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

        .results-list {
          max-height: 400px;
          overflow-y: auto;
        }

        .result-item {
          padding: 16px 20px;
          border-bottom: 1px solid #f0f0f0;
          cursor: pointer;
          transition: background 0.2s;
        }

        .result-item:hover {
          background: #fafafa;
        }

        .result-item:last-child {
          border-bottom: none;
        }

        .result-date {
          display: flex;
          align-items: center;
          gap: 10px;
          margin-bottom: 6px;
        }

        .result-date .icon {
          font-size: 16px;
        }

        .result-date .date {
          font-weight: 600;
          color: #2f4f4f;
        }

        .result-date .lunar {
          color: #8b0000;
          font-size: 14px;
        }

        .result-date .fortune {
          font-weight: 600;
          margin-left: auto;
        }

        .result-info {
          font-size: 13px;
          color: #696969;
          padding-left: 26px;
        }

        .no-results {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 12px;
          padding: 40px;
          color: #696969;
        }

        .no-results .icon {
          font-size: 32px;
        }
      `}</style>
    </div>
  );
};

export default AuspiciousFinder;
