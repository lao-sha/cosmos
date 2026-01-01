# 八字排盘链端接口设计文档

## 📋 文档概述

本文档详细描述八字排盘系统链端 Runtime API 的接口设计，重点说明如何返回以下关键字段给前端：

- **主星**（天干地支十神）
- **藏干**（地支藏干及其十神）
- **副星**（藏干十神）
- **星运**（十二长生）
- **空亡**（旬空）
- **纳音**（六十甲子纳音）
- **神煞**（吉凶神煞）

## 🎯 接口设计原则

### 1. 分层计算架构

```
┌─────────────────────────────────────────────────┐
│          前端 DApp (Mobile First)               │
│  React + TypeScript + Ant Design + Polkadot.js │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│        Runtime API (免费，实时计算)              │
│  - get_full_bazi_chart(chart_id) -> FullChart  │
│  - 无 gas 费用                                   │
│  - 响应快速 (< 100ms)                           │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│         链上存储 (按需缓存)                      │
│  - ChartById: 基础四柱数据                       │
│  - InterpretationCache: 可选解盘缓存            │
└─────────────────────────────────────────────────┘
```

### 2. 存储优化策略

- **基础数据**：存储四柱干支、出生时间、性别等必要字段（约 200 bytes）
- **计算字段**：主星、藏干、副星、星运、空亡、纳音、神煞均通过 Runtime API 实时计算
- **可选缓存**：用户可选择缓存解盘结果到链上（13 bytes 核心指标）

## 📊 数据结构设计

### 1. 完整八字命盘响应结构

```rust
/// 完整八字命盘（包含所有计算字段）
pub struct FullBaziChart {
    /// 命盘ID
    pub chart_id: u64,
    /// 所有者账户
    pub owner: AccountId,
    /// 出生时间
    pub birth_time: BirthTime,
    /// 性别
    pub gender: Gender,
    /// 四柱信息（包含主星、藏干、纳音）
    pub sizhu: EnhancedSiZhu,
    /// 大运信息
    pub dayun: DaYunInfo,
    /// 空亡信息
    pub kongwang: KongWangInfo,
    /// 神煞列表
    pub shensha_list: Vec<ShenShaEntry>,
    /// 星运（十二长生）
    pub xingyun: XingYunInfo,
    /// 五行强度
    pub wuxing_strength: WuXingStrength,
    /// 喜用神
    pub xiyong_shen: Option<WuXing>,
    /// 创建时间戳
    pub timestamp: u64,
}
```

### 2. 增强四柱结构（EnhancedSiZhu）

```rust
/// 增强四柱结构（包含所有分析字段）
pub struct EnhancedSiZhu {
    /// 年柱详情
    pub year_zhu: EnhancedZhu,
    /// 月柱详情
    pub month_zhu: EnhancedZhu,
    /// 日柱详情
    pub day_zhu: EnhancedZhu,
    /// 时柱详情
    pub hour_zhu: EnhancedZhu,
    /// 日主天干
    pub rizhu: TianGan,
}

/// 增强单柱结构
pub struct EnhancedZhu {
    /// 干支组合
    pub ganzhi: GanZhi,

    // ========== 主星 ==========
    /// 天干十神（主星）
    pub tiangan_shishen: ShiShen,
    /// 地支本气十神（主星）
    pub dizhi_benqi_shishen: ShiShen,

    // ========== 藏干（副星）==========
    /// 藏干详细信息
    pub canggan_list: Vec<CangGanInfo>,

    // ========== 纳音 ==========
    /// 纳音五行
    pub nayin: NaYin,

    // ========== 星运（十二长生）==========
    /// 日主在该地支的十二长生状态
    pub changsheng: ShiErChangSheng,
}

/// 藏干信息（包含副星）
pub struct CangGanInfo {
    /// 藏干天干
    pub gan: TianGan,
    /// 藏干十神（副星）
    pub shishen: ShiShen,
    /// 藏干类型（主气/中气/余气）
    pub canggan_type: CangGanType,
    /// 权重（用于五行强度计算）
    pub weight: u16,
}
```

### 3. 空亡信息结构（KongWangInfo）

```rust
/// 空亡信息
pub struct KongWangInfo {
    /// 年柱旬空
    pub year_kongwang: (DiZhi, DiZhi),
    /// 月柱旬空
    pub month_kongwang: (DiZhi, DiZhi),
    /// 日柱旬空（最重要）
    pub day_kongwang: (DiZhi, DiZhi),
    /// 时柱旬空
    pub hour_kongwang: (DiZhi, DiZhi),

    /// 四柱是否落空亡（实际应用）
    pub year_is_kong: bool,
    pub month_is_kong: bool,
    pub day_is_kong: bool,
    pub hour_is_kong: bool,
}
```

