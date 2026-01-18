//! # 六爻占卜计算模块
//!
//! 实现六爻（纳甲法）的排卦与解卦

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec, vec::Vec};

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{HexagramInfo, LineInfo, LiuYaoInput, LiuYaoResult};

/// 六爻值含义：0=老阴(变), 1=少阳, 2=少阴, 3=老阳(变)

/// 八卦名称
const BAGUA_NAMES: [&str; 8] = ["乾", "兑", "离", "震", "巽", "坎", "艮", "坤"];

/// 六亲
const SIX_RELATIVES: [&str; 6] = ["父母", "兄弟", "子孙", "妻财", "官鬼", "世应"];

/// 六神
const SIX_DEITIES: [&str; 6] = ["青龙", "朱雀", "勾陈", "螣蛇", "白虎", "玄武"];

/// 六十四卦名称（按先天八卦序）
const HEXAGRAM_NAMES: [&str; 64] = [
    "乾为天", "天泽履", "天火同人", "天雷无妄", "天风姤", "天水讼", "天山遁", "天地否",
    "泽天夬", "兑为泽", "泽火革", "泽雷随", "泽风大过", "泽水困", "泽山咸", "泽地萃",
    "火天大有", "火泽睽", "离为火", "火雷噬嗑", "火风鼎", "火水未济", "火山旅", "火地晋",
    "雷天大壮", "雷泽归妹", "雷火丰", "震为雷", "雷风恒", "雷水解", "雷山小过", "雷地豫",
    "风天小畜", "风泽中孚", "风火家人", "风雷益", "巽为风", "风水涣", "风山渐", "风地观",
    "水天需", "水泽节", "水火既济", "水雷屯", "水风井", "坎为水", "水山蹇", "水地比",
    "山天大畜", "山泽损", "山火贲", "山雷颐", "山风蛊", "山水蒙", "艮为山", "山地剥",
    "地天泰", "地泽临", "地火明夷", "地雷复", "地风升", "地水师", "地山谦", "坤为地",
];

/// 六爻计算器
pub struct LiuYaoCalculator;

