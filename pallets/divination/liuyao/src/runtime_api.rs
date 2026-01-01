//! # 六爻解卦 Runtime API
//!
//! 本模块定义了六爻解卦系统的 Runtime API，供前端通过 RPC 免费调用。
//!
//! ## 功能说明
//!
//! - `get_core_interpretation`: 获取核心解卦结果（吉凶、用神、世应、评分等）
//! - `get_full_interpretation`: 获取完整解卦结果（含六亲分析、各爻分析等）
//! - `get_interpretation_texts`: 获取解卦文本索引列表
//! - `gua_exists`: 检查卦象是否存在
//! - `get_gua_owner`: 获取卦象创建者
//!
//! ## 使用方式
//!
//! 前端通过 polkadot.js API 调用：
//! ```javascript
//! // 获取核心解卦
//! const core = await api.call.liuYaoApi.getCoreInterpretation(guaId, shiXiangType);
//!
//! // 访问核心数据
//! const { jiXiong, yongShenState, score, confidence } = core;
//!
//! // 获取完整解卦
//! const full = await api.call.liuYaoApi.getFullInterpretation(guaId, shiXiangType);
//!
//! // 获取解卦文本
//! const texts = await api.call.liuYaoApi.getInterpretationTexts(guaId, shiXiangType);
//! ```
//!
//! ## 数据结构
//!
//! ### 核心解卦 (LiuYaoCoreInterpretation, ~20 bytes)
//! - ji_xiong: 吉凶等级（大吉、吉、小吉、平、小凶、凶、大凶）
//! - yong_shen_qin: 用神六亲
//! - yong_shen_state: 用神状态（旺相、休囚、化进、化退等）
//! - dong_yao_count: 动爻数量
//! - ying_qi: 应期类型
//! - score: 综合评分 (0-100)
//! - confidence: 可信度 (0-100)
//!
//! ### 完整解卦 (LiuYaoFullInterpretation, ~165 bytes)
//! - core: 核心解卦
//! - gua_xiang: 卦象分析
//! - liu_qin: 六亲分析
//! - shen_sha: 神煞汇总
//! - yao_0..yao_5: 各爻分析

