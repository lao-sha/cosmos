# 六爻模块改进与扩展实施计划

## 概述

基于对比分析结果，本文档规划了六爻pallet的改进和扩展步骤。

**总工作量估计**: 约20小时

---

## 第一阶段：高优先级改进 (P0) - 约6小时

### P0-1: 添加时间起卦 Extrinsic

**目标**: 暴露已实现的`time_to_yaos`函数为链上可调用方法

**修改文件**: `src/lib.rs`

**实施步骤**:

```rust
// 1. 在 lib.rs 的 #[pallet::call] impl 中添加（约第470行后）

/// 时间起卦 - 根据年月日时起卦
///
/// # 参数
/// - `year_zhi`: 年地支索引 (0-11，子=0)
/// - `month_num`: 月数 (1-12)
/// - `day_num`: 日数 (1-31)
/// - `hour_zhi`: 时辰地支索引 (0-11)
/// - `year_gz`: 年干支（完整干支用于排盘）
/// - `month_gz`: 月干支
/// - `day_gz`: 日干支
/// - `hour_gz`: 时干支
#[pallet::call_index(7)]
#[pallet::weight(Weight::from_parts(100_000_000, 0))]
pub fn divine_by_time(
    origin: OriginFor<T>,
    year_zhi: u8,
    month_num: u8,
    day_num: u8,
    hour_zhi: u8,
    year_gz: (u8, u8),
    month_gz: (u8, u8),
    day_gz: (u8, u8),
    hour_gz: (u8, u8),
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    // 参数校验
    ensure!(year_zhi < 12, Error::<T>::InvalidNumber);
    ensure!(month_num >= 1 && month_num <= 12, Error::<T>::InvalidNumber);
    ensure!(day_num >= 1 && day_num <= 31, Error::<T>::InvalidNumber);
    ensure!(hour_zhi < 12, Error::<T>::InvalidNumber);

    // 检查每日限制
    Self::check_daily_limit(&who)?;

    // 调用时间起卦算法
    let yaos = time_to_yaos(year_zhi, month_num, day_num, hour_zhi);

    // 执行排卦
    let gua_id = Self::do_divine(
        &who,
        yaos,
        DivinationMethod::TimeMethod,
        (TianGan::from_index(year_gz.0), DiZhi::from_index(year_gz.1)),
        (TianGan::from_index(month_gz.0), DiZhi::from_index(month_gz.1)),
        (TianGan::from_index(day_gz.0), DiZhi::from_index(day_gz.1)),
        (TianGan::from_index(hour_gz.0), DiZhi::from_index(hour_gz.1)),
    )?;

    // 更新每日计数
    Self::increment_daily_count(&who);

    // 发出事件
    let gua = Guas::<T>::get(gua_id).ok_or(Error::<T>::GuaNotFound)?;
    Self::deposit_event(Event::GuaCreated {
        gua_id,
        creator: who,
        method: DivinationMethod::TimeMethod,
        original_name_idx: gua.original_name_idx,
    });

    Ok(())
}
```

**添加测试** (`src/tests.rs`):

```rust
#[test]
fn test_divine_by_time_works() {
    new_test_ext().execute_with(|| {
        // 甲辰年六月十一日未时
        assert_ok!(Liuyao::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            4,   // 辰年
            6,   // 六月
            11,  // 十一日
            7,   // 未时
            (0, 4),  // 甲辰
            (6, 6),  // 庚午
            (9, 5),  // 癸巳
            (5, 7),  // 己未
        ));

        let gua = Liuyao::guas(0).unwrap();
        assert_eq!(gua.method, DivinationMethod::TimeMethod);
    });
}

#[test]
fn test_divine_by_time_invalid_params() {
    new_test_ext().execute_with(|| {
        // 无效月份
        assert_noop!(
            Liuyao::divine_by_time(
                RuntimeOrigin::signed(ALICE),
                4, 13, 11, 7,  // month=13 无效
                (0, 4), (6, 6), (9, 5), (5, 7),
            ),
            Error::<Test>::InvalidNumber
        );
    });
}
```

