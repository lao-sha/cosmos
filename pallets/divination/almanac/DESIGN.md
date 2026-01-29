# pallet-almanac 开发方案设计

## 一、方案概述

### 1.1 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                     Cosmos Blockchain                      │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              pallet-almanac (Runtime)                   │ │
│  │                                                          │ │
│  │  ┌──────────────┐    ┌──────────────┐                 │ │
│  │  │ Storage      │    │ Extrinsics   │                 │ │
│  │  │ - AlmanacData│    │ - set_almanac│                 │ │
│  │  │ - Authorities│    │ - submit_data│                 │ │
│  │  └──────────────┘    └──────────────┘                 │ │
│  │                                                          │ │
│  │  ┌──────────────────────────────────┐                 │ │
│  │  │  Off-chain Worker (OCW)          │                 │ │
│  │  │  1. 定期触发(每日 00:00)           │                 │ │
│  │  │  2. 调用阿里云黄历 API             │                 │ │
│  │  │  3. 数据解析和验证                  │                 │ │
│  │  │  4. 签名交易提交到链上              │                 │ │
│  │  └──────────────────────────────────┘                 │ │
│  └────────────────────────────────────────────────────────┘ │
│                           ▲                                   │
│                           │ HTTP Request                      │
│                           │                                   │
│  ┌────────────────────────┼────────────────────────────────┐ │
│  │                        │                                 │ │
│  │              阿里云黄历 API                              │ │
│  │   https://jmhlysjjr.market.alicloudapi.com              │ │
│  └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                           ▲
                           │ RPC Query
                           │
┌─────────────────────────────────────────────────────────────┐
│                  cosmos-dapp (Frontend)                    │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  AlmanacService                                       │  │
│  │  - getAlmanacByDate(year, month, day)               │  │
│  │  - getMonthAlmanac(year, month)                     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  AlmanacPage Component                                │  │
│  │  - 日历视图                                            │  │
│  │  - 宜忌展示                                            │  │
│  │  - 五行、生肖、节气显示                                │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、数据结构设计

### 2.1 核心数据类型

```rust
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

/// 黄历数据结构
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AlmanacInfo {
    /// 农历年份 (如: 2024)
    pub lunar_year: u32,

    /// 农历月份 (1-12, 闰月标识: 13)
    pub lunar_month: u8,

    /// 农历日期 (1-30)
    pub lunar_day: u8,

    /// 天干 (0-9: 甲乙丙丁戊己庚辛壬癸)
    pub tiangan: u8,

    /// 地支 (0-11: 子丑寅卯辰巳午未申酉戌亥)
    pub dizhi: u8,

    /// 生肖 (0-11: 鼠牛虎兔龙蛇马羊猴鸡狗猪)
    pub zodiac: u8,

    /// 冲煞方向 (0-11: 对应地支)
    pub conflict: u8,

    /// 五行 (0-4: 金木水火土)
    pub wuxing: u8,

    /// 二十八宿 (0-27)
    pub constellation: u8,

    /// 建除十二神 (0-11: 建除满平定执破危成收开闭)
    pub jianchu: u8,

    /// 宜 (使用标记位表示)
    /// Bit 0: 嫁娶, Bit 1: 纳采, Bit 2: 祭祀, Bit 3: 祈福
    /// Bit 4: 出行, Bit 5: 动土, Bit 6: 破土, Bit 7: 安葬
    /// Bit 8: 开市, Bit 9: 交易, Bit 10: 立券, Bit 11: 移徙
    /// ... 最多支持 64 种宜事项
    pub suitable: u64,

    /// 忌 (使用标记位表示, 同上)
    pub avoid: u64,

    /// 节气 (0: 无节气, 1-24: 立春至大寒)
    pub solar_term: u8,

    /// 节日标记
    /// Bit 0: 元旦, Bit 1: 春节, Bit 2: 清明, Bit 3: 端午
    /// Bit 4: 中秋, Bit 5: 国庆, Bit 6: 元宵, Bit 7: 重阳
    /// ... 支持 32 种节日
    pub festivals: u32,

    /// 吉凶等级 (0: 吉, 1: 较吉, 2: 平, 3: 较凶, 4: 凶)
    pub fortune_level: u8,

    /// 数据更新时间戳
    pub updated_at: u64,

    /// 数据来源 (0: OCW, 1: 手动设置)
    pub source: u8,
}

/// 宜忌事项枚举 (用于前端展示)
pub enum SuitableItem {
    Marriage = 0,      // 嫁娶
    Betrothal = 1,     // 纳采
    Sacrifice = 2,     // 祭祀
    Prayer = 3,        // 祈福
    Travel = 4,        // 出行
    Groundbreaking = 5, // 动土
    Excavation = 6,    // 破土
    Burial = 7,        // 安葬
    OpenBusiness = 8,  // 开市
    Trading = 9,       // 交易
    Contract = 10,     // 立券
    Moving = 11,       // 移徙
    Renovation = 12,   // 修造
    Planting = 13,     // 栽种
    HarvestCrops = 14, // 收获
    Cleaning = 15,     // 扫舍
}

/// OCW 配置
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct OcwConfig {
    /// 是否启用 OCW
    pub enabled: bool,

    /// 更新时间 (UTC 小时, 0-23)
    pub update_hour: u8,

    /// API AppCode 哈希 (避免明文存储)
    pub appcode_hash: [u8; 32],

    /// 批量更新天数 (建议 7-30 天)
    pub batch_days: u8,

    /// 上次更新时间戳
    pub last_update: u64,
}
```

