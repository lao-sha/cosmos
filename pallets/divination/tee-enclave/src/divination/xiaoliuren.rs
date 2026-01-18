//! # 小六壬计算模块
//!
//! 小六壬是一种简单快速的占卜方法，通过月、日、时三个数字
//! 依次在大安、留连、速喜、赤口、小吉、空亡六宫循环推算。

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{XiaoLiuRenInput, XiaoLiuRenResult};

/// 小六壬六宫名称
const PALACES: [&str; 6] = ["大安", "留连", "速喜", "赤口", "小吉", "空亡"];

/// 六宫吉凶
const FORTUNES: [&str; 6] = ["大吉", "小凶", "大吉", "凶", "吉", "凶"];

/// 六宫详细解释
const INTERPRETATIONS: [&str; 6] = [
    "大安：身不动时，五行属木，颜色青色，方位东方。临青龙，谋事主一、五、七。有静止、心安之意。吉祥之兆。",
    "留连：人未归时，五行属水，颜色黑色，方位北方。临玄武，谋事主二、八、十。有暗昧不明、拖延之意。做事多阻碍。",
    "速喜：人即至时，五行属火，颜色红色，方位南方。临朱雀，谋事主三、六、九。有快速、喜庆之意。吉祥之兆。",
    "赤口：官事凶时，五行属金，颜色白色，方位西方。临白虎，谋事主四、七、十。有口舌是非、惊恐之意。凶兆。",
    "小吉：人来喜时，五行属木，颜色绿色，方位东方。临六合，谋事主一、五、七。有和合、吉祥之意。小吉之兆。",
    "空亡：音信稀时，五行属土，颜色黄色，方位中央。临勾陈，谋事主三、六、九。有虚空、不实之意。凶兆。",
];

/// 小六壬计算器
pub struct XiaoLiuRenCalculator;

impl XiaoLiuRenCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self
    }

    /// 执行小六壬计算
    ///
    /// 算法：
    /// 1. 从大安起月
    /// 2. 月上起日
    /// 3. 日上起时
    pub fn calculate(&self, input: &XiaoLiuRenInput) -> EnclaveResult<XiaoLiuRenResult> {
        // 验证输入
        if input.month < 1 || input.month > 12 {
            return Err(EnclaveError::InvalidInputData);
        }
        if input.day < 1 || input.day > 31 {
            return Err(EnclaveError::InvalidInputData);
        }
        if input.hour > 23 {
            return Err(EnclaveError::InvalidInputData);
        }

        // 计算落宫
        // 月份：从大安起正月
        let month_palace = (input.month as usize - 1) % 6;

        // 日期：从月落宫起初一
        let day_palace = (month_palace + input.day as usize - 1) % 6;

        // 时辰：从日落宫起子时 (0点为子时)
        // 时辰转换：0-1点子时(0), 1-3点丑时(1), ..., 23-24点子时(0)
        let shi_chen = self.hour_to_shichen(input.hour);
        let final_palace = (day_palace + shi_chen as usize) % 6;

        Ok(XiaoLiuRenResult {
            palace: PALACES[final_palace].to_string(),
            palace_index: final_palace as u8,
            fortune: FORTUNES[final_palace].to_string(),
            interpretation: INTERPRETATIONS[final_palace].to_string(),
        })
    }

    /// 将小时转换为时辰索引
    /// 子时(23-1), 丑时(1-3), 寅时(3-5), 卯时(5-7), 辰时(7-9), 巳时(9-11),
    /// 午时(11-13), 未时(13-15), 申时(15-17), 酉时(17-19), 戌时(19-21), 亥时(21-23)
    fn hour_to_shichen(&self, hour: u8) -> u8 {
        match hour {
            23 | 0 => 0,  // 子时
            1 | 2 => 1,   // 丑时
            3 | 4 => 2,   // 寅时
            5 | 6 => 3,   // 卯时
            7 | 8 => 4,   // 辰时
            9 | 10 => 5,  // 巳时
            11 | 12 => 6, // 午时
            13 | 14 => 7, // 未时
            15 | 16 => 8, // 申时
            17 | 18 => 9, // 酉时
            19 | 20 => 10, // 戌时
            21 | 22 => 11, // 亥时
            _ => 0,
        }
    }
}

impl Default for XiaoLiuRenCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xiaoliuren_basic() {
        let calc = XiaoLiuRenCalculator::new();

        // 正月初一子时，应该落在大安
        let input = XiaoLiuRenInput {
            month: 1,
            day: 1,
            hour: 0,
        };
        let result = calc.calculate(&input).unwrap();
        assert_eq!(result.palace, "大安");
        assert_eq!(result.palace_index, 0);
    }

    #[test]
    fn test_xiaoliuren_various() {
        let calc = XiaoLiuRenCalculator::new();

        // 五月十五日午时
        let input = XiaoLiuRenInput {
            month: 5,
            day: 15,
            hour: 12,
        };
        let result = calc.calculate(&input).unwrap();
        assert!(result.palace_index < 6);
        assert!(!result.palace.is_empty());
        assert!(!result.fortune.is_empty());
    }

    #[test]
    fn test_xiaoliuren_invalid_input() {
        let calc = XiaoLiuRenCalculator::new();

        // 无效月份
        let input = XiaoLiuRenInput {
            month: 13,
            day: 1,
            hour: 0,
        };
        assert!(calc.calculate(&input).is_err());

        // 无效日期
        let input = XiaoLiuRenInput {
            month: 1,
            day: 32,
            hour: 0,
        };
        assert!(calc.calculate(&input).is_err());

        // 无效小时
        let input = XiaoLiuRenInput {
            month: 1,
            day: 1,
            hour: 25,
        };
        assert!(calc.calculate(&input).is_err());
    }

    #[test]
    fn test_hour_to_shichen() {
        let calc = XiaoLiuRenCalculator::new();

        assert_eq!(calc.hour_to_shichen(0), 0);  // 子时
        assert_eq!(calc.hour_to_shichen(23), 0); // 子时
        assert_eq!(calc.hour_to_shichen(12), 6); // 午时
        assert_eq!(calc.hour_to_shichen(18), 9); // 酉时
    }
}
