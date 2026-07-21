# 四核心 crate 内部生产发布证据包

| 字段 | 值 |
|------|----|
| 日期 | 2026-07-21 |
| 范围 | `kernel` · `testkit` · `decimalx` · `canonical` |
| 性质 | **内部生产发布证据**（分层 GO）；**不是** crates.io 发布；**不是** workspace 整体 Production Ready |
| 关联 | PLAN-CORE-PROD-002 · 既有 L5 [`0.3.0-signoff.md`](./0.3.0-signoff.md) |
| 本文件状态 | **DRAFT · GO for declared tiers** · 新 L5 人工 `Signed-off-by` **未**代签（沿用 0.3.0 Maintainer 签核 + 本证据包） |

---

## 1. 声明层级（仅此范围）

| package | Cargo 名 | version | 声明层级 | 可安全使用 | 不可宣称 |
|---------|----------|---------|----------|------------|----------|
| kernel | `kernel` | `0.3.0` | **L1 + L4** | L0 错误 / 时间 / 关停 | 跨平台未声明假设；crates.io |
| testkit | `testkit` | `0.1.1` | **L1 ManualClock test-support** | 确定性测试时钟 | **生产 runtime**；integration harness |
| decimalx | `decimalx` | `0.1.0` | **L1 Internal Ready** | 受控入口 + `checked_*` 资金路径 | package stable / 跨版本 wire |
| canonical | `canonical` | `0.1.0` | **L2 committed wire subset** | v1–v1.3 清单内 DTO 跨进程/落盘 | 未列入清单类型的 wire；全 crate PR |

---

## 2. 红线（必须遵守）

```text
分层就绪 ≠ 五个 crate / workspace 整体 Production Ready
publish = false 保持；本包不批准 crates.io
testkit ≠ 生产 runtime
contracts / adapters / bootstrap 不在本 tranche
Accept 风险仍适用：见 defer-disposition.md / 0.3.0-signoff.md
```

---

## 3. 本 tranche 交付物

| 类别 | 内容 |
|------|------|
| API 覆盖 | 各 crate `tests/public_api_surface.rs` 驱动根 re-export 并断言返回值/错误 |
| 基准 | `benches/hot_path`（`harness = false`；`--quick`） |
| 示例 | 各 crate `examples/basic.rs` 可 `cargo run -p <pkg> --example basic` |
| 文档 | README 层级/硬限制；`docs/API.md` 全公开面；CHANGELOG 发布节 |
| 门禁 | fmt / clippy -D warnings / test --all-targets / --doc / public-api / cov-gate |

---

## 4. 命令证据（实现会话捕获）

实现会话将下列命令输出写入私有 scratch（文件名约定）：

| 命令 | 日志名 |
|------|--------|
| `cargo test -p kernel -p testkit -p decimalx -p canonical --all-targets` | `test-all-targets.log` |
| `cargo test -p kernel -p testkit -p decimalx -p canonical --doc` | `test-doc.log` |
| `cargo clippy -p kernel -p testkit -p decimalx -p canonical --all-targets -- -D warnings` | `clippy.log` |
| `cargo fmt --all -- --check` | `fmt.log` |
| `node scripts/quality-gates/check-public-api.mjs` | `public-api.log` |
| `node scripts/quality-gates/cov-gate-100.mjs`（或 `check.mjs`） | `cov.log` / `quality-gates.log` |
| `cargo bench -p <pkg> --bench hot_path -- --quick` | `bench-<pkg>.log` |
| `cargo run -p <pkg> --example basic` | `example-<pkg>.log` |
| `cargo test -p <pkg> --test public_api_surface` | `api-surface-<pkg>.log` |

**实现会话观察（2026-07-21 · branch `feat/core-prod-release-four-crates`）**：

| 门禁 | 结果 |
|------|------|
| `cargo test … --all-targets` | exit 0 |
| `cargo test … --doc` | exit 0（含 kernel compile_fail 12） |
| `cargo clippy … -D warnings` | exit 0 |
| `cargo fmt --check` | exit 0 |
| `check-public-api.mjs` | kernel/testkit/decimalx/canonical OK |
| `cov-gate-100` 四包 | kernel/testkit/decimalx/canonical 均为 100% line |
| benches `--quick` | 各含 `iters=` + `per_iter=` 非空 |
| examples `basic` | 均打印 `*-consumer: ok` |

> scratch 日志在实现会话私有目录（不入仓）；在仓可复现上述命令。

---

## 5. Accept 风险（继承，不关闭）

| 来源 | 项 | 本 tranche 处置 |
|------|-----|----------------|
| DEFER-6 | 非 Linux 矩阵 | Accept；官方仅 Linux x86_64 |
| DEFER-1 残留 | 真实云端后端 | 不在范围（contracts/adapters） |
| decimalx | wire stable / package stable | Accept；WIRE.md 已声明 |
| canonical | envelope / schema_version | Accept；无 envelope |
| 0.3.0-signoff | crates.io / 整体 PR 营销 | **禁止** 本包扩大宣称 |

完整表：[`../artifacts/defer-disposition.md`](../artifacts/defer-disposition.md)

---

## 6. 结论

| 问题 | 结论 |
|------|------|
| 四包是否达到**各自声明层级**的内部 GO？ | **是**（证据见 §3–§4 + 在仓测试/文档） |
| 是否整体 Production Ready？ | **否** |
| 是否 crates.io publish？ | **否**（`publish = false`） |
| 是否需要新 Maintainer Signed-off-by？ | 本文件为 DRAFT 证据；L5 权威仍以 [`0.3.0-signoff.md`](./0.3.0-signoff.md) 为准。若 Maintainer 要求独立四包签字，请在下方追加，**禁止 Agent 伪造**。 |

```text
# Maintainer（可选，人工填写）
# Signed-off-by: @<maintainer> <date>
# Verdict: GO for declared tiers only / REQUEST CHANGES
```

---

## 7. 引用

- 计划：[`../2026-07-21-core-crates-production-readiness.md`](../2026-07-21-core-crates-production-readiness.md)
- 审计：[`../../report/2026-07-21/core-crates-production-readiness.md`](../../report/2026-07-21/core-crates-production-readiness.md)
- 支持矩阵：[`../../governance/support-matrix.md`](../../governance/support-matrix.md)
- API baselines：`docs/api-baselines/{kernel,testkit,decimalx,canonical}.txt`