---

### P0-2: 添加互卦字段

**目标**: 在`LiuYaoGua`结构体中存储互卦信息

**修改文件**: `src/types.rs`

**实施步骤**:

```rust
// 在 LiuYaoGua 结构体中添加（约第666行后，changed_name_idx之后）

    // ===== 互卦信息 =====
    /// 互卦内卦（取2,3,4爻组成）
    pub hu_inner: Trigram,
    /// 互卦外卦（取3,4,5爻组成）
    pub hu_outer: Trigram,
    /// 互卦卦名索引 (0-63)
    pub hu_name_idx: u8,
```

---

### P0-3: 添加卦身字段

**目标**: 在`LiuYaoGua`结构体中存储卦身地支

**修改文件**: `src/types.rs`

**实施步骤**:

```rust
// 在 LiuYaoGua 结构体中添加（紧接互卦字段之后）

    // ===== 卦身 =====
    /// 卦身地支
    /// 阳爻从子起数到世爻位置，阴爻从午起数
    pub gua_shen: DiZhi,
```

---

### P0-4: 修改 do_divine 计算并存储互卦和卦身

**目标**: 在排卦时计算并保存互卦和卦身

**修改文件**: `src/lib.rs`

**实施步骤**:

```rust
// 在 do_divine 函数中，约第750行（计算动爻位图之后）添加：

    // 计算动爻位图
    let moving_yaos = calculate_moving_bitmap(&yaos);

    // ===== 新增：计算互卦 =====
    let (hu_inner, hu_outer) = calculate_hu_gua(&yaos);
    let hu_name_idx = calculate_gua_index(hu_inner, hu_outer);

    // ===== 新增：计算卦身 =====
    let shi_pos = gua_xu.shi_yao_pos();
    let shi_is_yang = yaos[(shi_pos - 1) as usize].is_yang();
    let gua_shen = calculate_gua_shen(shi_pos, shi_is_yang);

    // 查找伏神
    let fu_shen = find_fu_shen(gong, &liu_qin_array);

// 在创建 LiuYaoGua 结构体时添加新字段（约第770行）：

    let gua = LiuYaoGua {
        // ... 现有字段 ...
        changed_name_idx,

        // ===== 新增字段 =====
        hu_inner,
        hu_outer,
        hu_name_idx,
        gua_shen,

        moving_yaos,
        // ... 其余字段 ...
    };
```

**添加测试**:

```rust
#[test]
fn test_hu_gua_stored() {
    new_test_ext().execute_with(|| {
        // 创建乾为天卦
        let yaos = [1, 1, 1, 1, 1, 1]; // 全阳爻
        assert_ok!(Liuyao::divine_manual(
            RuntimeOrigin::signed(ALICE),
            yaos,
            (0, 0), (2, 2), (4, 4), (6, 6),
        ));

        let gua = Liuyao::guas(0).unwrap();

        // 乾为天的互卦是乾为天
        // 2,3,4爻 = 111 = 乾
        // 3,4,5爻 = 111 = 乾
        assert_eq!(gua.hu_inner, Trigram::Qian);
        assert_eq!(gua.hu_outer, Trigram::Qian);
        assert_eq!(gua.hu_name_idx, 63); // 乾为天索引
    });
}

#[test]
fn test_gua_shen_stored() {
    new_test_ext().execute_with(|| {
        // 乾为天卦，世爻在六爻，阳爻
        // 阳爻从子起数：子(0)+5=巳(5)
        let yaos = [1, 1, 1, 1, 1, 1];
        assert_ok!(Liuyao::divine_manual(
            RuntimeOrigin::signed(ALICE),
            yaos,
            (0, 0), (2, 2), (4, 4), (6, 6),
        ));

        let gua = Liuyao::guas(0).unwrap();
        // 世爻在6，阳爻从子起：(0+6-1)%12 = 5 = 巳
        assert_eq!(gua.gua_shen, DiZhi::Si);
    });
}
```

