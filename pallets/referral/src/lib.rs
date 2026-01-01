#![cfg_attr(not(feature = "std"), no_std)]

//! # 推荐关系管理模块 (pallet-affiliate-referral)
//!
//! ## 功能概述
//!
//! 本模块负责管理用户之间的推荐关系，从 pallet-affiliate 抽离而来：
//! - **推荐人绑定**：用户通过推荐码绑定推荐人
//! - **推荐码管理**：会员可认领自定义推荐码
//! - **推荐链查询**：获取用户的上级推荐链（最多15层）
//!
//! ## 存储项
//!
//! - `Sponsors`: 推荐人映射（账户 → 推荐人）
//! - `AccountByCode`: 推荐码映射（推荐码 → 账户）
//! - `CodeByAccount`: 账户推荐码（账户 → 推荐码）
//!
//! ## 抽离自
//!
//! - `pallet-affiliate/src/referral.rs`
//!
//! **版本**: 1.0.0
//! **抽离日期**: 2025-12-30

pub use pallet::*;

extern crate alloc;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, BoundedVec};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    /// 推荐链最大层数
    pub const MAX_REFERRAL_CHAIN: u32 = 15;

    /// 推荐码最小长度
    pub const MIN_CODE_LEN: usize = 4;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 🔴 stable2506 API 变更：RuntimeEvent 自动继承，无需显式声明
    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        /// 会员信息提供者（检查是否有效会员）
        type MembershipProvider: MembershipProvider<Self::AccountId>;

        /// 推荐码最大长度
        #[pallet::constant]
        type MaxCodeLen: Get<u32>;

        /// 推荐链最大搜索深度（防止无限循环）
        #[pallet::constant]
        type MaxSearchHops: Get<u32>;
    }

    // ========================================
    // 存储项（3个）
    // ========================================

    /// 推荐人映射：账户 → 推荐人
    #[pallet::storage]
    #[pallet::getter(fn sponsor_of)]
    pub type Sponsors<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId>;

    /// 推荐码映射：推荐码 → 账户
    #[pallet::storage]
    #[pallet::getter(fn account_by_code)]
    pub type AccountByCode<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxCodeLen>,
        T::AccountId,
    >;

    /// 账户推荐码：账户 → 推荐码
    #[pallet::storage]
    #[pallet::getter(fn code_by_account)]
    pub type CodeByAccount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u8, T::MaxCodeLen>,
    >;

    // ========================================
    // 事件
    // ========================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 推荐人绑定成功
        SponsorBound {
            who: T::AccountId,
            sponsor: T::AccountId,
        },
        /// 推荐码认领成功
        CodeClaimed {
            who: T::AccountId,
            code: BoundedVec<u8, T::MaxCodeLen>,
        },
    }

    // ========================================
    // 错误
    // ========================================

    #[pallet::error]
    pub enum Error<T> {
        /// 已绑定推荐人
        AlreadyBound,
        /// 推荐码不存在
        CodeNotFound,
        /// 不能绑定自己
        CannotBindSelf,
        /// 会形成循环绑定
        WouldCreateCycle,
        /// 推荐码过长
        CodeTooLong,
        /// 推荐码过短
        CodeTooShort,
        /// 推荐码已被占用
        CodeAlreadyTaken,
        /// 已拥有推荐码
        AlreadyHasCode,
        /// 非有效会员
        NotMember,
    }

    // ========================================
    // 可调用函数
    // ========================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 绑定推荐人
        ///
        /// 参数：
        /// - `sponsor_code`: 推荐人的推荐码
        ///
        /// 验证：
        /// - 用户未绑定过推荐人
        /// - 推荐码存在
        /// - 不能绑定自己
        /// - 不能形成循环
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn bind_sponsor(
            origin: OriginFor<T>,
            sponsor_code: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_bind_sponsor(who, sponsor_code)
        }

        /// 认领推荐码
        ///
        /// 参数：
        /// - `code`: 要认领的推荐码
        ///
        /// 验证：
        /// - 调用者是有效会员
        /// - 推荐码未被占用
        /// - 用户未认领其他推荐码
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn claim_code(
            origin: OriginFor<T>,
            code: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_claim_code(who, code)
        }
    }

    // ========================================
    // 内部实现
    // ========================================

    impl<T: Config> Pallet<T> {
        /// 获取推荐链（最多15层）
        ///
        /// 参数：
        /// - who: 起始账户
        ///
        /// 返回：推荐链（Vec<AccountId>），从直接推荐人开始
        pub fn get_referral_chain(who: &T::AccountId) -> Vec<T::AccountId> {
            let mut chain = Vec::new();
            let mut current = who.clone();

            for _ in 0..MAX_REFERRAL_CHAIN {
                if let Some(sponsor) = Sponsors::<T>::get(&current) {
                    chain.push(sponsor.clone());
                    current = sponsor;
                } else {
                    break;
                }
            }

            chain
        }

        /// 检查循环绑定
        ///
        /// 防止 A→B→C→A 这种循环
        pub fn would_create_cycle(who: &T::AccountId, sponsor: &T::AccountId) -> bool {
            let mut current = sponsor.clone();
            let max_hops = T::MaxSearchHops::get();

            for _ in 0..max_hops {
                if let Some(next_sponsor) = Sponsors::<T>::get(&current) {
                    if &next_sponsor == who {
                        return true;
                    }
                    current = next_sponsor;
                } else {
                    break;
                }
            }

            false
        }

        /// 通过推荐码查找账户
        pub fn find_account_by_code(
            code: &BoundedVec<u8, T::MaxCodeLen>,
        ) -> Option<T::AccountId> {
            AccountByCode::<T>::get(code)
        }

        /// 自动认领推荐码（默认推荐码）
        ///
        /// 规则：
        /// - 有效会员可自动认领默认推荐码
        /// - 默认推荐码格式：账户ID十六进制前8位
        pub fn try_auto_claim_code(who: &T::AccountId) -> bool {
            // 检查是否已认领
            if CodeByAccount::<T>::contains_key(who) {
                return false;
            }

            // 检查会员有效性
            if !T::MembershipProvider::is_valid_member(who) {
                return false;
            }

            // 生成默认推荐码（账户ID前8位十六进制）
            let account_bytes = who.encode();
            let hex_str: sp_std::vec::Vec<u8> = account_bytes
                .iter()
                .take(4)
                .flat_map(|b| {
                    let hex = alloc::format!("{:02x}", b);
                    hex.into_bytes()
                })
                .collect();

            if let Ok(default_code) = BoundedVec::<u8, T::MaxCodeLen>::try_from(hex_str) {
                // 检查推荐码是否已被占用
                if AccountByCode::<T>::contains_key(&default_code) {
                    return false;
                }

                // 认领推荐码
                AccountByCode::<T>::insert(&default_code, who);
                CodeByAccount::<T>::insert(who, &default_code);

                // 发射事件
                Self::deposit_event(Event::CodeClaimed {
                    who: who.clone(),
                    code: default_code,
                });

                true
            } else {
                false
            }
        }

        /// 绑定推荐人内部实现
        pub(crate) fn do_bind_sponsor(
            who: T::AccountId,
            sponsor_code: Vec<u8>,
        ) -> DispatchResult {
            // 验证：用户未绑定过
            ensure!(
                !Sponsors::<T>::contains_key(&who),
                Error::<T>::AlreadyBound
            );

            // 转换为 BoundedVec
            let code: BoundedVec<u8, T::MaxCodeLen> = sponsor_code
                .try_into()
                .map_err(|_| Error::<T>::CodeTooLong)?;

            // 验证：推荐码长度
            ensure!(
                code.len() >= MIN_CODE_LEN,
                Error::<T>::CodeTooShort
            );

            // 查找推荐人
            let sponsor = Self::find_account_by_code(&code)
                .ok_or(Error::<T>::CodeNotFound)?;

            // 验证：不能绑定自己
            ensure!(sponsor != who, Error::<T>::CannotBindSelf);

            // 验证：不能形成循环
            ensure!(
                !Self::would_create_cycle(&who, &sponsor),
                Error::<T>::WouldCreateCycle
            );

            // 绑定推荐人
            Sponsors::<T>::insert(&who, &sponsor);

            // 发射事件
            Self::deposit_event(Event::SponsorBound {
                who: who.clone(),
                sponsor: sponsor.clone(),
            });

            Ok(())
        }

        /// 认领推荐码内部实现
        pub(crate) fn do_claim_code(
            who: T::AccountId,
            code_vec: Vec<u8>,
        ) -> DispatchResult {
            // 验证：调用者是有效会员
            ensure!(
                T::MembershipProvider::is_valid_member(&who),
                Error::<T>::NotMember
            );

            // 转换为 BoundedVec
            let code: BoundedVec<u8, T::MaxCodeLen> = code_vec
                .try_into()
                .map_err(|_| Error::<T>::CodeTooLong)?;

            // 验证：推荐码长度
            ensure!(
                code.len() >= MIN_CODE_LEN,
                Error::<T>::CodeTooShort
            );

            // 验证：推荐码未被占用
            ensure!(
                !AccountByCode::<T>::contains_key(&code),
                Error::<T>::CodeAlreadyTaken
            );

            // 验证：用户未认领其他推荐码
            ensure!(
                !CodeByAccount::<T>::contains_key(&who),
                Error::<T>::AlreadyHasCode
            );

            // 认领推荐码
            AccountByCode::<T>::insert(&code, &who);
            CodeByAccount::<T>::insert(&who, &code);

            // 发射事件
            Self::deposit_event(Event::CodeClaimed {
                who: who.clone(),
                code,
            });

            Ok(())
        }
    }
}

// ========================================
// Trait 定义
// ========================================

/// 会员信息提供者 trait
pub trait MembershipProvider<AccountId> {
    /// 检查账户是否为有效会员
    fn is_valid_member(who: &AccountId) -> bool;
}

/// 推荐关系提供者 trait（供其他模块调用）
pub trait ReferralProvider<AccountId> {
    /// 获取推荐人
    fn get_sponsor(who: &AccountId) -> Option<AccountId>;

    /// 获取推荐链（最多15层）
    fn get_referral_chain(who: &AccountId) -> sp_std::vec::Vec<AccountId>;
}

impl<T: Config> ReferralProvider<T::AccountId> for Pallet<T> {
    fn get_sponsor(who: &T::AccountId) -> Option<T::AccountId> {
        Sponsors::<T>::get(who)
    }

    fn get_referral_chain(who: &T::AccountId) -> sp_std::vec::Vec<T::AccountId> {
        Self::get_referral_chain(who)
    }
}