### 2.2 存储设计

```rust
#[pallet::storage]
#[pallet::getter(fn almanac_data)]
/// 黄历数据存储: (公历年, 月, 日) => AlmanacInfo
pub type AlmanacData<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    (u32, u8, u8), // (year, month, day)
    AlmanacInfo,
    OptionQuery,
>;

#[pallet::storage]
#[pallet::getter(fn ocw_config)]
/// OCW 配置
pub type OcwConfigStorage<T: Config> = StorageValue<_, OcwConfig, ValueQuery>;

#[pallet::storage]
#[pallet::getter(fn data_authorities)]
/// 有权限提交数据的账户列表 (Sudo + OCW)
pub type DataAuthorities<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    bool,
    ValueQuery,
>;

#[pallet::storage]
#[pallet::getter(fn ocw_account)]
/// OCW 专用账户
pub type OcwAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

#[pallet::storage]
#[pallet::getter(fn data_stats)]
/// 数据统计: (年份) => (总天数, OCW 更新数, 手动更新数)
pub type DataStats<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u32, // year
    (u32, u32, u32), // (total_days, ocw_count, manual_count)
    ValueQuery,
>;
```

---

## 三、Off-chain Worker 实现方案

### 3.1 触发机制

```rust
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn offchain_worker(block_number: T::BlockNumber) {
        // 检查是否需要触发 OCW
        if Self::should_trigger_ocw(block_number) {
            log::info!("🗓️ Almanac OCW triggered at block {:?}", block_number);

            // 执行主逻辑
            if let Err(e) = Self::fetch_and_submit_almanac() {
                log::error!("❌ Almanac OCW error: {:?}", e);
            }
        }
    }
}

impl<T: Config> Pallet<T> {
    /// 判断是否需要触发 OCW
    fn should_trigger_ocw(block_number: T::BlockNumber) -> bool {
        let config = Self::ocw_config();

        // 检查 OCW 是否启用
        if !config.enabled {
            return false;
        }

        // 每 N 个区块检查一次 (避免频繁触发)
        // 假设 6 秒出块, 600 个区块约 1 小时
        if block_number % 600u32.into() != 0u32.into() {
            return false;
        }

        // 检查当前 UTC 时间是否为配置的更新时间
        let now = sp_io::offchain::timestamp();
        let hour = (now.unix_millis() / 1000 / 3600) % 24;

        if hour as u8 != config.update_hour {
            return false;
        }

        // 检查今天是否已更新 (避免重复)
        let today = now.unix_millis() / 1000 / 86400;
        let last_update_day = config.last_update / 86400;

        today > last_update_day
    }
}
```

### 3.2 API 调用实现

