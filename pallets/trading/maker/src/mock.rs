// 函数级详细中文注释：Maker Pallet Mock 环境
//
// 用于单元测试的模拟 runtime 环境

use crate as pallet_maker;
use frame_support::parameter_types;
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

// 配置模拟 runtime
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Maker: pallet_maker,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = sp_core::H256;
    type Hashing = sp_runtime::traits::BlakeTwo256;
    type AccountId = u64;
    type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// 🔮 延迟实现：Maker Config 需要以下依赖
// 1. pallet-balances (Currency)
// 2. pallet-trading-credit (MakerCredit)
// 3. pallet-trading-common (PricingProvider)
// 4. pallet-cosmos-ipfs (ContentRegistry)
// 5. pallet-timestamp (UnixTime)
// 
// 完整 mock 实现需要配置所有依赖 pallet
// 建议：使用集成测试或 runtime 级别测试验证功能

// 函数级详细中文注释：创建测试环境
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}

