//! 梅花易数解卦数据结构模块
//!
//! 本模块定义了梅花易数解卦所需的所有数据结构，包括：
//! - 解卦基础信息
//! - 体用分析结果
//! - 应期推算结果
//! - 辅助卦象数据
//! - AI解读数据结构
//!
//! ## 设计原则
//!
//! 1. **链上存储最小化**：仅存储核心数据，可推导数据通过 Runtime API 计算
//! 2. **隐私保护**：敏感信息（问题、姓名）仅存储哈希值
//! 3. **可扩展性**：预留扩展字段，支持未来功能升级
//! 4. **AI友好**：结构化数据便于AI解读和分析
//! 5. **前端友好**：提供完整的查询API，减少前端计算负担

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;

use crate::types::*;

// ==================== 基础信息结构 ====================

/// 农历日期信息（精简版）
///
/// 用于解卦时的时间上下文，影响五行旺衰判断
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct LunarDateInfo {
    /// 农历年份
    pub year: u16,
    /// 农历月份（1-12）
    pub month: u8,
    /// 农历日（1-30）
    pub day: u8,
    /// 时辰地支数（1-12）
    pub hour_zhi_num: u8,
    /// 是否闰月
    pub is_leap_month: bool,
}

/// 解卦基础信息
///
/// 存储占卜的基础上下文信息，用于AI解读和人工分析
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct InterpretationBasicInfo {
    /// 占卜时间戳（Unix秒）
    pub timestamp: u64,

    /// 农历年月日时（用于旺衰判断）
    pub lunar_date: LunarDateInfo,

    /// 起卦方式
    pub method: DivinationMethod,

    /// 占卜者性别（可选，用于某些流派的解卦）
    /// 0: 未指定, 1: 男, 2: 女
    pub gender: u8,

    /// 占卜类别（可选）
    /// 0: 未指定, 1: 事业, 2: 财运, 3: 感情, 4: 健康, 5: 学业, 6: 其他
    pub category: u8,
}

impl Default for InterpretationBasicInfo {
    fn default() -> Self {
        Self {
            timestamp: 0,
            lunar_date: LunarDateInfo::default(),
            method: DivinationMethod::default(),
            gender: 0,
            category: 0,
        }
    }
}

// ==================== 卦象核心数据 ====================

/// 卦象核心数据
///
/// 存储排盘的核心结果，所有其他信息可从此推导
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct HexagramCoreData {
    /// 上卦（外卦）
    pub shang_gua: SingleGua,

    /// 下卦（内卦）
    pub xia_gua: SingleGua,

    /// 动爻位置（1-6）
    pub dong_yao: u8,

    /// 体卦位置：true=上卦为体，false=下卦为体
    pub ti_is_shang: bool,
}

impl Default for HexagramCoreData {
    fn default() -> Self {
        Self {
            shang_gua: SingleGua::default(),
            xia_gua: SingleGua::default(),
            dong_yao: 1,
            ti_is_shang: true,
        }
    }
}

impl HexagramCoreData {
    /// 获取体卦
    pub fn ti_gua(&self) -> &SingleGua {
        if self.ti_is_shang {
            &self.shang_gua
        } else {
            &self.xia_gua
        }
    }

    /// 获取用卦
    pub fn yong_gua(&self) -> &SingleGua {
        if self.ti_is_shang {
            &self.xia_gua
        } else {
            &self.shang_gua
        }
    }
}

// ==================== 体用分析结果 ====================

/// 体用分析结果
///
/// 梅花易数核心：体用关系决定吉凶
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct TiYongAnalysis {
    /// 体卦五行
    pub ti_wuxing: WuXing,

    /// 用卦五行
    pub yong_wuxing: WuXing,

    /// 本卦体用关系
    pub ben_gua_relation: TiYongRelation,

    /// 变卦体用关系
    pub bian_gua_relation: TiYongRelation,

    /// 互卦体用关系
    pub hu_gua_relation: TiYongRelation,

    /// 体卦旺衰状态
    pub ti_wangshuai: WangShuai,

    /// 综合吉凶判断
    pub fortune: Fortune,

    /// 吉凶等级（0-4，4最吉）
    pub fortune_level: u8,
}

