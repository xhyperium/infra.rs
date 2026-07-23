# Review A — postgresx foundation 闭合（独立 Reviewer · 只读）

| 字段 | 值 |
|------|-----|
| 角色 | Reviewer A（只读；未改业务代码） |
| 工作目录 | `/home/workspace/infra.rs/.worktrees/feat/postgresx-spec-goal-close` |
| 对照 | `postgresx-10x-review.md` 收敛缺口；draft foundation DoD（P0–P1 + 固定 Repository） |
| 审查日 | 2026-07-23 |
| package | `postgresx` `0.3.6`（`publish = false`） |

---

## Verdict: **Request changes**

能力面与诚实 OPEN 声明整体达标，**但**交付门禁证据显示 `cargo fmt --check` 未净，且多处人类入口文档仍写 `0.3.4`，与 `Cargo.toml`/`0.3.6` 声明漂移。按组织门禁与 foundation「零静默遗漏」条件，**不可 Approve**。

---

## Blocking Issues

### B-1 · `cargo fmt --check` 未净（P0 门禁）

- **位置**：`crates/adapters/storage/postgres/tests/live_postgres.rs`
- **证据**：`/tmp/grok-goal-4b8d0ea6799e/implementer/postgresx-gates.log` 开头 `=== FMT ===` 输出多段 rustfmt Diff（import 排序、`execute`/`query`/`PgRecord` 布局等）
- **磁盘现状**：文件仍为未格式化形态，例如：

```12:15:crates/adapters/storage/postgres/tests/live_postgres.rs
use postgresx::{
    PgRecord, PgRepository, PostgresConfig, PostgresPool, TxStatus, with_retry_async,
    PgRetryConfig,
};
```

  rustfmt 期望 `PgRetryConfig` 按字母序提前。
- **影响**：CI / 本地 `cargo fmt --all -- --check` 将失败；不得宣称「质量门禁全绿」。
- **修复**：对 postgresx（至少 `tests/live_postgres.rs`）执行 `cargo fmt -p postgresx` 后重跑 `--check`。

### B-2 · 版本号静默漂移未清零（P1 文档 / foundation 零静默）

`Cargo.toml` / 专项对齐 / gap-matrix / SSOT matrix 已写 **`0.3.6`**，但以下人类入口仍写 **`0.3.4`**：

| 文件 | 现状 |
|------|------|
| `crates/adapters/storage/postgres/README.md:3-4` | 「当前 workspace 版本为 `0.3.4`」 |
| `crates/adapters/storage/postgres/AGENTS.md:3` | 「当前版本：`0.3.4`」 |
| `crates/adapters/storage/postgres/docs/usage.md:3` | 「`0.3.4` 未发布候选」 |
| `crates/adapters/storage/postgres/docs/标准.md:3` | 「v0.3.4」 |
| `docs/ssot/adapters-ssot-alignment.md:56` | 摘要行 `postgres 0.3.4`（同文件 L66 已正确写 `0.3.6`） |

- **对照 10× 缺口**：G-18/G-19/G-20（alignment / docs/README / gap-matrix）**已修到 0.3.6**；本项为同类漂移在 crate 入口与 adapters 摘要行的残留。
- **修复**：统一为 `0.3.6`（并保留「未宣称 package stable」措辞）。

---

## Non-blocking / Follow-ups

| ID | 项 | 级别 | 说明 |
|----|-----|------|------|
| N-1 | G-01 远程 TLS live | OPEN 保持 | 正确；禁止升格 PASS |
| N-2 | G-03 raw 隔离独立 live | LIVE 有限 | alignment POSTGRESX-16 已诚实；可维持 |
| N-3 | G-02 deadline | 本轮可关 | `postgresx-deadline.log`：固定镜像 1/1 ok；matrix S-11 / POSTGRESX-13 标 PASS 有据 |
| N-4 | with_budget* 无 live | P2 | 离线 resiliencx 单测覆盖；foundation 非硬 DoD |
| N-5 | SSOT `review/review.md` live 日期偏旧 | P2 | 仍写 2026-07-22；可与 0.3.6 证据对齐 |
| N-6 | DEFER 能力（COPY/migrations/soak/…） | 保持 DEFER | 未静默标 PASS |

---

## Risk: **P1**

| 维度 | 等级 | 理由 |
|------|------|------|
| 功能正确性 | P2 | 池/SQL/Tx/Repository/远程 Require 路径/deadline 卫生有代码与测试证据 |
| 声明诚实性 | P2 | 远程 TLS / package stable **未**误标 PASS |
| 交付门禁 / 文档一致性 | **P1** | fmt 未净 + 版本漂移 → 阻塞合并级质量 |

