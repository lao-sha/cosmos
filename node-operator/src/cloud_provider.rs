//! 云服务商 API 客户端
//!
//! 支持 Vultr、DigitalOcean 和 LNVPS 的服务器创建和管理
//!
//! 环境变量:
//! - VULTR_API_KEY: Vultr API 密钥
//! - DIGITALOCEAN_API_KEY: DigitalOcean API 密钥
//! - LNVPS_API_KEY: LNVPS API 密钥 (通过 Nostr 获取)
//! - CLOUD_PROVIDER: "vultr", "digitalocean", 或 "lnvps"
//! - CLOUD_BUDGET_MONTHLY: 月度预算上限（美元）

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// 云服务提供商
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudProvider {
    Vultr,
    DigitalOcean,
    Lnvps,
}

impl CloudProvider {
    pub fn from_env() -> Result<Self> {
        let provider = std::env::var("CLOUD_PROVIDER")
            .unwrap_or_else(|_| "vultr".to_string());
        match provider.to_lowercase().as_str() {
            "vultr" => Ok(Self::Vultr),
            "digitalocean" | "do" => Ok(Self::DigitalOcean),
            "lnvps" | "ln" => Ok(Self::Lnvps),
            _ => Err(anyhow!("Unknown cloud provider: {}", provider)),
        }
    }
}

/// 服务器规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPlan {
    pub id: String,
    pub name: String,
    pub vcpu: u32,
    pub ram_mb: u32,
    pub disk_gb: u32,
    pub bandwidth_tb: f32,
    pub price_monthly: f32,
    pub price_hourly: f32,
}

/// 区域信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub country: String,
}

/// 操作系统镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsImage {
    pub id: String,
    pub name: String,
    pub family: String,
}

/// 已创建的服务器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub region: String,
    pub plan: String,
    pub os: String,
    pub ip_address: String,
    pub status: String,
    pub created_at: String,
    pub monthly_cost: f32,
}

/// 创建服务器请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub region: String,
    pub plan: String,
    pub os: String,
    pub ssh_keys: Vec<String>,
    pub label: Option<String>,
}

/// 预算管理器
pub struct BudgetManager {
    pub monthly_limit: f32,
    pub current_spend: f32,
}

