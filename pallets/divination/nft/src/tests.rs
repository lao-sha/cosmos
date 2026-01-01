//! Tests for pallet-divination-nft

use crate::{mock::*, Error, Event};
use crate::types::NftStatus;
use frame_support::{assert_noop, assert_ok};
use pallet_divination_common::{DivinationType, Rarity, RarityInput};

// ==================== 铸造测试 ====================

#[test]
fn mint_nft_works_for_meihua() {
    new_test_ext().execute_with(|| {
        // 设置模拟数据：Alice 创建了一个梅花卦象
        MockDivinationProvider::add_result(
            DivinationType::Meihua,
            1,
            1, // Alice
            RarityInput::common(),
        );

        // Alice 铸造 NFT
        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Test Hexagram NFT".to_vec(),
            b"QmTestImageCid123".to_vec(),
            None,
            None,
            500, // 5% royalty
        ));

        // 验证 NFT 创建
        let nft = DivinationNftPallet::nfts(0).expect("NFT should exist");
        assert_eq!(nft.divination_type, DivinationType::Meihua);
        assert_eq!(nft.result_id, 1);
        assert_eq!(nft.owner, 1);
        assert_eq!(nft.creator, 1);
        assert_eq!(nft.royalty_rate, 500);
        assert_eq!(nft.status, NftStatus::Normal);

        // 验证映射
        assert_eq!(
            DivinationNftPallet::result_nft(DivinationType::Meihua, 1),
            Some(0)
        );

        // 验证统计
        let stats = DivinationNftPallet::nft_stats();
        assert_eq!(stats.total_minted, 1);

        // 验证事件
        System::assert_has_event(
            Event::NftMinted {
                nft_id: 0,
                divination_type: DivinationType::Meihua,
                result_id: 1,
                owner: 1,
                rarity: Rarity::Rare, // common() 返回 Rare
                mint_fee: 1_500_000_000_000, // 1.5x base fee for Rare
            }
            .into(),
        );
    });
}

#[test]
fn mint_nft_works_for_bazi() {
    new_test_ext().execute_with(|| {
        // 设置模拟数据：Bob 创建了一个八字命盘
        MockDivinationProvider::add_result(
            DivinationType::Bazi,
            100,
            2, // Bob
            RarityInput {
                primary_score: 50,
                secondary_score: 30,
                is_special_date: true,
                is_special_combination: false,
                custom_factors: [0, 0, 0, 0],
            },
        );

        // Bob 铸造 NFT
        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(2),
            DivinationType::Bazi,
            100,
            b"Bob's Destiny Chart".to_vec(),
            b"QmBaziImageCid456".to_vec(),
            Some(b"QmDescriptionCid".to_vec()),
            None,
            1000, // 10% royalty
        ));

        // 验证 NFT
        let nft = DivinationNftPallet::nfts(0).expect("NFT should exist");
        assert_eq!(nft.divination_type, DivinationType::Bazi);
        assert_eq!(nft.result_id, 100);
        assert_eq!(nft.owner, 2);

        // 验证按类型统计
        let type_stats = DivinationNftPallet::type_stats(DivinationType::Bazi);
        assert_eq!(type_stats.minted_count, 1);
    });
}

#[test]
fn mint_nft_fails_if_not_owner() {
    new_test_ext().execute_with(|| {
        // Alice 创建的卦象
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        // Bob 尝试铸造（应该失败）
        assert_noop!(
            DivinationNftPallet::mint_nft(
                RuntimeOrigin::signed(2), // Bob
                DivinationType::Meihua,
                1,
                b"Test".to_vec(),
                b"QmCid".to_vec(),
                None,
                None,
                500,
            ),
            Error::<Test>::NotResultOwner
        );
    });
}

#[test]
fn mint_nft_fails_if_already_minted() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        // 第一次铸造成功
        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Test".to_vec(),
            b"QmCid".to_vec(),
            None,
            None,
            500,
        ));

        // 第二次铸造失败 - ResultNotMintable 因为已被 mark_as_nfted
        assert_noop!(
            DivinationNftPallet::mint_nft(
                RuntimeOrigin::signed(1),
                DivinationType::Meihua,
                1,
                b"Test2".to_vec(),
                b"QmCid2".to_vec(),
                None,
                None,
                500,
            ),
            Error::<Test>::ResultNotMintable
        );
    });
}

#[test]
fn mint_nft_fails_if_result_not_found() {
    new_test_ext().execute_with(|| {
        // 没有添加任何模拟数据
        assert_noop!(
            DivinationNftPallet::mint_nft(
                RuntimeOrigin::signed(1),
                DivinationType::Meihua,
                999, // 不存在的 ID
                b"Test".to_vec(),
                b"QmCid".to_vec(),
                None,
                None,
                500,
            ),
            Error::<Test>::DivinationResultNotFound
        );
    });
}