impl LiuYaoCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self
    }

    /// 执行六爻计算
    pub fn calculate(&self, input: &LiuYaoInput) -> EnclaveResult<LiuYaoResult> {
        // 验证输入
        for &yao in &input.yao_data {
            if yao > 3 {
                return Err(EnclaveError::InvalidInputData);
            }
        }

        // 将六爻数据转换为卦象
        let (original_gua, changed_gua, has_change) = self.yao_to_hexagram(&input.yao_data);

        // 获取卦象信息
        let original_hexagram = self.get_hexagram_info(original_gua);

        let changed_hexagram = if has_change {
            Some(self.get_hexagram_info(changed_gua))
        } else {
            None
        };

        // 计算世应位置
        let (shi_position, ying_position) = self.calculate_shi_ying(original_gua);

        // 生成六爻详情
        let lines = self.generate_lines(&input.yao_data, input.timestamp);

        // 生成分析
        let analysis = self.generate_analysis(&original_hexagram.name, &changed_hexagram);

        Ok(LiuYaoResult {
            original_hexagram,
            changed_hexagram,
            lines,
            shi_position,
            ying_position,
            analysis,
        })
    }

    /// 将六爻数据转换为卦象
    fn yao_to_hexagram(&self, yao_data: &[u8; 6]) -> (u8, u8, bool) {
        let mut original: u8 = 0;
        let mut changed: u8 = 0;
        let mut has_change = false;

        for (i, &yao) in yao_data.iter().enumerate() {
            // 原卦：老阴(0)和少阴(2)为阴，老阳(3)和少阳(1)为阳
            if yao == 1 || yao == 3 {
                original |= 1 << i;
            }

            // 变卦：老阴变阳，老阳变阴
            match yao {
                0 => {
                    // 老阴变阳
                    changed |= 1 << i;
                    has_change = true;
                }
                1 => {
                    // 少阳不变
                    changed |= 1 << i;
                }
                2 => {
                    // 少阴不变
                }
                3 => {
                    // 老阳变阴
                    has_change = true;
                }
                _ => {}
            }
        }

        (original, changed, has_change)
    }

    /// 获取卦象信息
    fn get_hexagram_info(&self, gua: u8) -> HexagramInfo {
        // 分解为上下卦
        let lower = gua & 0x07;
        let upper = (gua >> 3) & 0x07;

        // 先天八卦转后天八卦序
        let hex_index = (upper as usize * 8 + lower as usize) % 64;

        HexagramInfo {
            name: HEXAGRAM_NAMES[hex_index].to_string(),
            number: (hex_index + 1) as u8,
            upper: BAGUA_NAMES[upper as usize].to_string(),
            lower: BAGUA_NAMES[lower as usize].to_string(),
            text: String::new(), // 简化处理
        }
    }

    /// 计算世应位置
    fn calculate_shi_ying(&self, gua: u8) -> (u8, u8) {
        // 简化算法：根据卦象计算
        let lower = gua & 0x07;
        let upper = (gua >> 3) & 0x07;

        // 如果上下卦相同（八纯卦）
        if upper == lower {
            return (6, 3);
        }

        // 简化：根据上下卦的异同确定世爻位置
        let diff = (upper ^ lower).count_ones();
        let shi = match diff {
            1 => 1,
            2 => 2,
            3 => 3,
            _ => 4,
        };

        let ying = if shi <= 3 { shi + 3 } else { shi - 3 };

        (shi as u8, ying as u8)
    }

    /// 生成六爻详情
    fn generate_lines(&self, yao_data: &[u8; 6], timestamp: u64) -> Vec<LineInfo> {
        let mut lines = Vec::with_capacity(6);

        // 根据时间戳确定六神起始
        let deity_start = ((timestamp / 86400) % 6) as usize;

        for (i, &yao) in yao_data.iter().enumerate() {
            let is_yang = yao == 1 || yao == 3;
            let is_moving = yao == 0 || yao == 3;

            lines.push(LineInfo {
                position: (i + 1) as u8,
                yin_yang: if is_yang { "阳".to_string() } else { "阴".to_string() },
                is_moving,
                six_relative: SIX_RELATIVES[i % 6].to_string(),
                six_deity: SIX_DEITIES[(deity_start + i) % 6].to_string(),
            });
        }

        lines
    }

    /// 生成分析
    fn generate_analysis(&self, original_name: &str, changed: &Option<HexagramInfo>) -> String {
        match changed {
            Some(ch) => {
                format!(
                    "本卦{}，变卦{}。卦中有动爻，需观变卦以断吉凶。",
                    original_name, ch.name
                )
            }
            None => {
                format!(
                    "本卦{}，无动爻，卦象稳定。以本卦断事。",
                    original_name
                )
            }
        }
    }
}

impl Default for LiuYaoCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liuyao_calculation() {
        let calc = LiuYaoCalculator::new();

        let input = LiuYaoInput {
            yao_data: [1, 2, 1, 3, 2, 1], // 混合阴阳
            timestamp: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        assert!(!result.original_hexagram.name.is_empty());
        assert_eq!(result.lines.len(), 6);
        assert!(result.shi_position >= 1 && result.shi_position <= 6);
        assert!(result.ying_position >= 1 && result.ying_position <= 6);
    }

    #[test]
    fn test_liuyao_with_changes() {
        let calc = LiuYaoCalculator::new();

        // 包含老阴和老阳的卦
        let input = LiuYaoInput {
            yao_data: [0, 1, 3, 2, 1, 0], // 有变爻
            timestamp: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        // 应该有变卦
        assert!(result.changed_hexagram.is_some());

        // 检查动爻
        let moving_count = result.lines.iter().filter(|l| l.is_moving).count();
        assert!(moving_count > 0);
    }

    #[test]
    fn test_liuyao_no_changes() {
        let calc = LiuYaoCalculator::new();

        // 只有少阴少阳的卦
        let input = LiuYaoInput {
            yao_data: [1, 2, 1, 2, 1, 2], // 无变爻
            timestamp: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        // 变卦应该和本卦相同（无动爻）
        let moving_count = result.lines.iter().filter(|l| l.is_moving).count();
        assert_eq!(moving_count, 0);
    }

    #[test]
    fn test_invalid_yao_data() {
        let calc = LiuYaoCalculator::new();

        let input = LiuYaoInput {
            yao_data: [1, 2, 5, 2, 1, 2], // 5 是无效值
            timestamp: 1704067200,
        };

        assert!(calc.calculate(&input).is_err());
    }
}
