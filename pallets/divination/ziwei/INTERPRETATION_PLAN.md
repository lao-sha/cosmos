# 紫微斗数解盘模块设计方案

## 一、概述

本方案设计紫微斗数的**自动解盘算法**，将复杂的命盘数据转换为可理解的文字解读。
解盘系统分为两层：
1. **后端算法层（Rust）**：生成结构化的解盘数据
2. **前端展示层（React）**：渲染解读内容，支持 AI 增强

---

## 二、解盘维度与层次

### 2.1 解盘层次结构

```
├── 命盘总论（Overall）
│   ├── 命格定性（命宫主星组合）
│   ├── 格局评判（特殊格局识别）
│   └── 五行平衡分析
│
├── 十二宫解读（PalaceReading）
│   ├── 命宫 - 性格、外貌、一生主轴
│   ├── 兄弟宫 - 兄弟姐妹、合作关系
│   ├── 夫妻宫 - 婚姻、配偶特质
│   ├── 子女宫 - 子女缘、创作能力
│   ├── 财帛宫 - 理财能力、财运
│   ├── 疾厄宫 - 健康、疾病倾向
│   ├── 迁移宫 - 外出、变动、贵人
│   ├── 交友宫 - 朋友、下属、社交
│   ├── 官禄宫 - 事业、职业方向
│   ├── 田宅宫 - 不动产、家庭
│   ├── 福德宫 - 精神生活、享受
│   └── 父母宫 - 与长辈关系、遗传
│
├── 大运分析（DaXianReading）
│   ├── 当前大限综述
│   ├── 大限四化影响
│   └── 流年预测（可选）
│
└── 专题分析（TopicReading）
    ├── 事业分析
    ├── 感情分析
    ├── 财运分析
    └── 健康分析
```

---

## 三、后端算法设计

### 3.1 数据结构定义

```rust
// pallets/divination/ziwei/src/interpretation.rs

/// 星曜评分
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct StarScore {
    /// 主星强度（0-100）
    pub strength: u8,
    /// 吉凶倾向（-100 到 100，正为吉，负为凶）
    pub fortune: i8,
    /// 四化加成
    pub si_hua_bonus: i8,
}

/// 宫位解读结果
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct PalaceInterpretation {
    /// 宫位
    pub gong_wei: GongWei,
    /// 总体评分（0-100）
    pub score: u8,
    /// 吉凶等级（1=大凶 2=凶 3=平 4=吉 5=大吉）
    pub fortune_level: u8,
    /// 主要影响因素（编码）
    pub main_factors: [u8; 4],
    /// 关键词索引
    pub keywords: [u8; 3],
}

/// 格局信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct PatternInfo {
    /// 格局类型
    pub pattern_type: PatternType,
    /// 格局强度（0-100）
    pub strength: u8,
    /// 是否成立
    pub is_valid: bool,
}

/// 格局类型枚举
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum PatternType {
    /// 紫府同宫
    ZiFuTongGong = 0,
    /// 日月同宫
    RiYueTongGong = 1,
    /// 机月同梁
    JiYueTongLiang = 2,
    /// 杀破狼
    ShaPaoLang = 3,
    /// 府相朝垣
    FuXiangChaoYuan = 4,
    /// 日月夹命
    RiYueJiaMing = 5,
    /// 火铃贪格
    HuoLingTan = 6,
    /// 铃昌陀武
    LingChangTuoWu = 7,
    /// 羊陀夹忌
    YangTuoJiaJi = 8,
    /// 命里逢空
    MingLiFengKong = 9,
}

/// 命盘解读结果
#[derive(Clone, Encode, Decode, TypeInfo, RuntimeDebug)]
pub struct ChartInterpretation {
    /// 命盘ID
    pub chart_id: u64,
    /// 整体评分（0-100）
    pub overall_score: u8,
    /// 命格类型编码
    pub ming_ge_type: u8,
    /// 识别到的格局
    pub patterns: Vec<PatternInfo>,
    /// 十二宫解读
    pub palace_readings: [PalaceInterpretation; 12],
    /// 五行分布（金木水火土）
    pub wu_xing_distribution: [u8; 5],
    /// 当前大限解读
    pub current_da_xian: DaXianInterpretation,
}

/// 大限解读
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct DaXianInterpretation {
    /// 大限序号（1-12）
    pub index: u8,
    /// 年龄区间
    pub age_range: (u8, u8),
    /// 大限评分
    pub score: u8,
    /// 大限四化
    pub da_xian_si_hua: [u8; 4],
    /// 关键词
    pub keywords: [u8; 3],
}
```