### 4. 神煞信息结构（ShenShaEntry）

```rust
/// 神煞条目
pub struct ShenShaEntry {
    /// 神煞类型
    pub shensha: ShenSha,
    /// 出现的位置（年/月/日/时）
    pub position: SiZhuPosition,
    /// 吉凶属性
    pub nature: ShenShaNature,
    /// 简要说明
    pub description: &'static str,
}

/// 神煞吉凶属性
pub enum ShenShaNature {
    /// 吉神
    JiShen,
    /// 凶神
    XiongShen,
    /// 中性
    Neutral,
}

/// 四柱位置
pub enum SiZhuPosition {
    Year,   // 年柱
    Month,  // 月柱
    Day,    // 日柱
    Hour,   // 时柱
}
```

### 5. 星运信息结构（XingYunInfo）

```rust
/// 星运信息（日主在四柱各支的十二长生状态）
pub struct XingYunInfo {
    /// 日主在年支的十二长生
    pub year_changsheng: ShiErChangSheng,
    /// 日主在月支的十二长生
    pub month_changsheng: ShiErChangSheng,
    /// 日主在日支的十二长生
    pub day_changsheng: ShiErChangSheng,
    /// 日主在时支的十二长生
    pub hour_changsheng: ShiErChangSheng,
}
```

## 🔧 Runtime API 接口定义

### 1. 主接口：获取完整八字命盘

```rust
/// Runtime API: 获取完整八字命盘（免费，实时计算）
///
/// # 参数
/// - chart_id: 八字命盘ID
///
/// # 返回
/// - Some(FullBaziChart): 完整八字命盘数据
/// - None: 命盘不存在
///
/// # 特点
/// - 完全免费（无 gas 费用）
/// - 响应快速（< 100ms）
/// - 包含所有计算字段（主星、藏干、副星、星运、空亡、纳音、神煞）
/// - 算法自动更新（无需数据迁移）
pub fn get_full_bazi_chart(chart_id: u64) -> Option<FullBaziChart>
```

### 2. 加密命盘接口

```rust
/// Runtime API: 基于加密命盘的四柱索引计算完整八字
///
/// # 参数
/// - chart_id: 加密八字命盘ID
///
/// # 返回
/// - Some(FullBaziChart): 完整八字命盘数据（不包含敏感出生时间）
/// - None: 命盘不存在
///
/// # 安全特性
/// - 基于四柱索引计算，无需解密敏感数据
/// - 完全免费（无 gas 费用）
/// - 保护用户隐私
pub fn get_encrypted_chart_full(chart_id: u64) -> Option<FullBaziChart>
```

## 🎨 前端调用示例

### 1. TypeScript 类型定义

```typescript
/**
 * 完整八字命盘（与 Rust 对应）
 */
export interface FullBaziChart {
  /** 命盘ID */
  chartId: number;
  /** 所有者地址 */
  owner: string;
  /** 出生时间 */
  birthTime: BirthTime;
  /** 性别 */
  gender: Gender;
  /** 四柱信息（包含主星、藏干、纳音） */
  sizhu: EnhancedSiZhu;
  /** 大运信息 */
  dayun: DaYunInfo;
  /** 空亡信息 */
  kongwang: KongWangInfo;
  /** 神煞列表 */
  shenshaList: ShenShaEntry[];
  /** 星运（十二长生） */
  xingyun: XingYunInfo;
  /** 五行强度 */
  wuxingStrength: WuXingStrength;
  /** 喜用神 */
  xiyongShen?: WuXing;
  /** 创建时间戳 */
  timestamp: number;
}

/**
 * 增强四柱结构
 */
export interface EnhancedSiZhu {
  /** 年柱详情 */
  yearZhu: EnhancedZhu;
  /** 月柱详情 */
  monthZhu: EnhancedZhu;
  /** 日柱详情 */
  dayZhu: EnhancedZhu;
  /** 时柱详情 */
  hourZhu: EnhancedZhu;
  /** 日主天干 */
  rizhu: TianGan;
}

/**
 * 增强单柱结构
 */
export interface EnhancedZhu {
  /** 干支组合 */
  ganzhi: GanZhi;
  /** 天干十神（主星） */
  tianganShishen: ShiShen;
  /** 地支本气十神（主星） */
  dizhiBenqiShishen: ShiShen;
  /** 藏干详细信息（副星） */
  cangganList: CangGanInfo[];
  /** 纳音五行 */
  nayin: NaYin;
  /** 日主在该地支的十二长生状态（星运） */
  changsheng: ShiErChangSheng;
}

/**
 * 空亡信息
 */
export interface KongWangInfo {
  /** 年柱旬空 */
  yearKongwang: [DiZhi, DiZhi];
  /** 月柱旬空 */
  monthKongwang: [DiZhi, DiZhi];
  /** 日柱旬空（最重要） */
  dayKongwang: [DiZhi, DiZhi];
  /** 时柱旬空 */
  hourKongwang: [DiZhi, DiZhi];
  /** 四柱是否落空亡 */
  yearIsKong: boolean;
  monthIsKong: boolean;
  dayIsKong: boolean;
  hourIsKong: boolean;
}

/**
 * 神煞条目
 */
export interface ShenShaEntry {
  /** 神煞类型 */
  shensha: ShenSha;
  /** 出现的位置 */
  position: SiZhuPosition;
  /** 吉凶属性 */
  nature: ShenShaNature;
  /** 简要说明 */
  description: string;
}

/**
 * 星运信息（十二长生）
 */
export interface XingYunInfo {
  /** 日主在年支的十二长生 */
  yearChangsheng: ShiErChangSheng;
  /** 日主在月支的十二长生 */
  monthChangsheng: ShiErChangSheng;
  /** 日主在日支的十二长生 */
  dayChangsheng: ShiErChangSheng;
  /** 日主在时支的十二长生 */
  hourChangsheng: ShiErChangSheng;
}
```

