//! # OCW TRC20 验证模块
//!
//! 用于验证 USDT TRC20 交易，支持：
//! - TronGrid API 调用
//! - 多端点故障转移
//! - 端点健康评分
//! - 金额多档判定

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use sp_runtime::offchain::{http, Duration};
use sp_core::offchain::StorageKind;
use codec::{Encode, Decode};

// ==================== 常量配置 ====================

/// 默认 TRON API 端点列表（按优先级排序）
pub const DEFAULT_ENDPOINTS: &[&str] = &[
    "https://api.trongrid.io",         // TronGrid 官方
    "https://api.tronstack.io",        // TronStack 第三方
    "https://apilist.tronscanapi.com", // TronScan
];

/// 主端点（用于 URL 构建）
pub const TRONGRID_MAINNET: &str = "https://api.trongrid.io";

/// 官方 USDT TRC20 合约地址 (Mainnet)
pub const USDT_CONTRACT: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";

/// HTTP 请求超时（毫秒）
pub const HTTP_TIMEOUT_MS: u64 = 10_000;

/// HTTP 请求超时（毫秒）- 并行竞速模式
pub const HTTP_TIMEOUT_RACE_MS: u64 = 5_000;

/// 最小确认数
pub const MIN_CONFIRMATIONS: u32 = 19;

/// 端点健康评分存储键前缀
const ENDPOINT_HEALTH_PREFIX: &[u8] = b"entity_market_ocw_health::";

/// 健康评分衰减因子
const HEALTH_DECAY_FACTOR: u32 = 90;

// ==================== 端点健康评分系统 ====================

/// 端点健康状态
#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct EndpointHealth {
    /// 成功次数
    pub success_count: u32,
    /// 失败次数
    pub failure_count: u32,
    /// 平均响应时间（毫秒）
    pub avg_response_ms: u32,
    /// 健康评分 (0-100)
    pub score: u32,
    /// 最后更新时间戳
    pub last_updated: u64,
}

impl EndpointHealth {
    /// 计算健康评分
    pub fn calculate_score(&self) -> u32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 50;
        }
        
        let success_rate = (self.success_count as u64 * 50 / total as u64) as u32;
        let speed_score = if self.avg_response_ms < 1000 {
            50
        } else if self.avg_response_ms > 10000 {
            0
        } else {
            50 - ((self.avg_response_ms - 1000) * 50 / 9000)
        };
        
        success_rate + speed_score
    }
    
    /// 记录成功请求
    pub fn record_success(&mut self, response_ms: u32) {
        self.success_count = self.success_count.saturating_add(1);
        
        if self.avg_response_ms == 0 {
            self.avg_response_ms = response_ms;
        } else {
            self.avg_response_ms = (self.avg_response_ms * HEALTH_DECAY_FACTOR 
                + response_ms * (100 - HEALTH_DECAY_FACTOR)) / 100;
        }
        
        self.score = self.calculate_score();
        self.last_updated = current_timestamp_ms();
    }
    
    /// 记录失败请求
    pub fn record_failure(&mut self) {
        self.failure_count = self.failure_count.saturating_add(1);
        self.score = self.calculate_score();
        self.last_updated = current_timestamp_ms();
    }
}

/// 获取当前时间戳（毫秒）
fn current_timestamp_ms() -> u64 {
    sp_io::offchain::timestamp().unix_millis()
}

/// 获取端点健康状态
pub fn get_endpoint_health(endpoint: &str) -> EndpointHealth {
    let key = [ENDPOINT_HEALTH_PREFIX, endpoint.as_bytes()].concat();
    
    sp_io::offchain::local_storage_get(StorageKind::PERSISTENT, &key)
        .and_then(|data| EndpointHealth::decode(&mut &data[..]).ok())
        .unwrap_or_default()
}

