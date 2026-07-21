> **Superseded（2026-07-22 / #178）**：独立 `contract-testkit` 已落地于 `crates/test-support/contracts`；本文「contract-testkit DEFER」仅作历史证据，以 `docs/ssot/testkit-ssot-alignment.md` 为准。

# testkit SSOT 对齐证据（infra.rs）

| 字段 | 值 |
|------|-----|
| date | 2026-07-21 |
| branch | `feat/testkit-ssot-align` |
| package | `xhyper-testkit` 0.1.1 |
| scope | core ManualClock 族；contract-testkit DEFER |

## 结论

**core 必选 GAP = 0**。详见 `docs/ssot/testkit-ssot-alignment.md` clause matrix。

## 本仓实测摘要

### cargo test -p xhyper-testkit

```text
unit (lib):                 17 passed  (含 mono_overflow_does_not_mutate)
api_compile:                 2 passed  (Send+Sync / !Default / !Clone 面)
manual_clock_concurrency:    2 passed
manual_clock_contract:       7 passed
manual_clock_properties:     5 passed  (含 mono_advance_checked / fault_set_clear_sequence)
public_surface:              4 passed
```

### cargo clippy / fmt

- `cargo clippy -p testkit --all-targets -- -D warnings` → PASS
- `cargo fmt --all --check` → PASS

### line coverage（§13.7）

```text
cargo llvm-cov -p testkit --fail-under-lines 95 --summary-only
Lines: 99.65%  (missed 1 / 283)
Functions: 100%
```

### mutants（§13.6）

```text
cargo mutants -p testkit --timeout 60
30 mutants tested: 10 caught, 20 unviable, missed=0
```

### Miri（§13.8）

```text
cargo +nightly miri test -p xhyper-testkit
all suites PASS（含 concurrency / property / public_surface）
```

## 同源命令日志

| 文件 | 内容 |
|------|------|
| `test.log` | cargo test |
| `clippy.log` | cargo clippy |
| `fmt.log` | cargo fmt --check |
| `cov.log` | cargo llvm-cov summary |
| `mutants.log` | cargo mutants |
| `miri.log` | cargo miri test |

## 禁止事项确认

- 未引用上游 `evidence/testkit/2026-07-14-stable-gates` 作为本仓 PASS
- 未修改 `.agents/ssot/testkit/**` 镜像
- contract-testkit / integration harness 显式 DEFER
