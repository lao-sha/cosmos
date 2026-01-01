# 小六壬解卦数据结构设计

## 一、需求分析

### 1.1 参考资料对比

根据对以下资料的分析：

1. **xuanxue/xiaoliuren/mysterious** - 现代Web应用实现
   - UI展示：六宫卡片、五行属性、六亲关系、神煞标注
   - AI解卦：集成Gemini和Claude，提供结构化解析
   - 核心算法：私有闭源（core.logic.ts）

2. **pallets/divination/xiaoliuren** - 当前区块链实现
   - 完整的类型系统：六宫、五行、阴阳、十二时辰等
   - 支持流派：道家流派、传统流派
   - 排盘算法：时间起课、数字起课、随机起课等
   - 高级分析：体用关系、八卦具象法、三宫具象法

3. **pallets/divination/liuyao** - 六爻解卦参考
   - 分层存储设计：核心指标链上，详细解释链下
   - 枚举索引优化：使用索引而非字符串
   - 实时计算：通过Runtime API免费获取

### 1.2 小六壬解卦的核心要素

小六壬作为简易占卜术，其解卦包括以下层次：

#### **基础层（六宫属性）**
- 六宫名称：大安、留连、速喜、赤口、小吉、空亡
- 五行属性：木、火、土、金、水
- 阴阳属性：阳、阴
- 天将神煞：青龙、玄武、朱雀、白虎、六合、勾陈
- 方位：东、南、西、北、东南、中央
- 吉凶等级：大吉(5)、吉(4)、平(2)、凶(1)

#### **三宫关系层**
- **月宫（天宫）** - 代表事情的起因或背景
- **日宫（地宫）** - 代表事情的经过或现状
- **时宫（人宫）** - 代表事情的结果或未来
- **五行生克关系**：相生、相克、比和、泄气、被克
- **特殊格局**：纯宫（三宫相同）、全吉、全凶

#### **高级分析层（道家流派）**
- **体用关系**：体（人宫）与用（时辰）的五行关系
  - 用生体（大吉）、体克用（小吉）、比肩/比助（中平）
  - 体生用（小凶）、用克体（大凶）
- **八卦具象法**：三宫阴阳转化为八卦
- **十二宫对应**：命宫、事业宫、感情宫等
- **三宫具象法**：天盘、地盘、人盘

## 二、设计原则

### 2.1 存储优化原则（借鉴六爻设计）

1. **分层存储**
   - **链上存储**：核心指标、吉凶等级、关键枚举值
   - **链下生成**：详细文字解释、卦辞、建议

2. **枚举优化**
   - 使用 u8 索引替代长字符串
   - 所有枚举实现 Encode, Decode, MaxEncodedLen

3. **实时计算**
   - 通过 Runtime API 提供免费查询
   - 算法可升级无需数据迁移

### 2.2 灵活性原则

1. **流派支持**：道家/传统流派可切换
2. **扩展性**：预留自定义字段
3. **兼容性**：支持与AI解卦模块集成

## 三、数据结构设计

### 3.1 解卦核心数据（链上存储）

```rust
/// 小六壬解卦核心数据（13 bytes）
///
/// 存储核心指标，链上永久保存
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct XiaoLiuRenInterpretation {
    /// 吉凶等级（1 byte）
    pub ji_xiong_level: JiXiongLevel,

    /// 综合评分（1 byte，0-100分）
    pub overall_score: u8,

    /// 三宫五行关系（1 byte）
    pub wu_xing_relation: WuXingRelation,

    /// 体用关系（可选，1+1 bytes = 2 bytes）
    pub ti_yong_relation: Option<TiYongRelation>,

    /// 八卦索引（可选，1+1 bytes = 2 bytes）
    pub ba_gua: Option<BaGua>,

    /// 特殊格局标记（1 byte）
    pub special_pattern: SpecialPattern,

    /// 建议类型（1 byte）
    pub advice_type: AdviceType,

    /// 流派（1 byte）
    pub school: XiaoLiuRenSchool,

    /// 应期类型（可选，1+1 bytes = 2 bytes）
    pub ying_qi: Option<YingQiType>,

    /// 预留字段（1 byte）
    pub reserved: u8,
}
```

**总大小：13 bytes**

### 3.2 吉凶等级枚举

