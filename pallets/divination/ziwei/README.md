# Pallet Ziwei (紫微斗数 Pallet)

完整的紫微斗数排盘区块链模块，基于 Substrate FRAME 框架开发。

[![License: MIT-0](https://img.shields.io/badge/License-MIT--0-blue.svg)](https://opensource.org/licenses/MIT-0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Substrate](https://img.shields.io/badge/substrate-polkadot--sdk-blue)](https://github.com/paritytech/polkadot-sdk)

## 目录

- [项目概述](#项目概述)
- [核心功能](#核心功能)
- [技术架构](#技术架构)
- [API 接口](#api-接口)
- [配置参数](#配置参数)
- [隐私保护](#隐私保护)
- [规划功能](#规划功能)
- [许可证](#许可证)

---

## 项目概述

本 Pallet 实现了传统紫微斗数排盘的完整功能，包括命盘排布、十四主星安星、辅星系统、四化飞星、大运推算等，支持链上存储和多种隐私模式。

### ✨ 核心特点

- **完整排盘**: 十四主星 + 六吉六煞 + 四化飞星
- **多种起盘**: 农历时间、公历时间、手动指定、随机起盘
- **隐私保护**: 支持 Public/Partial/Private 三种隐私模式
- **AI 解读**: 集成通用占卜 AI 解读系统
- **免费查询**: Runtime API 实时计算，无 Gas 费用

### 📊 功能完成度

- **命宫定位**: ✅ 100%
- **五行局计算**: ✅ 100%
- **紫微星系安星**: ✅ 100% (6星)
- **天府星系安星**: ✅ 100% (8星)
- **六吉星**: ✅ 100%
- **六煞星**: ✅ 100%
- **四化飞星**: ✅ 100%
- **大运计算**: ✅ 100%
- **隐私加密**: ✅ 100%

---

## 核心功能

### 1️⃣ 命盘排布

- **命宫定位**: 根据农历月份和出生时辰计算
- **身宫定位**: 根据农历月份和出生时辰计算
- **五行局**: 根据年干和命宫位置确定（水二局、木三局、金四局、土五局、火六局）

### 2️⃣ 十四主星

**紫微星系 (6星)**:
- 紫微、天机、太阳、武曲、天同、廉贞

**天府星系 (8星)**:
- 天府、太阴、贪狼、巨门、天相、天梁、七杀、破军

### 3️⃣ 辅星系统

**六吉星**: 文昌、文曲、左辅、右弼、天魁、天钺

**六煞星**: 擎羊、陀罗、火星、铃星、地空、地劫

**其他**: 禄存、天马

### 4️⃣ 四化飞星

根据年干确定四化星（化禄、化权、化科、化忌）落入的主星。

### 5️⃣ 大运推算

- **起运年龄**: 根据五行局数计算
- **顺逆排列**: 根据年干阴阳和性别确定

---

## 技术架构

### 📦 模块结构

```
pallet-ziwei/
├── src/
│   ├── lib.rs              # Pallet 主模块
│   ├── types.rs            # 数据类型定义
│   ├── algorithm/          # 排盘算法
│   │   ├── mod.rs
│   │   ├── ming_gong.rs    # 命宫计算
│   │   ├── wu_xing_ju.rs   # 五行局计算
│   │   ├── ziwei_series.rs # 紫微星系安星
│   │   ├── tianfu_series.rs# 天府星系安星
│   │   ├── liu_ji.rs       # 六吉星
│   │   ├── liu_sha.rs      # 六煞星
│   │   └── si_hua.rs       # 四化飞星
│   ├── interpretation.rs   # 解盘算法
│   ├── runtime_api.rs      # Runtime API
│   ├── mock.rs             # 测试环境
│   └── tests.rs            # 单元测试
├── Cargo.toml
└── README.md
```

### 🔑 关键数据类型

```rust
/// 紫微命盘
pub struct ZiweiChart<AccountId, BlockNumber, Moment, MaxCidLen> {
    pub id: u64,
    pub creator: AccountId,
    pub created_at: BlockNumber,
    pub timestamp: Moment,
    pub privacy_mode: PrivacyMode,
    pub lunar_year: Option<u16>,
    pub lunar_month: Option<u8>,
    pub lunar_day: Option<u8>,
    pub birth_hour: Option<DiZhi>,
    pub gender: Option<Gender>,
    pub wu_xing_ju: Option<WuXing>,
    pub ju_shu: Option<u8>,
    pub ming_gong_pos: Option<u8>,
    pub shen_gong_pos: Option<u8>,
    pub ziwei_pos: Option<u8>,
    pub tianfu_pos: Option<u8>,
    pub palaces: Option<[Palace; 12]>,
    pub si_hua_stars: Option<SiHuaStars>,
    pub qi_yun_age: Option<u8>,
    pub da_yun_shun: Option<bool>,
    pub ai_interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,
}

/// 十二宫
pub struct Palace {
    pub zhu_xing: [Option<ZhuXing>; 4],  // 主星（最多4颗）
    pub liu_ji: [bool; 6],                // 六吉星
    pub liu_sha: [bool; 6],               // 六煞星
    pub lu_cun: bool,                     // 禄存
    pub tian_ma: bool,                    // 天马
}
```

---

## API 接口

### 📞 可调用函数 (Extrinsics)

| 函数 | 说明 | 参数 |
|-----|------|------|
| `divine_by_time` | 农历时间起盘 | lunar_year, lunar_month, lunar_day, birth_hour, gender, is_leap_month |
| `divine_by_solar_time` | 公历时间起盘 | solar_year, solar_month, solar_day, birth_hour, gender |
| `divine_manual` | 手动指定起盘 | lunar_year, lunar_month, lunar_day, birth_hour, gender, year_gan, year_zhi |
| `divine_random` | 随机起盘 | - |
| `divine_by_time_encrypted` | 加密时间起盘 | encryption_level, lunar_year, ... , encrypted_data, data_hash, owner_key_backup |
| `set_chart_visibility` | 设置可见性 | chart_id, is_public |

### 🔍 Runtime API（免费查询）

| 函数 | 说明 | 返回 |
|-----|------|------|
| `get_chart` | 获取命盘 | `Option<ZiweiChart>` |
| `get_user_charts` | 获取用户命盘列表 | `Vec<u64>` |
| `get_public_charts` | 获取公开命盘列表 | `Vec<u64>` |

---

## 配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config + pallet_timestamp::Config {
    type Currency: Currency<Self::AccountId>;
    type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
    
    #[pallet::constant]
    type MaxUserCharts: Get<u32>;        // 每用户最大命盘数
    
    #[pallet::constant]
    type MaxPublicCharts: Get<u32>;      // 公开列表最大长度
    
    #[pallet::constant]
    type DailyFreeCharts: Get<u32>;      // 每日免费次数
    
    #[pallet::constant]
    type MaxDailyCharts: Get<u32>;       // 每日最大次数
    
    #[pallet::constant]
    type AiInterpretationFee: Get<BalanceOf<Self>>; // AI解读费用
    
    #[pallet::constant]
    type MaxEncryptedLen: Get<u32>;      // 加密数据最大长度
}
```

---

## 隐私保护

### 三种隐私模式

| 模式 | 计算数据 | 敏感数据 | 适用场景 |
|-----|---------|---------|---------|
| **Public** | 明文 | 明文 | 公开分享 |
| **Partial** | 明文 | 加密 | 保护出生信息 |
| **Private** | 加密 | 加密 | 完全隐私 |

### 加密流程

1. 前端使用 AES-256-GCM 加密敏感数据
2. 密钥从钱包签名派生
3. 链上存储加密数据和密钥备份
4. 解盘基于四柱索引计算，无需解密

---

## 规划功能

### 🔮 数据删除功能（计划中）

**功能描述**: 允许用户删除自己的命盘记录，释放链上存储空间。

```rust
/// 删除命盘记录
#[pallet::call_index(8)]
pub fn delete_chart(
    origin: OriginFor<T>,
    chart_id: u64,
) -> DispatchResult;
```

**特点**:
- 仅所有者可删除
- 删除后数据不可恢复
- 退还100%存储押金

### 💰 存储押金机制（计划中）

**功能描述**: 创建命盘时锁定押金，删除时退还。

| 操作 | 押金变化 |
|-----|---------|
| 创建命盘 | 锁定 0.8 USDT |
| 删除命盘 | 退还 0.8 USDT (100%) |
| 归档命盘 | 退还 0.4 USDT (50%) |

**免费配额**:
- 每日免费: 3次
- 每月免费: 10次

### 📦 数据归档功能（计划中）

**功能描述**: 将不常用的命盘归档到 IPFS，释放链上空间。

```rust
/// 归档命盘到IPFS
#[pallet::call_index(9)]
pub fn archive_chart(
    origin: OriginFor<T>,
    chart_id: u64,
) -> DispatchResult;

/// 从IPFS解档命盘
#[pallet::call_index(10)]
pub fn unarchive_chart(
    origin: OriginFor<T>,
    chart_id: u64,
    ipfs_cid: BoundedVec<u8, ConstU32<64>>,
) -> DispatchResult;
```

**数据生命周期**:
```
Active (链上) → Archive (IPFS) → Delete (清除)
   100%押金      50%押金退还      100%押金退还
```

---

## 许可证

MIT-0 License

---

**创建日期**: 2025-12-28  
**当前版本**: v1.0.0  
**维护团队**: Cosmos 开发团队
