//! # 占卜服务市场数据类型定义
//!
//! 本模块定义了通用占卜服务市场所需的所有核心数据结构，支持多种占卜类型：
//! - 梅花易数
//! - 八字命理
//! - 六爻占卜
//! - 奇门遁甲
//! - 紫微斗数
//!
//! ## 主要功能
//! - 服务提供者信息
//! - 服务套餐定义
//! - 订单状态管理
//! - 评价与评分系统

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use pallet_divination_common::DivinationType;
use scale_info::TypeInfo;

/// 服务提供者状态
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum ProviderStatus {
    /// 待审核 - 新注册等待验证
    #[default]
    Pending = 0,
    /// 已激活 - 正常运营
    Active = 1,
    /// 已暂停 - 暂时停止接单
    Paused = 2,
    /// 已封禁 - 违规被封
    Banned = 3,
    /// 已注销 - 主动退出
    Deactivated = 4,
}

/// 服务提供者认证等级
///
/// 根据经验、评分和认证情况分级
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum ProviderTier {
    /// 新手 - 刚入驻的提供者
    #[default]
    Novice = 0,
    /// 认证 - 通过基础认证
    Certified = 1,
    /// 资深 - 完成一定订单量
    Senior = 2,
    /// 专家 - 高评分高订单量
    Expert = 3,
    /// 大师 - 顶级认证
    Master = 4,
}

impl ProviderTier {
    /// 获取等级所需的最低订单数
    pub fn min_orders(&self) -> u32 {
        match self {
            ProviderTier::Novice => 0,
            ProviderTier::Certified => 10,
            ProviderTier::Senior => 50,
            ProviderTier::Expert => 200,
            ProviderTier::Master => 500,
        }
    }

    /// 获取等级所需的最低评分（* 100）
    pub fn min_rating(&self) -> u16 {
        match self {
            ProviderTier::Novice => 0,
            ProviderTier::Certified => 350, // 3.5 星
            ProviderTier::Senior => 400,    // 4.0 星
            ProviderTier::Expert => 450,    // 4.5 星
            ProviderTier::Master => 480,    // 4.8 星
        }
    }

    /// 获取平台抽成比例（基点，10000 = 100%）
    /// 
    /// 统一设置为 10%，不再按等级差异化
    pub fn platform_fee_rate(&self) -> u16 {
        match self {
            ProviderTier::Novice => 1000,    // 10%
            ProviderTier::Certified => 1000, // 10%
            ProviderTier::Senior => 1000,    // 10%
            ProviderTier::Expert => 1000,    // 10%
            ProviderTier::Master => 1000,    // 10%
        }
    }
}

/// 服务提供者信息
///
/// 支持多种占卜类型的服务提供者
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxNameLen, MaxBioLen))]
pub struct Provider<AccountId, Balance, BlockNumber, MaxNameLen: Get<u32>, MaxBioLen: Get<u32>> {
    /// 账户地址
    pub account: AccountId,
    /// 显示名称
    pub name: BoundedVec<u8, MaxNameLen>,
    /// 个人简介
    pub bio: BoundedVec<u8, MaxBioLen>,
    /// 头像 IPFS CID
    pub avatar_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    /// 认证等级
    pub tier: ProviderTier,
    /// 状态
    pub status: ProviderStatus,
    /// 保证金
    pub deposit: Balance,
    /// 注册时间
    pub registered_at: BlockNumber,
    /// 总订单数
    pub total_orders: u32,
    /// 完成订单数
    pub completed_orders: u32,
    /// 取消订单数
    pub cancelled_orders: u32,
    /// 总评分次数
    pub total_ratings: u32,
    /// 评分总和（用于计算平均分）
    pub rating_sum: u64,
    /// 总收入
    pub total_earnings: Balance,
    /// 擅长领域（位图）
    pub specialties: u16,
    /// 支持的占卜类型（位图）
    pub supported_divination_types: u8,
    /// 是否接受紧急订单
    pub accepts_urgent: bool,
    /// 最后活跃时间
    pub last_active_at: BlockNumber,
}

impl<AccountId, Balance: Default, BlockNumber, MaxNameLen: Get<u32>, MaxBioLen: Get<u32>>
    Provider<AccountId, Balance, BlockNumber, MaxNameLen, MaxBioLen>
{
    /// 计算平均评分（* 100，如 450 = 4.5 星）
    pub fn average_rating(&self) -> u16 {
        if self.total_ratings == 0 {
            return 0;
        }
        ((self.rating_sum * 100) / self.total_ratings as u64) as u16
    }

    /// 计算完成率（* 100）
    pub fn completion_rate(&self) -> u16 {
        if self.total_orders == 0 {
            return 10000; // 100%
        }
        ((self.completed_orders as u64 * 10000) / self.total_orders as u64) as u16
    }

    /// 检查是否擅长指定领域
    pub fn has_specialty(&self, specialty: Specialty) -> bool {
        let bit = 1u16 << (specialty as u16);
        self.specialties & bit != 0
    }

    /// 检查是否支持指定的占卜类型
    pub fn supports_divination_type(&self, divination_type: DivinationType) -> bool {
        let type_bit = 1u8 << (divination_type as u8);
        self.supported_divination_types & type_bit != 0
    }
}

