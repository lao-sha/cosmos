# create_bazi_chart 接口设计：可行性与合理性分析

## 📋 提议的接口参数

```rust
pub fn create_bazi_chart(
    origin: OriginFor<T>,
    // 1. 姓名
    name: BoundedVec<u8, ConstU32<64>>,
    // 2. 性别
    gender: Gender,
    // 3. 出生时间（支持公历/农历/直接四柱）
    birth_input: BirthTimeInput,
    // 4. 出生地经纬度
    location: Option<GeoLocation>,
    // 5. 是否使用真太阳时
    use_true_solar_time: bool,
    // 6. 子时模式
    zishi_mode: ZiShiMode,
) -> DispatchResult
```

---

## 一、各参数详细分析

### 1. 姓名 (name)

#### 数据结构
```rust
/// 姓名（UTF-8 编码，最多 64 字节）
pub type BaziName = BoundedVec<u8, ConstU32<64>>;
```

#### 可行性分析

| 维度 | 评估 | 说明 |
|------|------|------|
| **技术可行性** | ✅ 完全可行 | `BoundedVec` 是 Substrate 标准类型 |
| **存储成本** | ✅ 可控 | 64 字节上限，中文约 21 个字符 |
| **编码兼容** | ✅ 支持 | UTF-8 支持所有语言 |
| **验证复杂度** | ⚠️ 中等 | 需验证 UTF-8 有效性 |

#### 合理性分析

| 方面 | 评分 | 理由 |
|------|------|------|
| **业务必要性** | ⭐⭐⭐⭐ | 命盘需要标识，但可用 ID 替代 |
| **隐私风险** | ⚠️ 高 | 真实姓名上链，永久公开 |
| **用户体验** | ⭐⭐⭐⭐⭐ | 便于识别和管理多个命盘 |
| **国际化** | ⭐⭐⭐⭐⭐ | UTF-8 支持全球语言 |

#### 建议

**方案 A：可选姓名（推荐）**
```rust
name: Option<BoundedVec<u8, ConstU32<64>>>
```
- 用户可选择填写昵称/备注，而非真实姓名
- 默认 `None`，系统自动生成 "命盘 #12345"

**方案 B：前端加密（隐私优先）**
```rust
encrypted_name: Option<BoundedVec<u8, ConstU32<96>>>  // 加密后更长
```
- 前端使用用户密钥加密姓名
- 链上仅存储密文

---

### 2. 性别 (gender)

#### 数据结构（已存在）
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum Gender {
    Male = 1,
    Female = 0,
}
```

#### 可行性与合理性

| 维度 | 评估 | 说明 |
|------|------|------|
| **技术可行性** | ✅ 已实现 | 现有代码已支持 |
| **业务必要性** | ⭐⭐⭐⭐⭐ | 大运起运年龄计算必需 |
| **存储成本** | ✅ 1 byte | 极低 |
| **隐私风险** | ⚠️ 中等 | 性别信息相对敏感 |

**结论**：必需参数，无需修改。

---

### 3. 出生时间 (birth_input)

#### 数据结构（新设计）
```rust
/// 统一的出生时间输入类型
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum BirthTimeInput {
    /// 公历输入
    Solar {
        year: u16,      // 1900-2100
        month: u8,      // 1-12
        day: u8,        // 1-31
        hour: u8,       // 0-23
        minute: u8,     // 0-59
    },
    /// 农历输入
    Lunar {
        year: u16,      // 1901-2100
        month: u8,      // 1-12
        day: u8,        // 1-30
        is_leap: bool,  // 是否闰月
        hour: u8,       // 0-23
        minute: u8,     // 0-59
    },
    /// 直接四柱输入
    DirectPillars {
        year_pillar: u8,   // 年柱索引 0-59
        month_pillar: u8,  // 月柱索引 0-59
        day_pillar: u8,    // 日柱索引 0-59
        hour_pillar: u8,   // 时柱索引 0-59
    },
}
```

#### 可行性分析

| 方面 | 评估 | 说明 |
|------|------|------|
| **公历计算** | ✅ 已实现 | `calculate_*_ganzhi()` 函数完整 |
| **农历转换** | ✅ 已实现 | `almanac::lunar_to_solar()` 可用 |
| **直接四柱** | ✅ 可行 | `GanZhi::from_index()` 支持 |
| **存储大小** | ✅ 7 bytes | 枚举标签 1 byte + 最大变体 6 bytes |

#### 合理性分析

| 优点 | 缺点 |
|------|------|
| ✅ 统一接口，减少重复代码 | ⚠️ 枚举增加复杂度 |
| ✅ 支持三种主流输入方式 | ⚠️ 权重计算需分支处理 |
| ✅ 扩展性强（可加时间戳） | ⚠️ 前端需要类型判断 |

**结论**：高度合理，建议采用。

---

### 4. 出生地经纬度 (location)

#### 数据结构（新设计）
```rust
/// 地理位置（经纬度）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct GeoLocation {
    /// 经度（东经为正，西经为负）
    /// 精度：0.0001° ≈ 11 米
    /// 范围：-180.0000° ~ +180.0000°
    /// 存储：i32 = 经度 × 10000
    /// 例如：东经 116.4074° = 1164074
    pub longitude: i32,
    
    /// 纬度（北纬为正，南纬为负）
    /// 精度：0.0001° ≈ 11 米
    /// 范围：-90.0000° ~ +90.0000°
    /// 存储：i32 = 纬度 × 10000
    /// 例如：北纬 39.9042° = 399042
    pub latitude: i32,
}