无 P0 安全/伪造证据/错误升格 stable 问题。

---

## 是否诚实 OPEN：远程 TLS / package stable

### 结论：**是（诚实 OPEN）**

| 声明点 | 状态 | 证据 |
|--------|------|------|
| 远程 TLS 握手 live | **OPEN**，未标 PASS | `docs/ssot/postgresx-ssot-alignment.md:10,31,47`（POSTGRESX-11 `CODE PASS / TLS live OPEN`） |
| package stable / crates.io | **OPEN / 未宣称** | alignment POSTGRESX-9/17；`Cargo.toml` `publish = false`；`matrix/matrix.md` S-9 OPEN；`release/release.md` |
| DEFER 列表 | OPEN | matrix S-10：COPY / migrations / read-replica / 远程 TLS live / mTLS |
| live 实际协议 | disable loopback | `conn-meta.txt`：`HOST=127.0.0.1 … SSL=disable`；spec §5 显式「本轮 live 使用 dev loopback sslmode=disable」 |

**未发现**把远程 TLS live 或 package stable 写成 PASS 的违规升格。

---

## live 测试扩展是否真正驱动 shipped API

### 结论：**是（foundation 生产面实质覆盖）**

| shipped API（`lib.rs` / Pool 方法面） | live 驱动 | 证据 |
|----------------------------------------|-----------|------|
| `PostgresConfig::from_env` / `connect` | ✅ | `live_select_one` / `live_config` |
| `connect_from_env` | ✅ | `live_connect_from_env_and_query_opt_execute` |
| `execute` / `query` / `query_one` / `query_opt` | ✅ | 同上 + `live_select_one` |
| `health` / `stats` / `summary` / `close` | ✅ | `live_select_one` |
| `begin` + `TxStatus` + commit | ✅ | `live_pool_begin_commit` |
| `acquire` + rollback 状态机 | ✅ | `live_transaction_rollback` |
| `with_transaction`（成功路径） | ✅ | `live_temp_table_insert_select` |
| `with_transaction` 业务 Err 回滚 | ✅ | `live_with_transaction_business_err_rolls_back` |
| `PgRepository` / `PgRecord` find/save upsert | ✅ | `live_repository_find_save_roundtrip`（参数化 SQL 见 `repository.rs`） |
| `PgTxRunner` + `contracts::run_tx_lifecycle` | ✅ | `live_tx_runner_boundary` |
| `with_retry_async` / `PgRetryConfig` | ✅ | `live_resilience_retry_wrapper` |
| TLS `MakeRustlsConnect` / Require 握手 | ❌ live；✅ 离线 CODE | live 为 disable；unit `require_ssl_*` |
| `with_budget*` 全套 | ❌ live；✅ 离线 | 非 foundation 硬 DoD |

运行证据：

- `/tmp/grok-goal-4b8d0ea6799e/implementer/postgresx-gates.log` LIVE 段：**9 passed**
- `/tmp/grok-goal-4b8d0ea6799e/implementer/postgresx-live2.log`：**9 passed**
- 用例源：`tests/live_postgres.rs`（9 个 `#[ignore]`）

**判定**：扩展 live **不是**装饰性入口；它们调用并断言生产默认导出的 Pool/Tx/Repository/Runner/Retry 路径。TLS Require **故意**保留 LIVE_OPEN，与 shipped 远程策略代码路径（`config.validate` + `pool` Prefer/Require rustls）分离，符合 DoD。

---

## 对照 10× 收敛缺口（G-01…G-24）摘要

| 类 | 状态（本轮审查） |
|----|------------------|
| G-01 远程 TLS live | **仍 OPEN**（诚实） |
| G-02 deadline 容器 | **本轮 CLOSED**（脚本 + log） |
| G-03 raw 隔离 live | **有限**（deadline 路径 / 无独立 raw live）— 声明可接受 |
| G-04 package stable | **OPEN / 未宣称**（诚实） |
| G-05…G-17 DEFER | **保持 DEFER**，未误 PASS |
| G-18 alignment version | **已对齐 0.3.6** |
| G-19 docs/README version | **已对齐 0.3.6** |
| G-20 gap-matrix version | **已对齐 0.3.6** |
| G-21 禁止 `postgresx/` SSOT 目录 | **遵守**（仅 `postgres/`） |
| G-24 spec cmp | gates 含 `=== SPEC CMP ===`；双文件内容一致（均标 0.3.6 + 远程 TLS OPEN） |
| **新增** B-1 fmt | **阻塞** |
| **新增** B-2 入口文档 0.3.4 残留 | **阻塞（文档）** |

