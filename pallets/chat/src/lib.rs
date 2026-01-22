#![cfg_attr(not(feature = "std"), no_std)]

//! # Pallet Chat - Re-export 层
//! 
//! 此 crate 仅作为 `pallet-chat-core` 的 re-export。
//! 所有实现都在 `pallet-chat-core` 中。
//! 
//! ## 使用方式
//! 
//! ```ignore
//! // 推荐直接使用 pallet-chat-core
//! use pallet_chat_core::*;
//! 
//! // 或通过此 re-export
//! use pallet_chat::*;
//! ```
//! 
//! ## 迁移说明
//! 
//! 🆕 2026-01-20: 代码重复问题修复
//! - 原 `pallet-chat` 代码已迁移到 `pallet-chat-core`
//! - 此 crate 现在仅 re-export `pallet-chat-core`
//! - 建议新代码直接依赖 `pallet-chat-core`

// Re-export everything from pallet-chat-core
pub use pallet_chat_core::*;
