//! 塔罗牌排盘算法模块
//!
//! 本模块实现塔罗牌占卜的核心算法，包括：
//! - 随机洗牌与抽牌
//! - 基于时间的起卦
//! - 基于数字的起卦
//! - 正逆位判定
//! - 切牌机制

use crate::types::*;
use sp_std::prelude::*;

/// 塔罗牌总数（78张）
pub const TOTAL_CARDS: u8 = 78;

/// 大阿卡纳数量（22张）
pub const MAJOR_ARCANA_COUNT: u8 = 22;

/// 小阿卡纳数量（56张）
pub const MINOR_ARCANA_COUNT: u8 = 56;

/// 每种花色的牌数（14张）
pub const CARDS_PER_SUIT: u8 = 14;

/// 简易 BLAKE2 哈希实现（用于 no_std 环境）
///
/// 使用多轮混合产生伪随机输出，替代 sp_io::hashing
fn simple_hash(input: &[u8; 32]) -> [u8; 32] {
    let mut output = *input;

    // 多轮混合
    for round in 0..4 {
        for i in 0..32 {
            let j = (i + 1) % 32;
            let k = (i + 17) % 32;
            // 混合相邻字节
            output[i] = output[i]
                .wrapping_add(output[j])
                .wrapping_mul(0x6D)
                .rotate_left(((round + i) % 8) as u32);
            output[i] ^= output[k];
        }
    }

    output
}

/// 使用随机种子生成抽牌序列（增强版）
///
/// # 算法说明
/// 使用改进的 Fisher-Yates 洗牌算法，通过哈希链生成更多随机字节，
/// 解决原版在抽牌数量 > 32 时种子重复使用的问题。
///
/// # 参数
/// - `random_seed`: 32字节随机种子（来自链上随机源）
/// - `count`: 需要抽取的牌数
///
/// # 返回
/// - 抽取的牌ID列表和对应的正逆位
pub fn draw_cards_random(random_seed: &[u8; 32], count: u8) -> Vec<(u8, bool)> {
    let count = count.min(TOTAL_CARDS) as usize;
    let mut result = Vec::with_capacity(count);

    // 初始化牌组（0-77）
    let mut deck: Vec<u8> = (0..TOTAL_CARDS).collect();

    // 使用哈希链生成更多随机字节，避免种子重复使用
    let mut current_seed = *random_seed;

    // 使用种子进行洗牌
    for i in 0..count {
        // 每16次迭代生成新的随机种子（哈希链）
        if i > 0 && i % 16 == 0 {
            current_seed = simple_hash(&current_seed);
        }

        // 从当前种子中提取随机字节
        let seed_index = i % 32;
        let random_byte = current_seed[seed_index];

        // 增加额外的混合步骤，提高随机性
        let mixed_byte = random_byte ^ current_seed[(i + 7) % 32];

        // 计算交换位置
        let remaining = TOTAL_CARDS as usize - i;
        let swap_offset = (mixed_byte as usize) % remaining;
        let swap_index = i + swap_offset;

        // 交换牌
        deck.swap(i, swap_index);

        // 判断正逆位（使用不同的混合策略）
        let position_byte = current_seed[(i + 16) % 32] ^ current_seed[(i + 23) % 32];
        let is_reversed = position_byte & 1 == 1;

        result.push((deck[i], is_reversed));
    }

    result
}

/// 基于时间戳生成抽牌序列（增强版）
///
/// # 算法说明
/// 使用完整的时间戳字节，结合区块哈希和区块号生成更高熵值的种子。
/// 通过多层哈希混合确保结果难以预测。
///
/// # 参数
/// - `timestamp`: Unix时间戳（秒）
/// - `block_hash`: 当前区块哈希
/// - `block_number`: 当前区块号（新增参数，增加熵源）
/// - `count`: 需要抽取的牌数
///
/// # 返回
/// - 抽取的牌ID列表和对应的正逆位
pub fn draw_cards_by_time(
    timestamp: u64,
    block_hash: &[u8; 32],
    block_number: u64,
    count: u8,
) -> Vec<(u8, bool)> {
    let mut seed = [0u8; 32];

    // 完整时间戳编码（8字节）- 包含完整的时间信息
    let time_bytes = timestamp.to_le_bytes();
    seed[0..8].copy_from_slice(&time_bytes);

    // 区块号编码（8字节）- 增加额外熵源
    let block_num_bytes = block_number.to_le_bytes();
    seed[8..16].copy_from_slice(&block_num_bytes);

    // 时间分量的非线性混合（用于后半部分）
    let seconds = (timestamp % 60) as u8;
    let minutes = ((timestamp / 60) % 60) as u8;
    let hours = ((timestamp / 3600) % 24) as u8;
    let day_of_year = ((timestamp / 86400) % 365) as u16;
    let year_offset = ((timestamp / 31536000) % 100) as u8; // 年份偏移

    seed[16] = seconds.wrapping_mul(minutes).wrapping_add(hours);
    seed[17] = (day_of_year & 0xFF) as u8;
    seed[18] = ((day_of_year >> 8) as u8) ^ year_offset;
    seed[19] = hours.wrapping_mul(0x5A) ^ minutes;

    // 混入完整区块哈希（异或操作）
    for i in 0..32 {
        seed[i] ^= block_hash[i];
    }

    // 最终哈希混合，确保输出均匀分布
    let final_seed = simple_hash(&seed);

    draw_cards_random(&final_seed, count)
}

/// 基于时间戳生成抽牌序列（兼容旧接口）
///
/// 保留旧的三参数接口以兼容现有代码，内部使用 block_number = 0
#[allow(dead_code)]
pub fn draw_cards_by_time_legacy(
    timestamp: u64,
    block_hash: &[u8; 32],
    count: u8,
) -> Vec<(u8, bool)> {
    draw_cards_by_time(timestamp, block_hash, 0, count)
}

