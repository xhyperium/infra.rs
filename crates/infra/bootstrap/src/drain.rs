//! 组件逆序排空（compose-root drain 所有权在 bootstrap）。
//!
//! `AsyncDrain` 是保留的公开类型名；当前执行模型是同步 hook，不是异步 runtime。
//! 组合根持有可注册的 drain hook，关停时按单次快照内 **LIFO** 顺序执行，
//! 保证同批依赖方先于依赖目标关闭。

use std::sync::Mutex;

use kernel::{XError, XResult};

/// 单个排空步骤的结果摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrainStepResult {
    /// 步骤名（诊断用）。
    pub name: String,
    /// 是否成功。
    pub ok: bool,
    /// 失败时的错误上下文（成功为空）。
    pub error: Option<String>,
}

/// 关停排空编排器（组合根所有权）。
///
/// - `register` 后进先出（LIFO）；
/// - `drain` 消费全部 hook 并返回逐步结果；
/// - 某步失败不中断后续（记录错误，继续排空）。
///
/// # 并发语义
///
/// `register` 与 `drain` 的快照动作在同一 mutex 上线性化：快照前完成的注册进入
/// 本批，快照后的注册留给下一批。hook 在锁外执行，因此不同批次的 hook 可以并发，
/// 不提供跨批全局 LIFO 或全程串行保证。
///
/// # 阻塞与 panic 边界
///
/// hook 在调用线程同步执行；任一 hook 都可能永久阻塞。此类型不提供 deadline、
/// 取消或 panic 隔离，调用方必须在 hook 内自行建立这些边界。hook panic 会中断
/// 当前调用，未执行的快照项会随栈展开被丢弃。
#[derive(Default)]
pub struct AsyncDrain {
    hooks: Mutex<Vec<(String, Box<dyn FnOnce() -> XResult<()> + Send>)>>,
}

impl AsyncDrain {
    /// 空编排器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个关停 hook。
    ///
    /// hook 是同步闭包；如需等待异步组件，调用方负责在闭包内建立有界等待。
    /// 本 crate 不为 `block_on`、join、timeout 或取消提供安全保证。
    pub fn register<F>(&self, name: impl Into<String>, hook: F) -> XResult<()>
    where
        F: FnOnce() -> XResult<()> + Send + 'static,
    {
        let mut g = self.hooks.lock().map_err(|_| XError::internal("关停钩子锁中毒"))?;
        g.push((name.into(), Box::new(hook)));
        Ok(())
    }