---

## 第二阶段：中优先级扩展 (P1) - 约7小时

### P1-1: 添加5种缺失神煞

**目标**: 添加天喜、天医、阳刃、灾煞、谋星

**修改文件**: `src/shensha.rs`

**实施步骤**:

```rust
// 1. 扩展 ShenSha 枚举（约第28行）

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum ShenSha {
    // 现有9种
    TianYiGuiRen = 0,
    YiMa = 1,
    TaoHua = 2,
    LuShen = 3,
    WenChang = 4,
    JieSha = 5,
    HuaGai = 6,
    JiangXing = 7,
    WangShen = 8,

    // 新增5种
    /// 天喜 - 喜庆吉事
    TianXi = 9,
    /// 天医 - 医药健康
    TianYi = 10,
    /// 阳刃 - 刚猛凶险
    YangRen = 11,
    /// 灾煞 - 疾病灾祸
    ZaiSha = 12,
    /// 谋星 - 谋略智慧
    MouXing = 13,
}

// 2. 更新 name() 方法
impl ShenSha {
    pub fn name(&self) -> &'static str {
        match self {
            // ... 现有的 ...
            Self::TianXi => "天喜",
            Self::TianYi => "天医",
            Self::YangRen => "阳刃",
            Self::ZaiSha => "灾煞",
            Self::MouXing => "谋星",
        }
    }

    pub fn is_auspicious(&self) -> bool {
        matches!(
            self,
            Self::TianYiGuiRen | Self::LuShen | Self::WenChang
            | Self::JiangXing | Self::TianXi | Self::TianYi
        )
    }

    pub fn is_inauspicious(&self) -> bool {
        matches!(self, Self::JieSha | Self::WangShen | Self::YangRen | Self::ZaiSha)
    }
}

// 3. 添加计算函数

// ============================================================================
// 天喜
// ============================================================================

/// 天喜查询表（按日支）
///
/// 口诀：子午卯酉在酉，寅申巳亥在午，辰戌丑未在卯
const TIAN_XI: [DiZhi; 12] = [
    DiZhi::You,  // 子 -> 酉
    DiZhi::Mao,  // 丑 -> 卯
    DiZhi::Wu,   // 寅 -> 午
    DiZhi::Zi,   // 卯 -> 子（修正：卯在子）
    DiZhi::Mao,  // 辰 -> 卯
    DiZhi::Wu,   // 巳 -> 午
    DiZhi::You,  // 午 -> 酉
    DiZhi::Mao,  // 未 -> 卯
    DiZhi::Wu,   // 申 -> 午
    DiZhi::You,  // 酉 -> 酉（修正：酉在酉）
    DiZhi::Mao,  // 戌 -> 卯
    DiZhi::Wu,   // 亥 -> 午
];

/// 计算天喜
pub fn calculate_tian_xi(day_zhi: DiZhi) -> DiZhi {
    TIAN_XI[day_zhi.index() as usize]
}

pub fn is_tian_xi(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_tian_xi(day_zhi) == zhi
}

// ============================================================================
// 天医
// ============================================================================

/// 天医查询表（按月支）
///
/// 口诀：正月在丑，二月在寅...月建后一位
const TIAN_YI_MEDICAL: [DiZhi; 12] = [
    DiZhi::Chou, // 子月(11月) -> 丑
    DiZhi::Yin,  // 丑月(12月) -> 寅
    DiZhi::Mao,  // 寅月(正月) -> 卯
    DiZhi::Chen, // 卯月(二月) -> 辰
    DiZhi::Si,   // 辰月(三月) -> 巳
    DiZhi::Wu,   // 巳月(四月) -> 午
    DiZhi::Wei,  // 午月(五月) -> 未
    DiZhi::Shen, // 未月(六月) -> 申
    DiZhi::You,  // 申月(七月) -> 酉
    DiZhi::Xu,   // 酉月(八月) -> 戌
    DiZhi::Hai,  // 戌月(九月) -> 亥
    DiZhi::Zi,   // 亥月(十月) -> 子
];

/// 计算天医（按月支）
pub fn calculate_tian_yi_medical(month_zhi: DiZhi) -> DiZhi {
    TIAN_YI_MEDICAL[month_zhi.index() as usize]
}

pub fn is_tian_yi_medical(month_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_tian_yi_medical(month_zhi) == zhi
}

// ============================================================================
// 阳刃
// ============================================================================

/// 阳刃查询表（按日干）
///
/// 口诀：甲刃在卯，乙刃在寅，丙戊刃在午，丁己刃在巳，
///       庚刃在酉，辛刃在申，壬刃在子，癸刃在亥
const YANG_REN: [DiZhi; 10] = [
    DiZhi::Mao,  // 甲 -> 卯
    DiZhi::Yin,  // 乙 -> 寅（有争议，有说辰）
    DiZhi::Wu,   // 丙 -> 午
    DiZhi::Si,   // 丁 -> 巳
    DiZhi::Wu,   // 戊 -> 午
    DiZhi::Si,   // 己 -> 巳
    DiZhi::You,  // 庚 -> 酉
    DiZhi::Shen, // 辛 -> 申（有争议，有说戌）
    DiZhi::Zi,   // 壬 -> 子
    DiZhi::Hai,  // 癸 -> 亥
];

/// 计算阳刃
pub fn calculate_yang_ren(day_gan: TianGan) -> DiZhi {
    YANG_REN[day_gan.index() as usize]
}

pub fn is_yang_ren(day_gan: TianGan, zhi: DiZhi) -> bool {
    calculate_yang_ren(day_gan) == zhi
}

// ============================================================================
// 灾煞
// ============================================================================

/// 灾煞查询表（按日支三合局）
///
/// 口诀：申子辰见午，寅午戌见子，巳酉丑见卯，亥卯未见酉
const ZAI_SHA: [DiZhi; 12] = [
    DiZhi::Wu,   // 子 -> 午（申子辰）
    DiZhi::Mao,  // 丑 -> 卯（巳酉丑）
    DiZhi::Zi,   // 寅 -> 子（寅午戌）
    DiZhi::You,  // 卯 -> 酉（亥卯未）
    DiZhi::Wu,   // 辰 -> 午（申子辰）
    DiZhi::Mao,  // 巳 -> 卯（巳酉丑）
    DiZhi::Zi,   // 午 -> 子（寅午戌）
    DiZhi::You,  // 未 -> 酉（亥卯未）
    DiZhi::Wu,   // 申 -> 午（申子辰）
    DiZhi::Mao,  // 酉 -> 卯（巳酉丑）
    DiZhi::Zi,   // 戌 -> 子（寅午戌）
    DiZhi::You,  // 亥 -> 酉（亥卯未）
];

/// 计算灾煞
pub fn calculate_zai_sha(day_zhi: DiZhi) -> DiZhi {
    ZAI_SHA[day_zhi.index() as usize]
}

pub fn is_zai_sha(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_zai_sha(day_zhi) == zhi
}

// ============================================================================
// 谋星
// ============================================================================

/// 谋星查询表（按日支三合局）
///
/// 口诀：申子辰见巳，寅午戌见亥，巳酉丑见寅，亥卯未见申
const MOU_XING: [DiZhi; 12] = [
    DiZhi::Si,   // 子 -> 巳（申子辰）
    DiZhi::Yin,  // 丑 -> 寅（巳酉丑）
    DiZhi::Hai,  // 寅 -> 亥（寅午戌）
    DiZhi::Shen, // 卯 -> 申（亥卯未）
    DiZhi::Si,   // 辰 -> 巳（申子辰）
    DiZhi::Yin,  // 巳 -> 寅（巳酉丑）
    DiZhi::Hai,  // 午 -> 亥（寅午戌）
    DiZhi::Shen, // 未 -> 申（亥卯未）
    DiZhi::Si,   // 申 -> 巳（申子辰）
    DiZhi::Yin,  // 酉 -> 寅（巳酉丑）
    DiZhi::Hai,  // 戌 -> 亥（寅午戌）
    DiZhi::Shen, // 亥 -> 申（亥卯未）
];

/// 计算谋星
pub fn calculate_mou_xing(day_zhi: DiZhi) -> DiZhi {
    MOU_XING[day_zhi.index() as usize]
}

pub fn is_mou_xing(day_zhi: DiZhi, zhi: DiZhi) -> bool {
    calculate_mou_xing(day_zhi) == zhi
}

// 4. 更新 ShenShaInfo 结构体（约第473行）

#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct ShenShaInfo {
    // ... 现有字段 ...
    pub wang_shen: DiZhi,

    // 新增字段
    pub tian_xi: DiZhi,
    pub tian_yi: DiZhi,
    pub yang_ren: DiZhi,
    pub zai_sha: DiZhi,
    pub mou_xing: DiZhi,
}

// 5. 更新 calculate_all_shen_sha（约第504行）

pub fn calculate_all_shen_sha(
    day_gan: TianGan,
    day_zhi: DiZhi,
    month_zhi: DiZhi,  // 新增参数
) -> ShenShaInfo {
    ShenShaInfo {
        // ... 现有计算 ...
        wang_shen: calculate_wang_shen(day_zhi),

        // 新增计算
        tian_xi: calculate_tian_xi(day_zhi),
        tian_yi: calculate_tian_yi_medical(month_zhi),
        yang_ren: calculate_yang_ren(day_gan),
        zai_sha: calculate_zai_sha(day_zhi),
        mou_xing: calculate_mou_xing(day_zhi),
    }
}

// 6. 更新 get_shen_sha_for_zhi（约第527行）

pub fn get_shen_sha_for_zhi(
    day_gan: TianGan,
    day_zhi: DiZhi,
    month_zhi: DiZhi,  // 新增参数
    target_zhi: DiZhi,
) -> [Option<ShenSha>; 14] {  // 扩展数组大小
    let mut result: [Option<ShenSha>; 14] = [None; 14];
    let mut idx = 0;

    // ... 现有9种神煞检查 ...

    // 新增5种
    if is_tian_xi(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::TianXi);
        idx += 1;
    }
    if is_tian_yi_medical(month_zhi, target_zhi) {
        result[idx] = Some(ShenSha::TianYi);
        idx += 1;
    }
    if is_yang_ren(day_gan, target_zhi) {
        result[idx] = Some(ShenSha::YangRen);
        idx += 1;
    }
    if is_zai_sha(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::ZaiSha);
        idx += 1;
    }
    if is_mou_xing(day_zhi, target_zhi) {
        result[idx] = Some(ShenSha::MouXing);
    }

    result
}
```

