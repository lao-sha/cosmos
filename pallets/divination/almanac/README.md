# 黄历模块 (pallet-almanac)

## 概述

黄历模块是 Cosmos 占卜系统的核心基础设施模块，提供中国传统黄历数据的链上存储与查询服务。模块通过 Off-chain Worker (OCW) 自动从外部 API 获取黄历数据，同时支持授权账户手动设置数据。

本模块还内置了完整的农历计算引擎（1901-2100年，200年数据），为其他占卜模块提供公历转农历、干支计算、节气查询等核心功能。

## 核心功能

- **黄历数据管理**：支持单日设置和批量设置黄历数据
- **OCW 自动更新**：通过 Off-chain Worker 定期从阿里云黄历 API 获取数据
- **农历计算引擎**：内置 200 年农历数据，支持公历农历互转
- **干支计算**：年/月/日/时四柱干支计算
- **节气查询**：二十四节气信息
- **权限控制**：数据提交需要授权

## 数据结构

### AlmanacInfo - 黄历数据

```rust
pub struct AlmanacInfo {
    // 农历信息
    pub lunar_year: u16,      // 农历年份
    pub lunar_month: u8,      // 农历月份 (1-12, 闰月用 13-24)
    pub lunar_day: u8,        // 农历日期 (1-30)
    
    // 年干支
    pub year_tiangan: u8,     // 年天干 (0-9: 甲乙丙丁戊己庚辛壬癸)
    pub year_dizhi: u8,       // 年地支 (0-11: 子丑寅卯辰巳午未申酉戌亥)
    
    // 月干支
    pub month_tiangan: u8,
    pub month_dizhi: u8,
    
    // 日干支
    pub day_tiangan: u8,
    pub day_dizhi: u8,
    
    // 时干支 (子时)
    pub hour_tiangan: u8,
    pub hour_dizhi: u8,
    
    // 其他属性
    pub zodiac: u8,           // 生肖 (0-11)
    pub conflict_zodiac: u8,  // 冲煞生肖
    pub sha_direction: u8,    // 煞方 (0:东 1:南 2:西 3:北)
    pub wuxing: u8,           // 五行 (0-4: 金木水火土)
    pub jianchu: u8,          // 建除十二神 (0-11)
    pub constellation: u8,    // 二十八宿 (0-27)
    
    // 宜忌信息 (bit 标记)
    pub suitable: u64,        // 宜事项
    pub avoid: u64,           // 忌事项
    
    // 节气和节日
    pub solar_term: u8,       // 节气 (0:无, 1-24:立春至大寒)
    pub festivals: u32,       // 节日标记
    pub fortune_level: u8,    // 吉凶等级 (0:大吉 - 4:大凶)
    
    // 元数据
    pub updated_at: u64,      // 更新时间戳
    pub source: u8,           // 数据来源 (0:OCW 1:手动 2:算法)
}
```

### OcwConfig - OCW 配置

```rust
pub struct OcwConfig {
    pub enabled: bool,        // 是否启用
    pub update_hour: u8,      // 每日更新时间 (UTC)
    pub batch_days: u8,       // 批量获取天数 (1-90)
    pub last_update: u64,     // 上次更新时间
    pub failure_count: u8,    // 连续失败次数
    pub max_retries: u8,      // 最大重试次数
}
```

### 宜忌事项枚举

支持 32 种常见宜忌事项，使用 bit 标记存储：

| 编号 | 事项 | 编号 | 事项 |
|------|------|------|------|
| 0 | 嫁娶 | 16 | 安床 |
| 1 | 纳采 | 17 | 入宅 |
| 2 | 祭祀 | 18 | 安门 |
| 3 | 祈福 | 19 | 求嗣 |
| 4 | 出行 | 20 | 解除 |
| 5 | 动土 | 21 | 求医 |
| 6 | 破土 | 22 | 词讼 |
| 7 | 安葬 | 23 | 沐浴 |
| 8 | 开市 | 24 | 理发 |
| 9 | 交易 | 25 | 扫舍 |
| 10 | 立券 | 26 | 会友 |
| 11 | 移徙 | 27 | 上梁 |
| 12 | 修造 | 28 | 竖柱 |
| 13 | 栽种 | 29 | 纳畜 |
| 14 | 纳财 | 30 | 伐木 |
| 15 | 开光 | 31 | 作灶 |

