use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::warn;

/// 滑动窗口限流器
///
/// 防止:
/// 1. Webhook flood 攻击
/// 2. 恶意节点频繁请求 /v1/execute
pub struct RateLimiter {
    /// 窗口内请求数
    count: AtomicU64,
    /// 窗口开始时间
    window_start: Mutex<Instant>,
    /// 窗口大小（秒）
    window_secs: u64,
    /// 窗口内最大请求数
    max_requests: u64,
}

impl RateLimiter {
    pub fn new(window_secs: u64, max_requests: u64) -> Self {
        Self {
            count: AtomicU64::new(0),
            window_start: Mutex::new(Instant::now()),
            window_secs,
            max_requests,
        }
    }

    /// 检查是否允许请求
    ///
    /// 返回 true 如果允许，false 如果限流
    pub async fn check(&self) -> bool {
        let now = Instant::now();
        let mut start = self.window_start.lock().await;

        // 窗口已过期 → 重置
        if now.duration_since(*start).as_secs() >= self.window_secs {
            *start = now;
            self.count.store(1, Ordering::SeqCst);
            return true;
        }

        let current = self.count.fetch_add(1, Ordering::SeqCst) + 1;
        if current > self.max_requests {
            warn!(
                current,
                max = self.max_requests,
                window_secs = self.window_secs,
                "限流: 请求过多"
            );
            false
        } else {
            true
        }
    }

    /// 获取当前窗口内的请求数
    pub fn current_count(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(60, 10);

        for _ in 0..10 {
            assert!(limiter.check().await);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(60, 3);

        assert!(limiter.check().await);
        assert!(limiter.check().await);
        assert!(limiter.check().await);
        // 第 4 次超限
        assert!(!limiter.check().await);
    }
}