impl BudgetManager {
    pub fn from_env() -> Self {
        let limit = std::env::var("CLOUD_BUDGET_MONTHLY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100.0); // 默认 $100/月

        Self {
            monthly_limit: limit,
            current_spend: 0.0,
        }
    }

    pub fn can_afford(&self, monthly_cost: f32) -> bool {
        self.current_spend + monthly_cost <= self.monthly_limit
    }

    pub fn remaining(&self) -> f32 {
        self.monthly_limit - self.current_spend
    }
}

// ============================================================================
// Vultr API 客户端
// ============================================================================

pub struct VultrClient {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl VultrClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("VULTR_API_KEY")
            .map_err(|_| anyhow!("VULTR_API_KEY not set"))?;

        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://api.vultr.com/v2".to_string(),
        })
    }

    async fn request(&self, method: &str, endpoint: &str, body: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let mut req = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(anyhow!("Unsupported method: {}", method)),
        };

        req = req.header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            req = req.json(&body);
        }

        let response = req.send().await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("Vultr API error ({}): {}", status, text));
        }

        if text.is_empty() {
            Ok(serde_json::json!({}))
        } else {
            Ok(serde_json::from_str(&text)?)
        }
    }

    /// 获取可用区域
    pub async fn list_regions(&self) -> Result<Vec<Region>> {
        let resp = self.request("GET", "/regions", None).await?;
        
        let regions = resp["regions"].as_array()
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(regions.iter().map(|r| Region {
            id: r["id"].as_str().unwrap_or("").to_string(),
            name: r["city"].as_str().unwrap_or("").to_string(),
            country: r["country"].as_str().unwrap_or("").to_string(),
        }).collect())
    }

    /// 获取可用套餐
    pub async fn list_plans(&self) -> Result<Vec<ServerPlan>> {
        let resp = self.request("GET", "/plans", None).await?;
        
        let plans = resp["plans"].as_array()
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(plans.iter().filter_map(|p| {
            // 只返回云计算类型
            if p["type"].as_str() != Some("vc2") {
                return None;
            }
            Some(ServerPlan {
                id: p["id"].as_str().unwrap_or("").to_string(),
                name: format!("{} vCPU, {}MB RAM, {}GB SSD",
                    p["vcpu_count"].as_u64().unwrap_or(0),
                    p["ram"].as_u64().unwrap_or(0),
                    p["disk"].as_u64().unwrap_or(0)),
                vcpu: p["vcpu_count"].as_u64().unwrap_or(0) as u32,
                ram_mb: p["ram"].as_u64().unwrap_or(0) as u32,
                disk_gb: p["disk"].as_u64().unwrap_or(0) as u32,
                bandwidth_tb: p["bandwidth"].as_f64().unwrap_or(0.0) as f32 / 1024.0,
                price_monthly: p["monthly_cost"].as_f64().unwrap_or(0.0) as f32,
                price_hourly: p["hourly_cost"].as_f64().unwrap_or(0.0) as f32,
            })
        }).collect())
    }

    /// 获取操作系统镜像
    pub async fn list_os(&self) -> Result<Vec<OsImage>> {
        let resp = self.request("GET", "/os", None).await?;
        
        let images = resp["os"].as_array()
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(images.iter().filter_map(|o| {
            let family = o["family"].as_str().unwrap_or("");
            // 只返回常用系统
            if !["ubuntu", "debian", "centos"].contains(&family) {
                return None;
            }
            Some(OsImage {
                id: o["id"].as_u64().unwrap_or(0).to_string(),
                name: o["name"].as_str().unwrap_or("").to_string(),
                family: family.to_string(),
            })
        }).collect())
    }

    /// 创建服务器
    pub async fn create_server(&self, req: &CreateServerRequest) -> Result<Server> {
        info!("Creating Vultr server: {} in {}", req.name, req.region);

        let body = serde_json::json!({
            "region": req.region,
            "plan": req.plan,
            "os_id": req.os.parse::<u64>().unwrap_or(0),
            "label": req.name,
            "hostname": req.name,
            "sshkey_id": req.ssh_keys,
            "tags": ["node-operator", "substrate"]
        });

        let resp = self.request("POST", "/instances", Some(body)).await?;
        let instance = &resp["instance"];

        Ok(Server {
            id: instance["id"].as_str().unwrap_or("").to_string(),
            name: instance["label"].as_str().unwrap_or("").to_string(),
            region: instance["region"].as_str().unwrap_or("").to_string(),
            plan: instance["plan"].as_str().unwrap_or("").to_string(),
            os: instance["os"].as_str().unwrap_or("").to_string(),
            ip_address: instance["main_ip"].as_str().unwrap_or("pending").to_string(),
            status: instance["status"].as_str().unwrap_or("pending").to_string(),
            created_at: instance["date_created"].as_str().unwrap_or("").to_string(),
            monthly_cost: 0.0, // 需要从 plan 查询
        })
    }

    /// 获取服务器列表
    pub async fn list_servers(&self) -> Result<Vec<Server>> {
        let resp = self.request("GET", "/instances", None).await?;
        
        let instances = resp["instances"].as_array()
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(instances.iter().map(|i| Server {
            id: i["id"].as_str().unwrap_or("").to_string(),
            name: i["label"].as_str().unwrap_or("").to_string(),
            region: i["region"].as_str().unwrap_or("").to_string(),
            plan: i["plan"].as_str().unwrap_or("").to_string(),
            os: i["os"].as_str().unwrap_or("").to_string(),
            ip_address: i["main_ip"].as_str().unwrap_or("").to_string(),
            status: i["status"].as_str().unwrap_or("").to_string(),
            created_at: i["date_created"].as_str().unwrap_or("").to_string(),
            monthly_cost: 0.0,
        }).collect())
    }

    /// 删除服务器
    pub async fn destroy_server(&self, server_id: &str) -> Result<()> {
        warn!("Destroying Vultr server: {}", server_id);
        self.request("DELETE", &format!("/instances/{}", server_id), None).await?;
        Ok(())
    }

    /// 获取服务器详情
    pub async fn get_server(&self, server_id: &str) -> Result<Server> {
        let resp = self.request("GET", &format!("/instances/{}", server_id), None).await?;
        let i = &resp["instance"];

        Ok(Server {
            id: i["id"].as_str().unwrap_or("").to_string(),
            name: i["label"].as_str().unwrap_or("").to_string(),
            region: i["region"].as_str().unwrap_or("").to_string(),
            plan: i["plan"].as_str().unwrap_or("").to_string(),
            os: i["os"].as_str().unwrap_or("").to_string(),
            ip_address: i["main_ip"].as_str().unwrap_or("").to_string(),
            status: i["status"].as_str().unwrap_or("").to_string(),
            created_at: i["date_created"].as_str().unwrap_or("").to_string(),
            monthly_cost: 0.0,
        })
    }
}