```rust
/// 吉凶等级（1 byte）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum JiXiongLevel {
    /// 大吉 - 诸事顺遂，心想事成
    #[default]
    DaJi = 0,
    /// 吉 - 事可成，宜进取
    Ji = 1,
    /// 小吉 - 小有所得，不宜大动
    XiaoJi = 2,
    /// 平 - 平稳无波，守成为上
    Ping = 3,
    /// 小凶 - 小有阻碍，谨慎行事
    XiaoXiong = 4,
    /// 凶 - 事难成，宜退守
    Xiong = 5,
    /// 大凶 - 诸事不利，静待时机
    DaXiong = 6,
}
```

### 3.3 特殊格局枚举

```rust
/// 特殊格局（1 byte，使用位标志）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct SpecialPattern(pub u8);

impl SpecialPattern {
    /// 无特殊格局
    pub const NONE: u8 = 0b0000_0000;
    /// 纯宫（三宫相同）
    pub const PURE: u8 = 0b0000_0001;
    /// 全吉（三宫皆吉）
    pub const ALL_AUSPICIOUS: u8 = 0b0000_0010;
    /// 全凶（三宫皆凶）
    pub const ALL_INAUSPICIOUS: u8 = 0b0000_0100;
    /// 五行相生成环
    pub const SHENG_CYCLE: u8 = 0b0000_1000;
    /// 五行相克成环
    pub const KE_CYCLE: u8 = 0b0001_0000;
    /// 阴阳和合（体用阴阳互补）
    pub const YIN_YANG_HARMONY: u8 = 0b0010_0000;
    /// 特殊时辰（子时、午时、卯时、酉时）
    pub const SPECIAL_TIME: u8 = 0b0100_0000;
    /// 预留
    pub const RESERVED: u8 = 0b1000_0000;

    pub fn new() -> Self {
        Self(Self::NONE)
    }

    pub fn is_pure(&self) -> bool {
        self.0 & Self::PURE != 0
    }

    pub fn is_all_auspicious(&self) -> bool {
        self.0 & Self::ALL_AUSPICIOUS != 0
    }

    pub fn is_all_inauspicious(&self) -> bool {
        self.0 & Self::ALL_INAUSPICIOUS != 0
    }

    pub fn set_pure(&mut self) {
        self.0 |= Self::PURE;
    }

    pub fn set_all_auspicious(&mut self) {
        self.0 |= Self::ALL_AUSPICIOUS;
    }

    pub fn set_all_inauspicious(&mut self) {
        self.0 |= Self::ALL_INAUSPICIOUS;
    }
}
```

### 3.4 建议类型枚举

```rust
/// 建议类型（1 byte）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum AdviceType {
    /// 大胆进取 - 大吉时
    #[default]
    JinQu = 0,
    /// 稳步前进 - 吉时
    WenBu = 1,
    /// 守成为主 - 平时
    ShouCheng = 2,
    /// 谨慎观望 - 小凶时
    GuanWang = 3,
    /// 退守待时 - 凶时
    TuiShou = 4,
    /// 静待时机 - 大凶时
    JingDai = 5,
    /// 寻求帮助 - 特殊情况
    XunQiu = 6,
    /// 化解冲克 - 五行不利
    HuaJie = 7,
}
```

### 3.5 应期类型枚举

```rust
/// 应期类型（1 byte）
///
/// 小六壬的应期判断比六爻简单，主要根据六宫属性
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub enum YingQiType {
    /// 即刻应验 - 速喜
    #[default]
    JiKe = 0,
    /// 当日应验 - 大安、小吉
    DangRi = 1,
    /// 数日应验 - 需要3-7天
    ShuRi = 2,
    /// 延迟应验 - 留连，需要10天以上
    YanChi = 3,
    /// 难以应验 - 空亡
    NanYi = 4,
    /// 需要化解 - 赤口
    XuHuaJie = 5,
}
```

## 四、算法实现

### 4.1 解卦算法核心函数

