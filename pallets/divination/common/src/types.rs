//! # 玄学公共模块类型定义
//!
//! 本模块定义了所有玄学系统（梅花易数、八字排盘、六爻等）共用的类型。
//!
//! ## 核心类型
//!
//! - `DivinationType` - 占卜类型枚举
//! - `Rarity` - NFT 稀有度等级
//! - `RarityInput` - 稀有度计算输入数据
//! - `InterpretationType` - AI 解读类型
//! - `InterpretationStatus` - 解读请求状态
//! - `ServiceType` - 服务市场服务类型
//! - `OrderStatus` - 订单状态

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// 占卜类型枚举
///
/// 标识不同的玄学系统，用于公共模块区分数据来源。
///
/// # 扩展说明
/// 新增玄学系统时，在此枚举中添加对应类型。
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default, Hash)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum DivinationType {
    /// 梅花易数 - 基于时间、数字、文字起卦的占卜系统
    #[default]
    Meihua = 0,
    /// 八字排盘 - 基于出生时间的命理系统
    Bazi = 1,
    /// 六爻占卜 - 基于摇卦的占卜系统
    Liuyao = 2,
    /// 奇门遁甲 - 时空方位预测系统
    Qimen = 3,
    /// 紫微斗数 - 星命学系统
    Ziwei = 4,
    /// 太乙神数 - 古代预测术（预留）
    Taiyi = 5,
    /// 大六壬 - 时空预测系统
    Daliuren = 6,
    /// 小六壬 - 掐指速算占卜系统
    XiaoLiuRen = 7,
    /// 塔罗牌 - 西方占卜系统
    Tarot = 8,
}

impl DivinationType {
    /// 获取占卜类型的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Meihua => "梅花易数",
            Self::Bazi => "八字排盘",
            Self::Liuyao => "六爻占卜",
            Self::Qimen => "奇门遁甲",
            Self::Ziwei => "紫微斗数",
            Self::Taiyi => "太乙神数",
            Self::Daliuren => "大六壬",
            Self::XiaoLiuRen => "小六壬",
            Self::Tarot => "塔罗牌",
        }
    }

    /// 检查该类型是否已实现
    ///
    /// 已实现的占卜系统：
    /// - 梅花易数 (Meihua)
    /// - 八字命理 (Bazi)
    /// - 六爻占卜 (Liuyao)
    /// - 奇门遁甲 (Qimen)
    /// - 紫微斗数 (Ziwei)
    /// - 大六壬 (Daliuren)
    /// - 小六壬 (XiaoLiuRen)
    /// - 塔罗牌 (Tarot)
    ///
    /// 未实现（预留）：
    /// - 太乙神数 (Taiyi)
    pub fn is_implemented(&self) -> bool {
        matches!(
            self,
            Self::Meihua
                | Self::Bazi
                | Self::Liuyao
                | Self::Qimen
                | Self::Ziwei
                | Self::Daliuren
                | Self::XiaoLiuRen
                | Self::Tarot
        )
    }

    /// 获取所有已实现的占卜类型
    pub fn implemented_types() -> sp_std::vec::Vec<Self> {
        sp_std::vec![
            Self::Meihua,
            Self::Bazi,
            Self::Liuyao,
            Self::Qimen,
            Self::Ziwei,
            Self::Daliuren,
            Self::XiaoLiuRen,
            Self::Tarot,
        ]
    }
}

/// NFT 稀有度等级
///
/// 基于占卜结果的特征自动判定稀有度。
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum Rarity {
    /// 普通 - 常见结果
    #[default]
    Common = 0,
    /// 稀有 - 特殊组合或日期
    Rare = 1,
    /// 史诗 - 罕见组合
    Epic = 2,
    /// 传说 - 极其罕见
    Legendary = 3,
}

impl Rarity {
    /// 获取稀有度的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Common => "普通",
            Self::Rare => "稀有",
            Self::Epic => "史诗",
            Self::Legendary => "传说",
        }
    }

    /// 获取稀有度对应的颜色代码（前端使用）
    pub fn color(&self) -> &'static str {
        match self {
            Self::Common => "#8c8c8c",
            Self::Rare => "#1890ff",
            Self::Epic => "#722ed1",
            Self::Legendary => "#faad14",
        }
    }

    /// 获取铸造费用倍数
    pub fn fee_multiplier(&self) -> u32 {
        match self {
            Self::Common => 100,    // 1x
            Self::Rare => 150,      // 1.5x
            Self::Epic => 300,      // 3x
            Self::Legendary => 1000, // 10x
        }
    }
}

