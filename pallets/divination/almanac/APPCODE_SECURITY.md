# AppCode 安全配置方案

## 问题背景

阿里云 API 的 AppCode 是敏感凭证，不应该硬编码在源代码中，避免：
- 代码泄漏导致 AppCode 泄露
- 不同环境（开发/测试/生产）使用不同 AppCode
- AppCode 轮换时需要重新编译代码

## 方案对比

| 方案 | 安全性 | 灵活性 | 实现难度 | 推荐指数 |
|-----|-------|-------|---------|---------|
| **方案一: 节点启动参数** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **方案二: 环境变量** | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **方案三: 配置文件** | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| **方案四: 链上加密存储** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **方案五: 混合方案(推荐)** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## 方案一: 节点启动参数

### 1.1 实现方案

通过命令行参数传入 AppCode，节点启动时解析并存储在内存中，供 OCW 使用。

### 1.2 代码实现

#### 步骤 1: 定义命令行参数结构

```rust
// node/src/cli.rs

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[command(flatten)]
    pub run: sc_cli::RunCmd,

    /// 黄历 API AppCode (阿里云)
    #[arg(long, env = "ALMANAC_APPCODE")]
    pub almanac_appcode: Option<String>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    // ... 其他子命令
}
```

#### 步骤 2: 在 Service 中存储 AppCode

```rust
// node/src/service.rs

use sc_service::{Configuration, TaskManager};
use std::sync::{Arc, RwLock};

/// 全局 AppCode 存储
pub struct OcwSecrets {
    pub almanac_appcode: Option<String>,
}

impl OcwSecrets {
    pub fn new() -> Self {
        Self {
            almanac_appcode: None,
        }
    }

    pub fn set_almanac_appcode(&mut self, appcode: String) {
        self.almanac_appcode = Some(appcode);
    }

    pub fn get_almanac_appcode(&self) -> Option<&str> {
        self.almanac_appcode.as_deref()
    }
}

// 全局实例（使用 RwLock 保证线程安全）
lazy_static::lazy_static! {
    pub static ref OCW_SECRETS: Arc<RwLock<OcwSecrets>> = Arc::new(RwLock::new(OcwSecrets::new()));
}

pub fn new_partial(
    config: &Configuration,
    almanac_appcode: Option<String>,
) -> Result<...> {
    // 设置 AppCode
    if let Some(appcode) = almanac_appcode {
        OCW_SECRETS.write().unwrap().set_almanac_appcode(appcode);
    }

    // ... 其他初始化代码
}
```

#### 步骤 3: 修改 main.rs 传递参数

```rust
// node/src/main.rs

fn main() -> sc_cli::Result<()> {
    let cli = Cli::parse();

    match &cli.subcommand {
        Some(subcommand) => {
            // 处理子命令
        }
        None => {
            // 启动节点
            let runner = cli.create_runner(&cli.run)?;

            runner.run_node_until_exit(|config| async move {
                service::new_full(config, cli.almanac_appcode).map_err(sc_cli::Error::Service)
            })
        }
    }
}
```

#### 步骤 4: 在 OCW 中读取 AppCode

```rust
// pallets/divination/almanac/src/offchain.rs

impl<T: Config> Pallet<T> {
    fn get_appcode() -> Option<Vec<u8>> {
        // 从 OCW 本地存储读取
        let key = b"almanac::appcode";
        sp_io::offchain::local_storage_get(
            sp_core::offchain::StorageKind::PERSISTENT,
            key,
        )
    }

    fn fetch_almanac_from_api(
        year: u32,
        month: u8,
        day: u8,
    ) -> Result<AlmanacInfo, &'static str> {
        // 获取 AppCode
        let appcode = Self::get_appcode()
            .ok_or("AppCode not configured")?;

        let appcode_str = sp_std::str::from_utf8(&appcode)
            .map_err(|_| "Invalid AppCode UTF-8")?;

        // 构造请求
        let url = "https://jmhlysjjr.market.alicloudapi.com/almanac/day";
        let body = format!("year={}&month={}&day={}", year, month, day);

        let request = http::Request::post(url, vec![body.as_bytes()])
            .add_header("Authorization", &format!("APPCODE {}", appcode_str))
            .add_header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .deadline(sp_io::offchain::timestamp().add(Duration::from_millis(10000)));

        // ... 发送请求和解析逻辑
    }
}
```

