# 六爻解卦数据结构设计

## 一、设计背景

### 1.1 参考资料分析

#### 参考实现（xuanxue/liuyao）
- **hex.json**: 包含64卦的卦辞和爻辞（周易原文）
- **schema.json**: 定义了完整的六爻排盘结果 JSON Schema
- **divicast**: 包含六神、六亲、神煞等完整实现

#### 现有 Pallet 实现（pallets/divination/liuyao）
- **types.rs**: 已定义完整的排盘数据结构（LiuYaoGua）
- **shensha.rs**: 已实现14种神煞计算
- **algorithm.rs**: 已实现纳甲、世应、六神等核心算法

#### 八字解盘参考（pallets/divination/bazi）
- **interpretation.rs**: 分层设计（Core + Extended）
- 核心指标仅 13 bytes，存储优化
- 支持 Runtime API 实时计算

### 1.2 设计原则

1. **分层存储**: 核心指标链上存储，详细解释链下生成
2. **存储优化**: 使用枚举索引而非字符串
3. **实时计算**: 通过 Runtime API 免费获取解卦
4. **算法可升级**: 无需数据迁移

---

## 二、核心数据结构设计

### 2.1 吉凶判断枚举

```rust
/// 吉凶等级（1 byte）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum JiXiongLevel {
    /// 大吉
    DaJi = 0,
    /// 吉
    Ji = 1,
    /// 小吉
    XiaoJi = 2,
    /// 平
    Ping = 3,
    /// 小凶
    XiaoXiong = 4,
    /// 凶
    Xiong = 5,
    /// 大凶
    DaXiong = 6,
}
```

### 2.2 用神状态枚举

```rust
/// 用神状态（1 byte）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum YongShenState {
    /// 旺相 - 得时得地
    WangXiang = 0,
    /// 休囚 - 失时失地
    XiuQiu = 1,
    /// 动而化进 - 动爻化进神
    DongHuaJin = 2,
    /// 动而化退 - 动爻化退神
    DongHuaTui = 3,
    /// 动而化空 - 动爻化空亡
    DongHuaKong = 4,
    /// 伏藏 - 伏神状态
    FuCang = 5,
    /// 空亡 - 日空或月空
    KongWang = 6,
    /// 入墓 - 入墓库
    RuMu = 7,
    /// 受克 - 被克制
    ShouKe = 8,
    /// 得生 - 被生扶
    DeSheng = 9,
}
```

### 2.3 事项类型枚举

```rust
/// 占问事项类型（1 byte）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ShiXiangType {
    /// 财运
    CaiYun = 0,
    /// 事业
    ShiYe = 1,
    /// 婚姻感情
    HunYin = 2,
    /// 健康
    JianKang = 3,
    /// 考试学业
    KaoShi = 4,
    /// 官司诉讼
    GuanSi = 5,
    /// 出行
    ChuXing = 6,
    /// 寻人寻物
    XunRen = 7,
    /// 天气
    TianQi = 8,
    /// 其他
    QiTa = 9,
}
```

### 2.4 应期类型枚举

```rust
/// 应期类型（1 byte）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum YingQiType {
    /// 近期（日内）
    JinQi = 0,
    /// 短期（月内）
    DuanQi = 1,
    /// 中期（季度内）
    ZhongQi = 2,
    /// 长期（年内）
    ChangQi = 3,
    /// 远期（年后）
    YuanQi = 4,
    /// 不确定
    BuQueDing = 5,
}
```

---

## 三、核心解卦结构（Layer 1）

### 3.1 CoreInterpretation（约 20 bytes）

```rust
/// 六爻核心解卦结果
///
/// 包含六爻占卜的核心判断指标
/// 总大小：约 20 bytes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct LiuYaoCoreInterpretation {
    // ===== 基础判断 (4 bytes) =====

    /// 总体吉凶 (1 byte)
    pub ji_xiong: JiXiongLevel,

    /// 用神六亲 (1 byte) - 根据占问事项确定
    pub yong_shen_qin: LiuQin,

    /// 用神状态 (1 byte)
    pub yong_shen_state: YongShenState,

    /// 用神所在爻位 (1 byte, 0-5, 255=伏神)
    pub yong_shen_pos: u8,

    // ===== 动态分析 (4 bytes) =====

    /// 世爻状态 (1 byte)
    pub shi_yao_state: YongShenState,

    /// 应爻状态 (1 byte)
    pub ying_yao_state: YongShenState,

    /// 动爻数量 (1 byte, 0-6)
    pub dong_yao_count: u8,

    /// 主要动爻位置 (1 byte, 位图)
    pub dong_yao_bitmap: u8,

    // ===== 特殊状态 (4 bytes) =====

    /// 旬空爻位图 (1 byte) - 哪些爻逢空
    pub xun_kong_bitmap: u8,

    /// 月破爻位图 (1 byte) - 哪些爻月破
    pub yue_po_bitmap: u8,

    /// 日冲爻位图 (1 byte) - 哪些爻日冲
    pub ri_chong_bitmap: u8,

    /// 化空/化退位图 (1 byte) - 动爻变化状态
    pub hua_kong_bitmap: u8,

    // ===== 应期与评分 (4 bytes) =====

    /// 应期类型 (1 byte)
    pub ying_qi: YingQiType,

    /// 应期地支 (1 byte, 0-11)
    pub ying_qi_zhi: u8,

    /// 综合评分 (1 byte, 0-100)
    pub score: u8,

    /// 可信度 (1 byte, 0-100)
    pub confidence: u8,

    // ===== 元数据 (4 bytes) =====

    /// 解卦时间戳 - 区块号 (4 bytes)
    pub timestamp: u32,
}
```