/// 基于用户数字生成抽牌序列
///
/// # 算法说明
/// 将用户提供的数字作为随机种子的一部分，结合区块哈希生成牌序
/// 这种方式让用户参与到随机过程中，增加仪式感
///
/// # 参数
/// - `numbers`: 用户提供的数字列表
/// - `block_hash`: 当前区块哈希
/// - `count`: 需要抽取的牌数
///
/// # 返回
/// - 抽取的牌ID列表和对应的正逆位
pub fn draw_cards_by_numbers(numbers: &[u16], block_hash: &[u8; 32], count: u8) -> Vec<(u8, bool)> {
    let mut seed = [0u8; 32];

    // 将用户数字编码到种子中
    for (i, num) in numbers.iter().take(16).enumerate() {
        seed[i * 2] = (*num & 0xFF) as u8;
        seed[i * 2 + 1] = ((*num >> 8) & 0xFF) as u8;
    }

    // 混入区块哈希
    for i in 0..32 {
        seed[i] ^= block_hash[i];
    }

    draw_cards_random(&seed, count)
}

/// 计算牌组能量分布
///
/// 分析抽取的牌中大阿卡纳和各花色的分布
///
/// # 参数
/// - `cards`: 抽取的牌ID列表
///
/// # 返回
/// - (大阿卡纳数, 权杖数, 圣杯数, 宝剑数, 星币数)
pub fn analyze_element_distribution(cards: &[u8]) -> (u8, u8, u8, u8, u8) {
    let mut major = 0u8;
    let mut wands = 0u8;
    let mut cups = 0u8;
    let mut swords = 0u8;
    let mut pentacles = 0u8;

    for &card_id in cards {
        let card = TarotCard::from_id(card_id);
        match card.card_type {
            CardType::MajorArcana => major += 1,
            CardType::MinorArcana => match card.suit {
                Suit::Wands => wands += 1,
                Suit::Cups => cups += 1,
                Suit::Swords => swords += 1,
                Suit::Pentacles => pentacles += 1,
                Suit::None => {}
            },
        }
    }

    (major, wands, cups, swords, pentacles)
}

/// 计算逆位比例
///
/// # 参数
/// - `positions`: 正逆位列表
///
/// # 返回
/// - 逆位比例（0-100）
pub fn calculate_reversed_ratio(positions: &[bool]) -> u8 {
    if positions.is_empty() {
        return 0;
    }

    let reversed_count = positions.iter().filter(|&&r| r).count();
    ((reversed_count * 100) / positions.len()) as u8
}

/// 判断是否包含特殊牌组合
///
/// 检测一些有特殊意义的牌组合
///
/// # 参数
/// - `cards`: 抽取的牌ID列表
///
/// # 返回
/// - 是否包含特殊组合
pub fn has_special_combination(cards: &[u8]) -> bool {
    // 检查是否同时出现愚者(0)和世界(21)
    let has_fool = cards.contains(&0);
    let has_world = cards.contains(&21);

    if has_fool && has_world {
        return true;
    }

    // 检查是否有三张或以上的大阿卡纳
    let major_count = cards.iter().filter(|&&c| c < 22).count();
    if major_count >= 3 {
        return true;
    }

    // 检查是否有同花色的三连号
    for suit_start in [22u8, 36, 50, 64] {
        let suit_cards: Vec<u8> = cards
            .iter()
            .filter(|&&c| c >= suit_start && c < suit_start + 14)
            .map(|&c| c - suit_start)
            .collect();

        if suit_cards.len() >= 3 {
            // 检查连续性
            let mut sorted = suit_cards.clone();
            sorted.sort();
            for i in 0..sorted.len() - 2 {
                if sorted[i] + 1 == sorted[i + 1] && sorted[i + 1] + 1 == sorted[i + 2] {
                    return true;
                }
            }
        }
    }

    false
}

/// 验证抽牌结果的有效性
///
/// # 参数
/// - `cards`: 抽取的牌ID列表
///
/// # 返回
/// - 是否有效（无重复、ID在范围内）
pub fn validate_drawn_cards(cards: &[u8]) -> bool {
    // 检查范围
    if cards.iter().any(|&c| c >= TOTAL_CARDS) {
        return false;
    }

    // 检查重复
    let mut seen = [false; 78];
    for &card_id in cards {
        if seen[card_id as usize] {
            return false;
        }
        seen[card_id as usize] = true;
    }

    true
}

// ==================== 切牌机制 ====================

/// 切牌算法
///
/// 模拟真实塔罗牌的切牌过程：将牌组在指定位置分成两部分，然后交换顺序。
/// 这个过程增加了占卜的仪式感，同时也为用户提供了参与随机过程的机会。
///
/// # 参数
/// - `deck`: 待切牌的牌组（会被原地修改）
/// - `cut_position`: 切牌位置（1 到 len-1），如果为 0 则使用种子随机决定
/// - `seed`: 随机种子（用于 cut_position 为 0 时生成随机位置）
fn cut_deck(deck: &mut [u8], cut_position: u8, seed: &[u8; 32]) {
    let len = deck.len();
    if len < 2 {
        return;
    }

    // 确定切牌位置
    let pos = if cut_position == 0 {
        // 使用种子随机决定切牌位置（避免切在最边缘）
        let range = len - 2; // 至少保留两边各一张
        if range == 0 {
            1
        } else {
            ((seed[0] as usize) % range) + 1
        }
    } else {
        (cut_position as usize).clamp(1, len - 1)
    };

    // 切牌: 使用 rotate_left 将前 pos 张牌移到末尾
    deck.rotate_left(pos);
}

/// 内部洗牌函数
///
/// 使用 Fisher-Yates 算法对牌组进行洗牌
fn shuffle_deck(deck: &mut [u8], seed: &[u8; 32]) {
    let len = deck.len();
    if len < 2 {
        return;
    }

    let mut current_seed = *seed;

    for i in 0..len - 1 {
        // 每16次迭代更新种子
        if i > 0 && i % 16 == 0 {
            current_seed = simple_hash(&current_seed);
        }

        let seed_index = i % 32;
        let random_byte = current_seed[seed_index];

        // 计算交换位置（从 i 到 len-1）
        let remaining = len - i;
        let j = i + ((random_byte as usize) % remaining);

        deck.swap(i, j);
    }
}