/// 稀有度计算输入数据
///
/// 各玄学系统将自身特征转换为此结构，由统一算法计算稀有度。
///
/// # 计算规则
/// - primary_score * 3 + secondary_score * 2 + 特殊加成
/// - 0-100: Common, 101-200: Rare, 201-350: Epic, 350+: Legendary
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct RarityInput {
    /// 主要因素权重 (0-100)
    ///
    /// 示例：
    /// - 梅花：纯卦=80, 普通=30
    /// - 八字：日主极强/极弱=80, 普通=40
    pub primary_score: u8,

    /// 次要因素权重 (0-100)
    ///
    /// 示例：
    /// - 梅花：动爻初/上=20, 其他=10
    /// - 八字：特殊格局=50, 普通=20
    pub secondary_score: u8,

    /// 是否特殊日期
    ///
    /// 示例：节气、传统节日、特殊天文现象
    pub is_special_date: bool,

    /// 是否特殊组合
    ///
    /// 示例：
    /// - 梅花：六冲六合、纯卦
    /// - 八字：三奇、天干一字、地支一气
    pub is_special_combination: bool,

    /// 自定义稀有度因子 (0-255 each)
    ///
    /// 各系统可自定义额外加分项：
    /// - 索引0: 时间因子（如特定时辰）
    /// - 索引1: 空间因子（如特定方位）
    /// - 索引2: 数字因子（如吉祥数字组合）
    /// - 索引3: 预留
    pub custom_factors: [u8; 4],
}

impl RarityInput {
    /// 创建默认（普通）稀有度输入
    pub fn common() -> Self {
        Self {
            primary_score: 30,
            secondary_score: 10,
            is_special_date: false,
            is_special_combination: false,
            custom_factors: [0, 0, 0, 0],
        }
    }

    /// 计算稀有度等级
    ///
    /// # 算法
    /// 1. primary_score * 3（主要因素权重高）
    /// 2. secondary_score * 2
    /// 3. 特殊日期 +50
    /// 4. 特殊组合 +100
    /// 5. 自定义因子累加
    pub fn calculate_rarity(&self) -> Rarity {
        let mut score = 0u16;

        // 主要因素权重 (max: 100 * 3 = 300)
        score += (self.primary_score as u16).saturating_mul(3);

        // 次要因素权重 (max: 100 * 2 = 200)
        score += (self.secondary_score as u16).saturating_mul(2);

        // 特殊日期加成 (+50)
        if self.is_special_date {
            score = score.saturating_add(50);
        }

        // 特殊组合加成 (+100)
        if self.is_special_combination {
            score = score.saturating_add(100);
        }

        // 自定义因子累加 (max: 255 * 4 = 1020)
        for factor in self.custom_factors.iter() {
            score = score.saturating_add(*factor as u16);
        }

        // 根据分数判定等级
        match score {
            0..=100 => Rarity::Common,
            101..=200 => Rarity::Rare,
            201..=350 => Rarity::Epic,
            _ => Rarity::Legendary,
        }
    }

    /// 获取原始分数（用于调试）
    pub fn raw_score(&self) -> u16 {
        let mut score = 0u16;
        score += (self.primary_score as u16).saturating_mul(3);
        score += (self.secondary_score as u16).saturating_mul(2);
        if self.is_special_date {
            score = score.saturating_add(50);
        }
        if self.is_special_combination {
            score = score.saturating_add(100);
        }
        for factor in self.custom_factors.iter() {
            score = score.saturating_add(*factor as u16);
        }
        score
    }
}

/// AI 解读类型
///
/// 定义不同深度和方向的 AI 解读服务。
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum InterpretationType {
    /// 基础解读 - 简要说明卦象/命盘含义
    #[default]
    Basic = 0,
    /// 详细解读 - 深入分析各方面
    Detailed = 1,
    /// 专业解读 - 专家级深度分析
    Professional = 2,
    /// 事业解读 - 专注事业发展
    Career = 3,
    /// 感情解读 - 专注感情婚姻
    Relationship = 4,
    /// 健康解读 - 专注健康运势
    Health = 5,
    /// 财运解读 - 专注财运投资
    Wealth = 6,
    /// 学业解读 - 专注学业考试
    Education = 7,
    /// 流年解读 - 年度运势分析
    Annual = 8,
}