#### 步骤 5: 节点启动时写入 OCW 存储

```rust
// node/src/service.rs

pub fn new_full(
    config: Configuration,
    almanac_appcode: Option<String>,
) -> Result<TaskManager, ServiceError> {
    // ... 初始化代码

    // 将 AppCode 写入 OCW 本地存储
    if let Some(appcode) = almanac_appcode {
        let offchain_storage = network.offchain_storage_handle();
        let key = b"almanac::appcode";

        offchain_storage.set(
            sp_core::offchain::STORAGE_PREFIX,
            key,
            appcode.as_bytes(),
        );

        log::info!("✅ Almanac AppCode configured");
    }

    // ... 其他代码
}
```

### 1.3 使用方式

```bash
# 方式 1: 命令行参数
./target/release/solochain-template-node \
  --dev \
  --almanac-appcode "your_appcode_here"

# 方式 2: 环境变量 (clap 自动支持)
export ALMANAC_APPCODE="your_appcode_here"
./target/release/solochain-template-node --dev

# 方式 3: 配合 systemd
# /etc/systemd/system/stardust-node.service
[Service]
Environment="ALMANAC_APPCODE=your_appcode_here"
ExecStart=/usr/local/bin/stardust-node --chain=production
```

### 1.4 优点
✅ 不在源代码中暴露
✅ 支持环境变量和命令行参数
✅ 部署时灵活配置
✅ 重启节点即可更换

### 1.5 缺点
⚠️ 命令行参数可能在进程列表中可见 (ps aux)
⚠️ 需要修改节点代码

---

## 方案二: 纯环境变量方案

### 2.1 实现方案

直接在 OCW 中读取环境变量（使用 `sp_io::offchain::random_seed` 的替代方案）。

### 2.2 代码实现

#### 方案 2A: 启动时注入到 OCW 存储

```rust
// node/src/service.rs

pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    // 从环境变量读取
    if let Ok(appcode) = std::env::var("ALMANAC_APPCODE") {
        // 注入到 OCW 本地存储
        let storage_path = config.database.path().unwrap();
        let storage = sc_offchain::OffchainDb::new(
            offchain_storage::OffchainStorage::new(storage_path)
        );

        storage.local_storage_set(
            sp_core::offchain::StorageKind::PERSISTENT,
            b"almanac::appcode",
            appcode.as_bytes(),
        );

        log::info!("✅ Almanac AppCode loaded from environment");
    } else {
        log::warn!("⚠️ ALMANAC_APPCODE not set, OCW will not work");
    }

    // ... 其他初始化代码
}
```

#### 方案 2B: OCW 中直接读取（不推荐，Substrate OCW 不支持直接读取环境变量）

OCW 运行在隔离的 WASM 环境中，无法直接访问宿主机环境变量，必须通过主机函数注入。

### 2.3 使用方式

```bash
# 方式 1: 直接设置
export ALMANAC_APPCODE="your_appcode_here"
./target/release/solochain-template-node --dev

# 方式 2: .env 文件 (需要 dotenv 支持)
echo "ALMANAC_APPCODE=your_appcode_here" > .env
./target/release/solochain-template-node --dev

# 方式 3: systemd
[Service]
EnvironmentFile=/etc/stardust/secrets.env
ExecStart=/usr/local/bin/stardust-node
```

### 2.4 优点
✅ 最简单的方案
✅ 不会在进程列表中暴露
✅ 符合 12-Factor App 最佳实践

### 2.5 缺点
⚠️ 仍需节点代码支持
⚠️ 环境变量可能被其他进程读取

---

## 方案三: 配置文件方案

### 3.1 实现方案

创建独立的配置文件，节点启动时读取。

### 3.2 代码实现

```rust
// node/src/config.rs

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct OcwConfig {
    pub almanac_appcode: Option<String>,
}

impl OcwConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: OcwConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

// node/src/cli.rs

#[derive(Debug, Parser)]
pub struct Cli {
    /// OCW 配置文件路径
    #[arg(long, default_value = "/etc/stardust/ocw-secrets.toml")]
    pub ocw_config: String,

    // ... 其他参数
}
```

### 3.3 配置文件示例

```toml
# /etc/stardust/ocw-secrets.toml

# 黄历 API AppCode
almanac_appcode = "your_appcode_here"

# 其他 OCW 可能需要的密钥
# weather_api_key = "xxx"
```