### 2. 调用示例（React）

```typescript
import { getApi } from '@/lib/polkadot';
import type { FullBaziChart } from '@/types/bazi';

/**
 * 获取完整八字命盘（包含主星、藏干、副星、星运、空亡、纳音、神煞）
 *
 * @param chartId 命盘ID
 * @returns 完整八字命盘数据
 */
export async function getFullBaziChart(chartId: number): Promise<FullBaziChart | null> {
  const api = await getApi();

  try {
    // 调用 Runtime API（免费，不消耗 gas）
    const result = await api.call.baziChartApi.getFullBaziChart(chartId);

    if (result.isNone) {
      console.warn(`[BaziService] 命盘不存在: ${chartId}`);
      return null;
    }

    const rawData = result.unwrap();

    // 解析返回数据
    const fullChart: FullBaziChart = {
      chartId: rawData.chart_id.toNumber(),
      owner: rawData.owner.toString(),
      birthTime: {
        year: rawData.birth_time.year.toNumber(),
        month: rawData.birth_time.month.toNumber(),
        day: rawData.birth_time.day.toNumber(),
        hour: rawData.birth_time.hour.toNumber(),
        minute: rawData.birth_time.minute.toNumber(),
      },
      gender: parseGender(rawData.gender),
      sizhu: parseEnhancedSiZhu(rawData.sizhu),
      dayun: parseDaYunInfo(rawData.dayun),
      kongwang: parseKongWangInfo(rawData.kongwang),
      shenshaList: parseShenShaList(rawData.shensha_list),
      xingyun: parseXingYunInfo(rawData.xingyun),
      wuxingStrength: parseWuXingStrength(rawData.wuxing_strength),
      xiyongShen: rawData.xiyong_shen.isSome
        ? parseWuXing(rawData.xiyong_shen.unwrap())
        : undefined,
      timestamp: rawData.timestamp.toNumber(),
    };

    return fullChart;
  } catch (error) {
    console.error('[BaziService] 获取八字命盘失败:', error);
    throw error;
  }
}

/**
 * 解析增强四柱结构
 */
function parseEnhancedSiZhu(rawSiZhu: any): EnhancedSiZhu {
  return {
    yearZhu: parseEnhancedZhu(rawSiZhu.year_zhu),
    monthZhu: parseEnhancedZhu(rawSiZhu.month_zhu),
    dayZhu: parseEnhancedZhu(rawSiZhu.day_zhu),
    hourZhu: parseEnhancedZhu(rawSiZhu.hour_zhu),
    rizhu: parseTianGan(rawSiZhu.rizhu),
  };
}

/**
 * 解析增强单柱结构
 */
function parseEnhancedZhu(rawZhu: any): EnhancedZhu {
  return {
    ganzhi: parseGanZhi(rawZhu.ganzhi),
    tianganShishen: parseShiShen(rawZhu.tiangan_shishen),
    dizhiBenqiShishen: parseShiShen(rawZhu.dizhi_benqi_shishen),
    cangganList: rawZhu.canggan_list.map(parseCangGanInfo),
    nayin: parseNaYin(rawZhu.nayin),
    changsheng: parseShiErChangSheng(rawZhu.changsheng),
  };
}

/**
 * 解析空亡信息
 */
function parseKongWangInfo(rawKongwang: any): KongWangInfo {
  return {
    yearKongwang: [parseDiZhi(rawKongwang.year_kongwang[0]), parseDiZhi(rawKongwang.year_kongwang[1])],
    monthKongwang: [parseDiZhi(rawKongwang.month_kongwang[0]), parseDiZhi(rawKongwang.month_kongwang[1])],
    dayKongwang: [parseDiZhi(rawKongwang.day_kongwang[0]), parseDiZhi(rawKongwang.day_kongwang[1])],
    hourKongwang: [parseDiZhi(rawKongwang.hour_kongwang[0]), parseDiZhi(rawKongwang.hour_kongwang[1])],
    yearIsKong: rawKongwang.year_is_kong.valueOf(),
    monthIsKong: rawKongwang.month_is_kong.valueOf(),
    dayIsKong: rawKongwang.day_is_kong.valueOf(),
    hourIsKong: rawKongwang.hour_is_kong.valueOf(),
  };
}

/**
 * 解析神煞列表
 */
function parseShenShaList(rawList: any[]): ShenShaEntry[] {
  return rawList.map(item => ({
    shensha: parseShenSha(item.shensha),
    position: parseSiZhuPosition(item.position),
    nature: parseShenShaNature(item.nature),
    description: item.description.toString(),
  }));
}

/**
 * 解析星运信息
 */
function parseXingYunInfo(rawXingyun: any): XingYunInfo {
  return {
    yearChangsheng: parseShiErChangSheng(rawXingyun.year_changsheng),
    monthChangsheng: parseShiErChangSheng(rawXingyun.month_changsheng),
    dayChangsheng: parseShiErChangSheng(rawXingyun.day_changsheng),
    hourChangsheng: parseShiErChangSheng(rawXingyun.hour_changsheng),
  };
}
```