/// 带切牌的随机抽牌（完整版）
///
/// 模拟完整的塔罗牌洗牌-切牌-抽牌流程：
/// 1. 初始化牌组
/// 2. 第一次洗牌
/// 3. 执行切牌
/// 4. 第二次洗牌（可选，增加随机性）
/// 5. 抽取指定数量的牌
///
/// # 参数
/// - `random_seed`: 32字节随机种子
/// - `cut_position`: 切牌位置（1-77），0 表示随机切牌
/// - `count`: 需要抽取的牌数
///
/// # 返回
/// - 抽取的牌ID列表和对应的正逆位
pub fn draw_cards_with_cut(
    random_seed: &[u8; 32],
    cut_position: Option<u8>,
    count: u8,
) -> Vec<(u8, bool)> {
    let count = count.min(TOTAL_CARDS) as usize;
    let mut result = Vec::with_capacity(count);

    // 初始化牌组（0-77）
    let mut deck: Vec<u8> = (0..TOTAL_CARDS).collect();

    // 第一次洗牌
    shuffle_deck(&mut deck, random_seed);

    // 生成切牌用的种子
    let cut_seed = simple_hash(random_seed);

    // 执行切牌
    cut_deck(&mut deck, cut_position.unwrap_or(0), &cut_seed);

    // 第二次洗牌（切牌后的混合）
    let second_seed = simple_hash(&cut_seed);
    shuffle_deck(&mut deck, &second_seed);

    // 生成正逆位的种子
    let position_seed = simple_hash(&second_seed);

    // 抽取指定数量的牌并确定正逆位
    for i in 0..count {
        let position_byte = position_seed[(i + 16) % 32] ^ position_seed[(i + 23) % 32];
        let is_reversed = position_byte & 1 == 1;
        result.push((deck[i], is_reversed));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_cards_random() {
        let seed = [1u8; 32];
        let cards = draw_cards_random(&seed, 3);

        assert_eq!(cards.len(), 3);

        // 检查没有重复
        let ids: Vec<u8> = cards.iter().map(|(id, _)| *id).collect();
        assert!(validate_drawn_cards(&ids));
    }

    #[test]
    fn test_draw_cards_by_time() {
        let timestamp = 1700000000u64;
        let block_hash = [2u8; 32];
        let block_number = 12345u64;
        let cards = draw_cards_by_time(timestamp, &block_hash, block_number, 5);

        assert_eq!(cards.len(), 5);

        // 检查没有重复
        let ids: Vec<u8> = cards.iter().map(|(id, _)| *id).collect();
        assert!(validate_drawn_cards(&ids));
    }

    #[test]
    fn test_draw_cards_by_numbers() {
        let numbers = vec![7u16, 13, 42];
        let block_hash = [3u8; 32];
        let cards = draw_cards_by_numbers(&numbers, &block_hash, 3);

        assert_eq!(cards.len(), 3);
    }

    #[test]
    fn test_draw_cards_with_cut() {
        let seed = [4u8; 32];

        // 测试随机切牌
        let cards = draw_cards_with_cut(&seed, None, 5);
        assert_eq!(cards.len(), 5);
        let ids: Vec<u8> = cards.iter().map(|(id, _)| *id).collect();
        assert!(validate_drawn_cards(&ids));

        // 测试指定切牌位置
        let cards_cut = draw_cards_with_cut(&seed, Some(39), 5);
        assert_eq!(cards_cut.len(), 5);
        let ids_cut: Vec<u8> = cards_cut.iter().map(|(id, _)| *id).collect();
        assert!(validate_drawn_cards(&ids_cut));

        // 不同切牌位置应产生不同结果
        let cards_cut2 = draw_cards_with_cut(&seed, Some(10), 5);
        let ids_cut2: Vec<u8> = cards_cut2.iter().map(|(id, _)| *id).collect();
        assert_ne!(ids_cut, ids_cut2);
    }

    #[test]
    fn test_analyze_element_distribution() {
        // 测试牌：愚者(0), 权杖Ace(22), 圣杯2(37), 宝剑3(52)
        let cards = vec![0, 22, 37, 52];
        let (major, wands, cups, swords, pentacles) = analyze_element_distribution(&cards);

        assert_eq!(major, 1);
        assert_eq!(wands, 1);
        assert_eq!(cups, 1);
        assert_eq!(swords, 1);
        assert_eq!(pentacles, 0);
    }

    #[test]
    fn test_calculate_reversed_ratio() {
        let positions = vec![true, false, true, false, false];
        let ratio = calculate_reversed_ratio(&positions);
        assert_eq!(ratio, 40);
    }

    #[test]
    fn test_validate_drawn_cards() {
        // 有效
        assert!(validate_drawn_cards(&[0, 21, 45]));

        // 无效：超出范围
        assert!(!validate_drawn_cards(&[0, 78, 45]));

        // 无效：有重复
        assert!(!validate_drawn_cards(&[0, 21, 21]));
    }

    #[test]
    fn test_has_special_combination() {
        // 愚者 + 世界
        assert!(has_special_combination(&[0, 21, 45]));

        // 三张大阿卡纳
        assert!(has_special_combination(&[1, 5, 10]));

        // 普通组合
        assert!(!has_special_combination(&[22, 36, 50]));
    }
}

// ============================================================================
// 增强分析功能
// ============================================================================

use crate::constants::{
    get_upright_meaning, get_reversed_meaning, get_keywords,
    get_major_description, get_major_astrology, get_spread_position_info,
    SUIT_DESCRIPTIONS,
};

/// 占卜整体能量分析结果
#[derive(Clone, Debug, Default)]
pub struct ReadingEnergyAnalysis {
    /// 主导元素（火/水/风/土）
    pub dominant_element: Option<&'static str>,
    /// 主导元素数量
    pub dominant_element_count: u8,
    /// 大阿卡纳数量
    pub major_arcana_count: u8,
    /// 大阿卡纳比例（百分比）
    pub major_arcana_ratio: u8,
    /// 逆位数量
    pub reversed_count: u8,
    /// 逆位比例（百分比）
    pub reversed_ratio: u8,
    /// 宫廷牌数量
    pub court_cards_count: u8,
    /// 数字牌数量
    pub number_cards_count: u8,
    /// 是否有特殊组合
    pub has_special_combination: bool,
    /// 整体能量描述
    pub energy_description: &'static str,
    /// 整体建议
    pub advice: &'static str,
}

/// 单张牌的详细分析
#[derive(Clone, Debug)]
pub struct CardAnalysis {
    /// 牌ID
    pub card_id: u8,
    /// 牌名
    pub name: &'static str,
    /// 牌名（副名称，仅小阿卡纳）
    pub sub_name: Option<&'static str>,
    /// 是否逆位
    pub is_reversed: bool,
    /// 当前含义（根据正逆位）
    pub meaning: &'static str,
    /// 关键词
    pub keywords: &'static str,
    /// 元素（小阿卡纳）
    pub element: Option<&'static str>,
    /// 星座/行星对应（大阿卡纳）
    pub astrology: Option<(&'static str, &'static str)>,
    /// 牌面描述（大阿卡纳）
    pub description: Option<&'static str>,
    /// 在牌阵中的位置索引
    pub spread_position: u8,
    /// 位置含义
    pub position_name: &'static str,
    /// 位置描述
    pub position_description: &'static str,
    /// 位置解读指导
    pub position_guide: &'static str,
}

/// 完整占卜分析结果
#[derive(Clone, Debug)]
pub struct FullReadingAnalysis {
    /// 牌阵类型名称
    pub spread_name: &'static str,
    /// 牌阵描述
    pub spread_description: &'static str,
    /// 每张牌的详细分析
    pub cards: Vec<CardAnalysis>,
    /// 整体能量分析
    pub energy: ReadingEnergyAnalysis,
    /// AI 解读提示（用于生成 AI 解读的上下文）
    pub ai_prompt_context: Vec<u8>,
}

/// 分析抽取的牌
///
/// 提供单张牌的详细分析信息
///
/// # 参数
/// - `card_id`: 牌ID
/// - `is_reversed`: 是否逆位
/// - `spread_type`: 牌阵类型
/// - `spread_position`: 在牌阵中的位置
pub fn analyze_card(
    card_id: u8,
    is_reversed: bool,
    spread_type: u8,
    spread_position: u8,
) -> CardAnalysis {
    let card = TarotCard::from_id(card_id);
    let (name, sub_name) = card.display_name();

    // 获取位置信息
    let position_info = get_spread_position_info(spread_type, spread_position as usize);
    let (position_name, position_description, position_guide) = match position_info {
        Some(info) => (info.name, info.description, info.interpretation_guide),
        None => ("未知位置", "未知位置描述", "请根据牌意进行解读"),
    };

    CardAnalysis {
        card_id,
        name,
        sub_name,
        is_reversed,
        meaning: if is_reversed {
            get_reversed_meaning(card_id)
        } else {
            get_upright_meaning(card_id)
        },
        keywords: get_keywords(card_id),
        element: if card.is_major() {
            None
        } else {
            Some(card.element())
        },
        astrology: get_major_astrology(card_id),
        description: get_major_description(card_id),
        spread_position,
        position_name,
        position_description,
        position_guide,
    }
}

/// 分析整体能量
///
/// 分析一次占卜的整体能量分布和趋势
///
/// # 参数
/// - `cards`: 抽取的牌列表 (card_id, is_reversed)
pub fn analyze_reading_energy(cards: &[(u8, bool)]) -> ReadingEnergyAnalysis {
    if cards.is_empty() {
        return ReadingEnergyAnalysis::default();
    }

    let total = cards.len() as u8;
    let card_ids: Vec<u8> = cards.iter().map(|(id, _)| *id).collect();

    // 元素分布
    let (major, wands, cups, swords, pentacles) = analyze_element_distribution(&card_ids);

    // 找出主导元素
    let elements = [
        ("火", wands),
        ("水", cups),
        ("风", swords),
        ("土", pentacles),
    ];
    let dominant = elements.iter().max_by_key(|(_, count)| *count);
    let (dominant_element, dominant_count) = match dominant {
        Some((elem, count)) if *count > 0 => (Some(*elem), *count),
        _ => (None, 0),
    };

    // 逆位统计
    let reversed_count = cards.iter().filter(|(_, rev)| *rev).count() as u8;
    let reversed_ratio = if total > 0 {
        (reversed_count as u16 * 100 / total as u16) as u8
    } else {
        0
    };

    // 大阿卡纳比例
    let major_ratio = if total > 0 {
        (major as u16 * 100 / total as u16) as u8
    } else {
        0
    };

    // 宫廷牌和数字牌统计
    let mut court_count = 0u8;
    let mut number_count = 0u8;
    for &card_id in &card_ids {
        let card = TarotCard::from_id(card_id);
        if card.is_court_card() {
            court_count += 1;
        } else if !card.is_major() {
            number_count += 1;
        }
    }

    // 特殊组合检测
    let has_special = has_special_combination(&card_ids);

    // 能量描述和建议
    let (energy_desc, advice) = determine_energy_description(
        major,
        reversed_ratio,
        dominant_element,
        has_special,
        total,
    );

    ReadingEnergyAnalysis {
        dominant_element,
        dominant_element_count: dominant_count,
        major_arcana_count: major,
        major_arcana_ratio: major_ratio,
        reversed_count,
        reversed_ratio,
        court_cards_count: court_count,
        number_cards_count: number_count,
        has_special_combination: has_special,
        energy_description: energy_desc,
        advice,
    }
}

/// 确定能量描述和建议
fn determine_energy_description(
    major_count: u8,
    reversed_ratio: u8,
    dominant_element: Option<&'static str>,
    has_special: bool,
    total_cards: u8,
) -> (&'static str, &'static str) {
    // 特殊组合优先
    if has_special {
        return (
            "牌阵呈现强烈的命运指引能量，出现了具有特殊意义的牌组合",
            "这是一个重要的转折点，请认真对待牌面的指引，可能会有重大变化或机遇",
        );
    }

    // 大阿卡纳为主
    if total_cards > 0 && major_count as u16 * 2 >= total_cards as u16 {
        return (
            "大阿卡纳牌占主导，表明这次占卜涉及人生的重大主题和灵性成长",
            "关注牌面所揭示的人生课题，这不是日常琐事，而是需要深思的重要议题",
        );
    }

    // 逆位较多
    if reversed_ratio >= 60 {
        return (
            "逆位牌较多，表明当前存在阻碍或需要内省的能量",
            "现在可能不是行动的最佳时机，建议先理清内心的障碍和担忧",
        );
    }

    // 按主导元素给出描述
    match dominant_element {
        Some("火") => (
            "火元素主导，能量充满激情、创造力和行动力",
            "适合积极行动，追求目标，但注意不要冲动行事",
        ),
        Some("水") => (
            "水元素主导，能量偏向情感、直觉和人际关系",
            "倾听内心的声音，关注人际关系和情感需求",
        ),
        Some("风") => (
            "风元素主导，能量偏向思维、沟通和智力活动",
            "运用理性思考，注意沟通方式，可能需要做出重要决定",
        ),
        Some("土") => (
            "土元素主导，能量偏向物质、工作和实际事务",
            "脚踏实地，关注财务和健康，稳步推进计划",
        ),
        _ => (
            "能量分布较为均衡，各方面都需要关注",
            "保持平衡的心态，综合考虑各方面因素",
        ),
    }
}

/// 生成完整占卜分析
///
/// # 参数
/// - `cards`: 抽取的牌列表 (card_id, is_reversed)
/// - `spread_type`: 牌阵类型
pub fn full_reading_analysis(
    cards: &[(u8, bool)],
    spread_type: SpreadType,
) -> FullReadingAnalysis {
    let type_id = spread_type.type_id();

    // 分析每张牌
    let card_analyses: Vec<CardAnalysis> = cards
        .iter()
        .enumerate()
        .map(|(i, (card_id, is_reversed))| {
            analyze_card(*card_id, *is_reversed, type_id, i as u8)
        })
        .collect();

    // 分析整体能量
    let energy = analyze_reading_energy(cards);

    // 生成 AI 解读上下文
    let ai_context = generate_ai_context(&card_analyses, &energy, &spread_type);

    FullReadingAnalysis {
        spread_name: spread_type.name(),
        spread_description: spread_type.description(),
        cards: card_analyses,
        energy,
        ai_prompt_context: ai_context,
    }
}

/// 生成 AI 解读上下文
///
/// 创建用于 AI 服务的结构化上下文信息
fn generate_ai_context(
    cards: &[CardAnalysis],
    energy: &ReadingEnergyAnalysis,
    spread_type: &SpreadType,
) -> Vec<u8> {
    use sp_std::vec;

    let mut context = vec![];

    // 添加牌阵信息
    context.extend_from_slice(b"spread:");
    context.extend_from_slice(spread_type.name().as_bytes());
    context.push(b';');

    // 添加能量信息
    if let Some(elem) = energy.dominant_element {
        context.extend_from_slice(b"dominant:");
        context.extend_from_slice(elem.as_bytes());
        context.push(b';');
    }

    // 添加牌信息
    for card in cards {
        context.extend_from_slice(b"card:");
        context.extend_from_slice(card.name.as_bytes());
        if let Some(sub) = card.sub_name {
            context.push(b'-');
            context.extend_from_slice(sub.as_bytes());
        }
        context.push(if card.is_reversed { b'R' } else { b'U' });
        context.push(b'@');
        context.extend_from_slice(card.position_name.as_bytes());
        context.push(b';');
    }

    context
}

/// 获取元素描述
///
/// # 参数
/// - `element`: 元素名称（火/水/风/土）
pub fn get_element_description(element: &str) -> &'static str {
    match element {
        "火" => SUIT_DESCRIPTIONS[0],
        "水" => SUIT_DESCRIPTIONS[1],
        "风" => SUIT_DESCRIPTIONS[2],
        "土" => SUIT_DESCRIPTIONS[3],
        _ => "",
    }
}

