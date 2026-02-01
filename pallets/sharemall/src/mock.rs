//! # 商城模块测试 Mock

use crate as pallet_mall;
use frame_support::{
    derive_impl,
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128},
    PalletId,
};
use sp_runtime::{
    traits::{AccountIdConversion, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Escrow: pallet_escrow,
        Mall: pallet_mall,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type AccountData = pallet_balances::AccountData<u128>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
    type Balance = u128;
    type ExistentialDeposit = ConstU128<1>;
}

parameter_types! {
    pub const EscrowPalletId: PalletId = PalletId(*b"py/escro");
}

/// Mock ExpiryPolicy
pub struct MockExpiryPolicy;

impl pallet_escrow::pallet::ExpiryPolicy<u64, u64> for MockExpiryPolicy {
    fn on_expire(_id: u64) -> Result<pallet_escrow::pallet::ExpiryAction<u64>, sp_runtime::DispatchError> {
        Ok(pallet_escrow::pallet::ExpiryAction::Noop)
    }
    
    fn now() -> u64 {
        frame_system::Pallet::<Test>::block_number()
    }
}

impl pallet_escrow::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EscrowPalletId = EscrowPalletId;
    type AuthorizedOrigin = frame_system::EnsureRoot<u64>;
    type AdminOrigin = frame_system::EnsureRoot<u64>;
    type MaxExpiringPerBlock = ConstU32<10>;
    type ExpiryPolicy = MockExpiryPolicy;
}

parameter_types! {
    pub const PlatformAccount: u64 = 100;
    pub const MinShopDeposit: u128 = 100;
    pub const PlatformFeeRate: u16 = 200; // 2%
    pub const PaymentTimeout: u64 = 14400; // 24h
    pub const ShipTimeout: u64 = 43200; // 72h
    pub const ConfirmTimeout: u64 = 100800; // 7d
}

impl pallet_mall::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Escrow = Escrow;
    type PlatformAccount = PlatformAccount;
    type MinShopDeposit = MinShopDeposit;
    type PlatformFeeRate = PlatformFeeRate;
    type PaymentTimeout = PaymentTimeout;
    type ShipTimeout = ShipTimeout;
    type ConfirmTimeout = ConfirmTimeout;
    type MaxProductsPerShop = ConstU32<100>;
    type MaxShopNameLength = ConstU32<64>;
    type MaxCidLength = ConstU32<64>;
    type GovernanceOrigin = frame_system::EnsureRoot<u64>;
}

/// 构建测试外部状态
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    // 计算 escrow 账户地址
    let escrow_account: u64 = EscrowPalletId::get().into_account_truncating();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10000),       // 店主1
            (2, 10000),       // 店主2
            (3, 10000),       // 买家1
            (4, 10000),       // 买家2
            (100, 1),         // 平台账户
            (escrow_account, 1), // Escrow 账户
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
