//! 函数级中文注释：周结算子模块（精简版）
//!
//! 功能：
//! - 消费上报（report_consumption）
//! - 应得累计
//! - 活跃期管理（简化）
//! - 周期结算（settle_cycle）
//!
//! 整合自：pallet-affiliate-weekly（精简优化）
//!
//! 精简优化：
//! - 移除：持仓门槛验证
//! - 移除：复杂的直推有效数算法
//! - 保留：活跃期管理（供奉延长活跃期）
//! - 保留：简化的直推计数

use super::*;
use sp_runtime::traits::{Saturating, Zero, SaturatedConversion};
use pallet_affiliate_referral::MembershipProvider;

/// 函数级中文注释：周结算实现
impl<T: Config> Pallet<T> {
    /// 函数级中文注释：上报消费（Weekly模式）
    ///
    /// 参数：
    /// - buyer: 购买者/供奉者
    /// - distributable_amount: 可分配金额（已扣除系统费用）
    /// - duration_weeks: 供奉时长（周）
    /// - levels: 分配层数（用于 Hybrid 模式）
    ///
    /// 功能：
    /// - 计算当前周编号
    /// - 累计应得金额到各层推荐人
    /// - 更新活跃期（如有时长）
    pub fn do_report_consumption(
        buyer: &T::AccountId,
        distributable_amount: BalanceOf<T>,
        duration_weeks: Option<u32>,
        levels: u8,
    ) {
        // 计算当前周编号
        let now = <frame_system::Pallet<T>>::block_number();
        let blocks_per_week = BlocksPerWeek::<T>::get();
        let current_cycle = (now.saturated_into::<u32>()) / (blocks_per_week.saturated_into::<u32>());

        // 获取推荐链
        let referral_chain = Self::get_referral_chain(buyer);

        // 获取周结算分成比例配置
        let level_percents = WeeklyLevelPercents::<T>::get();

        let levels_to_process = levels.min(15) as usize;

        // 逐层累计应得
        for (index, referrer) in referral_chain.iter().enumerate().take(levels_to_process) {
            // 获取该层分成比例
            let percent = if let Some(p) = level_percents.get(index) {
                *p
            } else {
                0
            };

            if percent == 0 {
                continue;
            }

            // 简化验证：仅检查活跃期
            let active_until = ActiveUntilWeek::<T>::get(referrer);
            if active_until < current_cycle {
                // 未激活，跳过该层
                continue;
            }

            // 验证：是否为有效会员
            if !T::MembershipProvider::is_valid_member(referrer) {
                continue;
            }

            // 计算分成金额
            let share = if percent == 0 || percent > 100 {
                BalanceOf::<T>::zero()
            } else {
                let percent_balance: BalanceOf<T> = percent.into();
                let hundred: BalanceOf<T> = 100u32.into();
                distributable_amount.saturating_mul(percent_balance) / hundred
            };

            if share.is_zero() {
                continue;
            }

            // 累计应得金额
            Entitlement::<T>::mutate(current_cycle, referrer, |balance| {
                *balance = balance.saturating_add(share);
            });

            // 如果是第一层，且有时长，更新直推活跃数
            if index == 0 {
                if let Some(weeks) = duration_weeks {
                    if weeks > 0 {
                        // 更新购买者的活跃期
                        let new_active_until = current_cycle.saturating_add(weeks);
                        let old_active_until = ActiveUntilWeek::<T>::get(buyer);
                        
                        if new_active_until > old_active_until {
                            ActiveUntilWeek::<T>::insert(buyer, new_active_until);
                            
                            // 如果是新激活，增加推荐人的直推计数
                            if old_active_until < current_cycle {
                                DirectActiveCount::<T>::mutate(referrer, |count| {
                                    *count = count.saturating_add(1);
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// 函数级中文注释：结算指定周期
    ///
    /// 参数：
    /// - cycle: 周编号
    /// - max_accounts: 本次最多结算的账户数（分页）
    ///
    /// 返回：实际结算的账户数
    pub fn do_settle_cycle(
        cycle: u32,
        max_accounts: u32,
    ) -> Result<u32, DispatchError> {
        // 获取当前结算游标
        let cursor = SettleCursor::<T>::get(cycle);

        // 获取待结算账户列表（简化实现：迭代 Entitlement）
        let mut settled_count = 0u32;
        let mut current_index = cursor;

        // 批量转账列表
        let mut transfers = sp_std::vec::Vec::new();

        // 迭代 Entitlement 存储
        for (account, amount) in Entitlement::<T>::iter_prefix(cycle) {
            // 跳过已处理的账户
            if current_index > 0 {
                current_index = current_index.saturating_sub(1);
                continue;
            }

            if amount.is_zero() {
                continue;
            }

            // 添加到转账列表
            transfers.push((account.clone(), amount));

            settled_count += 1;

            // 达到批量上限
            if settled_count >= max_accounts {
                break;
            }
        }

        // 批量转账
        if !transfers.is_empty() {
            Self::do_batch_withdraw(&transfers)
                .map_err(|_| Error::<T>::WithdrawFailed)?;

            // 清理已结算账户
            for (account, _) in &transfers {
                Entitlement::<T>::remove(cycle, account);
            }

            // 更新累计统计
            let total_amount: BalanceOf<T> = transfers.iter()
                .fold(BalanceOf::<T>::zero(), |acc, (_, amount)| acc.saturating_add(*amount));

            TotalWeeklyDistributed::<T>::mutate(|total| {
                *total = total.saturating_add(total_amount);
            });

            // 发射事件
            Self::deposit_event(Event::CycleSettled {
                cycle,
                settled_count,
                total_amount,
            });
        }

        // 更新结算游标
        let new_cursor = cursor.saturating_add(settled_count);
        SettleCursor::<T>::insert(cycle, new_cursor);

        // 检查是否结算完成
        if settled_count < max_accounts {
            // 结算完成，清理游标
            SettleCursor::<T>::remove(cycle);
            CurrentSettlingCycle::<T>::kill();
        } else {
            // 未完成，记录当前结算周期
            CurrentSettlingCycle::<T>::put(Some(cycle));
        }

        Ok(settled_count)
    }

}