/// 判断两张牌之间的关系
///
/// 分析两张牌之间的能量互动
///
/// # 参数
/// - `card1`: 第一张牌
/// - `card2`: 第二张牌
///
/// # 返回
/// - 关系描述
pub fn analyze_card_relationship(card1: &TarotCard, card2: &TarotCard) -> &'static str {
    // 同为大阿卡纳
    if card1.is_major() && card2.is_major() {
        return "两张大阿卡纳相遇，强调命运的重要转折和深层的灵性课题";
    }

    // 同花色
    if card1.suit == card2.suit && card1.suit != Suit::None {
        match card1.suit {
            Suit::Wands => return "两张权杖牌相遇，强化了行动力和创造力的主题",
            Suit::Cups => return "两张圣杯牌相遇，情感和关系是核心议题",
            Suit::Swords => return "两张宝剑牌相遇，思维和沟通需要特别关注",
            Suit::Pentacles => return "两张星币牌相遇，物质和实际事务是重点",
            _ => {}
        }
    }

    // 大阿卡纳 + 小阿卡纳
    if card1.is_major() != card2.is_major() {
        return "大阿卡纳与小阿卡纳相遇，宏观命运与具体事务相互影响";
    }

    // 元素关系
    let elem1 = card1.suit.element();
    let elem2 = card2.suit.element();

    match (elem1, elem2) {
        ("火", "风") | ("风", "火") => "火与风相助，思想激发行动",
        ("水", "土") | ("土", "水") => "水与土相合，情感与现实结合",
        ("火", "水") | ("水", "火") => "火与水相冲，激情与情感需要平衡",
        ("风", "土") | ("土", "风") => "风与土相异，理想与现实需要协调",
        _ => "两张牌互相呼应，共同描绘完整的画面",
    }
}

