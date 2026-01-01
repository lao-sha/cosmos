use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use super::enums::*;
use crate::types::{BaGua, TiYongRelation, WuXingRelation, XiaoLiuRenSchool};

/// 小六壬解卦核心数据（13 bytes）
///
/// 存储核心指标，链上永久保存
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct XiaoLiuRenInterpretation {
    /// 吉凶等级（1 byte）
    pub ji_xiong_level: JiXiongLevel,

    /// 综合评分（1 byte，0-100分）
    pub overall_score: u8,

    /// 三宫五行关系（1 byte）
    pub wu_xing_relation: WuXingRelation,

    /// 体用关系（可选，1+1 bytes = 2 bytes）
    pub ti_yong_relation: Option<TiYongRelation>,

    /// 八卦索引（可选，1+1 bytes = 2 bytes）
    pub ba_gua: Option<BaGua>,

    /// 特殊格局标记（1 byte）
    pub special_pattern: SpecialPattern,

    /// 建议类型（1 byte）
    pub advice_type: AdviceType,

    /// 流派（1 byte）
    pub school: XiaoLiuRenSchool,

    /// 应期类型（可选，1+1 bytes = 2 bytes）
    pub ying_qi: Option<YingQiType>,

    /// 预留字段（1 byte）
    pub reserved: u8,
}

impl XiaoLiuRenInterpretation {
    /// 创建新的解卦结果
    pub fn new(
        ji_xiong_level: JiXiongLevel,
        overall_score: u8,
        wu_xing_relation: WuXingRelation,
        ti_yong_relation: Option<TiYongRelation>,
        ba_gua: Option<BaGua>,
        special_pattern: SpecialPattern,
        advice_type: AdviceType,
        school: XiaoLiuRenSchool,
        ying_qi: Option<YingQiType>,
    ) -> Self {
        Self {
            ji_xiong_level,
            overall_score,
            wu_xing_relation,
            ti_yong_relation,
            ba_gua,
            special_pattern,
            advice_type,
            school,
            ying_qi,
            reserved: 0,
        }
    }

    /// 判断是否为吉
    pub fn is_ji(&self) -> bool {
        self.ji_xiong_level.is_ji()
    }

    /// 判断是否为凶
    pub fn is_xiong(&self) -> bool {
        self.ji_xiong_level.is_xiong()
    }

    /// 获取吉凶等级分数（1-7）
    pub fn ji_xiong_score(&self) -> u8 {
        self.ji_xiong_level.score()
    }

    /// 是否有特殊格局
    pub fn has_special_pattern(&self) -> bool {
        self.special_pattern.has_any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size() {
        use core::mem::size_of;
        // 验证大小（Option被优化，实际大小可能小于理论值）
        let size = size_of::<XiaoLiuRenInterpretation>();
        println!("XiaoLiuRenInterpretation size: {} bytes", size);
        // 实际大小通常是10字节（由于Option的null pointer优化）
        assert!(size <= 16, "Size should be reasonable, got {}", size);
    }

    #[test]
    fn test_max_encoded_len() {
        use codec::MaxEncodedLen;
        // 验证编码后的最大长度
        let max_len = XiaoLiuRenInterpretation::max_encoded_len();
        println!("XiaoLiuRenInterpretation max_encoded_len: {}", max_len);
        // 编码后的最大长度应该在合理范围内
        assert!(max_len <= 30, "Max encoded len should be reasonable, got {}", max_len);
    }

    #[test]
    fn test_new() {
        let interp = XiaoLiuRenInterpretation::new(
            JiXiongLevel::DaJi,
            85,
            WuXingRelation::Sheng,
            None,
            None,
            SpecialPattern::new(),
            AdviceType::JinQu,
            XiaoLiuRenSchool::DaoJia,
            None,
        );

        assert!(interp.is_ji());
        assert!(!interp.is_xiong());
        assert_eq!(interp.overall_score, 85);
        assert!(!interp.has_special_pattern());
    }

    #[test]
    fn test_with_special_pattern() {
        let mut pattern = SpecialPattern::new();
        pattern.set_pure();

        let interp = XiaoLiuRenInterpretation::new(
            JiXiongLevel::DaJi,
            90,
            WuXingRelation::BiHe,
            None,
            None,
            pattern,
            AdviceType::JinQu,
            XiaoLiuRenSchool::DaoJia,
            None,
        );

        assert!(interp.has_special_pattern());
        assert!(interp.special_pattern.is_pure());
    }
}
