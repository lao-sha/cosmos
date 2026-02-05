//! Lightning Network 支付客户端
//!
//! 支持 LNbits API 进行自动化 Lightning 支付
//!
//! 环境变量:
//! - LNBITS_URL: LNbits 服务地址 (默认 https://legend.lnbits.com)
//! - LNBITS_ADMIN_KEY: LNbits Admin Key (用于支付)
//! - LNBITS_INVOICE_KEY: LNbits Invoice Key (用于创建收款)

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Lightning 支付状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Complete,
    Failed,
    Expired,
}

/// Lightning Invoice 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningInvoice {
    pub bolt11: String,
    pub payment_hash: String,
    pub amount_sats: u64,
    pub memo: Option<String>,
    pub expiry: u64,
    pub status: PaymentStatus,
}

/// 支付结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub payment_hash: String,
    pub checking_id: String,
    pub fee_sats: u64,
    pub preimage: Option<String>,
}

/// 钱包余额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub balance_sats: u64,
    pub balance_msat: u64,
}

/// LNbits API 客户端
pub struct LnbitsClient {
    base_url: String,
    admin_key: String,
    invoice_key: Option<String>,
    client: reqwest::Client,
}

impl LnbitsClient {
    /// 从环境变量创建客户端
    pub fn new() -> Result<Self> {
        let admin_key = std::env::var("LNBITS_ADMIN_KEY")
            .map_err(|_| anyhow!("LNBITS_ADMIN_KEY not set. Get it from your LNbits wallet."))?;

        let base_url = std::env::var("LNBITS_URL")
            .unwrap_or_else(|_| "https://legend.lnbits.com".to_string());

        let invoice_key = std::env::var("LNBITS_INVOICE_KEY").ok();

        Ok(Self {
            base_url,
            admin_key,
            invoice_key,
            client: reqwest::Client::new(),
        })
    }

