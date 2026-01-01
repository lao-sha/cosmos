//! # 小六壬解卦 Runtime API
//!
//! 提供免费的链下查询接口，用于获取课盘的解卦结果。
//!
//! ## 功能
//!
//! - 获取单个课盘的解卦结果
//! - 批量获取多个课盘的解卦结果
//!
//! ## 使用示例
//!
//! ```ignore
//! // 通过 RPC 调用
//! let interpretation = api.call.xiaoLiuRenInterpretationApi.getInterpretation(panId);
//! ```

use crate::interpretation::XiaoLiuRenInterpretation;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// 小六壬解卦 Runtime API
    pub trait XiaoLiuRenInterpretationApi {
        /// 获取课盘的解卦结果
        ///
        /// # 参数
        /// - `pan_id`: 课盘ID
        ///
        /// # 返回
        /// 解卦核心数据，如果课盘不存在则返回 None
        fn get_interpretation(pan_id: u64) -> Option<XiaoLiuRenInterpretation>;

        /// 批量获取解卦结果
        ///
        /// # 参数
        /// - `pan_ids`: 课盘ID列表
        ///
        /// # 返回
        /// 解卦结果列表，每个元素对应一个课盘ID
        fn get_interpretations_batch(pan_ids: Vec<u64>) -> Vec<Option<XiaoLiuRenInterpretation>>;
    }
}