### 3. UI 展示示例（React Component）

```tsx
import React from 'react';
import { Card, Descriptions, Tag, Space, Divider } from 'antd';
import type { FullBaziChart, EnhancedZhu } from '@/types/bazi';

interface BaziDetailPanelProps {
  chart: FullBaziChart;
}

/**
 * 八字详情面板
 */
export const BaziDetailPanel: React.FC<BaziDetailPanelProps> = ({ chart }) => {
  const { sizhu, kongwang, shenshaList, xingyun } = chart;

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      {/* 四柱展示 */}
      <Card title="四柱八字">
        <Space size="large">
          <ZhuColumn title="年柱" zhu={sizhu.yearZhu} kongwang={kongwang.yearKongwang} isKong={kongwang.yearIsKong} />
          <ZhuColumn title="月柱" zhu={sizhu.monthZhu} kongwang={kongwang.monthKongwang} isKong={kongwang.monthIsKong} />
          <ZhuColumn title="日柱" zhu={sizhu.dayZhu} kongwang={kongwang.dayKongwang} isKong={kongwang.dayIsKong} />
          <ZhuColumn title="时柱" zhu={sizhu.hourZhu} kongwang={kongwang.hourKongwang} isKong={kongwang.hourIsKong} />
        </Space>
      </Card>

      {/* 星运（十二长生） */}
      <Card title="星运（十二长生）">
        <Descriptions bordered column={2}>
          <Descriptions.Item label="年柱星运">
            <Tag color={getChangshengColor(xingyun.yearChangsheng)}>
              {xingyun.yearChangsheng.name}
            </Tag>
          </Descriptions.Item>
          <Descriptions.Item label="月柱星运">
            <Tag color={getChangshengColor(xingyun.monthChangsheng)}>
              {xingyun.monthChangsheng.name}
            </Tag>
          </Descriptions.Item>
          <Descriptions.Item label="日柱星运">
            <Tag color={getChangshengColor(xingyun.dayChangsheng)}>
              {xingyun.dayChangsheng.name}
            </Tag>
          </Descriptions.Item>
          <Descriptions.Item label="时柱星运">
            <Tag color={getChangshengColor(xingyun.hourChangsheng)}>
              {xingyun.hourChangsheng.name}
            </Tag>
          </Descriptions.Item>
        </Descriptions>
      </Card>

      {/* 神煞 */}
      <Card title="神煞">
        <Space wrap>
          {shenshaList.map((item, index) => (
            <Tag
              key={index}
              color={getShenShaColor(item.nature)}
              title={item.description}
            >
              {item.shensha.name} ({item.position})
            </Tag>
          ))}
        </Space>
      </Card>
    </Space>
  );
};

/**
 * 单柱展示组件
 */
const ZhuColumn: React.FC<{
  title: string;
  zhu: EnhancedZhu;
  kongwang: [DiZhi, DiZhi];
  isKong: boolean;
}> = ({ title, zhu, kongwang, isKong }) => {
  const { ganzhi, tianganShishen, dizhiBenqiShishen, cangganList, nayin, changsheng } = zhu;

  return (
    <Card
      title={title}
      size="small"
      style={{ width: 200 }}
      extra={isKong && <Tag color="volcano">空亡</Tag>}
    >
      {/* 干支 */}
      <div style={{ fontSize: 24, fontWeight: 'bold', textAlign: 'center' }}>
        {ganzhi.gan.name}{ganzhi.zhi.name}
      </div>

      <Divider />

      {/* 主星 */}
      <div>
        <strong>主星:</strong>
        <div>
          天干: <Tag color="blue">{tianganShishen.name}</Tag>
        </div>
        <div>
          地支: <Tag color="green">{dizhiBenqiShishen.name}</Tag>
        </div>
      </div>

      <Divider />

      {/* 藏干（副星） */}
      <div>
        <strong>藏干（副星）:</strong>
        {cangganList.map((cg, index) => (
          <div key={index}>
            {cg.gan.name} - <Tag size="small">{cg.shishen.name}</Tag>
            <span style={{ fontSize: 12, color: '#999' }}>
              ({cg.cangganType.name}, {cg.weight})
            </span>
          </div>
        ))}
      </div>

      <Divider />

      {/* 纳音 */}
      <div>
        <strong>纳音:</strong> {nayin.name}
      </div>

      {/* 空亡 */}
      {isKong && (
        <div style={{ marginTop: 8 }}>
          <strong>旬空:</strong> {kongwang[0].name}、{kongwang[1].name}
        </div>
      )}
    </Card>
  );
};

/**
 * 获取长生颜色
 */
function getChangshengColor(changsheng: ShiErChangSheng): string {
  if (changsheng.isProsperous()) return 'green';
  if (changsheng.isDeclining()) return 'red';
  return 'default';
}

/**
 * 获取神煞颜色
 */
function getShenShaColor(nature: ShenShaNature): string {
  switch (nature) {
    case 'JiShen': return 'success';
    case 'XiongShen': return 'error';
    default: return 'default';
  }
}
```