```rust
use sp_runtime::offchain::{http, Duration};

impl<T: Config> Pallet<T> {
    /// 从阿里云 API 获取黄历数据
    fn fetch_almanac_from_api(
        year: u32,
        month: u8,
        day: u8,
    ) -> Result<AlmanacInfo, &'static str> {
        // 1. 构造 API 请求
        let url = "https://jmhlysjjr.market.alicloudapi.com/almanac/day";
        let body = format!("year={}&month={}&day={}", year, month, day);

        // 2. 获取 AppCode (从链上配置读取)
        let config = Self::ocw_config();
        let appcode = Self::get_appcode(); // 从环境变量或配置读取

        // 3. 构造请求
        let request = http::Request::post(url, vec![body.as_bytes()])
            .add_header("Authorization", &format!("APPCODE {}", appcode))
            .add_header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .deadline(sp_io::offchain::timestamp().add(Duration::from_millis(10000)));

        // 4. 发送请求
        let pending = request.send().map_err(|_| "Failed to send request")?;
        let response = pending
            .try_wait(sp_io::offchain::timestamp().add(Duration::from_millis(10000)))
            .map_err(|_| "Request timeout")?
            .map_err(|_| "Request failed")?;

        // 5. 检查响应状态
        if response.code != 200 {
            log::error!("❌ API returned status: {}", response.code);
            return Err("API request failed");
        }

        // 6. 解析 JSON 响应
        let body = response.body().collect::<Vec<u8>>();
        let json_str = sp_std::str::from_utf8(&body).map_err(|_| "Invalid UTF-8")?;

        // 7. 解析为 AlmanacInfo
        Self::parse_api_response(json_str)
    }

    /// 解析 API 响应 JSON
    fn parse_api_response(json: &str) -> Result<AlmanacInfo, &'static str> {
        // 使用 lite-json 进行解析
        let json_val = lite_json::parse_json(json).map_err(|_| "JSON parse error")?;

        // 提取字段并构造 AlmanacInfo
        let almanac = AlmanacInfo {
            lunar_year: Self::extract_u32(&json_val, "lunar_year")?,
            lunar_month: Self::extract_u8(&json_val, "lunar_month")?,
            lunar_day: Self::extract_u8(&json_val, "lunar_day")?,
            tiangan: Self::extract_u8(&json_val, "tiangan")?,
            dizhi: Self::extract_u8(&json_val, "dizhi")?,
            zodiac: Self::extract_u8(&json_val, "zodiac")?,
            conflict: Self::extract_u8(&json_val, "conflict")?,
            wuxing: Self::extract_u8(&json_val, "wuxing")?,
            constellation: Self::extract_u8(&json_val, "constellation")?,
            jianchu: Self::extract_u8(&json_val, "jianchu")?,
            suitable: Self::extract_u64(&json_val, "suitable")?,
            avoid: Self::extract_u64(&json_val, "avoid")?,
            solar_term: Self::extract_u8(&json_val, "solar_term")?,
            festivals: Self::extract_u32(&json_val, "festivals")?,
            fortune_level: Self::extract_u8(&json_val, "fortune_level")?,
            updated_at: sp_io::offchain::timestamp().unix_millis(),
            source: 0, // OCW 来源
        };

        Ok(almanac)
    }

    /// 批量获取多天数据
    fn fetch_batch_almanac(start_date: (u32, u8, u8), days: u8) -> Vec<((u32, u8, u8), AlmanacInfo)> {
        let mut results = Vec::new();
        let (mut year, mut month, mut day) = start_date;

        for _ in 0..days {
            match Self::fetch_almanac_from_api(year, month, day) {
                Ok(info) => {
                    results.push(((year, month, day), info));
                    log::info!("✅ Fetched almanac for {}-{}-{}", year, month, day);
                }
                Err(e) => {
                    log::error!("❌ Failed to fetch {}-{}-{}: {}", year, month, day, e);
                }
            }

            // 计算下一天
            (year, month, day) = Self::next_day(year, month, day);

            // 延迟以避免 API 限流
            sp_io::offchain::sleep_until(
                sp_io::offchain::timestamp().add(Duration::from_millis(500))
            );
        }

        results
    }
}
```

### 3.3 签名交易提交

```rust
use frame_system::offchain::{
    AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer,
};

impl<T: Config> Pallet<T> {
    /// 提交签名交易到链上
    fn submit_almanac_signed(data: Vec<((u32, u8, u8), AlmanacInfo)>) -> Result<(), &'static str> {
        // 1. 获取 OCW 账户的签名者
        let signer = Signer::<T, T::AuthorityId>::any_account();

        if !signer.can_sign() {
            return Err("No signing keys available");
        }

        // 2. 批量提交数据
        for ((year, month, day), info) in data {
            let result = signer.send_signed_transaction(|_account| {
                Call::set_almanac {
                    year,
                    month,
                    day,
                    info: info.clone(),
                }
            });

            match result {
                Some((_, Ok(()))) => {
                    log::info!("✅ Submitted almanac {}-{}-{}", year, month, day);
                }
                _ => {
                    log::error!("❌ Failed to submit {}-{}-{}", year, month, day);
                }
            }
        }

        Ok(())
    }

    /// 主 OCW 逻辑
    fn fetch_and_submit_almanac() -> Result<(), &'static str> {
        let config = Self::ocw_config();

        // 1. 确定要更新的日期范围
        let today = Self::get_today_date();
        let batch_days = config.batch_days;

        log::info!("🔄 Starting almanac update: {} days from {:?}", batch_days, today);

        // 2. 批量获取数据
        let data = Self::fetch_batch_almanac(today, batch_days);

        if data.is_empty() {
            return Err("No data fetched");
        }

        log::info!("📊 Fetched {} days of almanac data", data.len());

        // 3. 提交到链上
        Self::submit_almanac_signed(data)?;

        // 4. 更新配置中的 last_update
        // 注意: 这里需要通过另一个交易来更新,或者在 set_almanac 中自动更新

        Ok(())
    }
}
```

