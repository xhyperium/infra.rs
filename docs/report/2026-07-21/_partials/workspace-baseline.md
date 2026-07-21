# 工作区质量基线（Workspace Baseline）

| 项 | 值 |
| --- | --- |
| 采集时间（UTC） | 2026-07-21T11:44:57Z |
| 工作目录 | `/home/workspace/infra.rs` |
| 角色 | 验证员（只读验证，不改源码） |
| rustc | `1.97.0 (2d8144b78 2026-07-07)` |
| cargo | `1.97.0 (c980f4866 2026-06-30)` |
| cargo-deny | `0.20.2` |
| workspace members | **21** |
| STATUS.md 平均完成度（输入给定） | **92%** |

## 总览

| 门禁 | 命令 | exit code | 结果 |
| --- | --- | ---: | --- |
| 测试 | `cargo test --workspace --all-targets` | **0** | **通过** — 54 suites / **522** tests passed；0 failed；0 ignored |
| Clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **0** | **通过** — 无 warning / error 输出 |
| 格式 | `cargo fmt --all -- --check` | **0** | **通过** — 无 diff |
| 依赖策略 | `cargo deny check` | **0** | **通过** — advisories / bans / licenses / sources 均 ok（1 条配置侧 license 提示，非阻断） |

**结论：当前工作区质量基线四项门禁全部通过，可作为生产门禁（prod gate）对照的绿基线。**

---

## 1. `cargo test --workspace --all-targets`

- **命令**：`cargo test --workspace --all-targets 2>&1 | tail -100`
- **exit code**：`0`
- **摘要**：
  - 测试套件（suite）：**54** 全部 `ok`
  - 用例合计：**522 passed / 0 failed / 0 ignored**
  - 失败 crate：**无**
  - 通过 crate（含 unit + integration / example 目标所覆盖的 21 个 workspace package）：**全部通过**

### 1.1 Workspace members（21）

| package | 版本 | 测试结果（本轮） |
| --- | --- | --- |
| `kernel` | 0.3.0 | 通过（unit 65 + 多组 integration） |
| `testkit` | 0.1.1 | 通过（unit 20 + ManualClock 相关 integration） |
| `configx` | 0.1.0 | 通过 |
| `schedulex` | 0.1.0 | 通过 |
| `bootstrap` | 0.3.0 | 通过（unit 31 + public_api） |
| `evidence` | 0.1.0 | 通过 |
| `observex` | 0.1.0 | 通过 |
| `resiliencx` | 0.1.0 | 通过（unit 26 + public_api + retry_contract） |
| `transportx` | 0.1.0 | 通过（unit 0 + mock_http 18 + reqwest_driver 14 + websocket 9） |
| `decimalx` | 0.1.0 | 通过（unit 63 + 多组 property/oracle integration） |
| `canonical` | 0.1.0 | 通过 |
| `contracts` | 0.1.0 | 通过（unit 22 + conformance / surface / venue_override） |
| `binancex` | 0.3.0 | 通过（scaffold unit） |
| `okxx` | 0.3.0 | 通过（scaffold unit） |
| `clickhousex` | 0.3.0 | 通过（scaffold unit） |
| `kafkax` | 0.3.0 | 通过 |
| `natsx` | 0.3.0 | 通过 |
| `ossx` | 0.3.0 | 通过 |
| `postgresx` | 0.3.0 | 通过 |
| `redisx` | 0.3.0 | 通过 |
| `taosx` | 0.3.0 | 通过 |

### 1.2 关键 suite 尾部摘录（末段）

```text
test result: ok. 18 passed; 0 failed; 0 ignored; ...  # mock_http (transportx)
test result: ok. 14 passed; 0 failed; 0 ignored; ...  # reqwest_driver
test result: ok. 9 passed; 0 failed; 0 ignored; ...   # websocket
EXIT_CODE=0
```

**失败日志关键行**：无（本轮无 `FAILED`）。

---

## 2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`

- **命令**：`cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -50`
- **exit code**：`0`
- **摘要**：
  - 以 `-D warnings` 运行（警告即错误）
  - 输出实质为 `Finished dev profile ...`（本机 target 已热，增量很快）
  - **0 error / 0 warning**
