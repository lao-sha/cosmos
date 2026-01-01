//! # 塔罗牌解卦 Runtime API
//!
//! 本模块定义了塔罗牌解卦系统的 Runtime API，供前端通过 RPC 免费调用。
//!
//! ## 功能说明
//!
//! - `get_core_interpretation`: 获取核心解卦结果（能量、元素、吉凶、评分等）
//! - `get_full_interpretation`: 获取完整解卦结果（含能量分析、牌间关系等）
//! - `get_interpretation_texts`: 获取解读文本索引列表
//! - `generate_ai_prompt_context`: 生成AI解读提示词上下文
//! - `reading_exists`: 检查占卜记录是否存在
//! - `get_reading_owner`: 获取占卜记录创建者
//! - `batch_get_core_interpretations`: 批量获取核心解卦
//!
//! ## 使用方式
//!
//! 前端通过 polkadot.js API 调用：
//! ```javascript
//! // 获取核心解卦
//! const core = await api.call.tarotApi.getCoreInterpretation(readingId);
//!
//! // 访问核心数据
//! const { fortuneTendency, dominantElement, overallScore, confidence } = core;
//!
//! // 获取完整解卦
//! const full = await api.call.tarotApi.getFullInterpretation(readingId);
//!
//! // 生成AI解读上下文
//! const aiContext = await api.call.tarotApi.generateAiPromptContext(readingId);
//! ```
//!
//! ## 数据结构
//!
//! ### 核心解卦 (TarotCoreInterpretation, ~30 bytes)
//! - overall_energy: 总体能量等级 (0-100)
//! - dominant_element: 主导元素（火/水/风/土/灵性）
//! - fortune_tendency: 吉凶倾向（大吉/吉/中平/小凶/凶）
//! - action_index...spiritual_index: 六大能量指数
//! - overall_score: 综合评分 (0-100)
//! - confidence: 可信度 (0-100)
//!
//! ### 完整解卦 (TarotFullInterpretation, ~175 bytes)
//! - core: 核心解卦
//! - spread_energy: 牌阵能量分析
//! - card_analyses: 各牌分析（可选）
//! - card_relationships: 牌间关系（可选）
//! - timeline_analysis: 时间线分析（可选）