impl InterpretationType {
    /// 获取解读类型的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Basic => "基础解读",
            Self::Detailed => "详细解读",
            Self::Professional => "专业解读",
            Self::Career => "事业解读",
            Self::Relationship => "感情解读",
            Self::Health => "健康解读",
            Self::Wealth => "财运解读",
            Self::Education => "学业解读",
            Self::Annual => "流年解读",
        }
    }

    /// 获取费用倍数（基础解读为 100 = 1x）
    pub fn fee_multiplier(&self) -> u32 {
        match self {
            Self::Basic => 100,        // 1x
            Self::Detailed => 200,     // 2x
            Self::Professional => 500, // 5x
            Self::Career => 150,       // 1.5x
            Self::Relationship => 150, // 1.5x
            Self::Health => 150,       // 1.5x
            Self::Wealth => 150,       // 1.5x
            Self::Education => 150,    // 1.5x
            Self::Annual => 300,       // 3x
        }
    }

    /// 检查该解读类型是否适用于指定的占卜类型
    pub fn is_applicable_to(&self, divination_type: DivinationType) -> bool {
        match divination_type {
            DivinationType::Meihua => true, // 梅花支持所有解读类型
            DivinationType::Bazi => true,   // 八字支持所有解读类型
            _ => matches!(self, Self::Basic | Self::Detailed), // 其他仅支持基础和详细
        }
    }
}

/// 解读请求状态
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum InterpretationStatus {
    /// 等待处理 - 请求已提交，等待预言机接单
    #[default]
    Pending = 0,
    /// 处理中 - 预言机已接单，正在生成解读
    Processing = 1,
    /// 已完成 - 解读已生成并提交
    Completed = 2,
    /// 已失败 - 解读生成失败
    Failed = 3,
    /// 已过期 - 请求超时未处理
    Expired = 4,
    /// 争议中 - 用户对结果提出争议
    Disputed = 5,
    /// 已退款 - 争议成立或超时，已退款
    Refunded = 6,
}

impl InterpretationStatus {
    /// 获取状态的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pending => "等待处理",
            Self::Processing => "处理中",
            Self::Completed => "已完成",
            Self::Failed => "已失败",
            Self::Expired => "已过期",
            Self::Disputed => "争议中",
            Self::Refunded => "已退款",
        }
    }

    /// 检查是否为终态
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Expired | Self::Refunded
        )
    }

    /// 检查是否可取消
    pub fn is_cancellable(&self) -> bool {
        matches!(self, Self::Pending)
    }
}

/// 服务市场服务类型
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum ServiceType {
    /// 文字解读 - 异步文字回复
    #[default]
    TextReading = 0,
    /// 语音解读 - 录制语音回复
    VoiceReading = 1,
    /// 视频解读 - 录制视频回复
    VideoReading = 2,
    /// 实时咨询 - 在线实时沟通
    LiveConsultation = 3,
}

impl ServiceType {
    /// 获取服务类型的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::TextReading => "文字解读",
            Self::VoiceReading => "语音解读",
            Self::VideoReading => "视频解读",
            Self::LiveConsultation => "实时咨询",
        }
    }

    /// 获取建议的最低价格倍数（基础为 100）
    pub fn min_price_multiplier(&self) -> u32 {
        match self {
            Self::TextReading => 100,       // 1x
            Self::VoiceReading => 200,      // 2x
            Self::VideoReading => 300,      // 3x
            Self::LiveConsultation => 500,  // 5x
        }
    }
}

/// 订单状态
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum OrderStatus {
    /// 待支付 - 订单已创建，等待买家支付
    #[default]
    PendingPayment = 0,
    /// 已支付 - 买家已支付，等待服务商接单
    Paid = 1,
    /// 已接单 - 服务商已接单，正在服务
    Accepted = 2,
    /// 已完成 - 服务已完成，等待评价
    Completed = 3,
    /// 已评价 - 买家已评价
    Reviewed = 4,
    /// 已取消 - 订单已取消
    Cancelled = 5,
    /// 已退款 - 订单已退款
    Refunded = 6,
    /// 争议中 - 订单存在争议
    Disputed = 7,
}

impl OrderStatus {
    /// 获取订单状态的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::PendingPayment => "待支付",
            Self::Paid => "已支付",
            Self::Accepted => "已接单",
            Self::Completed => "已完成",
            Self::Reviewed => "已评价",
            Self::Cancelled => "已取消",
            Self::Refunded => "已退款",
            Self::Disputed => "争议中",
        }
    }

    /// 检查是否为终态
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            Self::Reviewed | Self::Cancelled | Self::Refunded
        )
    }

    /// 检查是否可取消
    pub fn is_cancellable(&self) -> bool {
        matches!(self, Self::PendingPayment | Self::Paid)
    }

    /// 检查是否可退款
    pub fn is_refundable(&self) -> bool {
        matches!(self, Self::Paid | Self::Accepted | Self::Disputed)
    }
}