/// 擅长领域
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
)]
pub enum Specialty {
    /// 事业运势
    Career = 0,
    /// 感情婚姻
    Relationship = 1,
    /// 财运投资
    Wealth = 2,
    /// 健康养生
    Health = 3,
    /// 学业考试
    Education = 4,
    /// 出行旅游
    Travel = 5,
    /// 官司诉讼
    Legal = 6,
    /// 寻人寻物
    Finding = 7,
    /// 风水堪舆
    FengShui = 8,
    /// 择日选时
    DateSelection = 9,
}

/// 服务套餐类型
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum ServiceType {
    /// 文字解卦 - 纯文字回复
    #[default]
    TextReading = 0,
    /// 语音解卦 - 语音回复
    VoiceReading = 1,
    /// 视频解卦 - 视频回复
    VideoReading = 2,
    /// 实时咨询 - 一对一实时
    LiveConsultation = 3,
}

impl ServiceType {
    /// 获取基础时长（分钟）
    pub fn base_duration(&self) -> u32 {
        match self {
            ServiceType::TextReading => 0,       // 无时长限制
            ServiceType::VoiceReading => 10,     // 10分钟
            ServiceType::VideoReading => 15,     // 15分钟
            ServiceType::LiveConsultation => 30, // 30分钟
        }
    }
}

/// 服务套餐
///
/// 支持多种占卜类型的服务套餐
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxDescLen))]
pub struct ServicePackage<Balance, MaxDescLen: Get<u32>> {
    /// 套餐 ID
    pub id: u32,
    /// 占卜类型
    pub divination_type: DivinationType,
    /// 服务类型
    pub service_type: ServiceType,
    /// 套餐名称
    pub name: BoundedVec<u8, ConstU32<64>>,
    /// 套餐描述
    pub description: BoundedVec<u8, MaxDescLen>,
    /// 价格
    pub price: Balance,
    /// 服务时长（分钟，0 表示不限）
    pub duration: u32,
    /// 包含追问次数
    pub follow_up_count: u8,
    /// 是否支持加急
    pub urgent_available: bool,
    /// 加急加价比例（基点）
    pub urgent_surcharge: u16,
    /// 是否启用
    pub is_active: bool,
    /// 销量
    pub sales_count: u32,
}

/// 解读处理状态（OCW 异步流程）
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum InterpretationProcessStatus {
    /// 待处理
    #[default]
    Pending = 0,
    /// 处理中（OCW 已获取）
    Processing = 1,
    /// 已确认
    Confirmed = 2,
    /// 处理失败
    Failed = 3,
}

/// 待处理解读（多媒体内容）
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxCidLen, MaxMediaCount))]
pub struct PendingInterpretation<BlockNumber, MaxCidLen: Get<u32>, MaxMediaCount: Get<u32>> {
    /// 订单ID
    pub order_id: u64,
    /// 文字解读 CID（主要内容）
    pub text_cid: BoundedVec<u8, MaxCidLen>,
    /// 图片 CID 列表
    pub imgs: BoundedVec<BoundedVec<u8, MaxCidLen>, MaxMediaCount>,
    /// 视频 CID 列表
    pub vids: BoundedVec<BoundedVec<u8, MaxCidLen>, MaxMediaCount>,
    /// 文档 CID 列表
    pub docs: BoundedVec<BoundedVec<u8, MaxCidLen>, MaxMediaCount>,
    /// 提交时间
    pub submitted_at: BlockNumber,
    /// 处理状态
    pub status: InterpretationProcessStatus,
    /// 重试次数
    pub retry_count: u8,
}

/// 订单状态
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum OrderStatus {
    /// 待支付
    #[default]
    PendingPayment = 0,
    /// 已支付，等待接单
    Paid = 1,
    /// 已接单，处理中
    Accepted = 2,
    /// 已完成解读
    Completed = 3,
    /// 已评价
    Reviewed = 4,
    /// 已取消
    Cancelled = 5,
    /// 已退款
    Refunded = 6,
    /// 争议中
    Disputed = 7,
    /// 解读已提交，等待 OCW 确认（异步结算）
    InterpretationSubmitted = 8,
}

/// 订单信息
///
/// 支持多种占卜类型的通用订单
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct Order<AccountId, Balance, BlockNumber, MaxCidLen: Get<u32>> {
    /// 订单 ID
    pub id: u64,
    /// 客户账户
    pub customer: AccountId,
    /// 服务提供者账户
    pub provider: AccountId,
    /// 占卜类型
    pub divination_type: DivinationType,
    /// 关联的占卜结果 ID（卦象 ID、命盘 ID 等）
    pub result_id: u64,
    /// 服务套餐 ID
    pub package_id: u32,
    /// 订单金额
    pub amount: Balance,
    /// 平台手续费
    pub platform_fee: Balance,
    /// 是否加急
    pub is_urgent: bool,
    /// 状态
    pub status: OrderStatus,
    /// 客户问题描述 CID
    pub question_cid: BoundedVec<u8, MaxCidLen>,
    /// 解读结果 CID（服务提供者提交的专业解读内容）
    pub interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 支付时间
    pub paid_at: Option<BlockNumber>,
    /// 接单时间
    pub accepted_at: Option<BlockNumber>,
    /// 完成时间
    pub completed_at: Option<BlockNumber>,
    /// 剩余追问次数
    pub follow_ups_remaining: u8,
    /// 评分
    pub rating: Option<u8>,
    /// 评价内容 CID
    pub review_cid: Option<BoundedVec<u8, MaxCidLen>>,
}

/// 追问记录
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct FollowUp<BlockNumber, MaxCidLen: Get<u32>> {
    /// 追问内容 CID
    pub question_cid: BoundedVec<u8, MaxCidLen>,
    /// 回复内容 CID（服务提供者对追问的回复）
    pub reply_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 追问时间
    pub asked_at: BlockNumber,
    /// 回复时间
    pub replied_at: Option<BlockNumber>,
}

