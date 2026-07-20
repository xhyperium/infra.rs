# AGENTS.md — testkit

> 仓库级规则见 [`../../AGENTS.md`](../../AGENTS.md) 与 [`../../CONSTITUTION.md`](../../CONSTITUTION.md)。  
> 权威规范镜像：SPEC-TESTKIT-002 · [`.agents/ssot/testkit/spec/spec.md`](../../.agents/ssot/testkit/spec/spec.md)

## 身份

- **T0 test-support**（非生产 runtime；`publish = false`）
- package：`xhyper-testkit` · lib：`testkit` · path：`crates/testkit`
- 稳定公开面仅 `ManualClock` 族（4 类型）

## 本 crate 约束

- 业务 crate **只能**通过 `[dev-dependencies]` 引用
- 生产依赖白名单：**仅** `xhyper-kernel`（lib `kernel`）
- `default = []`；禁止 feature 泄漏
- 禁止真实时间 / sleep / 网络 / 文件 IO / 环境变量
- **禁止** 重新引入 `xlib_test!` / `mock!` / `FixtureBuilder` / provider 大宏
- 验证：`cargo test -p xhyper-testkit` · `cargo clippy -p xhyper-testkit --all-targets -- -D warnings`
- 质量：`cargo llvm-cov -p xhyper-testkit --fail-under-lines 95` · `cargo mutants -p xhyper-testkit` · `cargo +nightly miri test -p xhyper-testkit`
- 对齐矩阵：[`../../docs/testkit-ssot-alignment.md`](../../docs/testkit-ssot-alignment.md)

## 与 SSOT 镜像的关系

- `.agents/ssot/testkit` 是 **xhyper.rs 上游只读镜像**，其中 COMPLETE 叙事描述的是上游战役
- **本 crate 是 infra.rs 的落地实现**；以本仓 `cargo test` 与本仓 evidence 为准
- 禁止把上游 `evidence/testkit/2026-07-14-stable-gates/` 直接当作本仓 gate PASS

## 禁止占位

不得合并无行为 public placeholder（空 mock、零字段 builder、仅包装 `#[test]` 的宏）。
