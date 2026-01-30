//! Tests for pallet-meowstar-pet

use crate::{mock::*, Error, Event, Rarity, Element};
use frame_support::{assert_noop, assert_ok};

#[test]
fn hatch_pet_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // 孵化宠物
        assert_ok!(MeowstarPet::hatch_pet(
            RuntimeOrigin::signed(ALICE),
            b"Kitty".to_vec().try_into().unwrap()
        ));
        
        // 检查宠物是否创建
        let pet = MeowstarPet::pets(0).expect("Pet should exist");
        assert_eq!(pet.owner, ALICE);
        assert_eq!(pet.level, 1);
        assert_eq!(pet.evolution_stage, 0);
        
        // 检查事件
        System::assert_has_event(Event::PetHatched {
            pet_id: 0,
            owner: ALICE,
            rarity: pet.rarity,
            element: pet.element,
        }.into());
    });
}

#[test]
fn hatch_pet_fails_with_insufficient_balance() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // 余额不足的账户
        assert_noop!(
            MeowstarPet::hatch_pet(
                RuntimeOrigin::signed(POOR_ACCOUNT),
                b"Kitty".to_vec().try_into().unwrap()
            ),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn level_up_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // 先孵化宠物
        assert_ok!(MeowstarPet::hatch_pet(
            RuntimeOrigin::signed(ALICE),
            b"Kitty".to_vec().try_into().unwrap()
        ));
        
        // 升级
        assert_ok!(MeowstarPet::level_up(RuntimeOrigin::signed(ALICE), 0));
        
        // 检查等级
        let pet = MeowstarPet::pets(0).expect("Pet should exist");
        assert_eq!(pet.level, 2);
    });
}

#[test]
fn level_up_fails_not_owner() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Alice 孵化宠物
        assert_ok!(MeowstarPet::hatch_pet(
            RuntimeOrigin::signed(ALICE),
            b"Kitty".to_vec().try_into().unwrap()
        ));
        
        // Bob 尝试升级 Alice 的宠物
        assert_noop!(
            MeowstarPet::level_up(RuntimeOrigin::signed(BOB), 0),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Alice 孵化宠物
        assert_ok!(MeowstarPet::hatch_pet(
            RuntimeOrigin::signed(ALICE),
            b"Kitty".to_vec().try_into().unwrap()
        ));
        
        // 转移给 Bob
        assert_ok!(MeowstarPet::transfer(RuntimeOrigin::signed(ALICE), 0, BOB));
        
        // 检查所有权
        let pet = MeowstarPet::pets(0).expect("Pet should exist");
        assert_eq!(pet.owner, BOB);
        
        // 检查事件
        System::assert_has_event(Event::PetTransferred {
            pet_id: 0,
            from: ALICE,
            to: BOB,
        }.into());
    });
}

#[test]
fn transfer_fails_not_owner() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Alice 孵化宠物
        assert_ok!(MeowstarPet::hatch_pet(
            RuntimeOrigin::signed(ALICE),
            b"Kitty".to_vec().try_into().unwrap()
        ));
        
        // Bob 尝试转移 Alice 的宠物
        assert_noop!(
            MeowstarPet::transfer(RuntimeOrigin::signed(BOB), 0, CHARLIE),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn transfer_fails_to_self() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Alice 孵化宠物
        assert_ok!(MeowstarPet::hatch_pet(
            RuntimeOrigin::signed(ALICE),
            b"Kitty".to_vec().try_into().unwrap()
        ));
        
        // 尝试转移给自己
        assert_noop!(
            MeowstarPet::transfer(RuntimeOrigin::signed(ALICE), 0, ALICE),
            Error::<Test>::CannotTransferToSelf
        );
    });
}

#[test]
fn rename_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // 孵化宠物
        assert_ok!(MeowstarPet::hatch_pet(
            RuntimeOrigin::signed(ALICE),
            b"Kitty".to_vec().try_into().unwrap()
        ));
        
        // 重命名
        assert_ok!(MeowstarPet::rename(
            RuntimeOrigin::signed(ALICE),
            0,
            b"NewName".to_vec().try_into().unwrap()
        ));
        
        // 检查名字
        let pet = MeowstarPet::pets(0).expect("Pet should exist");
        assert_eq!(pet.name.to_vec(), b"NewName".to_vec());
    });
}

#[test]
fn evolve_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // 孵化宠物
        assert_ok!(MeowstarPet::hatch_pet(
            RuntimeOrigin::signed(ALICE),
            b"Kitty".to_vec().try_into().unwrap()
        ));
        
        // 升级到 10 级
        for _ in 0..9 {
            assert_ok!(MeowstarPet::level_up(RuntimeOrigin::signed(ALICE), 0));
        }
        
        let pet_before = MeowstarPet::pets(0).expect("Pet should exist");
        assert_eq!(pet_before.level, 10);
        
        // 进化
        assert_ok!(MeowstarPet::evolve(RuntimeOrigin::signed(ALICE), 0, None));
        
        let pet_after = MeowstarPet::pets(0).expect("Pet should exist");
        assert_eq!(pet_after.evolution_stage, 1);
    });
}

#[test]
fn evolve_fails_level_too_low() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // 孵化宠物
        assert_ok!(MeowstarPet::hatch_pet(
            RuntimeOrigin::signed(ALICE),
            b"Kitty".to_vec().try_into().unwrap()
        ));
        
        // 尝试进化 (等级不足)
        assert_noop!(
            MeowstarPet::evolve(RuntimeOrigin::signed(ALICE), 0, None),
            Error::<Test>::LevelTooLow
        );
    });
}

#[test]
fn max_pets_per_account_enforced() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // 孵化最大数量的宠物
        for i in 0..10 {
            assert_ok!(MeowstarPet::hatch_pet(
                RuntimeOrigin::signed(ALICE),
                format!("Pet{}", i).as_bytes().to_vec().try_into().unwrap()
            ));
        }
        
        // 尝试再孵化一个
        assert_noop!(
            MeowstarPet::hatch_pet(
                RuntimeOrigin::signed(ALICE),
                b"TooMany".to_vec().try_into().unwrap()
            ),
            Error::<Test>::MaxPetsReached
        );
    });
}

#[test]
fn get_pets_by_owner_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Alice 孵化 3 个宠物
        for i in 0..3 {
            assert_ok!(MeowstarPet::hatch_pet(
                RuntimeOrigin::signed(ALICE),
                format!("Pet{}", i).as_bytes().to_vec().try_into().unwrap()
            ));
        }
        
        // Bob 孵化 2 个宠物
        for i in 0..2 {
            assert_ok!(MeowstarPet::hatch_pet(
                RuntimeOrigin::signed(BOB),
                format!("BobPet{}", i).as_bytes().to_vec().try_into().unwrap()
            ));
        }
        
        // 检查 Alice 的宠物
        let alice_pets = MeowstarPet::get_pets_by_owner(&ALICE);
        assert_eq!(alice_pets.len(), 3);
        
        // 检查 Bob 的宠物
        let bob_pets = MeowstarPet::get_pets_by_owner(&BOB);
        assert_eq!(bob_pets.len(), 2);
    });
}
