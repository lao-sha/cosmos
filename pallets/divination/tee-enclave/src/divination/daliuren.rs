//! # 大六壬计算模块
//!
//! 实现大六壬的起课与排盘算法

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec, vec::Vec};

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{DaLiuRenInput, DaLiuRenResult};

/// 地支
const EARTHLY_BRANCHES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// 天将（十二天将）
const HEAVENLY_GENERALS: [&str; 12] = [
    "贵人", "螣蛇", "朱雀", "六合", "勾陈", "青龙",
    "天空", "白虎", "太常", "玄武", "太阴", "天后",
];

/// 四课名称
const FOUR_LESSONS: [&str; 4] = ["一课", "二课", "三课", "四课"];

/// 三传名称
const THREE_TRANSMISSIONS: [&str; 3] = ["初传", "中传", "末传"];

/// 大六壬计算器
pub struct DaLiuRenCalculator;

impl DaLiuRenCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self
    }

    /// 执行大六壬排盘
    pub fn calculate(&self, input: &DaLiuRenInput) -> EnclaveResult<DaLiuRenResult> {
        // 从时间戳计算日干支和时辰
        let (day_gan, day_zhi, hour_zhi) = self.parse_datetime(input.datetime);

        // 计算月将
        let yue_jiang = self.calculate_yue_jiang(input.datetime);

        // 计算天地盘
        let (heaven_plate, earth_plate) = self.calculate_plates(yue_jiang, hour_zhi);

        // 起四课
        let four_lessons = self.generate_four_lessons(day_gan, day_zhi, &heaven_plate, &earth_plate);

        // 发三传
        let three_transmissions = self.generate_three_transmissions(&four_lessons);

        // 排天将
        let heavenly_generals = self.arrange_heavenly_generals(input.datetime);

        // 生成分析
        let analysis = self.generate_analysis(&four_lessons, &three_transmissions);

        Ok(DaLiuRenResult {
            four_lessons,
            three_transmissions,
            heavenly_generals,
            analysis,
        })
    }

    /// 解析时间戳为干支
    fn parse_datetime(&self, timestamp: u64) -> (usize, usize, usize) {
        // 简化计算
        let days = (timestamp / 86400) as usize;
        let hours = ((timestamp % 86400) / 3600) as usize;

        // 日干
        let day_gan = (days + 4) % 10; // 从甲开始

        // 日支
        let day_zhi = (days + 4) % 12;

        // 时辰
        let hour_zhi = ((hours + 1) / 2) % 12;

        (day_gan, day_zhi, hour_zhi)
    }

    /// 计算月将
    fn calculate_yue_jiang(&self, timestamp: u64) -> usize {
        // 简化：根据时间戳的月份部分计算
        // 月将从正月亥将开始
        let month = ((timestamp / 2592000) % 12) as usize;
        (11 - month + 12) % 12 // 从亥倒数
    }

    /// 计算天地盘
    fn calculate_plates(&self, yue_jiang: usize, hour_zhi: usize) -> (Vec<usize>, Vec<usize>) {
        // 地盘固定：子丑寅卯...
        let earth_plate: Vec<usize> = (0..12).collect();

        // 天盘：月将加时，然后顺排
        let offset = (yue_jiang + 12 - hour_zhi) % 12;
        let heaven_plate: Vec<usize> = (0..12).map(|i| (i + offset) % 12).collect();

        (heaven_plate, earth_plate)
    }

    /// 生成四课
    fn generate_four_lessons(
        &self,
        day_gan: usize,
        day_zhi: usize,
        heaven_plate: &[usize],
        earth_plate: &[usize],
    ) -> Vec<String> {
        let mut lessons = Vec::with_capacity(4);

        // 一课：日干上神
        let lesson1_earth = day_gan % 12;
        let lesson1_heaven = heaven_plate[lesson1_earth];
        lessons.push(format!(
            "{}上{} - {}",
            EARTHLY_BRANCHES[lesson1_earth],
            EARTHLY_BRANCHES[lesson1_heaven],
            FOUR_LESSONS[0]
        ));

        // 二课：日干上神之上神
        let lesson2_earth = lesson1_heaven;
        let lesson2_heaven = heaven_plate[lesson2_earth];
        lessons.push(format!(
            "{}上{} - {}",
            EARTHLY_BRANCHES[lesson2_earth],
            EARTHLY_BRANCHES[lesson2_heaven],
            FOUR_LESSONS[1]
        ));

        // 三课：日支上神
        let lesson3_earth = day_zhi;
        let lesson3_heaven = heaven_plate[lesson3_earth];
        lessons.push(format!(
            "{}上{} - {}",
            EARTHLY_BRANCHES[lesson3_earth],
            EARTHLY_BRANCHES[lesson3_heaven],
            FOUR_LESSONS[2]
        ));

        // 四课：日支上神之上神
        let lesson4_earth = lesson3_heaven;
        let lesson4_heaven = heaven_plate[lesson4_earth];
        lessons.push(format!(
            "{}上{} - {}",
            EARTHLY_BRANCHES[lesson4_earth],
            EARTHLY_BRANCHES[lesson4_heaven],
            FOUR_LESSONS[3]
        ));

        lessons
    }

    /// 发三传
    fn generate_three_transmissions(&self, four_lessons: &[String]) -> Vec<String> {
        // 简化算法：使用贼克法的简化版本
        let mut transmissions = Vec::with_capacity(3);

        // 从四课中提取地支索引（简化处理）
        for (i, name) in THREE_TRANSMISSIONS.iter().enumerate() {
            // 简化：按顺序从四课中取
            let lesson_idx = i % 4;
            let parts: Vec<&str> = four_lessons[lesson_idx].split("上").collect();
            if let Some(zhi) = parts.first() {
                transmissions.push(format!("{} - {}", zhi.trim(), name));
            } else {
                transmissions.push(format!("? - {}", name));
            }
        }

        transmissions
    }

    /// 排天将
    fn arrange_heavenly_generals(&self, timestamp: u64) -> Vec<String> {
        // 简化：根据时间戳确定贵人位置，然后顺排
        let seed = (timestamp / 7200) as usize;
        let start = seed % 12;

        (0..12)
            .map(|i| HEAVENLY_GENERALS[(start + i) % 12].to_string())
            .collect()
    }

    /// 生成分析
    fn generate_analysis(&self, four_lessons: &[String], three_transmissions: &[String]) -> String {
        format!(
            "四课：{}、{}、{}、{}。三传：{}、{}、{}。课体需详细分析以定吉凶。",
            four_lessons.get(0).unwrap_or(&String::new()),
            four_lessons.get(1).unwrap_or(&String::new()),
            four_lessons.get(2).unwrap_or(&String::new()),
            four_lessons.get(3).unwrap_or(&String::new()),
            three_transmissions.get(0).unwrap_or(&String::new()),
            three_transmissions.get(1).unwrap_or(&String::new()),
            three_transmissions.get(2).unwrap_or(&String::new()),
        )
    }
}

impl Default for DaLiuRenCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daliuren_calculation() {
        let calc = DaLiuRenCalculator::new();

        let input = DaLiuRenInput {
            datetime: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        assert_eq!(result.four_lessons.len(), 4);
        assert_eq!(result.three_transmissions.len(), 3);
        assert_eq!(result.heavenly_generals.len(), 12);
        assert!(!result.analysis.is_empty());
    }

    #[test]
    fn test_four_lessons_format() {
        let calc = DaLiuRenCalculator::new();

        let input = DaLiuRenInput {
            datetime: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        // 检查四课格式
        for lesson in &result.four_lessons {
            assert!(lesson.contains("上"));
            assert!(lesson.contains("课"));
        }
    }

    #[test]
    fn test_heavenly_generals() {
        let calc = DaLiuRenCalculator::new();

        let input = DaLiuRenInput {
            datetime: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        // 检查十二天将是否完整
        assert_eq!(result.heavenly_generals.len(), 12);

        // 检查是否包含贵人
        assert!(result.heavenly_generals.iter().any(|g| g.contains("贵人")));
    }
}