// ============================================================================
// 解卦算法
// ============================================================================

use crate::interpretation::*;

/// 生成核心解卦数据
///
/// 基于抽取的牌生成核心解卦指标
///
/// # 参数
/// - `cards`: 抽取的牌列表 (card_id, is_reversed)
/// - `spread_type`: 牌阵类型
/// - `block_number`: 当前区块号
///
/// # 返回
/// - 核心解卦数据
pub fn generate_core_interpretation(
    cards: &[(u8, bool)],
    spread_type: SpreadType,
    block_number: u32,
) -> TarotCoreInterpretation {
    if cards.is_empty() {
        return TarotCoreInterpretation::default();
    }

    let total_cards = cards.len() as u8;
    let card_ids: Vec<u8> = cards.iter().map(|(id, _)| *id).collect();

    // 统计基础数据
    let (major, wands, cups, swords, pentacles) = analyze_element_distribution(&card_ids);
    let reversed_count = cards.iter().filter(|(_, rev)| *rev).count() as u8;
    let reversed_ratio = (reversed_count as u16 * 100 / total_cards as u16) as u8;

    // 统计牌类型
    let mut court_count = 0u8;
    let mut number_count = 0u8;
    for &card_id in &card_ids {
        let card = TarotCard::from_id(card_id);
        if card.is_court_card() {
            court_count += 1;
        } else if !card.is_major() {
            number_count += 1;
        }
    }

    // 计算主导元素
    let dominant_element = determine_dominant_element(major, wands, cups, swords, pentacles, total_cards);

    // 计算元素分布位图
    let element_bitmap = encode_element_bitmap(wands, cups, swords, pentacles);

    // 检测特殊组合
    let special_combination = detect_special_combinations(&card_ids, reversed_count, total_cards);

    // 计算能量指数
    let upright_count = total_cards - reversed_count;
    let upright_ratio = (upright_count as u16 * 100 / total_cards as u16) as u8;

    let action_index = calculate_action_index(wands, upright_ratio);
    let emotion_index = calculate_emotion_index(cups);
    let intellect_index = calculate_intellect_index(swords);
    let material_index = calculate_material_index(pentacles);
    let spiritual_index = calculate_spiritual_index(major, total_cards);
    let stability_index = calculate_stability_index(upright_ratio, number_count, total_cards);
    let change_index = calculate_change_index(reversed_ratio, court_count, total_cards);

    // 计算总体能量
    let overall_energy = calculate_overall_energy(upright_count, major, total_cards);

    // 判断吉凶倾向
    let fortune_tendency = determine_fortune_tendency(
        major,
        reversed_ratio,
        dominant_element,
        special_combination,
        total_cards,
    );

    // 计算综合评分
    let overall_score = calculate_overall_score(
        &fortune_tendency,
        overall_energy,
        stability_index,
        spiritual_index,
    );

    // 确定关键牌（通常是第一张或中心牌）
    let key_card_index = determine_key_card_index(cards.len(), &spread_type);
    let (key_card_id, key_card_reversed) = cards[key_card_index];

    TarotCoreInterpretation {
        overall_energy,
        dominant_element,
        fortune_tendency,
        reversed_ratio,
        major_arcana_count: major,
        court_cards_count: court_count,
        number_cards_count: number_count,
        element_bitmap,
        special_combination,
        key_card_id,
        key_card_reversed: if key_card_reversed { 1 } else { 0 },
        spread_type: spread_type.type_id(),
        action_index,
        emotion_index,
        intellect_index,
        material_index,
        spiritual_index,
        stability_index,
        change_index,
        overall_score,
        block_number,
        algorithm_version: 1,
        confidence: calculate_confidence(total_cards, major),
        reserved: [0; 4],
    }
}