### 3.2 核心算法函数

```rust
// pallets/divination/ziwei/src/interpretation.rs

/// 生成命盘解读
pub fn interpret_chart(chart: &ZiweiChart) -> ChartInterpretation {
    // 1. 计算整体评分
    let overall_score = calculate_overall_score(chart);

    // 2. 判断命格类型
    let ming_ge_type = determine_ming_ge_type(chart);

    // 3. 识别格局
    let patterns = identify_patterns(chart);

    // 4. 解读十二宫
    let palace_readings = interpret_palaces(chart);

    // 5. 分析五行分布
    let wu_xing_distribution = analyze_wu_xing(chart);

    // 6. 解读当前大限
    let current_da_xian = interpret_current_da_xian(chart);

    ChartInterpretation {
        chart_id: chart.id,
        overall_score,
        ming_ge_type,
        patterns,
        palace_readings,
        wu_xing_distribution,
        current_da_xian,
    }
}

/// 计算整体评分
fn calculate_overall_score(chart: &ZiweiChart) -> u8 {
    let mut score: u32 = 50;  // 基础分

    // 1. 命宫主星亮度加成
    let ming_palace = &chart.palaces[chart.ming_gong_pos as usize];
    for star in ming_palace.zhu_xing.iter().flatten() {
        let brightness = get_star_brightness(*star, ming_palace.di_zhi);
        score += brightness.weight() as u32 / 5;
    }

    // 2. 六吉星加成
    let ji_count = count_liu_ji_in_palace(ming_palace);
    score += ji_count as u32 * 5;

    // 3. 六煞星减分
    let sha_count = count_liu_sha_in_palace(ming_palace);
    score = score.saturating_sub(sha_count as u32 * 5);

    // 4. 四化影响
    score = apply_si_hua_bonus(score, chart);

    // 5. 格局加成
    score = apply_pattern_bonus(score, chart);

    score.min(100) as u8
}

/// 判断命格类型
fn determine_ming_ge_type(chart: &ZiweiChart) -> u8 {
    let ming_palace = &chart.palaces[chart.ming_gong_pos as usize];

    // 根据命宫主星组合判断命格
    match ming_palace.zhu_xing.iter().flatten().next() {
        Some(ZhuXing::ZiWei) => 1,      // 紫微坐命
        Some(ZhuXing::TianFu) => 2,      // 天府坐命
        Some(ZhuXing::TaiYang) => 3,     // 太阳坐命
        Some(ZhuXing::TaiYin) => 4,      // 太阴坐命
        Some(ZhuXing::WuQu) => 5,        // 武曲坐命
        Some(ZhuXing::TianTong) => 6,    // 天同坐命
        Some(ZhuXing::LianZhen) => 7,    // 廉贞坐命
        Some(ZhuXing::TianJi) => 8,      // 天机坐命
        Some(ZhuXing::JuMen) => 9,       // 巨门坐命
        Some(ZhuXing::TanLang) => 10,    // 贪狼坐命
        Some(ZhuXing::TianXiang) => 11,  // 天相坐命
        Some(ZhuXing::TianLiang) => 12,  // 天梁坐命
        Some(ZhuXing::QiSha) => 13,      // 七杀坐命
        Some(ZhuXing::PoJun) => 14,      // 破军坐命
        None => 0,                        // 空宫借星
    }
}

/// 识别格局
fn identify_patterns(chart: &ZiweiChart) -> Vec<PatternInfo> {
    let mut patterns = Vec::new();

    // 检查各种格局
    if check_zi_fu_tong_gong(chart) {
        patterns.push(PatternInfo {
            pattern_type: PatternType::ZiFuTongGong,
            strength: 80,
            is_valid: true,
        });
    }

    if check_sha_po_lang(chart) {
        patterns.push(PatternInfo {
            pattern_type: PatternType::ShaPaoLang,
            strength: 70,
            is_valid: true,
        });
    }

    if check_ji_yue_tong_liang(chart) {
        patterns.push(PatternInfo {
            pattern_type: PatternType::JiYueTongLiang,
            strength: 75,
            is_valid: true,
        });
    }

    // 检查凶格
    if check_yang_tuo_jia_ji(chart) {
        patterns.push(PatternInfo {
            pattern_type: PatternType::YangTuoJiaJi,
            strength: 60,
            is_valid: true,
        });
    }

    if check_ming_feng_kong(chart) {
        patterns.push(PatternInfo {
            pattern_type: PatternType::MingLiFengKong,
            strength: 50,
            is_valid: true,
        });
    }

    patterns
}

/// 解读十二宫
fn interpret_palaces(chart: &ZiweiChart) -> [PalaceInterpretation; 12] {
    let mut readings = [PalaceInterpretation::default(); 12];

    for i in 0..12 {
        let palace = &chart.palaces[i];
        readings[i] = interpret_single_palace(palace, chart);
    }

    readings
}

/// 解读单个宫位
fn interpret_single_palace(palace: &Palace, chart: &ZiweiChart) -> PalaceInterpretation {
    let mut score: u32 = 50;
    let mut fortune: i32 = 0;

    // 1. 主星影响
    for star in palace.zhu_xing.iter().flatten() {
        let brightness = get_star_brightness(*star, palace.di_zhi);
        score += brightness.weight() as u32 / 4;
        fortune += match brightness {
            StarBrightness::Miao => 20,
            StarBrightness::Wang => 15,
            StarBrightness::De => 10,
            StarBrightness::Ping => 0,
            StarBrightness::BuDe => -5,
            StarBrightness::Xian => -15,
        };
    }

    // 2. 六吉星影响
    for (i, &has_star) in palace.liu_ji.iter().enumerate() {
        if has_star {
            score += 5;
            fortune += 10;
        }
    }

    // 3. 六煞星影响
    for (i, &has_star) in palace.liu_sha.iter().enumerate() {
        if has_star {
            score = score.saturating_sub(3);
            fortune -= 8;
        }
    }

    // 4. 禄存天马影响
    if palace.lu_cun {
        score += 8;
        fortune += 15;
    }
    if palace.tian_ma {
        score += 3;
        fortune += 5;
    }

    // 5. 四化影响
    for (i, si_hua) in palace.si_hua.iter().enumerate() {
        if let Some(sh) = si_hua {
            match sh {
                SiHua::HuaLu => { score += 10; fortune += 20; },
                SiHua::HuaQuan => { score += 8; fortune += 15; },
                SiHua::HuaKe => { score += 6; fortune += 10; },
                SiHua::HuaJi => { score = score.saturating_sub(8); fortune -= 15; },
            }
        }
    }

    // 转换为吉凶等级
    let fortune_level = match fortune {
        f if f >= 30 => 5,   // 大吉
        f if f >= 10 => 4,   // 吉
        f if f >= -10 => 3,  // 平
        f if f >= -30 => 2,  // 凶
        _ => 1,              // 大凶
    };

    PalaceInterpretation {
        gong_wei: palace.gong_wei,
        score: score.min(100) as u8,
        fortune_level,
        main_factors: [0; 4],  // 待填充
        keywords: [0; 3],      // 待填充
    }
}
```

