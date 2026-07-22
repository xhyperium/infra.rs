//! 异步组件逆序排空（compose-root drain 所有权在 bootstrap）。
//!
//! 关闭「async drain DEFER」：组合根持有可注册的 drain hook，
//! 关停时按 **LIFO** 顺序执行，保证依赖方先于依赖目标关闭。

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

    /// 注册一个关停 hook（同步闭包；异步组件在闭包内 `block_on` 或已完成的 join）。
    pub fn register<F>(&self, name: impl Into<String>, hook: F) -> XResult<()>
    where
        F: FnOnce() -> XResult<()> + Send + 'static,
    {
        let mut g = self.hooks.lock().map_err(|_| XError::internal("async drain lock 中毒"))?;
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

    /// 按 LIFO 执行全部 hook，返回逐步结果。
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
                "drain 步骤失败: {} — {}",
                bad.name,
                bad.error.as_deref().unwrap_or("unknown")
            )));
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

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
        drain.register("fail", || Err(XError::unavailable("boom"))).unwrap();
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
        drain.register("x", || Err(XError::invalid("nope"))).unwrap();
        let err = drain.drain_strict().unwrap_err();
        assert!(err.context().contains("drain 步骤失败"));
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
        drain.register("after-poison", || Ok(())).unwrap();
        // 毒化 hooks 锁；drain 应 via into_inner 恢复
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = drain.hooks.lock().expect("lock");
            panic!("poison drain lock");
        }));
        let results = drain.drain();
        assert_eq!(results.len(), 1);
        assert!(results[0].ok);
        assert_eq!(results[0].name, "after-poison");
    }
}