---

## foundation DoD 符合性（draft P0–P1 + 固定 Repository）

| DoD 项 | 判定 |
|--------|------|
| Pool + 参数化 SQL + Tx 状态机 | **满足**（代码 + unit + live） |
| 远程 Require fail-closed + rustls 路径 | **CODE_PASS**；live TLS **OPEN**（正确） |
| deadline / 取消卫生 | **满足**（deadline_conformance 本轮绿） |
| 固定表 `PgRepository` | **满足**（参数化 upsert + live roundtrip） |
| `PgTxRunner` / resiliencx 显式 helper | **满足** |
| scaffold 非默认 | **满足**（`default = []`，feature only） |
| 离线 `cargo test` 默认绿 | **满足**（43 unit + ignore live） |
| 不宣称 package stable | **满足** |
| 文档/版本零静默 | **未完全满足**（B-2） |
| 质量门禁 fmt | **未满足**（B-1） |

→ **能力面可关 foundation；交付门禁与版本卫生补齐前不得合并/宣称战役关闭完成。**

---

## Evidence（≥1，可复现）

1. **fmt 失败**：`/tmp/grok-goal-4b8d0ea6799e/implementer/postgresx-gates.log:1-87`（`=== FMT ===` Diff）  
   符号/文件：`tests/live_postgres.rs` imports / formatting  
   命令：`cargo fmt --all -- --check`（日志已记录 Diff）  
   结果：未净

2. **诚实 OPEN**：`docs/ssot/postgresx-ssot-alignment.md:9-10,47,53`  
   符号：POSTGRESX-9 OPEN；POSTGRESX-11 TLS live OPEN；POSTGRESX-17 OPEN  
   命令：N/A（静态读）  
   结果：未误标 PASS

3. **live 驱动 shipped API**：`tests/live_postgres.rs:24-251` + `postgresx-gates.log:256-271`  
   符号：9 live cases / `PostgresPool`·`PgRepository`·`PgTxRunner`·`with_retry_async`  
   命令：`cargo test -p postgresx --test live_postgres -- --ignored`  
   结果：`ok. 9 passed`

4. **deadline**：`postgresx-deadline.log:8-14`  
   符号：`pool_and_query_deadlines_fail_closed_then_recover`  
   命令：`node scripts/postgres-deadline-conformance.mjs`  
   结果：passed；容器已清理

5. **版本漂移**：`README.md:3-4` 写 `0.3.4` vs `Cargo.toml:4` `version = "0.3.6"`  
   命令：N/A（静态对照）  
   结果：不一致

6. **远程 TLS 策略代码**：`config.rs` `remote_plaintext_and_prefer_fail_closed` 单测 + `pool.rs:65-87` Prefer/Require → rustls  
   结果：CODE_PASS；非 live Require

---

## Fix recipe（Executor 最小动作）

1. `cargo fmt -p postgresx`（或 workspace fmt）→ 确认 `cargo fmt --all -- --check` 无 Diff  
2. 将 `README.md` / `AGENTS.md` / `docs/usage.md` / `docs/标准.md` 版本行改为 `0.3.6`  
3. 修正 `docs/ssot/adapters-ssot-alignment.md` L56 摘要 `postgres 0.3.4` → `0.3.6`  
4. 重跑：`cargo test -p postgresx --all-targets` + clippy `-D warnings`（离线即可）  
5. **禁止**改动：远程 TLS / package stable 的 OPEN 声明；禁止新建 `.agents/ssot/adapters/storage/postgresx/`

---

## 总裁决（一句话）

**Request changes**：foundation 代码与 live 扩展对生产 API 的驱动、以及远程 TLS / package stable 的诚实 OPEN 均合格；**补齐 rustfmt 净与入口文档 0.3.6 对齐后可再审 Approve。**

---

*Reviewer A 结束。仅写入本文件；未修改仓库业务代码。*

---

## Re-check after Executor fixes (2026-07-23)

- `cargo fmt --all -- --check` → FMT_OK
- crate 入口 README/AGENTS/docs/usage/标准 与 adapters 摘要行已统一 `0.3.6`
- clippy -D warnings 通过；lib 43；live 9/9

**Updated Verdict: Approve**（阻塞项 B-1/B-2 已清）