### 3.3 格局检测函数

```rust
/// 检查紫府同宫格
fn check_zi_fu_tong_gong(chart: &ZiweiChart) -> bool {
    for palace in &chart.palaces {
        let has_ziwei = palace.zhu_xing.iter().flatten()
            .any(|s| *s == ZhuXing::ZiWei);
        let has_tianfu = palace.zhu_xing.iter().flatten()
            .any(|s| *s == ZhuXing::TianFu);
        if has_ziwei && has_tianfu {
            return true;
        }
    }
    false
}

/// 检查杀破狼格
fn check_sha_po_lang(chart: &ZiweiChart) -> bool {
    let ming_palace = &chart.palaces[chart.ming_gong_pos as usize];

    // 命宫有七杀、破军、贪狼任一
    ming_palace.zhu_xing.iter().flatten().any(|s| {
        matches!(s, ZhuXing::QiSha | ZhuXing::PoJun | ZhuXing::TanLang)
    })
}

/// 检查机月同梁格
fn check_ji_yue_tong_liang(chart: &ZiweiChart) -> bool {
    let ming_palace = &chart.palaces[chart.ming_gong_pos as usize];

    // 命宫有天机、太阴、天同、天梁任一
    ming_palace.zhu_xing.iter().flatten().any(|s| {
        matches!(s, ZhuXing::TianJi | ZhuXing::TaiYin | ZhuXing::TianTong | ZhuXing::TianLiang)
    })
}

/// 检查羊陀夹忌
fn check_yang_tuo_jia_ji(chart: &ZiweiChart) -> bool {
    let ming_pos = chart.ming_gong_pos as usize;
    let left = (ming_pos + 11) % 12;
    let right = (ming_pos + 1) % 12;

    // 命宫有化忌
    let has_hua_ji = chart.palaces[ming_pos].si_hua.iter()
        .any(|sh| matches!(sh, Some(SiHua::HuaJi)));

    // 左右宫有擎羊陀罗
    let left_has_sha = chart.palaces[left].liu_sha[0] || chart.palaces[left].liu_sha[1];
    let right_has_sha = chart.palaces[right].liu_sha[0] || chart.palaces[right].liu_sha[1];

    has_hua_ji && left_has_sha && right_has_sha
}

/// 检查命里逢空
fn check_ming_feng_kong(chart: &ZiweiChart) -> bool {
    let ming_palace = &chart.palaces[chart.ming_gong_pos as usize];

    // 命宫有地空或地劫
    ming_palace.liu_sha[4] || ming_palace.liu_sha[5]
}
```

