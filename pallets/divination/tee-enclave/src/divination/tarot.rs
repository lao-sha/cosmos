//! # 塔罗牌计算模块
//!
//! 实现塔罗牌的抽牌与解读算法

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec, vec::Vec};

use crate::error::{EnclaveError, EnclaveResult};
use crate::types::{TarotCard, TarotInput, TarotResult};

/// 大阿卡纳牌名（22张）
const MAJOR_ARCANA: [&str; 22] = [
    "愚者", "魔术师", "女祭司", "女皇", "皇帝", "教皇",
    "恋人", "战车", "力量", "隐者", "命运之轮", "正义",
    "倒吊人", "死神", "节制", "恶魔", "塔", "星星",
    "月亮", "太阳", "审判", "世界",
];

/// 大阿卡纳正位含义
const MAJOR_MEANINGS_UPRIGHT: [&str; 22] = [
    "新的开始、冒险、无忧无虑、信任直觉",
    "创造力、技能、意志力、专注",
    "直觉、神秘、潜意识、内在智慧",
    "丰收、母性、创造力、自然",
    "权威、结构、控制、父性",
    "传统、信仰、遵守规则、精神指引",
    "爱情、和谐、关系、选择",
    "胜利、决心、意志力、成功",
    "内在力量、勇气、耐心、自信",
    "内省、独处、寻求真理、智慧",
    "命运、转折点、机遇、变化",
    "公正、真相、因果、法律",
    "暂停、牺牲、新视角、放手",
    "结束、转变、过渡、新生",
    "平衡、适度、耐心、目标",
    "束缚、诱惑、物质主义、阴影",
    "突变、觉醒、解放、真相",
    "希望、灵感、平静、更新",
    "幻觉、恐惧、焦虑、潜意识",
    "快乐、成功、活力、积极",
    "觉醒、更新、召唤、重生",
    "完成、整合、成就、圆满",
];

/// 大阿卡纳逆位含义
const MAJOR_MEANINGS_REVERSED: [&str; 22] = [
    "鲁莽、冒险、愚蠢、不成熟",
    "欺骗、操纵、缺乏技能",
    "隐藏的议程、断开连接、撤退",
    "创造力受阻、依赖、空虚",
    "专横、僵化、缺乏纪律",
    "反叛、非传统、新方法",
    "不和谐、不平衡、错误选择",
    "缺乏控制、攻击性、失败",
    "自我怀疑、软弱、不安全",
    "孤立、偏执、被孤立",
    "坏运气、抗拒改变、打破循环",
    "不公正、不诚实、缺乏责任",
    "拖延、抗拒、固执",
    "抗拒改变、无法前进、停滞",
    "失衡、过度、缺乏长期愿景",
    "解脱、收回力量、面对恐惧",
    "避免灾难、推迟不可避免的事",
    "绝望、信念丧失、断开连接",
    "释放恐惧、驱除幻觉、清晰",
    "过度自信、悲观、不切实际",
    "自我怀疑、拒绝改变、内疚",
    "寻求结束、延迟、感觉不完整",
];

/// 牌阵类型
const SPREAD_NAMES: [&str; 4] = ["单牌", "三牌阵", "凯尔特十字", "马蹄形"];

/// 三牌阵位置含义
const THREE_CARD_POSITIONS: [&str; 3] = ["过去", "现在", "未来"];

/// 塔罗牌计算器
pub struct TarotCalculator;

