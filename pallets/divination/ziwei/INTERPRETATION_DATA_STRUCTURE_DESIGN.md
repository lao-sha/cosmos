# 紫微斗数解卦数据结构设计

## 一、概述

本文档设计紫微斗数的**链上解卦数据结构**，用于存储和展示命盘的解读信息。

### 设计原则

1. **链上存储最小化**：仅存储核心评分和索引，详细文案由前端/Runtime API生成
2. **可扩展性**：预留扩展字段，支持多流派（三合派、飞星派等）
3. **AI友好**：结构化数据便于AI解读和分析
4. **前端友好**：提供完整的查询API，减少前端计算负担
5. **隐私保护**：敏感信息可选加密存储

---

## 二、核心数据结构

### 2.1 命盘整体评分

```rust
/// 命盘整体评分
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct ChartOverallScore {
    /// 综合评分（0-100）
    pub overall_score: u8,

    /// 命格等级（0-5）
    /// 0=普通, 1=小贵, 2=中贵, 3=大贵, 4=极贵, 5=帝王格
    pub ming_ge_level: u8,

    /// 富贵指数（0-100）
    pub wealth_index: u8,

    /// 事业指数（0-100）
    pub career_index: u8,

    /// 感情指数（0-100）
    pub relationship_index: u8,

    /// 健康指数（0-100）
    pub health_index: u8,

    /// 福德指数（0-100）
    pub fortune_index: u8,
}
```

### 2.2 宫位解读数据

```rust
/// 单个宫位的解读数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct PalaceInterpretation {
    /// 宫位类型
    pub gong_wei: GongWei,

    /// 宫位评分（0-100）
    pub score: u8,

    /// 吉凶等级（0-6）
    /// 0=大吉, 1=吉, 2=小吉, 3=平, 4=小凶, 5=凶, 6=大凶
    pub fortune_level: u8,

    /// 主星强度（0-100）
    pub star_strength: u8,

    /// 四化影响（-50 ~ +50）
    pub si_hua_impact: i8,

    /// 六吉星数量（0-6）
    pub liu_ji_count: u8,

    /// 六煞星数量（0-6）
    pub liu_sha_count: u8,

    /// 关键词索引（最多3个）
    /// 索引对应预定义的关键词表
    pub keywords: [u8; 3],

    /// 主要影响因素（位标志）
    /// bit 0: 主星庙旺
    /// bit 1: 四化加持
    /// bit 2: 六吉会照
    /// bit 3: 六煞冲破
    /// bit 4: 空宫借星
    /// bit 5-7: 预留
    pub factors: u8,
}
```

### 2.3 格局信息

```rust
/// 格局类型枚举（1 byte）
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum PatternType {
    // ===== 富贵格局 =====
    /// 紫府同宫
    ZiFuTongGong = 0,
    /// 紫府朝垣
    ZiFuChaoYuan = 1,
    /// 天府朝垣
    TianFuChaoYuan = 2,
    /// 君臣庆会
    JunChenQingHui = 3,
    /// 府相朝垣
    FuXiangChaoYuan = 4,
    /// 机月同梁
    JiYueTongLiang = 5,
    /// 日月并明
    RiYueBingMing = 6,
    /// 日照雷门
    RiZhaoLeiMen = 7,
    /// 月朗天门
    YueLangTianMen = 8,
    /// 明珠出海
    MingZhuChuHai = 9,
    /// 阳梁昌禄
    YangLiangChangLu = 10,
    /// 贪武同行
    TanWuTongXing = 11,
    /// 火贪格
    HuoTanGeJu = 12,
    /// 铃贪格
    LingTanGeJu = 13,

    // ===== 权贵格局 =====
    /// 三奇嘉会（禄权科）
    SanQiJiaHui = 14,
    /// 双禄夹命
    ShuangLuJiaMing = 15,
    /// 双禄夹财
    ShuangLuJiaCai = 16,
    /// 科权禄夹
    KeQuanLuJia = 17,
    /// 左右夹命
    ZuoYouJiaMing = 18,
    /// 昌曲夹命
    ChangQuJiaMing = 19,
    /// 魁钺夹命
    KuiYueJiaMing = 20,
    /// 禄马交驰
    LuMaJiaoChiGeJu = 21,

    // ===== 凶格 =====
    /// 铃昌陀武
    LingChangTuoWu = 22,
    /// 巨机同宫
    JiJiTongGong = 23,
    /// 巨日同宫
    JuRiTongGong = 24,
    /// 命无正曜（空宫）
    MingWuZhengYao = 25,
    /// 马头带箭
    MaTouDaiJian = 26,
    /// 羊陀夹命
    YangTuoJiaMing = 27,
    /// 火铃夹命
    HuoLingJiaMing = 28,
    /// 空劫夹命
    KongJieJiaMing = 29,
    /// 羊陀夹忌
    YangTuoJiaJi = 30,
    /// 四煞冲命
    SiShaChongMing = 31,
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

    /// 是否吉格
    pub is_auspicious: bool,

    /// 格局分数（-50 ~ +50）
    pub score: i8,

    /// 关键宫位索引（最多3个）
    pub key_palaces: [u8; 3],
}
```

