//! # TEE 隐私计算模块 - Benchmarking
//!
//! 本模块提供所有 extrinsics 的性能基准测试。

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_std::vec;

use crate::types::{
    BatchRequestItem, BatchResultItem, TeeAttestation, TeeNodeStatus, TeeType,
};

// 辅助函数：创建测试账户
fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let account: T::AccountId = account(name, index, 0);
    let amount = T::Currency::minimum_balance().saturating_mul(10_000u32.into());
    let _ = T::Currency::make_free_balance_be(&account, amount);
    account
}

// 辅助函数：创建测试公钥
fn test_pubkey(seed: u8) -> [u8; 32] {
    let mut pubkey = [0u8; 32];
    pubkey[0] = seed;
    pubkey[31] = 0x01; // 确保非零
    pubkey
}

// 辅助函数：创建测试认证报告
fn test_attestation(seed: u8) -> TeeAttestation {
    let mut mr_enclave = [0u8; 32];
    mr_enclave[0] = seed;
    mr_enclave[31] = 0x01;

    let mut mr_signer = [0u8; 32];
    mr_signer[0] = seed;
    mr_signer[31] = 0x02;

    TeeAttestation {
        tee_type: TeeType::IntelSgx,
        mr_enclave,
        mr_signer,
        isv_prod_id: 1,
        isv_svn: 1,
        report_data: [0u8; 64],
        ias_signature: BoundedVec::try_from(vec![0u8; 64]).unwrap(),
        timestamp: 1_700_000_000,
    }
}

// 辅助函数：注册一个测试节点
fn register_test_node<T: Config>(seed: u8) -> T::AccountId {
    let account = funded_account::<T>("node", seed as u32);
    let pubkey = test_pubkey(seed);
    let attestation = test_attestation(seed);

    // 添加允许的 MR_ENCLAVE
    AllowedMrEnclaves::<T>::try_mutate(|enclaves| {
        if !enclaves.contains(&attestation.mr_enclave) {
            enclaves.try_push(attestation.mr_enclave)
        } else {
            Ok(())
        }
    })
    .ok();

    // 注册节点
    Pallet::<T>::register_tee_node(
        RawOrigin::Signed(account.clone()).into(),
        pubkey,
        attestation,
    )
    .expect("Failed to register test node");

    // 激活节点
    TeeNodes::<T>::mutate(&account, |node| {
        if let Some(ref mut n) = node {
            n.status = TeeNodeStatus::Active;
        }
    });

    // 添加到活跃节点列表
    ActiveNodes::<T>::try_mutate(|nodes| nodes.try_push(account.clone())).ok();

    account
}

// 辅助函数：创建测试请求
fn create_test_request<T: Config>(requester: &T::AccountId) -> u64 {
    let input_hash = [0u8; 32];
    let compute_type_id = 0u8;

    Pallet::<T>::submit_compute_request(
        RawOrigin::Signed(requester.clone()).into(),
        compute_type_id,
        input_hash,
        None, // 自动分配节点
    )
    .expect("Failed to create test request");

    NextRequestId::<T>::get() - 1
}

#[benchmarks]
mod benchmarks {
    use super::*;

    // ==================== TEE 节点管理 ====================

    #[benchmark]
    fn register_tee_node() {
        let caller = funded_account::<T>("caller", 0);
        let pubkey = test_pubkey(0);
        let attestation = test_attestation(0);

        // 添加允许的 MR_ENCLAVE
        AllowedMrEnclaves::<T>::try_mutate(|enclaves| {
            enclaves.try_push(attestation.mr_enclave)
        })
        .ok();

        #[extrinsic_call]
        register_tee_node(RawOrigin::Signed(caller.clone()), pubkey, attestation);

        assert!(TeeNodes::<T>::contains_key(&caller));
    }

    #[benchmark]
    fn update_attestation() {
        let node = register_test_node::<T>(0);
        let new_attestation = test_attestation(0);

        #[extrinsic_call]
        update_attestation(RawOrigin::Signed(node.clone()), new_attestation);

        assert!(TeeNodes::<T>::contains_key(&node));
    }