## 存储项

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `AlmanacData` | `Map<DateKey, AlmanacInfo>` | 黄历数据，键为 (年, 月, 日) |
| `OcwConfigStorage` | `Value<OcwConfig>` | OCW 配置 |
| `DataAuthorities` | `Map<AccountId, bool>` | 授权账户列表 |
| `DataStats` | `Map<u16, (u32, u32, u32)>` | 年度统计 (总数, OCW数, 手动数) |
| `LastUpdatedDate` | `Value<DateKey>` | 最近更新日期 |

## Extrinsics

### 用户调用

#### `set_almanac`
设置单日黄历数据。

```rust
fn set_almanac(
    origin: OriginFor<T>,
    year: u16,
    month: u8,
    day: u8,
    info: AlmanacInfo,
) -> DispatchResult
```

- **权限**：需要 DataAuthority 权限
- **参数**：公历日期和黄历数据

#### `batch_set_almanac`
批量设置黄历数据。

```rust
fn batch_set_almanac(
    origin: OriginFor<T>,
    data: Vec<(DateKey, AlmanacInfo)>,
) -> DispatchResult
```

- **权限**：需要 DataAuthority 权限
- **限制**：单次最多 `MaxBatchSize` 条

### 治理调用 (Root)

#### `configure_ocw`
配置 OCW 参数。

```rust
fn configure_ocw(
    origin: OriginFor<T>,
    config: OcwConfig,
) -> DispatchResult
```

#### `add_authority`
添加数据提交权限。

```rust
fn add_authority(
    origin: OriginFor<T>,
    account: T::AccountId,
) -> DispatchResult
```

#### `remove_authority`
移除数据提交权限。

```rust
fn remove_authority(
    origin: OriginFor<T>,
    account: T::AccountId,
) -> DispatchResult
```

#### `remove_almanac`
删除特定日期的黄历数据。

```rust
fn remove_almanac(
    origin: OriginFor<T>,
    year: u16,
    month: u8,
    day: u8,
) -> DispatchResult
```

## 事件

| 事件 | 说明 |
|------|------|
| `AlmanacUpdated` | 黄历数据已更新 |
| `AlmanacBatchUpdated` | 批量更新完成 |
| `AlmanacRemoved` | 黄历数据已删除 |
| `OcwConfigured` | OCW 配置已更新 |
| `AuthorityAdded` | 添加了数据提交权限 |
| `AuthorityRemoved` | 移除了数据提交权限 |
| `OcwFetchSuccess` | OCW 获取数据成功 |
| `OcwFetchFailed` | OCW 获取数据失败 |

## 错误

| 错误 | 说明 |
|------|------|
| `NoPermission` | 无操作权限 |
| `InvalidDate` | 无效的日期 |
| `InvalidConfig` | 无效的配置 |
| `BatchTooLarge` | 批量操作数量超限 |
| `DataNotFound` | 数据不存在 |
| `DataAlreadyExists` | 数据已存在 |
| `OcwNotEnabled` | OCW 未启用 |
| `AppCodeNotConfigured` | AppCode 未配置 |
| `ApiCallFailed` | API 调用失败 |
| `JsonParseFailed` | JSON 解析失败 |
| `DataValidationFailed` | 数据验证失败 |

## 配置参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `MaxBatchSize` | `u32` | 最大批量设置数量 |
| `MaxHistoryYears` | `u32` | 最大历史数据年限 (默认 3 年) |

## 农历计算模块

本模块内置完整的农历计算引擎，供其他占卜模块调用。

### 导出类型

```rust
// 核心类型
pub use lunar::{LunarDate, GanZhi, FourPillars};

// 梅花易数专用
pub use lunar::{MeihuaLunarDate, LunarConvertError};
```

### 导出函数

