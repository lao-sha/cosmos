//! # 紫微斗数计算模块
//!
//! 实现紫微斗数的排盘算法

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec, vec::Vec};

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{Gender, ZiWeiInput, ZiWeiPalace, ZiWeiResult};

/// 十二宫名称
const TWELVE_PALACES: [&str; 12] = [
    "命宫", "兄弟", "夫妻", "子女", "财帛", "疾厄",
    "迁移", "仆役", "官禄", "田宅", "福德", "父母",
];

/// 地支
const EARTHLY_BRANCHES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// 主星（十四主星）
const MAIN_STARS: [&str; 14] = [
    "紫微", "天机", "太阳", "武曲", "天同", "廉贞", "天府",
    "太阴", "贪狼", "巨门", "天相", "天梁", "七杀", "破军",
];

/// 辅星
const MINOR_STARS: [&str; 8] = [
    "文昌", "文曲", "左辅", "右弼", "天魁", "天钺", "禄存", "天马",
];

/// 命主
const MING_ZHU: [&str; 12] = [
    "贪狼", "巨门", "禄存", "文曲", "廉贞", "武曲",
    "破军", "武曲", "廉贞", "文曲", "禄存", "巨门",
];

/// 身主
const SHEN_ZHU: [&str; 12] = [
    "火星", "天相", "天梁", "天同", "文昌", "天机",
    "火星", "天相", "天梁", "天同", "文昌", "天机",
];

/// 紫微斗数计算器
pub struct ZiWeiCalculator;

impl ZiWeiCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self
    }

    /// 执行紫微斗数排盘
    pub fn calculate(&self, input: &ZiWeiInput) -> EnclaveResult<ZiWeiResult> {
        // 验证输入
        self.validate_input(input)?;

        // 计算命宫位置
        let ming_gong_idx = self.calculate_ming_gong(input.lunar_month, input.shi_chen);

        // 计算身宫位置
        let shen_gong_idx = self.calculate_shen_gong(input.lunar_month, input.shi_chen);

        // 排十二宫
        let twelve_palaces = self.arrange_palaces(ming_gong_idx, input);

        // 计算紫微星位置，并排布主星
        let main_stars = self.calculate_main_stars(input.lunar_day);

        // 确定命主和身主
        let ming_zhu = MING_ZHU[input.lunar_year as usize % 12].to_string();
        let shen_zhu = SHEN_ZHU[input.lunar_year as usize % 12].to_string();

        // 生成分析
        let analysis = self.generate_analysis(&twelve_palaces[0], &ming_zhu, &shen_zhu);

        Ok(ZiWeiResult {
            ming_gong: format!("{}{}", EARTHLY_BRANCHES[ming_gong_idx], TWELVE_PALACES[0]),
            shen_gong: format!("{}{}", EARTHLY_BRANCHES[shen_gong_idx], TWELVE_PALACES[shen_gong_idx]),
            twelve_palaces,
            main_stars,
            ming_zhu,
            shen_zhu,
            analysis,
        })
    }

    /// 验证输入
    fn validate_input(&self, input: &ZiWeiInput) -> EnclaveResult<()> {
        if input.lunar_year < 1900 || input.lunar_year > 2100 {
            return Err(EnclaveError::InvalidInputData);
        }
        if input.lunar_month < 1 || input.lunar_month > 12 {
            return Err(EnclaveError::InvalidInputData);
        }
        if input.lunar_day < 1 || input.lunar_day > 30 {
            return Err(EnclaveError::InvalidInputData);
        }
        if input.shi_chen > 11 {
            return Err(EnclaveError::InvalidInputData);
        }
        Ok(())
    }

    /// 计算命宫位置
    fn calculate_ming_gong(&self, month: u8, shi_chen: u8) -> usize {
        // 命宫公式：寅宫起正月，顺数至生月，再由生月逆数至时辰
        // 简化：(14 - month + 2 - shi_chen) % 12
        let ming = (14 - month as i32 + 2 - shi_chen as i32) % 12;
        if ming < 0 { (ming + 12) as usize } else { ming as usize }
    }

    /// 计算身宫位置
    fn calculate_shen_gong(&self, month: u8, shi_chen: u8) -> usize {
        // 身宫公式：寅宫起正月，顺数至生月，再由生月顺数至时辰
        ((month as usize + 2 + shi_chen as usize - 2) % 12)
    }

    /// 排十二宫
    fn arrange_palaces(&self, ming_gong_idx: usize, input: &ZiWeiInput) -> Vec<ZiWeiPalace> {
        let mut palaces = Vec::with_capacity(12);

        // 计算紫微星位置用于排主星
        let ziwei_pos = self.find_ziwei_position(input.lunar_day);

        for i in 0..12 {
            let palace_idx = (ming_gong_idx + i) % 12;
            let branch_idx = (2 + ming_gong_idx + i) % 12; // 从寅宫开始

            // 计算该宫的主星
            let main_stars = self.get_palace_main_stars(i, ziwei_pos);

            // 计算辅星
            let minor_stars = self.get_palace_minor_stars(i, input);

            palaces.push(ZiWeiPalace {
                name: TWELVE_PALACES[i].to_string(),
                branch: EARTHLY_BRANCHES[branch_idx].to_string(),
                main_stars,
                minor_stars,
            });
        }

        palaces
    }

    /// 查找紫微星位置
    fn find_ziwei_position(&self, lunar_day: u8) -> usize {
        // 简化的紫微星定位算法
        // 实际算法需要根据五行局和出生日推算
        ((lunar_day as usize + 2) % 12)
    }

    /// 计算主星列表
    fn calculate_main_stars(&self, lunar_day: u8) -> Vec<String> {
        // 返回命宫的主星列表
        let ziwei_pos = self.find_ziwei_position(lunar_day);

        // 简化：返回部分主星
        let mut stars = Vec::new();
        stars.push(MAIN_STARS[ziwei_pos % 14].to_string());

        if ziwei_pos % 3 == 0 {
            stars.push(MAIN_STARS[(ziwei_pos + 6) % 14].to_string());
        }

        stars
    }

    /// 获取某宫的主星
    fn get_palace_main_stars(&self, palace_idx: usize, ziwei_pos: usize) -> Vec<String> {
        let mut stars = Vec::new();

        // 简化的主星分布算法
        // 紫微星在ziwei_pos，其他主星按固定规则分布
        let relative_pos = (palace_idx + 12 - ziwei_pos) % 12;

        // 紫微星
        if relative_pos == 0 {
            stars.push("紫微".to_string());
        }

        // 天府星（与紫微相对）
        if relative_pos == 4 {
            stars.push("天府".to_string());
        }

        // 其他主星按简化规则
        match relative_pos {
            1 => stars.push("天机".to_string()),
            2 => stars.push("太阳".to_string()),
            3 => stars.push("武曲".to_string()),
            5 => stars.push("太阴".to_string()),
            6 => stars.push("贪狼".to_string()),
            7 => stars.push("巨门".to_string()),
            8 => stars.push("天相".to_string()),
            9 => stars.push("天梁".to_string()),
            10 => stars.push("七杀".to_string()),
            11 => stars.push("破军".to_string()),
            _ => {}
        }

        stars
    }

    /// 获取某宫的辅星
    fn get_palace_minor_stars(&self, palace_idx: usize, input: &ZiWeiInput) -> Vec<String> {
        let mut stars = Vec::new();

        // 文昌文曲
        if palace_idx == (10 - input.shi_chen as usize) % 12 {
            stars.push("文昌".to_string());
        }
        if palace_idx == (input.shi_chen as usize + 4) % 12 {
            stars.push("文曲".to_string());
        }

        // 左辅右弼
        if palace_idx == (input.lunar_month as usize + 3) % 12 {
            stars.push("左辅".to_string());
        }
        if palace_idx == (11 - input.lunar_month as usize) % 12 {
            stars.push("右弼".to_string());
        }

        stars
    }

    /// 生成分析
    fn generate_analysis(&self, ming_palace: &ZiWeiPalace, ming_zhu: &str, shen_zhu: &str) -> String {
        let main_stars_str = if ming_palace.main_stars.is_empty() {
            "无主星".to_string()
        } else {
            ming_palace.main_stars.join("、")
        };

        format!(
            "命宫在{}，主星{}。命主{}，身主{}。命格需综合分析十二宫位。",
            ming_palace.branch,
            main_stars_str,
            ming_zhu,
            shen_zhu
        )
    }
}

