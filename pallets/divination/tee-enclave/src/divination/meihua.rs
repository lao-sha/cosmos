//! # 梅花易数计算模块
//!
//! 实现梅花易数的起卦与解卦算法

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec::Vec};

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{HexagramInfo, MeiHuaInput, MeiHuaResult};

/// 八卦名称
const BAGUA_NAMES: [&str; 8] = ["乾", "兑", "离", "震", "巽", "坎", "艮", "坤"];

/// 八卦五行
const BAGUA_ELEMENTS: [&str; 8] = ["金", "金", "火", "木", "木", "水", "土", "土"];

/// 六十四卦名称
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

/// 卦辞（简化版）
const HEXAGRAM_TEXTS: [&str; 64] = [
    "元亨利贞", "履虎尾，不咥人，亨", "同人于野，亨", "元亨利贞，其匪正有眚",
    "女壮，勿用取女", "有孚窒惕，中吉，终凶", "亨，小利贞", "否之匪人，不利君子贞",
    "扬于王庭，孚号有厉", "亨，利贞", "已日乃孚，元亨，利贞", "元亨利贞，无咎",
    "栋桡，利有攸往", "亨，贞大人吉", "亨，利贞，取女吉", "亨，王假有庙，利见大人",
    "元亨", "小事吉", "利贞，亨", "亨，利用狱",
    "元吉，无咎，利贞", "亨", "亨，小利贞", "康侯用锡马蕃庶",
    "利贞", "征凶，无攸利", "亨，利贞", "元亨，利贞，无咎",
    "利贞，取女吉", "利西南", "亨，利贞", "利建侯行师",
    "亨", "豚鱼吉，利涉大川", "利女贞", "利有攸往，利涉大川",
    "小亨，利有攸往", "亨，利贞，用见大人", "女归吉，利贞", "盥而不荐，有孚顒若",
    "有孚，光亨，贞吉", "亨，苦节不可贞", "亨，利贞，初吉终乱", "元亨，利贞，勿用有攸往",
    "改邑不改井，无丧无得", "习坎，有孚，维心亨", "往蹇，来誉", "吉，原筮元永贞",
    "利贞，不家食吉", "有孚，元吉，无咎", "亨，小利贞", "贞吉，养正则吉",
    "元亨，利涉大川", "亨，匪我求童蒙", "亨", "不利有攸往",
    "小往大来，吉亨", "元亨，利贞", "利艰贞", "亨，七日来复",
    "元亨，利贞", "贞吉，无咎", "亨，君子有终", "元亨，利牝马之贞",
];

/// 梅花易数计算器
pub struct MeiHuaCalculator;

