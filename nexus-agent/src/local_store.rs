use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Agent 本地状态存储
///
/// 用于高频、低延迟的检测操作（防刷屏、警告计数等）。
/// 不需要共识，但会异步提交审计记录到 Node 网络。
///
/// 参考:
///   - FallenRobot: CHAT_FLOOD 内存字典 + user_data 内存字典
///   - tg-spam: duplicateDetector 内存窗口
pub struct LocalStore {
    /// 防刷屏计数器: (chat_id, user_id) → FloodCounter
    flood_counters: RwLock<HashMap<(i64, i64), FloodCounter>>,

    /// 警告计数: (chat_id, user_id) → warn_count
    warn_counts: RwLock<HashMap<(i64, i64), u8>>,

    /// 管理员缓存: chat_id → AdminCacheEntry
    admin_cache: RwLock<HashMap<i64, AdminCacheEntry>>,

    /// 最近消息指纹: chat_id → Vec<MessageFingerprint>
    /// 用于重复消息检测
    recent_messages: RwLock<HashMap<i64, Vec<MessageFingerprint>>>,
}

/// 防刷屏计数器
struct FloodCounter {
    count: u16,
    window_start: Instant,
}

/// 管理员缓存条目
struct AdminCacheEntry {
    admin_ids: Vec<i64>,
    cached_at: Instant,
    ttl_seconds: u64,
}

/// 消息指纹（用于重复检测）
struct MessageFingerprint {
    user_id: i64,
    text_hash: u64,
    timestamp: Instant,
}

impl LocalStore {
    pub fn new() -> Self {
        Self {
            flood_counters: RwLock::new(HashMap::new()),
            warn_counts: RwLock::new(HashMap::new()),
            admin_cache: RwLock::new(HashMap::new()),
            recent_messages: RwLock::new(HashMap::new()),
        }
    }

    // ═══════════════════════════════════════════════════════════
    // 防刷屏
    // ═══════════════════════════════════════════════════════════

    /// 检查并更新防刷屏计数
    ///
    /// 返回 true = 触发防刷屏（超过 limit）
    ///
    /// 参考: FallenRobot/modules/sql/antiflood_sql.py
    ///   - CHAT_FLOOD[chat_id] = {user_id: [count, limit]}
    ///   - 每条消息 count += 1，超 limit 触发
    pub fn check_flood(&self, chat_id: i64, user_id: i64, limit: u16, window_secs: u16) -> bool {
        if limit == 0 {
            return false;
        }

        let key = (chat_id, user_id);
        let mut counters = self.flood_counters.write().unwrap();
        let now = Instant::now();

        let counter = counters.entry(key).or_insert(FloodCounter {
            count: 0,
            window_start: now,
        });

        // 窗口过期 → 重置
        if now.duration_since(counter.window_start).as_secs() >= window_secs as u64 {
            counter.count = 1;
            counter.window_start = now;
            return false;
        }

        counter.count += 1;
        counter.count > limit
    }

    /// 重置某用户的防刷屏计数器
    pub fn reset_flood(&self, chat_id: i64, user_id: i64) {
        let mut counters = self.flood_counters.write().unwrap();
        counters.remove(&(chat_id, user_id));
    }

    // ═══════════════════════════════════════════════════════════
    // 警告系统
    // ═══════════════════════════════════════════════════════════

    /// 增加警告计数，返回新的计数值
    ///
    /// 参考: FallenRobot/modules/warns.py
    ///   - user_data[chat_id][user_id]["warnings"] += 1
    pub fn add_warn(&self, chat_id: i64, user_id: i64) -> u8 {
        let mut warns = self.warn_counts.write().unwrap();
        let count = warns.entry((chat_id, user_id)).or_insert(0);
        *count = count.saturating_add(1);
        *count
    }

