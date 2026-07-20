# EVID-KERNEL-002-TEST-014 — Branch / Line coverage

| 字段 | 值 |
|------|-----|
| Residual | RES-TEST-014 |
| Date | 2026-07-14 |
| Spec | SPEC-KERNEL-002 §11.5 |

## Commands

```bash
# line (stable) — executed
cargo llvm-cov -p kernel --summary-only

# branch — requires nightly; stable fails with -Z coverage-options=branch
cargo llvm-cov -p kernel --branch --summary-only
# → error: the option `Z` is only accepted on the nightly compiler
```

## Results

| Metric | Value | Gate | Status |
|--------|-------|------|--------|
| Line cover | **98.82%** (423 lines, 5 missed) | ≥95% | **PASS** |
| Region cover | 98.07% | — | info |
| Function cover | 95.59% | — | info |
| Branch cover | **not measured** (stable / no nightly branch) | ≥90% | **OPEN / DEFER** |

Per-file lines: clock 97.44% · error 100% · lifecycle 98.62%.

## Verdict

- Line coverage **closes** the line half of §11.5 for this campaign.
- Branch ≥90% remains **OPEN** until `cargo +nightly llvm-cov -p kernel --branch` is run in CI or local nightly.
- **Do not** mark RES-TEST-014 CLOSED until branch is measured PASS or formal permanent defer by owner.
