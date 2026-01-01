//! # 紫微斗数解卦模块
//!
//! 本模块实现紫微斗数命盘的解读功能，包括：
//!
//! - **评分系统**：宫位评分、整体评分、命格等级
//! - **格局识别**：32种常见格局的识别与评估
//! - **四化分析**：四化飞星的计算与影响分析
//! - **大限解读**：十二大限的运势评估
//!
//! ## 模块结构
//!
//! - `enums`: 枚举类型定义（吉凶等级、命格等级、格局类型）
//! - `structs`: 数据结构定义（评分、宫位解读、格局信息等）
//! - `keywords`: 关键词索引表（节省链上存储空间）
//!
//! ## 设计原则
//!
//! 1. **存储最小化**：使用索引和位标志代替完整文本
//! 2. **AI友好**：结构化数据便于AI分析
//! 3. **可扩展性**：预留扩展字段支持多流派
//!
//! ## 使用示例
//!
//! ```ignore
//! use pallet_ziwei::interpretation::*;
//!
//! // 创建宫位解读
//! let mut palace = PalaceInterpretation::default();
//! palace.score = 75;
//! palace.fortune_level = FortuneLevel::Ji;
//! palace.set_star_miao_wang();
//!
//! // 获取关键词
//! let keyword = get_ming_gong_keyword(0); // "贵气"
//!
//! // 判断格局类型
//! let pattern = PatternType::ZiFuTongGong;
//! assert!(pattern.is_auspicious());
//! ```

pub mod enums;
pub mod keywords;
pub mod pattern;
pub mod score;
pub mod sihua;
pub mod structs;

// 重导出常用类型，便于外部使用
pub use enums::*;
pub use keywords::*;
pub use pattern::*;
pub use score::*;
pub use sihua::*;
pub use structs::*;

// ============================================================================
// 模块级别的辅助函数
// ============================================================================

/// 根据评分获取吉凶等级
///
/// # 参数
/// - `score`: 评分（0-100）
///
/// # 返回
/// 对应的吉凶等级
pub fn score_to_fortune_level(score: u8) -> FortuneLevel {
    FortuneLevel::from_score(score)
}

/// 判断格局是否为吉格
///
/// # 参数
/// - `pattern`: 格局类型
///
/// # 返回
/// 是否为吉格
pub fn is_auspicious_pattern(pattern: PatternType) -> bool {
    pattern.is_auspicious()
}

/// 获取格局的基础分数
///
/// # 参数
/// - `pattern`: 格局类型
///
/// # 返回
/// 基础分数（-50 ~ +50）
pub fn get_pattern_base_score(pattern: PatternType) -> i8 {
    pattern.base_score()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_to_fortune_level() {
        assert_eq!(score_to_fortune_level(95), FortuneLevel::DaJi);
        assert_eq!(score_to_fortune_level(80), FortuneLevel::Ji);
        assert_eq!(score_to_fortune_level(50), FortuneLevel::Ping);
        assert_eq!(score_to_fortune_level(15), FortuneLevel::Xiong);
        assert_eq!(score_to_fortune_level(5), FortuneLevel::DaXiong);
    }

    #[test]
    fn test_is_auspicious_pattern() {
        assert!(is_auspicious_pattern(PatternType::ZiFuTongGong));
        assert!(is_auspicious_pattern(PatternType::SanQiJiaHui));
        assert!(!is_auspicious_pattern(PatternType::YangTuoJiaMing));
        assert!(!is_auspicious_pattern(PatternType::SiShaChongMing));
    }

    #[test]
    fn test_get_pattern_base_score() {
        assert_eq!(get_pattern_base_score(PatternType::ZiFuTongGong), 50);
        assert_eq!(get_pattern_base_score(PatternType::SanQiJiaHui), 45);
        assert_eq!(get_pattern_base_score(PatternType::YangTuoJiaMing), -40);
        assert_eq!(get_pattern_base_score(PatternType::SiShaChongMing), -50);
    }

    #[test]
    fn test_module_reexports() {
        // 测试重导出的类型可用
        let _level = FortuneLevel::DaJi;
        let _ming_ge = MingGeLevel::DaGui;
        let _pattern = PatternType::JunChenQingHui;

        // 测试结构体可用
        let _score = ChartOverallScore::default();
        let _palace = PalaceInterpretation::default();

        // 测试关键词函数可用
        let keyword = get_ming_gong_keyword(0);
        assert_eq!(keyword, "贵气");
    }
}