/// 保存端点健康状态
fn save_endpoint_health(endpoint: &str, health: &EndpointHealth) {
    let key = [ENDPOINT_HEALTH_PREFIX, endpoint.as_bytes()].concat();
    sp_io::offchain::local_storage_set(
        StorageKind::PERSISTENT,
        &key,
        &health.encode(),
    );
}

/// 获取按健康评分排序的端点列表
pub fn get_sorted_endpoints() -> Vec<String> {
    let mut endpoints_with_scores: Vec<(String, u32)> = DEFAULT_ENDPOINTS
        .iter()
        .map(|e| {
            let health = get_endpoint_health(e);
            (String::from(*e), health.score)
        })
        .collect();
    
    endpoints_with_scores.sort_by(|a, b| b.1.cmp(&a.1));
    endpoints_with_scores.into_iter().map(|(e, _)| e).collect()
}

// ==================== 验证结果类型 ====================

/// 金额匹配状态
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AmountStatus {
    #[default]
    Unknown,
    /// 完全匹配（±0.5%）
    Exact,
    /// 多付
    Overpaid { excess: u64 },
    /// 少付（50-100%）
    Underpaid { shortage: u64 },
    /// 严重少付（<50%）
    SeverelyUnderpaid { shortage: u64 },
    /// 无效
    Invalid,
}

/// TRC20 交易验证结果
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TronTxVerification {
    pub tx_hash: Vec<u8>,
    pub is_valid: bool,
    pub from_address: Option<Vec<u8>>,
    pub to_address: Option<Vec<u8>>,
    pub actual_amount: Option<u64>,
    pub expected_amount: Option<u64>,
    pub confirmations: u32,
    pub error: Option<Vec<u8>>,
    pub amount_status: AmountStatus,
}

// ==================== 核心验证函数 ====================

/// 验证 TRC20 交易
pub fn verify_trc20_transaction(
    tx_hash: &[u8],
    expected_to: &[u8],
    expected_amount: u64,
) -> Result<TronTxVerification, &'static str> {
    let tx_hash_hex = bytes_to_hex(tx_hash);
    let url = format!("{}/v1/transactions/{}", TRONGRID_MAINNET, tx_hash_hex);
    
    let response = fetch_url_with_fallback(&url)?;
    parse_tron_response(&response, expected_to, expected_amount)
}

/// 发送 HTTP GET 请求（带故障转移）
fn fetch_url_with_fallback(url: &str) -> Result<Vec<u8>, &'static str> {
    let sorted_endpoints = get_sorted_endpoints();
    let mut last_error = "No endpoints available";
    
    log::info!(target: "entity-market-ocw", "Fetching with {} endpoints", sorted_endpoints.len());
    
    for (idx, endpoint) in sorted_endpoints.iter().enumerate() {
        let target_url = url.replace(TRONGRID_MAINNET, endpoint);
        let start_time = current_timestamp_ms();
        
        match fetch_url(&target_url) {
            Ok(response) => {
                let response_ms = (current_timestamp_ms() - start_time) as u32;
                
                let mut health = get_endpoint_health(endpoint);
                health.record_success(response_ms);
                save_endpoint_health(endpoint, &health);
                
                if idx > 0 {
                    log::info!(target: "entity-market-ocw", "Fallback {} succeeded ({}ms)", endpoint, response_ms);
                }
                return Ok(response);
            },
            Err(e) => {
                let mut health = get_endpoint_health(endpoint);
                health.record_failure();
                save_endpoint_health(endpoint, &health);
                
                log::warn!(target: "entity-market-ocw", "Endpoint {} failed: {}", endpoint, e);
                last_error = e;
            }
        }
    }
    
    log::error!(target: "entity-market-ocw", "All endpoints failed");
    Err(last_error)
}

