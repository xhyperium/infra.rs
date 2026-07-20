# testkit SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 策略 | **B — 本仓移植 core testkit** |
| 日期 | 2026-07-21 |
| 分支 | `feat/testkit-port` |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游 SSOT 镜像 COMPLETE 叙事 | 仍是 xhyper 战役文档；**禁止**单独当作本仓交付证明 |
| 本仓 `crates/testkit` | **已落地**（package `xhyper-testkit` 0.1.1 · ManualClock V2） |
| 本仓 `contract-testkit` | **未**移植（scope 外） |
| 本仓 `evidence/testkit/` 上游 stable-gates | **不复制**；以本仓 `cargo test -p xhyper-testkit` 为准 |

## 本仓可观察事实（落地后）

```text
crates/testkit/                 EXISTS
Cargo.toml members              含 crates/testkit
package name                    xhyper-testkit
lib name                        testkit
publish                         false
deps                            xhyper-kernel only
```

验证：

```bash
cargo test -p xhyper-testkit
cargo clippy -p xhyper-testkit --all-targets -- -D warnings
```

## 与镜像文档的关系

- `.agents/ssot/testkit/**`：只读镜像；禁止本地改 CLOSED/COMPLETE 叙事冒充同步
- 实现 SSOT 以 **源码 + 本仓测试输出** 为准
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`「上游 SSOT 镜像与 testkit 落地」

## 未做（follow-up）

- `crates/test-support/contracts`（contract-testkit）
- mutants / Miri / line-cov CI job（可另开 quality 战役）
- 上游 SSOT 文档内部 STALE 收口（应在 xhyper.rs 修，再镜像同步）
