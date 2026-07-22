# testkit

T0 **deterministic test support**（SPEC-TESTKIT-002）。

| 项 | 值 |
|----|-----|
| package | `testkit` |
| lib | `testkit` |
| path | `crates/testkit` |
| version | `0.1.1` |
| publish | `false`（internal only） |
| **生产层级** | **L1 ManualClock test-support**（**不是**生产 runtime） |
| 支持矩阵 | Linux x86_64 · MSRV 1.85（随 kernel） |

> **仅** 作为业务 crate 的 `[dev-dependencies]` 使用。  
> **禁止** 把 ManualClock 当作生产时钟实现。

规范镜像：[`../../.agents/ssot/testkit/spec/spec.md`](../../.agents/ssot/testkit/spec/spec.md)  
对齐说明：[`../../docs/ssot/testkit-ssot-alignment.md`](../../docs/ssot/testkit-ssot-alignment.md)

## 公开面

```rust
pub use testkit::{
    ManualClock,
    ManualClockError,
    ManualClockFault,
    ManualClockSnapshot,
    IntegrationHarness,
    StepRecord,
};
```

- 墙钟 / 单调钟独立可控（checked，失败不改状态）
- wall fault 注入；一致 `snapshot`
- 无 `Default` / `Clone`；共享请用 `std::sync::Arc`
- `IntegrationHarness`：基于 ManualClock 的多步确定性测试 harness（非网络 / 非进程）

## 硬限制

- 不提供通用 mock 框架
- 不提供真实网络 / 进程级 integration harness（仅 ManualClock 步进 harness）
- contract trait suite 见独立 `contract-testkit`（`crates/test-support/contracts`）
- 生产依赖仅 `kernel`

## 最小用法

```bash
cargo run -p testkit --example basic
```

```rust
use kernel::{Clock, Timestamp};
use testkit::ManualClock;
use std::time::Duration;

let c = ManualClock::new(Timestamp::from_unix_nanos(0));
c.advance_wall(Duration::from_secs(1)).unwrap();
assert_eq!(c.now().unwrap().as_unix_nanos(), 1_000_000_000);
```

## 验证

```bash
cargo test -p testkit --all-targets
cargo test -p testkit --doc
cargo clippy -p testkit --all-targets -- -D warnings
cargo bench -p testkit --bench hot_path -- --quick
```

公开 API：[docs/API.md](./docs/API.md) · 变更日志：[CHANGELOG.md](./CHANGELOG.md)