    #[benchmark]
    fn update_node_status() {
        let node = register_test_node::<T>(0);
        let new_status = TeeNodeStatus::Suspended;

        #[extrinsic_call]
        update_node_status(RawOrigin::Signed(node.clone()), new_status);

        let stored_node = TeeNodes::<T>::get(&node).expect("Node should exist");
        assert_eq!(stored_node.status, new_status);
    }

    #[benchmark]
    fn deregister_tee_node() {
        let node = register_test_node::<T>(0);

        #[extrinsic_call]
        deregister_tee_node(RawOrigin::Signed(node.clone()));

        let stored_node = TeeNodes::<T>::get(&node).expect("Node should exist");
        assert_eq!(stored_node.status, TeeNodeStatus::Deregistered);
    }

    // ==================== 计算请求管理 ====================

    #[benchmark]
    fn submit_compute_request() {
        let _ = register_test_node::<T>(0);
        let requester = funded_account::<T>("requester", 0);
        let input_hash = [0u8; 32];
        let compute_type_id = 0u8;
        let assigned_node: Option<T::AccountId> = None;

        #[extrinsic_call]
        submit_compute_request(
            RawOrigin::Signed(requester.clone()),
            compute_type_id,
            input_hash,
            assigned_node,
        );

        assert!(ComputeRequests::<T>::contains_key(0u64));
    }

    #[benchmark]
    fn submit_compute_result() {
        let node = register_test_node::<T>(0);
        let requester = funded_account::<T>("requester", 0);
        let request_id = create_test_request::<T>(&requester);

        let output_hash = [0u8; 32];
        let enclave_signature = [0u8; 64];

        #[extrinsic_call]
        submit_compute_result(
            RawOrigin::Signed(node.clone()),
            request_id,
            output_hash,
            enclave_signature,
        );

        let result = ComputeResults::<T>::get(request_id);
        assert!(result.is_some());
    }

    #[benchmark]
    fn cancel_compute_request() {
        let _ = register_test_node::<T>(0);
        let requester = funded_account::<T>("requester", 0);
        let request_id = create_test_request::<T>(&requester);

        #[extrinsic_call]
        cancel_compute_request(RawOrigin::Signed(requester.clone()), request_id);

        assert!(!ComputeRequests::<T>::contains_key(request_id));
    }

    // ==================== 批处理优化 ====================

    #[benchmark]
    fn submit_batch_compute_requests(n: Linear<1, 10>) {
        let _ = register_test_node::<T>(0);
        let requester = funded_account::<T>("requester", 0);

        let mut items = Vec::new();
        for i in 0..n {
            let mut input_hash = [0u8; 32];
            input_hash[0] = i as u8;
            items.push(BatchRequestItem {
                compute_type_id: 0,
                input_hash,
            });
        }

        #[extrinsic_call]
        submit_batch_compute_requests(RawOrigin::Signed(requester.clone()), items);

        assert_eq!(NextRequestId::<T>::get(), n as u64);
    }

    #[benchmark]
    fn submit_batch_compute_results(n: Linear<1, 10>) {
        let node = register_test_node::<T>(0);
        let requester = funded_account::<T>("requester", 0);

        // 创建 n 个请求
        let mut result_items = Vec::new();
        for i in 0..n {
            let request_id = create_test_request::<T>(&requester);
            let mut output_hash = [0u8; 32];
            output_hash[0] = i as u8;
            result_items.push(BatchResultItem {
                request_id,
                output_hash,
                enclave_signature: [0u8; 64],
            });
        }

        #[extrinsic_call]
        submit_batch_compute_results(RawOrigin::Signed(node.clone()), result_items);

        // 验证所有结果都已提交
        for i in 0..n {
            assert!(ComputeResults::<T>::contains_key(i as u64));
        }
    }

    // ==================== 经济激励 ====================

    #[benchmark]
    fn stake() {
        let node = register_test_node::<T>(0);
        let amount = T::MinimumStake::get();

        #[extrinsic_call]
        stake(RawOrigin::Signed(node.clone()), amount);

        assert!(NodeStakes::<T>::contains_key(&node));
    }