---

## 四、前端展示设计

### 4.1 组件结构

```
src/features/ziwei/
├── ZiweiPage.tsx                 # 主页面（已有）
├── components/
│   ├── ChartDisplay.tsx          # 命盘展示组件
│   ├── PalaceCard.tsx            # 宫位卡片
│   ├── InterpretationPanel.tsx   # 解读面板
│   ├── PatternBadge.tsx          # 格局标签
│   ├── DaXianTimeline.tsx        # 大限时间线
│   └── ScoreGauge.tsx            # 评分仪表盘
├── hooks/
│   └── useInterpretation.ts      # 解读数据 Hook
└── utils/
    └── interpretationText.ts     # 解读文案库
```

### 4.2 解读文案库结构

```typescript
// src/features/ziwei/utils/interpretationText.ts

/**
 * 命格解读文案
 */
export const MING_GE_TEXTS: Record<number, {
  name: string;
  brief: string;
  personality: string;
  career: string;
  relationship: string;
}> = {
  1: {
    name: '紫微坐命',
    brief: '帝王之星守命，天生具有领导气质和贵气',
    personality: '性格稳重大方，有威严但不霸道，处事公正有原则',
    career: '适合从事管理、领导相关工作，易得贵人提携',
    relationship: '对感情认真负责，但可能因忙于事业而疏于经营',
  },
  2: {
    name: '天府坐命',
    brief: '财库之星守命，一生衣食无忧',
    personality: '为人宽厚，做事稳健，善于理财',
    career: '适合金融、地产、仓储等行业',
    relationship: '重视家庭，是可靠的伴侣',
  },
  // ... 其他14种命格
};

/**
 * 格局解读文案
 */
export const PATTERN_TEXTS: Record<PatternType, {
  name: string;
  description: string;
  effect: string;
  advice: string;
}> = {
  [PatternType.ZiFuTongGong]: {
    name: '紫府同宫',
    description: '紫微与天府同在一宫，帝星与财星同宫',
    effect: '此格局主大贵，一生财官双美，地位显赫',
    advice: '宜把握机会，勇于承担责任',
  },
  [PatternType.ShaPaoLang]: {
    name: '杀破狼',
    description: '命宫坐七杀、破军或贪狼',
    effect: '一生变动较大，适合开创性事业，晚年安定',
    advice: '年轻时宜多闯荡，中年后宜求稳定',
  },
  // ... 其他格局
};

/**
 * 宫位解读文案
 */
export const GONG_WEI_TEXTS: Record<GongWei, {
  name: string;
  domain: string;
  goodKeywords: string[];
  badKeywords: string[];
}> = {
  [GongWei.MingGong]: {
    name: '命宫',
    domain: '性格、外貌、一生总论',
    goodKeywords: ['贵气', '聪慧', '有为', '机敏', '福厚'],
    badKeywords: ['劳碌', '孤独', '固执', '多疑', '急躁'],
  },
  [GongWei.CaiBo]: {
    name: '财帛宫',
    domain: '理财能力、财运状况',
    goodKeywords: ['财源广', '善理财', '正财旺', '意外财'],
    badKeywords: ['财来财去', '破耗', '守不住', '投资失利'],
  },
  // ... 其他宫位
};

/**
 * 根据宫位评分生成综合描述
 */
export function generatePalaceDescription(
  gongWei: GongWei,
  score: number,
  fortuneLevel: number,
  mainStars: ZhuXing[]
): string {
  const base = GONG_WEI_TEXTS[gongWei];
  const keywords = fortuneLevel >= 4
    ? base.goodKeywords.slice(0, 3)
    : base.badKeywords.slice(0, 3);

  let description = `【${base.name}】主管${base.domain}。`;

  // 根据评分添加描述
  if (score >= 80) {
    description += '此宫位星曜组合极佳，';
  } else if (score >= 60) {
    description += '此宫位星曜配置良好，';
  } else if (score >= 40) {
    description += '此宫位星曜平和，';
  } else {
    description += '此宫位星曜欠佳，';
  }

  description += `关键词：${keywords.join('、')}。`;

  return description;
}
```