impl GeoLocation {
    /// 验证经纬度有效性
    pub fn is_valid(&self) -> bool {
        self.longitude >= -1800000 && self.longitude <= 1800000 &&
        self.latitude >= -900000 && self.latitude <= 900000
    }
    
    /// 获取浮点数经度
    pub fn longitude_f64(&self) -> f64 {
        self.longitude as f64 / 10000.0
    }
    
    /// 获取浮点数纬度
    pub fn latitude_f64(&self) -> f64 {
        self.latitude as f64 / 10000.0
    }
}
```

#### 可行性分析

| 维度 | 评估 | 说明 |
|------|------|------|
| **技术可行性** | ✅ 完全可行 | 标准数据类型，8 bytes |
| **精度要求** | ✅ 充分 | 0.0001° ≈ 11 米，远超需求 |
| **计算复杂度** | ⚠️ 中等 | 需实现真太阳时算法 |
| **存储成本** | ✅ 低 | 8 bytes 固定大小 |

#### 合理性分析

**支持理由**：
1. **传统命理需求**：古代用真太阳时，现代派有此需求
2. **专业用户价值**：命理师需要精确计算
3. **技术可实现**：算法成熟，计算量小

**反对理由**：
1. **普通用户困惑**：99% 用户不理解真太阳时
2. **数据获取困难**：用户可能不知道出生地经纬度
3. **隐私风险**：精确位置信息敏感
4. **实际影响小**：对大多数地区，时差 < 30 分钟，对四柱影响极小

#### 建议

**方案 A：可选参数（推荐）**
```rust
location: Option<GeoLocation>
```
- 默认 `None`，使用北京时间（东经 120°）
- 专业用户可填写精确经纬度

**方案 B：仅存储经度**
```rust
longitude: Option<i32>  // 仅 4 bytes
```
- 真太阳时只需经度，不需纬度
- 减少存储和隐私风险

**方案 C：前端计算（最优）**
```rust
// 链上不存储经纬度，前端传入修正后的时间
adjusted_hour: u8,
adjusted_minute: u8,
```
- 前端根据经纬度计算真太阳时修正
- 链上只存储修正后的时间
- 隐私保护 + 简化链上逻辑

---

### 5. 是否使用真太阳时 (use_true_solar_time)

#### 数据结构
```rust
pub use_true_solar_time: bool  // 1 byte
```

#### 可行性分析

| 维度 | 评估 | 说明 |
|------|------|------|
| **技术可行性** | ✅ 可行 | 需实现均时差算法 |
| **计算复杂度** | ⚠️ 中等 | 涉及浮点运算和三角函数 |
| **依赖关系** | ⚠️ 强依赖 | 必须配合 `location` 使用 |

#### 真太阳时计算算法

```rust
/// 计算真太阳时修正（分钟）
pub fn calculate_true_solar_correction(
    longitude: i32,      // 经度 × 10000
    year: u16,
    month: u8,
    day: u8,
) -> i16 {
    // 1. 经度修正：与东经 120° 的差值，每度 4 分钟
    let longitude_deg = longitude as f64 / 10000.0;
    let longitude_correction = ((longitude_deg - 120.0) * 4.0) as i16;
    
    // 2. 均时差修正（地球椭圆轨道引起）
    let day_of_year = calculate_day_of_year(year, month, day);
    let b = 2.0 * PI * (day_of_year as f64 - 81.0) / 365.0;
    
    // 简化公式（精度 ±1 分钟）
    let equation_of_time = (9.87 * (2.0 * b).sin() 
                          - 7.53 * b.cos() 
                          - 1.5 * b.sin()) as i16;
    
    longitude_correction + equation_of_time
}
```

#### 合理性分析

| 方面 | 评分 | 理由 |
|------|------|------|
| **业务必要性** | ⭐⭐ | 仅专业用户需要 |
| **用户理解度** | ⭐ | 普通用户完全不懂 |
| **实际影响** | ⭐⭐ | 对四柱影响小（除非在时辰边界） |
| **实现成本** | ⭐⭐⭐ | 需要浮点运算库 |

#### 建议

**不建议作为独立参数**，原因：
1. 与 `location` 强耦合，逻辑冗余
2. 增加用户困惑
3. 前端计算更合理

**替代方案**：
```rust
// 方案 1：自动判断
if location.is_some() {
    // 自动启用真太阳时修正
}

