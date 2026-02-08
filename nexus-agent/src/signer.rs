use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Signature};
use sha2::{Sha256, Digest};
use std::path::Path;
use tracing::{info, warn};

/// Ed25519 密钥管理器
///
/// - 首次启动: 生成密钥对 → 保存到文件
/// - 后续启动: 从文件加载
/// - 提供签名功能
pub struct KeyManager {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl KeyManager {
    /// 加载或生成密钥对
    pub fn load_or_generate(data_dir: &str) -> anyhow::Result<Self> {
        let key_path = Path::new(data_dir).join("agent.key");

        if key_path.exists() {
            // 从文件加载
            let key_bytes = std::fs::read(&key_path)?;
            if key_bytes.len() != 32 {
                anyhow::bail!("密钥文件损坏: 期望 32 字节, 实际 {} 字节", key_bytes.len());
            }
            let mut secret = [0u8; 32];
            secret.copy_from_slice(&key_bytes);
            let signing_key = SigningKey::from_bytes(&secret);
            let verifying_key = signing_key.verifying_key();

            info!(
                public_key = hex::encode(verifying_key.as_bytes()),
                "已加载 Ed25519 密钥对"
            );

            Ok(Self { signing_key, verifying_key })
        } else {
            // 生成新密钥对
            let mut csprng = rand::rngs::OsRng;
            let signing_key = SigningKey::generate(&mut csprng);
            let verifying_key = signing_key.verifying_key();

            // 确保目录存在
            if let Some(parent) = key_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // 保存私钥（仅 32 字节种子）
            std::fs::write(&key_path, signing_key.as_bytes())?;

            // 设置文件权限为 600（仅所有者可读写）
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
            }

            info!(
                public_key = hex::encode(verifying_key.as_bytes()),
                "已生成新 Ed25519 密钥对"
            );
            warn!("⚠️  请将以上公钥注册到链上 (pallet-bot-registry::register_bot)");

            Ok(Self { signing_key, verifying_key })
        }
    }

    /// 获取公钥字节
    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.verifying_key.as_bytes()
    }

    /// 获取公钥 hex 字符串
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.as_bytes())
    }

    /// Ed25519 签名
    ///
    /// 签名内容: owner_address + bot_id_hash + sequence + timestamp + message_hash
    pub fn sign(&self, data: &[u8]) -> [u8; 64] {
        let signature: Signature = self.signing_key.sign(data);
        signature.to_bytes()
    }

    /// 对原始数据签名（返回 64 字节签名）
    pub fn sign_raw(&self, data: &[u8]) -> [u8; 64] {
        self.sign(data)
    }

    /// 构造签名数据并签名
    ///
    /// 返回 (signature_bytes, message_hash)
    pub fn sign_message(
        &self,
        bot_id_hash: &[u8; 32],
        sequence: u64,
        timestamp: u64,
        telegram_update_json: &[u8],
    ) -> ([u8; 64], [u8; 32]) {
        // message_hash = SHA256(telegram_update_json)
        let mut hasher = Sha256::new();
        hasher.update(telegram_update_json);
        let hash_result = hasher.finalize();
        let mut message_hash = [0u8; 32];
        message_hash.copy_from_slice(&hash_result);

        // 签名内容 = public_key + bot_id_hash + sequence(LE) + timestamp(LE) + message_hash
        let mut sign_data = Vec::with_capacity(32 + 32 + 8 + 8 + 32);
        sign_data.extend_from_slice(&self.public_key_bytes());
        sign_data.extend_from_slice(bot_id_hash);
        sign_data.extend_from_slice(&sequence.to_le_bytes());
        sign_data.extend_from_slice(&timestamp.to_le_bytes());
        sign_data.extend_from_slice(&message_hash);

        let signature = self.sign(&sign_data);

        (signature, message_hash)
    }
}

/// 序列号管理器（原子递增 + 持久化）
pub struct SequenceManager {
    current: std::sync::atomic::AtomicU64,
    file_path: std::path::PathBuf,
}

impl SequenceManager {
    /// 加载或初始化序列号
    pub fn load_or_init(data_dir: &str) -> anyhow::Result<Self> {
        let file_path = Path::new(data_dir).join("sequence.dat");

        let current = if file_path.exists() {
            let data = std::fs::read(&file_path)?;
            if data.len() >= 8 {
                u64::from_le_bytes(data[..8].try_into().unwrap())
            } else {
                0
            }
        } else {
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            0
        };

        info!(sequence = current, "序列号已加载");

        Ok(Self {
            current: std::sync::atomic::AtomicU64::new(current),
            file_path: file_path.to_path_buf(),
        })
    }

    /// 获取下一个序列号（原子递增 + 持久化）
    pub fn next(&self) -> anyhow::Result<u64> {
        let seq = self.current.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

        // 持久化到文件
        atomicwrites::AtomicFile::new(
            &self.file_path,
            atomicwrites::OverwriteBehavior::AllowOverwrite,
        )
        .write(|f| {
            use std::io::Write;
            f.write_all(&seq.to_le_bytes())
        })?;

        Ok(seq)
    }

    /// 获取当前序列号（不递增）
    pub fn current(&self) -> u64 {
        self.current.load(std::sync::atomic::Ordering::SeqCst)
    }
}