### 3.4 文件权限设置

```bash
# 创建配置文件
sudo mkdir -p /etc/stardust
sudo touch /etc/stardust/ocw-secrets.toml
sudo chmod 600 /etc/stardust/ocw-secrets.toml  # 仅 owner 可读写
sudo chown stardust:stardust /etc/stardust/ocw-secrets.toml

# 编辑配置
sudo nano /etc/stardust/ocw-secrets.toml
```

### 3.5 优点
✅ 集中管理多个密钥
✅ 权限控制严格
✅ 便于运维管理

### 3.6 缺点
⚠️ 需要额外的文件管理
⚠️ 服务器被入侵时仍有风险

---

## 方案四: 链上加密存储方案

### 4.1 实现方案

通过 Sudo 将 AppCode **加密后**存储在链上，OCW 使用节点密钥解密。

### 4.2 代码实现

#### 步骤 1: 添加链上存储

```rust
// pallets/divination/almanac/src/lib.rs

#[pallet::storage]
#[pallet::getter(fn encrypted_appcode)]
/// 加密的 AppCode (使用 OCW 公钥加密)
pub type EncryptedAppCode<T: Config> = StorageValue<
    _,
    BoundedVec<u8, ConstU32<256>>,
    OptionQuery,
>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 设置加密的 AppCode (仅 Root)
    #[pallet::weight(10_000)]
    #[pallet::call_index(10)]
    pub fn set_encrypted_appcode(
        origin: OriginFor<T>,
        encrypted_data: Vec<u8>,
    ) -> DispatchResult {
        ensure_root(origin)?;

        let bounded_data: BoundedVec<u8, ConstU32<256>> = encrypted_data
            .try_into()
            .map_err(|_| Error::<T>::AppCodeTooLong)?;

        EncryptedAppCode::<T>::put(bounded_data);

        Self::deposit_event(Event::AppCodeUpdated);

        Ok(())
    }
}
```

#### 步骤 2: OCW 解密逻辑

```rust
// pallets/divination/almanac/src/offchain.rs

use sp_core::crypto::KeyTypeId;

// 定义 Almanac 专用密钥类型
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"alma");

pub mod crypto {
    use super::KEY_TYPE;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        MultiSignature, MultiSigner,
    };

    app_crypto!(sr25519, KEY_TYPE);

    pub struct AlmanacAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AlmanacAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

impl<T: Config> Pallet<T> {
    fn get_decrypted_appcode() -> Result<Vec<u8>, &'static str> {
        // 1. 从链上读取加密数据
        let encrypted = Self::encrypted_appcode()
            .ok_or("AppCode not configured")?;

        // 2. 获取 OCW 密钥
        let public_keys = crypto::Public::all();
        if public_keys.is_empty() {
            return Err("No OCW keys available");
        }

        // 3. 使用第一个密钥解密
        let key = &public_keys[0];

        // 使用 ECIES 或其他加密方案解密
        // 注意: Substrate 默认不提供高级加密原语,需要自己实现或使用库
        let decrypted = Self::decrypt_with_key(key, &encrypted)?;

        Ok(decrypted)
    }

    fn decrypt_with_key(
        key: &crypto::Public,
        encrypted: &[u8],
    ) -> Result<Vec<u8>, &'static str> {
        // 实现 ECIES 解密逻辑
        // 或使用简单的 XOR (不推荐生产环境)

        // 示例: 简单 XOR (仅演示,不安全!)
        let key_bytes = key.as_ref();
        let mut decrypted = Vec::new();

        for (i, &byte) in encrypted.iter().enumerate() {
            let key_byte = key_bytes[i % key_bytes.len()];
            decrypted.push(byte ^ key_byte);
        }

        Ok(decrypted)
    }
}
```

#### 步骤 3: 加密工具

```rust
// node/src/cli.rs

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// 加密 AppCode 工具
    EncryptAppCode {
        /// 要加密的 AppCode
        #[arg(long)]
        appcode: String,

        /// OCW 公钥
        #[arg(long)]
        public_key: String,
    },

    // ... 其他子命令
}

// node/src/command.rs

pub fn encrypt_appcode(appcode: String, public_key: String) -> sc_cli::Result<()> {
    use sp_core::crypto::Ss58Codec;

    // 解析公钥
    let pubkey = sp_core::sr25519::Public::from_ss58check(&public_key)
        .map_err(|_| "Invalid public key")?;

    // 加密 (使用简单 XOR,生产环境应使用 ECIES)
    let key_bytes = pubkey.as_ref();
    let mut encrypted = Vec::new();

    for (i, byte) in appcode.as_bytes().iter().enumerate() {
        let key_byte = key_bytes[i % key_bytes.len()];
        encrypted.push(byte ^ key_byte);
    }

    // 输出十六进制
    println!("Encrypted AppCode (hex): 0x{}", hex::encode(&encrypted));

    Ok(())
}
```

