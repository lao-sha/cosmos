//! # 奇门遁甲 Runtime API
//!
//! 本模块定义了奇门遁甲解卦的 Runtime API，供前端通过 RPC 调用。
//!
//! ## API 列表
//!
//! ### 解卦 API
//! 1. `get_core_interpretation` - 获取核心解卦
//! 2. `get_full_interpretation` - 获取完整解卦
//! 3. `get_palace_interpretation` - 获取单宫详细解读
//! 4. `get_yong_shen_analysis` - 获取用神分析
//! 5. `get_ying_qi_analysis` - 获取应期推算
//!
//! ### 隐私相关 API
//! 6. `get_encrypted_data` - 获取加密数据
//! 7. `get_owner_key_backup` - 获取所有者密钥备份
//! 8. `compute_chart` - 临时计算排盘（用于 Private 模式解密后的计算）
//!
//! ## 使用示例
//!
//! ```javascript
//! // 前端调用示例
//! const api = await ApiPromise.create({ provider: wsProvider });
//! const result = await api.call.qimenInterpretationApi.getCoreInterpretation(chartId);
//!
//! // 获取加密数据
//! const encryptedData = await api.call.qimenInterpretationApi.getEncryptedData(chartId);
//!
//! // Private 模式：前端解密后临时计算
//! const result = await api.call.qimenInterpretationApi.computeChart(
//!     solarYear, solarMonth, solarDay, hour, questionType, panMethod
//! );
//! ```

use crate::interpretation::*;
use crate::types::*;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// 奇门遁甲解卦 Runtime API
    ///
    /// 提供实时计算的解卦功能，无需链上存储
    pub trait QimenInterpretationApi {
        /// 获取核心解卦
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 核心解卦结果，如果排盘不存在返回 None
        fn get_core_interpretation(chart_id: u64) -> Option<QimenCoreInterpretation>;

        /// 获取完整解卦
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        /// - `question_type`: 问事类型
        ///
        /// # 返回
        ///
        /// 完整解卦结果，如果排盘不存在返回 None
        fn get_full_interpretation(
            chart_id: u64,
            question_type: QuestionType,
        ) -> Option<QimenFullInterpretation>;

        /// 获取单宫详细解读
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        /// - `palace_num`: 宫位数字（1-9）
        ///
        /// # 返回
        ///
        /// 单宫详细解读，如果排盘不存在或宫位无效返回 None
        fn get_palace_interpretation(
            chart_id: u64,
            palace_num: u8,
        ) -> Option<PalaceInterpretation>;

        /// 获取用神分析
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        /// - `question_type`: 问事类型
        ///
        /// # 返回
        ///
        /// 用神分析结果，如果排盘不存在返回 None
        fn get_yong_shen_analysis(
            chart_id: u64,
            question_type: QuestionType,
        ) -> Option<YongShenAnalysis>;

        /// 获取应期推算
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 应期推算结果，如果排盘不存在返回 None
        fn get_ying_qi_analysis(chart_id: u64) -> Option<YingQiAnalysis>;

        // ==================== 隐私相关 API ====================

        /// 获取加密数据
        ///
        /// 用于 Partial/Private 模式下获取链上存储的加密数据，
        /// 前端需要使用用户私钥解密。
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 加密数据（如果存在）
        fn get_encrypted_data(chart_id: u64) -> Option<Vec<u8>>;

        /// 获取所有者密钥备份
        ///
        /// 用于所有者恢复加密密钥或授权他人查看。
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 80 字节的密钥备份（如果存在）
        fn get_owner_key_backup(chart_id: u64) -> Option<[u8; 80]>;

        /// 临时计算排盘（用于 Private 模式）
        ///
        /// 当用户使用 Private 模式保存了排盘，但需要查看解读时：
        /// 1. 前端获取加密数据并解密
        /// 2. 使用解密后的日期时间参数调用此 API
        /// 3. 返回完整的排盘计算结果（不存储）
        ///
        /// # 参数
        ///
        /// - `solar_year`: 公历年份 (1901-2100)
        /// - `solar_month`: 公历月份 (1-12)
        /// - `solar_day`: 公历日期 (1-31)
        /// - `hour`: 小时 (0-23)
        /// - `question_type`: 问事类型
        /// - `pan_method`: 排盘方法 (0=转盘, 1=飞盘)
        ///
        /// # 返回
        ///
        /// 临时排盘结果（不存储到链上）
        fn compute_chart(
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            hour: u8,
            question_type: u8,
            pan_method: u8,
        ) -> Option<QimenChartResult>;

        /// 获取排盘元数据（公开信息）
        ///
        /// 返回排盘的公开元数据，不包含敏感信息。
        /// 适用于所有隐私模式。
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 公开元数据
        fn get_public_metadata(chart_id: u64) -> Option<QimenPublicMetadata>;
    }
}

/// 排盘公开元数据
///
/// 仅包含可公开的信息，不含敏感数据
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub struct QimenPublicMetadata {
    /// 排盘 ID
    pub id: u64,
    /// 隐私模式
    pub privacy_mode: pallet_divination_privacy::types::PrivacyMode,
    /// 起局方式
    pub method: DivinationMethod,
    /// 排盘方法
    pub pan_method: PanMethod,
    /// 排盘时间戳
    pub timestamp: u64,
    /// 问事类型
    pub question_type: Option<QuestionType>,
    /// 是否有加密数据
    pub has_encrypted_data: bool,
    /// 是否可解读（计算数据是否可用）
    pub can_interpret: bool,
}

/// 临时排盘结果
///
/// 用于 Private 模式下前端解密后的临时计算
/// 此结构不存储到链上，仅用于 Runtime API 返回
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo)]
pub struct QimenChartResult {
    /// 年柱干支
    pub year_ganzhi: GanZhi,
    /// 月柱干支
    pub month_ganzhi: GanZhi,
    /// 日柱干支
    pub day_ganzhi: GanZhi,
    /// 时柱干支
    pub hour_ganzhi: GanZhi,
    /// 节气
    pub jie_qi: JieQi,
    /// 阴阳遁
    pub dun_type: DunType,
    /// 三元
    pub san_yuan: SanYuan,
    /// 局数
    pub ju_number: u8,
    /// 值符星
    pub zhi_fu_xing: JiuXing,
    /// 值使门
    pub zhi_shi_men: BaMen,
    /// 九宫排盘
    pub palaces: [Palace; 9],
    /// 问事类型
    pub question_type: Option<QuestionType>,
    /// 排盘方法
    pub pan_method: PanMethod,
}