/// 评价详情
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct Review<AccountId, BlockNumber, MaxCidLen: Get<u32>> {
    /// 订单 ID
    pub order_id: u64,
    /// 评价者
    pub reviewer: AccountId,
    /// 被评价者
    pub reviewee: AccountId,
    /// 占卜类型
    pub divination_type: DivinationType,
    /// 总体评分（1-5）
    pub overall_rating: u8,
    /// 准确度评分
    pub accuracy_rating: u8,
    /// 服务态度评分
    pub attitude_rating: u8,
    /// 响应速度评分
    pub response_rating: u8,
    /// 评价内容 CID
    pub content_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 评价时间
    pub created_at: BlockNumber,
    /// 是否匿名
    pub is_anonymous: bool,
    /// 提供者回复 CID
    pub provider_reply_cid: Option<BoundedVec<u8, MaxCidLen>>,
}

// ==================== 🆕 存储膨胀防护：归档结构 ====================

/// 归档订单 L1（精简版，~64字节）
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct ArchivedOrderL1<AccountId> {
    /// 订单 ID
    pub id: u64,
    /// 客户账户
    pub customer: AccountId,
    /// 服务提供者账户
    pub provider: AccountId,
    /// 占卜类型
    pub divination_type: DivinationType,
    /// 订单金额（压缩为u64）
    pub amount: u64,
    /// 状态
    pub status: OrderStatus,
    /// 完成时间（区块号）
    pub completed_at: u32,
    /// 评分
    pub rating: Option<u8>,
}

/// 归档订单 L2（最小版，~16字节）
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct ArchivedOrderL2 {
    /// 订单 ID
    pub id: u64,
    /// 状态 (0-7)
    pub status: u8,
    /// 年月 (YYMM格式)
    pub year_month: u16,
    /// 金额档位 (0-5)
    pub amount_tier: u8,
    /// 占卜类型 (0-9)
    pub divination_type: u8,
    /// 评分 (1-5, 0=无评分)
    pub rating: u8,
}

/// 市场永久统计
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct MarketPermanentStats {
    /// 总归档订单数
    pub total_archived_orders: u64,
    /// 已完成订单数
    pub completed_orders: u64,
    /// 总交易额（压缩）
    pub total_volume: u64,
    /// 总评分（用于计算平均分）
    pub total_ratings: u64,
    /// 评分次数
    pub rating_count: u64,
}

/// 市场统计信息
#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub struct MarketStats<Balance: Default> {
    /// 活跃提供者数
    pub active_providers: u32,
    /// 总订单数
    pub total_orders: u64,
    /// 完成订单数
    pub completed_orders: u64,
    /// 总交易额
    pub total_volume: Balance,
    /// 平台总收入
    pub platform_earnings: Balance,
    /// 总评价数
    pub total_reviews: u64,
    /// 平均评分（* 100）
    pub average_rating: u16,
}

/// 按占卜类型的统计
#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub struct TypeMarketStats<Balance: Default> {
    /// 订单数量
    pub order_count: u64,
    /// 完成数量
    pub completed_count: u64,
    /// 交易额
    pub volume: Balance,
}

/// 提现请求状态
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum WithdrawalStatus {
    /// 待处理
    #[default]
    Pending = 0,
    /// 已完成
    Completed = 1,
    /// 已取消
    Cancelled = 2,
}

/// 提现请求
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct WithdrawalRequest<AccountId, Balance, BlockNumber> {
    /// 请求 ID
    pub id: u64,
    /// 申请者
    pub provider: AccountId,
    /// 提现金额
    pub amount: Balance,
    /// 状态
    pub status: WithdrawalStatus,
    /// 申请时间
    pub requested_at: BlockNumber,
    /// 处理时间
    pub processed_at: Option<BlockNumber>,
}

// ============================================================================
// 悬赏问答类型定义（混合模式 - 方案B 多人奖励）
// ============================================================================

/// 悬赏问答状态
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum BountyStatus {
    /// 开放中 - 接受回答
    #[default]
    Open = 0,
    /// 已关闭 - 停止接受回答，等待采纳
    Closed = 1,
    /// 已采纳 - 已选择答案，等待结算
    Adopted = 2,
    /// 已结算 - 奖励已分配
    Settled = 3,
    /// 已取消 - 提问者取消（无回答时）
    Cancelled = 4,
    /// 已过期 - 超时无人回答，退款
    Expired = 5,
}

/// 悬赏回答状态
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum BountyAnswerStatus {
    /// 待审核 - 等待提问者或社区审核
    #[default]
    Pending = 0,
    /// 已采纳 - 被选为最佳答案（第一名）
    Adopted = 1,
    /// 入选 - 入选优秀答案（第二、三名）
    Selected = 2,
    /// 参与奖 - 获得参与奖励
    Participated = 3,
    /// 未入选 - 未获得奖励
    Rejected = 4,
}

/// 奖励分配方案（方案B - 多人奖励）
///
/// 奖励分配比例（基点，10000 = 100%）：
/// - 第一名（被采纳）：60%
/// - 第二名：15%
/// - 第三名：5%
/// - 平台手续费：15%
/// - 其他参与者平分：5%
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
)]
pub struct RewardDistribution {
    /// 第一名奖励比例（基点）
    pub first_place: u16,
    /// 第二名奖励比例（基点）
    pub second_place: u16,
    /// 第三名奖励比例（基点）
    pub third_place: u16,
    /// 平台手续费比例（基点）
    pub platform_fee: u16,
    /// 参与奖总比例（基点）
    pub participation_pool: u16,
}