## 🔍 字段详细说明

### 1. 主星（TianGan ShiShen + DiZhi BenQi ShiShen）

**定义**: 主星包括天干十神和地支本气十神，是八字分析的核心要素。

**字段位置**:
- `EnhancedZhu.tiangan_shishen`: 天干十神（如：比肩、劫财、食神等）
- `EnhancedZhu.dizhi_benqi_shishen`: 地支本气十神

**计算方式**:
```rust
// 天干十神 = 根据日主和该柱天干的五行生克关系
pub fn calculate_shishen(rizhu: TianGan, target: TianGan) -> ShiShen

// 地支本气十神 = 地支藏干主气的十神
pub fn get_benqi_shishen(rizhu: TianGan, dizhi: DiZhi) -> ShiShen
```

**示例**:
- 日主为甲木，年柱天干为丙火 → 天干十神 = 食神（我生）
- 日主为甲木，年柱地支为寅（本气甲木）→ 地支本气十神 = 比肩（同我）

### 2. 藏干（CangGan + Weight）

**定义**: 每个地支藏有1-3个天干，分为主气、中气、余气，各有不同权重。

**字段位置**:
- `EnhancedZhu.canggan_list`: 藏干详细列表

**数据结构**:
```rust
pub struct CangGanInfo {
    pub gan: TianGan,           // 藏干天干
    pub shishen: ShiShen,       // 藏干十神（副星）
    pub canggan_type: CangGanType, // 主气/中气/余气
    pub weight: u16,            // 权重（用于五行强度计算）
}
```

**示例**:
- 辰支藏干：戊（主气，权重1800）、乙（中气，权重600）、癸（余气，权重600）

### 3. 副星（CangGan ShiShen）

**定义**: 副星即藏干的十神关系，辅助主星分析。

**字段位置**:
- `CangGanInfo.shishen`: 藏干十神