---

## 四、扩展解卦结构（Layer 2）

### 4.1 爻位分析

```rust
/// 单爻解析结果
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct YaoAnalysis {
    /// 爻位 (0-5)
    pub position: u8,

    /// 旺衰状态 (1 byte)
    pub wang_shuai: YongShenState,

    /// 是否逢空 (1 bit)
    pub is_kong: bool,

    /// 是否月破 (1 bit)
    pub is_yue_po: bool,

    /// 是否日冲 (1 bit)
    pub is_ri_chong: bool,

    /// 是否动爻 (1 bit)
    pub is_dong: bool,

    /// 动爻变化类型 (如果是动爻)
    pub hua_type: Option<HuaType>,

    /// 神煞列表 (最多4个)
    pub shen_sha: BoundedVec<ShenSha, ConstU32<4>>,
}

/// 动爻变化类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum HuaType {
    /// 化进神 - 变爻五行生本爻
    HuaJin = 0,
    /// 化退神 - 变爻五行克本爻
    HuaTui = 1,
    /// 化回头生 - 变爻生本爻
    HuaHuiTouSheng = 2,
    /// 化回头克 - 变爻克本爻
    HuaHuiTouKe = 3,
    /// 化空亡 - 变爻逢空
    HuaKong = 4,
    /// 化墓 - 变爻入墓
    HuaMu = 5,
    /// 化绝 - 变爻逢绝
    HuaJue = 6,
}
```

### 4.2 六亲分析

```rust
/// 六亲状态分析
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct LiuQinAnalysis {
    /// 父母爻状态
    pub fu_mu: QinState,
    /// 兄弟爻状态
    pub xiong_di: QinState,
    /// 子孙爻状态
    pub zi_sun: QinState,
    /// 妻财爻状态
    pub qi_cai: QinState,
    /// 官鬼爻状态
    pub guan_gui: QinState,
}

/// 单个六亲状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct QinState {
    /// 出现次数 (0-6)
    pub count: u8,
    /// 爻位列表 (位图)
    pub positions: u8,
    /// 是否有伏神
    pub has_fu_shen: bool,
    /// 伏神位置 (如果有)
    pub fu_shen_pos: u8,
    /// 整体旺衰
    pub wang_shuai: YongShenState,
}
```

### 4.3 卦象分析

```rust
/// 卦象综合分析
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct GuaXiangAnalysis {
    /// 本卦卦名索引 (0-63)
    pub ben_gua_idx: u8,

    /// 变卦卦名索引 (0-63, 255=无变卦)
    pub bian_gua_idx: u8,

    /// 互卦卦名索引 (0-63)
    pub hu_gua_idx: u8,

    /// 卦宫 (0-7)
    pub gong: u8,

    /// 卦序 (0-7)
    pub gua_xu: u8,

    /// 世爻位置 (1-6)
    pub shi_pos: u8,

    /// 应爻位置 (1-6)
    pub ying_pos: u8,

    /// 卦身地支 (0-11)
    pub gua_shen: u8,

    /// 本卦五行
    pub ben_gua_wuxing: WuXing,

    /// 变卦五行 (如果有)
    pub bian_gua_wuxing: Option<WuXing>,
}
```

---

## 五、完整解卦结构

### 5.1 FullInterpretation

```rust
/// 六爻完整解卦结果
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct LiuYaoFullInterpretation {
    /// 核心指标（必有）
    pub core: LiuYaoCoreInterpretation,

    /// 卦象分析（必有）
    pub gua_xiang: GuaXiangAnalysis,

    /// 六亲分析（可选）
    pub liu_qin: Option<LiuQinAnalysis>,

    /// 各爻分析（可选，最多6个）
    pub yao_analysis: Option<BoundedVec<YaoAnalysis, ConstU32<6>>>,

    /// 神煞汇总（可选）
    pub shen_sha_summary: Option<ShenShaSummary>,
}

/// 神煞汇总
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ShenShaSummary {
    /// 吉神列表（最多8个）
    pub ji_shen: BoundedVec<(ShenSha, u8), ConstU32<8>>,

    /// 凶煞列表（最多8个）
    pub xiong_sha: BoundedVec<(ShenSha, u8), ConstU32<8>>,
}
```