// 方案 2：前端预处理
// 前端根据用户选择计算修正后的时间，直接传入
```

---

### 6. 子时模式 (zishi_mode)

#### 数据结构（已存在）
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ZiShiMode {
    Traditional = 1,  // 23:00-23:59 属于次日
    Modern = 2,       // 23:00-23:59 属于当日
}
```

#### 可行性与合理性

| 维度 | 评估 | 说明 |
|------|------|------|
| **技术可行性** | ✅ 已实现 | 现有代码已支持 |
| **业务必要性** | ⭐⭐⭐⭐ | 23 点出生者必需 |
| **用户理解度** | ⭐⭐⭐ | 需要说明，但可理解 |
| **存储成本** | ✅ 1 byte | 极低 |

#### 建议

**保留，但提供默认值**：
```rust
zishi_mode: Option<ZiShiMode>  // None 时默认 Modern
```

---

## 二、综合接口设计建议

### 推荐方案：分层设计

```rust
/// 基础创建接口（简化版）
#[pallet::call_index(0)]
pub fn create_bazi_chart(
    origin: OriginFor<T>,
    name: Option<BoundedVec<u8, ConstU32<64>>>,
    gender: Gender,
    birth_input: BirthTimeInput,
) -> DispatchResult {
    Self::create_bazi_chart_with_options(
        origin,
        name,
        gender,
        birth_input,
        None,  // 使用默认选项
    )
}

/// 高级创建接口（完整版）
#[pallet::call_index(1)]
pub fn create_bazi_chart_with_options(
    origin: OriginFor<T>,
    name: Option<BoundedVec<u8, ConstU32<64>>>,
    gender: Gender,
    birth_input: BirthTimeInput,
    options: Option<BaziOptions>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    let opts = options.unwrap_or_default();
    
    // 验证姓名（如果提供）
    if let Some(ref n) = name {
        ensure!(
            sp_std::str::from_utf8(n).is_ok(),
            Error::<T>::InvalidName
        );
    }
    
    // 根据 birth_input 类型处理
    let (year, month, day, hour, minute) = match birth_input {
        BirthTimeInput::Solar { year, month, day, hour, minute } => {
            (year, month, day, hour, minute)
        },
        BirthTimeInput::Lunar { year, month, day, is_leap, hour, minute } => {
            // 农历转公历
            let (solar_year, solar_month, solar_day) = 
                pallet_almanac::lunar::lunar_to_solar(year, month, day, is_leap)
                    .ok_or(Error::<T>::InvalidLunarDate)?;
            (solar_year, solar_month, solar_day, hour, minute)
        },
        BirthTimeInput::DirectPillars { .. } => {
            // 直接使用四柱，跳过时间验证
            return Self::create_from_pillars(who, name, gender, birth_input);
        },
    };
    
    // 真太阳时修正（如果启用）
    let (final_hour, final_minute) = if let Some(ref loc) = opts.location {
        let correction = calculate_true_solar_correction(
            loc.longitude,
            year,
            month,
            day,
        );
        apply_time_correction(hour, minute, correction)
    } else {
        (hour, minute)
    };
    
    // 继续现有的四柱计算逻辑...
    // ...
}

/// 高级选项
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct BaziOptions {
    /// 出生地经度（仅用于真太阳时修正）
    pub location: Option<GeoLocation>,
    /// 子时模式（默认：现代派）
    pub zishi_mode: ZiShiMode,
}

impl Default for BaziOptions {
    fn default() -> Self {
        Self {
            location: None,
            zishi_mode: ZiShiMode::Modern,
        }
    }
}
```

---

## 三、存储结构设计

```rust
/// 八字命盘完整信息
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct BaziChart<T: Config> {
    /// 所有者
    pub owner: T::AccountId,
    
    /// 姓名/备注（可选，UTF-8，最多 64 字节）
    pub name: Option<BoundedVec<u8, ConstU32<64>>>,
    
    /// 性别
    pub gender: Gender,
    
    /// 出生时间（公历，已修正）
    pub birth_time: BirthTime,
    
    /// 出生地经度（可选，用于记录）
    pub longitude: Option<i32>,
    
    /// 子时模式
    pub zishi_mode: ZiShiMode,
    
    /// 四柱
    pub sizhu: SiZhu<T>,
    
    /// 大运
    pub dayun: DaYunInfo<T>,
    
    /// 五行强度
    pub wuxing_strength: WuXingStrength,
    
    /// 喜用神
    pub xiyong_shen: Option<WuXing>,
    
    /// 创建时间戳（区块号）
    pub created_at: u64,
}
```