**计算方式**:
```rust
// 藏干十神 = 根据日主和藏干天干的五行生克关系
for (canggan, type, weight) in canggan_list {
    let shishen = calculate_shishen(rizhu, canggan);
}
```

### 4. 星运（ShiErChangSheng - 十二长生）

**定义**: 日主在四柱各地支的生旺死绝状态，表示能量强弱。

**字段位置**:
- `XingYunInfo.year_changsheng`: 日主在年支的十二长生
- `XingYunInfo.month_changsheng`: 日主在月支的十二长生
- `XingYunInfo.day_changsheng`: 日主在日支的十二长生
- `XingYunInfo.hour_changsheng`: 日主在时支的十二长生

**十二长生类型**:
- 旺相: 长生、冠带、临官、帝旺
- 衰败: 衰、病、死、墓、绝
- 中性: 沐浴、胎、养

**计算方式**:
```rust
/// 十二长生查询表（5行12列）
/// 阳干: 甲、丙、戊、庚、壬
/// 阴干: 乙、丁、己、辛、癸
const CHANGSHENG_TABLE: [[ShiErChangSheng; 12]; 5] = [...];

pub fn get_changsheng(rizhu: TianGan, dizhi: DiZhi) -> ShiErChangSheng {
    let row = if rizhu.is_yang() {
        // 阳干查表
    } else {
        // 阴干查表
    };
    CHANGSHENG_TABLE[row][dizhi.0 as usize]
}
```

### 5. 空亡（KongWang - 旬空）

**定义**: 六十甲子每十个为一旬，每旬有两个地支空缺，称为旬空或空亡。

**字段位置**:
- `KongWangInfo.day_kongwang`: 日柱旬空（最重要）
- `KongWangInfo.year_kongwang`: 年柱旬空
- `KongWangInfo.month_kongwang`: 月柱旬空
- `KongWangInfo.hour_kongwang`: 时柱旬空
- `KongWangInfo.{year|month|day|hour}_is_kong`: 各柱是否落空亡

**旬空对照表**:
```
甲子旬: 戌亥空
甲戌旬: 申酉空
甲申旬: 午未空
甲午旬: 辰巳空
甲辰旬: 寅卯空
甲寅旬: 子丑空
```

**计算方式**:
```rust
/// 计算空亡
pub fn calculate_kongwang(ganzhi: GanZhi) -> (DiZhi, DiZhi) {
    let index = ganzhi.to_index(); // 0-59
    let xun = index / 10; // 确定哪一旬

    match xun {
        0 => (DiZhi(10), DiZhi(11)), // 戌、亥
        1 => (DiZhi(8), DiZhi(9)),   // 申、酉
        2 => (DiZhi(6), DiZhi(7)),   // 午、未
        3 => (DiZhi(4), DiZhi(5)),   // 辰、巳
        4 => (DiZhi(2), DiZhi(3)),   // 寅、卯
        5 => (DiZhi(0), DiZhi(1)),   // 子、丑
        _ => unreachable!(),
    }
}

/// 检查地支是否落空亡
pub fn is_kong(dizhi: DiZhi, kongwang: (DiZhi, DiZhi)) -> bool {
    dizhi == kongwang.0 || dizhi == kongwang.1
}
```

### 6. 纳音（NaYin）

**定义**: 六十甲子对应30种纳音五行，每两个相邻干支共享一个纳音。

**字段位置**:
- `EnhancedZhu.nayin`: 纳音五行

**纳音30种类型**:
- 金: 海中金、剑锋金、白蜡金、沙中金、金箔金、钗钏金
- 木: 大林木、杨柳木、松柏木、平地木、桑柘木、石榴木
- 水: 涧下水、泉中水、长流水、天河水、大溪水、大海水
- 火: 炉中火、山头火、霹雳火、山下火、覆灯火、天上火
- 土: 路旁土、城头土、屋上土、大驿土、壁上土、沙中土

**计算方式**:
```rust
/// 纳音查询表（60项）
const NAYIN_TABLE: [NaYin; 60] = [
    NaYin::HaiZhongJin,   // 甲子
    NaYin::HaiZhongJin,   // 乙丑
    NaYin::LuZhongHuo,    // 丙寅
    NaYin::LuZhongHuo,    // 丁卯
    // ... 共60项
];

pub fn calculate_nayin(ganzhi: &GanZhi) -> NaYin {
    let index = ganzhi.to_index();
    NAYIN_TABLE[index as usize]
}
```

### 7. 神煞（ShenSha）

**定义**: 八字命理中的吉凶神煞，包括贵人、桃花、华盖、羊刃等。