### 2.4 四化飞星分析

```rust
/// 四化飞星分析
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct SiHuaAnalysis {
    /// 生年四化星（禄权科忌）
    pub sheng_nian_si_hua: [SiHuaStar; 4],

    /// 命宫四化飞入宫位（禄权科忌）
    pub ming_gong_fei_ru: [u8; 4],

    /// 财帛宫四化飞入宫位
    pub cai_bo_fei_ru: [u8; 4],

    /// 官禄宫四化飞入宫位
    pub guan_lu_fei_ru: [u8; 4],

    /// 夫妻宫四化飞入宫位
    pub fu_qi_fei_ru: [u8; 4],

    /// 自化宫位（位标志，12 bits）
    /// bit 0-11 对应 12 个宫位
    pub zi_hua_palaces: u16,

    /// 化忌冲破宫位（位标志，12 bits）
    pub hua_ji_chong_po: u16,
}
```

### 2.5 大运分析

```rust
/// 单个大限的解读数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct DaXianInterpretation {
    /// 大限序号（1-12）
    pub index: u8,

    /// 起始年龄
    pub start_age: u8,

    /// 结束年龄
    pub end_age: u8,

    /// 大限宫位索引
    pub gong_index: u8,

    /// 大限评分（0-100）
    pub score: u8,

    /// 大限吉凶等级（0-6）
    pub fortune_level: u8,

    /// 大限四化飞入宫位（禄权科忌）
    pub si_hua_fei_ru: [u8; 4],

    /// 关键词索引（最多3个）
    pub keywords: [u8; 3],
}
```

### 2.6 完整解卦数据

```rust
/// 紫微斗数完整解卦数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct ZiweiInterpretation {
    /// 命盘ID
    pub chart_id: u64,

    /// 整体评分
    pub overall_score: ChartOverallScore,

    /// 十二宫解读
    pub palace_interpretations: [PalaceInterpretation; 12],

    /// 识别到的格局（最多10个）
    pub patterns: BoundedVec<PatternInfo, ConstU32<10>>,

    /// 四化飞星分析
    pub si_hua_analysis: SiHuaAnalysis,

    /// 十二大限解读
    pub da_xian_interpretations: [DaXianInterpretation; 12],

    /// 五行分布（金木水火土）
    pub wu_xing_distribution: [u8; 5],

    /// 命主星索引（0-13，对应14主星）
    pub ming_zhu_star: u8,

    /// 身主星索引
    pub shen_zhu_star: u8,

    /// 创建时间戳
    pub created_at: u64,

    /// AI解读CID（可选）
    pub ai_interpretation_cid: Option<BoundedVec<u8, ConstU32<64>>>,
}
```

---

## 三、关键词索引表

为了节省链上存储空间，使用索引代替完整文本：

### 3.1 命宫关键词表（0-99）

```rust
pub const MING_GONG_KEYWORDS: [&str; 100] = [
    // 0-19: 性格特质
    "贵气", "聪慧", "稳重", "果断", "温和", "刚毅", "机敏", "谨慎", "豪爽", "内敛",
    "乐观", "悲观", "固执", "灵活", "保守", "激进", "理性", "感性", "务实", "理想",

    // 20-39: 能力特质
    "领导力强", "执行力佳", "创造力足", "分析力强", "沟通力好", "学习力快", "适应力强", "抗压力好",
    "组织力强", "协调力佳", "判断力准", "洞察力深", "表达力强", "亲和力好", "影响力大", "号召力强",
    "专注力强", "耐力足", "爆发力强", "持久力好",

    // 40-59: 运势特征
    "一生顺遂", "早年辛劳", "中年发达", "晚年安康", "贵人相助", "自力更生", "波折较多", "平稳发展",
    "大器晚成", "少年得志", "起伏不定", "稳步上升", "突破性强", "渐进式好", "变动频繁", "安定为主",
    "机遇多", "挑战大", "转折点多", "关键时刻",

    // 60-79: 事业财运
    "官运亨通", "财运旺盛", "事业有成", "名利双收", "适合创业", "宜守成", "偏财运好", "正财稳定",
    "投资有道", "理财得当", "横财可期", "积累为主", "权力欲强", "淡泊名利", "追求卓越", "知足常乐",
    "商业头脑", "技术专长", "艺术天赋", "学术造诣",

    // 80-99: 感情健康
    "感情顺遂", "婚姻美满", "桃花旺盛", "晚婚为宜", "配偶贤良", "夫妻和睦", "子女孝顺", "家庭和谐",
    "身体健康", "精力充沛", "注意保养", "易有小疾", "长寿之相", "体质较弱", "心态平和", "情绪稳定",
    "人缘极佳", "朋友众多", "贵人运强", "小人较少",
];
```

