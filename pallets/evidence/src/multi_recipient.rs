// 函数级详细中文注释：多接收方证据支持模块
//
// 本模块为 pallet-evidence 提供多接收方加密证据的支持
// 
// 核心功能：
// - 记录多接收方证据的元数据
// - 记录授权接收方列表
// - 记录访问日志
// - 提供查询接口
//
// @author Cosmos Team
// @date 2025-10-23

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 函数级详细中文注释：多接收方证据元数据
/// 
/// 存储多接收方加密证据的元数据信息
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(AccountId, BlockNumber, MaxRecipients, MaxCidLen))]
pub struct MultiRecipientEvidenceMeta<
	AccountId,
	BlockNumber,
	MaxRecipients: Get<u32>,
	MaxCidLen: Get<u32>,
> {
	/// 证据CID（IPFS上的加密数据）
	pub cid: BoundedVec<u8, MaxCidLen>,
	
	/// 提交者账户
	pub submitter: AccountId,
	
	/// 授权接收方列表（委员会成员）
	pub authorized_recipients: BoundedVec<AccountId, MaxRecipients>,
	
	/// 提交时间
	pub submitted_at: BlockNumber,
	
	/// 加密方法（如 "AES-256-GCM + X25519"）
	pub encryption_method: BoundedVec<u8, ConstU32<64>>,
	
	/// 证据类型（如 "chat_evidence"）
	pub evidence_type: BoundedVec<u8, ConstU32<32>>,
	
	/// 原始内容大小（字节）
	pub original_size: u64,
	
	/// 是否已归档
	pub is_archived: bool,
}

/// 函数级详细中文注释：访问记录
/// 
/// 记录委员会成员访问加密证据的日志
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(AccountId, BlockNumber, MaxPurposeLen))]
pub struct AccessRecord<AccountId, BlockNumber, MaxPurposeLen: Get<u32>> {
	/// 访问者账户
	pub accessor: AccountId,
	
	/// 访问时间
	pub accessed_at: BlockNumber,
	
	/// 访问目的（如 "review", "audit"）
	pub purpose: BoundedVec<u8, MaxPurposeLen>,
}

impl<AccountId, BlockNumber, MaxRecipients, MaxCidLen>
	MultiRecipientEvidenceMeta<AccountId, BlockNumber, MaxRecipients, MaxCidLen>
where
	AccountId: Clone,
	BlockNumber: Clone,
	MaxRecipients: Get<u32>,
	MaxCidLen: Get<u32>,
{
	/// 函数级详细中文注释：检查账户是否为授权接收方
	pub fn is_authorized(&self, account: &AccountId) -> bool
	where
		AccountId: PartialEq,
	{
		self.authorized_recipients.iter().any(|r| r == account)
	}
	
	/// 函数级详细中文注释：获取授权接收方数量
	pub fn recipient_count(&self) -> u32 {
		self.authorized_recipients.len() as u32
	}
}