// ============================================================================
// DigitalOcean API 客户端
// ============================================================================

pub struct DigitalOceanClient {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl DigitalOceanClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("DIGITALOCEAN_API_KEY")
            .map_err(|_| anyhow!("DIGITALOCEAN_API_KEY not set"))?;

        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://api.digitalocean.com/v2".to_string(),
        })
    }

    async fn request(&self, method: &str, endpoint: &str, body: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let mut req = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(anyhow!("Unsupported method: {}", method)),
        };

        req = req.header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            req = req.json(&body);
        }

        let response = req.send().await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("DigitalOcean API error ({}): {}", status, text));
        }

        if text.is_empty() {
            Ok(serde_json::json!({}))
        } else {
            Ok(serde_json::from_str(&text)?)
        }
    }

    /// 获取可用区域
    pub async fn list_regions(&self) -> Result<Vec<Region>> {
        let resp = self.request("GET", "/regions", None).await?;
        
        let regions = resp["regions"].as_array()
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(regions.iter().filter_map(|r| {
            if !r["available"].as_bool().unwrap_or(false) {
                return None;
            }
            Some(Region {
                id: r["slug"].as_str().unwrap_or("").to_string(),
                name: r["name"].as_str().unwrap_or("").to_string(),
                country: r["slug"].as_str().unwrap_or("").to_string(),
            })
        }).collect())
    }

    /// 获取可用套餐
    pub async fn list_plans(&self) -> Result<Vec<ServerPlan>> {
        let resp = self.request("GET", "/sizes", None).await?;
        
        let sizes = resp["sizes"].as_array()
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(sizes.iter().filter_map(|s| {
            if !s["available"].as_bool().unwrap_or(false) {
                return None;
            }
            // 只返回常规 droplet
            let slug = s["slug"].as_str().unwrap_or("");
            if !slug.starts_with("s-") && !slug.starts_with("c-") {
                return None;
            }
            Some(ServerPlan {
                id: slug.to_string(),
                name: s["description"].as_str().unwrap_or(slug).to_string(),
                vcpu: s["vcpus"].as_u64().unwrap_or(0) as u32,
                ram_mb: s["memory"].as_u64().unwrap_or(0) as u32,
                disk_gb: s["disk"].as_u64().unwrap_or(0) as u32,
                bandwidth_tb: s["transfer"].as_f64().unwrap_or(0.0) as f32,
                price_monthly: s["price_monthly"].as_f64().unwrap_or(0.0) as f32,
                price_hourly: s["price_hourly"].as_f64().unwrap_or(0.0) as f32,
            })
        }).collect())
    }

    /// 获取操作系统镜像
    pub async fn list_os(&self) -> Result<Vec<OsImage>> {
        let resp = self.request("GET", "/images?type=distribution", None).await?;
        
        let images = resp["images"].as_array()
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(images.iter().filter_map(|o| {
            let distro = o["distribution"].as_str().unwrap_or("");
            if !["Ubuntu", "Debian", "CentOS"].contains(&distro) {
                return None;
            }
            Some(OsImage {
                id: o["slug"].as_str().unwrap_or("").to_string(),
                name: o["name"].as_str().unwrap_or("").to_string(),
                family: distro.to_lowercase(),
            })
        }).collect())
    }

    /// 创建服务器 (Droplet)
    pub async fn create_server(&self, req: &CreateServerRequest) -> Result<Server> {
        info!("Creating DigitalOcean droplet: {} in {}", req.name, req.region);

        let body = serde_json::json!({
            "name": req.name,
            "region": req.region,
            "size": req.plan,
            "image": req.os,
            "ssh_keys": req.ssh_keys,
            "tags": ["node-operator", "substrate"]
        });

        let resp = self.request("POST", "/droplets", Some(body)).await?;
        let droplet = &resp["droplet"];

        Ok(Server {
            id: droplet["id"].as_u64().unwrap_or(0).to_string(),
            name: droplet["name"].as_str().unwrap_or("").to_string(),
            region: droplet["region"]["slug"].as_str().unwrap_or("").to_string(),
            plan: droplet["size_slug"].as_str().unwrap_or("").to_string(),
            os: droplet["image"]["slug"].as_str().unwrap_or("").to_string(),
            ip_address: "pending".to_string(), // IP 需要等待分配
            status: droplet["status"].as_str().unwrap_or("new").to_string(),
            created_at: droplet["created_at"].as_str().unwrap_or("").to_string(),
            monthly_cost: 0.0,
        })
    }

    /// 获取服务器列表
    pub async fn list_servers(&self) -> Result<Vec<Server>> {
        let resp = self.request("GET", "/droplets?tag_name=node-operator", None).await?;
        
        let droplets = resp["droplets"].as_array()
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(droplets.iter().map(|d| {
            let ip = d["networks"]["v4"].as_array()
                .and_then(|nets| nets.iter().find(|n| n["type"] == "public"))
                .and_then(|n| n["ip_address"].as_str())
                .unwrap_or("");

            Server {
                id: d["id"].as_u64().unwrap_or(0).to_string(),
                name: d["name"].as_str().unwrap_or("").to_string(),
                region: d["region"]["slug"].as_str().unwrap_or("").to_string(),
                plan: d["size_slug"].as_str().unwrap_or("").to_string(),
                os: d["image"]["slug"].as_str().unwrap_or("").to_string(),
                ip_address: ip.to_string(),
                status: d["status"].as_str().unwrap_or("").to_string(),
                created_at: d["created_at"].as_str().unwrap_or("").to_string(),
                monthly_cost: 0.0,
            }
        }).collect())
    }

    /// 删除服务器
    pub async fn destroy_server(&self, server_id: &str) -> Result<()> {
        warn!("Destroying DigitalOcean droplet: {}", server_id);
        self.request("DELETE", &format!("/droplets/{}", server_id), None).await?;
        Ok(())
    }
}