---

## 六、解卦文本枚举

### 6.1 解卦文本类型

```rust
/// 解卦文本类型枚举
///
/// 用于链上存储解卦文本索引，前端根据索引显示对应文本
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum JieGuaTextType {
    // ===== 吉凶总断 (0-6) =====
    /// 大吉：诸事顺遂，心想事成
    DaJiZongDuan = 0,
    /// 吉：事可成，宜进取
    JiZongDuan = 1,
    /// 小吉：小有所得，不宜大动
    XiaoJiZongDuan = 2,
    /// 平：平稳无波，守成为上
    PingZongDuan = 3,
    /// 小凶：小有阻碍，谨慎行事
    XiaoXiongZongDuan = 4,
    /// 凶：事难成，宜退守
    XiongZongDuan = 5,
    /// 大凶：诸事不利，静待时机
    DaXiongZongDuan = 6,

    // ===== 用神状态 (7-16) =====
    /// 用神旺相：所求之事有望
    YongShenWangXiang = 7,
    /// 用神休囚：所求之事难成
    YongShenXiuQiu = 8,
    /// 用神动而化进：事情向好发展
    YongShenHuaJin = 9,
    /// 用神动而化退：事情有退步之象
    YongShenHuaTui = 10,
    /// 用神逢空：所求之事虚而不实
    YongShenKong = 11,
    /// 用神入墓：事情受阻，需待时机
    YongShenRuMu = 12,
    /// 用神伏藏：所求之事隐而未显
    YongShenFuCang = 13,
    /// 用神受克：所求之事受阻
    YongShenShouKe = 14,
    /// 用神得生：所求之事有贵人相助
    YongShenDeSheng = 15,
    /// 用神发动：事情有变化
    YongShenFaDong = 16,

    // ===== 世应关系 (17-22) =====
    /// 世应相生：双方和谐，事易成
    ShiYingXiangSheng = 17,
    /// 世应相克：双方有冲突
    ShiYingXiangKe = 18,
    /// 世应比和：双方势均力敌
    ShiYingBiHe = 19,
    /// 世爻旺应爻衰：我强彼弱
    ShiWangYingShuai = 20,
    /// 世爻衰应爻旺：我弱彼强
    ShiShuaiYingWang = 21,
    /// 世应俱空：双方皆虚
    ShiYingJuKong = 22,

    // ===== 动爻断语 (23-28) =====
    /// 无动爻：事情平稳，无大变化
    WuDongYao = 23,
    /// 一爻独发：事情明确，吉凶易断
    YiYaoDuFa = 24,
    /// 多爻齐动：事情复杂，变数较多
    DuoYaoQiDong = 25,
    /// 六爻皆动：大变之象，需谨慎
    LiuYaoJieDong = 26,
    /// 动爻化进：事情向好发展
    DongYaoHuaJin = 27,
    /// 动爻化退：事情有退步之象
    DongYaoHuaTui = 28,

    // ===== 特殊状态 (29-34) =====
    /// 用神逢日冲：近期有变
    YongShenRiChong = 29,
    /// 用神逢月破：本月不利
    YongShenYuePo = 30,
    /// 卦逢六冲：事情难成或有变
    GuaFengLiuChong = 31,
    /// 卦逢六合：事情顺利
    GuaFengLiuHe = 32,
    /// 反吟卦：事情反复
    FanYinGua = 33,
    /// 伏吟卦：事情停滞
    FuYinGua = 34,

    // ===== 应期断语 (35-40) =====
    /// 应期在日：近日可见分晓
    YingQiZaiRi = 35,
    /// 应期在月：本月可见分晓
    YingQiZaiYue = 36,
    /// 应期在季：本季可见分晓
    YingQiZaiJi = 37,
    /// 应期在年：年内可见分晓
    YingQiZaiNian = 38,
    /// 应期待冲：待冲空之日
    YingQiDaiChong = 39,
    /// 应期待合：待合之日
    YingQiDaiHe = 40,
}
```

---

## 七、Runtime API 设计

### 7.1 解卦 API