    #[benchmark]
    fn request_unstake() {
        let node = register_test_node::<T>(0);
        let amount = T::MinimumStake::get();

        // 先质押
        Pallet::<T>::stake(RawOrigin::Signed(node.clone()).into(), amount)
            .expect("Stake should succeed");

        #[extrinsic_call]
        request_unstake(RawOrigin::Signed(node.clone()));

        let stake_info = NodeStakes::<T>::get(&node).expect("Stake should exist");
        assert!(stake_info.is_unbonding);
    }

    #[benchmark]
    fn withdraw_unstaked() {
        let node = register_test_node::<T>(0);
        let amount = T::MinimumStake::get();

        // 先质押
        Pallet::<T>::stake(RawOrigin::Signed(node.clone()).into(), amount)
            .expect("Stake should succeed");

        // 申请解除质押
        Pallet::<T>::request_unstake(RawOrigin::Signed(node.clone()).into())
            .expect("Request unstake should succeed");

        // 模拟时间推进到解锁区块
        let current_block = frame_system::Pallet::<T>::block_number();
        let unlock_block = current_block + T::RequestTimeout::get().into();
        frame_system::Pallet::<T>::set_block_number(unlock_block + 1u32.into());

        #[extrinsic_call]
        withdraw_unstaked(RawOrigin::Signed(node.clone()));

        assert!(!NodeStakes::<T>::contains_key(&node));
    }

    // ==================== 超时处理 ====================

    #[benchmark]
    fn process_timeouts(n: Linear<1, 10>) {
        let _ = register_test_node::<T>(0);
        let requester = funded_account::<T>("requester", 0);

        // 创建 n 个请求
        for _ in 0..n {
            create_test_request::<T>(&requester);
        }

        // 推进区块到超时
        let timeout_block = frame_system::Pallet::<T>::block_number()
            + T::RequestTimeout::get().into()
            + 1u32.into();
        frame_system::Pallet::<T>::set_block_number(timeout_block);

        #[block]
        {
            Pallet::<T>::process_timeout_requests(timeout_block);
        }
    }

    // ==================== 管理功能 ====================

    #[benchmark]
    fn add_allowed_mr_enclave() {
        let mr_enclave = [1u8; 32];

        #[extrinsic_call]
        add_allowed_mr_enclave(RawOrigin::Root, mr_enclave);

        let enclaves = AllowedMrEnclaves::<T>::get();
        assert!(enclaves.contains(&mr_enclave));
    }

    #[benchmark]
    fn remove_allowed_mr_enclave() {
        let mr_enclave = [1u8; 32];

        // 先添加
        AllowedMrEnclaves::<T>::try_mutate(|enclaves| enclaves.try_push(mr_enclave)).ok();

        #[extrinsic_call]
        remove_allowed_mr_enclave(RawOrigin::Root, mr_enclave);

        let enclaves = AllowedMrEnclaves::<T>::get();
        assert!(!enclaves.contains(&mr_enclave));
    }

    #[benchmark]
    fn add_allowed_mr_signer() {
        let mr_signer = [1u8; 32];

        #[extrinsic_call]
        add_allowed_mr_signer(RawOrigin::Root, mr_signer);

        let signers = AllowedMrSigners::<T>::get();
        assert!(signers.contains(&mr_signer));
    }

    #[benchmark]
    fn remove_allowed_mr_signer() {
        let mr_signer = [1u8; 32];

        // 先添加
        AllowedMrSigners::<T>::try_mutate(|signers| signers.try_push(mr_signer)).ok();

        #[extrinsic_call]
        remove_allowed_mr_signer(RawOrigin::Root, mr_signer);

        let signers = AllowedMrSigners::<T>::get();
        assert!(!signers.contains(&mr_signer));
    }

    #[benchmark]
    fn set_audit_enabled() {
        let enabled = true;

        #[extrinsic_call]
        set_audit_enabled(RawOrigin::Root, enabled);

        assert_eq!(AuditEnabled::<T>::get(), enabled);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
