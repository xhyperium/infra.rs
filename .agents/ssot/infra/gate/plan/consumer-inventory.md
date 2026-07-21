# Consumer Inventory — runtime `xhyper-gate` / `gate`

| 字段 | 值 |
|------|-----|
| Date | 2026-07-15 |
| Baseline | `main@41c59584` |
| Method | `cargo tree -i xhyper-gate` + targeted `rg` + `cargo metadata` |
| Evidence | `evidence/gate-retirement/plan-package-2026-07-15/` |

---

## 1. Package identity（命名陷阱）

| 项 | 值 |
|----|-----|
| package name | `xhyper-gate` |
| lib name | `gate`（`use gate::`） |
| path | `crates/gate` |
| version | 0.1.0 |

```text
# 正确
cargo tree -i xhyper-gate --workspace

# 错误（package ID 不匹配）
cargo tree -i gate   # → did not match any packages
```

同源 **非** 本 inventory 对象：

| 名称 | 路径 | 说明 |
|------|------|------|
| `xhyper-archgate` | `tools/archgate` | 架构门禁工具 — **保留** |
| `.agent/gates/` | harness | CI/agent 门禁规格 — **保留** |
| `VenueSafetyGate` | `crates/domain/exchange` | 领域风控结构 — **保留** |
| risk "pre-trade risk gate" | domain risk 描述 | 文案 — **保留** |

---

## 2. Reverse dependency（live）

```text
xhyper-gate v0.1.0 (/home/workspace/infra.rs/crates/gate)
└── xhyper-bootstrap v0.1.0 (/home/workspace/infra.rs/crates/bootstrap)
```

| Dependent | Kind | Notes |
|-----------|------|-------|
| `xhyper-bootstrap` | production path dep | `crates/bootstrap/Cargo.toml` → `xhyper-gate = { path = "../gate" }` |

**无**其他 workspace 生产 package 依赖 `xhyper-gate`。

---

## 3. Source usage（runtime patterns）

过滤模式：

```text
use gate::|gate::(Gate|Capability)|register_capability|Gate::(new|register|resolve|with_evidence)
```

### 3.1 Production / library code

| File | Usage |
|------|-------|
| `crates/bootstrap/src/lib.rs` | `use gate::{Capability, Gate}`；`Gate::new()`；`register_capability`；`AppContext` 持有 `gate`；`gate()` accessor |

### 3.2 Tests

| File | Usage |
|------|-------|
| `crates/bootstrap/src/lib.rs` `#[cfg(test)]` | DummyCap、register、resolve、len、is_empty |
| `crates/bootstrap/tests/e2e.rs` | E2ECap、register、ctx.gate().resolve |
| `crates/gate/src/lib.rs` tests | Gate 自测（随 crate 删除） |

### 3.3 未发现

```text
- 生产 service 经 gate.resolve 获取 Binance/Redis/OKX
- examples/ / benches/ 使用 gate
- 其他 adapter 依赖 xhyper-gate
```

e2e 中 MockBinance / MockKv **已经**直接面向 contracts trait——目标架构样板，应保留并强化（源 §9.2）。

---

## 4. Docs / registry / specs 面

| 面 | 路径示例 | 处理 |
|----|----------|------|
| architecture SSOT | `docs/architecture/spec.md` L0 列 gate | T-DEL-008 |
| ADR | ADR-010 / ADR-012 提及 gate | 保留历史；退役 ADR 更新 |
| agent 入口 | CLAUDE.md / AGENTS.md | 对齐诚实状态 |
| active spec | `.agents/ssot/infra/gate/gate-spec.md` | T-DEL-006 Superseded |
| source plan | `xhyper-gate-retirement-complete-plan.md` | 本包 SSOT 内容 |
| structural reports | docs/crates-structural-analysis… | 历史可保留 |

---

## 5. CI / harness（必须保留 · 非消费者）

| 路径 | 动作 |
|------|------|
| `.agent/gates/` | **KEEP** |
| `tools/archgate` | **KEEP** |
| CI jobs named gate / policy | **KEEP**（rename 非阻塞） |

这些 **不是** `xhyper-gate` package 的 dependents。

---

## 6. External downstream

| 检查 | 结果 |
|------|------|
| 本 monorepo 外 package 引用 | 本环境无可访问外部私有仓自动扫描 |
| 仓内 `package = "gate"` / `xhyper-gate` path | 仅 bootstrap |
| 结论（plan-time） | **仓内 only**；PR-5 前复核 T-INV-004；若有外部 → T-COMPAT-001 |

---

## 7. Migration surface 摘要

```text
Must migrate:
  bootstrap src + unit tests + e2e gate paths

Delete with crate:
  crates/gate entire tree

Do not touch as "consumers":
  VenueSafetyGate, archgate, .agent/gates, risk gate wording
```

---

## 8. Commands to re-run（实现波）

```bash
cargo tree -i xhyper-gate --workspace
cargo metadata --format-version 1 --no-deps | jq '.packages[] | select(.name|test("gate")) | {name, manifest_path}'
rg -n 'use gate::|gate::(Gate|Capability)|register_capability' --glob '*.rs' crates/bootstrap crates/gate
```