    /// 减少警告计数，返回新的计数值
    pub fn remove_warn(&self, chat_id: i64, user_id: i64) -> u8 {
        let mut warns = self.warn_counts.write().unwrap();
        let count = warns.entry((chat_id, user_id)).or_insert(0);
        *count = count.saturating_sub(1);
        *count
    }

    /// 重置警告
    pub fn reset_warns(&self, chat_id: i64, user_id: i64) {
        let mut warns = self.warn_counts.write().unwrap();
        warns.remove(&(chat_id, user_id));
    }

    /// 获取警告数
    pub fn get_warns(&self, chat_id: i64, user_id: i64) -> u8 {
        let warns = self.warn_counts.read().unwrap();
        warns.get(&(chat_id, user_id)).copied().unwrap_or(0)
    }

    // ═══════════════════════════════════════════════════════════
    // 管理员缓存
    // ═══════════════════════════════════════════════════════════

    /// 检查是否为管理员（带 TTL 缓存）
    ///
    /// 返回 Some(true/false) 如果缓存有效，None 如果缓存过期或不存在
    ///
    /// 参考: FallenRobot/modules/helper_funcs/chat_status.py
    ///   - TTLCache(maxsize=512, ttl=300) 缓存管理员列表
    pub fn is_admin_cached(&self, chat_id: i64, user_id: i64) -> Option<bool> {
        let cache = self.admin_cache.read().unwrap();
        let entry = cache.get(&chat_id)?;

        // TTL 检查
        if entry.cached_at.elapsed().as_secs() >= entry.ttl_seconds {
            return None; // 过期
        }

        Some(entry.admin_ids.contains(&user_id))
    }

    /// 更新管理员缓存
    pub fn set_admin_cache(&self, chat_id: i64, admin_ids: Vec<i64>) {
        let mut cache = self.admin_cache.write().unwrap();
        cache.insert(chat_id, AdminCacheEntry {
            admin_ids,
            cached_at: Instant::now(),
            ttl_seconds: 300, // 5 分钟 TTL
        });
    }

    // ═══════════════════════════════════════════════════════════
    // 重复消息检测
    // ═══════════════════════════════════════════════════════════

    /// 记录消息指纹，返回在窗口内该用户发送相同内容的次数
    ///
    /// 参考: tg-spam/lib/tgspam/duplicate.go
    ///   - duplicateDetector: Window + Threshold
    ///   - 同一用户在窗口内发送相同内容 > Threshold → spam
    pub fn record_message(&self, chat_id: i64, user_id: i64, text: &str, window_secs: u64) -> u32 {
        let text_hash = Self::hash_text(text);
        let now = Instant::now();

        let mut messages = self.recent_messages.write().unwrap();
        let entries = messages.entry(chat_id).or_insert_with(Vec::new);

        // 清理过期条目
        entries.retain(|fp| now.duration_since(fp.timestamp).as_secs() < window_secs);

        // 计算重复次数
        let dup_count = entries.iter()
            .filter(|fp| fp.user_id == user_id && fp.text_hash == text_hash)
            .count() as u32;

        // 添加新条目
        entries.push(MessageFingerprint {
            user_id,
            text_hash,
            timestamp: now,
        });

        dup_count + 1 // 包含当前这条
    }

    /// 计算文本哈希
    fn hash_text(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.to_lowercase().hash(&mut hasher);
        hasher.finish()
    }

    // ═══════════════════════════════════════════════════════════
    // 维护
    // ═══════════════════════════════════════════════════════════

