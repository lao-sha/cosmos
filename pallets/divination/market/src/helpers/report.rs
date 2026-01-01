//! # 举报系统辅助函数
//!
//! 本模块包含举报处理相关的内部函数
//!
//! ## 主要功能
//! - 举报成立处理（扣除大师押金、奖励举报者）
//! - 举报驳回处理（退还举报押金）
//! - 恶意举报处理（没收押金、扣信用分）
//! - 辅助计算函数

use crate::pallet::*;
use crate::types::*;
use frame_support::{
    ensure,
    pallet_prelude::DispatchResult,
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{Saturating, Zero},
    SaturatedConversion,
};

impl<T: Config> Pallet<T> {
    // ==================== 举报成立处理 ====================

    /// 处理举报成立
    ///
    /// 执行流程：
    /// 1. 计算大师罚金
    /// 2. 计算举报者奖励
    /// 3. 扣除大师押金
    /// 4. 发放奖励给举报者
    /// 5. 剩余转入国库
    /// 6. 扣除大师信用分
    /// 7. 判断是否永久封禁
    /// 8. 更新举报档案
    ///
    /// # 参数
    /// - `report_id`: 举报 ID
    /// - `report`: 举报记录引用
    /// - `custom_penalty_rate`: 自定义惩罚比例（可选，覆盖默认值）
    ///
    /// # 返回
    /// - `Ok(())` 处理成功
    /// - `Err(DispatchError)` 处理失败
    pub(crate) fn handle_upheld_report(
        report_id: u64,
        report: &ReportOf<T>,
        custom_penalty_rate: Option<u16>,
    ) -> DispatchResult {
        let provider = &report.provider;
        let reporter = &report.reporter;
        let report_type = report.report_type;

        // 获取大师信息
        let provider_info =
            Providers::<T>::get(provider).ok_or(Error::<T>::ProviderNotFound)?;
        let provider_deposit = provider_info.deposit;

        // 1. 计算惩罚金额（从大师押金中扣除）
        let penalty_rate = custom_penalty_rate.unwrap_or(report_type.provider_penalty_rate());
        let penalty_amount = Self::calculate_penalty(provider_deposit, penalty_rate);

        // 2. 计算举报者奖励（惩罚金额的一部分）
        let reward_rate = report_type.reporter_reward_rate();
        let reporter_reward = Self::calculate_penalty(penalty_amount, reward_rate);

        // 3. 计算国库收入（惩罚金额剩余部分）
        let treasury_income = penalty_amount.saturating_sub(reporter_reward);

        // 4. 从大师预留押金中扣除（unreserve）
        let _actual_unreserved = T::Currency::unreserve(provider, penalty_amount);

        // 5. 发放奖励给举报者（包括退还举报押金）
        let total_to_reporter = reporter_reward.saturating_add(report.reporter_deposit);
        T::Currency::transfer(
            &Self::platform_account(),
            reporter,
            total_to_reporter,
            ExistenceRequirement::KeepAlive,
        )?;

        // 6. 剩余部分转入国库
        if !treasury_income.is_zero() {
            T::Currency::transfer(
                &Self::platform_account(),
                &T::TreasuryAccount::get(),
                treasury_income,
                ExistenceRequirement::KeepAlive,
            )?;
        }

        // 7. 扣除大师信用分
        let credit_deduction = report_type.credit_deduction();
        Self::deduct_credit_for_report(provider, credit_deduction);

        // 8. 判断是否永久封禁
        let is_banned = report_type.triggers_permanent_ban();
        if is_banned {
            Self::ban_provider(provider, report_type)?;
        }

        // 9. 更新举报记录
        Reports::<T>::mutate(report_id, |maybe_report| {
            if let Some(r) = maybe_report {
                r.provider_penalty = penalty_amount;
                r.reporter_reward = reporter_reward;
            }
        });

        // 10. 更新大师举报档案
        Self::update_provider_report_profile(provider, penalty_amount);

        // 11. 更新全局统计
        ReportStatistics::<T>::mutate(|stats| {
            stats.total_penalties = stats.total_penalties.saturating_add(penalty_amount);
            stats.total_rewards = stats.total_rewards.saturating_add(reporter_reward);
        });

        // 12. 发送事件
        Self::deposit_event(Event::ReportUpheld {
            report_id,
            provider: provider.clone(),
            penalty_amount,
            reporter_reward,
            is_banned,
        });

        Ok(())
    }