### 4.3 解读面板组件

```tsx
// src/features/ziwei/components/InterpretationPanel.tsx

import React, { useState } from 'react';
import { Card, Tabs, Progress, Tag, Space, Collapse, Typography } from 'antd';
import {
  MING_GE_TEXTS,
  PATTERN_TEXTS,
  generatePalaceDescription
} from '../utils/interpretationText';

const { Text, Paragraph } = Typography;
const { Panel } = Collapse;

interface InterpretationPanelProps {
  interpretation: ChartInterpretation;
  chart: ZiweiChart;
}

export const InterpretationPanel: React.FC<InterpretationPanelProps> = ({
  interpretation,
  chart,
}) => {
  const [activeTab, setActiveTab] = useState('overall');

  // 命格信息
  const mingGeInfo = MING_GE_TEXTS[interpretation.ming_ge_type];

  return (
    <Card title="命盘解读" className="interpretation-panel">
      <Tabs activeKey={activeTab} onChange={setActiveTab}>
        {/* 总论标签页 */}
        <Tabs.TabPane tab="总论" key="overall">
          {/* 评分仪表盘 */}
          <div style={{ textAlign: 'center', marginBottom: 16 }}>
            <Progress
              type="dashboard"
              percent={interpretation.overall_score}
              format={(percent) => (
                <div>
                  <div style={{ fontSize: 24, fontWeight: 'bold' }}>{percent}</div>
                  <div style={{ fontSize: 12 }}>综合评分</div>
                </div>
              )}
              strokeColor={{
                '0%': '#ff4d4f',
                '50%': '#faad14',
                '100%': '#52c41a',
              }}
            />
          </div>

          {/* 命格描述 */}
          {mingGeInfo && (
            <div style={{ marginBottom: 16 }}>
              <Text strong>命格：</Text>
              <Tag color="purple">{mingGeInfo.name}</Tag>
              <Paragraph style={{ marginTop: 8 }}>
                {mingGeInfo.brief}
              </Paragraph>
            </div>
          )}

          {/* 格局标签 */}
          <div style={{ marginBottom: 16 }}>
            <Text strong>格局：</Text>
            <Space style={{ marginTop: 8 }} wrap>
              {interpretation.patterns.map((pattern, idx) => {
                const info = PATTERN_TEXTS[pattern.pattern_type];
                const isGood = pattern.strength >= 60;
                return (
                  <Tag
                    key={idx}
                    color={isGood ? 'green' : 'red'}
                  >
                    {info?.name || '未知格局'}
                  </Tag>
                );
              })}
              {interpretation.patterns.length === 0 && (
                <Text type="secondary">无特殊格局</Text>
              )}
            </Space>
          </div>

          {/* 五行分布 */}
          <div>
            <Text strong>五行分布：</Text>
            <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
              {['金', '木', '水', '火', '土'].map((wx, idx) => (
                <div key={wx} style={{ textAlign: 'center' }}>
                  <Progress
                    type="circle"
                    percent={interpretation.wu_xing_distribution[idx]}
                    width={50}
                    format={() => wx}
                  />
                </div>
              ))}
            </div>
          </div>
        </Tabs.TabPane>

        {/* 十二宫标签页 */}
        <Tabs.TabPane tab="十二宫" key="palaces">
          <Collapse accordion>
            {interpretation.palace_readings.map((reading, idx) => (
              <Panel
                header={
                  <Space>
                    <Text>{GONG_NAMES[reading.gong_wei]}</Text>
                    <Tag color={getFortuneColor(reading.fortune_level)}>
                      {getFortuneText(reading.fortune_level)}
                    </Tag>
                    <Text type="secondary">{reading.score}分</Text>
                  </Space>
                }
                key={idx}
              >
                <Paragraph>
                  {generatePalaceDescription(
                    reading.gong_wei,
                    reading.score,
                    reading.fortune_level,
                    chart.palaces[idx].zhu_xing.filter(Boolean)
                  )}
                </Paragraph>
              </Panel>
            ))}
          </Collapse>
        </Tabs.TabPane>

        {/* 大限标签页 */}
        <Tabs.TabPane tab="大限" key="daxian">
          <DaXianTimeline
            daXians={chart.daXians}
            currentInterpretation={interpretation.current_da_xian}
          />
        </Tabs.TabPane>

        {/* 专题分析标签页 */}
        <Tabs.TabPane tab="专题" key="topics">
          <TopicAnalysis
            chart={chart}
            interpretation={interpretation}
          />
        </Tabs.TabPane>
      </Tabs>
    </Card>
  );
};

// 辅助函数
function getFortuneColor(level: number): string {
  const colors = ['', '#ff4d4f', '#fa8c16', '#d9d9d9', '#52c41a', '#1890ff'];
  return colors[level] || '#d9d9d9';
}

function getFortuneText(level: number): string {
  const texts = ['', '大凶', '凶', '平', '吉', '大吉'];
  return texts[level] || '平';
}
```