/// 服务提供者等级
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum ProviderTier {
    /// 新手 - 刚注册
    #[default]
    Novice = 0,
    /// 认证 - 通过身份认证
    Certified = 1,
    /// 资深 - 完成一定数量订单
    Senior = 2,
    /// 专家 - 高评分高完成率
    Expert = 3,
    /// 大师 - 顶级服务商
    Master = 4,
}

impl ProviderTier {
    /// 获取等级的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Novice => "新手",
            Self::Certified => "认证",
            Self::Senior => "资深",
            Self::Expert => "专家",
            Self::Master => "大师",
        }
    }

    /// 获取该等级所需的最低押金倍数（基础为 100）
    pub fn min_deposit_multiplier(&self) -> u32 {
        match self {
            Self::Novice => 100,     // 1x
            Self::Certified => 100,  // 1x
            Self::Senior => 150,     // 1.5x
            Self::Expert => 200,     // 2x
            Self::Master => 300,     // 3x
        }
    }

    /// 获取该等级的平台费率折扣（基点，10000 = 100%）
    pub fn platform_fee_discount(&self) -> u32 {
        match self {
            Self::Novice => 10000,    // 100% (无折扣)
            Self::Certified => 9500,  // 95%
            Self::Senior => 9000,     // 90%
            Self::Expert => 8000,     // 80%
            Self::Master => 7000,     // 70%
        }
    }
}

/// 争议状态
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum DisputeStatus {
    /// 待处理 - 争议已提交
    #[default]
    Pending = 0,
    /// 调查中 - 仲裁员正在调查
    Investigating = 1,
    /// 已裁决 - 仲裁完成
    Resolved = 2,
    /// 已撤销 - 申请人撤销争议
    Withdrawn = 3,
}

impl DisputeStatus {
    /// 获取争议状态的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pending => "待处理",
            Self::Investigating => "调查中",
            Self::Resolved => "已裁决",
            Self::Withdrawn => "已撤销",
        }
    }
}

/// 争议裁决结果
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum DisputeResolution {
    /// 支持申请人 - 全额退款
    InFavorOfApplicant,
    /// 支持被申请人 - 不退款
    InFavorOfRespondent,
    /// 部分支持 - 部分退款
    PartialRefund { refund_percent: u8 },
    /// 双方和解 - 按协商处理
    Settled,
}

/// 悬赏问答状态
///
/// 基于占卜结果发起的解读悬赏的状态流转
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum BountyStatus {
    /// 开放中 - 接受解读提交
    #[default]
    Open = 0,
    /// 已关闭 - 停止接受解读，等待采纳
    Closed = 1,
    /// 已采纳 - 已选择前三名，等待结算
    Adopted = 2,
    /// 已结算 - 奖励已分配完成（终态）
    Settled = 3,
    /// 已取消 - 提问者取消（无回答时）（终态）
    Cancelled = 4,
    /// 已过期 - 超时且无回答，已退款（终态）
    Expired = 5,
}

impl BountyStatus {
    /// 获取状态的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Open => "开放中",
            Self::Closed => "已关闭",
            Self::Adopted => "已采纳",
            Self::Settled => "已结算",
            Self::Cancelled => "已取消",
            Self::Expired => "已过期",
        }
    }

    /// 检查是否为终态
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Settled | Self::Cancelled | Self::Expired)
    }

    /// 检查是否可以接受新的解读
    pub fn is_accepting_answers(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// 检查是否可以采纳解读
    pub fn is_adoptable(&self) -> bool {
        matches!(self, Self::Open | Self::Closed)
    }

    /// 检查是否可以结算
    pub fn is_settleable(&self) -> bool {
        matches!(self, Self::Adopted)
    }
}

/// 悬赏解读回答状态
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum BountyAnswerStatus {
    /// 等待中 - 等待采纳/投票
    #[default]
    Pending = 0,
    /// 已采纳 - 被采纳为最佳答案（第一名）
    Adopted = 1,
    /// 已入选 - 入选优秀答案（第二/三名）
    Selected = 2,
    /// 参与奖 - 获得参与奖励
    Participated = 3,
    /// 未入选 - 未获得任何奖励
    Rejected = 4,
}

impl BountyAnswerStatus {
    /// 获取状态的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pending => "等待中",
            Self::Adopted => "已采纳",
            Self::Selected => "已入选",
            Self::Participated => "参与奖",
            Self::Rejected => "未入选",
        }
    }

    /// 检查是否获得了奖励
    pub fn has_reward(&self) -> bool {
        matches!(self, Self::Adopted | Self::Selected | Self::Participated)
    }
}