// ============================================================================
// LNVPS API 客户端 (Bitcoin Lightning 支付)
// ============================================================================

/// LNVPS 客户端 - 支持 Bitcoin Lightning 支付
/// 
/// 特点:
/// - 无需 KYC
/// - 使用 Lightning Network 支付
/// - 基于 Nostr 账户系统
/// - 开源: https://git.v0l.io/LNVPS
pub struct LnvpsClient {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl LnvpsClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("LNVPS_API_KEY")
            .map_err(|_| anyhow!("LNVPS_API_KEY not set. Get it from https://lnvps.net after login with Nostr"))?;

        let base_url = std::env::var("LNVPS_BASE_URL")
            .unwrap_or_else(|_| "https://api.lnvps.net".to_string());

        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
            base_url,
        })
    }

    async fn request(&self, method: &str, endpoint: &str, body: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let mut req = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(anyhow!("Unsupported method: {}", method)),
        };

        req = req.header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            req = req.json(&body);
        }

        let response = req.send().await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("LNVPS API error ({}): {}", status, text));
        }

        if text.is_empty() {
            Ok(serde_json::json!({}))
        } else {
            Ok(serde_json::from_str(&text)?)
        }
    }

    /// 获取可用区域
    pub async fn list_regions(&self) -> Result<Vec<Region>> {
        let resp = self.request("GET", "/api/v1/regions", None).await?;
        
        let regions = resp.as_array()
            .or_else(|| resp["regions"].as_array())
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(regions.iter().map(|r| {
            let id = r["id"].as_str()
                .map(|s| s.to_string())
                .or_else(|| r["id"].as_u64().map(|n| n.to_string()))
                .unwrap_or_default();
            Region {
                id,
                name: r["name"].as_str().unwrap_or("").to_string(),
                country: r["country"].as_str().unwrap_or("EU").to_string(),
            }
        }).collect())
    }

    /// 获取可用套餐 (templates)
    pub async fn list_plans(&self) -> Result<Vec<ServerPlan>> {
        let resp = self.request("GET", "/api/v1/templates", None).await?;
        
        let templates = resp.as_array()
            .or_else(|| resp["templates"].as_array())
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(templates.iter().map(|t| {
            let price_sats = t["cost_plan"]["amount"].as_u64().unwrap_or(0);
            // 转换 sats 到 EUR (大约 1 EUR = 2000 sats)
            let price_monthly = price_sats as f32 / 2000.0;
            
            ServerPlan {
                id: t["id"].as_u64().unwrap_or(0).to_string(),
                name: t["name"].as_str().unwrap_or("").to_string(),
                vcpu: t["cpu"].as_u64().unwrap_or(1) as u32,
                ram_mb: t["memory"].as_u64().unwrap_or(1024) as u32,
                disk_gb: t["disk_size"].as_u64().unwrap_or(20) as u32,
                bandwidth_tb: 1.0,
                price_monthly,
                price_hourly: price_monthly / 720.0,
            }
        }).collect())
    }

    /// 获取操作系统镜像
    pub async fn list_os(&self) -> Result<Vec<OsImage>> {
        let resp = self.request("GET", "/api/v1/images", None).await?;
        
        let images = resp.as_array()
            .or_else(|| resp["images"].as_array())
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(images.iter().map(|o| OsImage {
            id: o["id"].as_u64().unwrap_or(0).to_string(),
            name: o["name"].as_str().unwrap_or("").to_string(),
            family: o["distribution"].as_str().unwrap_or("linux").to_lowercase(),
        }).collect())
    }

    /// 创建服务器 - 返回 Lightning Invoice 用于支付
    pub async fn create_server(&self, req: &CreateServerRequest) -> Result<Server> {
        info!("Creating LNVPS VM: {} with template {}", req.name, req.plan);

        let body = serde_json::json!({
            "template_id": req.plan.parse::<u64>().unwrap_or(0),
            "image_id": req.os.parse::<u64>().unwrap_or(0),
            "ssh_key": req.ssh_keys.first().unwrap_or(&String::new()),
            "hostname": req.name
        });

        let resp = self.request("POST", "/api/v1/vm", Some(body)).await?;

        // LNVPS 返回一个 Lightning Invoice，需要支付后才能创建
        let invoice = resp["invoice"].as_str().unwrap_or("");
        let vm_id = resp["id"].as_u64().unwrap_or(0).to_string();

        if !invoice.is_empty() {
            info!("⚡ Lightning Invoice generated. Pay to activate VM.");
            info!("Invoice: {}", invoice);
        }

        Ok(Server {
            id: vm_id,
            name: req.name.clone(),
            region: req.region.clone(),
            plan: req.plan.clone(),
            os: req.os.clone(),
            ip_address: resp["ip"].as_str().unwrap_or("pending").to_string(),
            status: resp["status"].as_str().unwrap_or("awaiting_payment").to_string(),
            created_at: resp["created"].as_str().unwrap_or("").to_string(),
            monthly_cost: 0.0,
        })
    }

    /// 获取服务器列表
    pub async fn list_servers(&self) -> Result<Vec<Server>> {
        let resp = self.request("GET", "/api/v1/vm", None).await?;
        
        let vms = resp.as_array()
            .or_else(|| resp["vms"].as_array())
            .ok_or_else(|| anyhow!("Invalid response"))?;

        Ok(vms.iter().map(|v| Server {
            id: v["id"].as_u64().unwrap_or(0).to_string(),
            name: v["hostname"].as_str().unwrap_or("").to_string(),
            region: v["region"].as_str().unwrap_or("").to_string(),
            plan: v["template_id"].as_u64().unwrap_or(0).to_string(),
            os: v["image"].as_str().unwrap_or("").to_string(),
            ip_address: v["ip"].as_str().unwrap_or("").to_string(),
            status: v["status"].as_str().unwrap_or("").to_string(),
            created_at: v["created"].as_str().unwrap_or("").to_string(),
            monthly_cost: 0.0,
        }).collect())
    }

    /// 删除服务器
    pub async fn destroy_server(&self, server_id: &str) -> Result<()> {
        warn!("Destroying LNVPS VM: {}", server_id);
        self.request("DELETE", &format!("/api/v1/vm/{}", server_id), None).await?;
        Ok(())
    }

    /// 获取 Lightning Invoice 状态
    pub async fn check_invoice(&self, vm_id: &str) -> Result<String> {
        let resp = self.request("GET", &format!("/api/v1/vm/{}/invoice", vm_id), None).await?;
        Ok(resp["status"].as_str().unwrap_or("unknown").to_string())
    }
}