```rust
// 公历农历转换
solar_to_lunar(year, month, day) -> Option<LunarDate>
lunar_to_solar(year, month, day, is_leap) -> Option<(u16, u8, u8)>

// 干支计算
year_ganzhi(year) -> GanZhi
month_ganzhi(year, month) -> GanZhi
day_ganzhi(year, month, day) -> GanZhi
hour_ganzhi(day_gan, hour) -> GanZhi
four_pillars(year, month, day, hour) -> FourPillars

// 生肖
get_zodiac(year) -> u8
zodiac_name(zodiac) -> &'static str

// 节气
get_solar_term(year, month, day) -> Option<u8>
solar_term_name(term) -> &'static str

// 时间戳转换 (梅花易数)
timestamp_to_meihua_lunar(timestamp) -> MeihuaLunarDate
hour_to_dizhi_num(hour) -> u8
year_to_dizhi_num(year) -> u8
```

### 常量

```rust
pub const TIANGAN: [&str; 10];      // 天干
pub const DIZHI: [&str; 12];        // 地支
pub const SHENGXIAO: [&str; 12];    // 生肖
pub const LUNAR_MONTHS: [&str; 12]; // 农历月份
pub const LUNAR_DAYS: [&str; 30];   // 农历日期
pub const SOLAR_TERMS: [&str; 24];  // 二十四节气

pub const LUNAR_START_YEAR: u16 = 1901;
pub const LUNAR_END_YEAR: u16 = 2100;
```

## 使用示例

### 查询黄历数据

```rust
// 查询 2024年1月1日 的黄历
let info = Almanac::almanac_data((2024, 1, 1));
if let Some(almanac) = info {
    // 获取干支
    let (year_gan, year_zhi) = almanac.year_ganzhi();
    
    // 检查是否宜嫁娶
    if almanac.is_suitable(SuitableItem::Marriage) {
        // 今日宜嫁娶
    }
    
    // 获取节气
    if almanac.solar_term > 0 {
        let term_name = almanac.solar_term_name();
    }
}
```

### 使用农历计算

```rust
use pallet_almanac::{solar_to_lunar, four_pillars, get_zodiac, zodiac_name};

// 公历转农历
if let Some(lunar) = solar_to_lunar(2024, 2, 10) {
    // lunar.year = 2024, lunar.month = 1, lunar.day = 1 (春节)
}

// 计算八字
let pillars = four_pillars(2024, 2, 10, 12);
// pillars.year = 甲辰
// pillars.month = 丙寅
// pillars.day = ...
// pillars.hour = ...

// 获取生肖
let zodiac = get_zodiac(2024);
let name = zodiac_name(zodiac); // "龙"
```

### 批量设置黄历

```rust
let data = vec![
    ((2024, 1, 1), almanac_info_1),
    ((2024, 1, 2), almanac_info_2),
    ((2024, 1, 3), almanac_info_3),
];

Almanac::batch_set_almanac(origin, data)?;
```

## 安全设计

1. **AppCode 安全**：阿里云 API 的 AppCode 通过环境变量配置，不在链上存储
2. **权限控制**：数据提交需要 Root 授权的账户
3. **OCW 隔离**：Off-chain Worker 使用独立账户签名
4. **数据验证**：所有日期输入都经过有效性验证

## 存储优化

- 使用紧凑的数据结构，每日黄历约 50 bytes
- 宜忌事项使用 bit 标记，64 位支持 64 种事项
- 节日使用 32 位 bit 标记
- 支持批量操作减少交易次数

## 依赖模块

- `frame-system`
- `pallet-timestamp`

## 相关文档

- [DESIGN.md](./DESIGN.md) - 详细设计文档
- [APPCODE_SECURITY.md](./APPCODE_SECURITY.md) - AppCode 安全配置
- [NODE_APPCODE_USAGE.md](./NODE_APPCODE_USAGE.md) - 节点 AppCode 使用说明
- [NODE_IMPLEMENTATION_SUMMARY.md](./NODE_IMPLEMENTATION_SUMMARY.md) - 节点实现总结

## 版本历史

- v0.1.0 - 初始版本，支持黄历数据存储和 OCW 自动更新