// ==================== 转移测试 ====================

#[test]
fn transfer_nft_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        // 铸造
        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Test".to_vec(),
            b"QmCid".to_vec(),
            None,
            None,
            500,
        ));

        // 转移给 Bob
        assert_ok!(DivinationNftPallet::transfer_nft(RuntimeOrigin::signed(1), 0, 2));

        // 验证所有权变更
        let nft = DivinationNftPallet::nfts(0).unwrap();
        assert_eq!(nft.owner, 2);
        assert_eq!(nft.transfer_count, 1);

        // 验证用户 NFT 列表
        assert!(!DivinationNftPallet::user_nfts(1).contains(&0));
        assert!(DivinationNftPallet::user_nfts(2).contains(&0));
    });
}

#[test]
fn transfer_nft_fails_if_not_owner() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Test".to_vec(),
            b"QmCid".to_vec(),
            None,
            None,
            500,
        ));

        // Bob 尝试转移 Alice 的 NFT
        assert_noop!(
            DivinationNftPallet::transfer_nft(RuntimeOrigin::signed(2), 0, 3),
            Error::<Test>::NotNftOwner
        );
    });
}

// ==================== 挂单/购买测试 ====================

#[test]
fn list_and_buy_nft_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        // Alice 铸造
        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Test".to_vec(),
            b"QmCid".to_vec(),
            None,
            None,
            500, // 5% royalty
        ));

        let alice_balance_before = Balances::free_balance(1);

        // Alice 挂单
        let price = 10_000_000_000_000u64; // 10 UNIT
        assert_ok!(DivinationNftPallet::list_nft(
            RuntimeOrigin::signed(1),
            0,
            price,
            None,
        ));

        // 验证状态
        let nft = DivinationNftPallet::nfts(0).unwrap();
        assert_eq!(nft.status, NftStatus::Listed);

        // Bob 购买
        assert_ok!(DivinationNftPallet::buy_nft(RuntimeOrigin::signed(2), 0));

        // 验证所有权变更
        let nft = DivinationNftPallet::nfts(0).unwrap();
        assert_eq!(nft.owner, 2);
        assert_eq!(nft.status, NftStatus::Normal);

        // 验证费用分配
        // 平台费: 10 * 2.5% = 0.25 UNIT
        // 版税: 0 (创作者=卖家，不收版税)
        // 卖家收入: 10 - 0.25 = 9.75 UNIT
        let alice_balance_after = Balances::free_balance(1);
        let platform_fee = price * 250 / 10000;
        let seller_income = price - platform_fee;
        // alice_balance_before 是铸造之后的余额，所以直接比较差值
        assert_eq!(
            alice_balance_after - alice_balance_before,
            seller_income
        );

        // 验证统计
        let stats = DivinationNftPallet::nft_stats();
        assert_eq!(stats.total_trades, 1);
        assert_eq!(stats.total_volume, price);
    });
}

#[test]
fn royalty_paid_on_resale() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        // Alice 铸造
        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Test".to_vec(),
            b"QmCid".to_vec(),
            None,
            None,
            1000, // 10% royalty
        ));

        // 转移给 Bob（免费）
        assert_ok!(DivinationNftPallet::transfer_nft(RuntimeOrigin::signed(1), 0, 2));

        let alice_balance_before = Balances::free_balance(1);

        // Bob 挂单
        let price = 10_000_000_000_000u64;
        assert_ok!(DivinationNftPallet::list_nft(
            RuntimeOrigin::signed(2),
            0,
            price,
            None,
        ));

        // Charlie 购买
        assert_ok!(DivinationNftPallet::buy_nft(RuntimeOrigin::signed(3), 0));

        // 验证 Alice 收到版税
        let alice_balance_after = Balances::free_balance(1);
        let royalty = price * 1000 / 10000; // 10%
        assert_eq!(alice_balance_after - alice_balance_before, royalty);
    });
}

// ==================== 出价测试 ====================

#[test]
fn make_and_accept_offer_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Test".to_vec(),
            b"QmCid".to_vec(),
            None,
            None,
            500,
        ));

        let offer_amount = 5_000_000_000_000u64;

        // Bob 出价
        assert_ok!(DivinationNftPallet::make_offer(
            RuntimeOrigin::signed(2),
            0,
            offer_amount,
        ));

        // 验证资金被锁定
        assert_eq!(Balances::reserved_balance(2), offer_amount);

        // Alice 接受出价
        assert_ok!(DivinationNftPallet::accept_offer(RuntimeOrigin::signed(1), 0));

        // 验证所有权变更
        let nft = DivinationNftPallet::nfts(0).unwrap();
        assert_eq!(nft.owner, 2);

        // 验证资金解锁并转移
        assert_eq!(Balances::reserved_balance(2), 0);
    });
}

