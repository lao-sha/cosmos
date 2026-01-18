//! # 奇门遁甲计算模块
//!
//! 实现奇门遁甲的排盘算法

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec, vec::Vec};

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{PalaceInfo, QiMenInput, QiMenResult};

/// 九星
const NINE_STARS: [&str; 9] = ["天蓬", "天芮", "天冲", "天辅", "天禽", "天心", "天柱", "天任", "天英"];

/// 八门
const EIGHT_DOORS: [&str; 8] = ["休门", "生门", "伤门", "杜门", "景门", "死门", "惊门", "开门"];

/// 八神
const EIGHT_DEITIES: [&str; 8] = ["值符", "腾蛇", "太阴", "六合", "白虎", "玄武", "九地", "九天"];

/// 天干
const HEAVENLY_STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

/// 奇门遁甲计算器
pub struct QiMenCalculator;

impl QiMenCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self
    }

    /// 执行奇门遁甲排盘
    pub fn calculate(&self, input: &QiMenInput) -> EnclaveResult<QiMenResult> {
        // 从时间戳计算节气和局数
        let (ju_number, yin_yang) = self.calculate_ju(input.datetime)?;

        // 生成九宫数据
        let palaces = self.generate_palaces(ju_number, input.datetime);

        // 确定值符值使
        let (duty_symbol, duty_door) = self.get_duty_info(input.datetime);

        // 生成分析
        let analysis = self.generate_analysis(ju_number, &yin_yang, &duty_symbol, &duty_door);

        Ok(QiMenResult {
            ju_number,
            yin_yang,
            palaces,
            duty_symbol,
            duty_door,
            analysis,
        })
    }

    /// 计算局数和阴阳遁
    fn calculate_ju(&self, timestamp: u64) -> EnclaveResult<(i8, String)> {
        // 简化算法：根据时间戳判断节气
        // 冬至后阳遁，夏至后阴遁

        // 以一年的天数计算大致位置
        let day_of_year = (timestamp / 86400) % 365;

        // 简化判断：前半年阳遁，后半年阴遁
        let is_yang = day_of_year < 182;

        // 局数：1-9
        // 简化计算：根据时间戳取模
        let ju = ((timestamp / 7200) % 9 + 1) as i8;

        // 阴遁局数为负
        let ju_number = if is_yang { ju } else { -ju };

        let yin_yang = if is_yang {
            "阳遁".to_string()
        } else {
            "阴遁".to_string()
        };

        Ok((ju_number, yin_yang))
    }

    /// 生成九宫数据
    fn generate_palaces(&self, ju_number: i8, timestamp: u64) -> Vec<PalaceInfo> {
        let mut palaces = Vec::with_capacity(9);

        // 基于局数和时间生成各宫数据
        let base = ju_number.unsigned_abs() as usize;
        let seed = (timestamp / 3600) as usize;

        for i in 0..9 {
            let star_idx = (base + i + seed) % 9;
            let door_idx = (base + i + seed / 2) % 8;
            let deity_idx = (base + i + seed / 3) % 8;
            let earth_stem_idx = (base + i) % 10;
            let heaven_stem_idx = (base + i + seed) % 10;

            palaces.push(PalaceInfo {
                position: (i + 1) as u8,
                star: NINE_STARS[star_idx].to_string(),
                door: EIGHT_DOORS[door_idx].to_string(),
                deity: EIGHT_DEITIES[deity_idx].to_string(),
                earth_stem: HEAVENLY_STEMS[earth_stem_idx].to_string(),
                heaven_stem: HEAVENLY_STEMS[heaven_stem_idx].to_string(),
            });
        }

        palaces
    }

    /// 获取值符值使
    fn get_duty_info(&self, timestamp: u64) -> (String, String) {
        let seed = (timestamp / 7200) as usize;

        let symbol_idx = seed % 9;
        let door_idx = seed % 8;

        (
            NINE_STARS[symbol_idx].to_string(),
            EIGHT_DOORS[door_idx].to_string(),
        )
    }

    /// 生成分析
    fn generate_analysis(&self, ju_number: i8, yin_yang: &str, duty_symbol: &str, duty_door: &str) -> String {
        let ju_abs = ju_number.abs();
        format!(
            "{}{}局，值符{}，值使{}。此局{}气势，宜{}。",
            yin_yang,
            ju_abs,
            duty_symbol,
            duty_door,
            if ju_number > 0 { "阳" } else { "阴" },
            if ju_number > 0 { "进取" } else { "守成" }
        )
    }
}

impl Default for QiMenCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qimen_calculation() {
        let calc = QiMenCalculator::new();

        let input = QiMenInput {
            datetime: 1704067200,
            pan_type: 0,
            method: 0,
        };

        let result = calc.calculate(&input).unwrap();

        assert!(result.ju_number != 0);
        assert!(result.ju_number.abs() <= 9);
        assert_eq!(result.palaces.len(), 9);
        assert!(!result.duty_symbol.is_empty());
        assert!(!result.duty_door.is_empty());
    }

    #[test]
    fn test_palace_positions() {
        let calc = QiMenCalculator::new();

        let input = QiMenInput {
            datetime: 1704067200,
            pan_type: 0,
            method: 0,
        };

        let result = calc.calculate(&input).unwrap();

        // 验证九宫位置
        for (i, palace) in result.palaces.iter().enumerate() {
            assert_eq!(palace.position, (i + 1) as u8);
        }
    }
}