impl MeiHuaCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self
    }

    /// 执行梅花易数计算
    pub fn calculate(&self, input: &MeiHuaInput) -> EnclaveResult<MeiHuaResult> {
        // 根据起卦方式计算上下卦和动爻
        let (upper, lower, moving) = match input.method {
            0 => self.time_method(input.timestamp)?,
            1 => self.number_method(input.upper_num, input.lower_num, input.moving_line)?,
            _ => return Err(EnclaveError::InvalidInputData),
        };

        // 计算本卦
        let original = self.get_hexagram_info(upper, lower);

        // 计算变卦（动爻变化）
        let (changed_upper, changed_lower) = self.get_changed_gua(upper, lower, moving);
        let changed = self.get_hexagram_info(changed_upper, changed_lower);

        // 计算互卦
        let (mutual_upper, mutual_lower) = self.get_mutual_gua(upper, lower);
        let mutual = self.get_hexagram_info(mutual_upper, mutual_lower);

        // 确定体用卦
        let (ti_gua, yong_gua) = if moving <= 3 {
            // 动爻在下卦，下卦为用，上卦为体
            (BAGUA_NAMES[upper as usize].to_string(), BAGUA_NAMES[lower as usize].to_string())
        } else {
            // 动爻在上卦，上卦为用，下卦为体
            (BAGUA_NAMES[lower as usize].to_string(), BAGUA_NAMES[upper as usize].to_string())
        };

        // 生成分析
        let analysis = self.generate_analysis(&ti_gua, &yong_gua, &original.name);

        Ok(MeiHuaResult {
            original_hexagram: original,
            changed_hexagram: changed,
            mutual_hexagram: mutual,
            ti_gua,
            yong_gua,
            moving_line: moving,
            analysis,
        })
    }

    /// 时间起卦法
    fn time_method(&self, timestamp: u64) -> EnclaveResult<(u8, u8, u8)> {
        // 从时间戳提取年月日时
        // 简化处理：直接用时间戳的不同部分
        let year = ((timestamp / 31536000) % 100) as u16; // 年
        let month = ((timestamp / 2592000) % 12 + 1) as u8; // 月
        let day = ((timestamp / 86400) % 31 + 1) as u8; // 日
        let hour = ((timestamp / 3600) % 24) as u8; // 时

        // 上卦 = (年+月+日) % 8
        let upper = ((year as u32 + month as u32 + day as u32) % 8) as u8;

        // 下卦 = (年+月+日+时) % 8
        let lower = ((year as u32 + month as u32 + day as u32 + hour as u32) % 8) as u8;

        // 动爻 = (年+月+日+时) % 6 + 1
        let moving = (((year as u32 + month as u32 + day as u32 + hour as u32) % 6) + 1) as u8;

        Ok((upper, lower, moving))
    }

    /// 数字起卦法
    fn number_method(
        &self,
        upper_num: Option<u16>,
        lower_num: Option<u16>,
        moving_line: Option<u8>,
    ) -> EnclaveResult<(u8, u8, u8)> {
        let upper = upper_num.ok_or(EnclaveError::InvalidInputData)? % 8;
        let lower = lower_num.ok_or(EnclaveError::InvalidInputData)? % 8;
        let moving = moving_line.map(|m| (m - 1) % 6 + 1).unwrap_or(1);

        Ok((upper as u8, lower as u8, moving))
    }

    /// 获取卦象信息
    fn get_hexagram_info(&self, upper: u8, lower: u8) -> HexagramInfo {
        let hex_index = (upper as usize * 8 + lower as usize) % 64;

        HexagramInfo {
            name: HEXAGRAM_NAMES[hex_index].to_string(),
            number: (hex_index + 1) as u8,
            upper: BAGUA_NAMES[upper as usize].to_string(),
            lower: BAGUA_NAMES[lower as usize].to_string(),
            text: HEXAGRAM_TEXTS[hex_index].to_string(),
        }
    }

    /// 计算变卦
    fn get_changed_gua(&self, upper: u8, lower: u8, moving: u8) -> (u8, u8) {
        if moving <= 3 {
            // 动爻在下卦
            let new_lower = lower ^ (1 << (moving - 1));
            (upper, new_lower % 8)
        } else {
            // 动爻在上卦
            let new_upper = upper ^ (1 << (moving - 4));
            (new_upper % 8, lower)
        }
    }

    /// 计算互卦
    fn get_mutual_gua(&self, upper: u8, lower: u8) -> (u8, u8) {
        // 互卦：取2、3、4爻为下卦，3、4、5爻为上卦
        // 简化处理
        let combined = (upper << 3) | lower;
        let mutual_lower = (combined >> 1) & 0x07;
        let mutual_upper = (combined >> 2) & 0x07;
        (mutual_upper as u8, mutual_lower as u8)
    }

    /// 生成分析
    fn generate_analysis(&self, ti_gua: &str, yong_gua: &str, hexagram_name: &str) -> String {
        let ti_element = BAGUA_NAMES.iter()
            .position(|&n| n == ti_gua)
            .map(|i| BAGUA_ELEMENTS[i])
            .unwrap_or("土");

        let yong_element = BAGUA_NAMES.iter()
            .position(|&n| n == yong_gua)
            .map(|i| BAGUA_ELEMENTS[i])
            .unwrap_or("土");

        format!(
            "本卦{}，体卦{}属{}，用卦{}属{}。体用生克关系需详细分析。",
            hexagram_name, ti_gua, ti_element, yong_gua, yong_element
        )
    }
}

impl Default for MeiHuaCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meihua_time_method() {
        let calc = MeiHuaCalculator::new();

        let input = MeiHuaInput {
            method: 0,
            upper_num: None,
            lower_num: None,
            moving_line: None,
            timestamp: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        assert!(!result.original_hexagram.name.is_empty());
        assert!(!result.changed_hexagram.name.is_empty());
        assert!(!result.mutual_hexagram.name.is_empty());
        assert!(result.moving_line >= 1 && result.moving_line <= 6);
    }

    #[test]
    fn test_meihua_number_method() {
        let calc = MeiHuaCalculator::new();

        let input = MeiHuaInput {
            method: 1,
            upper_num: Some(5),
            lower_num: Some(3),
            moving_line: Some(2),
            timestamp: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        assert!(!result.original_hexagram.name.is_empty());
        assert_eq!(result.moving_line, 2);
    }

    #[test]
    fn test_invalid_method() {
        let calc = MeiHuaCalculator::new();

        let input = MeiHuaInput {
            method: 99,
            upper_num: None,
            lower_num: None,
            moving_line: None,
            timestamp: 1704067200,
        };

        assert!(calc.calculate(&input).is_err());
    }
}