impl Default for RewardDistribution {
    /// 默认方案B分配比例
    fn default() -> Self {
        Self {
            first_place: 6000,       // 60%
            second_place: 1500,      // 15%
            third_place: 500,        // 5%
            platform_fee: 1500,      // 15%
            participation_pool: 500, // 5%
        }
    }
}

impl RewardDistribution {
    /// 验证分配比例是否合法（总和必须等于10000）
    pub fn is_valid(&self) -> bool {
        self.first_place
            .saturating_add(self.second_place)
            .saturating_add(self.third_place)
            .saturating_add(self.platform_fee)
            .saturating_add(self.participation_pool)
            == 10000
    }
}

/// 悬赏问题（基于占卜结果）
///
/// **重要**: 悬赏问答必须基于已存在的占卜结果（盘/卦）
/// 这确保解读者有完整的结构化数据进行专业分析
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct BountyQuestion<AccountId, Balance, BlockNumber, MaxCidLen: Get<u32>> {
    /// 悬赏问题 ID
    pub id: u64,
    /// 提问者账户
    pub creator: AccountId,
    /// 占卜类型
    pub divination_type: DivinationType,
    /// 关联的占卜结果 ID（必填 - 如卦象 ID、命盘 ID）
    /// 悬赏必须基于已存在的占卜结果
    pub result_id: u64,
    /// 问题描述 IPFS CID
    pub question_cid: BoundedVec<u8, MaxCidLen>,
    /// 悬赏金额
    pub bounty_amount: Balance,
    /// 截止区块
    pub deadline: BlockNumber,
    /// 最小回答数（达到后可关闭）
    pub min_answers: u8,
    /// 最大回答数
    pub max_answers: u8,
    /// 状态
    pub status: BountyStatus,
    /// 被采纳的答案 ID（第一名）
    pub adopted_answer_id: Option<u64>,
    /// 第二名答案 ID
    pub second_place_id: Option<u64>,
    /// 第三名答案 ID
    pub third_place_id: Option<u64>,
    /// 当前回答数量
    pub answer_count: u32,
    /// 奖励分配方案
    pub reward_distribution: RewardDistribution,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 关闭时间
    pub closed_at: Option<BlockNumber>,
    /// 结算时间
    pub settled_at: Option<BlockNumber>,
    /// 擅长领域（用于匹配回答者）
    pub specialty: Option<Specialty>,
    /// 是否仅限认证提供者回答
    pub certified_only: bool,
    /// 是否允许社区投票辅助选择
    pub allow_voting: bool,
    /// 总投票数
    pub total_votes: u32,
}

/// 悬赏回答
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxCidLen))]
pub struct BountyAnswer<AccountId, Balance, BlockNumber, MaxCidLen: Get<u32>> {
    /// 回答 ID
    pub id: u64,
    /// 所属悬赏问题 ID
    pub bounty_id: u64,
    /// 回答者账户
    pub answerer: AccountId,
    /// 回答内容 IPFS CID
    pub answer_cid: BoundedVec<u8, MaxCidLen>,
    /// 状态
    pub status: BountyAnswerStatus,
    /// 获得票数
    pub votes: u32,
    /// 获得奖励金额
    pub reward_amount: Balance,
    /// 提交时间
    pub submitted_at: BlockNumber,
    /// 是否为认证提供者
    pub is_certified: bool,
    /// 回答者的提供者等级（如果是提供者）
    pub provider_tier: Option<ProviderTier>,
}

/// 投票记录
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct BountyVote<AccountId, BlockNumber> {
    /// 投票者
    pub voter: AccountId,
    /// 投票的答案 ID
    pub answer_id: u64,
    /// 投票时间
    pub voted_at: BlockNumber,
}

/// 悬赏问答统计
#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub struct BountyStats<Balance: Default> {
    /// 总悬赏问题数
    pub total_bounties: u64,
    /// 活跃悬赏数（Open状态）
    pub active_bounties: u64,
    /// 已结算悬赏数
    pub settled_bounties: u64,
    /// 总悬赏金额
    pub total_bounty_amount: Balance,
    /// 已发放奖励金额
    pub total_rewards_paid: Balance,
    /// 总回答数
    pub total_answers: u64,
    /// 平均每个悬赏的回答数（* 100）
    pub avg_answers_per_bounty: u16,
}

// ============================================================================
// 个人主页类型定义
// ============================================================================

/// 服务提供者详细资料
///
/// 用于个人主页展示的扩展信息
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxDetailLen, MaxCidLen))]
pub struct ProviderProfile<BlockNumber, MaxDetailLen: Get<u32>, MaxCidLen: Get<u32>> {
    /// 详细自我介绍 IPFS CID（支持富文本/Markdown）
    pub introduction_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 从业年限
    pub experience_years: u8,

    /// 师承/学习背景
    pub background: Option<BoundedVec<u8, MaxDetailLen>>,

    /// 服务理念/座右铭
    pub motto: Option<BoundedVec<u8, ConstU32<256>>>,

    /// 擅长问题类型描述
    pub expertise_description: Option<BoundedVec<u8, MaxDetailLen>>,

    /// 工作时间说明（如：每日 9:00-21:00）
    pub working_hours: Option<BoundedVec<u8, ConstU32<128>>>,

