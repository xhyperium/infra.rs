//! 可配置重试策略（仅 Transient / DeadlineExceeded / Unavailable）。
//!
//! **非幂等写默认不重试**；[`RetryPolicy::for_idempotent_write`] 启用写路径重试。

use std::time::Duration;

use kernel::{ErrorKind, XError, XResult};
use tokio::time::sleep;

/// 重试策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// 最大尝试次数（含首次；≥1）。
    pub max_attempts: u32,
    /// 初始退避。
    pub initial_backoff: Duration,
    /// 最大退避。
    pub max_backoff: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 1,
            initial_backoff: Duration::from_millis(50),
            max_backoff: Duration::from_secs(2),
        }
    }
}

impl RetryPolicy {
    /// 读路径默认：最多 3 次。
    #[must_use]
    pub fn for_read() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(50),
            max_backoff: Duration::from_secs(1),
        }
    }

    /// 幂等写路径：最多 3 次（调用方须保证幂等键/时间戳唯一）。
    #[must_use]
    pub fn for_idempotent_write() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(2),
        }
    }

    /// 是否可重试。
    #[must_use]
    pub fn is_retryable(err: &XError) -> bool {
        matches!(
            err.kind(),
            ErrorKind::Transient | ErrorKind::Unavailable | ErrorKind::DeadlineExceeded
        )
    }

    /// 在策略下执行异步操作。
    pub async fn run<T, F, Fut>(&self, mut op: F) -> XResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = XResult<T>>,
    {
        let attempts = self.max_attempts.max(1);
        let mut backoff = self.initial_backoff;
        let mut last = XError::internal("retry 未执行");
        for i in 0..attempts {
            match op().await {
                Ok(v) => return Ok(v),
                Err(e) if Self::is_retryable(&e) && i + 1 < attempts => {
                    last = e;
                    sleep(backoff).await;
                    backoff = (backoff.saturating_mul(2)).min(self.max_backoff);
                }
                Err(e) => return Err(e),
            }
        }
        Err(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn retries_transient_then_ok() {
        let n = AtomicU32::new(0);
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(5),
        };
        let v = policy
            .run(|| async {
                let c = n.fetch_add(1, Ordering::SeqCst);
                if c < 2 { Err(XError::unavailable("temp")) } else { Ok(42) }
            })
            .await
            .expect("ok");
        assert_eq!(v, 42);
        assert_eq!(n.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn no_retry_on_invalid() {
        let n = AtomicU32::new(0);
        let policy = RetryPolicy::for_read();
        let err = policy
            .run(|| async {
                n.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(XError::invalid("bad"))
            })
            .await
            .expect_err("invalid");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(n.load(Ordering::SeqCst), 1);
    }
}
