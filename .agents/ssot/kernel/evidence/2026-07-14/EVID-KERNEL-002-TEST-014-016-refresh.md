# EVID-KERNEL-002-TEST-014/015/016 — toolchain reconfirm (residual hygiene)

| 字段 | 值 |
|------|-----|
| Residuals | **RES-TEST-014**, **RES-TEST-015**, **RES-TEST-016** |
| Date | 2026-07-14（residual agent refresh） |
| Status | **全部保持 OPEN**（无新实测） |
| Prior | `EVID-KERNEL-002-TEST-014-branch.md` / `015-mutants.md` / `016-miri.md` |

## 再确认方法

静态检查（无 shell 安装/跑测）：

| 检查 | 结果 |
|------|------|
| `~/.cargo/bin/cargo-mutants` | **ABSENT**（bin 列表无） |
| `~/.cargo/bin/cargo-miri` | **PRESENT**（shim only） |
| `rust-toolchain.toml` | `channel = "stable"`；components = rustfmt, clippy |
| stable components 文件 | 无 `miri-preview` |
| nightly components 文件 | clippy/cargo/rust-docs/rust-std/rustfmt/rustc — **无 miri** |
| nightly `bin/` | 无 `miri` 二进制 |
| 1.94.1 components | cargo / rust-std / rustc only — **无 miri** |

## 分项

### RES-TEST-014 — branch coverage ≥90%

| 项 | 状态 |
|----|------|
| Line cover | 既有 **98.82% PASS**（`EVID-KERNEL-002-TEST-014-branch.md`） |
| Branch cover | **仍未测**；stable 上 `--branch` 需 `-Z` → nightly |
| 本会话新跑 | **无** |
| 结论 | **OPEN** |

### RES-TEST-015 — mutation score ≥90%

| 项 | 状态 |
|----|------|
| `cargo-mutants` | **ABSENT** |
| 本会话 `cargo mutants -p kernel` | **未跑** |
| 结论 | **OPEN / DEFER**（工具缺失；**不得 CLOSED**） |

### RES-TEST-016 — miri

| 项 | 状态 |
|----|------|
| miri 组件 stable/nightly | **ABSENT** |
| 本会话 `cargo miri test -p kernel` | **未跑** |
| 结论 | **OPEN / DEFER**（**不得 CLOSED**） |

## 诚实边界

- 本 refresh **仅**确认工具链与既有 evidence 仍成立。  
- **不**将 DEFER 计为 PASS。  
- **不**关闭 RES-TEST-014/015/016。  
- §18.3 相关勾选仍 OPEN。

## 关闭条件（未变）

| ID | 关闭条件 |
|----|----------|
| 014 | `cargo +nightly llvm-cov -p kernel --branch` 报告 branch ≥90% 并写入 evidence |
| 015 | 安装 cargo-mutants → 实跑 → score ≥90% 或存活=0/书面豁免 + mutants 报告 |
| 016 | `rustup +nightly component add miri` → `cargo +nightly miri test -p kernel` 全绿 |