### 3.2 其他宫位关键词表

类似地，为财帛宫、官禄宫、夫妻宫等定义专属关键词表。

---

## 四、评分算法

### 4.1 宫位评分算法

```rust
/// 计算宫位评分
pub fn calculate_palace_score(palace: &Palace, chart: &ZiweiChart) -> u8 {
    let mut score: i32 = 50; // 基础分

    // 1. 主星亮度加成（0-25分）
    for (star, brightness) in palace.zhu_xing.iter().zip(palace.zhu_xing_brightness.iter()) {
        if let Some(s) = star {
            score += match brightness {
                StarBrightness::Miao => 25,
                StarBrightness::Wang => 20,
                StarBrightness::De => 15,
                StarBrightness::Ping => 0,
                StarBrightness::BuDe => -5,
                StarBrightness::Xian => -15,
            };
        }
    }

    // 2. 六吉星加成（每颗+5分，最多+30分）
    let ji_count = palace.liu_ji.iter().filter(|&&x| x).count();
    score += (ji_count as i32) * 5;

    // 3. 六煞星减分（每颗-5分，最多-30分）
    let sha_count = palace.liu_sha.iter().filter(|&&x| x).count();
    score -= (sha_count as i32) * 5;

    // 4. 四化影响（-20 ~ +30分）
    for si_hua in palace.si_hua.iter().flatten() {
        score += match si_hua {
            SiHua::HuaLu => 15,
            SiHua::HuaQuan => 10,
            SiHua::HuaKe => 8,
            SiHua::HuaJi => -20,
        };
    }

    // 5. 禄存天马加成
    if palace.lu_cun { score += 10; }
    if palace.tian_ma { score += 5; }

    // 限制在 0-100 范围
    score.clamp(0, 100) as u8
}
```

### 4.2 整体评分算法

```rust
/// 计算命盘整体评分
pub fn calculate_overall_score(chart: &ZiweiChart, interpretations: &[PalaceInterpretation; 12]) -> ChartOverallScore {
    // 1. 命宫权重最高（40%）
    let ming_score = interpretations[chart.ming_gong_pos as usize].score as u32;

    // 2. 财官夫各占15%
    let cai_score = get_palace_score_by_type(interpretations, GongWei::CaiBo) as u32;
    let guan_score = get_palace_score_by_type(interpretations, GongWei::GuanLu) as u32;
    let fu_score = get_palace_score_by_type(interpretations, GongWei::FuQi) as u32;

    // 3. 其他宫位占15%
    let other_avg = calculate_other_palaces_avg(interpretations) as u32;

    // 加权平均
    let overall = (ming_score * 40 + cai_score * 15 + guan_score * 15 + fu_score * 15 + other_avg * 15) / 100;

    // 格局加成（-20 ~ +20分）
    let pattern_bonus = calculate_pattern_bonus(chart);

    let final_score = (overall as i32 + pattern_bonus).clamp(0, 100) as u8;

    ChartOverallScore {
        overall_score: final_score,
        ming_ge_level: determine_ming_ge_level(chart, final_score),
        wealth_index: cai_score as u8,
        career_index: guan_score as u8,
        relationship_index: fu_score as u8,
        health_index: get_palace_score_by_type(interpretations, GongWei::JiE) as u8,
        fortune_index: get_palace_score_by_type(interpretations, GongWei::FuDe) as u8,
    }
}
```

---

## 五、Runtime API 设计

### 5.1 查询接口