```rust
/// 解卦核心算法
///
/// 根据三宫结果、时辰、流派计算解卦数据
pub fn interpret(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
    school: XiaoLiuRenSchool,
) -> XiaoLiuRenInterpretation {
    // 1. 计算吉凶等级
    let ji_xiong_level = calculate_ji_xiong_level(san_gong, shi_chen);

    // 2. 计算综合评分
    let overall_score = calculate_overall_score(san_gong, shi_chen);

    // 3. 五行关系分析
    let wu_xing_relation = san_gong.wu_xing_analysis();

    // 4. 体用关系分析（如果有时辰）
    let ti_yong_relation = shi_chen.map(|sc|
        TiYongRelation::calculate(san_gong.shi_gong, sc)
    );

    // 5. 八卦具象分析
    let ba_gua = Some(BaGua::from_san_gong(san_gong));

    // 6. 特殊格局识别
    let special_pattern = identify_special_pattern(san_gong, shi_chen);

    // 7. 建议类型
    let advice_type = determine_advice_type(&ji_xiong_level, &wu_xing_relation);

    // 8. 应期推算
    let ying_qi = calculate_ying_qi(san_gong);

    XiaoLiuRenInterpretation {
        ji_xiong_level,
        overall_score,
        wu_xing_relation,
        ti_yong_relation,
        ba_gua,
        special_pattern,
        advice_type,
        school,
        ying_qi,
        reserved: 0,
    }
}
```

### 4.2 吉凶等级计算

```rust
/// 计算吉凶等级
///
/// 综合考虑：
/// 1. 时宫（结果）的吉凶等级（权重60%）
/// 2. 三宫整体平均等级（权重40%）
/// 3. 特殊格局加成/减分
/// 4. 体用关系影响
fn calculate_ji_xiong_level(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>
) -> JiXiongLevel {
    // 基础分数（1-5）
    let base_score = san_gong.fortune_level();

    // 特殊格局调整
    let pattern_modifier = if san_gong.is_pure() {
        if san_gong.shi_gong.is_auspicious() { 1 } else { -1 }
    } else if san_gong.is_all_auspicious() {
        1
    } else if san_gong.is_all_inauspicious() {
        -1
    } else {
        0
    };

    // 体用关系调整
    let ti_yong_modifier = if let Some(sc) = shi_chen {
        let ti_yong = TiYongRelation::calculate(san_gong.shi_gong, sc);
        match ti_yong {
            TiYongRelation::YongShengTi => 1,  // 大吉
            TiYongRelation::TiKeYong => 0,     // 小吉
            TiYongRelation::BiJian | TiYongRelation::BiZhu => 0, // 中平
            TiYongRelation::TiShengYong => -1, // 小凶
            TiYongRelation::YongKeTi => -2,    // 大凶
        }
    } else {
        0
    };

    // 计算最终分数
    let final_score = (base_score as i8) + pattern_modifier + ti_yong_modifier;

    // 转换为吉凶等级（限制在1-7范围）
    match final_score.clamp(1, 7) {
        7 => JiXiongLevel::DaJi,
        6 => JiXiongLevel::Ji,
        5 => JiXiongLevel::XiaoJi,
        4 => JiXiongLevel::Ping,
        3 => JiXiongLevel::XiaoXiong,
        2 => JiXiongLevel::Xiong,
        _ => JiXiongLevel::DaXiong,
    }
}
```

### 4.3 综合评分计算

```rust
/// 计算综合评分（0-100分）
///
/// 评分维度：
/// 1. 时宫吉凶（40分）
/// 2. 三宫整体（20分）
/// 3. 五行关系（20分）
/// 4. 体用关系（10分）
/// 5. 特殊格局（10分）
fn calculate_overall_score(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>
) -> u8 {
    // 1. 时宫得分（0-40）
    let shi_score = (san_gong.shi_gong.fortune_level() as u16 * 8) as u8;

    // 2. 三宫整体得分（0-20）
    let san_gong_score = (san_gong.fortune_level() as u16 * 4) as u8;

    // 3. 五行关系得分（0-20）
    let wu_xing_score = match san_gong.wu_xing_analysis() {
        WuXingRelation::Sheng => 20,  // 相生
        WuXingRelation::BiHe => 15,   // 比和
        WuXingRelation::XieSheng => 10, // 泄气
        WuXingRelation::Ke => 5,      // 相克
        WuXingRelation::BeiKe => 0,   // 被克
    };

    // 4. 体用关系得分（0-10）
    let ti_yong_score = if let Some(sc) = shi_chen {
        let ti_yong = TiYongRelation::calculate(san_gong.shi_gong, sc);
        match ti_yong {
            TiYongRelation::YongShengTi => 10, // 大吉
            TiYongRelation::TiKeYong => 8,     // 小吉
            TiYongRelation::BiJian => 6,       // 比肩
            TiYongRelation::BiZhu => 5,        // 比助
            TiYongRelation::TiShengYong => 3,  // 小凶
            TiYongRelation::YongKeTi => 0,     // 大凶
        }
    } else {
        5 // 无时辰信息，给予中性分数
    };

    // 5. 特殊格局得分（0-10）
    let pattern_score = if san_gong.is_pure() {
        if san_gong.shi_gong.is_auspicious() { 10 } else { 0 }
    } else if san_gong.is_all_auspicious() {
        10
    } else if san_gong.is_all_inauspicious() {
        0
    } else {
        5
    };

    // 汇总得分（0-100）
    let total = shi_score + san_gong_score + wu_xing_score + ti_yong_score + pattern_score;
    total.min(100)
}
```

