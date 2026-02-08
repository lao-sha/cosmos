#![cfg_attr(not(feature = "std"), no_std)]

//! # Pricing Pallet (定价模块)
//!
//! ## 概述
//! 本模块负责：
//! 1. NXS/USDT 市场价格聚合（P2P Buy + Sell 两方向）
//! 2. CNY/USDT 汇率获取（通过 Offchain Worker）
//! 3. 价格偏离检查
//!
//! ## 版本历史
//! - v1.4.0 (2026-02-08): 适配 P2P 统一模型，OTC→Buy, Bridge→Sell
//!
//! ## Offchain Worker
//! - 每24小时自动从 Exchange Rate API 获取 CNY/USD 汇率
//! - API: https://api.exchangerate-api.com/v4/latest/USD
//! - 汇率存储在 offchain local storage 中，供链上查询使用

pub use pallet::*;
pub use pallet::ExchangeRateData;

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod ocw;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        traits::Saturating,
        transaction_validity::{
            InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
        },
    };

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

    // ===== P3修复：类型安全的循环缓冲区索引 =====
    
    /// 循环缓冲区大小（10,000 条订单）
    pub const RING_BUFFER_SIZE: u32 = 10_000;
    
    /// 函数级中文注释：类型安全的循环缓冲区索引
    /// 封装索引操作，确保始终在有效范围内
    #[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default, PartialEq, Eq)]
    pub struct RingBufferIndex(pub u32);
    
    impl RingBufferIndex {
        /// 创建新索引（自动取模确保在范围内）
        pub fn new(value: u32) -> Self {
            Self(value % RING_BUFFER_SIZE)
        }
        
        /// 获取下一个索引
        pub fn next(self) -> Self {
            Self((self.0 + 1) % RING_BUFFER_SIZE)
        }
        
        /// 重置为 0
        pub fn reset() -> Self {
            Self(0)
        }
        
        /// 获取原始值
        pub fn value(self) -> u32 {
            self.0
        }
    }

    /// 函数级中文注释：订单快照
    /// 记录单笔订单的时间、价格和数量，用于后续计算滑动窗口均价
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct OrderSnapshot {
        /// 订单时间戳（Unix 时间戳，毫秒）
        pub timestamp: u64,
        /// USDT 单价（精度 10^6，即 1,000,000 = 1 USDT）
        pub price_usdt: u64,
        /// NXS 数量（精度 10^12，即 1,000,000,000,000 = 1 NXS）
        pub nxs_qty: u128,
    }

    /// 函数级中文注释：价格聚合数据
    /// 维护最近累计 1,000,000 NXS 的订单统计信息
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct PriceAggregateData {
        /// 累计 NXS 数量（精度 10^12）
        pub total_nxs: u128,
        /// 累计 USDT 金额（精度 10^6）
        pub total_usdt: u128,
        /// 订单数量
        pub order_count: u32,
        /// P3修复：最旧订单索引（类型安全）
        pub oldest_index: RingBufferIndex,
        /// P3修复：最新订单索引（类型安全）
        pub newest_index: RingBufferIndex,
    }

    /// 函数级中文注释：NXS 市场统计信息
    /// 综合 Buy 和 Sell 两方向的价格和交易数据
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct MarketStats {
        /// Buy 方向均价（精度 10^6）
        pub buy_price: u64,
        /// Sell 方向均价（精度 10^6）
        pub sell_price: u64,
        /// 加权平均价格（精度 10^6）
        pub weighted_price: u64,
        /// 简单平均价格（精度 10^6）
        pub simple_avg_price: u64,
        /// Buy 方向交易量（精度 10^12）
        pub buy_volume: u128,
        /// Sell 方向交易量（精度 10^12）
        pub sell_volume: u128,
        /// 总交易量（精度 10^12）
        pub total_volume: u128,
        /// Buy 方向订单数
        pub buy_order_count: u32,
        /// Sell 方向订单数
        pub sell_order_count: u32,
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

    /// 函数级中文注释：Buy 方向（USDT→NXS）价格聚合数据
    /// 维护最近累计 1,000,000 NXS 的 Buy 方向订单统计
    #[pallet::storage]
    #[pallet::getter(fn buy_aggregate)]
    pub type BuyPriceAggregate<T> = StorageValue<_, PriceAggregateData, ValueQuery>;

    /// 函数级中文注释：Buy 方向订单历史循环缓冲区
    /// 存储最多 10,000 笔订单快照，通过索引 0-9999 循环使用
    #[pallet::storage]
    pub type BuyOrderRingBuffer<T> = StorageMap<
        _,
        Blake2_128Concat,
        u32,  // 索引 0-9999
        OrderSnapshot,
    >;

    /// 函数级中文注释：Sell 方向（NXS→USDT）价格聚合数据
    /// 维护最近累计 1,000,000 NXS 的 Sell 方向订单统计
    #[pallet::storage]
    #[pallet::getter(fn sell_aggregate)]
    pub type SellPriceAggregate<T> = StorageValue<_, PriceAggregateData, ValueQuery>;

    /// 函数级中文注释：Sell 方向订单历史循环缓冲区
    /// 存储最多 10,000 笔订单快照，通过索引 0-9999 循环使用
    #[pallet::storage]
    pub type SellOrderRingBuffer<T> = StorageMap<
        _,
        Blake2_128Concat,
        u32,  // 索引 0-9999
        OrderSnapshot,
    >;

    /// 函数级中文注释：冷启动阈值（可治理调整）
    /// 当 Buy 和 Sell 两方向的交易量都低于此阈值时，使用默认价格
    /// 默认值：1,000,000,000 NXS（10亿，精度 10^12）
    #[pallet::storage]
    #[pallet::getter(fn cold_start_threshold)]
    pub type ColdStartThreshold<T> = StorageValue<_, u128, ValueQuery, DefaultColdStartThreshold>;

    #[pallet::type_value]
    pub fn DefaultColdStartThreshold() -> u128 {
        // 冷启动阈值：10亿 NXS
        1_000_000_000u128 * 1_000_000_000_000u128 // 10亿 NXS
    }

    /// 函数级中文注释：默认价格（可治理调整）
    /// 用于冷启动阶段的价格锚点
    /// 默认值：1（0.000001 USDT/NXS，精度 10^6）
    /// 注：实际要求 0.0000007，但受精度限制，向上取整为 1
    #[pallet::storage]
    #[pallet::getter(fn default_price)]
    pub type DefaultPrice<T> = StorageValue<_, u64, ValueQuery, DefaultPriceValue>;

    #[pallet::type_value]
    pub fn DefaultPriceValue() -> u64 {
        1u64 // 0.000001 USDT/NXS
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
        /// 函数级中文注释：Buy 方向成交添加到价格聚合
        BuyTradeAdded {
            timestamp: u64,
            price_usdt: u64,
            nxs_qty: u128,
            new_avg_price: u64,
        },
        /// 函数级中文注释：Sell 方向成交添加到价格聚合
        SellTradeAdded {
            timestamp: u64,
            price_usdt: u64,
            nxs_qty: u128,
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
            buy_volume: u128,
            sell_volume: u128,
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
        /// P1修复：无效的价格（必须 > 0）
        InvalidPrice,
        /// P1修复：无效的数量（必须 > 0）
        InvalidQuantity,
        /// P2修复：算术溢出
        ArithmeticOverflow,
        /// P3修复：单笔订单数量超过上限
        OrderTooLarge,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 函数级中文注释：Pallet 辅助方法（聚合数据管理）
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：添加 Buy 方向（USDT→NXS）成交到价格聚合
        /// 
        /// # 参数
        /// - `timestamp`: 成交时间戳（Unix 毫秒）
        /// - `price_usdt`: USDT 单价（精度 10^6）
        /// - `nxs_qty`: NXS 数量（精度 10^12）
        /// 
        /// # 逻辑
        /// 1. 读取当前聚合数据
        /// 2. 如果累计超过 1,000,000 NXS，删除最旧的订单直到满足限制
        /// 3. 添加新订单到循环缓冲区
        /// 4. 更新聚合统计数据
        /// 5. 发出事件
        /// P3修复：单笔订单最大 NXS 数量（1000万 NXS）
        const MAX_SINGLE_ORDER_COS: u128 = 10_000_000u128 * 1_000_000_000_000u128;
        
        pub fn add_buy_trade(
            timestamp: u64,
            price_usdt: u64,
            nxs_qty: u128,
        ) -> DispatchResult {
            // P1修复：输入验证
            ensure!(price_usdt > 0, Error::<T>::InvalidPrice);
            ensure!(nxs_qty > 0, Error::<T>::InvalidQuantity);
            // P3修复：单笔订单上限验证
            ensure!(nxs_qty <= Self::MAX_SINGLE_ORDER_COS, Error::<T>::OrderTooLarge);
            
            let mut agg = BuyPriceAggregate::<T>::get();
            let limit: u128 = 1_000_000u128 * 1_000_000_000_000u128; // 1,000,000 NXS（精度 10^12）
            
            // 如果添加后超过限制，删除最旧的订单
            let mut new_total = agg.total_nxs.saturating_add(nxs_qty);
            while new_total > limit && agg.order_count > 0 {
                // P3修复：使用类型安全的索引
                if let Some(oldest) = BuyOrderRingBuffer::<T>::take(agg.oldest_index.value()) {
                    // 从聚合数据中减去
                    agg.total_nxs = agg.total_nxs.saturating_sub(oldest.nxs_qty);
                    // P0修复：先乘后除，避免精度丢失
                    let oldest_usdt = oldest.nxs_qty
                        .saturating_mul(oldest.price_usdt as u128)
                        / 1_000_000_000_000u128;
                    agg.total_usdt = agg.total_usdt.saturating_sub(oldest_usdt);
                    agg.order_count = agg.order_count.saturating_sub(1);
                    
                    // P3修复：使用类型安全的索引移动
                    agg.oldest_index = agg.oldest_index.next();
                    
                    // 重新计算新总量
                    new_total = agg.total_nxs.saturating_add(nxs_qty);
                } else {
                    break;
                }
            }
            
            // 添加新订单到循环缓冲区
            // P0-2修复：order_count=0 时重置索引，避免覆盖旧数据
            // P3修复：使用类型安全的索引
            let new_index = if agg.order_count == 0 {
                agg.oldest_index = RingBufferIndex::reset();
                agg.newest_index = RingBufferIndex::reset();
                RingBufferIndex::reset()
            } else {
                agg.newest_index.next()
            };
            
            BuyOrderRingBuffer::<T>::insert(new_index.value(), OrderSnapshot {
                timestamp,
                price_usdt,
                nxs_qty,
            });
            
            // 更新聚合数据
            // P0修复：先乘后除，避免精度丢失
            // P2修复：使用 checked_mul/checked_add 防止溢出
            let order_usdt = nxs_qty
                .checked_mul(price_usdt as u128)
                .ok_or(Error::<T>::ArithmeticOverflow)?
                / 1_000_000_000_000u128;
            agg.total_nxs = agg.total_nxs
                .checked_add(nxs_qty)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            agg.total_usdt = agg.total_usdt
                .checked_add(order_usdt)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            agg.order_count = agg.order_count.saturating_add(1);
            agg.newest_index = new_index;
            
            // 保存聚合数据
            BuyPriceAggregate::<T>::put(agg.clone());
            
            // 计算新均价
            let new_avg_price = Self::get_buy_average_price();
            
            // 发出事件
            Self::deposit_event(Event::BuyTradeAdded {
                timestamp,
                price_usdt,
                nxs_qty,
                new_avg_price,
            });
            
            Ok(())
        }

        /// 函数级详细中文注释：添加 Sell 方向（NXS→USDT）成交到价格聚合
        /// 逻辑与 add_buy_trade 相同，但操作 Sell 方向的存储
        pub fn add_sell_trade(
            timestamp: u64,
            price_usdt: u64,
            nxs_qty: u128,
        ) -> DispatchResult {
            // P1修复：输入验证
            ensure!(price_usdt > 0, Error::<T>::InvalidPrice);
            ensure!(nxs_qty > 0, Error::<T>::InvalidQuantity);
            // P3修复：单笔订单上限验证
            ensure!(nxs_qty <= Self::MAX_SINGLE_ORDER_COS, Error::<T>::OrderTooLarge);
            
            let mut agg = SellPriceAggregate::<T>::get();
            let limit: u128 = 1_000_000u128 * 1_000_000_000_000u128; // 1,000,000 NXS
            
            // 删除旧订单直到满足限制
            let mut new_total = agg.total_nxs.saturating_add(nxs_qty);
            while new_total > limit && agg.order_count > 0 {
                // P3修复：使用类型安全的索引
                if let Some(oldest) = SellOrderRingBuffer::<T>::take(agg.oldest_index.value()) {
                    agg.total_nxs = agg.total_nxs.saturating_sub(oldest.nxs_qty);
                    // P0修复：先乘后除，避免精度丢失
                    let oldest_usdt = oldest.nxs_qty
                        .saturating_mul(oldest.price_usdt as u128)
                        / 1_000_000_000_000u128;
                    agg.total_usdt = agg.total_usdt.saturating_sub(oldest_usdt);
                    agg.order_count = agg.order_count.saturating_sub(1);
                    // P3修复：使用类型安全的索引移动
                    agg.oldest_index = agg.oldest_index.next();
                    new_total = agg.total_nxs.saturating_add(nxs_qty);
                } else {
                    break;
                }
            }
            
            // 添加新订单
            // P0-2修复：order_count=0 时重置索引，避免覆盖旧数据
            // P3修复：使用类型安全的索引
            let new_index = if agg.order_count == 0 {
                agg.oldest_index = RingBufferIndex::reset();
                agg.newest_index = RingBufferIndex::reset();
                RingBufferIndex::reset()
            } else {
                agg.newest_index.next()
            };
            
            SellOrderRingBuffer::<T>::insert(new_index.value(), OrderSnapshot {
                timestamp,
                price_usdt,
                nxs_qty,
            });
            
            // 更新聚合数据
            // P0修复：先乘后除，避免精度丢失
            // P2修复：使用 checked_mul/checked_add 防止溢出
            let order_usdt = nxs_qty
                .checked_mul(price_usdt as u128)
                .ok_or(Error::<T>::ArithmeticOverflow)?
                / 1_000_000_000_000u128;
            agg.total_nxs = agg.total_nxs
                .checked_add(nxs_qty)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            agg.total_usdt = agg.total_usdt
                .checked_add(order_usdt)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            agg.order_count = agg.order_count.saturating_add(1);
            agg.newest_index = new_index;
            
            SellPriceAggregate::<T>::put(agg.clone());
            
            let new_avg_price = Self::get_sell_average_price();
            
            Self::deposit_event(Event::SellTradeAdded {
                timestamp,
                price_usdt,
                nxs_qty,
                new_avg_price,
            });
            
            Ok(())
        }

        /// 函数级详细中文注释：获取 Buy 方向均价（USDT/NXS，精度 10^6）
        /// 
        /// # 返回
        /// - `u64`: 均价（精度 10^6），0 表示无数据
        /// 
        /// # 计算公式
        /// 均价 = 总 USDT / 总 NXS
        ///      = total_usdt / (total_nxs / 10^12)
        ///      = (total_usdt * 10^12) / total_nxs
        pub fn get_buy_average_price() -> u64 {
            let agg = BuyPriceAggregate::<T>::get();
            if agg.total_nxs == 0 {
                return 0;
            }
            // 均价 = (total_usdt * 10^12) / total_nxs
            let avg = agg.total_usdt
                .saturating_mul(1_000_000_000_000u128)
                .checked_div(agg.total_nxs)
                .unwrap_or(0);
            // P3修复：安全类型转换，避免截断
            avg.min(u64::MAX as u128) as u64
        }

        /// 函数级详细中文注释：获取 Sell 方向均价（USDT/NXS，精度 10^6）
        pub fn get_sell_average_price() -> u64 {
            let agg = SellPriceAggregate::<T>::get();
            if agg.total_nxs == 0 {
                return 0;
            }
            let avg = agg.total_usdt
                .saturating_mul(1_000_000_000_000u128)
                .checked_div(agg.total_nxs)
                .unwrap_or(0);
            // P3修复：安全类型转换，避免截断
            avg.min(u64::MAX as u128) as u64
        }

        /// 函数级详细中文注释：获取 Buy 方向聚合统计信息
        /// 返回：(累计NXS, 累计USDT, 订单数, 均价)
        pub fn get_buy_stats() -> (u128, u128, u32, u64) {
            let agg = BuyPriceAggregate::<T>::get();
            let avg = Self::get_buy_average_price();
            (agg.total_nxs, agg.total_usdt, agg.order_count, avg)
        }

        /// 函数级详细中文注释：获取 Sell 方向聚合统计信息
        /// 返回：(累计NXS, 累计USDT, 订单数, 均价)
        pub fn get_sell_stats() -> (u128, u128, u32, u64) {
            let agg = SellPriceAggregate::<T>::get();
            let avg = Self::get_sell_average_price();
            (agg.total_nxs, agg.total_usdt, agg.order_count, avg)
        }

        /// 函数级详细中文注释：获取 NXS 市场参考价格（简单平均 + 冷启动保护）
        /// 
        /// # 算法
        /// - 冷启动阶段：如果两个市场交易量都未达阈值，返回默认价格
        /// - 正常阶段：
        ///   - 如果两个方向都有数据：(Buy均价 + Sell均价) / 2
        ///   - 如果只有一个方向有数据：使用该方向的均价
        ///   - 如果都无数据：返回默认价格（兜底）
        /// 
        /// # 返回
        /// - `u64`: USDT/NXS 价格（精度 10^6）
        /// 
        /// # 用途
        /// - 前端显示参考价格
        /// - 价格偏离度计算
        /// - 简单的市场概览
        pub fn get_memo_reference_price() -> u64 {
            // 🆕 2026-01-18: 使用统一的冷启动检查函数，避免重复触发事件
            if Self::check_cold_start_and_maybe_exit() {
                return DefaultPrice::<T>::get();
            }
            
            // 正常市场价格计算
            let buy_avg = Self::get_buy_average_price();
            let sell_avg = Self::get_sell_average_price();
            
            match (buy_avg, sell_avg) {
                (0, 0) => DefaultPrice::<T>::get(),  // 无数据时返回默认价格
                (0, s) => s,                         // 只有 Sell
                (b, 0) => b,                         // 只有 Buy
                (b, s) => (b + s) / 2,              // 简单平均
            }
        }

        /// 函数级详细中文注释：获取 NXS 市场价格（加权平均 + 冷启动保护）
        /// 
        /// # 算法
        /// - 冷启动阶段：如果两个市场交易量都未达阈值，返回默认价格
        /// - 正常阶段：加权平均 = (Buy总USDT + Sell总USDT) / (Buy总NXS + Sell总NXS)
        /// 
        /// # 优点
        /// - 考虑交易量权重，更准确反映市场情况
        /// - 大交易量市场的价格权重更高
        /// - 符合市值加权指数的计算方式
        /// - 冷启动保护避免初期价格为0或被操纵
        /// 
        /// # 返回
        /// - `u64`: USDT/NXS 价格（精度 10^6）
        /// 
        /// # 用途
        /// - 资产估值（钱包总值计算）
        /// - 清算价格参考
        /// - 市场指数计算
        pub fn get_cos_market_price_weighted() -> u64 {
            // 🆕 2026-01-18: 使用统一的冷启动检查函数，避免重复触发事件
            if Self::check_cold_start_and_maybe_exit() {
                return DefaultPrice::<T>::get();
            }
            
            // 正常市场价格计算
            Self::calculate_weighted_average()
        }
        
        // ===== 🆕 2026-01-18: 冷启动检查统一函数 =====
        
        /// 函数级详细中文注释：检查并处理冷启动退出
        /// 
        /// ## 功能说明
        /// 统一的冷启动检查函数，避免在多个价格查询函数中重复触发事件。
        /// 
        /// ## 返回值
        /// - `true`: 仍在冷启动阶段，应使用默认价格
        /// - `false`: 已退出冷启动，可使用市场价格
        /// 
        /// ## 事件触发
        /// 仅在首次达到阈值时触发 `ColdStartExited` 事件（由存储状态保证）
        fn check_cold_start_and_maybe_exit() -> bool {
            // 已退出冷启动，直接返回
            if ColdStartExited::<T>::get() {
                return false;
            }
            
            let threshold = ColdStartThreshold::<T>::get();
            let buy_agg = BuyPriceAggregate::<T>::get();
            let sell_agg = SellPriceAggregate::<T>::get();
            
            // 未达阈值，仍在冷启动阶段
            if buy_agg.total_nxs < threshold && sell_agg.total_nxs < threshold {
                return true;
            }
            
            // 达到阈值，退出冷启动（仅执行一次，由 ColdStartExited 存储保证）
            ColdStartExited::<T>::put(true);
            
            // 发出退出冷启动事件
            let market_price = Self::calculate_weighted_average();
            Self::deposit_event(Event::ColdStartExited {
                final_threshold: threshold,
                buy_volume: buy_agg.total_nxs,
                sell_volume: sell_agg.total_nxs,
                market_price,
            });
            
            false
        }
        
        /// 函数级详细中文注释：内部辅助函数 - 计算加权平均价格
        /// 不包含冷启动逻辑，纯粹的数学计算
        fn calculate_weighted_average() -> u64 {
            let buy_agg = BuyPriceAggregate::<T>::get();
            let sell_agg = SellPriceAggregate::<T>::get();
            
            let total_nxs = buy_agg.total_nxs.saturating_add(sell_agg.total_nxs);
            if total_nxs == 0 {
                return DefaultPrice::<T>::get(); // 无数据时返回默认价格
            }
            
            // 加权平均 = 总USDT / 总NXS
            let total_usdt = buy_agg.total_usdt.saturating_add(sell_agg.total_usdt);
            let avg = total_usdt
                .saturating_mul(1_000_000_000_000u128)
                .checked_div(total_nxs)
                .unwrap_or(0);
            
            // P3修复：安全类型转换，避免截断
            avg.min(u64::MAX as u128) as u64
        }

        /// 函数级详细中文注释：获取完整的 NXS 市场统计信息
        /// 
        /// # 返回
        /// `MarketStats` 结构，包含：
        /// - Buy 和 Sell 各自的均价
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
            let buy_agg = BuyPriceAggregate::<T>::get();
            let sell_agg = SellPriceAggregate::<T>::get();
            
            let buy_price = Self::get_buy_average_price();
            let sell_price = Self::get_sell_average_price();
            let weighted_price = Self::get_cos_market_price_weighted();
            let simple_avg_price = Self::get_memo_reference_price();
            
            MarketStats {
                buy_price,
                sell_price,
                weighted_price,
                simple_avg_price,
                buy_volume: buy_agg.total_nxs,
                sell_volume: sell_agg.total_nxs,
                total_volume: buy_agg.total_nxs.saturating_add(sell_agg.total_nxs),
                buy_order_count: buy_agg.order_count,
                sell_order_count: sell_agg.order_count,
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
        /// - 基准价格：1.0 USDT/NXS（1,000,000）
        /// - MaxPriceDeviation：2000 bps（20%）
        /// - 允许范围：0.8 ~ 1.2 USDT/NXS
        /// - 订单价格 1.1 USDT/NXS → 偏离 10% → 通过 ✅
        /// - 订单价格 1.5 USDT/NXS → 偏离 50% → 拒绝 ❌
        /// 
        /// # 用途
        /// - P2P Buy 订单创建时的价格合理性检查
        /// - P2P Sell 订单创建时的价格合理性检查
        /// - 防止极端价格订单，保护买卖双方
        pub fn check_price_deviation(order_price_usdt: u64) -> DispatchResult {
            // 1. 获取基准价格（市场加权平均价格）
            let base_price = Self::get_cos_market_price_weighted();
            
            // 2. 验证基准价格有效
            ensure!(base_price > 0, Error::<T>::InvalidBasePrice);
            
            // 3. 计算偏离率（bps）
            // 偏离率 = |订单价格 - 基准价格| / 基准价格 × 10000
            let deviation_u128 = if order_price_usdt > base_price {
                // 订单价格高于基准价格（溢价）
                ((order_price_usdt - base_price) as u128)
                    .saturating_mul(10000)
                    .checked_div(base_price as u128)
                    .unwrap_or(0)
            } else {
                // 订单价格低于基准价格（折价）
                ((base_price - order_price_usdt) as u128)
                    .saturating_mul(10000)
                    .checked_div(base_price as u128)
                    .unwrap_or(0)
            };
            
            // P2修复：提前检查防止 u128 → u16 截断导致错误通过
            // 如果偏离率超过 u16::MAX，直接拒绝（极端价格）
            ensure!(
                deviation_u128 <= u16::MAX as u128,
                Error::<T>::PriceDeviationTooLarge
            );
            let deviation_bps = deviation_u128 as u16;
            
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
        
        /// P0-1修复：OCW 提交汇率（无签名交易）
        ///
        /// # 权限
        /// - 仅 OCW 可调用（通过 ValidateUnsigned 验证）
        ///
        /// # 参数
        /// - `cny_rate`: CNY/USD 汇率（精度 10^6）
        /// - `updated_at`: 更新时间戳（Unix 秒）
        ///
        /// # 验证
        /// - 汇率必须在合理范围内（5.0 ~ 10.0）
        /// - 更新间隔必须超过配置的最小间隔
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn ocw_submit_exchange_rate(
            origin: OriginFor<T>,
            cny_rate: u64,
            updated_at: u64,
        ) -> DispatchResult {
            ensure_none(origin)?;
            
            // 验证汇率在合理范围内（5.0 ~ 10.0 CNY/USD）
            ensure!(
                cny_rate >= 5_000_000 && cny_rate <= 10_000_000,
                Error::<T>::InvalidExchangeRate
            );
            
            // 更新链上存储
            let rate_data = ExchangeRateData {
                cny_rate,
                updated_at,
            };
            CnyUsdtRate::<T>::put(rate_data);
            
            // 更新最后更新区块
            let current_block = frame_system::Pallet::<T>::block_number();
            LastRateUpdateBlock::<T>::put(current_block);
            
            // 发出事件
            Self::deposit_event(Event::ExchangeRateUpdated {
                cny_rate,
                updated_at,
                block_number: current_block,
            });
            
            Ok(())
        }
    }
    
    // ===== P0-1修复：OCW 无签名交易验证 =====
    
    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;
        
        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            match call {
                Call::ocw_submit_exchange_rate { cny_rate, .. } => {
                    // 验证汇率在合理范围内
                    if *cny_rate < 5_000_000 || *cny_rate > 10_000_000 {
                        return InvalidTransaction::Call.into();
                    }
                    
                    // 检查更新间隔
                    let current_block = frame_system::Pallet::<T>::block_number();
                    let last_update = LastRateUpdateBlock::<T>::get();
                    let interval = T::ExchangeRateUpdateInterval::get();
                    
                    // 如果距离上次更新不足间隔时间，拒绝交易
                    let interval_block: BlockNumberFor<T> = interval.into();
                    if current_block.saturating_sub(last_update) < interval_block {
                        return InvalidTransaction::Stale.into();
                    }
                    
                    ValidTransaction::with_tag_prefix("PricingOCW")
                        .priority(100)
                        .longevity(5)
                        .and_provides([b"exchange_rate"])
                        .propagate(true)
                        .build()
                },
                _ => InvalidTransaction::Call.into(),
            }
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