---

## 四、Extrinsics 设计

### 4.1 核心交易方法

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 设置黄历数据 (需要权限)
    #[pallet::weight(T::WeightInfo::set_almanac())]
    #[pallet::call_index(0)]
    pub fn set_almanac(
        origin: OriginFor<T>,
        year: u32,
        month: u8,
        day: u8,
        info: AlmanacInfo,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        // 检查权限: 必须是 Authority 或 Sudo
        ensure!(
            Self::data_authorities(&who) || Self::is_sudo(&who),
            Error::<T>::NoPermission
        );

        // 验证日期有效性
        ensure!(month >= 1 && month <= 12, Error::<T>::InvalidDate);
        ensure!(day >= 1 && day <= 31, Error::<T>::InvalidDate);
        ensure!(year >= 2000 && year <= 2100, Error::<T>::InvalidDate);

        // 存储数据
        AlmanacData::<T>::insert((year, month, day), info.clone());

        // 更新统计
        Self::update_stats(year, info.source);

        // 发出事件
        Self::deposit_event(Event::AlmanacUpdated {
            date: (year, month, day),
            source: info.source,
            updater: who,
        });

        Ok(())
    }

    /// 批量设置黄历数据
    #[pallet::weight(T::WeightInfo::batch_set_almanac(data.len() as u32))]
    #[pallet::call_index(1)]
    pub fn batch_set_almanac(
        origin: OriginFor<T>,
        data: Vec<((u32, u8, u8), AlmanacInfo)>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        ensure!(
            Self::data_authorities(&who) || Self::is_sudo(&who),
            Error::<T>::NoPermission
        );

        // 限制批量大小
        ensure!(data.len() <= 100, Error::<T>::BatchTooLarge);

        for ((year, month, day), info) in data {
            AlmanacData::<T>::insert((year, month, day), info.clone());
            Self::update_stats(year, info.source);
        }

        Self::deposit_event(Event::AlmanacBatchUpdated {
            count: data.len() as u32,
            updater: who,
        });

        Ok(())
    }

    /// 配置 OCW 参数 (需要 Sudo)
    #[pallet::weight(T::WeightInfo::configure_ocw())]
    #[pallet::call_index(2)]
    pub fn configure_ocw(
        origin: OriginFor<T>,
        config: OcwConfig,
    ) -> DispatchResult {
        ensure_root(origin)?;

        // 验证配置
        ensure!(config.update_hour < 24, Error::<T>::InvalidConfig);
        ensure!(config.batch_days > 0 && config.batch_days <= 90, Error::<T>::InvalidConfig);

        OcwConfigStorage::<T>::put(config);

        Self::deposit_event(Event::OcwConfigured);

        Ok(())
    }

    /// 添加数据提交权限
    #[pallet::weight(T::WeightInfo::add_authority())]
    #[pallet::call_index(3)]
    pub fn add_authority(
        origin: OriginFor<T>,
        account: T::AccountId,
    ) -> DispatchResult {
        ensure_root(origin)?;

        DataAuthorities::<T>::insert(&account, true);

        Self::deposit_event(Event::AuthorityAdded { account });

        Ok(())
    }

    /// 移除数据提交权限
    #[pallet::weight(T::WeightInfo::remove_authority())]
    #[pallet::call_index(4)]
    pub fn remove_authority(
        origin: OriginFor<T>,
        account: T::AccountId,
    ) -> DispatchResult {
        ensure_root(origin)?;

        DataAuthorities::<T>::remove(&account);

        Self::deposit_event(Event::AuthorityRemoved { account });

        Ok(())
    }

    /// 删除特定日期的黄历数据
    #[pallet::weight(T::WeightInfo::remove_almanac())]
    #[pallet::call_index(5)]
    pub fn remove_almanac(
        origin: OriginFor<T>,
        year: u32,
        month: u8,
        day: u8,
    ) -> DispatchResult {
        ensure_root(origin)?;

        AlmanacData::<T>::remove((year, month, day));

        Self::deposit_event(Event::AlmanacRemoved {
            date: (year, month, day),
        });

        Ok(())
    }
}
```

---

## 五、RPC 接口设计

### 5.1 自定义 RPC

```rust
// pallets/divination/almanac/rpc/src/lib.rs

