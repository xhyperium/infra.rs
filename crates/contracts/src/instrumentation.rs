//! 可观测性契约（ADR-005 / SPEC-INFRA-OBSERVEX）。

#![deny(missing_docs)]

/// 可观测性注入点。
///
/// - `observex` 提供默认实现（`TracingInstrumentation`）
/// - `resiliencx` 等消费者只依赖本 trait，不依赖具体后端
///
/// 实现必须 `Send + Sync`，可放入 `Arc<dyn Instrumentation>`。
pub trait Instrumentation: Send + Sync {
    /// 记录一次重试（`op` 为受控操作名，`attempt` 为尝试序号，通常从 1 起）。
    fn record_retry(&self, op: &str, attempt: u32);

    /// 记录熔断打开。
    fn record_circuit_open(&self, op: &str);

    /// 记录熔断关闭。
    fn record_circuit_close(&self, op: &str);
}

#[cfg(test)]
mod tests {
    use super::Instrumentation;

    struct Mock;

    impl Instrumentation for Mock {
        fn record_retry(&self, _op: &str, _attempt: u32) {}
        fn record_circuit_open(&self, _op: &str) {}
        fn record_circuit_close(&self, _op: &str) {}
    }

    #[test]
    fn mock_implements_and_is_object_safe() {
        let m = Mock;
        let obj: &dyn Instrumentation = &m;
        obj.record_retry("op", 1);
        obj.record_circuit_open("op");
        obj.record_circuit_close("op");
    }
}