impl TarotCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self
    }

    /// 执行塔罗牌占卜
    pub fn calculate(&self, input: &TarotInput) -> EnclaveResult<TarotResult> {
        // 验证牌阵类型
        if input.spread > 3 {
            return Err(EnclaveError::InvalidInputData);
        }

        // 根据牌阵确定抽牌数量
        let card_count = match input.spread {
            0 => 1,  // 单牌
            1 => 3,  // 三牌阵
            2 => 10, // 凯尔特十字
            3 => 7,  // 马蹄形
            _ => 1,
        };

        // 使用种子生成伪随机序列
        let cards = self.draw_cards(&input.seed, card_count, input.spread);

        // 生成综合解读
        let interpretation = self.generate_interpretation(&cards, input.spread);

        Ok(TarotResult {
            cards,
            spread_name: SPREAD_NAMES[input.spread as usize].to_string(),
            interpretation,
        })
    }

    /// 抽取塔罗牌
    fn draw_cards(&self, seed: &[u8; 32], count: usize, spread: u8) -> Vec<TarotCard> {
        let mut cards = Vec::with_capacity(count);
        let mut used_indices = Vec::new();

        for i in 0..count {
            // 使用种子的不同部分生成每张牌
            let idx = self.next_card_index(seed, i, &used_indices);
            used_indices.push(idx);

            // 判断正逆位
            let reversed = self.is_reversed(seed, i);

            // 获取位置含义
            let position_meaning = self.get_position_meaning(spread, i);

            // 获取牌义
            let meaning = if reversed {
                MAJOR_MEANINGS_REVERSED[idx].to_string()
            } else {
                MAJOR_MEANINGS_UPRIGHT[idx].to_string()
            };

            cards.push(TarotCard {
                name: MAJOR_ARCANA[idx].to_string(),
                number: idx as u8,
                reversed,
                position_meaning,
                meaning,
            });
        }

        cards
    }

    /// 生成下一张牌的索引
    fn next_card_index(&self, seed: &[u8; 32], position: usize, used: &[usize]) -> usize {
        let mut idx = 0;
        let mut attempts = 0;

        loop {
            // 使用种子的不同字节组合生成索引
            let seed_pos = (position * 4 + attempts) % 32;
            let raw = (seed[seed_pos] as usize
                + seed[(seed_pos + 1) % 32] as usize * 256
                + position * 37
                + attempts * 13)
                % 22;

            idx = raw;
            attempts += 1;

            // 确保不重复抽取
            if !used.contains(&idx) || attempts > 100 {
                break;
            }
        }

        idx
    }

    /// 判断是否逆位
    fn is_reversed(&self, seed: &[u8; 32], position: usize) -> bool {
        let seed_pos = (position * 3 + 7) % 32;
        seed[seed_pos] % 2 == 1
    }

    /// 获取位置含义
    fn get_position_meaning(&self, spread: u8, position: usize) -> String {
        match spread {
            0 => "当前状况".to_string(),
            1 => {
                if position < 3 {
                    THREE_CARD_POSITIONS[position].to_string()
                } else {
                    "额外信息".to_string()
                }
            }
            2 => {
                // 凯尔特十字牌阵
                let meanings = [
                    "当前处境", "挑战", "意识", "潜意识",
                    "过去", "近期未来", "自我认知", "外部影响",
                    "希望与恐惧", "最终结果",
                ];
                if position < 10 {
                    meanings[position].to_string()
                } else {
                    "额外信息".to_string()
                }
            }
            3 => {
                // 马蹄形牌阵
                let meanings = [
                    "过去", "现在", "未来", "环境",
                    "建议", "障碍", "结果",
                ];
                if position < 7 {
                    meanings[position].to_string()
                } else {
                    "额外信息".to_string()
                }
            }
            _ => "未知位置".to_string(),
        }
    }

    /// 生成综合解读
    fn generate_interpretation(&self, cards: &[TarotCard], spread: u8) -> String {
        if cards.is_empty() {
            return "无法解读".to_string();
        }

        let spread_name = SPREAD_NAMES[spread as usize];

        let card_summary: Vec<String> = cards
            .iter()
            .map(|c| {
                let direction = if c.reversed { "逆位" } else { "正位" };
                format!("【{}】{}{}：{}", c.position_meaning, c.name, direction, c.meaning)
            })
            .collect();

        format!(
            "{}解读：\n\n{}\n\n综合分析：牌阵显示了一个完整的画面，需要结合具体问题进行深入解读。",
            spread_name,
            card_summary.join("\n\n")
        )
    }
}

impl Default for TarotCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tarot_single_card() {
        let calc = TarotCalculator::new();

        let input = TarotInput {
            spread: 0,
            seed: [42u8; 32],
            timestamp: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        assert_eq!(result.cards.len(), 1);
        assert_eq!(result.spread_name, "单牌");
    }

    #[test]
    fn test_tarot_three_cards() {
        let calc = TarotCalculator::new();

        let input = TarotInput {
            spread: 1,
            seed: [42u8; 32],
            timestamp: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        assert_eq!(result.cards.len(), 3);
        assert_eq!(result.spread_name, "三牌阵");

        // 检查位置含义
        assert_eq!(result.cards[0].position_meaning, "过去");
        assert_eq!(result.cards[1].position_meaning, "现在");
        assert_eq!(result.cards[2].position_meaning, "未来");
    }

    #[test]
    fn test_tarot_celtic_cross() {
        let calc = TarotCalculator::new();

        let input = TarotInput {
            spread: 2,
            seed: [42u8; 32],
            timestamp: 1704067200,
        };

        let result = calc.calculate(&input).unwrap();

        assert_eq!(result.cards.len(), 10);
        assert_eq!(result.spread_name, "凯尔特十字");
    }

    #[test]
    fn test_different_seeds() {
        let calc = TarotCalculator::new();

        let input1 = TarotInput {
            spread: 1,
            seed: [1u8; 32],
            timestamp: 1704067200,
        };

        let input2 = TarotInput {
            spread: 1,
            seed: [2u8; 32],
            timestamp: 1704067200,
        };

        let result1 = calc.calculate(&input1).unwrap();
        let result2 = calc.calculate(&input2).unwrap();

        // 不同种子应该产生不同结果
        let cards1: Vec<u8> = result1.cards.iter().map(|c| c.number).collect();
        let cards2: Vec<u8> = result2.cards.iter().map(|c| c.number).collect();

        // 至少有一张牌不同
        assert!(cards1 != cards2 || result1.cards[0].reversed != result2.cards[0].reversed);
    }

    #[test]
    fn test_invalid_spread() {
        let calc = TarotCalculator::new();

        let input = TarotInput {
            spread: 99,
            seed: [42u8; 32],
            timestamp: 1704067200,
        };

        assert!(calc.calculate(&input).is_err());
    }
}