/// 确定主导元素
fn determine_dominant_element(
    major: u8,
    wands: u8,
    cups: u8,
    swords: u8,
    pentacles: u8,
    total: u8,
) -> DominantElement {
    // 大阿卡纳占比超过50%，灵性主导
    if major as u16 * 2 >= total as u16 {
        return DominantElement::Spirit;
    }

    // 找出最多的元素
    let max_element = wands.max(cups).max(swords).max(pentacles);

    // 如果最多的元素数量不足2张，认为无主导
    if max_element < 2 {
        return DominantElement::None;
    }

    // 返回主导元素
    if wands == max_element {
        DominantElement::Fire
    } else if cups == max_element {
        DominantElement::Water
    } else if swords == max_element {
        DominantElement::Air
    } else if pentacles == max_element {
        DominantElement::Earth
    } else {
        DominantElement::None
    }
}

/// 编码元素分布位图
fn encode_element_bitmap(wands: u8, cups: u8, swords: u8, pentacles: u8) -> u8 {
    let fire = wands.min(3);
    let water = cups.min(3);
    let air = swords.min(3);
    let earth = pentacles.min(3);

    fire | (water << 2) | (air << 4) | (earth << 6)
}

/// 检测特殊组合
fn detect_special_combinations(cards: &[u8], reversed_count: u8, total: u8) -> u8 {
    let mut flags = 0u8;

    // bit 0: 愚者+世界组合
    if cards.contains(&0) && cards.contains(&21) {
        flags |= 0b00000001;
    }

    // bit 1: 三张以上大阿卡纳
    let major_count = cards.iter().filter(|&&c| c < 22).count();
    if major_count >= 3 {
        flags |= 0b00000010;
    }

    // bit 2: 同花色三连号
    if has_special_combination(cards) {
        flags |= 0b00000100;
    }

    // bit 3: 全逆位
    if reversed_count == total {
        flags |= 0b00001000;
    }

    // bit 4: 全正位
    if reversed_count == 0 {
        flags |= 0b00010000;
    }

    flags
}

/// 计算行动力指数
fn calculate_action_index(wands: u8, upright_ratio: u8) -> u8 {
    let wands_score = (wands as u16 * 25).min(100);
    let upright_score = (upright_ratio as u16 / 2).min(50);
    ((wands_score + upright_score).min(100)) as u8
}

/// 计算情感指数
fn calculate_emotion_index(cups: u8) -> u8 {
    ((cups as u16 * 25).min(100)) as u8
}

/// 计算思维指数
fn calculate_intellect_index(swords: u8) -> u8 {
    ((swords as u16 * 25).min(100)) as u8
}

