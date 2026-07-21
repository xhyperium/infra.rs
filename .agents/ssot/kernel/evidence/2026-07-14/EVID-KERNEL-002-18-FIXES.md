> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# EVID-KERNEL-002-18-FIXES — 设计 PR Plan 全量可执行修复

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Branch | `feat/kernel-002-e2-migrate-banned-apis` |
| 依据 | DESIGN-KERNEL-002 PR Plan + Open Questions |

## 已执行

| Track | 项 | 结果 |
|-------|-----|------|
| **PR-1 Docs** | design.md Active 长文；AGENTS 删「Next: G2」；monotonic=`origin.elapsed()`；README PR OPEN | **DONE** |
| **PR-6 API** | `XError::from_static` + 9× `*_static`（Cow::Borrowed） | **DONE** additive |
| **OQ-7 / TIME-004** | archgate `KERNEL-TIME-004` allowlist `from_clock_elapsed` | **DONE** · 规则 **14** 条 |
| **PR-9** | `cargo public-api` 重生成 `.architecture/api/kernel-public-api.txt` | **DONE** |
| **RES-PERF-001** | 由 DEFER → **implemented additive** | **CLOSED** |
| **RES-GATE-010** | TIME-004 新 residual | **CLOSED** |

## 工具链结果（诚实）

| 项 | 结果 |
|----|------|
| line cov | **98.95%** PASS（`kernel-line-cov.txt`） |
| branch cov | **100%**（8/8）via `cargo +nightly llvm-cov -p kernel --branch`（`kernel-branch-cov-nightly.txt`） |
| miri | **21 passed** via `MIRIFLAGS=-Zmiri-disable-isolation cargo +nightly miri test -p kernel --lib`（`kernel-miri-lib.txt`） |
| cargo-mutants | **仍 ABSENT**（未装；RES-TEST-015 waiver DEFER 保持） |

## 明确未执行（依赖人/网络）

| 项 | 原因 |
|----|------|
| PR-0b merge #235 → main | 需审批 + push/merge 权限；本会话不自动 land main |
| PR-7 git tag | 设计禁止 land 前 tag |
| PR-5 KERNEL-API-002 | 仍 DEFER 至 RFC registry |
| crates.io publish | `publish=false` |

## 验证命令

```bash
cargo test -p kernel
cargo clippy -p kernel --all-targets -- -D warnings
cargo run -p archgate -- --json   # KERNEL-* 14 条含 TIME-004
```


## Post-ship update (2026-07-14)

- **PR-5 KERNEL-API-002**：已 implemented（#241）· 非 DEFER
- **crates.io**：`xhyper-kernel` 0.1.1 **published**
- **mutants**：measured missed=0（#240）· 非 ABSENT waiver
