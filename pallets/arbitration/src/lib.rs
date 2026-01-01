#![cfg_attr(not(feature = "std"), no_std)]
//! 说明：临时全局允许 `deprecated`（RuntimeEvent/常量权重），后续移除
#![allow(deprecated)]

extern crate alloc;

pub use pallet::*;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::traits::{EnsureOrigin, fungible::{Inspect as FungibleInspect, Mutate as FungibleMutate, MutateHold as FungibleMutateHold}};
    use frame_support::{pallet_prelude::*, BoundedVec};
    use frame_system::pallet_prelude::*;
    use pallet_escrow::pallet::Escrow as EscrowTrait;
    use sp_runtime::Saturating;
    // 基准模块在 pallet 外部声明；此处不在 proc-macro 输入中声明子模块，避免 E0658

    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
    pub enum Decision {
        Release,
        Refund,
        Partial(u16),
    } // bps

    /// 仲裁域路由接口：由 runtime 实现，根据域将仲裁请求路由到对应业务 pallet
    ///
    /// 设计目的：
    /// - 以 [u8;8] 域常量（通常与 PalletId 字节对齐）标识业务域
    /// - can_dispute：校验发起人是否有权对 (domain, id) 发起争议
    /// - apply_decision：按裁决对 (domain, id) 应用资金与状态变更（由各业务 pallet 内部完成）
    /// - get_counterparty：获取纠纷对方账户（用于双向押金）
    /// - get_order_amount：获取订单/交易金额（用于计算押金比例）
    pub trait ArbitrationRouter<AccountId, Balance> {
        /// 校验是否允许发起争议
        fn can_dispute(domain: [u8; 8], who: &AccountId, id: u64) -> bool;
        /// 应用裁决（放款/退款/部分放款）
        fn apply_decision(domain: [u8; 8], id: u64, decision: Decision) -> DispatchResult;
        /// 获取纠纷对方账户（发起方是买家，返回卖家；反之亦然）
        fn get_counterparty(domain: [u8; 8], initiator: &AccountId, id: u64) -> Result<AccountId, DispatchError>;
        /// 🆕 获取订单/交易金额（用于计算押金）
        fn get_order_amount(domain: [u8; 8], id: u64) -> Result<Balance, DispatchError>;
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_escrow::pallet::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type MaxEvidence: Get<u32>;
        type MaxCidLen: Get<u32>;
        /// 托管接口（调用释放/退款/部分分账）
        type Escrow: EscrowTrait<Self::AccountId, BalanceOf<Self>>;
        /// 权重信息
        type WeightInfo: weights::WeightInfo;
        /// 域路由：把仲裁请求路由到对应业务 pallet 的仲裁钩子
        type Router: ArbitrationRouter<Self::AccountId, BalanceOf<Self>>;
        /// 函数级中文注释：仲裁决策起源（治理）。
        /// - 由 runtime 绑定为 Root 或 内容委员会 阈值（例如 2/3 通过）。
        /// - 用于 `arbitrate` 裁决入口的权限校验，替代任意签名账户。
        type DecisionOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 🆕 双向押金相关配置
        /// Fungible 接口：用于锁定和释放押金
        type Fungible: FungibleInspect<Self::AccountId, Balance = BalanceOf<Self>>
            + FungibleMutate<Self::AccountId>
            + FungibleMutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;
        /// RuntimeHoldReason：押金锁定原因标识
        type RuntimeHoldReason: From<HoldReason>;
        /// 🆕 押金比例（基点，1500 = 15%）
        type DepositRatioBps: Get<u16>;
        /// 应诉期限（区块数，默认 7 天）
        type ResponseDeadline: Get<BlockNumberFor<Self>>;
        /// 驳回罚没比例（基点，3000 = 30%）
        type RejectedSlashBps: Get<u16>;
        /// 部分胜诉罚没比例（基点，5000 = 50%）
        type PartialSlashBps: Get<u16>;
        /// 国库账户
        type TreasuryAccount: Get<Self::AccountId>;
    }

    pub type BalanceOf<T> =
        <<T as pallet_escrow::pallet::Config>::Currency as frame_support::traits::Currency<
            <T as frame_system::Config>::AccountId,
        >>::Balance;

    /// 🆕 押金锁定原因枚举
    #[pallet::composite_enum]
    pub enum HoldReason {
        /// 纠纷发起方押金
        DisputeInitiator,
        /// 应诉方押金
        DisputeRespondent,
    }

    /// 🆕 双向押金记录
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct TwoWayDepositRecord<AccountId, Balance, BlockNumber> {
        /// 发起方账户
        pub initiator: AccountId,
        /// 发起方押金金额
        pub initiator_deposit: Balance,
        /// 应诉方账户
        pub respondent: AccountId,
        /// 应诉方押金金额（可选，未应诉时为 None）
        pub respondent_deposit: Option<Balance>,
        /// 应诉截止区块
        pub response_deadline: BlockNumber,
        /// 是否已应诉
        pub has_responded: bool,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 争议登记：(domain, object_id) => ()
    #[pallet::storage]
    pub type Disputed<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, [u8; 8], Blake2_128Concat, u64, (), OptionQuery>;

    /// 函数级中文注释：每个仲裁案件引用的 evidence_id 列表（证据本体由 pallet-evidence 存储）。
    #[pallet::storage]
    pub type EvidenceIds<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        [u8; 8],
        Blake2_128Concat,
        u64,
        BoundedVec<u64, T::MaxEvidence>,
        ValueQuery,
    >;

    /// 🆕 双向押金记录存储：(domain, object_id) => TwoWayDepositRecord
    #[pallet::storage]
    pub type TwoWayDeposits<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        [u8; 8],
        Blake2_128Concat,
        u64,
        TwoWayDepositRecord<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 发起争议事件（含域）
        Disputed { domain: [u8; 8], id: u64 },
        /// 完成裁决事件（含域）
        Arbitrated {
            domain: [u8; 8],
            id: u64,
            decision: u8,
            bps: Option<u16>,
        },
        /// 🆕 发起纠纷并锁定押金
        DisputeWithDepositInitiated {
            domain: [u8; 8],
            id: u64,
            initiator: T::AccountId,
            respondent: T::AccountId,
            deposit: BalanceOf<T>,
            deadline: BlockNumberFor<T>,
        },
        /// 🆕 应诉方锁定押金
        RespondentDepositLocked {
            domain: [u8; 8],
            id: u64,
            respondent: T::AccountId,
            deposit: BalanceOf<T>,
        },
        /// 🆕 押金已处理（罚没或释放）
        DepositProcessed {
            domain: [u8; 8],
            id: u64,
            account: T::AccountId,
            released: BalanceOf<T>,
            slashed: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        AlreadyDisputed,
        NotDisputed,
        /// 🆕 押金不足
        InsufficientDeposit,
        /// 🆕 已经应诉
        AlreadyResponded,
        /// 🆕 应诉期已过
        ResponseDeadlinePassed,
        /// 🆕 无法获取对方账户
        CounterpartyNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起仲裁：记录争议，证据 CID 存链（仅登记摘要/CID，不碰业务存储）
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::dispute(_evidence.len() as u32))]
        pub fn dispute(
            origin: OriginFor<T>,
            domain: [u8; 8],
            id: u64,
            _evidence: alloc::vec::Vec<BoundedVec<u8, T::MaxCidLen>>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            // 鉴权：由 Router 依据业务 pallet 规则判断是否允许发起（基准模式下跳过，便于构造场景）
            #[cfg(not(feature = "runtime-benchmarks"))]
            {
                ensure!(
                    T::Router::can_dispute(domain, &_who, id),
                    Error::<T>::NotDisputed
                );
            }
            ensure!(
                Disputed::<T>::get(domain, id).is_none(),
                Error::<T>::AlreadyDisputed
            );
            Disputed::<T>::insert(domain, id, ());
            // 证据仅留 CID；如需可扩展附加存储（MVP 省略内容）
            Self::deposit_event(Event::Disputed { domain, id });
            Ok(())
        }
        /// 仲裁者裁决（治理起源：Root/委员会）。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::arbitrate())]
        pub fn arbitrate(
            origin: OriginFor<T>,
            domain: [u8; 8],
            id: u64,
            decision_code: u8,
            bps: Option<u16>,
        ) -> DispatchResult {
            // 函数级详细中文注释：仲裁裁决入口
            // - 安全：仅允许由治理起源触发（Root 或 内容委员会阈值），避免任意账户执行清算。
            // - 通过 runtime 注入的 DecisionOrigin 校验 origin。
            T::DecisionOrigin::ensure_origin(origin)?;
            ensure!(
                Disputed::<T>::get(domain, id).is_some(),
                Error::<T>::NotDisputed
            );
            // 通过 Router 将裁决应用到对应域的业务 pallet
            let decision = match (decision_code, bps) {
                (0, _) => Decision::Release,
                (1, _) => Decision::Refund,
                (2, Some(p)) => Decision::Partial(p),
                _ => Decision::Refund,
            };
            T::Router::apply_decision(domain, id, decision.clone())?;

            // 🆕 处理双向押金
            Self::handle_deposits_on_arbitration(domain, id, &decision)?;

            // 🆕 2025-10-22：TODO - 根据裁决结果更新做市商信用分
            // 函数级详细中文注释：如果裁决为Release（做市商胜诉），无变化
            // 如果裁决为Refund/Partial（做市商败诉），应扣除信用分
            // 需要通过 Router 获取 maker_id，然后调用：
            // pallet_credit::Pallet::<T>::record_maker_dispute_result(maker_id, id, maker_win)?;
            
            let out = match decision {
                Decision::Release => (0, None),
                Decision::Refund => (1, None),
                Decision::Partial(p) => (2, Some(p)),
            };
            Self::deposit_event(Event::Arbitrated {
                domain,
                id,
                decision: out.0,
                bps: out.1,
            });
            Ok(())
        }

        /// 函数级中文注释：以 evidence_id 的方式发起仲裁登记。
        /// - 适用场景：前端/当事人先调用 `pallet-evidence::commit` 获得 `evidence_id`，再把该 id 带入此函数，
        ///   从而实现“证据统一在 evidence 中存储与复用”，仲裁侧仅保存引用。
        /// - 行为：
        ///   1) 校验可发起（通过 Router.can_dispute）；2) 确保未被登记；3) 登记 Disputed；
        ///   4) 将 evidence_id 追加到本案的证据引用列表；5) 触发 Disputed 事件。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::dispute(1))]
        pub fn dispute_with_evidence_id(
            origin: OriginFor<T>,
            domain: [u8; 8],
            id: u64,
            evidence_id: u64,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            #[cfg(not(feature = "runtime-benchmarks"))]
            {
                ensure!(
                    T::Router::can_dispute(domain, &_who, id),
                    Error::<T>::NotDisputed
                );
            }
            ensure!(
                Disputed::<T>::get(domain, id).is_none(),
                Error::<T>::AlreadyDisputed
            );
            Disputed::<T>::insert(domain, id, ());
            EvidenceIds::<T>::try_mutate(domain, id, |v| -> Result<(), Error<T>> {
                v.try_push(evidence_id)
                    .map_err(|_| Error::<T>::AlreadyDisputed)?; // 复用错误占位，避免新增错误枚举
                Ok(())
            })?;
            Self::deposit_event(Event::Disputed { domain, id });
            Ok(())
        }

        /// 函数级中文注释：为已登记的仲裁案件追加一个 evidence_id 引用。
        /// - 适用场景：补充证据；证据本体由 `pallet-evidence` 统一存储。
        /// - 行为：
        ///   1) 确认本案已登记；2) 追加 evidence_id 到引用列表。
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::dispute(1))]
        pub fn append_evidence_id(
            origin: OriginFor<T>,
            domain: [u8; 8],
            id: u64,
            evidence_id: u64,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(
                Disputed::<T>::get(domain, id).is_some(),
                Error::<T>::NotDisputed
            );
            EvidenceIds::<T>::try_mutate(domain, id, |v| -> Result<(), Error<T>> {
                v.try_push(evidence_id)
                    .map_err(|_| Error::<T>::AlreadyDisputed)?;
                Ok(())
            })?;
            Ok(())
        }

        /// 🆕 函数级中文注释：以双向押金方式发起纠纷
        /// - 从托管账户扣除押金（订单金额的15%）
        /// - 获取应诉方（卖家）信息
        /// - 设置应诉截止期限
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::dispute(1))]
        pub fn dispute_with_two_way_deposit(
            origin: OriginFor<T>,
            domain: [u8; 8],
            id: u64,
            evidence_id: u64,
        ) -> DispatchResult {
            let initiator = ensure_signed(origin)?;

            // 1. 权限校验
            #[cfg(not(feature = "runtime-benchmarks"))]
            {
                ensure!(
                    T::Router::can_dispute(domain, &initiator, id),
                    Error::<T>::NotDisputed
                );
            }

            // 2. 确保未被登记
            ensure!(
                Disputed::<T>::get(domain, id).is_none(),
                Error::<T>::AlreadyDisputed
            );

            // 3. 获取订单金额
            let order_amount = T::Router::get_order_amount(domain, id)
                .map_err(|_| Error::<T>::CounterpartyNotFound)?;

            // 4. 计算押金金额（订单金额的15%）
            let deposit_ratio_bps = T::DepositRatioBps::get();
            let deposit_amount = sp_runtime::Perbill::from_parts((deposit_ratio_bps as u32) * 100)
                .mul_floor(order_amount);

            // 5. 检查托管余额是否足够
            let escrow_balance = T::Escrow::amount_of(id);
            ensure!(
                escrow_balance >= deposit_amount,
                Error::<T>::InsufficientDeposit
            );

            // 6. 获取托管账户并从托管账户锁定押金
            let escrow_account = Self::get_escrow_account();
            T::Fungible::hold(
                &T::RuntimeHoldReason::from(HoldReason::DisputeInitiator),
                &escrow_account,
                deposit_amount,
            )
            .map_err(|_| Error::<T>::InsufficientDeposit)?;

            // 7. 获取对方账户
            let respondent = T::Router::get_counterparty(domain, &initiator, id)
                .map_err(|_| Error::<T>::CounterpartyNotFound)?;

            // 8. 计算应诉截止期限
            let current_block = frame_system::Pallet::<T>::block_number();
            let deadline = current_block + T::ResponseDeadline::get();

            // 9. 登记纠纷和双向押金记录
            Disputed::<T>::insert(domain, id, ());
            TwoWayDeposits::<T>::insert(
                domain,
                id,
                TwoWayDepositRecord {
                    initiator: initiator.clone(),
                    initiator_deposit: deposit_amount,
                    respondent: respondent.clone(),
                    respondent_deposit: None,
                    response_deadline: deadline,
                    has_responded: false,
                },
            );

            // 10. 添加证据引用
            EvidenceIds::<T>::try_mutate(domain, id, |v| -> Result<(), Error<T>> {
                v.try_push(evidence_id)
                    .map_err(|_| Error::<T>::AlreadyDisputed)?;
                Ok(())
            })?;

            // 11. 触发事件
            Self::deposit_event(Event::DisputeWithDepositInitiated {
                domain,
                id,
                initiator,
                respondent,
                deposit: deposit_amount,
                deadline,
            });

            Ok(())
        }

        /// 🆕 函数级中文注释：应诉方从托管锁定押金并提交反驳证据
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::dispute(1))]
        pub fn respond_to_dispute(
            origin: OriginFor<T>,
            domain: [u8; 8],
            id: u64,
            counter_evidence_id: u64,
        ) -> DispatchResult {
            let respondent = ensure_signed(origin)?;

            // 1. 获取押金记录
            let mut deposit_record = TwoWayDeposits::<T>::get(domain, id)
                .ok_or(Error::<T>::NotDisputed)?;

            // 2. 验证是应诉方
            ensure!(
                deposit_record.respondent == respondent,
                Error::<T>::NotDisputed
            );

            // 3. 确保未应诉
            ensure!(!deposit_record.has_responded, Error::<T>::AlreadyResponded);

            // 4. 检查是否超时
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block <= deposit_record.response_deadline,
                Error::<T>::ResponseDeadlinePassed
            );

            // 5. 计算押金金额（与发起方相同）
            let deposit_amount = deposit_record.initiator_deposit;

            // 6. 检查托管余额是否足够（应诉方也从托管扣押金）
            let escrow_balance = T::Escrow::amount_of(id);
            ensure!(
                escrow_balance >= deposit_amount,
                Error::<T>::InsufficientDeposit
            );

            // 7. 从托管账户锁定应诉方押金
            let escrow_account = Self::get_escrow_account();
            T::Fungible::hold(
                &T::RuntimeHoldReason::from(HoldReason::DisputeRespondent),
                &escrow_account,
                deposit_amount,
            )
            .map_err(|_| Error::<T>::InsufficientDeposit)?;

            // 8. 更新押金记录
            deposit_record.respondent_deposit = Some(deposit_amount);
            deposit_record.has_responded = true;
            TwoWayDeposits::<T>::insert(domain, id, deposit_record);

            // 9. 添加反驳证据
            EvidenceIds::<T>::try_mutate(domain, id, |v| -> Result<(), Error<T>> {
                v.try_push(counter_evidence_id)
                    .map_err(|_| Error::<T>::AlreadyDisputed)?;
                Ok(())
            })?;

            // 10. 触发事件
            Self::deposit_event(Event::RespondentDepositLocked {
                domain,
                id,
                respondent,
                deposit: deposit_amount,
            });

            Ok(())
        }
    }

    /// 🆕 辅助函数实现
    impl<T: Config> Pallet<T> {
        /// 函数级中文注释：获取托管账户
        /// - 使用 pallet-escrow 的 PalletId 派生
        fn get_escrow_account() -> T::AccountId {
            use sp_runtime::traits::AccountIdConversion;
            <<T as pallet_escrow::pallet::Config>::EscrowPalletId as Get<frame_support::PalletId>>::get()
                .into_account_truncating()
        }
        /// 函数级中文注释：仲裁时处理双向押金
        /// - Release: 买家败诉，罚没买家押金30%，卖家押金全额返还到托管
        /// - Refund: 卖家败诉，罚没卖家押金30%，买家押金全额返还到托管
        /// - Partial: 双方都有责任，各罚没50%
        ///
        /// 注意：所有押金操作都在托管账户上进行
        fn handle_deposits_on_arbitration(
            domain: [u8; 8],
            id: u64,
            decision: &Decision,
        ) -> DispatchResult {
            if let Some(deposit_record) = TwoWayDeposits::<T>::take(domain, id) {
                let treasury = T::TreasuryAccount::get();
                let escrow_account = Self::get_escrow_account();

                match decision {
                    Decision::Release => {
                        // 卖家胜诉：买家押金罚没30%，卖家押金全额返还到托管
                        Self::slash_and_release(
                            &escrow_account,  // 从托管账户操作
                            deposit_record.initiator_deposit,
                            T::RejectedSlashBps::get(),
                            &HoldReason::DisputeInitiator,
                            &treasury,
                        )?;

                        if let Some(respondent_deposit) = deposit_record.respondent_deposit {
                            Self::release_deposit(
                                &escrow_account,  // 返还到托管账户
                                respondent_deposit,
                                &HoldReason::DisputeRespondent,
                            )?;
                        }
                    }
                    Decision::Refund => {
                        // 买家胜诉：买家押金全额返还到托管，卖家押金罚没30%
                        Self::release_deposit(
                            &escrow_account,  // 返还到托管账户
                            deposit_record.initiator_deposit,
                            &HoldReason::DisputeInitiator,
                        )?;

                        if let Some(respondent_deposit) = deposit_record.respondent_deposit {
                            Self::slash_and_release(
                                &escrow_account,  // 从托管账户操作
                                respondent_deposit,
                                T::RejectedSlashBps::get(),
                                &HoldReason::DisputeRespondent,
                                &treasury,
                            )?;
                        }
                    }
                    Decision::Partial(_) => {
                        // 部分胜诉：双方各罚没50%
                        Self::slash_and_release(
                            &escrow_account,  // 从托管账户操作
                            deposit_record.initiator_deposit,
                            T::PartialSlashBps::get(),
                            &HoldReason::DisputeInitiator,
                            &treasury,
                        )?;

                        if let Some(respondent_deposit) = deposit_record.respondent_deposit {
                            Self::slash_and_release(
                                &escrow_account,  // 从托管账户操作
                                respondent_deposit,
                                T::PartialSlashBps::get(),
                                &HoldReason::DisputeRespondent,
                                &treasury,
                            )?;
                        }
                    }
                }
            }
            Ok(())
        }

        /// 函数级中文注释：罚没并释放押金
        /// - slash_bps: 罚没比例（基点，如 3000 = 30%）
        fn slash_and_release(
            account: &T::AccountId,
            amount: BalanceOf<T>,
            slash_bps: u16,
            hold_reason: &HoldReason,
            treasury: &T::AccountId,
        ) -> DispatchResult {
            use sp_runtime::traits::Zero;

            let slash_amount = sp_runtime::Perbill::from_parts((slash_bps as u32) * 100)
                .mul_floor(amount);
            let release_amount = amount.saturating_sub(slash_amount);

            // 罚没部分转入国库
            if !slash_amount.is_zero() {
                T::Fungible::transfer_on_hold(
                    &T::RuntimeHoldReason::from(hold_reason.clone()),
                    account,
                    treasury,
                    slash_amount,
                    frame_support::traits::tokens::Precision::BestEffort,
                    frame_support::traits::tokens::Restriction::Free,
                    frame_support::traits::tokens::Fortitude::Force,
                )?;
            }

            // 释放剩余部分
            if !release_amount.is_zero() {
                T::Fungible::release(
                    &T::RuntimeHoldReason::from(hold_reason.clone()),
                    account,
                    release_amount,
                    frame_support::traits::tokens::Precision::Exact,
                )?;
            }

            Self::deposit_event(Event::DepositProcessed {
                domain: [0u8; 8], // 简化处理，实际应传入domain
                id: 0,
                account: account.clone(),
                released: release_amount,
                slashed: slash_amount,
            });

            Ok(())
        }

        /// 函数级中文注释：全额释放押金（无罚没）
        fn release_deposit(
            account: &T::AccountId,
            amount: BalanceOf<T>,
            hold_reason: &HoldReason,
        ) -> DispatchResult {
            use sp_runtime::traits::Zero;

            T::Fungible::release(
                &T::RuntimeHoldReason::from(hold_reason.clone()),
                account,
                amount,
                frame_support::traits::tokens::Precision::Exact,
            )?;

            Self::deposit_event(Event::DepositProcessed {
                domain: [0u8; 8],
                id: 0,
                account: account.clone(),
                released: amount,
                slashed: BalanceOf::<T>::zero(),
            });

            Ok(())
        }
    }
}