### 4.4 特殊格局识别

```rust
/// 识别特殊格局
fn identify_special_pattern(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>
) -> SpecialPattern {
    let mut pattern = SpecialPattern::new();

    // 检查纯宫
    if san_gong.is_pure() {
        pattern.set_pure();
    }

    // 检查全吉/全凶
    if san_gong.is_all_auspicious() {
        pattern.set_all_auspicious();
    } else if san_gong.is_all_inauspicious() {
        pattern.set_all_inauspicious();
    }

    // 检查五行成环
    let wx1 = san_gong.yue_gong.wu_xing();
    let wx2 = san_gong.ri_gong.wu_xing();
    let wx3 = san_gong.shi_gong.wu_xing();

    if wx1.generates() == wx2 && wx2.generates() == wx3 && wx3.generates() == wx1 {
        pattern.0 |= SpecialPattern::SHENG_CYCLE;
    }

    if wx1.restrains() == wx2 && wx2.restrains() == wx3 && wx3.restrains() == wx1 {
        pattern.0 |= SpecialPattern::KE_CYCLE;
    }

    // 检查阴阳和合
    if let Some(sc) = shi_chen {
        let ti_yy = san_gong.shi_gong.yin_yang();
        let yong_yy = sc.yin_yang();
        if ti_yy != yong_yy {
            pattern.0 |= SpecialPattern::YIN_YANG_HARMONY;
        }
    }

    // 检查特殊时辰
    if let Some(sc) = shi_chen {
        if matches!(sc, ShiChen::Zi | ShiChen::Wu | ShiChen::Mao | ShiChen::You) {
            pattern.0 |= SpecialPattern::SPECIAL_TIME;
        }
    }

    pattern
}
```

### 4.5 应期计算

```rust
/// 计算应期类型
fn calculate_ying_qi(san_gong: &SanGong) -> Option<YingQiType> {
    // 主要根据时宫（结果）判断
    let ying_qi = match san_gong.shi_gong {
        LiuGong::SuXi => YingQiType::JiKe,      // 速喜 - 即刻
        LiuGong::DaAn | LiuGong::XiaoJi => YingQiType::DangRi, // 当日
        LiuGong::LiuLian => YingQiType::YanChi,  // 留连 - 延迟
        LiuGong::KongWang => YingQiType::NanYi,  // 空亡 - 难以应验
        LiuGong::ChiKou => YingQiType::XuHuaJie, // 赤口 - 需要化解
    };

    Some(ying_qi)
}
```

### 4.6 建议类型确定

```rust
/// 确定建议类型
fn determine_advice_type(
    ji_xiong_level: &JiXiongLevel,
    wu_xing_relation: &WuXingRelation
) -> AdviceType {
    match ji_xiong_level {
        JiXiongLevel::DaJi => AdviceType::JinQu,
        JiXiongLevel::Ji => AdviceType::WenBu,
        JiXiongLevel::XiaoJi => AdviceType::WenBu,
        JiXiongLevel::Ping => AdviceType::ShouCheng,
        JiXiongLevel::XiaoXiong => AdviceType::GuanWang,
        JiXiongLevel::Xiong => AdviceType::TuiShou,
        JiXiongLevel::DaXiong => AdviceType::JingDai,
    }
}
```

## 五、Runtime API 设计

### 5.1 解卦查询 API