/// 发送 HTTP GET 请求
fn fetch_url(url: &str) -> Result<Vec<u8>, &'static str> {
    let request = http::Request::get(url);
    let timeout = sp_io::offchain::timestamp()
        .add(Duration::from_millis(HTTP_TIMEOUT_MS));
    
    let pending = request
        .deadline(timeout)
        .send()
        .map_err(|_| "Failed to send HTTP request")?;
    
    let response = pending
        .try_wait(timeout)
        .map_err(|_| "HTTP request timeout")?
        .map_err(|_| "HTTP request failed")?;
    
    if response.code != 200 {
        return Err("Non-200 HTTP response");
    }
    
    let body = response.body().collect::<Vec<u8>>();
    if body.is_empty() {
        return Err("Empty response body");
    }
    
    Ok(body)
}

/// 解析 TronGrid API 响应
fn parse_tron_response(
    response: &[u8],
    expected_to: &[u8],
    expected_amount: u64,
) -> Result<TronTxVerification, &'static str> {
    let response_str = core::str::from_utf8(response)
        .map_err(|_| "Invalid UTF-8 response")?;
    
    let mut result = TronTxVerification::default();
    result.expected_amount = Some(expected_amount);
    
    // 检查交易是否成功
    if !response_str.contains("\"contractRet\":\"SUCCESS\"") 
        && !response_str.contains("\"contractRet\": \"SUCCESS\"") {
        result.error = Some(b"Transaction not successful".to_vec());
        return Ok(result);
    }
    
    // 检查收款地址
    let expected_to_hex = bytes_to_hex(expected_to);
    if !response_str.contains(&expected_to_hex) {
        result.error = Some(b"Recipient address mismatch".to_vec());
        return Ok(result);
    }
    
    // 提取实际金额
    let actual_amount = extract_amount(response_str);
    result.actual_amount = actual_amount;
    
    // 计算金额匹配状态
    let (amount_status, is_acceptable) = match actual_amount {
        Some(actual) => {
            let min_exact = expected_amount * 995 / 1000;
            let max_exact = expected_amount * 1005 / 1000;
            let severe_threshold = expected_amount / 2;
            
            if actual >= min_exact && actual <= max_exact {
                (AmountStatus::Exact, true)
            } else if actual > max_exact {
                let excess = actual.saturating_sub(expected_amount);
                (AmountStatus::Overpaid { excess }, true)
            } else if actual >= severe_threshold {
                let shortage = expected_amount.saturating_sub(actual);
                (AmountStatus::Underpaid { shortage }, false)
            } else if actual > 0 {
                let shortage = expected_amount.saturating_sub(actual);
                (AmountStatus::SeverelyUnderpaid { shortage }, false)
            } else {
                (AmountStatus::Invalid, false)
            }
        },
        None => (AmountStatus::Invalid, false)
    };
    
    result.amount_status = amount_status.clone();
    
    if !is_acceptable {
        let error_msg = match &amount_status {
            AmountStatus::Underpaid { shortage } => 
                format!("Underpaid by {}", shortage),
            AmountStatus::SeverelyUnderpaid { shortage } => 
                format!("Severely underpaid by {}", shortage),
            AmountStatus::Invalid => 
                "Invalid or zero amount".to_string(),
            _ => "Amount mismatch".to_string(),
        };
        result.error = Some(error_msg.into_bytes());
        return Ok(result);
    }
    
    result.is_valid = true;
    Ok(result)
}

/// 从响应中提取金额
fn extract_amount(response: &str) -> Option<u64> {
    let patterns = ["\"amount\":", "\"amount\": "];
    
    for pattern in patterns {
        if let Some(start) = response.find(pattern) {
            let after_key = &response[start + pattern.len()..];
            let trimmed = after_key.trim_start();
            let num_str: String = trimmed.chars()
                .take_while(|c| c.is_numeric())
                .collect();
            if !num_str.is_empty() {
                if let Ok(amount) = num_str.parse::<u64>() {
                    return Some(amount);
                }
            }
        }
    }
    None
}

/// 字节数组转十六进制字符串
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// 十六进制字符串转字节数组
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, &'static str> {
    if hex.len() % 2 != 0 {
        return Err("Invalid hex length");
    }
    
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| "Invalid hex"))
        .collect()
}
