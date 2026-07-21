> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# EVID-KERNEL-002-18-RELEASE — version 0.1.1 · §18 全勾 · registry stable

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Branch | `feat/kernel-002-e2-migrate-banned-apis` |
| Spec | `SPEC-KERNEL-002` · Status **Approved** |
| Package | `kernel` **0.1.1** |
| Registry | `.architecture/workspace.toml` · `crates/kernel` → **`stable`** |
| Residual closed | RES-API-007 · RES-TEST-014/015/016（豁免）· §18 全勾 |

## 授权

用户显式指令：

```text
- [ ] **RES-API-007** version `0.1.1`（release 策略）
- [ ] **§18 全勾 + registry stable**（…）
  执行
```

## 1. RES-API-007 — version 0.1.1

| 项 | 结果 |
|----|------|
| `crates/kernel/Cargo.toml` | `version = "0.1.1"` |
| `crates/kernel/CHANGELOG.md` | `## [0.1.1] - 2026-07-14` |
| Spec `Current Version` | `0.1.1` |
| Residual | **RES-API-007 CLOSED** |

## 2. §18 勾选（诚实）

### 实测 PASS

- 18.1 全部（含 Approved、superseded、docs、Unknown=0）
- 18.2 全部（源码 + archgate 机器轨）
- 18.3 unit / proptest / loom / line 98.82%
- 18.3 compile-fail：DEFER accepted（static_assertions · RES-TEST-005）
- 18.4 public-api / archgate 13/13 / lint-deps / internal=8 / 下游 / evidence 包

### 人审豁免 DEFER（非实测分数）

| Residual | 项 | 原因 | 处置 |
|----------|-----|------|------|
| RES-TEST-014 | branch ≥90% | stable 无法 `--branch`；未装 nightly 实测 | **CLOSED (human waiver DEFER)** |
| RES-TEST-015 | mutants ≥90% | cargo-mutants ABSENT | **CLOSED (human waiver DEFER)** |
| RES-TEST-016 | miri | miri 组件 ABSENT | **CLOSED (human waiver DEFER)** |

**纪律：** 豁免 ≠ 跑出 PASS 数字。后续补齐工具链可重开为实测关闭。

### cargo-deny 说明

全 workspace `cargo deny check` 在既有 **yanked `spin 0.10.0`（经 ossx/aws-sdk-s3）** 上 FAIL；与 `kernel` 依赖图无关（kernel 生产仅 `thiserror`）。人审接受 kernel 面 deny 语义闭合；workspace yank 另案。

## 3. Registry stable

```toml
[[unit]]
path = "crates/kernel"
layer = "kernel"
status = "stable"   # was incubating
publish = false
```

与 Spec Status **Approved** + §18 勾选一致（§15.3 禁止 draft+stable 双重事实）。

**仅** `crates/kernel` 标 stable；testkit / 其它 unit 仍 incubating。

## 4. 明确不宣称

- crates.io `cargo publish`（`publish = false`）
- git tag / push（需另指令）
- branch/mutants/miri **实测分数**

## 5. 校验命令（release 后）

```bash
rg -n '^version' crates/kernel/Cargo.toml          # 0.1.1
rg -n 'status' -A2 -B2 'crates/kernel' .architecture/workspace.toml
rg -n '^RES-.*: OPEN' .agents/ssot/kernel/evidence/2026-07-14/residual-open.txt
# 期望: 无 OPEN（或仅无关）
cargo test -p kernel
cargo run -p archgate -- --json
```

## Decision

```text
RES-API-007: CLOSED (0.1.1)
RES-TEST-014/015/016: CLOSED (human waiver DEFER)
§18: FULL CHECK (with documented waivers on 18.3 last three)
registry crates/kernel: stable
publish: still false
```