**添加测试**:

```rust
#[test]
fn test_tian_xi_calculation() {
    // 子午卯酉在酉
    assert_eq!(calculate_tian_xi(DiZhi::Zi), DiZhi::You);
    assert_eq!(calculate_tian_xi(DiZhi::Wu), DiZhi::You);

    // 寅申巳亥在午
    assert_eq!(calculate_tian_xi(DiZhi::Yin), DiZhi::Wu);
    assert_eq!(calculate_tian_xi(DiZhi::Shen), DiZhi::Wu);

    // 辰戌丑未在卯
    assert_eq!(calculate_tian_xi(DiZhi::Chen), DiZhi::Mao);
}

#[test]
fn test_yang_ren_calculation() {
    assert_eq!(calculate_yang_ren(TianGan::Jia), DiZhi::Mao);
    assert_eq!(calculate_yang_ren(TianGan::Bing), DiZhi::Wu);
    assert_eq!(calculate_yang_ren(TianGan::Geng), DiZhi::You);
    assert_eq!(calculate_yang_ren(TianGan::Ren), DiZhi::Zi);
}

#[test]
fn test_zai_sha_calculation() {
    // 申子辰见午
    assert_eq!(calculate_zai_sha(DiZhi::Zi), DiZhi::Wu);
    assert_eq!(calculate_zai_sha(DiZhi::Shen), DiZhi::Wu);

    // 寅午戌见子
    assert_eq!(calculate_zai_sha(DiZhi::Yin), DiZhi::Zi);
    assert_eq!(calculate_zai_sha(DiZhi::Wu), DiZhi::Zi);
}

#[test]
fn test_mou_xing_calculation() {
    // 申子辰见巳
    assert_eq!(calculate_mou_xing(DiZhi::Zi), DiZhi::Si);

    // 寅午戌见亥
    assert_eq!(calculate_mou_xing(DiZhi::Yin), DiZhi::Hai);
}
```