### 4.3 使用流程

```bash
# 1. 生成 OCW 密钥
./target/release/solochain-template-node key generate --scheme Sr25519 --key-type alma

# 输出:
# Secret seed: 0x1234...
# Public key: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKv3gB

# 2. 加密 AppCode
./target/release/solochain-template-node encrypt-appcode \
  --appcode "your_appcode_here" \
  --public-key "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKv3gB"

# 输出:
# Encrypted AppCode (hex): 0xabcdef1234567890...

# 3. 通过 Sudo 设置加密数据
# 在 Polkadot.js Apps 中:
# Developer -> Sudo -> almanac.setEncryptedAppcode(0xabcdef...)

# 4. 启动节点并注入密钥
./target/release/solochain-template-node key insert \
  --base-path /tmp/node \
  --chain dev \
  --scheme Sr25519 \
  --suri "0x1234..." \
  --key-type alma

./target/release/solochain-template-node --dev
```

### 4.4 优点
✅ 最安全的方案
✅ 密钥轮换通过链上治理
✅ 审计友好 (链上记录)
✅ 多节点共享密钥

### 4.5 缺点
⚠️ 实现复杂度高
⚠️ 需要额外的加密/解密工具
⚠️ 性能开销 (解密操作)

---

## 方案五: 混合方案 (推荐)

### 5.1 方案组合

结合多种方案的优点:

1. **开发环境**: 使用环境变量 (方便快速测试)
2. **测试环境**: 使用命令行参数 (灵活配置)
3. **生产环境**: 使用链上加密存储 (最高安全性)

### 5.2 实现逻辑

```rust
// pallets/divination/almanac/src/offchain.rs

impl<T: Config> Pallet<T> {
    /// 获取 AppCode (优先级: 链上加密 > OCW 存储 > 降级失败)
    fn get_appcode() -> Result<Vec<u8>, &'static str> {
        // 优先级 1: 链上加密存储 (生产环境)
        if let Some(encrypted) = Self::encrypted_appcode() {
            if let Ok(decrypted) = Self::get_decrypted_appcode() {
                log::debug!("🔐 Using encrypted on-chain AppCode");
                return Ok(decrypted);
            }
        }

        // 优先级 2: OCW 本地存储 (开发/测试环境)
        if let Some(appcode) = sp_io::offchain::local_storage_get(
            sp_core::offchain::StorageKind::PERSISTENT,
            b"almanac::appcode",
        ) {
            log::debug!("📁 Using local storage AppCode");
            return Ok(appcode);
        }

        // 失败
        log::error!("❌ AppCode not configured");
        Err("AppCode not configured")
    }
}
```

### 5.3 配置优先级

```
┌─────────────────────────────────────┐
│   1. 链上加密存储 (最高优先级)         │
│      - 生产环境使用                   │
│      - 通过 Sudo 设置                 │
│      - OCW 密钥解密                   │
└─────────────────────────────────────┘
              ↓ (如果不存在)
┌─────────────────────────────────────┐
│   2. OCW 本地存储                     │
│      - 节点启动时从环境变量/参数注入   │
│      - 开发/测试环境使用               │
└─────────────────────────────────────┘
              ↓ (如果不存在)
┌─────────────────────────────────────┐
│   3. 降级失败                         │
│      - OCW 跳过更新                   │
│      - 记录错误日志                   │
└─────────────────────────────────────┘
```

---

## 完整实现示例

### 步骤 1: 修改 node/src/cli.rs

```rust
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[command(flatten)]
    pub run: sc_cli::RunCmd,

    /// 黄历 API AppCode
    #[arg(long, env = "ALMANAC_APPCODE")]
    pub almanac_appcode: Option<String>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// 加密 AppCode
    EncryptAppCode {
        #[arg(long)]
        appcode: String,
        #[arg(long)]
        public_key: String,
    },

    // ... 其他子命令
    Key(sc_cli::KeySubcommand),
    ChainInfo(sc_cli::ChainInfoCmd),
}
```

