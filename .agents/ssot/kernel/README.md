# kernel — 本仓 SSOT 入口

| 项 | 当前事实 |
|---|---|
| 实现 | `crates/kernel` |
| package / lib / version | `kernel` / `kernel` / `0.3.1` |
| Active Spec | [spec/spec.md](spec/spec.md)（SPEC-KERNEL-002） |
| 声明边界 | L1 Internal Ready；L4 仅限最终 SHA 新鲜证据覆盖面 |
| 分发 | `publish = false`；不宣称 crates.io / production-certified |

## 当前管线

| 层 | 入口 | 当前用途 |
|---|---|---|
| Spec | [spec/spec.md](spec/spec.md) | 唯一 current-state 验收合同 |
| Design | [design/design.md](design/design.md) | seam、依赖与权衡 |
| Test | [test/test.md](test/test.md) | 当前测试合同与待验证命令 |
| Gate | [gate/gate.md](gate/gate.md) | 当前候选门禁状态 |
| Matrix | [matrix/matrix.md](matrix/matrix.md) | clause → code → test → claim |
| Evidence | [evidence/](evidence/) | 历史不可变来源，不继承 PASS |

## 硬边界

- 仅 `error` / `clock` / `lifecycle`，默认 feature 为空，生产依赖仅 `thiserror`。
- `ClockDomain` 与进程共享单调 origin 是当前批准语义；跨 domain 比较返回 `None`。
- `wait_timeout` 的不可表示 deadline 返回 typed error，不得伪装普通超时。
- 同目录 `spec/xhyper-kernel-complete-spec.md` 是 active spec 的机械镜像，必须逐字同构；dated campaign、crates.io 记录与旧 evidence 才是历史来源，不能证明当前提交。

## 验证

```bash
cargo test -p kernel --all-targets
cargo test -p kernel --doc
cargo clippy -p kernel --all-targets -- -D warnings
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
node scripts/quality-gates/check-public-api.mjs -p kernel
```
