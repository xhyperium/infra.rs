# testkit

T0 **deterministic test support**（SPEC-TESTKIT-002）。

| 项 | 值 |
|----|-----|
| package | `xhyper-testkit` |
| lib | `testkit` |
| path | `crates/testkit` |
| version | `0.1.1` |
| publish | `false`（internal only） |

> **不是**生产 runtime。与生产依赖图正交；业务 crate **只能** `[dev-dependencies]` 引用。

规范镜像：[`../../.agents/ssot/testkit/spec/spec.md`](../../.agents/ssot/testkit/spec/spec.md)  
对齐说明：[`../../docs/testkit-ssot-alignment.md`](../../docs/testkit-ssot-alignment.md)

## 公开面

```rust
pub use testkit::{
    ManualClock,
    ManualClockError,
    ManualClockFault,
    ManualClockSnapshot,
};
```

- 墙钟 / 单调钟独立可控（checked，失败不改状态）
- wall fault 注入；一致 `snapshot`
- 无 `Default` / `Clone`；共享请用 `std::sync::Arc`

## 依赖

- 生产依赖：仅 `xhyper-kernel`（path `../kernel`）
- 测试依赖：`proptest`（property tests）

## 验证

```bash
cargo test -p xhyper-testkit
cargo clippy -p xhyper-testkit --all-targets -- -D warnings
```

## 非职责

- 不提供通用 mock 框架
- 不提供 integration harness（真实网络 / 进程）
- contract trait suite 若需要，另开 `contract-testkit` 战役（本波未包含）