impl Default for TiYongAnalysis {
    fn default() -> Self {
        Self {
            ti_wuxing: WuXing::default(),
            yong_wuxing: WuXing::default(),
            ben_gua_relation: TiYongRelation::default(),
            bian_gua_relation: TiYongRelation::default(),
            hu_gua_relation: TiYongRelation::default(),
            ti_wangshuai: WangShuai::default(),
            fortune: Fortune::default(),
            fortune_level: 2,
        }
    }
}

// ==================== 应期推算结果 ====================

/// 应期推算结果
///
/// 预测事情应验的时间
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct YingQiAnalysis {
    /// 体卦卦数
    pub ti_gua_num: u8,

    /// 用卦卦数
    pub yong_gua_num: u8,

    /// 主要应期数（基于体用卦数）
    pub primary_num: u8,

    /// 次要应期数（基于五行卦数）
    pub secondary_nums: [u8; 2],

    /// 生体五行（喜神）
    pub sheng_ti_wuxing: WuXing,

    /// 克体五行（忌神）
    pub ke_ti_wuxing: WuXing,

    /// 应期分析文本（简短）
    pub analysis: BoundedVec<u8, ConstU32<256>>,
}

impl Default for YingQiAnalysis {
    fn default() -> Self {
        Self {
            ti_gua_num: 1,
            yong_gua_num: 1,
            primary_num: 1,
            secondary_nums: [1, 1],
            sheng_ti_wuxing: WuXing::default(),
            ke_ti_wuxing: WuXing::default(),
            analysis: BoundedVec::default(),
        }
    }
}

// ==================== 辅助卦象数据 ====================

/// 辅助卦象数据
///
/// 变卦、互卦、错卦、综卦、伏卦
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct AuxiliaryHexagrams {
    /// 变卦（上卦，下卦）
    pub bian_gua: (SingleGua, SingleGua),

    /// 互卦（上卦，下卦）
    pub hu_gua: (SingleGua, SingleGua),

    /// 错卦（上卦，下卦）
    pub cuo_gua: (SingleGua, SingleGua),

    /// 综卦（上卦，下卦）
    pub zong_gua: (SingleGua, SingleGua),

    /// 伏卦（上卦，下卦）
    pub fu_gua: (SingleGua, SingleGua),
}

impl Default for AuxiliaryHexagrams {
    fn default() -> Self {
        Self {
            bian_gua: (SingleGua::default(), SingleGua::default()),
            hu_gua: (SingleGua::default(), SingleGua::default()),
            cuo_gua: (SingleGua::default(), SingleGua::default()),
            zong_gua: (SingleGua::default(), SingleGua::default()),
            fu_gua: (SingleGua::default(), SingleGua::default()),
        }
    }
}

// ==================== 完整解卦数据 ====================

/// 完整解卦数据
///
/// 包含所有解卦所需的信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct InterpretationData {
    /// 基础信息
    pub basic_info: InterpretationBasicInfo,

    /// 卦象核心数据
    pub hexagram_core: HexagramCoreData,

    /// 体用分析
    pub tiyong_analysis: TiYongAnalysis,

    /// 应期推算
    pub yingqi_analysis: YingQiAnalysis,

    /// 辅助卦象
    pub auxiliary_hexagrams: AuxiliaryHexagrams,
}

impl Default for InterpretationData {
    fn default() -> Self {
        Self {
            basic_info: InterpretationBasicInfo::default(),
            hexagram_core: HexagramCoreData::default(),
            tiyong_analysis: TiYongAnalysis::default(),
            yingqi_analysis: YingQiAnalysis::default(),
            auxiliary_hexagrams: AuxiliaryHexagrams::default(),
        }
    }
}

// ==================== 详细解读数据（用于前端展示） ====================