```rust
sp_api::decl_runtime_apis! {
    /// 六爻解卦 Runtime API
    pub trait LiuYaoInterpretationApi {
        /// 获取核心解卦（免费实时计算）
        fn get_core_interpretation(
            gua_id: u64,
            shi_xiang: ShiXiangType,
        ) -> Option<LiuYaoCoreInterpretation>;

        /// 获取完整解卦（免费实时计算）
        fn get_full_interpretation(
            gua_id: u64,
            shi_xiang: ShiXiangType,
        ) -> Option<LiuYaoFullInterpretation>;

        /// 获取解卦文本索引列表
        fn get_interpretation_texts(
            gua_id: u64,
            shi_xiang: ShiXiangType,
        ) -> Option<BoundedVec<JieGuaTextType, ConstU32<20>>>;

        /// 获取卦辞爻辞（基于卦象索引）
        fn get_gua_ci(gua_idx: u8) -> Option<GuaCi>;

        /// 获取爻辞（基于卦象索引和爻位）
        fn get_yao_ci(gua_idx: u8, yao_pos: u8) -> Option<YaoCi>;
    }
}
```

### 7.2 卦辞爻辞结构

```rust
/// 卦辞（链下存储，通过 API 获取）
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct GuaCi {
    /// 卦名
    pub name: BoundedVec<u8, ConstU32<16>>,
    /// 卦辞原文
    pub ci: BoundedVec<u8, ConstU32<256>>,
    /// 白话解释
    pub bai_hua: BoundedVec<u8, ConstU32<512>>,
}

/// 爻辞
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct YaoCi {
    /// 爻位名称（初九、六二等）
    pub name: BoundedVec<u8, ConstU32<8>>,
    /// 爻辞原文
    pub ci: BoundedVec<u8, ConstU32<128>>,
    /// 白话解释
    pub bai_hua: BoundedVec<u8, ConstU32<256>>,
}
```

---

## 八、存储优化分析

### 8.1 存储大小估算

| 结构 | 大小 | 说明 |
|------|------|------|
| LiuYaoCoreInterpretation | ~20 bytes | 核心指标 |
| GuaXiangAnalysis | ~12 bytes | 卦象分析 |
| LiuQinAnalysis | ~30 bytes | 六亲分析 |
| YaoAnalysis × 6 | ~60 bytes | 各爻分析 |
| ShenShaSummary | ~40 bytes | 神煞汇总 |
| **总计** | **~162 bytes** | 完整解卦 |

### 8.2 与八字解盘对比

| 项目 | 八字 | 六爻 | 说明 |
|------|------|------|------|
| 核心指标 | 13 bytes | 20 bytes | 六爻信息更多 |
| 完整解盘 | ~50 bytes | ~162 bytes | 六爻结构更复杂 |
| 实时计算 | ✅ | ✅ | 都支持 Runtime API |

---

## 九、实现计划

### 9.1 第一阶段：核心结构
1. 定义所有枚举类型
2. 实现 LiuYaoCoreInterpretation
3. 实现基础解卦算法

### 9.2 第二阶段：扩展结构
1. 实现 GuaXiangAnalysis
2. 实现 LiuQinAnalysis
3. 实现 YaoAnalysis

### 9.3 第三阶段：Runtime API
1. 实现解卦 API
2. 添加卦辞爻辞数据
3. 前端集成

---

## 十、解卦算法核心逻辑

### 10.1 用神确定规则

```
占财运 → 用神为妻财
占事业 → 用神为官鬼
占婚姻 → 男占用妻财，女占用官鬼
占健康 → 用神为世爻
占考试 → 用神为父母
占官司 → 用神为官鬼
占出行 → 用神为世爻
占寻人 → 用神为用事之爻
```

### 10.2 旺衰判断规则

```
1. 月建生扶 → 旺
2. 月建克制 → 衰
3. 日辰生扶 → 有气
4. 日辰克制 → 无气
5. 动爻生扶 → 得助
6. 动爻克制 → 受克
```

### 10.3 吉凶判断规则

```
用神旺相 + 无克制 → 吉
用神休囚 + 有克制 → 凶
用神逢空 → 事虚
用神入墓 → 事阻
用神化进 → 事进
用神化退 → 事退
```

### 10.4 应期推算规则

```
用神旺相 → 应期在生旺之时
用神休囚 → 应期在生扶之时
用神逢空 → 应期在冲空之日
用神入墓 → 应期在冲墓之日
动爻 → 应期在合或冲之日
```

---

## 十一、总结

本设计参考了：
1. **xuanxue/liuyao** 的卦辞爻辞数据和排盘 Schema
2. **pallets/divination/liuyao** 的现有类型定义和算法
3. **pallets/divination/bazi** 的分层存储设计

核心特点：
- **分层设计**：Core（20 bytes）+ Extended（~142 bytes）
- **枚举索引**：所有文本使用枚举索引，前端映射显示
- **实时计算**：通过 Runtime API 免费获取解卦
- **可扩展性**：支持不同占问事项的定制解卦