    /// 平均响应时间（分钟）
    pub avg_response_time: Option<u32>,

    /// 是否接受预约
    pub accepts_appointment: bool,

    /// 个人主页背景图 IPFS CID
    pub banner_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 资料最后更新时间
    pub updated_at: BlockNumber,
}

/// 资质证书类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum CertificateType {
    /// 学历证书
    #[default]
    Education = 0,
    /// 专业资格证书
    Professional = 1,
    /// 行业协会认证
    Association = 2,
    /// 师承证明
    Apprenticeship = 3,
    /// 获奖证书
    Award = 4,
    /// 其他
    Other = 5,
}

/// 资质证书
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxNameLen, MaxCidLen))]
pub struct Certificate<BlockNumber, MaxNameLen: Get<u32>, MaxCidLen: Get<u32>> {
    /// 证书 ID
    pub id: u32,

    /// 证书名称
    pub name: BoundedVec<u8, MaxNameLen>,

    /// 证书类型
    pub cert_type: CertificateType,

    /// 颁发机构
    pub issuer: Option<BoundedVec<u8, MaxNameLen>>,

    /// 证书图片 IPFS CID
    pub image_cid: BoundedVec<u8, MaxCidLen>,

    /// 颁发时间（区块号）
    pub issued_at: Option<BlockNumber>,

    /// 是否已验证（管理员验证）
    pub is_verified: bool,

    /// 上传时间
    pub uploaded_at: BlockNumber,
}

/// 案例类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum PortfolioCaseType {
    /// 经典解读案例
    #[default]
    ClassicCase = 0,
    /// 教学文章
    Tutorial = 1,
    /// 理论研究
    Research = 2,
    /// 心得分享
    Sharing = 3,
}

/// 作品集/案例展示
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxTitleLen, MaxCidLen))]
pub struct PortfolioItem<BlockNumber, MaxTitleLen: Get<u32>, MaxCidLen: Get<u32>> {
    /// 作品 ID
    pub id: u32,

    /// 作品标题
    pub title: BoundedVec<u8, MaxTitleLen>,

    /// 占卜类型
    pub divination_type: DivinationType,

    /// 案例类型
    pub case_type: PortfolioCaseType,

    /// 案例内容 IPFS CID（脱敏后的解读案例）
    pub content_cid: BoundedVec<u8, MaxCidLen>,

    /// 封面图片 IPFS CID
    pub cover_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 是否精选（置顶展示）
    pub is_featured: bool,

    /// 浏览次数
    pub view_count: u32,

    /// 点赞次数
    pub like_count: u32,

    /// 发布时间
    pub published_at: BlockNumber,
}

/// 技能标签类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum SkillTagType {
    /// 占卜类型相关
    #[default]
    DivinationType = 0,
    /// 擅长领域
    Specialty = 1,
    /// 服务特色
    ServiceFeature = 2,
    /// 自定义标签
    Custom = 3,
}

/// 技能标签
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxLabelLen))]
pub struct SkillTag<MaxLabelLen: Get<u32>> {
    /// 标签名称
    pub label: BoundedVec<u8, MaxLabelLen>,

    /// 标签类型
    pub tag_type: SkillTagType,

    /// 熟练程度（1-5）
    pub proficiency: u8,
}

/// 评价标签统计
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct ReviewTagStats {
    /// "解读准确" 次数
    pub accurate_count: u32,
    /// "态度友好" 次数
    pub friendly_count: u32,
    /// "回复及时" 次数
    pub quick_response_count: u32,
    /// "专业深入" 次数
    pub professional_count: u32,
    /// "耐心解答" 次数
    pub patient_count: u32,
    /// "物超所值" 次数
    pub value_for_money_count: u32,
}

// ============================================================================
// 信用体系类型定义
// ============================================================================

/// 信用等级
///
/// 根据信用分划分的等级，影响用户权益
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum CreditLevel {
    /// 失信 (0-199)
    Bad = 0,
    /// 不良 (200-399)
    Poor = 1,
    /// 警示 (400-599)
    Warning = 2,
    /// 一般 (600-749)
    #[default]
    Fair = 3,
    /// 优秀 (750-899)
    Good = 4,
    /// 卓越 (900-1000)
    Excellent = 5,
}

impl CreditLevel {
    /// 根据分数获取等级
    pub fn from_score(score: u16) -> Self {
        match score {
            0..=199 => CreditLevel::Bad,
            200..=399 => CreditLevel::Poor,
            400..=599 => CreditLevel::Warning,
            600..=749 => CreditLevel::Fair,
            750..=899 => CreditLevel::Good,
            _ => CreditLevel::Excellent,
        }
    }

    /// 是否可以接单
    pub fn can_accept_orders(&self) -> bool {
        !matches!(self, CreditLevel::Bad)
    }

    /// 是否可以创建套餐
    pub fn can_create_packages(&self) -> bool {
        !matches!(self, CreditLevel::Bad | CreditLevel::Poor)
    }

    /// 是否可以回答悬赏
    pub fn can_answer_bounties(&self) -> bool {
        matches!(self, CreditLevel::Fair | CreditLevel::Good | CreditLevel::Excellent)
    }

    /// 获取最大同时进行订单数
    pub fn max_active_orders(&self) -> u8 {
        match self {
            CreditLevel::Bad => 0,
            CreditLevel::Poor => 1,
            CreditLevel::Warning => 3,
            CreditLevel::Fair => 5,
            CreditLevel::Good => 10,
            CreditLevel::Excellent => 20,
        }
    }

