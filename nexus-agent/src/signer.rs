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
        platform_event_json: &[u8],
    ) -> ([u8; 64], [u8; 32]) {
        // message_hash = SHA256(platform_event_json)
        let mut hasher = Sha256::new();
        hasher.update(platform_event_json);
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
    /// 保护 fetch_add + 文件写入的原子性
    write_lock: std::sync::Mutex<()>,
    /// 审计日志独立序列号（内存态，不持久化，不消耗主序列号）
    audit_sequence: std::sync::atomic::AtomicU64,
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
            write_lock: std::sync::Mutex::new(()),
            audit_sequence: std::sync::atomic::AtomicU64::new(0),
        })
    }

    /// 获取下一个序列号（原子递增 + 持久化）
    ///
    /// Mutex 保证 fetch_add 和文件写入是原子操作，
    /// 防止并发调用导致文件写入乱序（旧值覆盖新值）。
    pub fn next(&self) -> anyhow::Result<u64> {
        let _guard = self.write_lock.lock()
            .unwrap_or_else(|e| e.into_inner());

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

    /// 获取下一个审计序列号（仅内存递增，不持久化）
    ///
    /// 审计日志使用独立序列号空间，不消耗主消息序列号。
    pub fn next_audit(&self) -> u64 {
        self.audit_sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Verifier;

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    // ═══════════════════════════════════════════
    // KeyManager 测试
    // ═══════════════════════════════════════════

    #[test]
    fn test_key_generate_and_reload() {
        let dir = temp_dir();
        let path = dir.path().to_str().unwrap();

        // 首次生成
        let km1 = KeyManager::load_or_generate(path).unwrap();
        let pk1 = km1.public_key_hex();
        assert_eq!(pk1.len(), 64); // 32 bytes hex

        // 二次加载
        let km2 = KeyManager::load_or_generate(path).unwrap();
        assert_eq!(km2.public_key_hex(), pk1, "重新加载公钥应一致");
    }

    #[test]
    fn test_key_corrupt_file() {
        let dir = temp_dir();
        let key_path = dir.path().join("agent.key");
        std::fs::write(&key_path, b"too_short").unwrap();

        let result = KeyManager::load_or_generate(dir.path().to_str().unwrap());
        assert!(result.is_err(), "损坏密钥文件应返回错误");
    }

    #[test]
    fn test_sign_and_verify() {
        let dir = temp_dir();
        let km = KeyManager::load_or_generate(dir.path().to_str().unwrap()).unwrap();

        let data = b"hello nexus";
        let sig_bytes = km.sign(data);

        // 用 ed25519 验证
        let sig = Signature::from_bytes(&sig_bytes);
        let vk = VerifyingKey::from_bytes(&km.public_key_bytes()).unwrap();
        assert!(vk.verify(data, &sig).is_ok(), "签名验证应通过");
    }

    #[test]
    fn test_sign_different_data_different_sig() {
        let dir = temp_dir();
        let km = KeyManager::load_or_generate(dir.path().to_str().unwrap()).unwrap();

        let sig1 = km.sign(b"data_a");
        let sig2 = km.sign(b"data_b");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_sign_message_deterministic() {
        let dir = temp_dir();
        let km = KeyManager::load_or_generate(dir.path().to_str().unwrap()).unwrap();

        let bot_id = [0xABu8; 32];
        let json = b"{\"update_id\":1}";

        let (sig1, hash1) = km.sign_message(&bot_id, 1, 1000, json);
        let (sig2, hash2) = km.sign_message(&bot_id, 1, 1000, json);

        assert_eq!(hash1, hash2, "相同输入的 message_hash 应相同");
        assert_eq!(sig1, sig2, "相同输入的签名应相同");
    }

    #[test]
    fn test_sign_message_different_sequence() {
        let dir = temp_dir();
        let km = KeyManager::load_or_generate(dir.path().to_str().unwrap()).unwrap();

        let bot_id = [0xABu8; 32];
        let json = b"{\"update_id\":1}";

        let (sig1, hash1) = km.sign_message(&bot_id, 1, 1000, json);
        let (sig2, hash2) = km.sign_message(&bot_id, 2, 1000, json);

        assert_eq!(hash1, hash2, "message_hash 不受 sequence 影响");
        assert_ne!(sig1, sig2, "不同 sequence 签名不同");
    }

    #[test]
    fn test_sign_message_verifiable() {
        let dir = temp_dir();
        let km = KeyManager::load_or_generate(dir.path().to_str().unwrap()).unwrap();

        let bot_id = [0xCDu8; 32];
        let json = b"test_payload";
        let seq = 42u64;
        let ts = 99999u64;

        let (sig_bytes, msg_hash) = km.sign_message(&bot_id, seq, ts, json);

        // 重建签名数据
        let mut sign_data = Vec::new();
        sign_data.extend_from_slice(&km.public_key_bytes());
        sign_data.extend_from_slice(&bot_id);
        sign_data.extend_from_slice(&seq.to_le_bytes());
        sign_data.extend_from_slice(&ts.to_le_bytes());
        sign_data.extend_from_slice(&msg_hash);

        let sig = Signature::from_bytes(&sig_bytes);
        let vk = VerifyingKey::from_bytes(&km.public_key_bytes()).unwrap();
        assert!(vk.verify(&sign_data, &sig).is_ok(), "sign_message 签名应可验证");
    }

    // ═══════════════════════════════════════════
    // SequenceManager 测试
    // ═══════════════════════════════════════════

    #[test]
    fn test_sequence_init_from_zero() {
        let dir = temp_dir();
        let sm = SequenceManager::load_or_init(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(sm.current(), 0);
    }

    #[test]
    fn test_sequence_increment() {
        let dir = temp_dir();
        let sm = SequenceManager::load_or_init(dir.path().to_str().unwrap()).unwrap();

        assert_eq!(sm.next().unwrap(), 1);
        assert_eq!(sm.next().unwrap(), 2);
        assert_eq!(sm.next().unwrap(), 3);
        assert_eq!(sm.current(), 3);
    }

    #[test]
    fn test_sequence_persistence() {
        let dir = temp_dir();
        let path = dir.path().to_str().unwrap();

        {
            let sm = SequenceManager::load_or_init(path).unwrap();
            sm.next().unwrap(); // 1
            sm.next().unwrap(); // 2
            sm.next().unwrap(); // 3
        }

        // 重新加载
        let sm2 = SequenceManager::load_or_init(path).unwrap();
        assert_eq!(sm2.current(), 3, "重启后应从持久化值恢复");
        assert_eq!(sm2.next().unwrap(), 4, "续接递增");
    }

    #[test]
    fn test_audit_sequence_independent() {
        let dir = temp_dir();
        let sm = SequenceManager::load_or_init(dir.path().to_str().unwrap()).unwrap();

        assert_eq!(sm.next().unwrap(), 1);
        assert_eq!(sm.next_audit(), 1);
        assert_eq!(sm.next_audit(), 2);
        assert_eq!(sm.next().unwrap(), 2); // 主序列号不受 audit 影响
        assert_eq!(sm.current(), 2);
    }

    #[test]
    fn test_audit_sequence_not_persisted() {
        let dir = temp_dir();
        let path = dir.path().to_str().unwrap();

        {
            let sm = SequenceManager::load_or_init(path).unwrap();
            sm.next_audit();
            sm.next_audit();
            sm.next_audit(); // audit=3
        }

        let sm2 = SequenceManager::load_or_init(path).unwrap();
        assert_eq!(sm2.next_audit(), 1, "审计序列号不持久化，重启归零");
    }

    #[test]
    fn test_sequence_concurrent_safety() {
        let dir = temp_dir();
        let sm = std::sync::Arc::new(
            SequenceManager::load_or_init(dir.path().to_str().unwrap()).unwrap()
        );

        let mut handles = vec![];
        for _ in 0..10 {
            let sm_clone = sm.clone();
            handles.push(std::thread::spawn(move || {
                let mut seqs = vec![];
                for _ in 0..10 {
                    seqs.push(sm_clone.next().unwrap());
                }
                seqs
            }));
        }

        let mut all_seqs: Vec<u64> = handles.into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect();
        all_seqs.sort();
        all_seqs.dedup();

        assert_eq!(all_seqs.len(), 100, "100 次并发递增应产生 100 个唯一序列号");
        assert_eq!(*all_seqs.first().unwrap(), 1);
        assert_eq!(*all_seqs.last().unwrap(), 100);
    }
}
