//! # 八字命理计算模块
//!
//! 根据出生年月日时计算四柱八字

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec, vec::Vec};

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{BaZiInput, BaZiResult, FiveElementsCount, Gender, MajorCycle};

/// 天干
const HEAVENLY_STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

/// 地支
const EARTHLY_BRANCHES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// 天干五行对应
const STEM_ELEMENTS: [usize; 10] = [1, 1, 3, 3, 4, 4, 0, 0, 2, 2]; // 木木火火土土金金水水

/// 地支五行对应
const BRANCH_ELEMENTS: [usize; 12] = [2, 4, 1, 1, 4, 3, 3, 4, 0, 0, 4, 2]; // 水土木木土火火土金金土水

/// 十神名称
const TEN_GODS: [&str; 10] = [
    "比肩", "劫财", "食神", "伤官", "偏财", "正财", "七杀", "正官", "偏印", "正印",
];

/// 八字计算器
pub struct BaZiCalculator;

impl BaZiCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self
    }

    /// 执行八字计算
    pub fn calculate(&self, input: &BaZiInput) -> EnclaveResult<BaZiResult> {
        // 验证输入
        self.validate_input(input)?;

        // 计算四柱
        let year_pillar = self.calculate_year_pillar(input.year);
        let month_pillar = self.calculate_month_pillar(input.year, input.month);
        let day_pillar = self.calculate_day_pillar(input.year, input.month, input.day);
        let hour_pillar = self.calculate_hour_pillar(&day_pillar, input.hour);

        // 提取日主
        let day_master = year_pillar.chars().next().unwrap_or('甲').to_string();

        // 计算五行统计
        let five_elements = self.calculate_five_elements(&year_pillar, &month_pillar, &day_pillar, &hour_pillar);

        // 计算十神
        let ten_gods = self.calculate_ten_gods(&day_pillar, &[&year_pillar, &month_pillar, &hour_pillar]);

        // 计算大运
        let major_cycles = self.calculate_major_cycles(input.year, input.month, input.gender);

        // 生成摘要
        let summary = self.generate_summary(&day_master, &five_elements);

        Ok(BaZiResult {
            year_pillar,
            month_pillar,
            day_pillar,
            hour_pillar,
            day_master,
            five_elements,
            ten_gods,
            major_cycles,
            summary,
        })
    }

    /// 验证输入
    fn validate_input(&self, input: &BaZiInput) -> EnclaveResult<()> {
        if input.year < 1900 || input.year > 2100 {
            return Err(EnclaveError::InvalidInputData);
        }
        if input.month < 1 || input.month > 12 {
            return Err(EnclaveError::InvalidInputData);
        }
        if input.day < 1 || input.day > 31 {
            return Err(EnclaveError::InvalidInputData);
        }
        if input.hour > 23 {
            return Err(EnclaveError::InvalidInputData);
        }
        Ok(())
    }

    /// 计算年柱
    fn calculate_year_pillar(&self, year: u16) -> String {
        // 以立春为界，简化处理使用公历年
        let stem_index = ((year as i32 - 4) % 10).abs() as usize;
        let branch_index = ((year as i32 - 4) % 12).abs() as usize;

        format!("{}{}", HEAVENLY_STEMS[stem_index], EARTHLY_BRANCHES[branch_index])
    }

    /// 计算月柱
    fn calculate_month_pillar(&self, year: u16, month: u8) -> String {
        // 简化算法：根据年干推月干
        let year_stem = ((year as i32 - 4) % 10).abs() as usize;

        // 月干 = (年干 * 2 + 月 - 1) % 10
        let month_stem = (year_stem * 2 + month as usize) % 10;

        // 月支：正月寅月
        let month_branch = (month as usize + 1) % 12;

        format!("{}{}", HEAVENLY_STEMS[month_stem], EARTHLY_BRANCHES[month_branch])
    }

    /// 计算日柱
    fn calculate_day_pillar(&self, year: u16, month: u8, day: u8) -> String {
        // 简化的日柱计算（使用公式近似）
        let y = year as i32;
        let m = month as i32;
        let d = day as i32;

        // 基于高斯公式的简化日柱计算
        let century = y / 100;
        let year_part = y % 100;

        let days = (century / 4 - 2 * century + year_part + year_part / 4 + 13 * (m + 1) / 5 + d - 1) % 60;
        let days = if days < 0 { days + 60 } else { days } as usize;

        let stem_index = days % 10;
        let branch_index = days % 12;

        format!("{}{}", HEAVENLY_STEMS[stem_index], EARTHLY_BRANCHES[branch_index])
    }

    /// 计算时柱
    fn calculate_hour_pillar(&self, day_pillar: &str, hour: u8) -> String {
        // 根据日干推时干
        let day_stem = day_pillar.chars().next().unwrap_or('甲');
        let day_stem_index = HEAVENLY_STEMS.iter().position(|&s| s == day_stem.to_string()).unwrap_or(0);

        // 时支：0-1点子时，2-3点丑时...
        let hour_branch = (hour as usize + 1) / 2 % 12;

        // 时干 = (日干 * 2 + 时支) % 10
        let hour_stem = (day_stem_index * 2 + hour_branch) % 10;

        format!("{}{}", HEAVENLY_STEMS[hour_stem], EARTHLY_BRANCHES[hour_branch])
    }

    /// 计算五行统计
    fn calculate_five_elements(
        &self,
        year: &str,
        month: &str,
        day: &str,
        hour: &str,
    ) -> FiveElementsCount {
        let mut count = FiveElementsCount::default();

        for pillar in &[year, month, day, hour] {
            let chars: Vec<char> = pillar.chars().collect();
            if chars.len() >= 2 {
                // 天干五行
                if let Some(idx) = HEAVENLY_STEMS.iter().position(|&s| s == chars[0].to_string()) {
                    match STEM_ELEMENTS[idx] {
                        0 => count.metal += 1,
                        1 => count.wood += 1,
                        2 => count.water += 1,
                        3 => count.fire += 1,
                        4 => count.earth += 1,
                        _ => {}
                    }
                }

                // 地支五行
                if let Some(idx) = EARTHLY_BRANCHES.iter().position(|&s| s == chars[1].to_string()) {
                    match BRANCH_ELEMENTS[idx] {
                        0 => count.metal += 1,
                        1 => count.wood += 1,
                        2 => count.water += 1,
                        3 => count.fire += 1,
                        4 => count.earth += 1,
                        _ => {}
                    }
                }
            }
        }

        count
    }

    /// 计算十神
    fn calculate_ten_gods(&self, day_pillar: &str, other_pillars: &[&str]) -> Vec<String> {
        let mut gods = Vec::new();

        let day_stem = day_pillar.chars().next().unwrap_or('甲');
        let day_stem_index = HEAVENLY_STEMS.iter().position(|&s| s == day_stem.to_string()).unwrap_or(0);

        for pillar in other_pillars {
            if let Some(stem) = pillar.chars().next() {
                if let Some(idx) = HEAVENLY_STEMS.iter().position(|&s| s == stem.to_string()) {
                    let diff = (idx + 10 - day_stem_index) % 10;
                    gods.push(TEN_GODS[diff].to_string());
                }
            }
        }

        gods
    }

    /// 计算大运
    fn calculate_major_cycles(&self, year: u16, month: u8, gender: Gender) -> Vec<MajorCycle> {
        let mut cycles = Vec::new();

        // 简化计算：根据性别和年干确定顺逆
        let year_stem = ((year as i32 - 4) % 10).abs() as usize;
        let is_yang = year_stem % 2 == 0;
        let forward = matches!((&gender, is_yang), (Gender::Male, true) | (Gender::Female, false));

        // 起运年龄（简化为固定值）
        let start_age = 3u8;

        // 生成10步大运
        for i in 0..10 {
            let age = start_age + i * 10;
            let month_stem = ((year_stem * 2 + month as usize) % 10) as i32;
            let month_branch = ((month as usize + 1) % 12) as i32;

            let offset = if forward { i as i32 } else { -(i as i32) };

            let stem_idx = ((month_stem + offset) % 10 + 10) % 10;
            let branch_idx = ((month_branch + offset) % 12 + 12) % 12;

            cycles.push(MajorCycle {
                start_age: age,
                gan_zhi: format!("{}{}", HEAVENLY_STEMS[stem_idx as usize], EARTHLY_BRANCHES[branch_idx as usize]),
            });
        }

        cycles
    }

    /// 生成摘要
    fn generate_summary(&self, day_master: &str, elements: &FiveElementsCount) -> String {
        let element_names = ["金", "木", "水", "火", "土"];
        let element_counts = [elements.metal, elements.wood, elements.water, elements.fire, elements.earth];

        let max_idx = element_counts.iter().enumerate().max_by_key(|(_, &v)| v).map(|(i, _)| i).unwrap_or(0);
        let min_idx = element_counts.iter().enumerate().min_by_key(|(_, &v)| v).map(|(i, _)| i).unwrap_or(0);

        format!(
            "日主{}，五行{}旺{}弱。金{}木{}水{}火{}土{}。",
            day_master,
            element_names[max_idx],
            element_names[min_idx],
            elements.metal,
            elements.wood,
            elements.water,
            elements.fire,
            elements.earth
        )
    }
}

impl Default for BaZiCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bazi_calculation() {
        let calc = BaZiCalculator::new();

        let input = BaZiInput {
            year: 1990,
            month: 6,
            day: 15,
            hour: 12,
            gender: Gender::Male,
            longitude: None,
        };

        let result = calc.calculate(&input).unwrap();

        assert!(!result.year_pillar.is_empty());
        assert!(!result.month_pillar.is_empty());
        assert!(!result.day_pillar.is_empty());
        assert!(!result.hour_pillar.is_empty());
        assert!(!result.day_master.is_empty());
        assert!(!result.major_cycles.is_empty());
    }

    #[test]
    fn test_invalid_input() {
        let calc = BaZiCalculator::new();

        let input = BaZiInput {
            year: 1800, // 无效年份
            month: 6,
            day: 15,
            hour: 12,
            gender: Gender::Male,
            longitude: None,
        };

        assert!(calc.calculate(&input).is_err());
    }
}