    /// 获取提现延迟（区块数，假设 6 秒/区块）
    pub fn withdrawal_delay_blocks(&self) -> u32 {
        match self {
            CreditLevel::Bad => 0,       // 禁止提现
            CreditLevel::Poor => 100800, // 7天
            CreditLevel::Warning => 43200, // 3天
            CreditLevel::Fair => 14400,   // 1天
            CreditLevel::Good => 0,       // 即时
            CreditLevel::Excellent => 0,  // 即时
        }
    }

    /// 获取平台费用调整（基点，正数增加，负数减少）
    pub fn platform_fee_modifier(&self) -> i16 {
        match self {
            CreditLevel::Bad => 0,        // 不适用
            CreditLevel::Poor => 3000,    // +30%
            CreditLevel::Warning => 1500, // +15%
            CreditLevel::Fair => 0,       // 无调整
            CreditLevel::Good => -500,    // -5%
            CreditLevel::Excellent => -1000, // -10%
        }
    }

    /// 获取搜索展示降权（基点，10000=完全隐藏）
    pub fn visibility_penalty(&self) -> u16 {
        match self {
            CreditLevel::Bad => 10000, // 完全隐藏
            CreditLevel::Poor => 5000, // 50% 降权
            CreditLevel::Warning => 2000, // 20% 降权
            _ => 0, // 无降权
        }
    }
}

/// 扣分原因
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum DeductionReason {
    /// 差评扣分
    NegativeReview = 0,
    /// 订单取消
    OrderCancellation = 1,
    /// 订单超时
    OrderTimeout = 2,
    /// 客户投诉成立
    ComplaintUpheld = 3,
    /// 违规行为
    Violation = 4,
    /// 虚假宣传
    FalseAdvertising = 5,
    /// 服务欺诈
    Fraud = 6,
    /// 辱骂客户
    Abuse = 7,
    /// 泄露隐私
    PrivacyBreach = 8,
    /// 其他
    Other = 9,
}

impl DeductionReason {
    /// 获取默认扣分值
    pub fn default_deduction(&self) -> u16 {
        match self {
            DeductionReason::NegativeReview => 5,
            DeductionReason::OrderCancellation => 10,
            DeductionReason::OrderTimeout => 15,
            DeductionReason::ComplaintUpheld => 30,
            DeductionReason::Violation => 50,
            DeductionReason::FalseAdvertising => 80,
            DeductionReason::Fraud => 200,
            DeductionReason::Abuse => 100,
            DeductionReason::PrivacyBreach => 150,
            DeductionReason::Other => 20,
        }
    }
}

/// 服务提供者信用档案
///
/// 记录提供者的信用评估数据和历史
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct CreditProfile<BlockNumber> {
    /// 当前信用分（0-1000）
    pub score: u16,

    /// 当前信用等级
    pub level: CreditLevel,

    /// 历史最高分
    pub highest_score: u16,

    /// 历史最低分
    pub lowest_score: u16,

    // ========== 服务质量维度 ==========

    /// 服务质量分（0-350）
    pub service_quality_score: u16,

    /// 平均综合评分（*100，如 450 = 4.5星）
    pub avg_overall_rating: u16,

    /// 平均准确度评分
    pub avg_accuracy_rating: u16,

    /// 平均服务态度评分
    pub avg_attitude_rating: u16,

    /// 平均响应速度评分
    pub avg_response_rating: u16,

    /// 5星好评数
    pub five_star_count: u32,

    /// 1星差评数
    pub one_star_count: u32,

    // ========== 行为规范维度 ==========

    /// 行为规范分（0-250）
    pub behavior_score: u16,

    /// 累计违规次数
    pub violation_count: u32,

    /// 累计警告次数
    pub warning_count: u32,

    /// 累计投诉次数
    pub complaint_count: u32,

    /// 投诉成立次数
    pub complaint_upheld_count: u32,

    /// 当前活跃违规数（未过期）
    pub active_violations: u8,

    // ========== 履约能力维度 ==========

    /// 履约能力分（0-300）
    pub fulfillment_score: u16,

    /// 订单完成率（基点，10000 = 100%）
    pub completion_rate: u16,

    /// 按时完成率（基点）
    pub on_time_rate: u16,

    /// 取消率（基点）
    pub cancellation_rate: u16,

    /// 超时次数
    pub timeout_count: u32,

    /// 主动取消次数
    pub active_cancel_count: u32,

    /// 平均响应时间（区块数）
    pub avg_response_blocks: u32,

    // ========== 加分项 ==========

    /// 加分项总分（0-100）
    pub bonus_score: u16,

    /// 悬赏被采纳次数
    pub bounty_adoption_count: u32,

    /// 获得认证数
    pub certification_count: u8,

    /// 连续好评天数
    pub consecutive_positive_days: u16,

    /// 是否通过实名认证
    pub is_verified: bool,

    /// 是否缴纳保证金
    pub has_deposit: bool,

    // ========== 扣分记录 ==========

    /// 累计扣分
    pub total_deductions: u16,

    /// 最近一次扣分原因
    pub last_deduction_reason: Option<DeductionReason>,

    /// 最近一次扣分时间
    pub last_deduction_at: Option<BlockNumber>,

    // ========== 统计数据 ==========

    /// 总订单数（用于计算比率）
    pub total_orders: u32,

    /// 完成订单数
    pub completed_orders: u32,

    /// 总评价数
    pub total_reviews: u32,

    // ========== 时间戳 ==========

    /// 信用档案创建时间
    pub created_at: BlockNumber,

    /// 最近更新时间
    pub updated_at: BlockNumber,

    /// 最近评估时间
    pub last_evaluated_at: BlockNumber,
}