use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
};
use pallet_almanac_runtime_api::AlmanacApi as AlmanacRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[rpc(client, server)]
pub trait AlmanacApi<BlockHash> {
    /// 获取指定日期的黄历
    #[method(name = "almanac_getByDate")]
    fn get_by_date(&self, year: u32, month: u8, day: u8, at: Option<BlockHash>) -> RpcResult<Option<AlmanacInfo>>;

    /// 获取指定月份的所有黄历
    #[method(name = "almanac_getByMonth")]
    fn get_by_month(&self, year: u32, month: u8, at: Option<BlockHash>) -> RpcResult<Vec<(u8, AlmanacInfo)>>;

    /// 获取指定年份的所有节气
    #[method(name = "almanac_getSolarTerms")]
    fn get_solar_terms(&self, year: u32, at: Option<BlockHash>) -> RpcResult<Vec<((u8, u8), u8)>>;

    /// 获取数据统计
    #[method(name = "almanac_getStats")]
    fn get_stats(&self, year: u32, at: Option<BlockHash>) -> RpcResult<(u32, u32, u32)>;
}

pub struct AlmanacRpc<C, Block> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> AlmanacRpc<C, Block> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

#[async_trait]
impl<C, Block> AlmanacApiServer<<Block as BlockT>::Hash> for AlmanacRpc<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: AlmanacRuntimeApi<Block>,
{
    fn get_by_date(
        &self,
        year: u32,
        month: u8,
        day: u8,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Option<AlmanacInfo>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_almanac(at, year, month, day)
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))
    }

    fn get_by_month(
        &self,
        year: u32,
        month: u8,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<(u8, AlmanacInfo)>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_month_almanac(at, year, month)
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))
    }

    fn get_solar_terms(
        &self,
        year: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<((u8, u8), u8)>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_solar_terms(at, year)
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))
    }

    fn get_stats(
        &self,
        year: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<(u32, u32, u32)> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_stats(at, year)
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))
    }
}
```

### 5.2 Runtime API

```rust
// pallets/divination/almanac/runtime-api/src/lib.rs

sp_api::decl_runtime_apis! {
    pub trait AlmanacApi {
        fn get_almanac(year: u32, month: u8, day: u8) -> Option<AlmanacInfo>;
        fn get_month_almanac(year: u32, month: u8) -> Vec<(u8, AlmanacInfo)>;
        fn get_solar_terms(year: u32) -> Vec<((u8, u8), u8)>;
        fn get_stats(year: u32) -> (u32, u32, u32);
    }
}
```

---

## 六、前端集成方案

### 6.1 服务层实现

```typescript
// cosmos-dapp/src/services/almanacService.ts

import { ApiPromise } from '@polkadot/api';

export interface AlmanacInfo {
  lunarYear: number;
  lunarMonth: number;
  lunarDay: number;
  tiangan: number;
  dizhi: number;
  zodiac: number;
  conflict: number;
  wuxing: number;
  constellation: number;
  jianchu: number;
  suitable: bigint;
  avoid: bigint;
  solarTerm: number;
  festivals: number;
  fortuneLevel: number;
  updatedAt: number;
  source: number;
}

export class AlmanacService {
  private api: ApiPromise;

  constructor(api: ApiPromise) {
    this.api = api;
  }

  /**
   * 获取指定日期的黄历
   */
  async getAlmanacByDate(year: number, month: number, day: number): Promise<AlmanacInfo | null> {
    try {
      const result: any = await this.api.rpc['almanac'].getByDate(year, month, day);

      if (result.isNone) {
        return null;
      }

      const data = result.unwrap();
      return this.parseAlmanacInfo(data);
    } catch (error) {
      console.error('获取黄历失败:', error);
      return null;
    }
  }

  /**
   * 获取指定月份的黄历
   */
  async getMonthAlmanac(year: number, month: number): Promise<Map<number, AlmanacInfo>> {
    try {
      const result: any = await this.api.rpc['almanac'].getByMonth(year, month);

      const almanacMap = new Map<number, AlmanacInfo>();

      for (const [day, info] of result) {
        almanacMap.set(day.toNumber(), this.parseAlmanacInfo(info));
      }

      return almanacMap;
    } catch (error) {
      console.error('获取月份黄历失败:', error);
      return new Map();
    }
  }