---

### P1-2: 集成神煞信息到卦象

**目标**: 在查询卦象时能够获取神煞信息

**方案选择**:
- **方案A**: 存储在LiuYaoGua中（增加存储成本）
- **方案B**: 提供RPC查询接口（推荐）

**采用方案B - 添加查询方法**:

**修改文件**: `src/lib.rs`

```rust
// 在 impl<T: Config> Pallet<T> 中添加公开查询方法

impl<T: Config> Pallet<T> {
    // ... 现有方法 ...

    /// 查询卦象的神煞信息
    ///
    /// # 参数
    /// - `gua_id`: 卦象ID
    ///
    /// # 返回
    /// 神煞信息（如果卦象存在）
    pub fn get_shen_sha_info(gua_id: u64) -> Option<ShenShaInfo> {
        let gua = Guas::<T>::get(gua_id)?;

        Some(calculate_all_shen_sha(
            gua.day_gz.0,
            gua.day_gz.1,
            gua.month_gz.1,  // 月支
        ))
    }

    /// 查询单爻携带的神煞
    ///
    /// # 参数
    /// - `gua_id`: 卦象ID
    /// - `yao_pos`: 爻位置 (0-5，初爻到上爻)
    ///
    /// # 返回
    /// 该爻携带的神煞列表
    pub fn get_yao_shen_sha(gua_id: u64, yao_pos: u8) -> Option<[Option<ShenSha>; 14]> {
        if yao_pos > 5 {
            return None;
        }

        let gua = Guas::<T>::get(gua_id)?;
        let yao_zhi = gua.original_yaos[yao_pos as usize].di_zhi;

        Some(get_shen_sha_for_zhi(
            gua.day_gz.0,
            gua.day_gz.1,
            gua.month_gz.1,
            yao_zhi,
        ))
    }
}
```

