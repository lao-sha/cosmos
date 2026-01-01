/**
 * Stardust 万年历模块
 * 基于 almanac pallet 数据结构设计
 */

// ============ 常量定义 ============

// 天干
export const TIANGAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'] as const;

// 地支
export const DIZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'] as const;

// 生肖
export const ZODIAC = ['鼠', '牛', '虎', '兔', '龙', '蛇', '马', '羊', '猴', '鸡', '狗', '猪'] as const;

// 五行
export const WUXING = ['金', '木', '水', '火', '土'] as const;

// 方位
export const DIRECTION = ['东', '南', '西', '北'] as const;

// 吉凶等级
export const FORTUNE_LEVELS = ['大吉', '吉', '平', '凶', '大凶'] as const;

// 建除十二神
export const JIANCHU = ['建', '除', '满', '平', '定', '执', '破', '危', '成', '收', '开', '闭'] as const;

// 二十四节气
export const SOLAR_TERMS = [
  '无', '立春', '雨水', '惊蛰', '春分', '清明', '谷雨',
  '立夏', '小满', '芒种', '夏至', '小暑', '大暑',
  '立秋', '处暑', '白露', '秋分', '寒露', '霜降',
  '立冬', '小雪', '大雪', '冬至', '小寒', '大寒'
] as const;

// 农历月份
export const LUNAR_MONTHS = [
  '正月', '二月', '三月', '四月', '五月', '六月',
  '七月', '八月', '九月', '十月', '冬月', '腊月'
] as const;

// 农历日期
export const LUNAR_DAYS = [
  '初一', '初二', '初三', '初四', '初五', '初六', '初七', '初八', '初九', '初十',
  '十一', '十二', '十三', '十四', '十五', '十六', '十七', '十八', '十九', '二十',
  '廿一', '廿二', '廿三', '廿四', '廿五', '廿六', '廿七', '廿八', '廿九', '三十'
] as const;

// 宜忌活动 (32种)
export const SUITABLE_ITEMS = [
  '嫁娶', '纳采', '祭祀', '祈福', '出行', '动土', '破土', '安葬',
  '开市', '交易', '立券', '移徙', '修造', '栽种', '纳财', '开光',
  '安床', '入宅', '安门', '求嗣', '解除', '求医', '词讼', '沐浴',
  '理发', '扫舍', '会友', '上梁', '竖柱', '纳畜', '伐木', '作灶'
] as const;

// 二十八宿
export const CONSTELLATIONS = [
  '角', '亢', '氐', '房', '心', '尾', '箕',
  '斗', '牛', '女', '虚', '危', '室', '壁',
  '奎', '娄', '胃', '昴', '毕', '觜', '参',
  '井', '鬼', '柳', '星', '张', '翼', '轸'
] as const;

// 生肖 Emoji
export const ZODIAC_EMOJI = ['🐭', '🐮', '🐯', '🐰', '🐲', '🐍', '🐴', '🐏', '🐵', '🐔', '🐶', '🐷'] as const;

// 宜忌活动枚举
export enum SuitableItem {
  Marriage = 0,      // 嫁娶
  Betrothal = 1,     // 纳采
  Sacrifice = 2,     // 祭祀
  Prayer = 3,        // 祈福
  Travel = 4,        // 出行
  Groundbreaking = 5,// 动土
  Excavation = 6,    // 破土
  Burial = 7,        // 安葬
  OpenBusiness = 8,  // 开市
  Trading = 9,       // 交易
  Contract = 10,     // 立券
  Moving = 11,       // 移徙
  Renovation = 12,   // 修造
  Planting = 13,     // 栽种
  ReceiveMoney = 14, // 纳财
  Consecration = 15, // 开光
  PlaceBed = 16,     // 安床
  EnterHouse = 17,   // 入宅
  InstallDoor = 18,  // 安门
  PrayForChildren = 19, // 求嗣
  Remove = 20,       // 解除
  SeekMedical = 21,  // 求医
  Lawsuit = 22,      // 词讼
  Bathing = 23,      // 沐浴
  Haircut = 24,      // 理发
  Cleaning = 25,     // 扫舍
  MeetFriends = 26,  // 会友
  RaiseBeam = 27,    // 上梁
  ErectPillar = 28,  // 竖柱
  RaiseLivestock = 29, // 纳畜
  Logging = 30,      // 伐木
  BuildStove = 31,   // 作灶
}