```rust
decl_runtime_apis! {
    /// 小六壬解卦 Runtime API
    pub trait XiaoLiuRenInterpretationApi {
        /// 获取课盘的解卦结果
        ///
        /// # 参数
        /// - `pan_id`: 课盘ID
        ///
        /// # 返回
        /// 解卦核心数据
        fn get_interpretation(pan_id: u64) -> Option<XiaoLiuRenInterpretation>;

        /// 获取详细解卦文本
        ///
        /// # 参数
        /// - `pan_id`: 课盘ID
        /// - `lang`: 语言（"zh", "en"）
        ///
        /// # 返回
        /// JSON格式的详细解卦文本
        fn get_interpretation_text(pan_id: u64, lang: Vec<u8>) -> Option<Vec<u8>>;

        /// 批量获取解卦结果
        fn get_interpretations_batch(pan_ids: Vec<u64>) -> Vec<Option<XiaoLiuRenInterpretation>>;
    }
}
```

### 5.2 解卦文本生成

```rust
/// 生成详细解卦文本（链下调用）
pub fn generate_interpretation_text(
    interpretation: &XiaoLiuRenInterpretation,
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
) -> InterpretationText {
    InterpretationText {
        // 一、总体吉凶
        ji_xiong: JiXiongSection {
            level: interpretation.ji_xiong_level.name(),
            score: interpretation.overall_score,
            summary: generate_ji_xiong_summary(interpretation),
        },

        // 二、三宫详解
        san_gong: SanGongSection {
            yue: generate_gong_detail(san_gong.yue_gong, "月宫（起因）"),
            ri: generate_gong_detail(san_gong.ri_gong, "日宫（过程）"),
            shi: generate_gong_detail(san_gong.shi_gong, "时宫（结果）"),
        },

        // 三、五行生克
        wu_xing: WuXingSection {
            relation: interpretation.wu_xing_relation.name(),
            analysis: generate_wu_xing_analysis(san_gong),
        },

        // 四、体用关系（可选）
        ti_yong: interpretation.ti_yong_relation.map(|ty| {
            TiYongSection {
                relation: ty.name(),
                description: ty.fortune_desc(),
                analysis: generate_ti_yong_analysis(&ty, san_gong.shi_gong, shi_chen),
            }
        }),

        // 五、八卦具象（可选）
        ba_gua: interpretation.ba_gua.map(|bg| {
            BaGuaSection {
                name: bg.name(),
                symbol: bg.symbol(),
                wu_xing: bg.wu_xing().name(),
                description: bg.brief(),
            }
        }),

        // 六、特殊格局
        special: if interpretation.special_pattern.0 != SpecialPattern::NONE {
            Some(generate_special_pattern_text(&interpretation.special_pattern))
        } else {
            None
        },

        // 七、应期推断
        ying_qi: interpretation.ying_qi.map(|yq| {
            YingQiSection {
                type_name: yq.name(),
                description: generate_ying_qi_description(&yq, san_gong),
            }
        }),

        // 八、建议
        advice: AdviceSection {
            type_name: interpretation.advice_type.name(),
            content: generate_advice_content(&interpretation.advice_type, interpretation),
        },
    }
}
```

## 六、存储设计

### 6.1 存储项

```rust
/// 课盘ID -> 解卦数据
///
/// 采用懒加载：首次查询时计算并缓存
#[pallet::storage]
#[pallet::getter(fn interpretations)]
pub type Interpretations<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64, // pan_id
    XiaoLiuRenInterpretation,
>;
```

### 6.2 懒加载机制

```rust
impl<T: Config> Pallet<T> {
    /// 获取解卦数据（懒加载）
    pub fn get_or_create_interpretation(pan_id: u64) -> Option<XiaoLiuRenInterpretation> {
        // 1. 检查缓存
        if let Some(interpretation) = Interpretations::<T>::get(pan_id) {
            return Some(interpretation);
        }

        // 2. 获取课盘
        let pan = Pans::<T>::get(pan_id)?;

        // 3. 计算解卦
        let interpretation = crate::interpretation::interpret(
            &pan.san_gong,
            pan.shi_chen,
            XiaoLiuRenSchool::DaoJia, // 使用道家流派
        );

        // 4. 缓存结果
        Interpretations::<T>::insert(pan_id, interpretation);

        Some(interpretation)
    }
}
```

## 七、与AI解卦的集成

### 7.1 为AI提供结构化数据