    /// 当前已注册 hook 数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.hooks.lock().map(|g| g.len()).unwrap_or(0)
    }

    /// 是否无 hook。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 按本次快照内 LIFO 执行全部 hook，返回逐步结果。
    ///
    /// 返回值包含所有正常返回 `Ok`/`Err` 的 hook；单步 `Err` 不会提前终止。
    /// 阻塞或 panic 边界见类型级文档。
    pub fn drain(&self) -> Vec<DrainStepResult> {
        let mut hooks = match self.hooks.lock() {
            Ok(mut g) => std::mem::take(&mut *g),
            Err(poisoned) => std::mem::take(&mut *poisoned.into_inner()),
        };
        let mut results = Vec::with_capacity(hooks.len());
        while let Some((name, hook)) = hooks.pop() {
            match hook() {
                Ok(()) => results.push(DrainStepResult { name, ok: true, error: None }),
                Err(e) => results.push(DrainStepResult {
                    name,
                    ok: false,
                    error: Some(e.context().to_string()),
                }),
            }
        }
        results
    }

    /// 排空且任一步失败则返回聚合错误。
    pub fn drain_strict(&self) -> XResult<Vec<DrainStepResult>> {
        let results = self.drain();
        if let Some(bad) = results.iter().find(|r| !r.ok) {
            return Err(XError::unavailable(format!(
                "排空步骤失败：{}；原因：{}",
                bad.name,
                bad.error.as_deref().unwrap_or("未知错误")
            )));
        }
        Ok(results)
    }

    /// 仅供 crate 单元测试覆盖锁中毒失败路径。
    #[cfg(test)]
    pub(crate) fn poison_hooks_for_test(&self) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = self.hooks.lock().expect("测试必须先取得关停钩子锁");
            panic!("测试毒化关停钩子锁");
        }));
        assert!(result.is_err(), "测试必须实际毒化关停钩子锁");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex, mpsc};
    use std::time::Duration;

    fn successful_hook() -> XResult<()> {
        Ok(())
    }

    #[test]
    fn drain_runs_lifo_order() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let drain = AsyncDrain::new();
        for name in ["a", "b", "c"] {
            let order = Arc::clone(&order);
            let n = name.to_string();
            drain
                .register(name, move || {
                    order.lock().expect("lock").push(n);
                    Ok(())
                })
                .expect("reg");
        }
        assert_eq!(drain.len(), 3);
        let results = drain.drain();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.ok));
        // LIFO: c, b, a
        assert_eq!(*order.lock().expect("lock"), vec!["c", "b", "a"]);
        assert!(drain.is_empty());
    }

    #[test]
    fn drain_continues_after_failure() {
        let drain = AsyncDrain::new();
        drain.register("ok1", || Ok(())).unwrap();
        drain.register("fail", || Err(XError::unavailable("下层不可用"))).unwrap();
        drain.register("ok2", || Ok(())).unwrap();
        let results = drain.drain();
        // LIFO: ok2, fail, ok1
        assert_eq!(results[0].name, "ok2");
        assert!(results[0].ok);
        assert_eq!(results[1].name, "fail");
        assert!(!results[1].ok);
        assert_eq!(results[2].name, "ok1");
        assert!(results[2].ok);
    }

    #[test]
    fn drain_strict_errors_on_failure() {
        let drain = AsyncDrain::new();
        drain.register("x", || Err(XError::invalid("参数无效"))).unwrap();
        let err = drain.drain_strict().unwrap_err();
        assert_eq!(err.context(), "排空步骤失败：x；原因：参数无效");
    }

    #[test]
    fn empty_drain_ok() {
        let drain = AsyncDrain::new();
        assert!(drain.is_empty());
        assert!(drain.drain().is_empty());
        assert!(drain.drain_strict().unwrap().is_empty());
    }

    #[test]
    fn drain_recovers_from_poisoned_lock() {
        let drain = AsyncDrain::new();
        drain.register("after-poison", successful_hook).unwrap();
        drain.poison_hooks_for_test();
        let results = drain.drain();
        assert_eq!(results.len(), 1);
        assert!(results[0].ok);
        assert_eq!(results[0].name, "after-poison");
    }

    #[test]
    fn register_reports_internal_error_when_lock_is_poisoned() {
        let drain = AsyncDrain::new();
        drain.poison_hooks_for_test();
        let error = drain.register("rejected", successful_hook).expect_err("中毒锁必须拒绝注册");
        assert_eq!(error.kind(), kernel::ErrorKind::Internal);
        assert_eq!(error.context(), "关停钩子锁中毒");
    }

    #[test]
    fn register_after_snapshot_is_deferred_to_next_drain() {
        let drain = Arc::new(AsyncDrain::new());
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        drain
            .register("early", move || {
                started_tx.send(()).expect("报告 hook 已开始");
                release_rx.recv_timeout(Duration::from_secs(2)).expect("测试必须释放阻塞 hook");
                Ok(())
            })
            .expect("注册 early");

        let worker_drain = Arc::clone(&drain);
        let worker = std::thread::spawn(move || worker_drain.drain());
        started_rx.recv_timeout(Duration::from_secs(2)).expect("early 必须在锁外开始执行");

        drain.register("late", || Ok(())).expect("执行中仍可注册下一批");
        release_tx.send(()).expect("释放 early");

        let first = worker.join().expect("drain 线程不得 panic");
        assert_eq!(first.iter().map(|step| step.name.as_str()).collect::<Vec<_>>(), ["early"]);
        let second = drain.drain();
        assert_eq!(second.iter().map(|step| step.name.as_str()).collect::<Vec<_>>(), ["late"]);
    }
}