**字段位置**:
- `FullBaziChart.shensha_list`: 神煞列表

**主要神煞类型**:

**贵人类**:
- 天乙贵人: 遇难呈祥，最大吉神
- 文昌贵人: 聪明好学，利考试
- 天德贵人: 逢凶化吉

**桃花婚姻类**:
- 桃花（咸池）: 异性缘佳
- 红鸾: 婚姻吉星
- 孤辰、寡宿: 孤独之星

**财官类**:
- 将星: 领导才能
- 驿马: 奔波走动
- 华盖: 聪明孤高

**凶神类**:
- 羊刃: 刚强暴躁
- 亡神: 灾厄之星
- 劫煞: 劫难之星

**计算方式**:
```rust
/// 计算神煞列表
pub fn calculate_shensha_list(
    sizhu: &SiZhu,
    rizhu: TianGan,
) -> Vec<ShenShaEntry> {
    let mut list = Vec::new();

    // 天乙贵人（以日干或年干查地支）
    let tiangui = get_tianyi_guiren(rizhu);
    if [sizhu.year_zhu.ganzhi.zhi, ...].contains(&tiangui) {
        list.push(ShenShaEntry {
            shensha: ShenSha::TianYiGuiRen,
            position: SiZhuPosition::Year,
            nature: ShenShaNature::JiShen,
            description: "遇难呈祥，有贵人助",
        });
    }

    // 桃花（以日支或年支查其他支）
    let taohua = get_taohua(sizhu.day_zhu.ganzhi.zhi);
    // ...

    // 其他神煞计算
    // ...

    list
}

/// 天乙贵人查询表
fn get_tianyi_guiren(gan: TianGan) -> Vec<DiZhi> {
    match gan.0 {
        0 | 4 => vec![DiZhi(1), DiZhi(7)],   // 甲戊 -> 丑未
        1 | 5 => vec![DiZhi(0), DiZhi(8)],   // 乙己 -> 子申
        2 | 3 => vec![DiZhi(11), DiZhi(9)],  // 丙丁 -> 亥酉
        6 | 7 => vec![DiZhi(5), DiZhi(3)],   // 庚辛 -> 巳卯
        8 | 9 => vec![DiZhi(2), DiZhi(6)],   // 壬癸 -> 寅午
        _ => vec![],
    }
}
```

## 📝 实现步骤

### 阶段1: 数据结构扩展（1-2天）

1. **新增类型定义** (`src/types.rs`)
   - `EnhancedSiZhu`, `EnhancedZhu`
   - `KongWangInfo`
   - `ShenShaEntry`, `ShenShaNature`, `SiZhuPosition`
   - `XingYunInfo`
   - `FullBaziChart`

2. **实现序列化支持**
   - 添加 `Encode`, `Decode`, `TypeInfo`, `MaxEncodedLen` derives
   - 确保与前端 JSON 序列化兼容

### 阶段2: 计算模块实现（3-5天）

1. **空亡计算模块** (`src/kongwang.rs`)
   - 实现旬空查询表
   - 实现 `calculate_kongwang()` 函数
   - 实现 `is_kong()` 判断函数

2. **星运计算模块** (`src/xingyun.rs`)
   - 实现十二长生查询表（阳干/阴干分表）
   - 实现 `get_changsheng()` 函数

3. **神煞计算模块** (`src/shensha.rs` 扩展)
   - 完善现有神煞计算逻辑
   - 实现 `calculate_shensha_list()` 函数
   - 添加神煞查询表（天乙贵人、桃花等）

4. **纳音计算**
   - 已实现于 `src/constants.rs`
   - 验证准确性

### 阶段3: Runtime API 实现（2-3天）

1. **定义 Runtime API** (`src/runtime_api.rs`)
   ```rust
   sp_api::decl_runtime_apis! {
       pub trait BaziChartApi<AccountId> where AccountId: Codec {
           fn get_full_bazi_chart(chart_id: u64) -> Option<FullBaziChart>;
           fn get_encrypted_chart_full(chart_id: u64) -> Option<FullBaziChart>;
       }
   }
   ```

