#![cfg_attr(not(feature = "std"), no_std)]

//! # Trading Common (交易公共工具库)
//!
//! ## 概述
//!
//! 本 crate 提供交易相关的公共工具函数，包括：
//! - 脱敏函数（姓名、身份证、生日）
//! - TRON 地址验证
//! - EPAY 配置验证
//!
//! ## 特点
//!
//! - ✅ 纯 Rust crate，无链上存储
//! - ✅ 可被多个 pallet 共享
//! - ✅ no_std 兼容

pub mod mask;
pub mod validation;

// 重新导出主要函数
pub use mask::{mask_name, mask_id_card, mask_birthday};
pub use validation::{is_valid_tron_address, is_valid_epay_config};
