//! # 占卜计算引擎
//!
//! 实现各种占卜方法的计算逻辑
//!
//! ## 支持的占卜类型
//!
//! 1. **八字命理** (BaZi) - 根据出生时间推算命运
//! 2. **梅花易数** (MeiHua) - 先天易数占卜法
//! 3. **奇门遁甲** (QiMen) - 古代兵法术数
//! 4. **六爻占卜** (LiuYao) - 周易六爻预测
//! 5. **紫微斗数** (ZiWei) - 命理星相学
//! 6. **塔罗占卜** (Tarot) - 西方塔罗牌占卜
//! 7. **大六壬** (DaLiuRen) - 古代数术
//! 8. **小六壬** (XiaoLiuRen) - 简化六壬速断

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

mod bazi;
mod daliuren;
mod liuyao;
mod meihua;
mod qimen;
mod tarot;
mod xiaoliuren;
mod ziwei;

pub use bazi::*;
pub use daliuren::*;
pub use liuyao::*;
pub use meihua::*;
pub use qimen::*;
pub use tarot::*;
pub use xiaoliuren::*;
pub use ziwei::*;

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{ComputeInput, ComputeOutput};

/// 占卜计算引擎
///
/// 统一的计算接口，根据输入类型分发到对应的计算模块
pub struct DivinationEngine {
    /// 八字计算器
    bazi: BaZiCalculator,
    /// 梅花易数计算器
    meihua: MeiHuaCalculator,
    /// 奇门遁甲计算器
    qimen: QiMenCalculator,
    /// 六爻计算器
    liuyao: LiuYaoCalculator,
    /// 紫微斗数计算器
    ziwei: ZiWeiCalculator,
    /// 塔罗计算器
    tarot: TarotCalculator,
    /// 大六壬计算器
    daliuren: DaLiuRenCalculator,
    /// 小六壬计算器
    xiaoliuren: XiaoLiuRenCalculator,
}

impl DivinationEngine {
    /// 创建新的占卜引擎
    pub fn new() -> Self {
        Self {
            bazi: BaZiCalculator::new(),
            meihua: MeiHuaCalculator::new(),
            qimen: QiMenCalculator::new(),
            liuyao: LiuYaoCalculator::new(),
            ziwei: ZiWeiCalculator::new(),
            tarot: TarotCalculator::new(),
            daliuren: DaLiuRenCalculator::new(),
            xiaoliuren: XiaoLiuRenCalculator::new(),
        }
    }

    /// 执行计算
    pub fn compute(&self, input: &ComputeInput) -> EnclaveResult<ComputeOutput> {
        match input {
            ComputeInput::BaZi(params) => {
                let result = self.bazi.calculate(params)?;
                Ok(ComputeOutput::BaZi(result))
            }
            ComputeInput::MeiHua(params) => {
                let result = self.meihua.calculate(params)?;
                Ok(ComputeOutput::MeiHua(result))
            }
            ComputeInput::QiMen(params) => {
                let result = self.qimen.calculate(params)?;
                Ok(ComputeOutput::QiMen(result))
            }
            ComputeInput::LiuYao(params) => {
                let result = self.liuyao.calculate(params)?;
                Ok(ComputeOutput::LiuYao(result))
            }
            ComputeInput::ZiWei(params) => {
                let result = self.ziwei.calculate(params)?;
                Ok(ComputeOutput::ZiWei(result))
            }
            ComputeInput::Tarot(params) => {
                let result = self.tarot.calculate(params)?;
                Ok(ComputeOutput::Tarot(result))
            }
            ComputeInput::DaLiuRen(params) => {
                let result = self.daliuren.calculate(params)?;
                Ok(ComputeOutput::DaLiuRen(result))
            }
            ComputeInput::XiaoLiuRen(params) => {
                let result = self.xiaoliuren.calculate(params)?;
                Ok(ComputeOutput::XiaoLiuRen(result))
            }
        }
    }
}

impl Default for DivinationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 天干
pub const HEAVENLY_STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

/// 地支
pub const EARTHLY_BRANCHES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// 五行
pub const FIVE_ELEMENTS: [&str; 5] = ["金", "木", "水", "火", "土"];

