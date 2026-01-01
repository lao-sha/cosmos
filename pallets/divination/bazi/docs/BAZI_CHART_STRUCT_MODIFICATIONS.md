# BaziChart 结构体修改建议

## 📋 当前状态分析

### create_bazi_chart 接口（已修改）

```rust
pub fn create_bazi_chart(
    origin: OriginFor<T>,
    name: Option<BoundedVec<u8, ConstU32<32>>>,
    input: BaziInputType,  // ← 统一输入枚举
    gender: Gender,
    zishi_mode: ZiShiMode,
    longitude: Option<i32>,  // ← 有值即启用真太阳时
) -> DispatchResult
```

### BaziInputType 枚举（已实现）

```rust
pub enum BaziInputType {
    Solar { year, month, day, hour, minute },
    Lunar { year, month, day, is_leap_month, hour, minute },
    SiZhu { year_gz, month_gz, day_gz, hour_gz, birth_year },
}
```

### BaziChart 结构体（当前）

```rust
pub struct BaziChart<T: Config> {
    pub owner: T::AccountId,
    pub name: BoundedVec<u8, ConstU32<32>>,
    pub birth_time: BirthTime,        // ← 存储公历时间
    pub gender: Gender,
    pub zishi_mode: ZiShiMode,
    pub longitude: Option<i32>,       // ← 已有，用于真太阳时
    pub sizhu: SiZhu<T>,
    pub dayun: DaYunInfo<T>,
    pub wuxing_strength: WuXingStrength,
    pub xiyong_shen: Option<WuXing>,
    pub timestamp: u64,
}
```

---

## 🎯 需要修改的地方

### 问题 1：缺少历法类型字段

**现状**：
- `birth_time` 统一存储公历时间
- 无法区分用户输入的是公历还是农历
- 用户查看命盘时可能困惑

**影响**：
```
用户输入：农历 1990年四月廿一
链上存储：1990-05-15（公历）
用户查看：1990年5月15日 ← 用户困惑："我输入的是农历啊？"
```

### 问题 2：直接四柱输入的 birth_time 处理

**现状**：
- `BaziInputType::SiZhu` 没有具体日期
- 但 `BaziChart.birth_time` 是必填字段
- 需要存储虚拟值或使用 `Option`

**影响**：
```rust
// 直接四柱输入时，birth_time 应该存什么？
BaziInputType::SiZhu {
    year_gz: 0,   // 甲子
    month_gz: 2,  // 丙寅
    day_gz: 4,    // 戊辰
    hour_gz: 0,   // 甲子
    birth_year: 1984,
}
// → birth_time = ??? (没有具体日期)
```

---

## ✅ 推荐修改方案

### 方案：添加历法类型字段 + birth_time 改为可选

```rust
/// 历法类型（输入来源）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum CalendarType {
    /// 公历输入
    Solar = 0,
    /// 农历输入（已转换为公历存储）
    Lunar = 1,
    /// 直接四柱输入（无具体日期）
    Direct = 2,
}

impl Default for CalendarType {
    fn default() -> Self {
        Self::Solar
    }
}

/// 完整八字信息（修改后）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct BaziChart<T: crate::pallet::Config> {
    /// 所有者账户
    pub owner: T::AccountId,
    
    /// 命盘名称（可选，最大32字节UTF-8）
    pub name: BoundedVec<u8, ConstU32<32>>,
    
    /// 出生时间（公历）
    /// - Solar/Lunar: 存储公历时间
    /// - Direct: 存储虚拟时间或 None
    pub birth_time: BirthTime,
    
    /// 历法类型（新增）
    /// 记录用户输入的类型，用于前端智能展示
    pub calendar_type: CalendarType,
    
    /// 性别
    pub gender: Gender,
    
    /// 子时模式
    pub zishi_mode: ZiShiMode,
    
    /// 出生地经度（可选，1/100000 度）
    /// 有值时自动使用真太阳时修正
    pub longitude: Option<i32>,
    
    /// 四柱
    pub sizhu: SiZhu<T>,
    
    /// 大运
    pub dayun: DaYunInfo<T>,
    
    /// 五行强度
    pub wuxing_strength: WuXingStrength,
    
    /// 喜用神
    pub xiyong_shen: Option<WuXing>,
    
    /// 创建时间戳（区块号）
    pub timestamp: u64,
}
```

---

## 🔧 create_bazi_chart 函数修改

### 需要在构建 BaziChart 时设置 calendar_type