---

## 第三阶段：低优先级扩展 (P2) - 约7小时

### P2-1: 添加床帐和香闺神煞

**目标**: 基于卦身计算床帐（卦身所生）和香闺（卦身所克）

**修改文件**: `src/shensha.rs`

```rust
// 添加床帐香闺计算

/// 计算床帐（卦身所生的地支）
///
/// 床帐是卦身地支所生的五行对应的地支
pub fn calculate_chuang_zhang(gua_shen: DiZhi) -> Vec<DiZhi> {
    let gua_shen_wx = gua_shen.wu_xing();
    let sheng_wx = gua_shen_wx.generates();  // 我生

    // 返回属于该五行的所有地支
    let mut result = Vec::new();
    for i in 0..12 {
        let zhi = DiZhi::from_index(i);
        if zhi.wu_xing() == sheng_wx {
            result.push(zhi);
        }
    }
    result
}

/// 计算香闺（卦身所克的地支）
///
/// 香闺是卦身地支所克的五行对应的地支
pub fn calculate_xiang_gui(gua_shen: DiZhi) -> Vec<DiZhi> {
    let gua_shen_wx = gua_shen.wu_xing();
    let ke_wx = gua_shen_wx.restrains();  // 我克

    let mut result = Vec::new();
    for i in 0..12 {
        let zhi = DiZhi::from_index(i);
        if zhi.wu_xing() == ke_wx {
            result.push(zhi);
        }
    }
    result
}

/// 判断地支是否为床帐
pub fn is_chuang_zhang(gua_shen: DiZhi, target_zhi: DiZhi) -> bool {
    let sheng_wx = gua_shen.wu_xing().generates();
    target_zhi.wu_xing() == sheng_wx
}

/// 判断地支是否为香闺
pub fn is_xiang_gui(gua_shen: DiZhi, target_zhi: DiZhi) -> bool {
    let ke_wx = gua_shen.wu_xing().restrains();
    target_zhi.wu_xing() == ke_wx
}
```

