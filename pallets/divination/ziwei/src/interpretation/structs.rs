//! # 紫微斗数解卦结构体定义
//!
//! 本模块定义解卦系统所需的所有数据结构

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::types::*;
use super::enums::*;

// ============================================================================
// 命盘整体评分
// ============================================================================

/// 命盘整体评分
///
/// 包含命盘的综合评分和各项指数
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct ChartOverallScore {
    /// 综合评分（0-100）
    pub overall_score: u8,

    /// 命格等级（0-5）
    pub ming_ge_level: MingGeLevel,

    /// 富贵指数（0-100）
    pub wealth_index: u8,

    /// 事业指数（0-100）
    pub career_index: u8,

    /// 感情指数（0-100）
    pub relationship_index: u8,

    /// 健康指数（0-100）
    pub health_index: u8,

    /// 福德指数（0-100）
    pub fortune_index: u8,
}

// ============================================================================
// 宫位解读数据
// ============================================================================

/// 单个宫位的解读数据
///
/// 包含宫位的评分、吉凶等级、影响因素等
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct PalaceInterpretation {
    /// 宫位类型
    pub gong_wei: GongWei,

    /// 宫位评分（0-100）
    pub score: u8,

    /// 吉凶等级
    pub fortune_level: FortuneLevel,

    /// 主星强度（0-100）
    pub star_strength: u8,

    /// 四化影响（-50 ~ +50）
    pub si_hua_impact: i8,

    /// 六吉星数量（0-6）
    pub liu_ji_count: u8,

    /// 六煞星数量（0-6）
    pub liu_sha_count: u8,

    /// 关键词索引（最多3个）
    /// 索引对应预定义的关键词表
    pub keywords: [u8; 3],

    /// 主要影响因素（位标志，8 bits）
    /// bit 0: 主星庙旺
    /// bit 1: 四化加持
    /// bit 2: 六吉会照
    /// bit 3: 六煞冲破
    /// bit 4: 空宫借星
    /// bit 5: 禄存同宫
    /// bit 6: 天马同宫
    /// bit 7: 预留
    pub factors: u8,
}

impl PalaceInterpretation {
    /// 检查是否有主星庙旺
    pub fn has_star_miao_wang(&self) -> bool {
        self.factors & 0b0000_0001 != 0
    }

    /// 检查是否有四化加持
    pub fn has_si_hua_boost(&self) -> bool {
        self.factors & 0b0000_0010 != 0
    }

    /// 检查是否有六吉会照
    pub fn has_liu_ji(&self) -> bool {
        self.factors & 0b0000_0100 != 0
    }

    /// 检查是否有六煞冲破
    pub fn has_liu_sha(&self) -> bool {
        self.factors & 0b0000_1000 != 0
    }

    /// 检查是否为空宫借星
    pub fn is_kong_gong(&self) -> bool {
        self.factors & 0b0001_0000 != 0
    }

    /// 检查是否有禄存
    pub fn has_lu_cun(&self) -> bool {
        self.factors & 0b0010_0000 != 0
    }

    /// 检查是否有天马
    pub fn has_tian_ma(&self) -> bool {
        self.factors & 0b0100_0000 != 0
    }

    /// 设置主星庙旺标志
    pub fn set_star_miao_wang(&mut self) {
        self.factors |= 0b0000_0001;
    }

    /// 设置四化加持标志
    pub fn set_si_hua_boost(&mut self) {
        self.factors |= 0b0000_0010;
    }

    /// 设置六吉会照标志
    pub fn set_liu_ji(&mut self) {
        self.factors |= 0b0000_0100;
    }

    /// 设置六煞冲破标志
    pub fn set_liu_sha(&mut self) {
        self.factors |= 0b0000_1000;
    }

    /// 设置空宫借星标志
    pub fn set_kong_gong(&mut self) {
        self.factors |= 0b0001_0000;
    }

    /// 设置禄存标志
    pub fn set_lu_cun(&mut self) {
        self.factors |= 0b0010_0000;
    }

    /// 设置天马标志
    pub fn set_tian_ma(&mut self) {
        self.factors |= 0b0100_0000;
    }
}