  /**
   * 获取指定年份的节气
   */
  async getSolarTerms(year: number): Promise<Array<{ date: [number, number], term: number }>> {
    try {
      const result: any = await this.api.rpc['almanac'].getSolarTerms(year);

      return result.map((item: any) => ({
        date: [item[0][0].toNumber(), item[0][1].toNumber()],
        term: item[1].toNumber(),
      }));
    } catch (error) {
      console.error('获取节气失败:', error);
      return [];
    }
  }

  /**
   * 解析黄历数据
   */
  private parseAlmanacInfo(data: any): AlmanacInfo {
    return {
      lunarYear: data.lunarYear.toNumber(),
      lunarMonth: data.lunarMonth.toNumber(),
      lunarDay: data.lunarDay.toNumber(),
      tiangan: data.tiangan.toNumber(),
      dizhi: data.dizhi.toNumber(),
      zodiac: data.zodiac.toNumber(),
      conflict: data.conflict.toNumber(),
      wuxing: data.wuxing.toNumber(),
      constellation: data.constellation.toNumber(),
      jianchu: data.jianchu.toNumber(),
      suitable: data.suitable.toBigInt(),
      avoid: data.avoid.toBigInt(),
      solarTerm: data.solarTerm.toNumber(),
      festivals: data.festivals.toNumber(),
      fortuneLevel: data.fortuneLevel.toNumber(),
      updatedAt: data.updatedAt.toNumber(),
      source: data.source.toNumber(),
    };
  }

  /**
   * 获取宜事项列表
   */
  getSuitableItems(suitable: bigint): string[] {
    const items: string[] = [];
    const itemNames = [
      '嫁娶', '纳采', '祭祀', '祈福', '出行', '动土', '破土', '安葬',
      '开市', '交易', '立券', '移徙', '修造', '栽种', '收获', '扫舍',
    ];

    for (let i = 0; i < itemNames.length; i++) {
      if ((suitable & (1n << BigInt(i))) !== 0n) {
        items.push(itemNames[i]);
      }
    }

    return items;
  }

  /**
   * 获取忌事项列表
   */
  getAvoidItems(avoid: bigint): string[] {
    return this.getSuitableItems(avoid); // 使用相同的解析逻辑
  }

  /**
   * 获取生肖名称
   */
  getZodiacName(zodiac: number): string {
    const zodiacs = ['鼠', '牛', '虎', '兔', '龙', '蛇', '马', '羊', '猴', '鸡', '狗', '猪'];
    return zodiacs[zodiac] || '未知';
  }

  /**
   * 获取五行名称
   */
  getWuxingName(wuxing: number): string {
    const wuxings = ['金', '木', '水', '火', '土'];
    return wuxings[wuxing] || '未知';
  }

  /**
   * 获取天干地支
   */
  getGanzhi(tiangan: number, dizhi: number): string {
    const tianganNames = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
    const dizhiNames = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

    return `${tianganNames[tiangan]}${dizhiNames[dizhi]}`;
  }

  /**
   * 获取节气名称
   */
  getSolarTermName(term: number): string {
    const terms = [
      '', '立春', '雨水', '惊蛰', '春分', '清明', '谷雨',
      '立夏', '小满', '芒种', '夏至', '小暑', '大暑',
      '立秋', '处暑', '白露', '秋分', '寒露', '霜降',
      '立冬', '小雪', '大雪', '冬至', '小寒', '大寒',
    ];
    return terms[term] || '';
  }
}
```

### 6.2 React 组件示例

```typescript
// cosmos-dapp/src/features/almanac/AlmanacPage.tsx

import React, { useEffect, useState } from 'react';
import { Calendar, Card, Tag, Divider } from 'antd';
import { useApi } from '@/hooks/useApi';
import { AlmanacService, AlmanacInfo } from '@/services/almanacService';
import type { Dayjs } from 'dayjs';
import dayjs from 'dayjs';
import './AlmanacPage.css';

