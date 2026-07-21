# EVID-KERNEL-002-G2 — archgate KERNEL-* + CI loom

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Branch | `feat/kernel-002-e2-migrate-banned-apis` |
| PR | [#235](https://github.com/xhyperium/infra.rs/pull/235) |
| Scope | G2 余项：命名 KERNEL-* 机器规则 + CI `kernel-loom` |
| §18 | **仍 OPEN**（Spec 未 Approved；禁止 stable） |

## 命令证据

### archgate

```bash
cargo run -p archgate -- --json
# exit 0
# kernel_rules: 13 rules all ok:true
# KERNEL-DEP-001/002, FEATURE-001, API-001, TIME-001/002/003,
# ERR-001/002, SERDE-001, ASYNC-001, UNSAFE-001, LIFECYCLE-001
# kernel_internal_count: 8 (baseline ≤8)
```

### loom（本地 + 将由 CI 执行）

```bash
RUSTFLAGS="--cfg loom" cargo test -p kernel --test lifecycle_concurrency_loom --release --locked
# 2 passed
```

### CI

- `.github/workflows/ci.yml` 新增 job **`kernel-loom`**
- `scripts/validate-workflows.sh` PASS

## 实现面

| 路径 | 作用 |
|------|------|
| `tools/archgate/src/kernel_rules.rs` | 命名规则实现 |
| `tools/archgate/src/main.rs` | 接入 + JSON `kernel_rules` |
| `.github/workflows/ci.yml` | `kernel-loom` job |

## Residual 变更

| ID | 前 → 后 |
|----|---------|
| RES-GATE-002…008 | OPEN → **CLOSED**（规则落地；TIME allowlist 含 FixedClock residual） |
| RES-TEST-012 | OPEN → **CLOSED**（CI loom 挂接） |
| RES-DOWN-005 | OPEN → **CLOSED**（CI loom） |
| RES-CLK-009 | 仍 OPEN（`from_std` / FixedClock Instant 路径） |
| RES-TEST-004/005/014–016 | 仍 OPEN（属性穷尽 / trybuild UI / branch / mutants / miri） |
| RES-API-007 | 仍 OPEN（0.1.1 bump） |
| RES-DOWN-006 | 仍 OPEN（外部 sleep 计时） |

> **HISTORICAL（G2 时点快照 · 非 live SSOT）**  
> Team-R10c 后：RES-CLK-009 / RES-TEST-005 / RES-DOWN-006 等已 CLOSED 或 DEFER。  
> **Live residual：** `residual-open.txt`。OPEN 仅余 RES-API-007 · RES-TEST-014/015/016（RES-18-APPROVED 已 CLOSED · Spec Approved）。

## 明确不宣称

- §18 全勾 / registry `stable` / Spec Approved
- FixedClock 已迁离 Instant::now（仍在 TIME-002 allowlist）
- trybuild / mutants / miri 完成