impl Default for ZiWeiCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ziwei_calculation() {
        let calc = ZiWeiCalculator::new();

        let input = ZiWeiInput {
            lunar_year: 1990,
            lunar_month: 5,
            lunar_day: 15,
            shi_chen: 6,
            gender: Gender::Male,
            is_leap_month: false,
        };

        let result = calc.calculate(&input).unwrap();

        assert!(!result.ming_gong.is_empty());
        assert!(!result.shen_gong.is_empty());
        assert_eq!(result.twelve_palaces.len(), 12);
        assert!(!result.ming_zhu.is_empty());
        assert!(!result.shen_zhu.is_empty());
    }

    #[test]
    fn test_twelve_palaces() {
        let calc = ZiWeiCalculator::new();

        let input = ZiWeiInput {
            lunar_year: 1990,
            lunar_month: 5,
            lunar_day: 15,
            shi_chen: 6,
            gender: Gender::Male,
            is_leap_month: false,
        };

        let result = calc.calculate(&input).unwrap();

        // 验证十二宫名称
        let palace_names: Vec<&str> = result.twelve_palaces.iter().map(|p| p.name.as_str()).collect();
        assert!(palace_names.contains(&"命宫"));
        assert!(palace_names.contains(&"财帛"));
        assert!(palace_names.contains(&"官禄"));
    }

    #[test]
    fn test_invalid_input() {
        let calc = ZiWeiCalculator::new();

        let input = ZiWeiInput {
            lunar_year: 1990,
            lunar_month: 13, // 无效月份
            lunar_day: 15,
            shi_chen: 6,
            gender: Gender::Male,
            is_leap_month: false,
        };

        assert!(calc.calculate(&input).is_err());
    }
}