```rust
/// 生成AI解卦的提示词数据
pub fn generate_ai_prompt_data(
    pan_id: u64,
    interpretation: &XiaoLiuRenInterpretation,
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
    question: Option<&str>,
) -> serde_json::Value {
    json!({
        "pan_id": pan_id,
        "question": question,
        "basic": {
            "yue_gong": {
                "name": san_gong.yue_gong.name(),
                "wu_xing": san_gong.yue_gong.wu_xing().name(),
                "tian_jiang": san_gong.yue_gong.tian_jiang(),
                "gua_ci": san_gong.yue_gong.gua_ci(),
            },
            "ri_gong": {
                "name": san_gong.ri_gong.name(),
                "wu_xing": san_gong.ri_gong.wu_xing().name(),
                "tian_jiang": san_gong.ri_gong.tian_jiang(),
                "gua_ci": san_gong.ri_gong.gua_ci(),
            },
            "shi_gong": {
                "name": san_gong.shi_gong.name(),
                "wu_xing": san_gong.shi_gong.wu_xing().name(),
                "tian_jiang": san_gong.shi_gong.tian_jiang(),
                "gua_ci": san_gong.shi_gong.gua_ci(),
            },
        },
        "interpretation": {
            "ji_xiong_level": interpretation.ji_xiong_level.name(),
            "overall_score": interpretation.overall_score,
            "wu_xing_relation": interpretation.wu_xing_relation.name(),
            "ti_yong_relation": interpretation.ti_yong_relation.map(|t| t.name()),
            "ba_gua": interpretation.ba_gua.map(|b| b.name()),
            "special_pattern": format_special_pattern(&interpretation.special_pattern),
            "advice_type": interpretation.advice_type.name(),
            "ying_qi": interpretation.ying_qi.map(|y| y.name()),
        },
        "shi_chen": shi_chen.map(|sc| sc.name()),
    })
}
```

## 八、对比总结

### 8.1 与六爻解卦的对比

| 维度 | 小六壬 | 六爻 |
|------|--------|------|
| **复杂度** | 简单（6宫） | 复杂（64卦384爻） |
| **核心数据大小** | 13 bytes | ~20 bytes |
| **用神判断** | 简单（时宫为主） | 复杂（需根据事项选择） |
| **应期推算** | 简单（根据六宫） | 复杂（动爻、伏神、空亡等） |
| **五行分析** | 三宫关系 | 六爻全面分析 |
| **特殊格局** | 8种基础格局 | 数十种复杂格局 |

### 8.2 设计优势

1. **极致优化**：13字节核心数据，比六爻更精简
2. **实时计算**：通过Runtime API免费查询
3. **算法升级**：无需数据迁移
4. **流派支持**：道家/传统流派灵活切换
5. **AI友好**：结构化数据便于AI理解

## 九、实施计划

### 9.1 第一阶段：核心数据结构

- [ ] 实现 `XiaoLiuRenInterpretation` 结构体
- [ ] 实现所有枚举类型
- [ ] 实现 `SpecialPattern` 位标志

### 9.2 第二阶段：算法实现

- [ ] 实现 `interpret()` 核心函数
- [ ] 实现吉凶等级计算
- [ ] 实现综合评分算法
- [ ] 实现特殊格局识别

### 9.3 第三阶段：Runtime API

- [ ] 实现 Runtime API 定义
- [ ] 实现懒加载机制
- [ ] 实现文本生成函数

### 9.4 第四阶段：AI集成

- [ ] 实现AI提示词数据生成
- [ ] 对接 `pallet_divination_ai`
- [ ] 测试AI解卦质量

### 9.5 第五阶段：测试与文档

- [ ] 单元测试
- [ ] 集成测试
- [ ] API文档
- [ ] 用户手册

## 十、附录

### 10.1 参考文献

1. `pallets/divination/xiaoliuren/src/types.rs` - 类型定义
2. `pallets/divination/xiaoliuren/src/algorithm.rs` - 算法实现
3. `pallets/divination/liuyao/src/interpretation.rs` - 六爻解卦参考
4. `xuanxue/xiaoliuren/mysterious/README.md` - Web应用实现

### 10.2 相关文档

- [小六壬流派说明](../src/types.rs#L10-L22)
- [体用关系分析](../src/types.rs#L1041-L1158)
- [八卦具象法](../src/algorithm.rs#L381-L493)
- [六爻解卦设计](../../liuyao/INTERPRETATION_DESIGN.md)