// ============================================================================
// 格局信息
// ============================================================================

/// 格局信息
///
/// 描述命盘中识别到的格局
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct PatternInfo {
    /// 格局类型
    pub pattern_type: PatternType,

    /// 格局强度（0-100）
    pub strength: u8,

    /// 是否成立
    pub is_valid: bool,

    /// 是否吉格
    pub is_auspicious: bool,

    /// 格局分数（-50 ~ +50）
    pub score: i8,

    /// 关键宫位索引（最多3个，0-11）
    pub key_palaces: [u8; 3],
}

impl PatternInfo {
    /// 创建新的格局信息
    pub fn new(pattern_type: PatternType, strength: u8, key_palaces: [u8; 3]) -> Self {
        let is_auspicious = pattern_type.is_auspicious();
        let base_score = pattern_type.base_score();
        let score = (base_score as i16 * strength as i16 / 100).clamp(-50, 50) as i8;

        Self {
            pattern_type,
            strength,
            is_valid: true,
            is_auspicious,
            score,
            key_palaces,
        }
    }
}

// ============================================================================
// 四化飞星分析
// ============================================================================

/// 四化飞星分析
///
/// 分析命盘中的四化飞星关系
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct SiHuaAnalysis {
    /// 生年四化星（禄权科忌）
    pub sheng_nian_si_hua: [SiHuaStar; 4],

    /// 命宫四化飞入宫位（禄权科忌，0-11）
    pub ming_gong_fei_ru: [u8; 4],

    /// 财帛宫四化飞入宫位（0-11）
    pub cai_bo_fei_ru: [u8; 4],

    /// 官禄宫四化飞入宫位（0-11）
    pub guan_lu_fei_ru: [u8; 4],

    /// 夫妻宫四化飞入宫位（0-11）
    pub fu_qi_fei_ru: [u8; 4],

    /// 自化宫位（位标志，12 bits）
    /// bit 0-11 对应 12 个宫位
    pub zi_hua_palaces: u16,

    /// 化忌冲破宫位（位标志，12 bits）
    /// bit 0-11 对应 12 个宫位
    pub hua_ji_chong_po: u16,
}

impl SiHuaAnalysis {
    /// 检查指定宫位是否有自化
    pub fn has_zi_hua(&self, gong_index: u8) -> bool {
        if gong_index >= 12 {
            return false;
        }
        self.zi_hua_palaces & (1 << gong_index) != 0
    }

    /// 检查指定宫位是否被化忌冲破
    pub fn has_hua_ji_chong_po(&self, gong_index: u8) -> bool {
        if gong_index >= 12 {
            return false;
        }
        self.hua_ji_chong_po & (1 << gong_index) != 0
    }

    /// 设置自化宫位
    pub fn set_zi_hua(&mut self, gong_index: u8) {
        if gong_index < 12 {
            self.zi_hua_palaces |= 1 << gong_index;
        }
    }

    /// 设置化忌冲破宫位
    pub fn set_hua_ji_chong_po(&mut self, gong_index: u8) {
        if gong_index < 12 {
            self.hua_ji_chong_po |= 1 << gong_index;
        }
    }
}

// ============================================================================
// 大限解读
// ============================================================================

/// 单个大限的解读数据
///
/// 描述10年大限的运势
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct DaXianInterpretation {
    /// 大限序号（1-12）
    pub index: u8,

    /// 起始年龄
    pub start_age: u8,

    /// 结束年龄
    pub end_age: u8,

    /// 大限宫位索引（0-11）
    pub gong_index: u8,

    /// 大限评分（0-100）
    pub score: u8,

    /// 大限吉凶等级
    pub fortune_level: FortuneLevel,

    /// 大限四化飞入宫位（禄权科忌，0-11）
    pub si_hua_fei_ru: [u8; 4],

    /// 关键词索引（最多3个）
    pub keywords: [u8; 3],
}

// ============================================================================
// 完整解卦数据
// ============================================================================