/// 擅长领域（用于匹配回答者）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum Specialty {
    /// 事业
    Career = 0,
    /// 感情
    Relationship = 1,
    /// 健康
    Health = 2,
    /// 财运
    Wealth = 3,
    /// 学业
    Education = 4,
    /// 流年运势
    Annual = 5,
    /// 综合
    General = 6,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rarity_common() {
        let input = RarityInput {
            primary_score: 20,
            secondary_score: 10,
            is_special_date: false,
            is_special_combination: false,
            custom_factors: [0, 0, 0, 0],
        };
        // 20*3 + 10*2 = 60 + 20 = 80 → Common
        assert_eq!(input.calculate_rarity(), Rarity::Common);
    }

    #[test]
    fn test_rarity_rare() {
        let input = RarityInput {
            primary_score: 40,
            secondary_score: 20,
            is_special_date: false,
            is_special_combination: false,
            custom_factors: [0, 0, 0, 0],
        };
        // 40*3 + 20*2 = 120 + 40 = 160 → Rare
        assert_eq!(input.calculate_rarity(), Rarity::Rare);
    }

    #[test]
    fn test_rarity_epic() {
        let input = RarityInput {
            primary_score: 50,
            secondary_score: 30,
            is_special_date: true,
            is_special_combination: false,
            custom_factors: [0, 0, 0, 0],
        };
        // 50*3 + 30*2 + 50 = 150 + 60 + 50 = 260 → Epic
        assert_eq!(input.calculate_rarity(), Rarity::Epic);
    }

    #[test]
    fn test_rarity_legendary() {
        let input = RarityInput {
            primary_score: 80,
            secondary_score: 50,
            is_special_date: true,
            is_special_combination: true,
            custom_factors: [10, 10, 0, 0],
        };
        // 80*3 + 50*2 + 50 + 100 + 20 = 240 + 100 + 50 + 100 + 20 = 510 → Legendary
        assert_eq!(input.calculate_rarity(), Rarity::Legendary);
    }

    #[test]
    fn test_rarity_special_combination_bonus() {
        let input = RarityInput {
            primary_score: 30,
            secondary_score: 10,
            is_special_date: false,
            is_special_combination: true,
            custom_factors: [0, 0, 0, 0],
        };
        // 30*3 + 10*2 + 100 = 90 + 20 + 100 = 210 → Epic
        assert_eq!(input.calculate_rarity(), Rarity::Epic);
    }

    #[test]
    fn test_divination_type_implemented() {
        // 已实现的占卜类型
        assert!(DivinationType::Meihua.is_implemented());
        assert!(DivinationType::Bazi.is_implemented());
        assert!(DivinationType::Liuyao.is_implemented());
        assert!(DivinationType::Qimen.is_implemented());
        assert!(DivinationType::Ziwei.is_implemented());
        assert!(DivinationType::Daliuren.is_implemented());
        assert!(DivinationType::XiaoLiuRen.is_implemented());
        assert!(DivinationType::Tarot.is_implemented());

        // 未实现的占卜类型
        assert!(!DivinationType::Taiyi.is_implemented());

        // 验证 implemented_types 返回正确数量
        assert_eq!(DivinationType::implemented_types().len(), 8);
    }

    #[test]
    fn test_interpretation_status_final() {
        assert!(!InterpretationStatus::Pending.is_final());
        assert!(!InterpretationStatus::Processing.is_final());
        assert!(InterpretationStatus::Completed.is_final());
        assert!(InterpretationStatus::Failed.is_final());
        assert!(InterpretationStatus::Expired.is_final());
        assert!(InterpretationStatus::Refunded.is_final());
    }

    #[test]
    fn test_order_status_cancellable() {
        assert!(OrderStatus::PendingPayment.is_cancellable());
        assert!(OrderStatus::Paid.is_cancellable());
        assert!(!OrderStatus::Accepted.is_cancellable());
        assert!(!OrderStatus::Completed.is_cancellable());
    }

    #[test]
    fn test_provider_tier_ordering() {
        assert!(ProviderTier::Novice < ProviderTier::Certified);
        assert!(ProviderTier::Certified < ProviderTier::Senior);
        assert!(ProviderTier::Senior < ProviderTier::Expert);
        assert!(ProviderTier::Expert < ProviderTier::Master);
    }

    #[test]
    fn test_rarity_fee_multiplier() {
        assert_eq!(Rarity::Common.fee_multiplier(), 100);
        assert_eq!(Rarity::Rare.fee_multiplier(), 150);
        assert_eq!(Rarity::Epic.fee_multiplier(), 300);
        assert_eq!(Rarity::Legendary.fee_multiplier(), 1000);
    }
}