    /// 使用指定配置创建客户端
    pub fn with_config(base_url: &str, admin_key: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            admin_key: admin_key.to_string(),
            invoice_key: None,
            client: reqwest::Client::new(),
        }
    }

    /// 获取钱包余额
    pub async fn get_balance(&self) -> Result<WalletBalance> {
        let url = format!("{}/api/v1/wallet", self.base_url);
        
        let resp = self.client
            .get(&url)
            .header("X-Api-Key", &self.admin_key)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(anyhow!("LNbits API error ({}): {}", status, text));
        }

        let json: serde_json::Value = serde_json::from_str(&text)?;
        
        Ok(WalletBalance {
            balance_sats: json["balance"].as_u64().unwrap_or(0) / 1000,
            balance_msat: json["balance"].as_u64().unwrap_or(0),
        })
    }

    /// 支付 Lightning Invoice
    pub async fn pay_invoice(&self, bolt11: &str) -> Result<PaymentResult> {
        info!("⚡ Paying Lightning invoice...");
        
        let url = format!("{}/api/v1/payments", self.base_url);
        
        let body = serde_json::json!({
            "out": true,
            "bolt11": bolt11
        });

        let resp = self.client
            .post(&url)
            .header("X-Api-Key", &self.admin_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(anyhow!("Payment failed ({}): {}", status, text));
        }

        let json: serde_json::Value = serde_json::from_str(&text)?;
        
        info!("✅ Payment successful!");
        
        Ok(PaymentResult {
            payment_hash: json["payment_hash"].as_str().unwrap_or("").to_string(),
            checking_id: json["checking_id"].as_str().unwrap_or("").to_string(),
            fee_sats: 0,
            preimage: json["preimage"].as_str().map(|s| s.to_string()),
        })
    }

    /// 检查支付状态
    pub async fn check_payment(&self, payment_hash: &str) -> Result<PaymentStatus> {
        let url = format!("{}/api/v1/payments/{}", self.base_url, payment_hash);
        
        let resp = self.client
            .get(&url)
            .header("X-Api-Key", &self.admin_key)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(anyhow!("Check payment failed ({}): {}", status, text));
        }

        let json: serde_json::Value = serde_json::from_str(&text)?;
        
        let paid = json["paid"].as_bool().unwrap_or(false);
        let pending = json["pending"].as_bool().unwrap_or(true);

        if paid {
            Ok(PaymentStatus::Complete)
        } else if pending {
            Ok(PaymentStatus::Pending)
        } else {
            Ok(PaymentStatus::Failed)
        }
    }

    /// 创建 Invoice (收款)
    pub async fn create_invoice(&self, amount_sats: u64, memo: &str) -> Result<LightningInvoice> {
        let url = format!("{}/api/v1/payments", self.base_url);
        
        let key = self.invoice_key.as_ref().unwrap_or(&self.admin_key);
        
        let body = serde_json::json!({
            "out": false,
            "amount": amount_sats,
            "memo": memo
        });

        let resp = self.client
            .post(&url)
            .header("X-Api-Key", key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(anyhow!("Create invoice failed ({}): {}", status, text));
        }

        let json: serde_json::Value = serde_json::from_str(&text)?;
        
        Ok(LightningInvoice {
            bolt11: json["payment_request"].as_str().unwrap_or("").to_string(),
            payment_hash: json["payment_hash"].as_str().unwrap_or("").to_string(),
            amount_sats,
            memo: Some(memo.to_string()),
            expiry: 3600,
            status: PaymentStatus::Pending,
        })
    }

    /// 解码 Invoice
    pub async fn decode_invoice(&self, bolt11: &str) -> Result<LightningInvoice> {
        // 使用简单的 bolt11 解析
        // bolt11 格式: lnbc<amount><...>
        let amount_sats = self.parse_bolt11_amount(bolt11)?;
        
        Ok(LightningInvoice {
            bolt11: bolt11.to_string(),
            payment_hash: String::new(),
            amount_sats,
            memo: None,
            expiry: 3600,
            status: PaymentStatus::Pending,
        })
    }

    /// 解析 bolt11 金额
    fn parse_bolt11_amount(&self, bolt11: &str) -> Result<u64> {
        let lower = bolt11.to_lowercase();
        
        // 找到 lnbc 后的数字部分
        let start = if lower.starts_with("lnbc") {
            4
        } else if lower.starts_with("lntb") {
            4
        } else {
            return Err(anyhow!("Invalid bolt11 format"));
        };

        let chars: Vec<char> = lower[start..].chars().collect();
        let mut num_str = String::new();
        let mut multiplier_char = None;

        for c in chars {
            if c.is_ascii_digit() {
                num_str.push(c);
            } else {
                multiplier_char = Some(c);
                break;
            }
        }

        if num_str.is_empty() {
            return Ok(0); // Zero-amount invoice
        }

        let base: u64 = num_str.parse()?;
        
        // 乘数: m=milli, u=micro, n=nano, p=pico
        let sats = match multiplier_char {
            Some('m') => base * 100_000,      // mBTC -> sats
            Some('u') => base * 100,          // uBTC -> sats
            Some('n') => base / 10,           // nBTC -> sats
            Some('p') => base / 10_000,       // pBTC -> sats
            _ => base * 100_000_000,          // BTC -> sats
        };

        Ok(sats)
    }

    /// 等待支付完成
    pub async fn wait_for_payment(&self, payment_hash: &str, timeout_secs: u64) -> Result<bool> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                warn!("Payment timeout after {} seconds", timeout_secs);
                return Ok(false);
            }

            match self.check_payment(payment_hash).await? {
                PaymentStatus::Complete => {
                    info!("✅ Payment confirmed!");
                    return Ok(true);
                }
                PaymentStatus::Failed => {
                    return Err(anyhow!("Payment failed"));
                }
                PaymentStatus::Expired => {
                    return Err(anyhow!("Invoice expired"));
                }
                PaymentStatus::Pending => {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
}

/// Lightning 支付管理器 - 集成到云服务
pub struct LightningPaymentManager {
    client: Option<LnbitsClient>,
    auto_pay_enabled: bool,
    max_auto_pay_sats: u64,
}

impl LightningPaymentManager {
    pub fn new() -> Self {
        let client = LnbitsClient::new().ok();
        let auto_pay_enabled = std::env::var("LIGHTNING_AUTO_PAY")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let max_auto_pay_sats = std::env::var("LIGHTNING_MAX_AUTO_PAY_SATS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100_000); // 默认最多自动支付 100,000 sats (~$30)

        Self {
            client,
            auto_pay_enabled,
            max_auto_pay_sats,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    pub fn is_auto_pay_enabled(&self) -> bool {
        self.auto_pay_enabled && self.client.is_some()
    }

    /// 自动支付 Invoice (如果启用且金额在限制内)
    pub async fn auto_pay_if_enabled(&self, bolt11: &str) -> Result<Option<PaymentResult>> {
        let client = match &self.client {
            Some(c) => c,
            None => return Ok(None),
        };

        if !self.auto_pay_enabled {
            info!("⚡ Auto-pay disabled. Manual payment required.");
            return Ok(None);
        }

        // 解析金额
        let invoice = client.decode_invoice(bolt11).await?;
        
        if invoice.amount_sats > self.max_auto_pay_sats {
            warn!(
                "Invoice amount {} sats exceeds auto-pay limit {} sats",
                invoice.amount_sats, self.max_auto_pay_sats
            );
            return Ok(None);
        }

        // 检查余额
        let balance = client.get_balance().await?;
        if balance.balance_sats < invoice.amount_sats {
            warn!(
                "Insufficient balance: {} sats, need {} sats",
                balance.balance_sats, invoice.amount_sats
            );
            return Err(anyhow!("Insufficient LNbits balance"));
        }

        // 执行支付
        info!("⚡ Auto-paying {} sats...", invoice.amount_sats);
        let result = client.pay_invoice(bolt11).await?;
        
        Ok(Some(result))
    }

    /// 获取钱包余额
    pub async fn get_balance(&self) -> Result<WalletBalance> {
        match &self.client {
            Some(c) => c.get_balance().await,
            None => Err(anyhow!("LNbits not configured")),
        }
    }

    /// 手动支付
    pub async fn pay(&self, bolt11: &str) -> Result<PaymentResult> {
        match &self.client {
            Some(c) => c.pay_invoice(bolt11).await,
            None => Err(anyhow!("LNbits not configured")),
        }
    }
}

impl Default for LightningPaymentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bolt11_amount() {
        let client = LnbitsClient::with_config("http://localhost", "test");
        
        // 1000 sats = 10u (10 micro BTC)
        assert_eq!(client.parse_bolt11_amount("lnbc10u1...").unwrap(), 1000);
        
        // 100000 sats = 1m (1 milli BTC)
        assert_eq!(client.parse_bolt11_amount("lnbc1m1...").unwrap(), 100_000);
        
        // 100 sats = 1u
        assert_eq!(client.parse_bolt11_amount("lnbc1u1...").unwrap(), 100);
    }
}