// ============ 类型定义 ============

// 日期键
export type DateKey = [number, number, number]; // [year, month, day]

// 黄历信息 (对应链上 AlmanacInfo)
export interface AlmanacInfo {
  // 农历
  lunarYear: number;
  lunarMonth: number;  // 1-12, 13-24为闰月
  lunarDay: number;

  // 干支
  yearTiangan: number;   // 0-9
  yearDizhi: number;     // 0-11
  monthTiangan: number;
  monthDizhi: number;
  dayTiangan: number;
  dayDizhi: number;
  hourTiangan: number;
  hourDizhi: number;

  // 生肖五行
  zodiac: number;          // 0-11
  conflictZodiac: number;  // 0-11
  shaDirection: number;    // 0-3 (东南西北)
  wuxing: number;          // 0-4 (金木水火土)
  jianchu: number;         // 0-11 (建除十二神)
  constellation: number;   // 0-27 (二十八宿)

  // 宜忌 (位图)
  suitable: bigint;  // u64
  avoid: bigint;     // u64

  // 节气节日
  solarTerm: number;    // 0-24
  festivals: number;    // u32 位图
  fortuneLevel: number; // 0-4

  // 元数据
  updatedAt: number;  // Unix timestamp
  source: number;     // 0: OCW, 1: Manual, 2: Calculated
}

// 四柱
export interface FourPillars {
  year: string;   // 年柱
  month: string;  // 月柱
  day: string;    // 日柱
  hour: string;   // 时柱
}

// 择吉查询条件
export interface AuspiciousQuery {
  startDate: DateKey;
  endDate: DateKey;
  mustSuitable: SuitableItem[];   // 必须宜
  mustNotAvoid: SuitableItem[];   // 不能忌
  fortuneLevels: number[];         // 吉凶要求
  avoidZodiacs: number[];          // 避免冲的生肖
}

// ============ 工具函数 ============

/**
 * 解析宜忌位图
 */
export function parseSuitableItems(bitmap: bigint): SuitableItem[] {
  const items: SuitableItem[] = [];
  for (let i = 0; i < 32; i++) {
    if ((bitmap >> BigInt(i)) & BigInt(1)) {
      items.push(i as SuitableItem);
    }
  }
  return items;
}

/**
 * 获取宜活动名称列表
 */
export function getSuitableNames(bitmap: bigint): string[] {
  return parseSuitableItems(bitmap).map(i => SUITABLE_ITEMS[i]);
}

/**
 * 获取忌活动名称列表
 */
export function getAvoidNames(bitmap: bigint): string[] {
  return parseSuitableItems(bitmap).map(i => SUITABLE_ITEMS[i]);
}

/**
 * 获取干支名称
 */
export function getGanZhi(tiangan: number, dizhi: number): string {
  return TIANGAN[tiangan] + DIZHI[dizhi];
}

/**
 * 获取四柱
 */
export function getFourPillars(info: AlmanacInfo): FourPillars {
  return {
    year: getGanZhi(info.yearTiangan, info.yearDizhi),
    month: getGanZhi(info.monthTiangan, info.monthDizhi),
    day: getGanZhi(info.dayTiangan, info.dayDizhi),
    hour: getGanZhi(info.hourTiangan, info.hourDizhi),
  };
}

/**
 * 干支对应五行
 */
export function getGanWuxing(tiangan: number): string {
  const ganWuxing = ['木', '木', '火', '火', '土', '土', '金', '金', '水', '水'];
  return ganWuxing[tiangan];
}

/**
 * 格式化农历日期
 */
export function formatLunarDate(info: AlmanacInfo): string {
  const isLeap = info.lunarMonth > 12;
  const month = isLeap ? info.lunarMonth - 12 : info.lunarMonth;
  const monthName = (isLeap ? '闰' : '') + LUNAR_MONTHS[month - 1];
  const dayName = LUNAR_DAYS[info.lunarDay - 1];
  return `${monthName}${dayName}`;
}

/**
 * 格式化农历年份
 */
export function formatLunarYear(info: AlmanacInfo): string {
  const ganZhi = getGanZhi(info.yearTiangan, info.yearDizhi);
  const zodiac = ZODIAC[info.zodiac];
  return `${ganZhi}年 (${zodiac}年)`;
}