// ============================================================================
// 统一云服务接口
// ============================================================================

pub enum CloudClient {
    Vultr(VultrClient),
    DigitalOcean(DigitalOceanClient),
    Lnvps(LnvpsClient),
}

impl CloudClient {
    pub fn from_env() -> Result<Self> {
        let provider = CloudProvider::from_env()?;
        match provider {
            CloudProvider::Vultr => Ok(Self::Vultr(VultrClient::new()?)),
            CloudProvider::DigitalOcean => Ok(Self::DigitalOcean(DigitalOceanClient::new()?)),
            CloudProvider::Lnvps => Ok(Self::Lnvps(LnvpsClient::new()?)),
        }
    }

    pub async fn list_regions(&self) -> Result<Vec<Region>> {
        match self {
            Self::Vultr(c) => c.list_regions().await,
            Self::DigitalOcean(c) => c.list_regions().await,
            Self::Lnvps(c) => c.list_regions().await,
        }
    }

    pub async fn list_plans(&self) -> Result<Vec<ServerPlan>> {
        match self {
            Self::Vultr(c) => c.list_plans().await,
            Self::DigitalOcean(c) => c.list_plans().await,
            Self::Lnvps(c) => c.list_plans().await,
        }
    }

    pub async fn list_os(&self) -> Result<Vec<OsImage>> {
        match self {
            Self::Vultr(c) => c.list_os().await,
            Self::DigitalOcean(c) => c.list_os().await,
            Self::Lnvps(c) => c.list_os().await,
        }
    }