```rust
pub fn create_bazi_chart(
    origin: OriginFor<T>,
    name: Option<BoundedVec<u8, ConstU32<32>>>,
    input: BaziInputType,
    gender: Gender,
    zishi_mode: ZiShiMode,
    longitude: Option<i32>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // ... 验证和计算逻辑 ...
    
    // 根据输入类型确定历法类型
    let calendar_type = match input {
        BaziInputType::Solar { .. } => CalendarType::Solar,
        BaziInputType::Lunar { .. } => CalendarType::Lunar,
        BaziInputType::SiZhu { .. } => CalendarType::Direct,
    };
    
    // 构建八字信息
    let bazi_chart = BaziChart {
        owner: who.clone(),
        name: name.unwrap_or_default(),
        birth_time,
        calendar_type,  // ← 新增字段
        gender,
        zishi_mode,
        longitude,
        sizhu,
        dayun: dayun_info,
        wuxing_strength,
        xiyong_shen,
        timestamp: frame_system::Pallet::<T>::block_number().saturated_into(),
    };
    
    // ... 存储逻辑 ...
}
```

---

## 📊 修改对比

### 存储大小变化

| 字段 | 修改前 | 修改后 | 差异 |
|------|--------|--------|------|
| calendar_type | - | 1 byte | +1 byte |
| **总计** | ~841 bytes | ~842 bytes | +0.12% |

**结论**：存储成本几乎可忽略。

---

## 🎨 前端展示优化

### 智能展示逻辑

```typescript
function displayBirthInfo(chart: BaziChart) {
    switch (chart.calendar_type) {
        case "Solar":
            // 公历输入 → 主显示公历
            return {
                primary: formatSolar(chart.birth_time),
                secondary: formatLunar(solarToLunar(chart.birth_time)),
                label: "出生日期（公历）"
            };
        
        case "Lunar":
            // 农历输入 → 主显示农历
            const lunar = solarToLunar(chart.birth_time);
            return {
                primary: formatLunar(lunar),
                secondary: formatSolar(chart.birth_time),
                label: "出生日期（农历）"
            };
        
        case "Direct":
            // 直接四柱 → 只显示四柱
            return {
                primary: formatSiZhu(chart.sizhu),
                secondary: null,
                label: "四柱"
            };
    }
}

// 示例输出
// Solar:  "1990年5月15日 14:30（公历）"
// Lunar:  "农历1990年四月廿一 14:30"
// Direct: "庚午年 辛巳月 甲子日 辛未时"
```

---

## 🔄 PartialEq 实现更新

```rust
impl<T: crate::pallet::Config> PartialEq for BaziChart<T> {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner &&
        self.name == other.name &&
        self.birth_time == other.birth_time &&
        self.calendar_type == other.calendar_type &&  // ← 新增
        self.gender == other.gender &&
        self.zishi_mode == other.zishi_mode &&
        self.longitude == other.longitude &&
        self.sizhu == other.sizhu &&
        self.dayun == other.dayun &&
        self.wuxing_strength == other.wuxing_strength &&
        self.xiyong_shen == other.xiyong_shen &&
        self.timestamp == other.timestamp
    }
}
```

---

## 🧪 测试用例更新

### 需要验证 calendar_type 正确设置

```rust
#[test]
fn test_calendar_type_solar() {
    new_test_ext().execute_with(|| {
        let input = BaziInputType::Solar {
            year: 1990, month: 5, day: 15,
            hour: 14, minute: 30,
        };
        
        assert_ok!(BaziChart::create_bazi_chart(
            RuntimeOrigin::signed(ALICE),
            None,
            input,
            Gender::Male,
            ZiShiMode::Modern,
            None,
        ));
        
        let chart = ChartById::<Test>::get(0).unwrap();
        assert_eq!(chart.calendar_type, CalendarType::Solar);
    });
}

#[test]
fn test_calendar_type_lunar() {
    new_test_ext().execute_with(|| {
        let input = BaziInputType::Lunar {
            year: 1990, month: 4, day: 21,
            is_leap_month: false,
            hour: 14, minute: 30,
        };
        
        assert_ok!(BaziChart::create_bazi_chart(
            RuntimeOrigin::signed(ALICE),
            None,
            input,
            Gender::Male,
            ZiShiMode::Modern,
            None,
        ));
        
        let chart = ChartById::<Test>::get(0).unwrap();
        assert_eq!(chart.calendar_type, CalendarType::Lunar);
        // birth_time 应该是转换后的公历时间
        assert_eq!(chart.birth_time.year, 1990);
        assert_eq!(chart.birth_time.month, 5);
        assert_eq!(chart.birth_time.day, 15);
    });
}

#[test]
fn test_calendar_type_direct() {
    new_test_ext().execute_with(|| {
        let input = BaziInputType::SiZhu {
            year_gz: 0,   // 甲子
            month_gz: 2,  // 丙寅
            day_gz: 4,    // 戊辰
            hour_gz: 0,   // 甲子
            birth_year: 1984,
        };
        
        assert_ok!(BaziChart::create_bazi_chart(
            RuntimeOrigin::signed(ALICE),
            None,
            input,
            Gender::Male,
            ZiShiMode::Modern,
            None,
        ));
        
        let chart = ChartById::<Test>::get(0).unwrap();
        assert_eq!(chart.calendar_type, CalendarType::Direct);
    });
}
```