/// 违规类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum ViolationType {
    /// 轻微违规
    Minor = 0,
    /// 一般违规
    Moderate = 1,
    /// 严重违规
    Severe = 2,
    /// 特别严重违规
    Critical = 3,
}

impl ViolationType {
    /// 获取违规等级对应的惩罚系数（基点）
    pub fn penalty_multiplier(&self) -> u16 {
        match self {
            ViolationType::Minor => 100,      // 1x
            ViolationType::Moderate => 200,   // 2x
            ViolationType::Severe => 500,     // 5x
            ViolationType::Critical => 1000,  // 10x
        }
    }

    /// 获取违规记录有效期（区块数，假设 6 秒/区块）
    pub fn record_duration(&self) -> u32 {
        match self {
            ViolationType::Minor => 432000,     // 30天
            ViolationType::Moderate => 1296000, // 90天
            ViolationType::Severe => 2592000,   // 180天
            ViolationType::Critical => 5256000, // 1年
        }
    }
}

/// 处罚类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum PenaltyType {
    /// 仅扣分
    #[default]
    DeductionOnly = 0,
    /// 警告
    Warning = 1,
    /// 限制接单
    OrderRestriction = 2,
    /// 暂停服务
    ServiceSuspension = 3,
    /// 永久封禁
    PermanentBan = 4,
}

/// 申诉结果
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum AppealResult {
    /// 申诉成功，撤销处罚
    Upheld = 0,
    /// 申诉部分成功，减轻处罚
    PartiallyUpheld = 1,
    /// 申诉失败
    Rejected = 2,
}

/// 违规记录
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxReasonLen))]
pub struct ViolationRecord<AccountId, BlockNumber, MaxReasonLen: Get<u32>> {
    /// 记录 ID
    pub id: u64,

    /// 提供者账户
    pub provider: AccountId,

    /// 违规类型
    pub violation_type: ViolationType,

    /// 违规原因描述
    pub reason: BoundedVec<u8, MaxReasonLen>,

    /// 关联订单 ID（如有）
    pub related_order_id: Option<u64>,

    /// 扣分数值
    pub deduction_points: u16,

    /// 处罚措施
    pub penalty: PenaltyType,

    /// 处罚期限（区块数，0表示永久）
    pub penalty_duration: u32,

    /// 是否已申诉
    pub is_appealed: bool,

    /// 申诉结果
    pub appeal_result: Option<AppealResult>,

    /// 记录时间
    pub recorded_at: BlockNumber,

    /// 过期时间（信用恢复点）
    pub expires_at: Option<BlockNumber>,

    /// 是否活跃（未过期）
    pub is_active: bool,
}

/// 信用变更原因
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum CreditChangeReason {
    /// 好评加分
    PositiveReview = 0,
    /// 差评扣分
    NegativeReview = 1,
    /// 完成订单
    OrderCompleted = 2,
    /// 取消订单
    OrderCancelled = 3,
    /// 超时未响应
    ResponseTimeout = 4,
    /// 悬赏被采纳
    BountyAdopted = 5,
    /// 获得认证
    CertificationGained = 6,
    /// 违规处罚
    ViolationPenalty = 7,
    /// 申诉成功恢复
    AppealRestored = 8,
    /// 信用修复
    CreditRepair = 9,
    /// 定期评估调整
    PeriodicAdjustment = 10,
    /// 系统奖励
    SystemBonus = 11,
    /// 连续好评奖励
    ConsecutiveBonus = 12,
}

/// 信用变更记录
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxReasonLen))]
pub struct CreditChangeRecord<BlockNumber, MaxReasonLen: Get<u32>> {
    /// 变更前分数
    pub previous_score: u16,

    /// 变更后分数
    pub new_score: u16,

    /// 变更值（正数加分，负数扣分）
    pub change_amount: i16,

    /// 变更原因
    pub reason: CreditChangeReason,

    /// 详细说明
    pub description: Option<BoundedVec<u8, MaxReasonLen>>,

    /// 关联 ID（订单/违规记录等）
    pub related_id: Option<u64>,

    /// 变更时间
    pub changed_at: BlockNumber,
}

/// 信用修复任务类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum RepairTaskType {
    /// 完成 N 个订单
    CompleteOrders = 0,
    /// 获得 N 个好评
    GetPositiveReviews = 1,
    /// 连续 N 天无投诉
    NoComplaintDays = 2,
    /// 缴纳额外保证金
    ExtraDeposit = 3,
    /// 完成培训课程
    CompleteTraining = 4,
    /// 通过认证考试
    PassCertification = 5,
}

impl RepairTaskType {
    /// 获取任务的默认奖励分数
    pub fn default_reward(&self) -> u16 {
        match self {
            RepairTaskType::CompleteOrders => 20,
            RepairTaskType::GetPositiveReviews => 30,
            RepairTaskType::NoComplaintDays => 25,
            RepairTaskType::ExtraDeposit => 50,
            RepairTaskType::CompleteTraining => 40,
            RepairTaskType::PassCertification => 60,
        }
    }

    /// 获取任务的默认目标值
    pub fn default_target(&self) -> u32 {
        match self {
            RepairTaskType::CompleteOrders => 5,
            RepairTaskType::GetPositiveReviews => 3,
            RepairTaskType::NoComplaintDays => 14,
            RepairTaskType::ExtraDeposit => 1,
            RepairTaskType::CompleteTraining => 1,
            RepairTaskType::PassCertification => 1,
        }
    }