export const AlmanacPage: React.FC = () => {
  const { api } = useApi();
  const [selectedDate, setSelectedDate] = useState<Dayjs>(dayjs());
  const [almanacInfo, setAlmanacInfo] = useState<AlmanacInfo | null>(null);
  const [monthData, setMonthData] = useState<Map<number, AlmanacInfo>>(new Map());
  const [loading, setLoading] = useState(false);

  const almanacService = new AlmanacService(api!);

  // 加载月份数据
  useEffect(() => {
    if (!api) return;

    const loadMonthData = async () => {
      const year = selectedDate.year();
      const month = selectedDate.month() + 1;
      const data = await almanacService.getMonthAlmanac(year, month);
      setMonthData(data);
    };

    loadMonthData();
  }, [api, selectedDate.year(), selectedDate.month()]);

  // 加载当日详情
  useEffect(() => {
    if (!api) return;

    const loadDayData = async () => {
      setLoading(true);
      const year = selectedDate.year();
      const month = selectedDate.month() + 1;
      const day = selectedDate.date();

      const info = await almanacService.getAlmanacByDate(year, month, day);
      setAlmanacInfo(info);
      setLoading(false);
    };

    loadDayData();
  }, [api, selectedDate]);

  // 日历单元格渲染
  const cellRender = (date: Dayjs) => {
    const day = date.date();
    const info = monthData.get(day);

    if (!info) return null;

    const fortuneColors = ['#52c41a', '#73d13d', '#faad14', '#ff7a45', '#f5222d'];
    const fortuneColor = fortuneColors[info.fortuneLevel];

    return (
      <div className="almanac-cell">
        <div className="lunar-date">
          {info.lunarMonth}-{info.lunarDay}
        </div>
        <div className="fortune-dot" style={{ backgroundColor: fortuneColor }} />
        {info.solarTerm > 0 && (
          <Tag color="blue" className="solar-term-tag">
            {almanacService.getSolarTermName(info.solarTerm)}
          </Tag>
        )}
      </div>
    );
  };

  return (
    <div className="almanac-page">
      <Card title="黄历" className="almanac-card">
        <Calendar
          value={selectedDate}
          onChange={setSelectedDate}
          cellRender={cellRender}
        />
      </Card>

      {almanacInfo && (
        <Card className="almanac-detail-card" loading={loading}>
          <div className="almanac-header">
            <h2>{selectedDate.format('YYYY年MM月DD日')}</h2>
            <div className="lunar-info">
              农历 {almanacInfo.lunarYear}年{almanacInfo.lunarMonth}月{almanacInfo.lunarDay}日
            </div>
            <div className="ganzhi-info">
              {almanacService.getGanzhi(almanacInfo.tiangan, almanacInfo.dizhi)}年
              {almanacService.getZodiacName(almanacInfo.zodiac)}年
            </div>
          </div>

          <Divider />

          <div className="almanac-section">
            <h3>五行</h3>
            <Tag color="gold">{almanacService.getWuxingName(almanacInfo.wuxing)}</Tag>
          </div>

          <Divider />

          <div className="almanac-section">
            <h3>宜</h3>
            <div className="items-list">
              {almanacService.getSuitableItems(almanacInfo.suitable).map((item, idx) => (
                <Tag key={idx} color="green">{item}</Tag>
              ))}
            </div>
          </div>

          <Divider />

          <div className="almanac-section">
            <h3>忌</h3>
            <div className="items-list">
              {almanacService.getAvoidItems(almanacInfo.avoid).map((item, idx) => (
                <Tag key={idx} color="red">{item}</Tag>
              ))}
            </div>
          </div>

          <Divider />

          <div className="almanac-section">
            <h3>冲煞</h3>
            <div>冲 {almanacService.getZodiacName(almanacInfo.conflict)}</div>
          </div>

          {almanacInfo.solarTerm > 0 && (
            <>
              <Divider />
              <div className="almanac-section">
                <h3>节气</h3>
                <Tag color="blue">{almanacService.getSolarTermName(almanacInfo.solarTerm)}</Tag>
              </div>
            </>
          )}
        </Card>
      )}
    </div>
  );
};
```

---

## 七、开发步骤

### 第一阶段:基础 Pallet 开发 (3-5 天)

1. **创建 pallet 骨架**
   ```bash
   cd pallets/divination
   mkdir -p almanac/src
   ```

2. **实现核心数据结构**
   - `types.rs`: AlmanacInfo、OcwConfig
   - `lib.rs`: Config、Storage、Errors、Events

3. **实现基础 Extrinsics**
   - `set_almanac`: 手动设置黄历
   - `configure_ocw`: 配置 OCW
   - `add_authority`: 权限管理

4. **编写单元测试**
   - 测试数据存储和读取
   - 测试权限验证
   - 测试数据有效性验证

### 第二阶段:OCW 开发 (5-7 天)

5. **实现 OCW 基础框架**
   - `offchain.rs`: OCW 入口逻辑
   - 触发机制实现
   - 时间判断逻辑

6. **实现 API 调用**
   - HTTP 请求构造
   - JSON 解析 (使用 lite-json)
   - 错误处理和重试机制

7. **实现签名交易提交**
   - 配置 OCW 密钥
   - 签名交易构造
   - 批量提交优化

8. **测试 OCW 功能**
   - 本地节点测试
   - API 调用测试
   - 数据上链验证

### 第三阶段:RPC 和 Runtime API (2-3 天)

9. **实现 Runtime API**
   - 定义 `AlmanacApi` trait
   - 实现查询接口

10. **实现自定义 RPC**
    - RPC server 实现
    - 注册到 node

11. **测试 RPC 接口**
    - Postman 测试
    - Polkadot.js Apps 测试

### 第四阶段:前端集成 (3-5 天)

12. **实现前端服务层**
    - `almanacService.ts`
    - API 封装和类型定义

13. **实现黄历页面**
    - 日历组件
    - 详情展示
    - 样式优化

14. **测试前端功能**
    - 功能测试
    - 性能测试
    - 移动端适配

### 第五阶段:集成测试和优化 (2-3 天)

15. **端到端测试**
    - OCW 自动更新测试
    - 前端查询测试
    - 异常处理测试

16. **性能优化**
    - Storage 优化
    - RPC 缓存
    - 前端优化

17. **文档编写**
    - README.md
    - API 文档
    - 用户手册

---

## 八、安全考虑

### 8.1 权限控制

1. **OCW 账户隔离**: OCW 使用独立账户,避免权限滥用
2. **Authority 白名单**: 只有授权账户可以提交数据
3. **Sudo 权限**: 敏感操作需要 Root 权限

### 8.2 数据验证

1. **日期有效性**: 验证年月日范围
2. **数据完整性**: 验证必填字段
3. **数据一致性**: 检查农历与公历对应关系

### 8.3 API 安全

1. **AppCode 保护**: 不在链上明文存储,使用哈希或环境变量
2. **限流保护**: 限制 API 调用频率
3. **错误处理**: 避免敏感信息泄露

### 8.4 存储优化

1. **过期数据清理**: 定期清理历史数据 (如 3 年前)
2. **存储限制**: 限制单次批量写入大小
3. **压缩存储**: 使用 Bit 标记减少存储空间

---

## 九、配置示例

### 9.1 Runtime 配置

```rust
// runtime/src/lib.rs