/// 八卦名称
pub const BAGUA_NAMES: [&str; 8] = ["乾", "兑", "离", "震", "巽", "坎", "艮", "坤"];

/// 六十四卦名称
pub const HEXAGRAM_NAMES: [&str; 64] = [
    "乾为天", "坤为地", "水雷屯", "山水蒙", "水天需", "天水讼", "地水师", "水地比",
    "风天小畜", "天泽履", "地天泰", "天地否", "天火同人", "火天大有", "地山谦", "雷地豫",
    "泽雷随", "山风蛊", "地泽临", "风地观", "火雷噬嗑", "山火贲", "山地剥", "地雷复",
    "天雷无妄", "山天大畜", "山雷颐", "泽风大过", "坎为水", "离为火", "泽山咸", "雷风恒",
    "天山遁", "雷天大壮", "火地晋", "地火明夷", "风火家人", "火泽睽", "水山蹇", "雷水解",
    "山泽损", "风雷益", "泽天夬", "天风姤", "泽地萃", "地风升", "泽水困", "水风井",
    "泽火革", "火风鼎", "震为雷", "艮为山", "风山渐", "雷泽归妹", "雷火丰", "火山旅",
    "巽为风", "兑为泽", "风水涣", "水泽节", "风泽中孚", "雷山小过", "水火既济", "火水未济",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_divination_engine_xiaoliuren() {
        let engine = DivinationEngine::new();

        let input = ComputeInput::XiaoLiuRen(XiaoLiuRenInput {
            month: 5,
            day: 15,
            hour: 10,
        });

        let result = engine.compute(&input).unwrap();

        if let ComputeOutput::XiaoLiuRen(xlr) = result {
            assert!(!xlr.palace.is_empty());
            assert!(xlr.palace_index < 6);
        } else {
            panic!("Wrong output type");
        }
    }

    #[test]
    fn test_all_calculation_types() {
        let engine = DivinationEngine::new();

        // 测试八字
        let bazi_result = engine.compute(&ComputeInput::BaZi(BaZiInput {
            year: 1990,
            month: 6,
            day: 15,
            hour: 12,
            gender: Gender::Male,
            longitude: None,
        }));
        assert!(bazi_result.is_ok());

        // 测试梅花易数
        let meihua_result = engine.compute(&ComputeInput::MeiHua(MeiHuaInput {
            method: 1,
            upper_num: Some(5),
            lower_num: Some(3),
            moving_line: Some(2),
            timestamp: 1704067200,
        }));
        assert!(meihua_result.is_ok());

        // 测试奇门遁甲
        let qimen_result = engine.compute(&ComputeInput::QiMen(QiMenInput {
            datetime: 1704067200,
            pan_type: 0,
            method: 0,
        }));
        assert!(qimen_result.is_ok());

        // 测试六爻
        let liuyao_result = engine.compute(&ComputeInput::LiuYao(LiuYaoInput {
            yao_data: [1, 2, 1, 3, 2, 1],
            timestamp: 1704067200,
        }));
        assert!(liuyao_result.is_ok());

        // 测试紫微斗数
        let ziwei_result = engine.compute(&ComputeInput::ZiWei(ZiWeiInput {
            lunar_year: 1990,
            lunar_month: 5,
            lunar_day: 15,
            shi_chen: 6,
            gender: Gender::Male,
            is_leap_month: false,
        }));
        assert!(ziwei_result.is_ok());

        // 测试塔罗
        let tarot_result = engine.compute(&ComputeInput::Tarot(TarotInput {
            spread: 1,
            seed: [42u8; 32],
            timestamp: 1704067200,
        }));
        assert!(tarot_result.is_ok());

        // 测试大六壬
        let daliuren_result = engine.compute(&ComputeInput::DaLiuRen(DaLiuRenInput {
            datetime: 1704067200,
        }));
        assert!(daliuren_result.is_ok());

        // 测试小六壬
        let xiaoliuren_result = engine.compute(&ComputeInput::XiaoLiuRen(XiaoLiuRenInput {
            month: 5,
            day: 15,
            hour: 10,
        }));
        assert!(xiaoliuren_result.is_ok());
    }
}