use crate::interpretation::{
    CardInterpretation, CardRelationship, InterpretationTextType, SpreadEnergyAnalysis,
    TarotCoreInterpretation, TarotFullInterpretation, TimelineAnalysis,
};
use codec::Codec;
use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// 塔罗牌解卦 Runtime API
    ///
    /// 提供塔罗牌占卜的免费查询接口，无需支付 Gas 费用。
    ///
    /// ## 设计原则
    ///
    /// - 分层接口：提供核心和完整两种接口，前端按需选择
    /// - 实时计算：每次调用实时计算，无需链上存储解卦结果
    /// - AI友好：提供结构化的AI解读上下文生成
    /// - 批量支持：支持批量查询以提高效率
    pub trait TarotApi<AccountId>
    where
        AccountId: Codec,
    {
        /// 获取核心解卦结果
        ///
        /// 返回最关键的解卦指标，约 30 bytes。
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(TarotCoreInterpretation)`: 核心解卦结果
        /// - `None`: 占卜记录不存在
        ///
        /// # 返回数据结构
        /// ```text
        /// TarotCoreInterpretation (~30 bytes)
        /// ├── overall_energy      总体能量 (0-100)
        /// ├── dominant_element    主导元素（无/火/水/风/土/灵性）
        /// ├── fortune_tendency    吉凶倾向（大吉/吉/中平/小凶/凶）
        /// ├── reversed_ratio      逆位比例 (0-100)
        /// ├── major_arcana_count  大阿卡纳数量
        /// ├── court_cards_count   宫廷牌数量
        /// ├── number_cards_count  数字牌数量
        /// ├── element_bitmap      元素分布位图
        /// ├── special_combination 特殊组合标志位图
        /// ├── key_card_id         关键牌ID
        /// ├── action_index        行动力指数 (0-100)
        /// ├── emotion_index       情感指数 (0-100)
        /// ├── intellect_index     思维指数 (0-100)
        /// ├── material_index      物质指数 (0-100)
        /// ├── spiritual_index     灵性指数 (0-100)
        /// ├── stability_index     稳定性指数 (0-100)
        /// ├── change_index        变化性指数 (0-100)
        /// ├── overall_score       综合评分 (0-100)
        /// └── confidence          可信度 (0-100)
        /// ```
        ///
        /// # 示例
        /// ```javascript
        /// const core = await api.call.tarotApi.getCoreInterpretation(readingId);
        /// console.log('吉凶倾向:', core.fortuneTendency);  // 0=大吉, 4=凶
        /// console.log('主导元素:', core.dominantElement);  // 1=火, 2=水, 3=风, 4=土, 5=灵性
        /// console.log('综合评分:', core.overallScore);
        /// console.log('灵性指数:', core.spiritualIndex);
        /// ```
        fn get_core_interpretation(
            reading_id: u64,
        ) -> Option<TarotCoreInterpretation>;

        /// 获取完整解卦结果
        ///
        /// 返回包含所有分析的完整解卦，约 175 bytes。
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(TarotFullInterpretation)`: 完整解卦结果
        /// - `None`: 占卜记录不存在
        ///
        /// # 返回数据结构
        /// ```text
        /// TarotFullInterpretation (~175 bytes)
        /// ├── core: TarotCoreInterpretation       核心指标 (~30 bytes)
        /// ├── spread_energy: SpreadEnergyAnalysis 牌阵能量分析 (~8 bytes)
        /// │   ├── past_energy     过去能量 (0-100)
        /// │   ├── present_energy  现在能量 (0-100)
        /// │   ├── future_energy   未来能量 (0-100)
        /// │   ├── inner_energy    内在能量 (0-100)
        /// │   ├── outer_energy    外在能量 (0-100)
        /// │   ├── energy_flow     能量流动方向
        /// │   └── energy_balance  能量平衡度 (0-100)
        /// ├── card_analyses: Vec<CardInterpretation>  各牌分析（可选）
        /// │   └── [每张牌] card_id, is_reversed, energy_strength, relations
        /// ├── card_relationships: Vec<CardRelationship>  牌间关系（可选）
        /// │   └── [每对牌] card1_index, card2_index, relationship_type, strength
        /// └── timeline_analysis: TimelineAnalysis     时间线分析（可选）
        ///     ├── past_trend      过去趋势
        ///     ├── present_state   现在状态
        ///     ├── future_trend    未来趋势
        ///     └── turning_point   转折点位置
        /// ```
        ///
        /// # 示例
        /// ```javascript
        /// const full = await api.call.tarotApi.getFullInterpretation(readingId);
        /// // 核心数据
        /// console.log('吉凶:', full.core.fortuneTendency);
        /// // 能量分析
        /// console.log('过去能量:', full.spreadEnergy.pastEnergy);
        /// console.log('未来能量:', full.spreadEnergy.futureEnergy);
        /// console.log('能量流动:', full.spreadEnergy.energyFlow);
        /// // 各牌分析
        /// full.cardAnalyses?.forEach((card, i) => {
        ///     console.log(`牌${i}: ID=${card.cardId}, 能量=${card.energyStrength}`);
        /// });
        /// ```
        fn get_full_interpretation(
            reading_id: u64,
        ) -> Option<TarotFullInterpretation<ConstU32<12>>>;

        /// 获取解读文本索引列表
        ///
        /// 返回适用于当前占卜的解读文本类型列表，前端根据索引显示对应文本。
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(Vec<InterpretationTextType>)`: 解读文本索引列表（最多20个）
        /// - `None`: 占卜记录不存在
        ///
        /// # 文本类型索引
        /// - 0-9: 总体能量描述（高/中/低/波动）
        /// - 10-19: 元素主导描述（火/水/风/土/灵性/平衡）
        /// - 20-29: 吉凶判断（大吉/吉/中平/小凶/凶）
        /// - 30-39: 特殊组合（愚者+世界、多大阿卡纳等）
        /// - 40-59: 行动建议（积极行动/谨慎观察/内省调整等）
        /// - 60-69: 时间线描述（过去/现在/未来趋势）
        ///
        /// # 示例
        /// ```javascript
        /// const texts = await api.call.tarotApi.getInterpretationTexts(readingId);
        /// texts.forEach(textType => {
        ///     console.log(getTextByType(textType));
        /// });
        /// ```
        fn get_interpretation_texts(
            reading_id: u64,
        ) -> Option<Vec<InterpretationTextType>>;

        /// 生成AI解读提示词上下文
        ///
        /// 生成结构化的AI解读上下文，用于调用外部AI服务生成深度解读。
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(Vec<u8>)`: AI提示词上下文（UTF-8编码，最多2048字节）
        /// - `None`: 占卜记录不存在
        ///
        /// # 上下文格式
        /// ```text
        /// spread:牌阵名称;
        /// dominant:主导元素;
        /// energy:整体能量描述;
        /// fortune:吉凶倾向;
        /// card:牌名-位置[U/R]@位置名称;
        /// card:牌名-位置[U/R]@位置名称;
        /// ...
        /// special:特殊组合描述;
        /// advice:建议方向;
        /// ```
        ///
        /// # 示例
        /// ```javascript
        /// const context = await api.call.tarotApi.generateAiPromptContext(readingId);
        /// const contextStr = new TextDecoder().decode(context);
        ///
        /// // 构建完整的AI提示词
        /// const prompt = `你是一位专业的塔罗牌解读师，请根据以下牌阵信息进行深度解读：
        /// ${contextStr}
        /// 请从以下几个方面进行解读：
        /// 1. 整体能量概览
        /// 2. 逐张牌面解析
        /// 3. 牌面间的互动关系
        /// 4. 核心洞察与行动建议
        /// 5. 总结与精神指引`;
        ///
        /// // 调用AI服务
        /// const interpretation = await aiService.generate(prompt);
        /// ```
        fn generate_ai_prompt_context(
            reading_id: u64,
        ) -> Option<Vec<u8>>;

        /// 检查占卜记录是否存在
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `true`: 占卜记录存在
        /// - `false`: 占卜记录不存在
        fn reading_exists(reading_id: u64) -> bool;

        /// 获取占卜记录创建者
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(AccountId)`: 占卜者地址
        /// - `None`: 占卜记录不存在
        fn get_reading_owner(reading_id: u64) -> Option<AccountId>;

        /// 批量获取核心解卦结果
        ///
        /// 一次调用获取多个占卜记录的核心解卦，提高查询效率。
        ///
        /// # 参数
        /// - `reading_ids`: 塔罗牌占卜记录 ID 列表（最多100个）
        ///
        /// # 返回
        /// - `Vec<(u64, Option<TarotCoreInterpretation>)>`: ID和解卦结果的列表
        ///
        /// # 示例
        /// ```javascript
        /// const ids = [1, 2, 3, 4, 5];
        /// const results = await api.call.tarotApi.batchGetCoreInterpretations(ids);
        /// results.forEach(([id, core]) => {
        ///     if (core) {
        ///         console.log(`ID ${id}: 评分=${core.overallScore}`);
        ///     } else {
        ///         console.log(`ID ${id}: 不存在`);
        ///     }
        /// });
        /// ```
        fn batch_get_core_interpretations(
            reading_ids: Vec<u64>,
        ) -> Vec<(u64, Option<TarotCoreInterpretation>)>;

        /// 分析单张牌在特定牌阵位置的含义
        ///
        /// 独立分析某张牌在指定牌阵位置的解读，无需完整占卜记录。
        ///
        /// # 参数
        /// - `card_id`: 牌ID (0-77)
        /// - `is_reversed`: 是否逆位
        /// - `spread_type`: 牌阵类型
        /// - `position`: 在牌阵中的位置索引
        ///
        /// # 返回
        /// - `Some(CardInterpretation)`: 牌的分析结果
        /// - `None`: 参数无效
        ///
        /// # 示例
        /// ```javascript
        /// // 分析愚者牌在三张牌阵现在位置的含义
        /// const analysis = await api.call.tarotApi.analyzeCardInSpread(0, false, 3, 1);
        /// console.log('能量强度:', analysis.energyStrength);
        /// console.log('位置权重:', analysis.positionWeight);
        /// ```
        fn analyze_card_in_spread(
            card_id: u8,
            is_reversed: bool,
            spread_type: u8,
            position: u8,
        ) -> Option<CardInterpretation>;

        /// 分析两张牌之间的关系
        ///
        /// 分析任意两张牌之间的能量互动关系。
        ///
        /// # 参数
        /// - `card1_id`: 第一张牌ID (0-77)
        /// - `card2_id`: 第二张牌ID (0-77)
        ///
        /// # 返回
        /// - `Some(CardRelationship)`: 牌间关系分析
        /// - `None`: 参数无效
        ///
        /// # 关系类型
        /// - 0: 无明显关系
        /// - 1: 相生（能量互相增强）
        /// - 2: 相克（能量互相制约）
        /// - 3: 同元素强化
        /// - 4: 对立冲突
        /// - 5: 互补
        ///
        /// # 示例
        /// ```javascript
        /// // 分析愚者(0)和世界(21)的关系
        /// const relation = await api.call.tarotApi.analyzeCardRelationship(0, 21);
        /// console.log('关系类型:', relation.relationshipType);  // 5=互补
        /// console.log('关系强度:', relation.strength);
        /// ```
        fn analyze_card_relationship(
            card1_id: u8,
            card2_id: u8,
        ) -> Option<CardRelationship>;

        /// 获取牌阵能量分析
        ///
        /// 单独获取牌阵的能量分析，无需完整解卦。
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(SpreadEnergyAnalysis)`: 能量分析结果
        /// - `None`: 占卜记录不存在
        fn get_spread_energy(
            reading_id: u64,
        ) -> Option<SpreadEnergyAnalysis>;

        /// 获取时间线分析
        ///
        /// 获取占卜的时间线分析（仅适用于时间相关牌阵）。
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(TimelineAnalysis)`: 时间线分析结果
        /// - `None`: 占卜记录不存在或牌阵不支持时间线分析
        fn get_timeline_analysis(
            reading_id: u64,
        ) -> Option<TimelineAnalysis>;
    }
}