#[test]
fn cancel_offer_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Test".to_vec(),
            b"QmCid".to_vec(),
            None,
            None,
            500,
        ));

        let offer_amount = 5_000_000_000_000u64;
        let bob_balance_before = Balances::free_balance(2);

        // Bob 出价
        assert_ok!(DivinationNftPallet::make_offer(
            RuntimeOrigin::signed(2),
            0,
            offer_amount,
        ));

        // Bob 取消出价
        assert_ok!(DivinationNftPallet::cancel_offer(RuntimeOrigin::signed(2), 0));

        // 验证资金返还
        assert_eq!(Balances::free_balance(2), bob_balance_before);
        assert_eq!(Balances::reserved_balance(2), 0);
    });
}

// ==================== 收藏集测试 ====================

#[test]
fn collection_operations_work() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());
        MockDivinationProvider::add_result(DivinationType::Meihua, 2, 1, RarityInput::common());

        // 铸造两个 NFT
        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"NFT 1".to_vec(),
            b"QmCid1".to_vec(),
            None,
            None,
            500,
        ));

        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            2,
            b"NFT 2".to_vec(),
            b"QmCid2".to_vec(),
            None,
            None,
            500,
        ));

        // 创建收藏集
        assert_ok!(DivinationNftPallet::create_collection(
            RuntimeOrigin::signed(1),
            b"My Collection".to_vec(),
            None,
            None,
            true,
        ));

        // 添加 NFT 到收藏集
        assert_ok!(DivinationNftPallet::add_to_collection(
            RuntimeOrigin::signed(1),
            0,
            0,
        ));

        assert_ok!(DivinationNftPallet::add_to_collection(
            RuntimeOrigin::signed(1),
            1,
            0,
        ));

        // 验证收藏集
        let collection = DivinationNftPallet::collections(0).unwrap();
        assert_eq!(collection.nft_count, 2);

        let collection_nfts = DivinationNftPallet::collection_nfts(0);
        assert_eq!(collection_nfts.len(), 2);

        // 移除 NFT
        assert_ok!(DivinationNftPallet::remove_from_collection(
            RuntimeOrigin::signed(1),
            0,
            0,
        ));

        let collection = DivinationNftPallet::collections(0).unwrap();
        assert_eq!(collection.nft_count, 1);
    });
}

// ==================== 销毁测试 ====================

#[test]
fn burn_nft_works() {
    new_test_ext().execute_with(|| {
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());

        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Test".to_vec(),
            b"QmCid".to_vec(),
            None,
            None,
            500,
        ));

        // 销毁
        assert_ok!(DivinationNftPallet::burn_nft(RuntimeOrigin::signed(1), 0));

        // 验证状态
        let nft = DivinationNftPallet::nfts(0).unwrap();
        assert_eq!(nft.status, NftStatus::Burned);

        // 验证映射被移除
        assert_eq!(DivinationNftPallet::result_nft(DivinationType::Meihua, 1), None);

        // 验证统计
        let stats = DivinationNftPallet::nft_stats();
        assert_eq!(stats.total_burned, 1);
    });
}

// ==================== 多类型测试 ====================

#[test]
fn multiple_divination_types_work() {
    new_test_ext().execute_with(|| {
        // 添加不同类型的占卜结果
        MockDivinationProvider::add_result(DivinationType::Meihua, 1, 1, RarityInput::common());
        MockDivinationProvider::add_result(DivinationType::Bazi, 1, 1, RarityInput::common());
        MockDivinationProvider::add_result(DivinationType::Liuyao, 1, 1, RarityInput::common());

        // 同一 result_id 不同类型可以分别铸造
        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Meihua,
            1,
            b"Meihua NFT".to_vec(),
            b"QmMeihua".to_vec(),
            None,
            None,
            500,
        ));

        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Bazi,
            1,
            b"Bazi NFT".to_vec(),
            b"QmBazi".to_vec(),
            None,
            None,
            500,
        ));

        assert_ok!(DivinationNftPallet::mint_nft(
            RuntimeOrigin::signed(1),
            DivinationType::Liuyao,
            1,
            b"Liuyao NFT".to_vec(),
            b"QmLiuyao".to_vec(),
            None,
            None,
            500,
        ));

        // 验证各类型独立映射
        assert!(DivinationNftPallet::result_nft(DivinationType::Meihua, 1).is_some());
        assert!(DivinationNftPallet::result_nft(DivinationType::Bazi, 1).is_some());
        assert!(DivinationNftPallet::result_nft(DivinationType::Liuyao, 1).is_some());

        // 验证各类型统计
        assert_eq!(DivinationNftPallet::type_stats(DivinationType::Meihua).minted_count, 1);
        assert_eq!(DivinationNftPallet::type_stats(DivinationType::Bazi).minted_count, 1);
        assert_eq!(DivinationNftPallet::type_stats(DivinationType::Liuyao).minted_count, 1);
    });
}