**添加测试**:

```rust
#[test]
fn test_chuang_zhang_calculation() {
    // 卦身在子（水），水生木，床帐在寅卯
    let cz = calculate_chuang_zhang(DiZhi::Zi);
    assert!(cz.contains(&DiZhi::Yin));
    assert!(cz.contains(&DiZhi::Mao));
}

#[test]
fn test_xiang_gui_calculation() {
    // 卦身在子（水），水克火，香闺在巳午
    let xg = calculate_xiang_gui(DiZhi::Zi);
    assert!(xg.contains(&DiZhi::Si));
    assert!(xg.contains(&DiZhi::Wu));
}
```

---

### P2-2: 添加卦辞爻辞数据支持

**目标**: 支持通过IPFS存储和引用卦辞爻辞数据

**修改文件**: `src/lib.rs` 和 `src/types.rs`

**实施步骤**:

```rust
// 1. 在 types.rs 的 LiuYaoGua 中添加字段

    // ===== 卦辞引用 =====
    /// 卦辞数据CID（IPFS地址）
    pub gua_ci_cid: Option<BoundedVec<u8, MaxCidLen>>,

// 2. 在 lib.rs 添加存储项

    /// 全局卦辞数据CID（由管理员设置）
    #[pallet::storage]
    #[pallet::getter(fn gua_ci_data_cid)]
    pub type GuaCiDataCid<T: Config> = StorageValue<_, BoundedVec<u8, T::MaxCidLen>>;

// 3. 添加设置卦辞CID的Extrinsic

    /// 设置全局卦辞数据CID（仅root可调用）
    #[pallet::call_index(8)]
    #[pallet::weight(Weight::from_parts(10_000_000, 0))]
    pub fn set_gua_ci_cid(
        origin: OriginFor<T>,
        cid: BoundedVec<u8, T::MaxCidLen>,
    ) -> DispatchResult {
        ensure_root(origin)?;
        GuaCiDataCid::<T>::put(cid);
        Ok(())
    }
```

**卦辞数据格式建议** (JSON):

```json
{
  "乾为天": {
    "gua_ci": "元亨利贞",
    "tuan": "大哉乾元，万物资始，乃统天...",
    "xiang": "天行健，君子以自强不息",
    "yao_ci": [
      "初九：潜龙勿用",
      "九二：见龙在田，利见大人",
      "九三：君子终日乾乾，夕惕若厉，无咎",
      "九四：或跃在渊，无咎",
      "九五：飞龙在天，利见大人",
      "上九：亢龙有悔"
    ]
  },
  // ... 64卦
}
```

---

### P2-3: 清理废弃代码

**目标**: 移除或隔离已废弃的AI解读代码

