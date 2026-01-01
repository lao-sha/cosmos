//! # Off-Chain Worker (OCW) 模块 - 汇率获取
//!
//! 本模块实现链下工作者，负责：
//! 1. 每24小时自动从 Exchange Rate API 获取 CNY/USD 汇率
//! 2. 计算 CNY/USDT 汇率（假设 USDT = USD）
//! 3. 将汇率数据存储到 offchain local storage
//!
//! ## API 数据源
//! - Exchange Rate API (免费): https://api.exchangerate-api.com/v4/latest/USD
//! - 每月 1500 次请求限制，每24小时请求1次足够使用
//!
//! ## 存储方式
//! - 使用 offchain local storage 存储汇率数据
//! - 链上 `get_cny_usdt_rate()` 函数提供默认值（7.2）
//! - 如需链上存储，可通过治理调用单独更新

extern crate alloc;
use alloc::{string::String, vec::Vec};

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{
    offchain::{http, Duration},
    traits::SaturatedConversion,
};

use crate::{Config, Pallet, ExchangeRateData};

/// 汇率 API URL
const EXCHANGE_RATE_API_URL: &str = "https://api.exchangerate-api.com/v4/latest/USD";

/// 每24小时更新一次（假设6秒一个区块，24小时 = 14400 个区块）
const UPDATE_INTERVAL_BLOCKS: u64 = 14400;

/// OCW 本地存储键 - 上次更新区块号
const LAST_UPDATE_BLOCK_KEY: &[u8] = b"pricing::last_update_block";

impl<T: Config> Pallet<T> {
    /// OCW 主入口函数
    ///
    /// 在每个区块执行一次，检查是否需要更新汇率
    pub fn offchain_worker(block_number: BlockNumberFor<T>) {
        log::info!("💱 Pricing OCW 执行于区块 #{:?}", block_number);

        // 检查是否应该在这个区块执行更新
        if !Self::should_fetch_rate(block_number) {
            log::debug!("⏭️ 跳过汇率更新，未到更新时间");
            return;
        }

        // 获取汇率数据
        match Self::fetch_exchange_rate() {
            Ok(rate_data) => {
                log::info!(
                    "✅ 获取汇率成功: CNY/USDT = {}.{:06}",
                    rate_data.cny_rate / 1_000_000,
                    rate_data.cny_rate % 1_000_000
                );

                // 直接存储到链上（使用 offchain_index）
                // 注意：这种方式只是本地存储，需要配合 ValidateUnsigned 来更新链上状态
                Self::update_last_fetch_block(block_number);

                // 存储到 offchain 本地存储供后续使用
                Self::store_rate_locally(&rate_data);

                log::info!(
                    "📊 汇率数据已缓存到本地存储: CNY/USDT = {}.{:06}",
                    rate_data.cny_rate / 1_000_000,
                    rate_data.cny_rate % 1_000_000
                );
            }
            Err(e) => {
                log::error!("❌ 汇率获取失败: {:?}", e);
            }
        }
    }

    /// 判断是否应该获取汇率
    ///
    /// 基于本地存储判断是否已过24小时
    fn should_fetch_rate(current_block: BlockNumberFor<T>) -> bool {
        let current_block_u64: u64 = current_block.saturated_into();

        // 从本地存储读取上次更新的区块号
        let last_block = sp_io::offchain::local_storage_get(
            sp_core::offchain::StorageKind::PERSISTENT,
            LAST_UPDATE_BLOCK_KEY,
        )
        .and_then(|bytes| {
            if bytes.len() == 8 {
                let arr: [u8; 8] = bytes.try_into().ok()?;
                Some(u64::from_le_bytes(arr))
            } else {
                None
            }
        })
        .unwrap_or(0);

        // 如果距离上次更新超过 UPDATE_INTERVAL_BLOCKS 个区块，则需要更新
        current_block_u64.saturating_sub(last_block) >= UPDATE_INTERVAL_BLOCKS
    }

    /// 更新本地存储的最后获取区块号
    fn update_last_fetch_block(block_number: BlockNumberFor<T>) {
        let block_u64: u64 = block_number.saturated_into();
        sp_io::offchain::local_storage_set(
            sp_core::offchain::StorageKind::PERSISTENT,
            LAST_UPDATE_BLOCK_KEY,
            &block_u64.to_le_bytes(),
        );
    }

    /// 存储汇率到本地 offchain 存储
    fn store_rate_locally(rate_data: &ExchangeRateData) {
        let key = b"pricing::cny_rate";
        let value = rate_data.encode();
        sp_io::offchain::local_storage_set(
            sp_core::offchain::StorageKind::PERSISTENT,
            key,
            &value,
        );
    }

