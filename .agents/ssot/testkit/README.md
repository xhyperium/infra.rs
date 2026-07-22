# testkit — 本仓 SSOT 入口

| 项 | 当前事实 |
|---|---|
| 实现 | `crates/testkit` |
| package / lib / version | `testkit` / `testkit` / `0.1.3` |
| Active Spec | [spec/spec.md](spec/spec.md)（SPEC-TESTKIT-002） |
| 平面 | T0 / L1 deterministic test-support；不是生产 runtime |
| 分发 | `publish = false`；只允许 dev-dependency 消费 |

## 当前管线

| 层 | 入口 | 当前用途 |
|---|---|---|
| Spec | [spec/spec.md](spec/spec.md) | ManualClock 与 scenario runner 的唯一 current-state 合同 |
| Design | [design/design.md](design/design.md) | 消费型 builder 与 fail-closed seam |
| Test | [test/test.md](test/test.md) | 当前测试合同与待验证命令 |
| Gate | [gate/gate.md](gate/gate.md) | 当前候选门禁状态 |
| Matrix | [matrix/matrix.md](matrix/matrix.md) | clause → code → test → claim |
| Historical | `goal/` / `design/DESIGN-TESTKIT-002.md` / `plan/` / `review/` / `release/` / `evidence/` | 旧战役来源，不继承 PASS |

## 边界

- `ManualClock`：单 Mutex、checked 控制、fault/snapshot、独立 domain、无 Clone/Default。
- crate 内 `IntegrationHarness` 是确定性进程内 scenario runner，不是网络/进程/真实服务 harness。
- runner 用消费型 builder 消除运行后追加与重跑；错误/panic 返回 terminal report。
- `StepRecord` 不使用 epoch 0 哨兵，fault 状态必须显式可见。

## 验证

```bash
cargo test -p testkit --all-targets
cargo test -p testkit --doc
cargo clippy -p testkit --all-targets -- -D warnings
node scripts/quality-gates/check-public-api.mjs -p testkit
```