/**
 * 获取吉凶等级名称
 */
export function getFortuneName(level: number): string {
  return FORTUNE_LEVELS[level] || '未知';
}

/**
 * 获取吉凶等级颜色
 */
export function getFortuneColor(level: number): string {
  const colors = ['#228B22', '#32CD32', '#FFD700', '#FF6347', '#DC143C'];
  return colors[level] || '#888888';
}

/**
 * 获取生肖名称
 */
export function getZodiacName(zodiac: number): string {
  return ZODIAC[zodiac];
}

/**
 * 获取冲煞信息
 */
export function getConflictInfo(info: AlmanacInfo): string {
  const conflictZodiac = ZODIAC[info.conflictZodiac];
  const direction = DIRECTION[info.shaDirection];
  return `冲${conflictZodiac} 煞${direction}`;
}

/**
 * 获取节气名称
 */
export function getSolarTermName(term: number): string {
  return term > 0 && term <= 24 ? SOLAR_TERMS[term] : '';
}

/**
 * 获取建除名称
 */
export function getJianchuName(jianchu: number): string {
  return JIANCHU[jianchu];
}

/**
 * 检查日期是否符合择吉条件
 */
export function matchesAuspiciousQuery(info: AlmanacInfo, query: AuspiciousQuery): boolean {
  // 检查吉凶等级
  if (query.fortuneLevels.length > 0 && !query.fortuneLevels.includes(info.fortuneLevel)) {
    return false;
  }

  // 检查必须宜的活动
  for (const item of query.mustSuitable) {
    if (!((info.suitable >> BigInt(item)) & BigInt(1))) {
      return false;
    }
  }

  // 检查不能忌的活动
  for (const item of query.mustNotAvoid) {
    if ((info.avoid >> BigInt(item)) & BigInt(1)) {
      return false;
    }
  }

  // 检查避免冲的生肖
  if (query.avoidZodiacs.includes(info.conflictZodiac)) {
    return false;
  }

  return true;
}

/**
 * 获取月份天数
 */
export function getDaysInMonth(year: number, month: number): number {
  return new Date(year, month, 0).getDate();
}

/**
 * 获取月份第一天是星期几 (0=周日, 1=周一, ...)
 */
export function getFirstDayOfMonth(year: number, month: number): number {
  return new Date(year, month - 1, 1).getDay();
}

// ============ API 接口 ============

import { ApiPromise, WsProvider } from '@polkadot/api';

let api: ApiPromise | null = null;

/**
 * 初始化 API 连接
 */
export async function initApi(wsEndpoint: string = 'ws://127.0.0.1:9944'): Promise<ApiPromise> {
  if (api && api.isConnected) {
    return api;
  }

  const provider = new WsProvider(wsEndpoint);
  api = await ApiPromise.create({ provider });
  await api.isReady;
  return api;
}

/**
 * 获取 API 实例
 */
export function getApi(): ApiPromise | null {
  return api;
}

/**
 * 将链上数据解析为 AlmanacInfo
 */