    // ==================== 举报驳回处理 ====================

    /// 处理举报驳回
    ///
    /// 全额退还举报者押金，不做其他处理
    ///
    /// # 参数
    /// - `report_id`: 举报 ID
    /// - `report`: 举报记录引用
    pub(crate) fn handle_rejected_report(
        report_id: u64,
        report: &ReportOf<T>,
    ) -> DispatchResult {
        // 全额退还举报押金
        T::Currency::transfer(
            &Self::platform_account(),
            &report.reporter,
            report.reporter_deposit,
            ExistenceRequirement::KeepAlive,
        )?;

        Self::deposit_event(Event::ReportRejected {
            report_id,
            reporter: report.reporter.clone(),
            deposit_refunded: report.reporter_deposit,
        });

        Ok(())
    }

    // ==================== 恶意举报处理 ====================

    /// 处理恶意举报
    ///
    /// 执行流程：
    /// 1. 没收举报押金转入国库
    /// 2. 扣除举报者信用分（如果有信用档案）
    ///
    /// # 参数
    /// - `report_id`: 举报 ID
    /// - `report`: 举报记录引用
    pub(crate) fn handle_malicious_report(
        report_id: u64,
        report: &ReportOf<T>,
    ) -> DispatchResult {
        // 1. 没收举报押金，转入国库
        T::Currency::transfer(
            &Self::platform_account(),
            &T::TreasuryAccount::get(),
            report.reporter_deposit,
            ExistenceRequirement::KeepAlive,
        )?;

        // 2. 扣除举报者信用分（如果有信用档案）
        let penalty = T::MaliciousReportPenalty::get();
        CreditProfiles::<T>::mutate(&report.reporter, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                profile.total_deductions = profile.total_deductions.saturating_add(penalty);
                Self::recalculate_credit_score(profile);
            }
        });

        // 3. 更新统计
        ReportStatistics::<T>::mutate(|stats| {
            stats.total_confiscated_deposits = stats
                .total_confiscated_deposits
                .saturating_add(report.reporter_deposit);
        });

        Self::deposit_event(Event::MaliciousReportPenalized {
            report_id,
            reporter: report.reporter.clone(),
            deposit_confiscated: report.reporter_deposit,
        });

        Ok(())
    }

    // ==================== 辅助函数 ====================

    /// 计算惩罚/奖励金额
    ///
    /// 根据基础金额和基点比例计算实际金额
    ///
    /// # 参数
    /// - `amount`: 基础金额
    /// - `rate_bps`: 比例（基点，10000 = 100%）
    ///
    /// # 返回
    /// 计算后的金额
    fn calculate_penalty(amount: BalanceOf<T>, rate_bps: u16) -> BalanceOf<T> {
        amount.saturating_mul(rate_bps.into()) / 10000u32.into()
    }

    /// 扣除信用分（因举报成立）
    ///
    /// 更新大师的信用档案，记录投诉相关数据
    ///
    /// # 参数
    /// - `provider`: 大师账户
    /// - `deduction`: 扣除分数
    fn deduct_credit_for_report(provider: &T::AccountId, deduction: u16) {
        CreditProfiles::<T>::mutate(provider, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                let current_block = <frame_system::Pallet<T>>::block_number();

                profile.total_deductions = profile.total_deductions.saturating_add(deduction);
                profile.complaint_count = profile.complaint_count.saturating_add(1);
                profile.complaint_upheld_count = profile.complaint_upheld_count.saturating_add(1);
                profile.last_deduction_reason = Some(DeductionReason::ComplaintUpheld);
                profile.last_deduction_at = Some(current_block);

                Self::recalculate_credit_score(profile);
            }
        });
    }

    /// 封禁大师
    ///
    /// 将大师状态设为 Banned 并加入黑名单
    ///
    /// # 参数
    /// - `provider`: 大师账户
    /// - `reason`: 封禁原因（举报类型）
    fn ban_provider(provider: &T::AccountId, reason: ReportType) -> DispatchResult {
        // 更新大师状态
        Providers::<T>::mutate(provider, |maybe_p| {
            if let Some(p) = maybe_p {
                p.status = ProviderStatus::Banned;
            }
        });

        // 加入信用黑名单
        let current_block = <frame_system::Pallet<T>>::block_number();
        CreditBlacklist::<T>::insert(provider, current_block);

        Self::deposit_event(Event::ProviderBanned {
            provider: provider.clone(),
            reason,
        });

        Ok(())
    }

    /// 更新大师举报档案
    ///
    /// 记录举报成立的相关数据，并判断是否需要进入观察期
    ///
    /// # 参数
    /// - `provider`: 大师账户
    /// - `penalty_amount`: 本次被扣除的押金金额
    fn update_provider_report_profile(provider: &T::AccountId, penalty_amount: BalanceOf<T>) {
        let current_block = <frame_system::Pallet<T>>::block_number();

        ProviderReportProfiles::<T>::mutate(provider, |profile| {
            profile.upheld_count = profile.upheld_count.saturating_add(1);
            profile.total_penalty_amount = profile
                .total_penalty_amount
                .saturating_add(penalty_amount.saturated_into());

            // 多次举报成立（>=3次），进入观察期
            if profile.upheld_count >= 3 && !profile.under_watch {
                profile.under_watch = true;
                // 观察期 30 天（假设 6 秒/区块）
                let watch_duration: BlockNumberFor<T> = 432000u32.into();
                let watch_end = current_block.saturating_add(watch_duration);
                profile.watch_period_end = Some(watch_end);

                Self::deposit_event(Event::ProviderUnderWatch {
                    provider: provider.clone(),
                    watch_end,
                });
            }
        });
    }

    /// 计算举报押金
    ///
    /// 根据举报类型计算需要缴纳的押金
    ///
    /// # 参数
    /// - `report_type`: 举报类型
    ///
    /// # 返回
    /// 需要缴纳的押金金额
    pub(crate) fn calculate_report_deposit(report_type: ReportType) -> BalanceOf<T> {
        let base_deposit = T::MinReportDeposit::get();
        let multiplier = report_type.deposit_multiplier();
        // multiplier 是百分比（100 = 1x），所以除以 100
        base_deposit.saturating_mul(multiplier.into()) / 100u32.into()
    }

    /// 验证举报冷却期
    ///
    /// 检查举报者是否可以对该大师发起举报
    ///
    /// # 参数
    /// - `reporter`: 举报者账户
    /// - `provider`: 被举报的大师账户
    ///
    /// # 返回
    /// - `Ok(())` 可以举报
    /// - `Err(ReportCooldownActive)` 在冷却期内
    pub(crate) fn check_report_cooldown(
        reporter: &T::AccountId,
        provider: &T::AccountId,
    ) -> DispatchResult {
        let current_block = <frame_system::Pallet<T>>::block_number();

        if let Some(last_report) = ReportCooldown::<T>::get(reporter, provider) {
            ensure!(
                current_block > last_report.saturating_add(T::ReportCooldownPeriod::get()),
                Error::<T>::ReportCooldownActive
            );
        }

        Ok(())
    }

    /// 从待处理队列移除举报
    ///
    /// # 参数
    /// - `report_id`: 要移除的举报 ID
    pub(crate) fn remove_from_pending(report_id: u64) {
        PendingReports::<T>::mutate(|list| {
            list.retain(|&id| id != report_id);
        });
    }

    /// 更新举报统计（处理完成时）
    ///
    /// 根据处理结果更新全局统计数据
    ///
    /// # 参数
    /// - `result`: 处理结果状态
    pub(crate) fn update_report_stats_on_resolve(result: ReportStatus) {
        ReportStatistics::<T>::mutate(|stats| {
            stats.pending_reports = stats.pending_reports.saturating_sub(1);
            match result {
                ReportStatus::Upheld => stats.upheld_reports += 1,
                ReportStatus::Rejected => stats.rejected_reports += 1,
                ReportStatus::Malicious => stats.malicious_reports += 1,
                _ => {}
            }
        });
    }

    /// 获取平台账户
    ///
    /// 返回平台收款账户地址
    pub(crate) fn platform_account() -> T::AccountId {
        T::PlatformAccount::get()
    }
}