use crate::interpretation::{
    JieGuaTextType, LiuYaoCoreInterpretation, LiuYaoFullInterpretation,
};
use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// 六爻解卦 Runtime API
    ///
    /// 提供六爻卦象的免费查询接口，无需支付 Gas 费用。
    ///
    /// ## 设计原则
    ///
    /// - 分层接口：提供核心和完整两种接口，前端按需选择
    /// - 实时计算：每次调用实时计算，无需链上存储解卦结果
    /// - 事项导向：根据占问事项类型确定用神，针对性解卦
    pub trait LiuYaoApi<AccountId>
    where
        AccountId: Codec,
    {
        /// 获取核心解卦结果
        ///
        /// 返回最关键的解卦指标，约 20 bytes。
        ///
        /// # 参数
        /// - `gua_id`: 六爻卦象 ID
        /// - `shi_xiang`: 占问事项类型（财运、事业、婚姻等）
        ///
        /// # 返回
        /// - `Some(LiuYaoCoreInterpretation)`: 核心解卦结果
        /// - `None`: 卦象不存在
        ///
        /// # 返回数据结构
        /// ```text
        /// LiuYaoCoreInterpretation (~20 bytes)
        /// ├── ji_xiong          吉凶等级（大吉/吉/小吉/平/小凶/凶/大凶）
        /// ├── yong_shen_qin     用神六亲（父母/兄弟/子孙/妻财/官鬼）
        /// ├── yong_shen_state   用神状态（旺相/休囚/化进/化退/逢空等）
        /// ├── yong_shen_pos     用神所在爻位 (0-5, 255=伏神)
        /// ├── shi_yao_state     世爻状态
        /// ├── ying_yao_state    应爻状态
        /// ├── dong_yao_count    动爻数量 (0-6)
        /// ├── dong_yao_bitmap   动爻位图
        /// ├── xun_kong_bitmap   旬空位图
        /// ├── yue_po_bitmap     月破位图
        /// ├── ri_chong_bitmap   日冲位图
        /// ├── ying_qi           应期类型
        /// ├── ying_qi_zhi       应期地支
        /// ├── score             综合评分 (0-100)
        /// └── confidence        可信度 (0-100)
        /// ```
        ///
        /// # 示例
        /// ```javascript
        /// // 占问财运
        /// const core = await api.call.liuYaoApi.getCoreInterpretation(guaId, 0);
        /// console.log('吉凶:', core.jiXiong);  // 0=大吉, 6=大凶
        /// console.log('评分:', core.score);
        ///
        /// // 占问事业
        /// const core2 = await api.call.liuYaoApi.getCoreInterpretation(guaId, 1);
        /// ```
        fn get_core_interpretation(
            gua_id: u64,
            shi_xiang: u8,
        ) -> Option<LiuYaoCoreInterpretation>;

        /// 获取完整解卦结果
        ///
        /// 返回包含所有分析的完整解卦，约 165 bytes。
        ///
        /// # 参数
        /// - `gua_id`: 六爻卦象 ID
        /// - `shi_xiang`: 占问事项类型（0-9）
        ///
        /// # 返回
        /// - `Some(LiuYaoFullInterpretation)`: 完整解卦结果
        /// - `None`: 卦象不存在
        ///
        /// # 返回数据结构
        /// ```text
        /// LiuYaoFullInterpretation (~165 bytes)
        /// ├── core: LiuYaoCoreInterpretation  核心指标
        /// ├── gua_xiang: GuaXiangAnalysis     卦象分析
        /// │   ├── ben_gua_idx    本卦索引 (0-63)
        /// │   ├── bian_gua_idx   变卦索引 (255=无变卦)
        /// │   ├── hu_gua_idx     互卦索引
        /// │   ├── gong           卦宫 (0-7)
        /// │   ├── shi_pos        世爻位置
        /// │   ├── ying_pos       应爻位置
        /// │   ├── is_liu_chong   是否六冲卦
        /// │   └── is_liu_he      是否六合卦
        /// ├── liu_qin: LiuQinAnalysis         六亲分析
        /// │   ├── fu_mu      父母爻状态
        /// │   ├── xiong_di   兄弟爻状态
        /// │   ├── zi_sun     子孙爻状态
        /// │   ├── qi_cai     妻财爻状态
        /// │   └── guan_gui   官鬼爻状态
        /// ├── shen_sha: ShenShaSummary        神煞汇总
        /// │   ├── ji_shen_count    吉神数量
        /// │   └── xiong_sha_count  凶煞数量
        /// └── yao_0..yao_5: YaoAnalysis       各爻分析
        ///     ├── position     爻位
        ///     ├── wang_shuai   旺衰状态
        ///     ├── is_kong      是否逢空
        ///     ├── is_yue_po    是否月破
        ///     ├── is_ri_chong  是否日冲
        ///     └── is_dong      是否动爻
        /// ```
        ///
        /// # 示例
        /// ```javascript
        /// const full = await api.call.liuYaoApi.getFullInterpretation(guaId, 0);
        /// // 核心数据
        /// console.log('吉凶:', full.core.jiXiong);
        /// // 卦象分析
        /// console.log('本卦:', full.guaXiang.benGuaIdx);
        /// console.log('变卦:', full.guaXiang.bianGuaIdx);
        /// // 六亲分析
        /// console.log('妻财爻数量:', full.liuQin.qiCai.count);
        /// ```
        fn get_full_interpretation(
            gua_id: u64,
            shi_xiang: u8,
        ) -> Option<LiuYaoFullInterpretation>;

        /// 获取解卦文本索引列表
        ///
        /// 返回适用于当前卦象的解卦文本类型列表，前端根据索引显示对应文本。
        ///
        /// # 参数
        /// - `gua_id`: 六爻卦象 ID
        /// - `shi_xiang`: 占问事项类型
        ///
        /// # 返回
        /// - `Some(Vec<JieGuaTextType>)`: 解卦文本索引列表（最多20个）
        /// - `None`: 卦象不存在
        ///
        /// # 文本类型索引
        /// - 0-6: 吉凶总断（大吉、吉、小吉、平、小凶、凶、大凶）
        /// - 7-16: 用神状态（旺相、休囚、化进、化退等）
        /// - 17-22: 世应关系
        /// - 23-28: 动爻断语
        /// - 29-34: 特殊状态
        /// - 35-40: 应期断语
        ///
        /// # 示例
        /// ```javascript
        /// const texts = await api.call.liuYaoApi.getInterpretationTexts(guaId, 0);
        /// texts.forEach(textType => {
        ///     console.log(getTextByType(textType));
        /// });
        /// ```
        fn get_interpretation_texts(
            gua_id: u64,
            shi_xiang: u8,
        ) -> Option<Vec<JieGuaTextType>>;

        /// 检查卦象是否存在
        ///
        /// # 参数
        /// - `gua_id`: 六爻卦象 ID
        ///
        /// # 返回
        /// - `true`: 卦象存在
        /// - `false`: 卦象不存在
        fn gua_exists(gua_id: u64) -> bool;

        /// 获取卦象创建者
        ///
        /// # 参数
        /// - `gua_id`: 六爻卦象 ID
        ///
        /// # 返回
        /// - `Some(AccountId)`: 卦象创建者地址
        /// - `None`: 卦象不存在
        fn get_gua_owner(gua_id: u64) -> Option<AccountId>;
    }
}
