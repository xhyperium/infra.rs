//! Kafka 客户端生命周期与在途操作跟踪。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use kernel::{XError, XResult};
use tokio::sync::{mpsc, watch};

/// 可克隆的关闭状态；所有 broker I/O 与后台消费任务都必须持有操作守卫。
#[derive(Clone)]
pub(crate) struct Lifecycle {
    inner: Arc<LifecycleInner>,
}

struct LifecycleInner {
    closed: AtomicBool,
    active: AtomicUsize,
    shutdown: watch::Sender<bool>,
    activity: watch::Sender<u64>,
}

/// 在途操作守卫；销毁时通知等待关闭的调用方。
pub(crate) struct OperationGuard {
    inner: Arc<LifecycleInner>,
}

impl Lifecycle {
    pub(crate) fn new() -> Self {
        let (shutdown, _) = watch::channel(false);
        let (activity, _) = watch::channel(0);
        Self {
            inner: Arc::new(LifecycleInner {
                closed: AtomicBool::new(false),
                active: AtomicUsize::new(0),
                shutdown,
                activity,
            }),
        }
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }

    pub(crate) fn ensure_open(&self) -> XResult<()> {
        if self.is_closed() { Err(XError::cancelled("kafkax: pool 已关闭")) } else { Ok(()) }
    }

    /// 注册在途操作，并用二次关闭检查消除 close/start 竞态。
    pub(crate) fn start_operation(&self) -> XResult<OperationGuard> {
        self.ensure_open()?;
        self.inner.active.fetch_add(1, Ordering::AcqRel);
        let guard = OperationGuard { inner: Arc::clone(&self.inner) };
        if self.is_closed() {
            drop(guard);
            return Err(XError::cancelled("kafkax: pool 正在关闭"));
        }
        Ok(guard)
    }

    pub(crate) fn subscribe_shutdown(&self) -> watch::Receiver<bool> {
        self.inner.shutdown.subscribe()
    }

    /// 停止接收新操作、取消在途操作，并在 deadline 内等待守卫全部释放。
    pub(crate) async fn close(&self, deadline: Duration) -> XResult<()> {
        let mut activity = self.inner.activity.subscribe();
        self.inner.closed.store(true, Ordering::Release);
        self.inner.shutdown.send_replace(true);

        tokio::time::timeout(deadline, async {
            while self.inner.active.load(Ordering::Acquire) != 0 {
                activity.changed().await.map_err(|error| {
                    XError::internal("kafkax: 生命周期通知已关闭").with_source(error)
                })?;
            }
            Ok(())
        })
        .await
        .map_err(|error| {
            XError::deadline_exceeded("kafkax close 等待在途操作超时").with_source(error)
        })?
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        self.inner.active.fetch_sub(1, Ordering::AcqRel);
        self.inner.activity.send_modify(|generation| {
            *generation = generation.wrapping_add(1);
        });
    }
}

/// 等待关闭信号；发送端销毁同样视为关闭。
pub(crate) async fn wait_for_shutdown(shutdown: &mut watch::Receiver<bool>) {
    if *shutdown.borrow() {
        return;
    }
    while shutdown.changed().await.is_ok() {
        if *shutdown.borrow() {
            return;
        }
    }
}

/// 在有界队列背压与关闭信号之间竞争；关闭优先，避免后台任务永久阻塞。
pub(crate) async fn send_or_shutdown<T>(
    sender: &mpsc::Sender<T>,
    value: T,
    shutdown: &mut watch::Receiver<bool>,
) -> bool {
    tokio::select! {
        biased;
        () = wait_for_shutdown(shutdown) => false,
        result = sender.send(value) => result.is_ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::ErrorKind;

    #[tokio::test]
    async fn close_waits_for_active_operation_then_rejects_new_work() {
        let lifecycle = Lifecycle::new();
        let operation = lifecycle.start_operation().expect("注册在途操作");
        let closing = lifecycle.clone();
        let mut close_task =
            tokio::spawn(async move { closing.close(Duration::from_secs(1)).await });

        tokio::time::timeout(Duration::from_millis(20), &mut close_task)
            .await
            .expect_err("存在在途操作时 close 不得提前成功");
        drop(operation);
        close_task.await.expect("等待 close 任务").expect("关闭应在守卫释放后完成");

        let error = match lifecycle.start_operation() {
            Ok(_) => panic!("关闭后必须拒绝新操作"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ErrorKind::Cancelled);
    }

    #[tokio::test]
    async fn close_deadline_expires_but_pool_stays_closed() {
        let lifecycle = Lifecycle::new();
        let operation = lifecycle.start_operation().expect("注册在途操作");
        let error = lifecycle
            .close(Duration::from_millis(5))
            .await
            .expect_err("在途操作未释放时应触发 close deadline");
        assert_eq!(error.kind(), ErrorKind::DeadlineExceeded);
        assert!(lifecycle.is_closed());
        drop(operation);
    }

    #[tokio::test]
    async fn close_cancels_sender_blocked_by_bounded_backpressure() {
        let lifecycle = Lifecycle::new();
        let operation = lifecycle.start_operation().expect("注册后台任务");
        let mut shutdown = lifecycle.subscribe_shutdown();
        let (sender, mut receiver) = mpsc::channel(1);
        sender.send(1_u8).await.expect("填满有界队列");
        let send_task = tokio::spawn(async move {
            let _operation = operation;
            send_or_shutdown(&sender, 2_u8, &mut shutdown).await
        });

        lifecycle.close(Duration::from_secs(1)).await.expect("关闭应取消背压发送");
        assert!(!send_task.await.expect("等待背压任务"));
        assert_eq!(receiver.recv().await, Some(1));
        assert_eq!(receiver.recv().await, None);
    }
}