/// 单卦详细信息
///
/// 包含单个卦的所有文本信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct SingleGuaDetail {
    /// 卦名（如"乾"）
    pub name: BoundedVec<u8, ConstU32<16>>,

    /// 卦象符号（如"☰"）
    pub symbol: BoundedVec<u8, ConstU32<8>>,

    /// 五行（如"金"）
    pub wuxing: BoundedVec<u8, ConstU32<8>>,

    /// 卦数（1-8）
    pub number: u8,

    /// 二进制表示（3 bits）
    pub binary: u8,

    /// 卦象含义（如"天"、"泽"）
    pub meaning: BoundedVec<u8, ConstU32<32>>,
}

impl Default for SingleGuaDetail {
    fn default() -> Self {
        Self {
            name: BoundedVec::default(),
            symbol: BoundedVec::default(),
            wuxing: BoundedVec::default(),
            number: 1,
            binary: 0,
            meaning: BoundedVec::default(),
        }
    }
}

/// 六十四卦详细信息
///
/// 包含完整的卦辞、爻辞等文本信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct HexagramFullDetail {
    /// 六十四卦名称（如"乾为天"）
    pub name: BoundedVec<u8, ConstU32<32>>,

    /// 上卦详细信息
    pub shang_gua: SingleGuaDetail,

    /// 下卦详细信息
    pub xia_gua: SingleGuaDetail,

    /// 卦辞
    pub guaci: BoundedVec<u8, ConstU32<256>>,

    /// 动爻名称（如"初爻"）
    pub dong_yao_name: BoundedVec<u8, ConstU32<16>>,

    /// 动爻爻名（如"初九"、"六二"）
    pub dong_yao_ming: BoundedVec<u8, ConstU32<16>>,

    /// 动爻爻辞
    pub dong_yao_ci: BoundedVec<u8, ConstU32<256>>,

    /// 体用关系名称（如"用生体"）
    pub tiyong_name: BoundedVec<u8, ConstU32<16>>,

    /// 吉凶名称（如"大吉"）
    pub fortune_name: BoundedVec<u8, ConstU32<16>>,
}

impl Default for HexagramFullDetail {
    fn default() -> Self {
        Self {
            name: BoundedVec::default(),
            shang_gua: SingleGuaDetail::default(),
            xia_gua: SingleGuaDetail::default(),
            guaci: BoundedVec::default(),
            dong_yao_name: BoundedVec::default(),
            dong_yao_ming: BoundedVec::default(),
            dong_yao_ci: BoundedVec::default(),
            tiyong_name: BoundedVec::default(),
            fortune_name: BoundedVec::default(),
        }
    }
}

/// 完整解读详情
///
/// 用于前端展示的完整数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct InterpretationFullDetail {
    /// 基础信息
    pub basic_info: InterpretationBasicInfo,

    /// 本卦详细信息
    pub ben_gua: HexagramFullDetail,

    /// 变卦详细信息
    pub bian_gua: HexagramFullDetail,

    /// 互卦详细信息
    pub hu_gua: HexagramFullDetail,

    /// 错卦详细信息
    pub cuo_gua: HexagramFullDetail,

    /// 综卦详细信息
    pub zong_gua: HexagramFullDetail,

    /// 伏卦详细信息
    pub fu_gua: HexagramFullDetail,

    /// 体用分析
    pub tiyong_analysis: TiYongAnalysis,

    /// 体用关系详细解读
    pub tiyong_interpretation: BoundedVec<u8, ConstU32<512>>,

    /// 应期推算
    pub yingqi_analysis: YingQiAnalysis,

    /// 综合解读建议（简短）
    pub summary: BoundedVec<u8, ConstU32<512>>,
}