/// 紫微斗数完整解卦数据
///
/// 包含命盘的所有解读信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct ZiweiInterpretation {
    /// 命盘ID
    pub chart_id: u64,

    /// 整体评分
    pub overall_score: ChartOverallScore,

    /// 十二宫解读
    pub palace_interpretations: [PalaceInterpretation; 12],

    /// 识别到的格局（最多10个）
    pub patterns: BoundedVec<PatternInfo, ConstU32<10>>,

    /// 四化飞星分析
    pub si_hua_analysis: SiHuaAnalysis,

    /// 十二大限解读
    pub da_xian_interpretations: [DaXianInterpretation; 12],

    /// 五行分布（金木水火土，0-100）
    pub wu_xing_distribution: [u8; 5],

    /// 命主星索引（0-13，对应14主星，255表示无）
    pub ming_zhu_star: u8,

    /// 身主星索引（0-13，255表示无）
    pub shen_zhu_star: u8,

    /// 创建时间戳（Unix秒）
    pub created_at: u64,

    /// AI解读CID（可选）
    pub ai_interpretation_cid: Option<BoundedVec<u8, ConstU32<64>>>,
}

impl ZiweiInterpretation {
    /// 创建新的解卦数据
    pub fn new(chart_id: u64, created_at: u64) -> Self {
        Self {
            chart_id,
            overall_score: ChartOverallScore::default(),
            palace_interpretations: Default::default(),
            patterns: BoundedVec::default(),
            si_hua_analysis: SiHuaAnalysis::default(),
            da_xian_interpretations: Default::default(),
            wu_xing_distribution: [0; 5],
            ming_zhu_star: 255,
            shen_zhu_star: 255,
            created_at,
            ai_interpretation_cid: None,
        }
    }

    /// 获取指定宫位的解读
    pub fn get_palace_interpretation(&self, gong_wei: GongWei) -> Option<&PalaceInterpretation> {
        self.palace_interpretations
            .iter()
            .find(|p| p.gong_wei == gong_wei)
    }

    /// 获取吉格数量
    pub fn auspicious_pattern_count(&self) -> usize {
        self.patterns.iter().filter(|p| p.is_auspicious).count()
    }

    /// 获取凶格数量
    pub fn inauspicious_pattern_count(&self) -> usize {
        self.patterns.iter().filter(|p| !p.is_auspicious).count()
    }

    /// 获取格局总分
    pub fn total_pattern_score(&self) -> i32 {
        self.patterns.iter().map(|p| p.score as i32).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palace_interpretation_factors() {
        let mut palace = PalaceInterpretation::default();

        assert!(!palace.has_star_miao_wang());
        palace.set_star_miao_wang();
        assert!(palace.has_star_miao_wang());

        assert!(!palace.has_si_hua_boost());
        palace.set_si_hua_boost();
        assert!(palace.has_si_hua_boost());

        assert!(!palace.has_lu_cun());
        palace.set_lu_cun();
        assert!(palace.has_lu_cun());
    }

    #[test]
    fn test_si_hua_analysis() {
        let mut analysis = SiHuaAnalysis::default();

        assert!(!analysis.has_zi_hua(0));
        analysis.set_zi_hua(0);
        assert!(analysis.has_zi_hua(0));
        assert!(!analysis.has_zi_hua(1));

        assert!(!analysis.has_hua_ji_chong_po(5));
        analysis.set_hua_ji_chong_po(5);
        assert!(analysis.has_hua_ji_chong_po(5));
    }

    #[test]
    fn test_pattern_info() {
        let pattern = PatternInfo::new(
            PatternType::ZiFuTongGong,
            80,
            [0, 1, 2],
        );

        assert!(pattern.is_valid);
        assert!(pattern.is_auspicious);
        assert_eq!(pattern.strength, 80);
        assert!(pattern.score > 0);
    }

    #[test]
    fn test_ziwei_interpretation() {
        let interp = ZiweiInterpretation::new(1, 1234567890);

        assert_eq!(interp.chart_id, 1);
        assert_eq!(interp.created_at, 1234567890);
        assert_eq!(interp.auspicious_pattern_count(), 0);
        assert_eq!(interp.inauspicious_pattern_count(), 0);
        assert_eq!(interp.total_pattern_score(), 0);
    }
}
