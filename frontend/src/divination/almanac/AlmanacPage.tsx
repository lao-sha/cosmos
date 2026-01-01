/**
 * 万年历主页面组件
 * AlmanacPage - 整合所有子组件
 */

import React, { useState, useEffect, useCallback } from 'react';
import TodayCard from './components/TodayCard';
import CalendarGrid from './components/CalendarGrid';
import DateDetail from './components/DateDetail';
import AuspiciousFinder from './components/AuspiciousFinder';
import {
  AlmanacInfo,
  DateKey,
  getAlmanac,
  getMonthAlmanac,
  initApi,
} from './index';

type TabType = 'calendar' | 'finder' | 'terms';

export const AlmanacPage: React.FC = () => {
  // 当前日期状态
  const today = new Date();
  const [currentYear, setCurrentYear] = useState(today.getFullYear());
  const [currentMonth, setCurrentMonth] = useState(today.getMonth() + 1);
  const [selectedDay, setSelectedDay] = useState<number | null>(today.getDate());

  // 数据状态
  const [todayInfo, setTodayInfo] = useState<AlmanacInfo | null>(null);
  const [monthData, setMonthData] = useState<Map<number, AlmanacInfo>>(new Map());
  const [selectedDayInfo, setSelectedDayInfo] = useState<AlmanacInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [apiReady, setApiReady] = useState(false);

  // UI状态
  const [activeTab, setActiveTab] = useState<TabType>('calendar');
  const [showDetail, setShowDetail] = useState(false);

  // 初始化 API
  useEffect(() => {
    const init = async () => {
      try {
        await initApi();
        setApiReady(true);
      } catch (error) {
        console.error('API 初始化失败:', error);
      }
    };
    init();
  }, []);

  // 加载今日数据
  useEffect(() => {
    if (!apiReady) return;

    const loadToday = async () => {
      const info = await getAlmanac(
        today.getFullYear(),
        today.getMonth() + 1,
        today.getDate()
      );
      setTodayInfo(info);
    };

    loadToday();
  }, [apiReady]);

  // 加载月度数据
  useEffect(() => {
    if (!apiReady) return;

    const loadMonth = async () => {
      setLoading(true);
      const data = await getMonthAlmanac(currentYear, currentMonth);
      setMonthData(data);
      setLoading(false);
    };

    loadMonth();
  }, [apiReady, currentYear, currentMonth]);

  // 加载选中日期数据
  useEffect(() => {
    if (selectedDay) {
      const info = monthData.get(selectedDay);
      setSelectedDayInfo(info || null);
    } else {
      setSelectedDayInfo(null);
    }
  }, [selectedDay, monthData]);

  // 月份切换
  const handleMonthChange = useCallback((year: number, month: number) => {
    setCurrentYear(year);
    setCurrentMonth(month);
    setSelectedDay(null);
  }, []);

  // 日期选择
  const handleDaySelect = useCallback((day: number) => {
    setSelectedDay(day);
    setShowDetail(true);
  }, []);

  // 从择吉日选择
  const handleAuspiciousDaySelect = useCallback((date: DateKey, info: AlmanacInfo) => {
    const [year, month, day] = date;
    setCurrentYear(year);
    setCurrentMonth(month);
    setSelectedDay(day);
    setSelectedDayInfo(info);
    setActiveTab('calendar');
    setShowDetail(true);
  }, []);

  // 渲染加载状态
  if (!apiReady) {
    return (
      <div className="almanac-loading">
        <div className="spinner" />
        <span>正在连接区块链...</span>
      </div>
    );
  }

  return (
    <div className="almanac-page">
      {/* 页面标题 */}
      <header className="page-header">
        <h1>📅 万年历</h1>
        <p>链上黄历 · 择吉问卜</p>
      </header>

      {/* 主内容区 */}
      <main className="page-content">
        {/* 左侧：今日黄历 */}
        <aside className="today-section">
          {todayInfo ? (
            <TodayCard
              date={today}
              info={todayInfo}
              onDateClick={() => {
                setCurrentYear(today.getFullYear());
                setCurrentMonth(today.getMonth() + 1);
                setSelectedDay(today.getDate());
                setShowDetail(true);
              }}
            />
          ) : (
            <div className="today-loading">
              <div className="spinner-small" />
              <span>加载今日黄历...</span>
            </div>
          )}
        </aside>

        {/* 右侧：功能区 */}
        <section className="main-section">
          {/* 标签页切换 */}
          <nav className="tab-nav">
            <button
              className={`tab-btn ${activeTab === 'calendar' ? 'active' : ''}`}
              onClick={() => setActiveTab('calendar')}
            >
              📆 月历视图
            </button>
            <button
              className={`tab-btn ${activeTab === 'finder' ? 'active' : ''}`}
              onClick={() => setActiveTab('finder')}
            >
              🔮 择吉查询
            </button>
            <button
              className={`tab-btn ${activeTab === 'terms' ? 'active' : ''}`}
              onClick={() => setActiveTab('terms')}
            >
              🌿 节气时间
            </button>
          </nav>

          {/* 标签页内容 */}
          <div className="tab-content">
            {activeTab === 'calendar' && (
              <div className="calendar-view">
                <CalendarGrid
                  year={currentYear}
                  month={currentMonth}
                  monthData={monthData}
                  selectedDay={selectedDay}
                  onDaySelect={handleDaySelect}
                  onMonthChange={handleMonthChange}
                  loading={loading}
                />
              </div>
            )}

            {activeTab === 'finder' && (
              <AuspiciousFinder onDaySelect={handleAuspiciousDaySelect} />
            )}

            {activeTab === 'terms' && (
              <div className="solar-terms-view">
                <h3>🌿 {currentYear}年二十四节气</h3>
                <p className="coming-soon">功能开发中...</p>
              </div>
            )}
          </div>
        </section>
      </main>

      {/* 日期详情弹窗 */}
      {showDetail && selectedDayInfo && (
        <div className="detail-modal" onClick={() => setShowDetail(false)}>
          <div className="detail-wrapper" onClick={(e) => e.stopPropagation()}>
            <DateDetail
              date={new Date(currentYear, currentMonth - 1, selectedDay || 1)}
              info={selectedDayInfo}
              onClose={() => setShowDetail(false)}
            />
          </div>
        </div>
      )}

      <style jsx>{`
        .almanac-page {
          min-height: 100vh;
          background: linear-gradient(180deg, #faf8f5, #f5f0e8);
        }

        .almanac-loading {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          min-height: 100vh;
          gap: 16px;
          color: #8b0000;
        }

        .spinner {
          width: 48px;
          height: 48px;
          border: 4px solid #f3f3f3;
          border-top: 4px solid #8b0000;
          border-radius: 50%;
          animation: spin 1s linear infinite;
        }

        .spinner-small {
          width: 24px;
          height: 24px;
          border: 3px solid #f3f3f3;
          border-top: 3px solid #8b0000;
          border-radius: 50%;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }

        .page-header {
          text-align: center;
          padding: 32px 20px 24px;
          background: linear-gradient(135deg, #8b0000, #a52a2a);
          color: white;
        }

        .page-header h1 {
          margin: 0;
          font-size: 28px;
          font-weight: 700;
        }

        .page-header p {
          margin: 8px 0 0;
          opacity: 0.9;
          font-size: 14px;
        }

        .page-content {
          display: flex;
          gap: 24px;
          padding: 24px;
          max-width: 1400px;
          margin: 0 auto;
        }

        .today-section {
          flex-shrink: 0;
        }

        .today-loading {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          width: 320px;
          height: 400px;
          background: #fff;
          border-radius: 12px;
          gap: 12px;
          color: #696969;
        }

        .main-section {
          flex: 1;
          min-width: 0;
        }

        .tab-nav {
          display: flex;
          gap: 8px;
          margin-bottom: 16px;
        }

        .tab-btn {
          padding: 10px 20px;
          border: none;
          background: #fff;
          border-radius: 8px;
          font-size: 14px;
          cursor: pointer;
          transition: all 0.2s;
          color: #696969;
        }

        .tab-btn:hover {
          background: #f5f0e8;
        }

        .tab-btn.active {
          background: #8b0000;
          color: white;
        }

        .tab-content {
          min-height: 500px;
        }

        .calendar-view {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .solar-terms-view {
          background: #fff;
          border-radius: 12px;
          padding: 24px;
          text-align: center;
        }

        .solar-terms-view h3 {
          margin: 0 0 16px;
          color: #228b22;
        }

        .coming-soon {
          color: #696969;
          font-style: italic;
        }

        .detail-modal {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
          padding: 20px;
        }

        .detail-wrapper {
          max-width: 800px;
          width: 100%;
          max-height: 90vh;
          overflow-y: auto;
        }

        @media (max-width: 1024px) {
          .page-content {
            flex-direction: column;
          }

          .today-section {
            display: flex;
            justify-content: center;
          }
        }

        @media (max-width: 640px) {
          .page-header {
            padding: 20px 16px;
          }

          .page-header h1 {
            font-size: 22px;
          }

          .page-content {
            padding: 16px;
          }

          .tab-nav {
            flex-wrap: wrap;
          }

          .tab-btn {
            flex: 1;
            min-width: 100px;
            padding: 8px 12px;
            font-size: 13px;
          }
        }
      `}</style>
    </div>
  );
};

export default AlmanacPage;
