# EVID — SPEC-KERNEL-002 PR CI 修复台账（agent team）

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Team | agent team（lint/docs/fmt · coverage · docs-branch rebase） |
| Head branch | `feat/kernel-002-e2-migrate-banned-apis` |
| Related PRs | #232 (docs) · #233 (E1) · #234 (E2 stack) · #235 (full path → main) |

## PR 状态（修复前 → 动作）

| PR | 主题 | 修复前失败 | 动作 |
|----|------|-----------|------|
| #232 | docs SSOT + pipeline | clippy `nonminimal_bool` @ `approval_auto.rs`（分支落后 main） | rebase → `origin/main`（吸收 #231 修复） |
| #233 | E1 ErrorKind | 已绿 | 无代码动作 |
| #234 / #235 | E2–E3/C/L（同 head） | fmt · clippy unused `ErrorKind` · docs-check · coverage · kafkax-real | 本 branch 合入修复 |

## 代码修复清单（#234/#235 head）

1. **fmt**：`clock.rs` / `lifecycle.rs` / `public_api.rs` / `testkit` rustfmt
2. **clippy unused_imports**：`ErrorKind` 仅测试使用 → 移入 `#[cfg(test)]`  
   crates: resiliencx, kafkax, ossx, clickhousex, natsx, binance, okx, postgresx, transportx
3. **rustdoc 链接**：opaque `XError` 后禁止 `XError::Variant` 链到枚举变体  
   → `XError::invalid` / `unavailable` / `transient`（decimalx, redisx, postgresx, taosx）
4. **xtask**：`approval_auto` nonminimal_bool 简化（与 main 一致）
5. **coverage**：kernel 行覆盖率 CI 门槛 95%；本机 `cargo llvm-cov -p kernel` → **99.00% lines**（19 unit + 3 public_api）

## 本地验证证据

```text
cargo fmt -- --check                          OK
cargo test -p kernel --all-targets            19 + 3 PASS
cargo clippy -p resiliencx -p kafkax -p kernel \
  -p decimalx -p xtask -p transportx -p redisx \
  -p postgresx --all-targets -- -D warnings   OK
RUSTDOCFLAGS='-D warnings' cargo doc \
  -p decimalx -p redisx -p postgresx -p kernel --no-deps
  # + taosx after link fix
cargo llvm-cov -p kernel --fail-under-lines 95
  TOTAL lines: 401 / 4 missed = 99.00%
```

## 对齐结论（不得越权宣称）

```text
PASS: CI 阻塞项本地已清；代码主路径 E1–E3/C/L 保持；registry incubating
NOT PASS: §18 全勾 / stable / loom / proptest / KERNEL-* archgate 全量（见 residual-open.txt）
NEXT: 推送后等 GitHub CI 复跑绿；G2 residual 另开
```

## Residual 不变（诚实）

仍 OPEN：RES-TEST-001..006/010..016、RES-GATE-001..008、RES-LC-004、RES-API-004/007 等。  
详见 `residual-open.txt` 与 `EVID-KERNEL-002-R10-verdict.md`。


## GitHub CI 终态（babysit 2026-07-14）

| PR | required checks | mergeStateStatus | reviewDecision | 说明 |
|----|-----------------|------------------|----------------|------|
| #232 | **全绿**（14 pass / 3 skip） | BLOCKED | REVIEW_REQUIRED | 仅等人审 |
| #233 | **全绿** | BEHIND | REVIEW_REQUIRED | 落后 main；建议 close 由 #235 覆盖 |
| #234 | **全绿** | CLEAN | (none) | base=E1；建议 close 由 #235 覆盖 |
| #235 | **全绿** | BLOCKED | REVIEW_REQUIRED | **主合入候选**；仅等人审 |

`fail_found=0`。CI 侧 merge-ready；组织/仓库若强制 review 则 `BLOCKED` 属预期。


## GitHub CI 终态（babysit re-confirm · head 7af048f1）

| PR | required checks | mergeStateStatus | reviewDecision | 说明 |
|----|-----------------|------------------|----------------|------|
| #232 | **全绿** | BLOCKED | REVIEW_REQUIRED | 仅人审 |
| #233 | **全绿** | BEHIND | REVIEW_REQUIRED | 建议 close superseded by #235 |
| #234 | **全绿** | CLEAN | (none) | 与 #235 同 head；建议 close after #235 |
| #235 | **全绿** | BLOCKED | REVIEW_REQUIRED | **主合入候选**；CI merge-ready |

Head: `7af048f1`（含 Codex P1/P2 修复 + review disposition）。  
`fail_found=0`。剩余阻塞：**人工 review**，非 CI。