### 步骤 2: 修改 node/src/service.rs

```rust
use sc_offchain::OffchainDb;
use sp_core::offchain::OffchainStorage;

pub fn new_full(
    config: Configuration,
    almanac_appcode: Option<String>,
) -> Result<TaskManager, ServiceError> {
    // ... 其他初始化代码

    // 将 AppCode 注入到 OCW 本地存储
    if let Some(appcode) = almanac_appcode {
        let db_path = config.database.path().expect("Database path required");
        let mut offchain_db = OffchainDb::new(
            offchain_storage::OffchainStorage::new(db_path.clone())
        );

        offchain_db.local_storage_set(
            sp_core::offchain::StorageKind::PERSISTENT,
            b"almanac::appcode",
            appcode.as_bytes(),
        );

        log::info!("✅ Almanac AppCode configured from CLI/env");
    }

    // ... 启动服务
}
```

### 步骤 3: 在 pallet 中实现获取逻辑

```rust
// pallets/divination/almanac/src/offchain.rs

impl<T: Config> Pallet<T> {
    fn get_appcode() -> Result<Vec<u8>, &'static str> {
        // 优先级 1: 链上加密存储
        if let Some(encrypted) = Self::encrypted_appcode() {
            if let Ok(decrypted) = Self::decrypt_appcode(&encrypted) {
                return Ok(decrypted);
            }
        }

        // 优先级 2: OCW 本地存储
        if let Some(appcode) = sp_io::offchain::local_storage_get(
            sp_core::offchain::StorageKind::PERSISTENT,
            b"almanac::appcode",
        ) {
            return Ok(appcode);
        }

        Err("AppCode not configured")
    }
}
```

---

## 安全最佳实践

### 1. 环境隔离

```bash
# 开发环境
export ALMANAC_APPCODE="dev_appcode"

# 测试环境
export ALMANAC_APPCODE="test_appcode"

# 生产环境 (使用链上加密)
# 不设置环境变量,通过 Sudo 链上配置
```

### 2. 权限控制

```bash
# OCW 存储目录权限
chmod 700 /var/lib/stardust/offchain
chown stardust:stardust /var/lib/stardust/offchain

# 配置文件权限
chmod 600 /etc/stardust/ocw-secrets.toml
chown stardust:stardust /etc/stardust/ocw-secrets.toml
```

### 3. 日志脱敏

```rust
// 避免在日志中打印完整 AppCode
log::info!("AppCode configured: {}***", &appcode[..4]);
```

### 4. 定期轮换

```bash
# 每 90 天轮换 AppCode
# 1. 在阿里云生成新的 AppCode
# 2. 加密新 AppCode
# 3. 通过 Sudo 更新链上存储
# 4. 删除旧的本地存储
```

---

## 推荐配置

### 开发环境

```bash
# .env
ALMANAC_APPCODE=your_dev_appcode

# 启动
./target/release/solochain-template-node --dev
```

### 生产环境

```bash
# 1. 生成 OCW 密钥
./target/release/solochain-template-node key generate --scheme Sr25519 --key-type alma

# 2. 加密 AppCode
./target/release/solochain-template-node encrypt-appcode \
  --appcode "prod_appcode" \
  --public-key "5Grw..."

# 3. 通过 Sudo 设置
# polkadot.js: almanac.setEncryptedAppcode(0xabc...)

# 4. 注入密钥并启动
./target/release/solochain-template-node key insert \
  --base-path /var/lib/stardust \
  --chain production \
  --scheme Sr25519 \
  --suri "secret_seed" \
  --key-type alma

systemctl start stardust-node
```

---

## 总结

**推荐方案**: **混合方案 (方案五)**

- ✅ **开发环境**: 环境变量 `ALMANAC_APPCODE`
- ✅ **测试环境**: 命令行参数 `--almanac-appcode`
- ✅ **生产环境**: 链上加密存储 + OCW 密钥解密

这样既保证了开发效率,又确保了生产环境的安全性。

**安全清单**:
- [ ] 不在源代码中硬编码
- [ ] 不在日志中打印完整密钥
- [ ] 文件权限设置为 600
- [ ] 定期轮换 AppCode
- [ ] 生产环境使用加密存储
- [ ] 配置监控告警 (密钥过期/失效)