2. **实现 API 函数** (`src/lib.rs`)
   ```rust
   impl<T: Config> Pallet<T> {
       pub fn get_full_bazi_chart(chart_id: u64) -> Option<FullBaziChart> {
           let chart = ChartById::<T>::get(chart_id)?;

           // 计算增强四柱
           let enhanced_sizhu = Self::build_enhanced_sizhu(&chart.sizhu, chart.sizhu.rizhu);

           // 计算空亡
           let kongwang = Self::calculate_all_kongwang(&chart.sizhu);

           // 计算神煞
           let shensha_list = shensha::calculate_shensha_list(&chart.sizhu, chart.sizhu.rizhu);

           // 计算星运
           let xingyun = Self::calculate_xingyun(&chart.sizhu);

           Some(FullBaziChart {
               chart_id,
               owner: chart.owner,
               birth_time: chart.birth_time,
               gender: chart.gender,
               sizhu: enhanced_sizhu,
               dayun: chart.dayun,
               kongwang,
               shensha_list,
               xingyun,
               wuxing_strength: chart.wuxing_strength,
               xiyong_shen: chart.xiyong_shen,
               timestamp: chart.timestamp,
           })
       }
   }
   ```

### 阶段4: 前端集成（2-3天）

1. **类型定义** (`stardust-dapp/src/types/bazi.ts`)
   - 定义 TypeScript 类型（参考上文）

2. **服务层实现** (`stardust-dapp/src/services/baziChainService.ts`)
   - 实现 `getFullBaziChart()` 函数
   - 实现数据解析函数

3. **UI 组件开发**
   - 四柱展示组件
   - 神煞展示组件
   - 星运展示组件
   - 空亡标识

### 阶段5: 测试与优化（2-3天）

1. **单元测试**
   - 空亡计算测试
   - 星运计算测试
   - 神煞计算测试
   - 纳音计算测试

2. **集成测试**
   - Runtime API 调用测试
   - 前端数据解析测试

3. **性能优化**
   - 响应时间优化（目标 < 100ms）
   - 缓存策略（可选）

## 🎯 技术亮点

### 1. 零存储成本
- 所有计算字段通过 Runtime API 实时生成
- 无需额外存储空间
- 算法升级立即生效，无需数据迁移

### 2. 零 Gas 费用
- Runtime API 调用完全免费
- 用户无需支付任何查询费用
- 支持高频查询

### 3. 移动端优先
- 数据结构简洁清晰
- 响应快速（< 100ms）
- 支持离线缓存

### 4. 隐私保护
- 支持加密命盘模式
- 敏感数据前端加密
- 链上仅存储四柱索引

### 5. 算法透明
- 所有计算逻辑开源
- 查询表可验证
- 支持社区审计

## 📋 开发清单

### Phase 1: 基础设施（第1周）
- [ ] 新增数据类型定义（`types.rs`）
- [ ] 实现空亡计算模块（`kongwang.rs`）
- [ ] 实现星运计算模块（`xingyun.rs`）
- [ ] 扩展神煞计算模块（`shensha.rs`）

### Phase 2: Runtime API（第2周）
- [ ] 定义 Runtime API 接口（`runtime_api.rs`）
- [ ] 实现 `get_full_bazi_chart()` 函数
- [ ] 实现 `get_encrypted_chart_full()` 函数
- [ ] 添加单元测试

### Phase 3: 前端集成（第3周）
- [ ] 定义 TypeScript 类型
- [ ] 实现服务层调用函数
- [ ] 实现数据解析函数
- [ ] 开发 UI 展示组件

### Phase 4: 测试与文档（第4周）
- [ ] 编写单元测试和集成测试
- [ ] 性能测试和优化
- [ ] 编写用户文档
- [ ] 编写开发者文档

## 🔗 相关文档

- [八字排盘 Pallet 源码](../src/lib.rs)
- [数据类型定义](../src/types.rs)
- [神煞系统](../src/shensha.rs)
- [前端服务层](../../../../stardust-dapp/src/services/baziChainService.ts)
- [项目开发规范](../../../../CLAUDE.md)

## ✅ 验收标准

### 功能验收
- [ ] Runtime API 可正常调用，返回完整数据结构
- [ ] 主星、藏干、副星、星运、空亡、纳音、神煞计算准确
- [ ] 前端可正确解析和展示所有字段
- [ ] 支持加密命盘模式

### 性能验收
- [ ] Runtime API 响应时间 < 100ms
- [ ] 前端页面渲染流畅，无卡顿
- [ ] 支持并发查询

### 代码质量
- [ ] 所有函数包含详细中文注释
- [ ] 单元测试覆盖率 > 80%
- [ ] 通过 `cargo test` 和 `cargo clippy`

### 文档验收
- [ ] API 文档完整清晰
- [ ] 前端调用示例可运行
- [ ] 用户使用手册完善

---

**文档版本**: v1.0
**最后更新**: 2025-12-20
**维护者**: Stardust 开发团队