    pub async fn create_server(&self, req: &CreateServerRequest) -> Result<Server> {
        match self {
            Self::Vultr(c) => c.create_server(req).await,
            Self::DigitalOcean(c) => c.create_server(req).await,
            Self::Lnvps(c) => c.create_server(req).await,
        }
    }

    pub async fn list_servers(&self) -> Result<Vec<Server>> {
        match self {
            Self::Vultr(c) => c.list_servers().await,
            Self::DigitalOcean(c) => c.list_servers().await,
            Self::Lnvps(c) => c.list_servers().await,
        }
    }

    pub async fn destroy_server(&self, server_id: &str) -> Result<()> {
        match self {
            Self::Vultr(c) => c.destroy_server(server_id).await,
            Self::DigitalOcean(c) => c.destroy_server(server_id).await,
            Self::Lnvps(c) => c.destroy_server(server_id).await,
        }
    }

    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::Vultr(_) => "Vultr",
            Self::DigitalOcean(_) => "DigitalOcean",
            Self::Lnvps(_) => "LNVPS",
        }
    }

    /// 检查 Lightning Invoice 状态 (仅 LNVPS)
    pub async fn check_lightning_invoice(&self, vm_id: &str) -> Result<String> {
        match self {
            Self::Lnvps(c) => c.check_invoice(vm_id).await,
            _ => Err(anyhow!("Lightning invoice only available for LNVPS")),
        }
    }
}

/// 推荐的节点配置
pub fn recommend_plan_for_node(node_type: &str) -> (&'static str, u32, u32) {
    // 返回 (描述, 最小vCPU, 最小RAM MB)
    match node_type {
        "validator" => ("Validator 节点需要高性能", 4, 16384),
        "archive" => ("归档节点需要大存储", 4, 32768),
        "full" => ("全节点", 2, 8192),
        _ => ("轻节点", 1, 2048),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_manager() {
        let mut budget = BudgetManager {
            monthly_limit: 100.0,
            current_spend: 50.0,
        };
        
        assert!(budget.can_afford(40.0));
        assert!(!budget.can_afford(60.0));
        assert_eq!(budget.remaining(), 50.0);
    }

    #[test]
    fn test_recommend_plan() {
        let (desc, vcpu, ram) = recommend_plan_for_node("validator");
        assert!(vcpu >= 4);
        assert!(ram >= 16384);
    }
}
