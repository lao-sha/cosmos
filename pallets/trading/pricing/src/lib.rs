#![cfg_attr(not(feature = "std"), no_std)]

//! # Pricing Pallet (定价模块)
//!
//! ## 概述
//! 本模块负责：
//! 1. DUST/USDT 市场价格聚合（OTC + Bridge）
//! 2. CNY/USDT 汇率获取（通过 Offchain Worker）
//! 3. 价格偏离检查
//!
//! ## Offchain Worker
//! - 每24小时自动从 Exchange Rate API 获取 CNY/USD 汇率
//! - API: https://api.exchangerate-api.com/v4/latest/USD
//! - 汇率存储在 offchain local storage 中，供链上查询使用

pub use pallet::*;
pub use pallet::ExchangeRateData;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod ocw;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 函数级中文注释：事件类型绑定到运行时事件
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 函数级中文注释：最大价格偏离（基点，bps）
        /// 用于检查订单创建时的价格是否在合理范围内
        /// 例如：2000 bps = 20%，表示订单价格不能超过基准价格的 ±20%
        /// 目的：防止极端价格订单，保护买卖双方利益
        #[pallet::constant]
        type MaxPriceDeviation: Get<u16>;

        /// 函数级中文注释：汇率更新间隔（区块数）
        /// 默认 14400 个区块（约24小时，假设6秒出块）
        #[pallet::constant]
        type ExchangeRateUpdateInterval: Get<u32>;
    }

    /// 函数级中文注释：订单快照（用于循环缓冲区）
    /// 记录单笔订单的时间、价格和数量，用于后续计算滑动窗口均价
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct OrderSnapshot {
        /// 订单时间戳（Unix 时间戳，毫秒）
        pub timestamp: u64,
        /// USDT 单价（精度 10^6，即 1,000,000 = 1 USDT）
        pub price_usdt: u64,
        /// DUST 数量（精度 10^12，即 1,000,000,000,000 = 1 DUST）
        pub dust_qty: u128,
    }

    /// 函数级中文注释：价格聚合数据
    /// 维护最近累计 1,000,000 DUST 的订单统计信息
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct PriceAggregateData {
        /// 累计 DUST 数量（精度 10^12）
        pub total_dust: u128,
        /// 累计 USDT 金额（精度 10^6）
        pub total_usdt: u128,
        /// 订单数量
        pub order_count: u32,
        /// 最旧订单索引（循环缓冲区指针，0-9999）
        pub oldest_index: u32,
        /// 最新订单索引（循环缓冲区指针，0-9999）
        pub newest_index: u32,
    }

    /// 函数级中文注释：DUST 市场统计信息
    /// 综合 OTC 和 Bridge 两个市场的价格和交易数据
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct MarketStats {
        /// OTC 均价（精度 10^6）
        pub otc_price: u64,
        /// Bridge 均价（精度 10^6）
        pub bridge_price: u64,
        /// 加权平均价格（精度 10^6）
        pub weighted_price: u64,
        /// 简单平均价格（精度 10^6）
        pub simple_avg_price: u64,
        /// OTC 交易量（精度 10^12）
        pub otc_volume: u128,
        /// Bridge 交易量（精度 10^12）
        pub bridge_volume: u128,
        /// 总交易量（精度 10^12）
        pub total_volume: u128,
        /// OTC 订单数
        pub otc_order_count: u32,
        /// Bridge 兑换数
        pub bridge_swap_count: u32,
    }

    /// 函数级中文注释：汇率数据结构
    /// 存储 CNY/USDT 汇率（通过 OCW 从外部 API 获取）
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct ExchangeRateData {
        /// CNY/USD 汇率（精度 10^6，即 7.2345 → 7_234_500）
        /// 注意：假设 USDT = USD，因此 CNY/USDT ≈ CNY/USD
        pub cny_rate: u64,
        /// 更新时间戳（Unix 秒）
        pub updated_at: u64,
    }

    /// 函数级中文注释：OTC 订单价格聚合数据
    /// 维护最近累计 1,000,000 DUST 的 OTC 订单统计
    #[pallet::storage]
    #[pallet::getter(fn otc_aggregate)]
    pub type OtcPriceAggregate<T> = StorageValue<_, PriceAggregateData, ValueQuery>;

    /// 函数级中文注释：OTC 订单历史循环缓冲区
    /// 存储最多 10,000 笔订单快照，通过索引 0-9999 循环使用
    #[pallet::storage]
    pub type OtcOrderRingBuffer<T> = StorageMap<
        _,
        Blake2_128Concat,
        u32,  // 索引 0-9999
        OrderSnapshot,
    >;

    /// 函数级中文注释：Bridge 兑换价格聚合数据
    /// 维护最近累计 1,000,000 DUST 的桥接兑换统计
    #[pallet::storage]
    #[pallet::getter(fn bridge_aggregate)]
    pub type BridgePriceAggregate<T> = StorageValue<_, PriceAggregateData, ValueQuery>;

    /// 函数级中文注释：Bridge 兑换历史循环缓冲区
    /// 存储最多 10,000 笔兑换快照，通过索引 0-9999 循环使用
    #[pallet::storage]
    pub type BridgeOrderRingBuffer<T> = StorageMap<
        _,
        Blake2_128Concat,
        u32,  // 索引 0-9999
        OrderSnapshot,
    >;

    /// 函数级中文注释：冷启动阈值（可治理调整）
    /// 当 OTC 和 Bridge 的交易量都低于此阈值时，使用默认价格
    /// 默认值：100,000,000 DUST（1亿，精度 10^12）
    #[pallet::storage]
    #[pallet::getter(fn cold_start_threshold)]
    pub type ColdStartThreshold<T> = StorageValue<_, u128, ValueQuery, DefaultColdStartThreshold>;

    #[pallet::type_value]
    pub fn DefaultColdStartThreshold() -> u128 {
        100_000_000u128 * 1_000_000_000_000u128 // 1亿MEMO
    }

    /// 函数级中文注释：默认价格（可治理调整）
    /// 用于冷启动阶段的价格锚点
    /// 默认值：1（0.000001 USDT/DUST，精度 10^6）
    /// 注：实际要求 0.0000007，但受精度限制，向上取整为 1
    #[pallet::storage]
    #[pallet::getter(fn default_price)]
    pub type DefaultPrice<T> = StorageValue<_, u64, ValueQuery, DefaultPriceValue>;

    #[pallet::type_value]
    pub fn DefaultPriceValue() -> u64 {
        1u64 // 0.000001 USDT/DUST
        // 注：用户要求 0.0000007，但精度 10^6 下为 0.7，向上取整为 1（最小精度单位）
    }

    /// 函数级中文注释：冷启动退出标记（单向锁定）
    /// 一旦达到阈值并退出冷启动，此标记永久为 true，不再回退到默认价格
    /// 这避免了在阈值附近价格剧烈波动的问题
    #[pallet::storage]
    #[pallet::getter(fn cold_start_exited)]
    pub type ColdStartExited<T> = StorageValue<_, bool, ValueQuery>;

    // ===== CNY/USDT 汇率相关存储 =====

    /// 函数级中文注释：CNY/USDT 汇率数据
    /// 由 Offchain Worker 每24小时从外部 API 获取并更新
    #[pallet::storage]
    #[pallet::getter(fn cny_usdt_rate)]
    pub type CnyUsdtRate<T> = StorageValue<_, ExchangeRateData, ValueQuery>;

    /// 函数级中文注释：上次汇率更新的区块号
    /// 用于判断是否需要触发 OCW 更新
    #[pallet::storage]
    #[pallet::getter(fn last_rate_update_block)]
    pub type LastRateUpdateBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 函数级中文注释：OTC 订单添加到价格聚合
        OtcOrderAdded {
            timestamp: u64,
            price_usdt: u64,
            dust_qty: u128,
            new_avg_price: u64,
        },
        /// 函数级中文注释：Bridge 兑换添加到价格聚合
        BridgeSwapAdded {
            timestamp: u64,
            price_usdt: u64,
            dust_qty: u128,
            new_avg_price: u64,
        },
        /// 函数级中文注释：冷启动参数更新事件
        ColdStartParamsUpdated {
            threshold: Option<u128>,
            default_price: Option<u64>,
        },
        /// 函数级中文注释：冷启动退出事件（标志性事件，市场进入正常定价阶段）
        ColdStartExited {
            final_threshold: u128,
            otc_volume: u128,
            bridge_volume: u128,
            market_price: u64,
        },
        /// M-3修复：冷启动重置事件（治理紧急恢复机制）
        ColdStartReset {
            reason: BoundedVec<u8, ConstU32<256>>,
        },
        /// 函数级中文注释：CNY/USDT 汇率更新事件
        /// 由 Offchain Worker 触发
        ExchangeRateUpdated {
            /// CNY/USD 汇率（精度 10^6）
            cny_rate: u64,
            /// 更新时间戳（Unix 秒）
            updated_at: u64,
            /// 更新时的区块号
            block_number: BlockNumberFor<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 函数级中文注释：冷启动已退出，无法再调整冷启动参数
        ColdStartAlreadyExited,
        /// 函数级中文注释：价格偏离过大，超出允许的最大偏离范围
        /// 订单价格与基准价格的偏离超过了 MaxPriceDeviation 配置的限制
        PriceDeviationTooLarge,
        /// 函数级中文注释：基准价格无效（为0或获取失败）
        InvalidBasePrice,
        /// M-3修复：冷启动未退出，无法重置
        ColdStartNotExited,
        /// 函数级中文注释：汇率无效（为0或格式错误）
        InvalidExchangeRate,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 函数级中文注释：Pallet 辅助方法（聚合数据管理）
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：添加 OTC 订单到价格聚合
        /// 
        /// # 参数
        /// - `timestamp`: 订单时间戳（Unix 毫秒）
        /// - `price_usdt`: USDT 单价（精度 10^6）
        /// - `dust_qty`: DUST 数量（精度 10^12）
        /// 
        /// # 逻辑
        /// 1. 读取当前聚合数据
        /// 2. 如果累计超过 1,000,000 DUST，删除最旧的订单直到满足限制
        /// 3. 添加新订单到循环缓冲区
        /// 4. 更新聚合统计数据
        /// 5. 发出事件
        pub fn add_otc_order(
            timestamp: u64,
            price_usdt: u64,
            dust_qty: u128,
        ) -> DispatchResult {
            let mut agg = OtcPriceAggregate::<T>::get();
            let limit: u128 = 1_000_000u128 * 1_000_000_000_000u128; // 1,000,000 DUST（精度 10^12）
            
            // 如果添加后超过限制，删除最旧的订单
            let mut new_total = agg.total_dust.saturating_add(dust_qty);
            while new_total > limit && agg.order_count > 0 {
                if let Some(oldest) = OtcOrderRingBuffer::<T>::take(agg.oldest_index) {
                    // 从聚合数据中减去
                    agg.total_dust = agg.total_dust.saturating_sub(oldest.dust_qty);
                    let oldest_usdt = (oldest.dust_qty / 1_000_000_000_000u128)
                        .saturating_mul(oldest.price_usdt as u128);
                    agg.total_usdt = agg.total_usdt.saturating_sub(oldest_usdt);
                    agg.order_count = agg.order_count.saturating_sub(1);
                    
                    // 移动最旧索引
                    agg.oldest_index = (agg.oldest_index + 1) % 10000;
                    
                    // 重新计算新总量
                    new_total = agg.total_dust.saturating_add(dust_qty);
                } else {
                    break;
                }
            }
            
            // 添加新订单到循环缓冲区
            let new_index = if agg.order_count == 0 {
                0
            } else {
                (agg.newest_index + 1) % 10000
            };
            
            OtcOrderRingBuffer::<T>::insert(new_index, OrderSnapshot {
                timestamp,
                price_usdt,
                dust_qty,
            });
            
            // 更新聚合数据
            let order_usdt = (dust_qty / 1_000_000_000_000u128)
                .saturating_mul(price_usdt as u128);
            agg.total_dust = agg.total_dust.saturating_add(dust_qty);
            agg.total_usdt = agg.total_usdt.saturating_add(order_usdt);
            agg.order_count = agg.order_count.saturating_add(1);
            agg.newest_index = new_index;
            
            // 保存聚合数据
            OtcPriceAggregate::<T>::put(agg.clone());
            
            // 计算新均价
            let new_avg_price = Self::get_otc_average_price();
            
            // 发出事件
            Self::deposit_event(Event::OtcOrderAdded {
                timestamp,
                price_usdt,
                dust_qty,
                new_avg_price,
            });
            
            Ok(())
        }

        /// 函数级详细中文注释：添加 Bridge 兑换到价格聚合
        /// 逻辑与 add_otc_order 相同，但操作 Bridge 相关的存储
        pub fn add_bridge_swap(
            timestamp: u64,
            price_usdt: u64,
            dust_qty: u128,
        ) -> DispatchResult {
            let mut agg = BridgePriceAggregate::<T>::get();
            let limit: u128 = 1_000_000u128 * 1_000_000_000_000u128; // 1,000,000 DUST
            
            // 删除旧订单直到满足限制
            let mut new_total = agg.total_dust.saturating_add(dust_qty);
            while new_total > limit && agg.order_count > 0 {
                if let Some(oldest) = BridgeOrderRingBuffer::<T>::take(agg.oldest_index) {
                    agg.total_dust = agg.total_dust.saturating_sub(oldest.dust_qty);
                    let oldest_usdt = (oldest.dust_qty / 1_000_000_000_000u128)
                        .saturating_mul(oldest.price_usdt as u128);
                    agg.total_usdt = agg.total_usdt.saturating_sub(oldest_usdt);
                    agg.order_count = agg.order_count.saturating_sub(1);
                    agg.oldest_index = (agg.oldest_index + 1) % 10000;
                    new_total = agg.total_dust.saturating_add(dust_qty);
                } else {
                    break;
                }
            }
            
            // 添加新订单
            let new_index = if agg.order_count == 0 {
                0
            } else {
                (agg.newest_index + 1) % 10000
            };
            
            BridgeOrderRingBuffer::<T>::insert(new_index, OrderSnapshot {
                timestamp,
                price_usdt,
                dust_qty,
            });
            
            // 更新聚合数据
            let order_usdt = (dust_qty / 1_000_000_000_000u128)
                .saturating_mul(price_usdt as u128);
            agg.total_dust = agg.total_dust.saturating_add(dust_qty);
            agg.total_usdt = agg.total_usdt.saturating_add(order_usdt);
            agg.order_count = agg.order_count.saturating_add(1);
            agg.newest_index = new_index;
            
            BridgePriceAggregate::<T>::put(agg.clone());
            
            let new_avg_price = Self::get_bridge_average_price();
            
            Self::deposit_event(Event::BridgeSwapAdded {
                timestamp,
                price_usdt,
                dust_qty,
                new_avg_price,
            });
            
            Ok(())
        }

        /// 函数级详细中文注释：获取 OTC 订单均价（USDT/DUST，精度 10^6）
        /// 
        /// # 返回
        /// - `u64`: 均价（精度 10^6），0 表示无数据
        /// 
        /// # 计算公式
        /// 均价 = 总 USDT / 总 DUST
        ///      = total_usdt / (total_dust / 10^12)
        ///      = (total_usdt * 10^12) / total_dust
        pub fn get_otc_average_price() -> u64 {
            let agg = OtcPriceAggregate::<T>::get();
            if agg.total_dust == 0 {
                return 0;
            }
            // 均价 = (total_usdt * 10^12) / total_dust
            let avg = agg.total_usdt
                .saturating_mul(1_000_000_000_000u128)
                .checked_div(agg.total_dust)
                .unwrap_or(0);
            avg as u64
        }

        /// 函数级详细中文注释：获取 Bridge 兑换均价（USDT/DUST，精度 10^6）
        pub fn get_bridge_average_price() -> u64 {
            let agg = BridgePriceAggregate::<T>::get();
            if agg.total_dust == 0 {
                return 0;
            }
            let avg = agg.total_usdt
                .saturating_mul(1_000_000_000_000u128)
                .checked_div(agg.total_dust)
                .unwrap_or(0);
            avg as u64
        }

        /// 函数级详细中文注释：获取 OTC 聚合统计信息
        /// 返回：(累计DUST, 累计USDT, 订单数, 均价)
        pub fn get_otc_stats() -> (u128, u128, u32, u64) {
            let agg = OtcPriceAggregate::<T>::get();
            let avg = Self::get_otc_average_price();
            (agg.total_dust, agg.total_usdt, agg.order_count, avg)
        }

        /// 函数级详细中文注释：获取 Bridge 聚合统计信息
        /// 返回：(累计DUST, 累计USDT, 订单数, 均价)
        pub fn get_bridge_stats() -> (u128, u128, u32, u64) {
            let agg = BridgePriceAggregate::<T>::get();
            let avg = Self::get_bridge_average_price();
            (agg.total_dust, agg.total_usdt, agg.order_count, avg)
        }

        /// 函数级详细中文注释：获取 DUST 市场参考价格（简单平均 + 冷启动保护）
        /// 
        /// # 算法
        /// - 冷启动阶段：如果两个市场交易量都未达阈值，返回默认价格
        /// - 正常阶段：
        ///   - 如果两个市场都有数据：(OTC均价 + Bridge均价) / 2
        ///   - 如果只有一个市场有数据：使用该市场的均价
        ///   - 如果都无数据：返回默认价格（兜底）
        /// 
        /// # 返回
        /// - `u64`: USDT/DUST 价格（精度 10^6）
        /// 
        /// # 用途
        /// - 前端显示参考价格
        /// - 价格偏离度计算
        /// - 简单的市场概览
        pub fn get_memo_reference_price() -> u64 {
            // 冷启动检查
            if !ColdStartExited::<T>::get() {
                let threshold = ColdStartThreshold::<T>::get();
                let otc_agg = OtcPriceAggregate::<T>::get();
                let bridge_agg = BridgePriceAggregate::<T>::get();
                
                // 如果两个市场都未达阈值，使用默认价格
                if otc_agg.total_dust < threshold && bridge_agg.total_dust < threshold {
                    return DefaultPrice::<T>::get();
                }
                
                // 达到阈值，退出冷启动
                ColdStartExited::<T>::put(true);
                
                // 发出退出冷启动事件
                let market_price = Self::calculate_weighted_average();
                Self::deposit_event(Event::ColdStartExited {
                    final_threshold: threshold,
                    otc_volume: otc_agg.total_dust,
                    bridge_volume: bridge_agg.total_dust,
                    market_price,
                });
            }
            
            // 正常市场价格计算
            let otc_avg = Self::get_otc_average_price();
            let bridge_avg = Self::get_bridge_average_price();
            
            match (otc_avg, bridge_avg) {
                (0, 0) => DefaultPrice::<T>::get(),  // 无数据时返回默认价格
                (0, b) => b,                         // 只有 Bridge
                (o, 0) => o,                         // 只有 OTC
                (o, b) => (o + b) / 2,              // 简单平均
            }
        }

        /// 函数级详细中文注释：获取 DUST 市场价格（加权平均 + 冷启动保护）
        /// 
        /// # 算法
        /// - 冷启动阶段：如果两个市场交易量都未达阈值，返回默认价格
        /// - 正常阶段：加权平均 = (OTC总USDT + Bridge总USDT) / (OTC总MEMO + Bridge总DUST)
        /// 
        /// # 优点
        /// - 考虑交易量权重，更准确反映市场情况
        /// - 大交易量市场的价格权重更高
        /// - 符合市值加权指数的计算方式
        /// - 冷启动保护避免初期价格为0或被操纵
        /// 
        /// # 返回
        /// - `u64`: USDT/DUST 价格（精度 10^6）
        /// 
        /// # 用途
        /// - 资产估值（钱包总值计算）
        /// - 清算价格参考
        /// - 市场指数计算
        pub fn get_dust_market_price_weighted() -> u64 {
            // 冷启动检查
            if !ColdStartExited::<T>::get() {
                let threshold = ColdStartThreshold::<T>::get();
                let otc_agg = OtcPriceAggregate::<T>::get();
                let bridge_agg = BridgePriceAggregate::<T>::get();
                
                // 如果两个市场都未达阈值，使用默认价格
                if otc_agg.total_dust < threshold && bridge_agg.total_dust < threshold {
                    return DefaultPrice::<T>::get();
                }
                
                // 达到阈值，退出冷启动
                ColdStartExited::<T>::put(true);
                
                // 发出退出冷启动事件
                let market_price = Self::calculate_weighted_average();
                Self::deposit_event(Event::ColdStartExited {
                    final_threshold: threshold,
                    otc_volume: otc_agg.total_dust,
                    bridge_volume: bridge_agg.total_dust,
                    market_price,
                });
            }
            
            // 正常市场价格计算
            Self::calculate_weighted_average()
        }
        
        /// 函数级详细中文注释：内部辅助函数 - 计算加权平均价格
        /// 不包含冷启动逻辑，纯粹的数学计算
        fn calculate_weighted_average() -> u64 {
            let otc_agg = OtcPriceAggregate::<T>::get();
            let bridge_agg = BridgePriceAggregate::<T>::get();
            
            let total_dust = otc_agg.total_dust.saturating_add(bridge_agg.total_dust);
            if total_dust == 0 {
                return DefaultPrice::<T>::get(); // 无数据时返回默认价格
            }
            
            // 加权平均 = 总USDT / 总DUST
            let total_usdt = otc_agg.total_usdt.saturating_add(bridge_agg.total_usdt);
            let avg = total_usdt
                .saturating_mul(1_000_000_000_000u128)
                .checked_div(total_dust)
                .unwrap_or(0);
            
            avg as u64
        }

        /// 函数级详细中文注释：获取完整的 DUST 市场统计信息
        /// 
        /// # 返回
        /// `MarketStats` 结构，包含：
        /// - OTC 和 Bridge 各自的均价
        /// - 加权平均价格和简单平均价格
        /// - 各市场的交易量和订单数
        /// - 总交易量
        /// 
        /// # 用途
        /// - 市场概况 Dashboard
        /// - 价格比较和分析
        /// - 交易量统计
        /// - API 查询接口
        pub fn get_market_stats() -> MarketStats {
            let otc_agg = OtcPriceAggregate::<T>::get();
            let bridge_agg = BridgePriceAggregate::<T>::get();
            
            let otc_price = Self::get_otc_average_price();
            let bridge_price = Self::get_bridge_average_price();
            let weighted_price = Self::get_dust_market_price_weighted();
            let simple_avg_price = Self::get_memo_reference_price();
            
            MarketStats {
                otc_price,
                bridge_price,
                weighted_price,
                simple_avg_price,
                otc_volume: otc_agg.total_dust,
                bridge_volume: bridge_agg.total_dust,
                total_volume: otc_agg.total_dust.saturating_add(bridge_agg.total_dust),
                otc_order_count: otc_agg.order_count,
                bridge_swap_count: bridge_agg.order_count,
            }
        }

        /// 函数级详细中文注释：检查价格是否在允许的偏离范围内
        /// 
        /// # 参数
        /// - `order_price_usdt`: 订单价格（USDT单价，精度 10^6，即 1,000,000 = 1 USDT）
        /// 
        /// # 返回
        /// - `Ok(())`: 价格在允许的范围内
        /// - `Err(Error::InvalidBasePrice)`: 基准价格无效（为0）
        /// - `Err(Error::PriceDeviationTooLarge)`: 价格偏离超过限制
        /// 
        /// # 逻辑
        /// 1. 获取当前市场加权平均价格作为基准价格
        /// 2. 验证基准价格有效（> 0）
        /// 3. 计算订单价格与基准价格的偏离率（绝对值，单位：bps）
        /// 4. 检查偏离率是否超过 MaxPriceDeviation 配置的限制
        /// 
        /// # 示例
        /// - 基准价格：1.0 USDT/DUST（1,000,000）
        /// - MaxPriceDeviation：2000 bps（20%）
        /// - 允许范围：0.8 ~ 1.2 USDT/DUST
        /// - 订单价格 1.1 USDT/DUST → 偏离 10% → 通过 ✅
        /// - 订单价格 1.5 USDT/DUST → 偏离 50% → 拒绝 ❌
        /// 
        /// # 用途
        /// - OTC 订单创建时的价格合理性检查
        /// - Bridge 兑换创建时的价格合理性检查
        /// - 防止极端价格订单，保护买卖双方
        pub fn check_price_deviation(order_price_usdt: u64) -> DispatchResult {
            // 1. 获取基准价格（市场加权平均价格）
            let base_price = Self::get_dust_market_price_weighted();
            
            // 2. 验证基准价格有效
            ensure!(base_price > 0, Error::<T>::InvalidBasePrice);
            
            // 3. 计算偏离率（bps）
            // 偏离率 = |订单价格 - 基准价格| / 基准价格 × 10000
            let deviation_bps = if order_price_usdt > base_price {
                // 订单价格高于基准价格（溢价）
                ((order_price_usdt - base_price) as u128)
                    .saturating_mul(10000)
                    .checked_div(base_price as u128)
                    .unwrap_or(0) as u16
            } else {
                // 订单价格低于基准价格（折价）
                ((base_price - order_price_usdt) as u128)
                    .saturating_mul(10000)
                    .checked_div(base_price as u128)
                    .unwrap_or(0) as u16
            };
            
            // 4. 检查是否超出限制
            let max_deviation = T::MaxPriceDeviation::get();
            ensure!(
                deviation_bps <= max_deviation,
                Error::<T>::PriceDeviationTooLarge
            );
            
            Ok(())
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：治理调整冷启动参数
        /// 
        /// # 参数
        /// - `origin`: 必须是 Root 权限
        /// - `threshold`: 可选，新的冷启动阈值（MEMO数量，精度10^12）
        /// - `default_price`: 可选，新的默认价格（USDT/DUST，精度10^6）
        /// 
        /// # 限制
        /// - 只能在冷启动期间调整（ColdStartExited = false）
        /// - 一旦退出冷启动，无法再调整这些参数
        /// 
        /// # 事件
        /// - `ColdStartParamsUpdated`: 参数更新成功
        /// 
        /// # 错误
        /// - `ColdStartAlreadyExited`: 已退出冷启动，无法调整参数
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 2))]
        pub fn set_cold_start_params(
            origin: OriginFor<T>,
            threshold: Option<u128>,
            default_price: Option<u64>,
        ) -> DispatchResult {
            frame_system::EnsureRoot::<T::AccountId>::ensure_origin(origin)?;
            
            // 验证：只能在冷启动期间调整
            ensure!(
                !ColdStartExited::<T>::get(), 
                Error::<T>::ColdStartAlreadyExited
            );
            
            // 更新阈值
            if let Some(t) = threshold {
                ColdStartThreshold::<T>::put(t);
            }
            
            // 更新默认价格
            if let Some(p) = default_price {
                DefaultPrice::<T>::put(p);
            }
            
            // 发出事件
            Self::deposit_event(Event::ColdStartParamsUpdated {
                threshold,
                default_price,
            });
            
            Ok(())
        }
        
        /// M-3修复：治理紧急重置冷启动状态
        ///
        /// 函数级详细中文注释：在极端市场条件下，允许治理重新进入冷启动状态
        ///
        /// # 使用场景
        /// - 市场崩盘，价格长期失真
        /// - 系统维护，需要暂停市场定价
        /// - 数据异常，需要重新校准
        ///
        /// # 参数
        /// - `origin`: 必须是 Root 权限
        /// - `reason`: 重置原因（最多256字节，用于审计和追溯）
        ///
        /// # 效果
        /// - 将 `ColdStartExited` 设置为 false
        /// - 系统将重新使用 `DefaultPrice` 直到市场恢复
        /// - 发出 `ColdStartReset` 事件
        ///
        /// # 错误
        /// - `ColdStartNotExited`: 当前未退出冷启动，无需重置
        ///
        /// # 安全考虑
        /// - 仅限 Root 权限（通常需要治理投票）
        /// - 不清理历史数据，保留市场记录
        /// - 可多次调用，适应复杂市场环境
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn reset_cold_start(
            origin: OriginFor<T>,
            reason: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            frame_system::EnsureRoot::<T::AccountId>::ensure_origin(origin)?;

            // 验证：只有已退出冷启动才能重置
            ensure!(
                ColdStartExited::<T>::get(),
                Error::<T>::ColdStartNotExited
            );

            // 重置冷启动状态
            ColdStartExited::<T>::put(false);

            // 发出事件
            Self::deposit_event(Event::ColdStartReset { reason });

            Ok(())
        }
    }

    // ===== Offchain Worker 钩子 =====

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Offchain Worker 入口点
        ///
        /// 每个区块执行一次，检查是否需要更新汇率
        /// 汇率数据存储在 offchain local storage 中
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            Self::offchain_worker(block_number);
        }
    }

    // ===== 辅助方法：获取 CNY/USDT 汇率 =====

    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：获取当前 CNY/USDT 汇率
        ///
        /// # 返回
        /// - `u64`: CNY/USD 汇率（精度 10^6），如果未设置则返回默认值 7_200_000（7.2）
        pub fn get_cny_usdt_rate() -> u64 {
            let rate_data = CnyUsdtRate::<T>::get();
            if rate_data.cny_rate > 0 {
                rate_data.cny_rate
            } else {
                // 默认汇率：7.2 CNY/USD
                7_200_000
            }
        }

        /// 函数级详细中文注释：将 USDT 金额转换为 CNY
        ///
        /// # 参数
        /// - `usdt_amount`: USDT 金额（精度 10^6）
        ///
        /// # 返回
        /// - `u64`: CNY 金额（精度 10^6）
        ///
        /// # 计算公式
        /// CNY = USDT × 汇率
        pub fn usdt_to_cny(usdt_amount: u64) -> u64 {
            let rate = Self::get_cny_usdt_rate();
            // CNY = USDT * rate / 1_000_000
            (usdt_amount as u128)
                .saturating_mul(rate as u128)
                .saturating_div(1_000_000)
                as u64
        }

        /// 函数级详细中文注释：将 CNY 金额转换为 USDT
        ///
        /// # 参数
        /// - `cny_amount`: CNY 金额（精度 10^6）
        ///
        /// # 返回
        /// - `u64`: USDT 金额（精度 10^6）
        ///
        /// # 计算公式
        /// USDT = CNY / 汇率
        pub fn cny_to_usdt(cny_amount: u64) -> u64 {
            let rate = Self::get_cny_usdt_rate();
            if rate == 0 {
                return 0;
            }
            // USDT = CNY * 1_000_000 / rate
            (cny_amount as u128)
                .saturating_mul(1_000_000)
                .saturating_div(rate as u128)
                as u64
        }
    }
}