impl Default for InterpretationFullDetail {
    fn default() -> Self {
        Self {
            basic_info: InterpretationBasicInfo::default(),
            ben_gua: HexagramFullDetail::default(),
            bian_gua: HexagramFullDetail::default(),
            hu_gua: HexagramFullDetail::default(),
            cuo_gua: HexagramFullDetail::default(),
            zong_gua: HexagramFullDetail::default(),
            fu_gua: HexagramFullDetail::default(),
            tiyong_analysis: TiYongAnalysis::default(),
            tiyong_interpretation: BoundedVec::default(),
            yingqi_analysis: YingQiAnalysis::default(),
            summary: BoundedVec::default(),
        }
    }
}

// ==================== AI解读数据结构 ====================

/// AI解读请求数据
///
/// 发送给AI的结构化数据
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct AiInterpretationRequest {
    /// 卦象ID
    pub hexagram_id: u64,

    /// 完整解卦数据
    pub interpretation_data: InterpretationData,

    /// 占卜问题（加密或哈希）
    pub question_hash: [u8; 32],

    /// 占卜类别
    pub category: u8,

    /// 请求时间戳
    pub request_timestamp: u64,
}

/// AI解读结果
///
/// AI返回的解读内容
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct AiInterpretationResult {
    /// 卦象ID
    pub hexagram_id: u64,

    /// 解读内容的IPFS CID
    pub interpretation_cid: BoundedVec<u8, ConstU32<64>>,

    /// 解读摘要（链上存储）
    pub summary: BoundedVec<u8, ConstU32<512>>,

    /// 吉凶评分（0-100）
    pub fortune_score: u8,

    /// 可信度评分（0-100）
    pub confidence_score: u8,

    /// 提交时间戳
    pub submit_timestamp: u64,

    /// AI模型版本
    pub model_version: BoundedVec<u8, ConstU32<32>>,
}

impl Default for AiInterpretationResult {
    fn default() -> Self {
        Self {
            hexagram_id: 0,
            interpretation_cid: BoundedVec::default(),
            summary: BoundedVec::default(),
            fortune_score: 50,
            confidence_score: 50,
            submit_timestamp: 0,
            model_version: BoundedVec::default(),
        }
    }
}

// ==================== 辅助函数 ====================

