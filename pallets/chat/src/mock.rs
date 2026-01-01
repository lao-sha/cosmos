//! # Mock环境配置
//! 
//! 用于Chat Pallet的单元测试

use crate as pallet_chat;
use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU64, Randomness, UnixTime},
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// 配置测试运行时
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Chat: pallet_chat,
	}
);

// System配置
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = sp_core::H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
	type RuntimeTask = ();
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
	type ExtensionsWeightInfo = ();
}

// Chat Pallet配置
parameter_types! {
	/// IPFS CID最大长度：100字节（足够容纳加密后的CID）
	pub const MaxCidLen: u32 = 100;
	/// 每个用户最多会话数：100（已废弃，但保留以兼容）
	pub const MaxSessionsPerUser: u32 = 100;
	/// 每个会话最多消息数：1000（已废弃，但保留以兼容）
	pub const MaxMessagesPerSession: u32 = 1000;
	/// 频率限制：时间窗口（100个区块 ≈ 10分钟）
	pub const RateLimitWindow: u64 = 100;
	/// 频率限制：时间窗口内最大消息数（10条/10分钟）
	pub const MaxMessagesPerWindow: u32 = 10;
	/// 消息过期时间：1000个区块（测试用）
	pub const MessageExpirationTime: u64 = 1000;
}

/// 简单的测试用随机数生成器
pub struct TestRandomness;
impl Randomness<sp_core::H256, u64> for TestRandomness {
	fn random(subject: &[u8]) -> (sp_core::H256, u64) {
		// 简单的伪随机实现用于测试
		let mut seed = [0u8; 32];
		for (i, byte) in subject.iter().enumerate() {
			if i < 32 {
				seed[i] = *byte;
			}
		}
		// 使用简单的变换生成不同的随机值
		for i in 0..32 {
			seed[i] = seed[i].wrapping_add(i as u8).wrapping_add(1);
		}
		(sp_core::H256::from(seed), frame_system::Pallet::<Test>::block_number())
	}
}

/// 简单的测试用时间戳
pub struct TestTime;
impl UnixTime for TestTime {
	fn now() -> core::time::Duration {
		// 返回基于区块号的简单时间戳
		let block_number = frame_system::Pallet::<Test>::block_number();
		core::time::Duration::from_secs(block_number * 6) // 6秒/块
	}
}

impl pallet_chat::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_chat::SubstrateWeight<Test>;
	type MaxCidLen = MaxCidLen;
	type MaxSessionsPerUser = MaxSessionsPerUser;
	type MaxMessagesPerSession = MaxMessagesPerSession;
	type RateLimitWindow = RateLimitWindow;
	type MaxMessagesPerWindow = MaxMessagesPerWindow;
	type MessageExpirationTime = MessageExpirationTime;
	// ChatUserId相关配置
	type Randomness = TestRandomness;
	type UnixTime = TestTime;
	type MaxNicknameLength = frame_support::traits::ConstU32<64>;
	type MaxSignatureLength = frame_support::traits::ConstU32<256>;
}

/// 函数级详细中文注释：构建测试存储
/// 用于初始化测试环境
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

/// 函数级详细中文注释：运行到指定区块
/// 用于测试中推进区块高度
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
	}
}

/// 函数级详细中文注释：生成加密的CID
/// 用于测试，模拟加密后的IPFS CID
pub fn encrypted_cid(id: u8) -> Vec<u8> {
	// 生成一个大于50字节的CID，模拟加密后的CID
	let mut cid = b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_vec();
	cid.push(id); // 添加一个字节使其唯一
	cid
}

/// 函数级详细中文注释：生成未加密的CID
/// 用于测试CID加密检查
pub fn unencrypted_cid() -> Vec<u8> {
	// 标准的CIDv0，46字节，以Qm开头
	b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec()
}