    /// 获取任务期限（区块数）
    pub fn default_duration(&self) -> u32 {
        match self {
            RepairTaskType::CompleteOrders => 432000,     // 30天
            RepairTaskType::GetPositiveReviews => 432000, // 30天
            RepairTaskType::NoComplaintDays => 201600,    // 14天
            RepairTaskType::ExtraDeposit => 100800,       // 7天
            RepairTaskType::CompleteTraining => 201600,   // 14天
            RepairTaskType::PassCertification => 432000,  // 30天
        }
    }
}

/// 信用修复任务
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct CreditRepairTask<BlockNumber> {
    /// 任务 ID
    pub id: u32,

    /// 任务类型
    pub task_type: RepairTaskType,

    /// 完成后恢复的分数
    pub reward_points: u16,

    /// 任务目标值
    pub target_value: u32,

    /// 当前进度
    pub current_progress: u32,

    /// 是否已完成
    pub is_completed: bool,

    /// 任务开始时间
    pub started_at: BlockNumber,

    /// 任务截止时间
    pub deadline: BlockNumber,

    /// 完成时间
    pub completed_at: Option<BlockNumber>,
}

/// 全局信用统计
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct GlobalCreditStats {
    /// 总提供者数
    pub total_providers: u32,
    /// 卓越等级数量
    pub excellent_count: u32,
    /// 优秀等级数量
    pub good_count: u32,
    /// 一般等级数量
    pub fair_count: u32,
    /// 警示等级数量
    pub warning_count: u32,
    /// 不良等级数量
    pub poor_count: u32,
    /// 失信等级数量
    pub bad_count: u32,
    /// 黑名单数量
    pub blacklisted_count: u32,
    /// 平均信用分
    pub average_score: u16,
    /// 本周新增违规数
    pub weekly_violations: u32,
}

// 注：举报系统类型已迁移到统一仲裁模块 (pallet-arbitration)
// 使用 ComplaintType, ComplaintStatus, Complaint 等替代原有类型

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_tier_requirements() {
        assert_eq!(ProviderTier::Novice.min_orders(), 0);
        assert_eq!(ProviderTier::Expert.min_orders(), 200);
        assert_eq!(ProviderTier::Master.min_rating(), 480);
    }

    #[test]
    fn test_provider_tier_fees() {
        // 统一 10% 平台抽成
        assert_eq!(ProviderTier::Novice.platform_fee_rate(), 1000);
        assert_eq!(ProviderTier::Master.platform_fee_rate(), 1000);
    }

    #[test]
    fn test_service_type_duration() {
        assert_eq!(ServiceType::TextReading.base_duration(), 0);
        assert_eq!(ServiceType::LiveConsultation.base_duration(), 30);
    }

    #[test]
    fn test_reward_distribution_default() {
        let dist = RewardDistribution::default();
        assert!(dist.is_valid());
        assert_eq!(dist.first_place, 6000);
        assert_eq!(dist.second_place, 1500);
        assert_eq!(dist.third_place, 500);
        assert_eq!(dist.platform_fee, 1500);
        assert_eq!(dist.participation_pool, 500);
    }

    #[test]
    fn test_reward_distribution_invalid() {
        let dist = RewardDistribution {
            first_place: 7000,
            second_place: 1500,
            third_place: 500,
            platform_fee: 1500,
            participation_pool: 500,
        };
        assert!(!dist.is_valid()); // 总和 11000 != 10000
    }

    #[test]
    fn test_credit_level_from_score() {
        assert_eq!(CreditLevel::from_score(0), CreditLevel::Bad);
        assert_eq!(CreditLevel::from_score(199), CreditLevel::Bad);
        assert_eq!(CreditLevel::from_score(200), CreditLevel::Poor);
        assert_eq!(CreditLevel::from_score(400), CreditLevel::Warning);
        assert_eq!(CreditLevel::from_score(600), CreditLevel::Fair);
        assert_eq!(CreditLevel::from_score(750), CreditLevel::Good);
        assert_eq!(CreditLevel::from_score(900), CreditLevel::Excellent);
        assert_eq!(CreditLevel::from_score(1000), CreditLevel::Excellent);
    }

    #[test]
    fn test_credit_level_permissions() {
        assert!(!CreditLevel::Bad.can_accept_orders());
        assert!(CreditLevel::Poor.can_accept_orders());

        assert!(!CreditLevel::Bad.can_create_packages());
        assert!(!CreditLevel::Poor.can_create_packages());
        assert!(CreditLevel::Warning.can_create_packages());

        assert!(!CreditLevel::Warning.can_answer_bounties());
        assert!(CreditLevel::Fair.can_answer_bounties());
    }

    #[test]
    fn test_violation_type_duration() {
        assert_eq!(ViolationType::Minor.record_duration(), 432000);
        assert_eq!(ViolationType::Critical.record_duration(), 5256000);
    }

    #[test]
    fn test_deduction_reason_values() {
        assert_eq!(DeductionReason::NegativeReview.default_deduction(), 5);
        assert_eq!(DeductionReason::Fraud.default_deduction(), 200);
    }

    #[test]
    fn test_repair_task_defaults() {
        assert_eq!(RepairTaskType::CompleteOrders.default_reward(), 20);
        assert_eq!(RepairTaskType::CompleteOrders.default_target(), 5);
    }
}