impl InterpretationData {
    /// 从完整卦象创建解卦数据
    ///
    /// # 参数
    /// - `full_divination`: 完整卦象
    /// - `timestamp`: 占卜时间戳
    /// - `lunar_date`: 农历日期
    /// - `method`: 起卦方式
    /// - `gender`: 性别
    /// - `category`: 类别
    ///
    /// # 返回
    /// - 完整解卦数据
    pub fn from_full_divination<AccountId: Clone, BlockNumber: Clone>(
        full_divination: &FullDivination<AccountId, BlockNumber>,
        timestamp: u64,
        lunar_date: LunarDateInfo,
        method: DivinationMethod,
        gender: u8,
        category: u8,
    ) -> Self {
        use crate::algorithm;

        // 基础信息
        let basic_info = InterpretationBasicInfo {
            timestamp,
            lunar_date: lunar_date.clone(),
            method,
            gender,
            category,
        };

        // 卦象核心数据
        let hexagram_core = HexagramCoreData {
            shang_gua: full_divination.ben_gua.shang_gua,
            xia_gua: full_divination.ben_gua.xia_gua,
            dong_yao: full_divination.ben_gua.dong_yao,
            ti_is_shang: full_divination.ben_gua.ti_is_shang,
        };

        // 体用分析
        let ti_gua = hexagram_core.ti_gua();
        let yong_gua = hexagram_core.yong_gua();
        let ti_wuxing = ti_gua.wuxing();
        let yong_wuxing = yong_gua.wuxing();

        // 计算互卦体用关系
        let hu_relation = if hexagram_core.ti_is_shang {
            TiYongRelation::calculate(&full_divination.hu_gua.0.wuxing(), &full_divination.hu_gua.1.wuxing())
        } else {
            TiYongRelation::calculate(&full_divination.hu_gua.1.wuxing(), &full_divination.hu_gua.0.wuxing())
        };

        // 计算旺衰
        let ti_wangshuai = algorithm::calc_wangshuai(ti_gua, lunar_date.month);

        let tiyong_analysis = TiYongAnalysis {
            ti_wuxing,
            yong_wuxing,
            ben_gua_relation: full_divination.ben_gua_relation,
            bian_gua_relation: full_divination.bian_gua_relation,
            hu_gua_relation: hu_relation,
            ti_wangshuai,
            fortune: full_divination.fortune,
            fortune_level: full_divination.fortune as u8,
        };

        // 应期推算
        let yingqi_result = algorithm::calc_yingqi(ti_gua, yong_gua, lunar_date.month);

        // 将 YingQiResult 转换为 YingQiAnalysis
        // YingQiAnalysis 的 analysis 字段最大为 256 字节，需要截断
        let analysis_bytes = yingqi_result.analysis.to_vec();
        let truncated_analysis = if analysis_bytes.len() > 256 {
            &analysis_bytes[..256]
        } else {
            &analysis_bytes[..]
        };

        let yingqi_analysis = YingQiAnalysis {
            ti_gua_num: yingqi_result.ti_gua_num,
            yong_gua_num: yingqi_result.yong_gua_num,
            primary_num: yingqi_result.primary_num,
            secondary_nums: yingqi_result.secondary_nums,
            sheng_ti_wuxing: yingqi_result.sheng_ti_wuxing,
            ke_ti_wuxing: yingqi_result.ke_ti_wuxing,
            analysis: BoundedVec::try_from(truncated_analysis.to_vec())
                .unwrap_or_default(),
        };

        // 辅助卦象
        let (cuo_shang, cuo_xia) = algorithm::calc_cuo_gua(&hexagram_core.shang_gua, &hexagram_core.xia_gua);
        let (zong_shang, zong_xia) = algorithm::calc_zong_gua(&hexagram_core.shang_gua, &hexagram_core.xia_gua);
        let (fu_shang, fu_xia) = algorithm::calc_fu_gua(&hexagram_core.shang_gua, &hexagram_core.xia_gua);

        let auxiliary_hexagrams = AuxiliaryHexagrams {
            bian_gua: full_divination.bian_gua,
            hu_gua: full_divination.hu_gua,
            cuo_gua: (cuo_shang, cuo_xia),
            zong_gua: (zong_shang, zong_xia),
            fu_gua: (fu_shang, fu_xia),
        };

        Self {
            basic_info,
            hexagram_core,
            tiyong_analysis,
            yingqi_analysis,
            auxiliary_hexagrams,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexagram_core_data() {
        let core = HexagramCoreData {
            shang_gua: SingleGua::from_num(1), // 乾
            xia_gua: SingleGua::from_num(8),   // 坤
            dong_yao: 1,
            ti_is_shang: true,
        };

        // 体卦应该是上卦（乾）
        assert_eq!(core.ti_gua().bagua, Bagua::Qian);
        // 用卦应该是下卦（坤）
        assert_eq!(core.yong_gua().bagua, Bagua::Kun);
    }

    #[test]
    fn test_tiyong_analysis() {
        let analysis = TiYongAnalysis {
            ti_wuxing: WuXing::Jin,  // 金
            yong_wuxing: WuXing::Mu, // 木
            ben_gua_relation: TiYongRelation::TiKeYong, // 体克用（金克木）
            bian_gua_relation: TiYongRelation::BiHe,
            hu_gua_relation: TiYongRelation::BiHe,
            ti_wangshuai: WangShuai::Wang,
            fortune: Fortune::Ping,
            fortune_level: 2,
        };

        assert_eq!(analysis.fortune_level, 2);
        assert!(analysis.ti_wangshuai.is_strong());
    }

    #[test]
    fn test_lunar_date_info() {
        let lunar = LunarDateInfo {
            year: 2024,
            month: 11,
            day: 20,
            hour_zhi_num: 1,
            is_leap_month: false,
        };

        assert_eq!(lunar.year, 2024);
        assert_eq!(lunar.month, 11);
    }
}