impl pallet_almanac::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AuthorityId = pallet_almanac::crypto::AlmanacId;
    type WeightInfo = pallet_almanac::weights::SubstrateWeight<Runtime>;
    type MaxBatchSize = ConstU32<100>;
    type MaxYearRange = ConstU32<100>; // 支持 100 年范围
}
```

### 9.2 OCW 启动配置

```bash
# 生成 OCW 密钥
./target/release/cosmos-node key insert \
  --base-path /tmp/node01 \
  --chain local \
  --scheme Sr25519 \
  --suri "your_secret_seed" \
  --key-type alma

# 启动节点时配置环境变量
export ALMANAC_APPCODE="your_aliyun_appcode"
./target/release/cosmos-node \
  --dev \
  --offchain-worker=Always \
  --enable-offchain-indexing=true
```

---

## 十、预期成果

1. **链上黄历数据库**: 存储至少 1 年的黄历数据 (~365 条记录)
2. **自动更新机制**: OCW 每日自动更新未来 7-30 天数据
3. **前端黄历页面**: 日历视图 + 详情展示
4. **RPC 查询接口**: 支持按日期、月份、年份查询
5. **数据统计**: 数据来源统计、更新状态监控

---

## 十一、后续扩展

1. **择吉功能**: 根据用户需求(如结婚、搬家)推荐吉日
2. **个人八字**: 结合用户生辰八字计算个人宜忌
3. **占卜集成**: 与六爻、梅花易数等占卜功能结合
4. **提醒功能**: 节气、节日提醒
5. **数据可视化**: 年度黄历热力图、吉凶分析图表

---

**总预估工作量**: 15-25 天 (1 人全职开发)

**技术难点**:
1. OCW HTTP 请求和 JSON 解析
2. 签名交易提交
3. 自定义 RPC 注册
4. 前端移动端适配

**关键依赖**:
- 阿里云 API AppCode
- Polkadot SDK offchain 特性
- lite-json 解析库
- React + Ant Design

---

如有任何问题或需要调整,请随时反馈!