/// 计算物质指数
fn calculate_material_index(pentacles: u8) -> u8 {
    ((pentacles as u16 * 25).min(100)) as u8
}

/// 计算灵性指数
fn calculate_spiritual_index(major: u8, total: u8) -> u8 {
    ((major as u16 * 100 / total as u16).min(100)) as u8
}

/// 计算稳定性指数
fn calculate_stability_index(upright_ratio: u8, number_count: u8, total: u8) -> u8 {
    let upright_score = (upright_ratio as u16 * 60 / 100).min(60);
    let number_score = (number_count as u16 * 40 / total as u16).min(40);
    ((upright_score + number_score).min(100)) as u8
}

/// 计算变化性指数
fn calculate_change_index(reversed_ratio: u8, court_count: u8, total: u8) -> u8 {
    let reversed_score = (reversed_ratio as u16 * 60 / 100).min(60);
    let court_score = (court_count as u16 * 40 / total as u16).min(40);
    ((reversed_score + court_score).min(100)) as u8
}

/// 计算总体能量
fn calculate_overall_energy(upright_count: u8, major: u8, total: u8) -> u8 {
    let upright_score = (upright_count as u16 * 10).min(100);
    let major_score = (major as u16 * 15).min(100);
    ((upright_score + major_score) / total as u16).min(100) as u8
}

/// 判断吉凶倾向
fn determine_fortune_tendency(
    major: u8,
    reversed_ratio: u8,
    dominant_element: DominantElement,
    special_combination: u8,
    total: u8,
) -> FortuneTendency {
    // 特殊组合优先判断
    if special_combination & 0b00000001 != 0 {
        // 愚者+世界：完整旅程
        return FortuneTendency::Good;
    }

    // 大阿卡纳为主
    if major as u16 * 2 >= total as u16 {
        if reversed_ratio < 40 {
            return FortuneTendency::Good;
        } else {
            return FortuneTendency::Neutral;
        }
    }

    // 逆位较多
    if reversed_ratio >= 70 {
        return FortuneTendency::Bad;
    } else if reversed_ratio >= 50 {
        return FortuneTendency::MinorBad;
    }

    // 按主导元素判断
    match dominant_element {
        DominantElement::Fire => {
            if reversed_ratio < 30 {
                FortuneTendency::Excellent
            } else {
                FortuneTendency::Good
            }
        }
        DominantElement::Water => {
            if reversed_ratio < 40 {
                FortuneTendency::Good
            } else {
                FortuneTendency::Neutral
            }
        }
        DominantElement::Air => FortuneTendency::Neutral,
        DominantElement::Earth => {
            if reversed_ratio < 30 {
                FortuneTendency::Good
            } else {
                FortuneTendency::Neutral
            }
        }
        DominantElement::Spirit => FortuneTendency::Good,
        DominantElement::None => FortuneTendency::Neutral,
    }
}

/// 计算综合评分
fn calculate_overall_score(
    fortune: &FortuneTendency,
    energy: u8,
    stability: u8,
    spiritual: u8,
) -> u8 {
    let fortune_score = match fortune {
        FortuneTendency::Excellent => 90,
        FortuneTendency::Good => 75,
        FortuneTendency::Neutral => 50,
        FortuneTendency::MinorBad => 35,
        FortuneTendency::Bad => 20,
    };

    let weighted_score = (fortune_score as u16 * 40
        + energy as u16 * 30
        + stability as u16 * 20
        + spiritual as u16 * 10)
        / 100;

    weighted_score.min(100) as u8
}

/// 计算可信度
fn calculate_confidence(total: u8, major: u8) -> u8 {
    // 牌数越多，可信度越高
    let count_score = (total as u16 * 10).min(100);

    // 大阿卡纳比例适中时可信度更高
    let major_ratio = (major as u16 * 100 / total as u16) as u8;
    let major_score = if major_ratio >= 20 && major_ratio <= 60 {
        100
    } else if major_ratio < 20 {
        50 + major_ratio * 2
    } else {
        160 - major_ratio
    };

    ((count_score + major_score as u16) / 2).min(100) as u8
}

/// 确定关键牌索引
fn determine_key_card_index(_card_count: usize, spread_type: &SpreadType) -> usize {
    match spread_type {
        SpreadType::SingleCard => 0,
        SpreadType::ThreeCardTime | SpreadType::ThreeCardSituation => 1, // 中间牌
        SpreadType::CelticCross => 0, // 第一张（当前状况）
        _ => 0, // 默认第一张
    }
}

/// 生成牌阵能量分析
///
/// # 参数
/// - `cards`: 抽取的牌列表 (card_id, is_reversed)
///
/// # 返回
/// - 牌阵能量分析
pub fn generate_spread_energy_analysis(cards: &[(u8, bool)]) -> SpreadEnergyAnalysis {
    if cards.is_empty() {
        return SpreadEnergyAnalysis::default();
    }

    let total = cards.len();
    let third = total / 3;

    // 计算过去、现在、未来能量
    let past_energy = calculate_section_energy(&cards[0..third.max(1)]);
    let present_energy = if total >= 3 {
        calculate_section_energy(&cards[third..third * 2])
    } else {
        calculate_section_energy(cards)
    };
    let future_energy = if total >= 3 {
        calculate_section_energy(&cards[third * 2..])
    } else {
        calculate_section_energy(cards)
    };

    // 计算内在/外在能量
    let (inner_energy, outer_energy) = calculate_inner_outer_energy(cards);

    // 判断能量流动方向
    let energy_flow = determine_energy_flow(past_energy, present_energy, future_energy);

    // 计算能量平衡度
    let energy_balance = calculate_energy_balance(cards);

    SpreadEnergyAnalysis {
        past_energy,
        present_energy,
        future_energy,
        inner_energy,
        outer_energy,
        energy_flow,
        energy_balance,
    }
}

/// 计算某段牌的能量
fn calculate_section_energy(cards: &[(u8, bool)]) -> u8 {
    if cards.is_empty() {
        return 50;
    }

    let mut energy = 0u16;
    for (card_id, is_reversed) in cards {
        let card = TarotCard::from_id(*card_id);
        let base_energy = if card.is_major() { 80 } else { 60 };
        let position_modifier = if *is_reversed { -20 } else { 20 };
        energy += (base_energy + position_modifier) as u16;
    }

    ((energy / cards.len() as u16).min(100)) as u8
}

