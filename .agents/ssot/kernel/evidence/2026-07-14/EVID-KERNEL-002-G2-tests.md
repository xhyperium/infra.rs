# EVID-KERNEL-002-G2 — 测试 / 快照轨（非 §18 全闭合）

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Branch | `feat/kernel-002-e2-migrate-banned-apis` |
| PR | [#235](https://github.com/xhyperium/infra.rs/pull/235) |
| Scope | G2 子集：dev-deps · loom · §11 测试文件 · public-api 快照 |
| §18 | **仍 OPEN**（不得 stable） |

## 命令证据

### 默认 `cargo test -p kernel`

```text
lib: 18 passed
tests/api_compile: 1 passed
tests/clock_contract: 7 passed (proptest + ComponentState matrix + series)
tests/lifecycle_concurrency: 2 passed
tests/lifecycle_concurrency_loom: 0 (cfg(not loom) — expected)
tests/public_api: 3 passed
clippy -p kernel --all-targets -D warnings: ok
```

### loom

```bash
RUSTFLAGS="--cfg loom" cargo test -p kernel --test lifecycle_concurrency_loom --release
# loom_trigger_wakes_waiter ... ok
# loom_two_waiters ... ok
# 2 passed
```

### public API 快照

```bash
cargo public-api -p kernel --simplified > .architecture/api/kernel-public-api.txt
# 490 lines; no Component trait; no from_std / from_clock_elapsed on freeze face (doc hidden)
```

## Cargo 面

- `[features] default = []` 保持
- `proptest` / `static_assertions` → `[dev-dependencies]`
- `loom` → `[target.'cfg(loom)'.dependencies]`（非 feature，满足 §3.4）
- workspace.lints：`unexpected_cfgs` check-cfg `cfg(loom)`（crate 不可 override workspace lints）

## Residual 变更（本轨）

| ID | 前 → 后 |
|----|---------|
| RES-API-004 | OPEN → **CLOSED** |
| RES-LC-004 | OPEN → **CLOSED** |
| RES-TEST-001/002/003/006/010/011/013 | OPEN → **CLOSED** |
| RES-GATE-001 | OPEN → **CLOSED** |
| RES-TEST-012 / RES-GATE-008 | OPEN（loom 本地 PASS；**CI 未挂**） |
| RES-GATE-002…007 | OPEN（archgate 命名规则未实现） |

## 明确不宣称

- §18.1 Approved / §18.3 测试全勾 / §18.4 门禁全勾
- registry `stable`
- KERNEL-* archgate 机器全量
- trybuild UI / mutants / miri / branch≥90%
