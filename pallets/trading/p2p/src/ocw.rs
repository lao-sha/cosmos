//! # P2P OCW 模块
//!
//! 调用 `pallet-trading-trc20-verifier` 进行 TRC20 交易验证。
//! 本模块仅包含 P2P 特有的 OCW 逻辑（验证触发、结果提交）。
//!
//! ## 设计
//! - TRC20 验证逻辑委托给独立 crate `pallet-trading-trc20-verifier`
//! - 本模块负责：何时触发验证、如何将结果写回链上

pub use pallet_trading_trc20_verifier::{
    verify_trc20_transaction,
    verify_trc20_transaction_simple,
    TronTxVerification,
    AmountStatus,
};