**存储大小估算**：
- AccountId: 32 bytes
- name: 1 + 64 = 65 bytes（Option + BoundedVec）
- gender: 1 byte
- birth_time: 7 bytes
- longitude: 5 bytes（Option + i32）
- zishi_mode: 1 byte
- sizhu: ~200 bytes
- dayun: ~500 bytes
- wuxing_strength: ~20 bytes
- xiyong_shen: 2 bytes
- created_at: 8 bytes

**总计**：约 841 bytes（可接受）

---

## 四、权重设计

```rust
impl<T: Config> WeightInfo for () {
    fn create_bazi_chart() -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    
    fn create_bazi_chart_with_options() -> Weight {
        // 基础权重
        let mut weight = Self::create_bazi_chart();
        
        // 真太阳时计算额外开销（浮点运算）
        weight = weight.saturating_add(Weight::from_parts(2_000_000, 0));
        
        weight
    }
}
```

---

## 五、错误类型扩展

```rust
#[pallet::error]
pub enum Error<T> {
    // ... 现有错误 ...
    
    /// 姓名无效（非 UTF-8 或超长）
    InvalidName,
    /// 农历日期无效
    InvalidLunarDate,
    /// 经纬度无效
    InvalidGeoLocation,
    /// 四柱索引无效
    InvalidPillarIndex,
}
```

---

## 六、前端交互设计

### 简单模式（推荐给普通用户）

```typescript
// 只需填写基本信息
await api.tx.baziChart.createBaziChart(
  "张三",           // 姓名（可选）
  "Female",         // 性别
  {
    Solar: {        // 公历
      year: 1990,
      month: 5,
      day: 15,
      hour: 14,
      minute: 30,
    }
  }
).signAndSend(account);
```

### 高级模式（专业用户）

```typescript
await api.tx.baziChart.createBaziChartWithOptions(
  "李四",
  "Male",
  {
    Lunar: {        // 农历
      year: 1990,
      month: 4,
      day: 21,
      isLeap: false,
      hour: 14,
      minute: 30,
    }
  },
  {
    location: {     // 出生地（启用真太阳时）
      longitude: 1164074,  // 东经 116.4074°
      latitude: 399042,    // 北纬 39.9042°
    },
    zishiMode: "Traditional",  // 传统派
  }
).signAndSend(account);
```

---

## 七、总结与建议

### 各参数评分

| 参数 | 必要性 | 可行性 | 合理性 | 建议 |
|------|--------|--------|--------|------|
| **姓名** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | 可选，支持昵称 |
| **性别** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 必需，无需修改 |
| **出生时间** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 枚举统一，强烈推荐 |
| **经纬度** | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | 可选，仅存经度 |
| **真太阳时** | ⭐⭐ | ⭐⭐⭐ | ⭐⭐ | 不建议独立参数 |
| **子时模式** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | 保留，提供默认值 |

### 最终推荐接口

```rust
/// 基础接口（80% 用户）
pub fn create_bazi_chart(
    origin: OriginFor<T>,
    name: Option<BoundedVec<u8, ConstU32<64>>>,
    gender: Gender,
    birth_input: BirthTimeInput,
) -> DispatchResult

/// 高级接口（20% 专业用户）
pub fn create_bazi_chart_with_options(
    origin: OriginFor<T>,
    name: Option<BoundedVec<u8, ConstU32<64>>>,
    gender: Gender,
    birth_input: BirthTimeInput,
    options: Option<BaziOptions>,  // 包含经度和子时模式
) -> DispatchResult
```

### 关键决策

1. **姓名**：可选，默认 None，前端可生成 "命盘 #ID"
2. **出生时间**：使用 `BirthTimeInput` 枚举统一三种输入
3. **经纬度**：可选，仅存储经度（4 bytes），自动启用真太阳时
4. **真太阳时**：不作为独立参数，有经度即启用
5. **子时模式**：保留，默认现代派

### 实现优先级

1. **P0（必须）**：`BirthTimeInput` 枚举 + 基础接口
2. **P1（重要）**：姓名支持 + 农历转换
3. **P2（可选）**：真太阳时修正 + 高级接口
4. **P3（未来）**：加密姓名 + 隐私保护

---

## 八、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| **隐私泄露** | 高 | 姓名可选 + 加密选项 |
| **存储膨胀** | 中 | 限制姓名长度 + 可选字段 |
| **计算复杂** | 低 | 真太阳时算法简单 |
| **用户困惑** | 中 | 分层接口 + 合理默认值 |
| **兼容性** | 低 | 保留旧接口 + 渐进迁移 |

---

**结论**：提议的接口设计在技术上完全可行，在业务上基本合理。建议采用分层设计，为普通用户提供简化接口，为专业用户提供完整选项。