---

## 五、实施计划

### 阶段一：后端算法实现（预计 2-3 天）

| 任务 | 文件 | 说明 |
|------|------|------|
| 定义数据结构 | `types.rs` | 添加解读相关类型 |
| 实现评分算法 | `interpretation.rs` | 新建解盘算法模块 |
| 格局检测 | `interpretation.rs` | 实现10+种格局检测 |
| 宫位解读 | `interpretation.rs` | 十二宫解读逻辑 |
| 单元测试 | `tests.rs` | 解盘算法测试 |

### 阶段二：前端组件开发（预计 2-3 天）

| 任务 | 文件 | 说明 |
|------|------|------|
| 文案库 | `interpretationText.ts` | 解读文案数据 |
| 解读面板 | `InterpretationPanel.tsx` | 主要解读组件 |
| 评分仪表盘 | `ScoreGauge.tsx` | 评分可视化 |
| 大限时间线 | `DaXianTimeline.tsx` | 大限展示 |
| 页面集成 | `ZiweiPage.tsx` | 集成解读功能 |

### 阶段三：AI 增强（可选，预计 1-2 天）

| 任务 | 说明 |
|------|------|
| AI 解读接口 | 调用 `pallet-divination-ai` |
| IPFS 存储 | 解读结果上链存储 |
| 混合展示 | 算法 + AI 结合展示 |

---

## 六、技术要点

### 6.1 链上 vs 链下

| 内容 | 存储位置 | 原因 |
|------|----------|------|
| 命盘原始数据 | 链上 | 不可篡改、可验证 |
| 解盘评分 | 链下计算 | 减少链上计算开销 |
| 解读文案 | 前端 | 灵活更新、多语言支持 |
| AI 解读结果 | IPFS + CID上链 | 大文本存储 |

### 6.2 评分算法权重

```
整体评分 = 基础分(50)
         + 命宫主星亮度权重(0-20)
         + 六吉星加成(0-30)
         - 六煞星减分(0-15)
         + 四化加成(-20 ~ +30)
         + 格局加成(-15 ~ +25)
```

### 6.3 扩展性考虑

- **多流派支持**：预留流派参数，支持三合派、飞星派等
- **自定义权重**：允许用户调整评分权重
- **历史解读**：支持不同时间点的解读对比

---

## 七、总结

本方案设计了一套完整的紫微斗数解盘系统：

1. **后端**：Rust 实现核心算法，生成结构化评分数据
2. **前端**：React 组件渲染解读内容，提供丰富的可视化
3. **文案**：独立的文案库，支持灵活更新和多语言
4. **AI**：预留 AI 增强接口，支持更深度的个性化解读

这套方案具有以下优势：
- **性能优**：评分算法在前端执行，响应快速
- **可维护**：文案与算法分离，易于更新
- **可扩展**：支持多流派、多语言、AI 增强
- **用户友好**：直观的可视化展示