**修改文件**: `src/lib.rs`

**方案A - 完全移除**（推荐）:
```rust
// 删除以下内容：
// - request_ai_interpretation (call_index 4)
// - submit_ai_interpretation (call_index 5)
// - AiInterpretationRequests 存储项
// - AiInterpretationRequested 事件
// - AiInterpretationSubmitted 事件
// - AiInterpretationAlreadyRequested 错误
// - AiInterpretationNotRequested 错误
// - ai_interpretation_cid 字段（保留但标记废弃）
```

**方案B - 条件编译**:
```rust
#[cfg(feature = "deprecated-ai")]
#[pallet::call_index(4)]
pub fn request_ai_interpretation(...) { ... }
```

---

### P2-4: 添加完整测试用例

**目标**: 确保新功能有完整的测试覆盖

**修改文件**: `src/tests.rs`

**需要添加的测试**:

1. `test_divine_by_time_works` - 时间起卦正常流程
2. `test_divine_by_time_invalid_params` - 时间起卦参数校验
3. `test_hu_gua_stored` - 互卦存储验证
4. `test_gua_shen_stored` - 卦身存储验证
5. `test_tian_xi_calculation` - 天喜计算
6. `test_tian_yi_calculation` - 天医计算
7. `test_yang_ren_calculation` - 阳刃计算
8. `test_zai_sha_calculation` - 灾煞计算
9. `test_mou_xing_calculation` - 谋星计算
10. `test_chuang_zhang_calculation` - 床帐计算
11. `test_xiang_gui_calculation` - 香闺计算
12. `test_get_shen_sha_info` - 神煞查询接口
13. `test_get_yao_shen_sha` - 单爻神煞查询

---

## 实施顺序建议

```
Week 1 (P0 - 高优先级):
├── Day 1-2: P0-1 时间起卦
├── Day 2-3: P0-2/3 互卦和卦身字段
└── Day 3-4: P0-4 修改do_divine + 测试

Week 2 (P1 - 中优先级):
├── Day 1-3: P1-1 添加5种神煞
└── Day 3-4: P1-2 神煞查询接口

Week 3 (P2 - 低优先级):
├── Day 1: P2-1 床帐香闺
├── Day 2: P2-2 卦辞支持
├── Day 3: P2-3 清理废弃代码
└── Day 4: P2-4 完整测试
```

---

## 验收标准

### P0阶段完成标准
- [ ] `divine_by_time` Extrinsic 可正常调用
- [ ] 卦象数据包含 `hu_inner`, `hu_outer`, `hu_name_idx`
- [ ] 卦象数据包含 `gua_shen`
- [ ] 所有现有测试通过
- [ ] 新增测试覆盖率 > 90%

### P1阶段完成标准
- [ ] `ShenSha` 枚举包含14种神煞
- [ ] `calculate_all_shen_sha` 返回完整神煞信息
- [ ] `get_shen_sha_info` 和 `get_yao_shen_sha` 可正常调用
- [ ] 神煞测试全部通过

### P2阶段完成标准
- [ ] 床帐香闺计算函数实现
- [ ] 卦辞CID存储和查询功能
- [ ] 废弃代码清理完成
- [ ] 完整测试套件通过
- [ ] 文档更新完成

---

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| LiuYaoGua结构体变更导致存储迁移 | 高 | 添加runtime migration |
| 神煞口诀有多种版本 | 中 | 参考主流实现，添加注释说明 |
| 时间起卦参数复杂 | 低 | 详细文档和前端辅助 |
| 测试覆盖不完整 | 中 | 强制code review |

---

## 参考资料

- Python najia库: `/home/xiaodong/文档/stardust/xuanxue/liuyao/najia`
- Python divicast库: `/home/xiaodong/文档/stardust/xuanxue/liuyao/divicast`
- 神煞口诀来源: 《增删卜易》《卜筮正宗》