---

## 📝 文档更新

### BaziChart 结构体文档

```rust
/// 完整八字信息
///
/// # 字段说明
///
/// - `owner`: 命盘所有者账户
/// - `name`: 命盘名称（可选，如"张三"、"父亲命盘"）
/// - `birth_time`: 出生时间（统一存储为公历）
/// - `calendar_type`: 历法类型（记录用户输入的类型）
///   - `Solar`: 用户输入公历
///   - `Lunar`: 用户输入农历（已转换为公历存储）
///   - `Direct`: 用户直接输入四柱（birth_time 可能为虚拟值）
/// - `gender`: 性别（影响大运顺逆）
/// - `zishi_mode`: 子时模式（影响 23:00-23:59 的时柱）
/// - `longitude`: 出生地经度（有值时使用真太阳时修正）
/// - `sizhu`: 四柱（年月日时柱）
/// - `dayun`: 大运信息
/// - `wuxing_strength`: 五行强度
/// - `xiyong_shen`: 喜用神
/// - `timestamp`: 创建时间戳（区块号）
///
/// # 存储大小
///
/// 约 842 bytes
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct BaziChart<T: crate::pallet::Config> {
    // ...
}
```

---

## 🎯 总结

### 必须修改的地方

1. ✅ **添加 `calendar_type` 字段**
   - 类型：`CalendarType` 枚举
   - 大小：1 byte
   - 位置：在 `birth_time` 之后

2. ✅ **更新 `create_bazi_chart` 函数**
   - 根据 `input` 类型设置 `calendar_type`
   - 构建 `BaziChart` 时包含新字段

3. ✅ **更新 `PartialEq` 实现**
   - 添加 `calendar_type` 的比较

4. ✅ **添加测试用例**
   - 验证三种输入类型的 `calendar_type` 正确设置

### 可选修改的地方

1. ⭐ **birth_time 改为 Option**（如果需要严格区分）
   ```rust
   pub birth_time: Option<BirthTime>,  // Direct 时为 None
   ```
   - 优点：语义更清晰
   - 缺点：增加复杂度，需要处理 None 情况

2. ⭐ **扩展 CalendarType 记录闰月**
   ```rust
   pub enum CalendarType {
       Solar,
       Lunar { is_leap: bool },  // 记录是否闰月
       Direct,
   }
   ```
   - 优点：保留更多原始信息
   - 缺点：增加 1 byte

### 推荐实施顺序

1. **Phase 1**：添加 `CalendarType` 枚举和 `calendar_type` 字段
2. **Phase 2**：更新 `create_bazi_chart` 函数
3. **Phase 3**：更新 `PartialEq` 实现
4. **Phase 4**：添加测试用例
5. **Phase 5**：更新文档

---

## ⚠️ 迁移注意事项

### 现有数据兼容性

如果链上已有数据，需要考虑迁移：

```rust
// 方案 A：使用 Option（推荐）
pub calendar_type: Option<CalendarType>,  // 旧数据为 None

// 方案 B：使用 Default
impl Default for CalendarType {
    fn default() -> Self {
        Self::Solar  // 旧数据默认为公历
    }
}

// 方案 C：版本化存储
pub enum BaziChartVersion<T: Config> {
    V1(BaziChartV1<T>),  // 旧版本
    V2(BaziChartV2<T>),  // 新版本（含 calendar_type）
}
```

**推荐**：如果是测试网或新链，直接修改。如果是主网，使用方案 C 版本化存储。

---

**结论**：`BaziChart` 结构体必须添加 `calendar_type` 字段，以配合 `create_bazi_chart` 接口的修改，提升用户体验和数据完整性。