    /// 从本地 offchain 存储读取汇率
    pub fn get_rate_from_local_storage() -> Option<ExchangeRateData> {
        let key = b"pricing::cny_rate";
        sp_io::offchain::local_storage_get(
            sp_core::offchain::StorageKind::PERSISTENT,
            key,
        )
        .and_then(|bytes| ExchangeRateData::decode(&mut &bytes[..]).ok())
    }

    /// 从 Exchange Rate API 获取汇率
    ///
    /// API 响应格式:
    /// ```json
    /// {
    ///   "base": "USD",
    ///   "rates": {
    ///     "CNY": 7.2345,
    ///     ...
    ///   }
    /// }
    /// ```
    fn fetch_exchange_rate() -> Result<ExchangeRateData, &'static str> {
        log::info!("🌐 正在从 {} 获取汇率...", EXCHANGE_RATE_API_URL);

        // 创建 HTTP GET 请求
        let request = http::Request::get(EXCHANGE_RATE_API_URL);

        // 设置超时时间（10秒）
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(10_000));

        // 发送请求
        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| "HTTP 请求发送失败")?;

        // 等待响应
        let response = pending
            .try_wait(deadline)
            .map_err(|_| "HTTP 请求超时")?
            .map_err(|_| "HTTP 响应错误")?;

        // 检查状态码
        if response.code != 200 {
            log::error!("❌ HTTP 状态码: {}", response.code);
            return Err("HTTP 状态码非 200");
        }

        // 读取响应体
        let body = response.body().collect::<Vec<u8>>();
        let body_str = sp_std::str::from_utf8(&body).map_err(|_| "响应体不是有效的 UTF-8")?;

        log::debug!("📥 API 响应: {}", body_str);

        // 解析 JSON 获取 CNY 汇率
        let cny_rate = Self::parse_cny_rate(body_str)?;

        // 获取当前时间戳
        let timestamp = sp_io::offchain::timestamp().unix_millis() / 1000; // 转换为秒

        Ok(ExchangeRateData {
            cny_rate,
            updated_at: timestamp,
        })
    }

    /// 从 JSON 响应中解析 CNY 汇率
    ///
    /// 使用简单的字符串匹配解析，避免依赖完整的 JSON 库
    ///
    /// # 返回
    /// - `u64`: CNY/USD 汇率（精度 10^6，即 7.2345 → 7_234_500）
    fn parse_cny_rate(json: &str) -> Result<u64, &'static str> {
        // 查找 "CNY": 的位置
        let cny_pattern = "\"CNY\":";
        let start = json.find(cny_pattern).ok_or("JSON 中未找到 CNY 汇率")?;
        let value_start = start + cny_pattern.len();

        // 提取数值部分
        let remaining = &json[value_start..];

        // 跳过空白字符
        let remaining = remaining.trim_start();

        // 找到数值的结束位置（逗号、右括号或空白）
        let end_chars = [',', '}', ' ', '\n', '\r', '\t'];
        let mut value_end = remaining.len();
        for (i, ch) in remaining.char_indices() {
            if end_chars.contains(&ch) {
                value_end = i;
                break;
            }
        }

        let value_str = &remaining[..value_end];
        log::debug!("🔢 解析 CNY 汇率字符串: '{}'", value_str);

        // 解析浮点数并转换为精度 10^6 的整数
        Self::parse_rate_string(value_str)
    }

    /// 解析汇率字符串为整数（精度 10^6）
    ///
    /// 例如: "7.2345" → 7_234_500
    fn parse_rate_string(s: &str) -> Result<u64, &'static str> {
        // 分离整数部分和小数部分
        let parts: Vec<&str> = s.split('.').collect();

        let integer_part: u64 = parts.get(0)
            .ok_or("无效的汇率格式")?
            .parse()
            .map_err(|_| "整数部分解析失败")?;

        let decimal_part: u64 = if parts.len() > 1 {
            let decimal_str = parts[1];
            // 补齐或截断到6位小数
            let mut padded = String::from(decimal_str);
            while padded.len() < 6 {
                padded.push('0');
            }
            padded.truncate(6);
            padded.parse().map_err(|_| "小数部分解析失败")?
        } else {
            0
        };

        // 组合为精度 10^6 的整数
        let rate = integer_part
            .checked_mul(1_000_000)
            .ok_or("汇率溢出")?
            .checked_add(decimal_part)
            .ok_or("汇率溢出")?;

        Ok(rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要 mock 环境，暂时注释掉
    // 可以在集成测试中验证
    /*
    #[test]
    fn test_parse_rate_string() {
        // 测试正常汇率
        assert_eq!(
            Pallet::<crate::mock::Test>::parse_rate_string("7.2345").unwrap(),
            7_234_500
        );
    }
    */
}
