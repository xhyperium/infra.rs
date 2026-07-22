# observex 公开 API

**角色**：Tracing instrumentation

## 公开消费面

`TracingInstrumentation` / 别名 `ObservexInstrumentation`，实现 `contracts::Instrumentation`。

## 最小用法

```rust
use contracts::Instrumentation;
use observex::TracingInstrumentation;

let i = TracingInstrumentation::new();
i.record_retry("op", 1);
```

## 2026-07-22 dual-bar surface

Public helpers added for STATUS 100% structure + declared-surface hardening; see crate root docs and ssot alignment. **Not** Production Ready / L5.