function parseChainAlmanacInfo(data: any): AlmanacInfo | null {
  if (!data || data.isNone) {
    return null;
  }

  const info = data.unwrap ? data.unwrap() : data;

  return {
    lunarYear: info.lunarYear?.toNumber?.() ?? info.lunarYear ?? 0,
    lunarMonth: info.lunarMonth?.toNumber?.() ?? info.lunarMonth ?? 0,
    lunarDay: info.lunarDay?.toNumber?.() ?? info.lunarDay ?? 0,
    yearTiangan: info.yearTiangan?.toNumber?.() ?? info.yearTiangan ?? 0,
    yearDizhi: info.yearDizhi?.toNumber?.() ?? info.yearDizhi ?? 0,
    monthTiangan: info.monthTiangan?.toNumber?.() ?? info.monthTiangan ?? 0,
    monthDizhi: info.monthDizhi?.toNumber?.() ?? info.monthDizhi ?? 0,
    dayTiangan: info.dayTiangan?.toNumber?.() ?? info.dayTiangan ?? 0,
    dayDizhi: info.dayDizhi?.toNumber?.() ?? info.dayDizhi ?? 0,
    hourTiangan: info.hourTiangan?.toNumber?.() ?? info.hourTiangan ?? 0,
    hourDizhi: info.hourDizhi?.toNumber?.() ?? info.hourDizhi ?? 0,
    zodiac: info.zodiac?.toNumber?.() ?? info.zodiac ?? 0,
    conflictZodiac: info.conflictZodiac?.toNumber?.() ?? info.conflictZodiac ?? 0,
    shaDirection: info.shaDirection?.toNumber?.() ?? info.shaDirection ?? 0,
    wuxing: info.wuxing?.toNumber?.() ?? info.wuxing ?? 0,
    jianchu: info.jianchu?.toNumber?.() ?? info.jianchu ?? 0,
    constellation: info.constellation?.toNumber?.() ?? info.constellation ?? 0,
    suitable: BigInt(info.suitable?.toString?.() ?? info.suitable ?? '0'),
    avoid: BigInt(info.avoid?.toString?.() ?? info.avoid ?? '0'),
    solarTerm: info.solarTerm?.toNumber?.() ?? info.solarTerm ?? 0,
    festivals: info.festivals?.toNumber?.() ?? info.festivals ?? 0,
    fortuneLevel: info.fortuneLevel?.toNumber?.() ?? info.fortuneLevel ?? 0,
    updatedAt: info.updatedAt?.toNumber?.() ?? info.updatedAt ?? 0,
    source: info.source?.toNumber?.() ?? info.source ?? 0,
  };
}

/**
 * 查询单日黄历
 */
export async function getAlmanac(year: number, month: number, day: number): Promise<AlmanacInfo | null> {
  if (!api) {
    console.warn('API 未初始化');
    return null;
  }

  try {
    const result = await (api.query as any).almanac?.almanacData?.([year, month, day]);
    return parseChainAlmanacInfo(result);
  } catch (error) {
    console.error('查询黄历失败:', error);
    return null;
  }
}

/**
 * 查询整月黄历
 */
export async function getMonthAlmanac(year: number, month: number): Promise<Map<number, AlmanacInfo>> {
  const monthData = new Map<number, AlmanacInfo>();

  if (!api) {
    console.warn('API 未初始化');
    return monthData;
  }

  const daysInMonth = getDaysInMonth(year, month);

  try {
    // 批量查询整月数据
    const queries = [];
    for (let day = 1; day <= daysInMonth; day++) {
      queries.push((api.query as any).almanac?.almanacData?.([year, month, day]));
    }

    const results = await Promise.all(queries);

    results.forEach((result, index) => {
      const info = parseChainAlmanacInfo(result);
      if (info) {
        monthData.set(index + 1, info);
      }
    });
  } catch (error) {
    console.error('查询月度黄历失败:', error);
  }

  return monthData;
}

/**
 * 择吉日查询
 */
export async function findAuspiciousDays(
  query: AuspiciousQuery
): Promise<Array<{ date: DateKey; info: AlmanacInfo }>> {
  const results: Array<{ date: DateKey; info: AlmanacInfo }> = [];

  if (!api) {
    console.warn('API 未初始化');
    return results;
  }

  const [startYear, startMonth, startDay] = query.startDate;
  const [endYear, endMonth, endDay] = query.endDate;

  let currentDate = new Date(startYear, startMonth - 1, startDay);
  const endDate = new Date(endYear, endMonth - 1, endDay);

  while (currentDate <= endDate) {
    const year = currentDate.getFullYear();
    const month = currentDate.getMonth() + 1;
    const day = currentDate.getDate();

    const info = await getAlmanac(year, month, day);

    if (info && matchesAuspiciousQuery(info, query)) {
      results.push({
        date: [year, month, day] as DateKey,
        info,
      });
    }

    // 移到下一天
    currentDate.setDate(currentDate.getDate() + 1);
  }

  return results;
}

// ============ 组件导出 ============

export { TodayCard } from './components/TodayCard';
export { CalendarGrid } from './components/CalendarGrid';
export { DateDetail } from './components/DateDetail';
export { AuspiciousFinder } from './components/AuspiciousFinder';
export { RealtimeClockWeb, type FourPillarsInfo } from './components/RealtimeClockWeb';
export { AlmanacPage } from './AlmanacPage';