- **失败 crate**：**无**

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.26s
EXIT_CODE=0
```

---

## 3. `cargo fmt --all -- --check`

- **命令**：`cargo fmt --all -- --check 2>&1 | tail -20`
- **exit code**：`0`
- **摘要**：无任何需要重新格式化的文件；stdout 为空。
- **失败文件**：**无**

```text
EXIT_CODE=0
```

---

## 4. `cargo deny check`

- **命令**：`cargo deny check 2>&1 | tail -30`
- **exit code**：`0`
- **工具版本**：`cargo-deny 0.20.2`（可用）
- **摘要**：
  - `advisories ok`
  - `bans ok`
  - `licenses ok`
  - `sources ok`
  - 配置提示（**非失败**）：`deny.toml` 允许列表中的 `"Zlib"` 在当前依赖图中未出现（`license-not-encountered`）

```text
warning[license-not-encountered]: license was not encountered
   ┌─ deny.toml:19:4
   │
19 │   "Zlib",
   │    ━━━━ unmatched license allowance

advisories ok, bans ok, licenses ok, sources ok
EXIT_CODE=0
```

---

## 5. 统计

| 指标 | 值 |
| --- | --- |
| Workspace members | **21** |
| 测试 suites | 54 |
| 测试用例通过 | **522** |
| 测试用例失败 | **0** |
| Clippy 告警（`-D warnings`） | **0** |
| fmt 不合规文件 | **0** |
| cargo-deny 阻断项 | **0** |
| STATUS.md 平均完成度（给定） | **92%** |

---

## 6. 与生产门禁的关系说明

| 门禁项 | 本仓 / 组织约定 | 本轮基线 | 生产含义 |
| --- | --- | --- | --- |
| `cargo test --workspace`（本轮加 `--all-targets`） | AGENTS.md / CLAUDE.md / Rust 规范：提交前必须 test | **绿** | 功能回归门槛；`--all-targets` 覆盖 unit + integration + example 测试目标，严于「仅 lib 测试」 |
| `cargo clippy ... -D warnings` | 组织 Rust P0：clippy 警告阻断合并 | **绿** | lint 质量门槛；与 CI/PR 生产门禁对齐 |
| `cargo fmt --check` | 组织 Rust P0：fmt 为空气门禁 | **绿** | 风格一致性门槛 |
| `cargo deny check` | 推荐 / CI 依赖审计（许可证 + 漏洞 + 来源） | **绿**（仅 Zlib allow 未命中提示） | 供应链门槛；当前无 advisories/bans 阻断 |
| 完成度 92% | 模块 STATUS 进度（非编译门禁） | 输入给定 | **完成度 ≠ 可 ship**：adapters 等仍可为 scaffold；门禁绿只证明「当前代码可编译、可测、可 lint」，不自动等价于业务 COMPLETE |

### 解读要点

1. **基线用途**：本文件记录 **2026-07-21** 工作区快照，供 `infra-status-modules-prod-audit` 审计对照；后续回归应复跑同命令并比对 exit code / 失败列表。
2. **未伪造**：四项命令均在本会话实际执行；失败时应保留日志关键行——本轮无失败，故无失败栈。
3. **与 92% 完成度关系**：平均完成度 92% 来自 STATUS 文档口径；本基线证明工程门禁健康，但 **adapters scaffold / 未落地 tools** 等产品完备性仍以对齐文档与 STATUS 为准，不可仅凭门禁绿宣称「全模块生产就绪」。
4. **建议生产 PR 门禁最小集**（与本仓一致）：
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo test --workspace --all-targets`（或至少 `--workspace`）
   - `cargo deny check`（若 CI 已装 `cargo-deny`）

---

## 7. 原始日志位置（本机会话临时）

| 产物 | 路径 |
| --- | --- |
| test | `/tmp/cargo-test-workspace.log` |
| clippy | `/tmp/cargo-clippy-workspace.log` |
| fmt | `/tmp/cargo-fmt-check.log` |
| deny | `/tmp/cargo-deny-check.log` |

> 注：`/tmp` 日志为验证会话临时文件，不保证跨会话保留；以本 markdown 摘要为审计落盘 SSOT。