    /// 清理过期数据（定时任务调用）
    pub fn cleanup_expired(&self) {
        let now = Instant::now();

        // 清理防刷屏（超过 60 秒未更新的计数器）
        {
            let mut counters = self.flood_counters.write().unwrap();
            counters.retain(|_, v| now.duration_since(v.window_start).as_secs() < 60);
        }

        // 清理管理员缓存（过期条目）
        {
            let mut cache = self.admin_cache.write().unwrap();
            cache.retain(|_, v| v.cached_at.elapsed().as_secs() < v.ttl_seconds);
        }

        // 清理消息指纹（超过 5 分钟的）
        {
            let mut messages = self.recent_messages.write().unwrap();
            for entries in messages.values_mut() {
                entries.retain(|fp| now.duration_since(fp.timestamp).as_secs() < 300);
            }
            messages.retain(|_, v| !v.is_empty());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flood_within_limit() {
        let store = LocalStore::new();
        // limit=5, 发 5 条不触发
        for i in 0..5 {
            let triggered = store.check_flood(-100, 42, 5, 10);
            if i < 5 {
                assert!(!triggered, "should not trigger at count {}", i + 1);
            }
        }
    }

    #[test]
    fn test_flood_exceeds_limit() {
        let store = LocalStore::new();
        for _ in 0..5 {
            store.check_flood(-100, 42, 5, 10);
        }
        // 第 6 条触发
        assert!(store.check_flood(-100, 42, 5, 10));
    }

    #[test]
    fn test_flood_different_users() {
        let store = LocalStore::new();
        for _ in 0..5 {
            store.check_flood(-100, 1, 5, 10);
        }
        // user 2 不受 user 1 影响
        assert!(!store.check_flood(-100, 2, 5, 10));
    }

    #[test]
    fn test_flood_disabled() {
        let store = LocalStore::new();
        // limit=0 → 永远不触发
        assert!(!store.check_flood(-100, 42, 0, 10));
    }

    #[test]
    fn test_warn_add_and_get() {
        let store = LocalStore::new();
        assert_eq!(store.get_warns(-100, 42), 0);
        assert_eq!(store.add_warn(-100, 42), 1);
        assert_eq!(store.add_warn(-100, 42), 2);
        assert_eq!(store.get_warns(-100, 42), 2);
    }

    #[test]
    fn test_warn_remove() {
        let store = LocalStore::new();
        store.add_warn(-100, 42);
        store.add_warn(-100, 42);
        assert_eq!(store.remove_warn(-100, 42), 1);
        assert_eq!(store.remove_warn(-100, 42), 0);
        // 不会下溢
        assert_eq!(store.remove_warn(-100, 42), 0);
    }

    #[test]
    fn test_warn_reset() {
        let store = LocalStore::new();
        store.add_warn(-100, 42);
        store.add_warn(-100, 42);
        store.reset_warns(-100, 42);
        assert_eq!(store.get_warns(-100, 42), 0);
    }

    #[test]
    fn test_warn_different_chats() {
        let store = LocalStore::new();
        store.add_warn(-100, 42);
        store.add_warn(-200, 42);
        assert_eq!(store.get_warns(-100, 42), 1);
        assert_eq!(store.get_warns(-200, 42), 1);
    }

    #[test]
    fn test_admin_cache() {
        let store = LocalStore::new();
        assert!(store.is_admin_cached(-100, 1).is_none());

        store.set_admin_cache(-100, vec![1, 2, 3]);
        assert_eq!(store.is_admin_cached(-100, 1), Some(true));
        assert_eq!(store.is_admin_cached(-100, 4), Some(false));
        assert!(store.is_admin_cached(-200, 1).is_none());
    }

    #[test]
    fn test_duplicate_detection() {
        let store = LocalStore::new();
        assert_eq!(store.record_message(-100, 42, "spam msg", 60), 1);
        assert_eq!(store.record_message(-100, 42, "spam msg", 60), 2);
        assert_eq!(store.record_message(-100, 42, "spam msg", 60), 3);
        // 不同内容不计重复
        assert_eq!(store.record_message(-100, 42, "different msg", 60), 1);
        // 不同用户不计重复
        assert_eq!(store.record_message(-100, 99, "spam msg", 60), 1);
    }

    #[test]
    fn test_cleanup() {
        let store = LocalStore::new();
        store.add_warn(-100, 42);
        store.set_admin_cache(-100, vec![1]);
        store.record_message(-100, 42, "test", 60);
        // 不应 panic
        store.cleanup_expired();
    }
}