```rust
decl_runtime_apis! {
    pub trait ZiweiInterpretationApi {
        /// 生成命盘解读数据
        fn generate_interpretation(chart_id: u64) -> Result<ZiweiInterpretation, DispatchError>;

        /// 获取宫位详细解读文本
        fn get_palace_detail(chart_id: u64, gong_wei: GongWei) -> PalaceDetailText;

        /// 获取格局详细说明
        fn get_pattern_detail(pattern_type: PatternType) -> PatternDetailText;

        /// 获取大限详细解读
        fn get_da_xian_detail(chart_id: u64, age: u8) -> DaXianDetailText;

        /// 获取流年运势
        fn get_liu_nian_fortune(chart_id: u64, year: u16) -> LiuNianFortune;
    }
}
```

### 5.2 详细文本结构

```rust
/// 宫位详细解读文本
pub struct PalaceDetailText {
    /// 宫位名称
    pub name: String,
    /// 综合描述
    pub summary: String,
    /// 主星解读
    pub star_interpretation: Vec<String>,
    /// 四化影响
    pub si_hua_impact: String,
    /// 吉凶分析
    pub fortune_analysis: String,
    /// 建议
    pub advice: String,
}
```

---

## 六、前端展示设计

### 6.1 组件结构

```
src/features/ziwei/interpretation/
├── InterpretationPanel.tsx          # 主解读面板
├── OverallScoreCard.tsx             # 整体评分卡片
├── PalaceInterpretationList.tsx    # 十二宫解读列表
├── PatternBadges.tsx                # 格局标签
├── SiHuaFlowChart.tsx               # 四化飞星流程图
├── DaXianTimeline.tsx               # 大限时间线
└── LiuNianFortune.tsx               # 流年运势
```

### 6.2 数据流

```
1. 用户查看命盘 → 调用 Runtime API generate_interpretation()
2. 获取 ZiweiInterpretation 结构化数据
3. 前端根据索引查询关键词表，生成完整文本
4. 渲染各个组件展示
5. 用户点击详情 → 调用 get_palace_detail() 获取详细文本
```

---

## 七、存储优化

### 7.1 数据大小估算

```
ChartOverallScore:        7 bytes
PalaceInterpretation:     12 bytes × 12 = 144 bytes
PatternInfo:              8 bytes × 10 = 80 bytes (BoundedVec)
SiHuaAnalysis:            32 bytes
DaXianInterpretation:     8 bytes × 12 = 96 bytes
其他字段:                  ~50 bytes
─────────────────────────────────────
总计:                      ~410 bytes
```

相比存储完整文本（可能数KB），节省了90%以上的存储空间。

### 7.2 缓存策略

- 解卦数据生成后缓存在链上
- 详细文本由 Runtime API 实时生成，不存储
- 前端可缓存常用关键词表和文案模板

---

## 八、扩展性设计

### 8.1 多流派支持

```rust
/// 流派类型
pub enum School {
    SanHe,      // 三合派
    FeiXing,    // 飞星派
    SiHua,      // 四化派
    ZiWei,      // 紫微派
}

/// 在 ZiweiInterpretation 中添加
pub school: School,
```

### 8.2 自定义权重

```rust
/// 评分权重配置
pub struct ScoreWeights {
    pub ming_gong_weight: u8,
    pub cai_bo_weight: u8,
    pub guan_lu_weight: u8,
    pub fu_qi_weight: u8,
    // ...
}
```

---

## 九、实施计划

### 阶段一：核心数据结构（1-2天）
- [ ] 定义所有数据结构
- [ ] 实现评分算法
- [ ] 编写单元测试

### 阶段二：Runtime API（1-2天）
- [ ] 实现 generate_interpretation()
- [ ] 实现详细文本生成函数
- [ ] 集成到 pallet-ziwei

### 阶段三：前端组件（2-3天）
- [ ] 实现解读面板组件
- [ ] 实现数据可视化
- [ ] 集成到 ZiweiPage

### 阶段四：测试与优化（1天）
- [ ] 端到端测试
- [ ] 性能优化
- [ ] 文档完善

---

## 十、总结

本设计方案具有以下优势：

1. **存储高效**：使用索引和位标志，节省90%存储空间
2. **计算灵活**：评分算法可调整，支持多流派
3. **扩展性强**：预留扩展字段，支持未来功能
4. **AI友好**：结构化数据便于AI分析
5. **用户友好**：前端可灵活展示，支持多语言

该方案参考了：
- xuanxue/ziwei/ZhouYiLab 的 C++ 实现
- pallets/divination/meihua 的解卦数据结构
- pallets/divination/xiaoliuren 的枚举设计
- pallets/divination/ziwei/INTERPRETATION_PLAN.md 的规划

可直接用于区块链项目的紫微斗数解卦功能实现。