/// 计算内在/外在能量
fn calculate_inner_outer_energy(cards: &[(u8, bool)]) -> (u8, u8) {
    let mut inner = 0u16;
    let mut outer = 0u16;
    let mut inner_count = 0u16;
    let mut outer_count = 0u16;

    for (card_id, is_reversed) in cards {
        let card = TarotCard::from_id(*card_id);
        let base_energy = if card.is_major() { 70 } else { 50 };

        if *is_reversed {
            inner += (base_energy + 20) as u16;
            inner_count += 1;
        } else {
            outer += (base_energy + 20) as u16;
            outer_count += 1;
        }
    }

    let inner_avg = if inner_count > 0 {
        (inner / inner_count).min(100) as u8
    } else {
        50
    };

    let outer_avg = if outer_count > 0 {
        (outer / outer_count).min(100) as u8
    } else {
        50
    };

    (inner_avg, outer_avg)
}

/// 判断能量流动方向
fn determine_energy_flow(past: u8, present: u8, future: u8) -> EnergyFlow {
    let past_to_present = present as i16 - past as i16;
    let present_to_future = future as i16 - present as i16;

    if past_to_present > 10 && present_to_future > 10 {
        EnergyFlow::Rising
    } else if past_to_present < -10 && present_to_future < -10 {
        EnergyFlow::Declining
    } else if past_to_present.abs() < 10 && present_to_future.abs() < 10 {
        EnergyFlow::Stable
    } else {
        EnergyFlow::Volatile
    }
}

/// 计算能量平衡度
fn calculate_energy_balance(cards: &[(u8, bool)]) -> u8 {
    let (_major, wands, cups, swords, pentacles) = analyze_element_distribution(
        &cards.iter().map(|(id, _)| *id).collect::<Vec<u8>>(),
    );

    // 使用整数计算元素分布的方差（避免浮点数）
    let elements = [wands as u32, cups as u32, swords as u32, pentacles as u32];
    let sum: u32 = elements.iter().sum();
    let avg_4x = sum; // 乘以4来避免除法

    // 计算方差的4倍（避免浮点数）
    // variance * 4 = sum((e * 4 - avg_4x)^2) / 4
    let variance_16x: u32 = elements
        .iter()
        .map(|&e| {
            let diff = (e * 4).saturating_sub(avg_4x) as i32;
            let diff_abs = diff.unsigned_abs();
            diff_abs * diff_abs
        })
        .sum();

    // 方差（乘以16）与标准差阈值的关系：
    // std_dev < 0.5 => variance < 0.25 => variance_16x < 4
    // std_dev < 1.0 => variance < 1.0 => variance_16x < 16
    // std_dev < 1.5 => variance < 2.25 => variance_16x < 36
    // std_dev < 2.0 => variance < 4.0 => variance_16x < 64
    let balance = if variance_16x < 4 {
        100
    } else if variance_16x < 16 {
        80
    } else if variance_16x < 36 {
        60
    } else if variance_16x < 64 {
        40
    } else {
        20
    };

    balance
}

#[cfg(test)]
mod interpretation_tests {
    use super::*;

    #[test]
    fn test_generate_core_interpretation() {
        // 测试三张牌：愚者(正位)、魔术师(正位)、世界(逆位)
        let cards = vec![(0, false), (1, false), (21, true)];
        let core = generate_core_interpretation(&cards, SpreadType::ThreeCardTime, 1000);

        assert_eq!(core.major_arcana_count, 3);
        assert_eq!(core.reversed_ratio, 33); // 1/3 ≈ 33%
        assert!(core.has_fool_world_combo());
        assert!(core.has_many_major_arcana());
        assert_eq!(core.dominant_element, DominantElement::Spirit);
    }

    #[test]
    fn test_element_distribution() {
        // 测试权杖牌：权杖Ace(22), 权杖2(23), 权杖3(24)
        let cards = vec![(22, false), (23, false), (24, false)];
        let core = generate_core_interpretation(&cards, SpreadType::ThreeCardTime, 1000);

        assert_eq!(core.dominant_element, DominantElement::Fire);
        assert_eq!(core.fire_count(), 3);
        assert_eq!(core.action_index, 100); // 3张权杖 + 全正位
    }

    #[test]
    fn test_spread_energy_analysis() {
        let cards = vec![(0, false), (1, false), (21, false)];
        let energy = generate_spread_energy_analysis(&cards);

        assert!(energy.past_energy > 0);
        assert!(energy.present_energy > 0);
        assert!(energy.future_energy > 0);
        assert!(energy.outer_energy > energy.inner_energy); // 全正位，外在能量更强
    }
}

// ============================================================================
// OCW-TEE 集成辅助函数
// ============================================================================

/// 获取牌阵所需的牌数量
pub fn get_spread_card_count(spread_type: u8) -> u8 {
    match spread_type {
        0 => 1,  // 单张
        1 => 3,  // 三牌阵
        2 => 5,  // 五牌阵
        3 => 10, // 凯尔特十字
        4 => 7,  // 七星阵
        _ => 3,  // 默认三牌
    }
}

/// 从数字生成牌列表
pub fn generate_cards_from_number(num: u32, count: u8) -> Vec<u8> {
    let mut cards = Vec::with_capacity(count as usize);
    let mut used = [false; 78];
    
    for i in 0..count {
        let seed = num.wrapping_add(i as u32 * 17);
        let mut card_id = (seed % 78) as u8;
        
        // 避免重复
        while used[card_id as usize] {
            card_id = (card_id + 1) % 78;
        }
        used[card_id as usize] = true;
        cards.push(card_id);
    }
    
    cards
}

/// 从数字生成正逆位列表
pub fn generate_reversed_from_number(num: u32, count: u8) -> Vec<bool> {
    (0..count)
        .map(|i| ((num >> (i % 32)) & 1) == 1)
        .collect()
}
