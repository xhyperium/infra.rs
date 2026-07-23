# postgresx 10× 独立对照审查（Analyst · 只读）

| 字段 | 值 |
|------|-----|
| 角色 | Analyst（只读；不改业务代码） |
| 工作目录 | `/home/workspace/infra.rs/.worktrees/feat/postgresx-spec-goal-close` |
| 审计日 | 2026-07-23 |
| package | `postgresx` `0.3.5`（`publish = false`） |
| 实现 | `crates/adapters/storage/postgres` |
| SSOT 目录 | **`.agents/ssot/adapters/storage/postgres/`**（正确；**不存在也不需要** `postgresx` 目录） |
| draft 权威快照 | `.agents/ssot/adapters/storage/postgres/plan/infra-rs-draft-spec-goal.md` |
| draft 原路径 | `.cargo/draft/postgresx_SPEC_GOAL.md`（本 worktree 磁盘缺失；SSOT 已入库） |
| 「100%」定义 | foundation 可关闭 DoD + 缺口零静默遗漏；**P2 COPY/migrations、P3 soak、量化产品平台默认 DEFER** |
| package stable / 未跑 live TLS | **禁止标 PASS** |

---

## 状态语义

| 状态 | 含义 |
|------|------|
| **CLOSED** | foundation DoD 已满足（代码 + 默认离线证据 + 文档/SSOT 诚实声明） |
| **CODE_PASS** | 代码路径已实现且离线可证；live 非本轮强制 |
| **LIVE_OPEN** | 依赖真实 Postgres / TLS 的证据本轮未跑或仅入口存在；**不得**升格为 PASS |
| **GAP** | foundation 关闭前须补齐，或文档/SSOT 与代码静默漂移 |
| **DEFER** | draft P2/P3 或量化/平台扩展；默认不阻塞 foundation 关闭 |

---

## 路径裁定（先于 10 轮）

| 问题 | 裁定 |
|------|------|
| 是否缺 `.agents/ssot/adapters/storage/postgresx`？ | **否。** 正确 SSOT 路径是 **`postgres`**（域目录 = storage backend 名）。package 名是 `postgresx`，目录名是 `postgres`。 |
| 是否应新建 `postgresx/` 目录？ | **不需要。** 建 `postgresx/` 会造成双入口，违反 SSOT 单源。 |
| draft 磁盘文件缺失？ | 以 `plan/infra-rs-draft-spec-goal.md` 为战役合同快照；对齐文档与 gap-matrix 均指向该快照。 |

证据：

- `.agents/ssot/adapters/storage/` 仅有 `postgres/` 等 7 后端目录，无 `postgresx/`
- `docs/ssot/adapters-ssot-alignment.md`：`storage/postgres` → package `postgresx`
- `docs/ssot/postgresx-ssot-alignment.md`：`SSOT = .agents/ssot/adapters/storage/postgres/`

---

# Round 1 — draft 范围与里程碑边界

### 焦点

draft GOAL/SPEC 的 foundation 边界 vs 量化/扩展 DEFER；P0–P3 哪些可关、哪些默认不关。

### 条款对照

| 条款ID | 条款 | 状态 | 证据路径 |
|--------|------|------|----------|
| D-R1-01 | GOAL：真实 `PostgresPool` + tokio-postgres + deadpool | **CLOSED** | `src/pool.rs`；`Cargo.toml` deps |
| D-R1-02 | 兼容 `contracts::{Repository, TxRunner}`，生产 SQL 用专属 API | **CLOSED** | `src/repository.rs`；`src/runner.rs`（诚实限制：TxContext 不传 SQL） |
| D-R1-03 | 非 ORM；参数化 SQL；迁移与业务分层 | **CLOSED** / **DEFER**(迁移) | 无 ORM；无 migrations feature（DEFER） |
| D-R1-04 | 不跨库事务；不默认自动重试事务体 | **CLOSED** | `with_transaction` 无隐式重试；resiliencx 显式 helper |
| D-R1-05 | 里程碑 P0：Pool/query/TLS/error（telemetry 弱） | **CODE_PASS**（telemetry 见 R6） | draft §2.12；实现 `pool/conn/tx/error/tls` |
| D-R1-06 | P1：transaction/statement cache/cancel | **CLOSED**(tx/cancel) / **DEFER**(statement cache API) | `tx.rs`/`conn.rs` RAII；无独立 Statement cache 公共 API |
| D-R1-07 | P2：repository codec / COPY / migration | **CODE_PASS**(固定表 Repository) / **DEFER**(COPY/migration/通用 codec) | `repository.rs`；spec OPEN |
| D-R1-08 | P3：failover / soak / performance 门禁 | **DEFER** | 无 24h soak；bench 有界但非回归门禁 |
| D-R1-09 | 量化数据平台 / Cluster·HA 产品面 | **DEFER** | goal Not in scope；landing「Cluster…DEFER」 |
| D-R1-10 | package stable / crates.io | **LIVE_OPEN** / 未宣称 | `release/release.md`；`publish = false` |

### 本轮结论

foundation 战役主线 = draft 前半 P0–P1 + 固定生产 Repository + 远程 Require 路径。  
后半扩展（COPY/migrations/soak/HA/量化）**默认 DEFER**，不得静默标 PASS。

---

# Round 2 — 公共 API 面 vs draft §2.3 / lib.rs

### 焦点

draft 列出的类型与方法是否落在生产默认导出；命名漂移是否构成 GAP。

### 条款对照

| 条款ID | 条款 | 状态 | 证据路径 |
|--------|------|------|----------|
| D-R2-01 | `PostgresConfig` / `PostgresConfigBuilder` | **CLOSED** | `src/config.rs`；`lib.rs` re-export |
| D-R2-02 | `PostgresPool`（Clone） | **CLOSED** | `src/pool.rs` |
| D-R2-03 | `PgConnection` checkout guard | **CLOSED** | `src/conn.rs` |
| D-R2-04 | `PgTransaction` 借用/状态机 | **CLOSED** | `src/tx.rs` `TxStatus` |
| D-R2-05 | `connect` / `acquire` / `execute` / `with_transaction` / `health` / `stats` / `close` | **CLOSED** | `pool.rs` |
| D-R2-06 | `acquire_with(deadline)` 独立参数 | **DEFER**/弱 GAP | 仅 `acquire` + 配置 `acquire_timeout`；无 per-call override |
| D-R2-07 | `QueryOptions` / `TxOptions` | **DEFER** | 无类型；超时走 config 字段 |
| D-R2-08 | `PostgresHealth` 结构化 | **DEFER** | `health() -> XResult<()>` |
| D-R2-09 | `PostgresPoolStats` 命名 | **CODE_PASS** | 实现为 `PoolStats`（`waiting` 字段） |
| D-R2-10 | `close(deadline)` async | **DEFER** | `close()` 同步关闭；无 deadline 排空 |
| D-R2-11 | 禁止「裸 URL 作唯一配置」 | **CLOSED** | 结构化字段 + URL 一次解析重建；`database_url` deprecated |
| D-R2-12 | `Statement` 对象 + operation name | **DEFER** | SQL 为 `&str` + `ToSql` 参数 |
| D-R2-13 | `PgTxRunner` / `PgRepository`（本仓验收扩展） | **CLOSED** | goal Acceptance；`lib.rs` |

### 本轮结论

生产可用最小 API **已闭合**。draft 部分类型是「理想形状」；本仓用等价简化（`PoolStats`、config 级 timeout）。  
这些命名/形状差若在 foundation DoD 中未要求，记 **DEFER**，不阻塞关闭；须在文档/对照表中显式列出（零静默）。

---

# Round 3 — 配置、池规则、deadline

### 焦点

draft §2.4 池规则 + 本仓 deadline/隔离合同。

### 条款对照

| 条款ID | 条款 | 状态 | 证据路径 |
|--------|------|------|----------|
| D-R3-01 | `max_size > 0` 校验 | **CLOSED** | `config.validate` |
| D-R3-02 | 无效配置启动失败 | **CLOSED** | `connect` → `validate` |
| D-R3-03 | acquire 有界 | **CLOSED** | `acquire_timeout` + deadpool wait/create/recycle |
| D-R3-04 | session 初始化 timezone/search_path | **DEFER** | 仅下发 `statement_timeout` |
| D-R3-05 | 未结束事务/desync 归还丢弃 | **CODE_PASS** | `RecyclingMethod::Clean`；timeout RAII `Object::take` |
| D-R3-06 | max lifetime + jitter | **DEFER** | 未暴露 max lifetime 配置 |
| D-R3-07 | checkout 不可 Clone | **CLOSED** | `PgConnection` 非 Clone |
| D-R3-08 | 每请求总 deadline + 取消后卫生 | **CODE_PASS** / **LIVE_OPEN** | `conn.rs`/`tx.rs`；`tests/deadline_conformance.rs` `#[ignore]` |
| D-R3-09 | 远程 `sslmode=disable/prefer` fail-closed | **CLOSED** | `config.validate` + 单测 `remote_plaintext_and_prefer_fail_closed` |
| D-R3-10 | 环境变量 FOUNDATIONX_* / DATABASE_URL | **CLOSED** | `config.rs`；goal Acceptance §3 |
| D-R3-11 | 密码 Debug 脱敏 | **CLOSED** | `PostgresConfig::Debug` `***` |
| D-R3-12 | 多 host / secret provider / TCP keepalive | **DEFER** | URL 多 host 拒绝；密码 env 明文字段 |

### 本轮结论

池与 deadline **foundation 路径 CODE_PASS/CLOSED**。  
`deadline_conformance` 与真实容器证据属 **LIVE_OPEN**（脚本存在：`scripts/postgres-deadline-conformance.mjs`）；本审查会话未重新跑 live，**不得**关 LIVE。

---

# Round 4 — SQL 安全、事务、错误映射

### 焦点

draft §2.5–2.8。

### 条款对照

| 条款ID | 条款 | 状态 | 证据路径 |
|--------|------|------|----------|
| D-R4-01 | 值参数化 `$N` + `ToSql` | **CLOSED** | 公共 SQL API 签名；lib 文档 |
| D-R4-02 | 标识符 allowlist + quote | **DEFER** | 无动态标识符 API（调用方自控 SQL 文本） |
| D-R4-03 | 大结果 stream/cursor 默认上限 | **DEFER** | `query` → `Vec<Row>` 无行数上限 |
| D-R4-04 | prepared statement cache + reprepare | **DEFER** | 依赖底层驱动；无本仓缓存 API |
| D-R4-05 | `TxStatus` Active/Committed/RolledBack/Failed | **CLOSED** | `tx.rs` `#[non_exhaustive]` |
| D-R4-06 | commit/rollback 一次终结 | **CLOSED** | `tx.rs` Invariant |
| D-R4-07 | Drop Active → 不归池 / 服务端回滚 | **CODE_PASS** | `tx.rs` 文档 + Drop 路径 |
| D-R4-08 | isolation / read_only / deferrable | **DEFER** | 无 `TxOptions` |
| D-R4-09 | `transaction_retry` 幂等 helper | **DEFER** | 有 resiliencx 通用 retry，无事务专用 |
| D-R4-10 | savepoint | **DEFER** | 未实现 |
| D-R4-11 | SQLSTATE 映射表 + 测试 | **CODE_PASS** | `error_kind_from_sqlstate` + unit tests |
| D-R4-12 | 双错误 `TransactionRollbackFailure` | **CLOSED** | `error.rs`；`with_transaction` 路径 |
| D-R4-13 | FK/check → draft 倾向 Conflict | **CODE_PASS***(语义选型)* | 代码：`23503` FK → **Invalid**；unique → Conflict（与 draft 表有意/无意偏差，须文档锚定，不算静默功能缺失） |
| D-R4-14 | `PgTxRunner` ≠ 完整事务 SQL 面 | **CLOSED** | `runner.rs` 诚实限制文档 |

### 本轮结论

事务状态机与参数化 + SQLSTATE **foundation 闭合**。  
stream/COPY/隔离级别/savepoint/retry-tx 为 **DEFER**。  
FK 映射偏差：记录为「已知语义」而非实现空洞，Executor 可选补文档一行。

---

# Round 5 — TLS / 安全 / 迁移

### 焦点

draft §2.9 + 本仓 TLS 合同。

### 条款对照

| 条款ID | 条款 | 状态 | 证据路径 |
|--------|------|------|----------|
| D-R5-01 | TLS 证书校验默认开启（Require 路径） | **CODE_PASS** | `tls.rs` webpki-roots；无 insecure 旁路 |
| D-R5-02 | Prefer/Require 走 rustls | **CODE_PASS** | `pool.connect` match sslmode |
| D-R5-03 | 真实远程 TLS 握手 live 验证 | **LIVE_OPEN** | 对齐文档 POSTGRESX-11 明确「未宣称真实 TLS 已验证」 |
| D-R5-04 | 自定义 CA / 客户端证书 mTLS | **DEFER** | spec OPEN |
| D-R5-05 | channel binding / SCRAM-PLUS | **DEFER** / 诚实声明 | `CHANNEL_BINDING_ENABLED=false`；`channel_binding()→none` |
| D-R5-06 | migration advisory lock / checksum | **DEFER** | 无 migrations feature |
| D-R5-07 | migration role ≠ runtime role | **DEFER** | 运维约定未代码化 |
| D-R5-08 | 外部 secret provider / 短期 token | **DEFER** | 密码为 `String` 字段 |
| D-R5-09 | `DATABASE_URL` query allowlist | **CLOSED** | `sslmode`/`application_name`/`connect_timeout` only |
| D-R5-10 | keyword DSN fail-closed | **CLOSED** | `validate_database_url_query` |
| D-R5-11 | feature 矩阵 draft：default=[runtime-tokio,tls-rustls]… | **DEFER**形状 | 本仓 `default=[]`；依赖默认启用；仅 `scaffold` 可选 |

### 本轮结论

TLS **实现路径 CODE_PASS**；**真实 TLS live = LIVE_OPEN（禁止 PASS）**。  
channel binding / mTLS / migrations = **DEFER**，已在 CHANGELOG 0.3.5 / spec OPEN 诚实声明。

---

# Round 6 — 可观测性、测试、文档、门禁

### 焦点

draft §2.10–2.11 + goal Acceptance + gate。

### 条款对照

| 条款ID | 条款 | 状态 | 证据路径 |
|--------|------|------|----------|
| D-R6-01 | 指标：acquire wait / pool / query latency… | **DEFER** | 无 metrics feature；仅 `PoolStats` + tracing 可选 dep |
| D-R6-02 | readiness = acquire + SELECT 1 | **CLOSED** | `health()` |
| D-R6-03 | liveness 不访问 DB | **DEFER** | 无独立 liveness API |
| D-R6-04 | diagnostics 脱敏 host + stats | **CODE_PASS** | `summary()` + `stats()` |
| D-R6-05 | 单元测试（config/SQLSTATE/状态机/脱敏） | **CLOSED** | 多模块 `#[cfg(test)]`；CHANGELOG 称 52 passed |
| D-R6-06 | live `#[ignore]` 入口 | **CLOSED**(入口) / **LIVE_OPEN**(本轮结果) | `tests/live_postgres.rs` |
| D-R6-07 | deadline 固定镜像实验入口 | **CODE_PASS**(入口) / **LIVE_OPEN** | `tests/deadline_conformance.rs` + script |
| D-R6-08 | bench 有界、不挂 `--all-targets` | **CLOSED** | `benches/query_hot_path.rs` harness=false；无 env 则 skip |
| D-R6-09 | crate docs usage/config/operations | **CLOSED** | `docs/*`（版本行有漂移 → R9 GAP） |
| D-R6-10 | 默认 `cargo test` 离线绿灯 | **CLOSED** | goal；gate；live ignore |
| D-R6-11 | scaffold 非默认 | **CLOSED** | `feature = "scaffold"` only |
| D-R6-12 | 24h soak / 多版本矩阵 / loom | **DEFER** | P3 |
| D-R6-13 | SBOM / cargo deny 专项于 crate | **DEFER**/workspace 级 | 组织门禁，非 crate 专属证据包 |

### 本轮结论

测试/文档 **foundation 门禁闭合**；观测指标与 soak **DEFER**。  
live 结果本轮 **LIVE_OPEN**。

---

# Round 7 — Repository / contracts / resiliencx（本仓 OBJECTIVE）

### 焦点

goal + alignment 中「原 OBJECTIVE DEFER → PASS」项。

### 条款对照

| 条款ID | 条款 | 状态 | 证据路径 |
|--------|------|------|----------|
| D-R7-01 | 生产 `PgRepository` + `PgRecord` | **CLOSED** | `repository.rs` 表 `infra_pg_records` |
| D-R7-02 | 参数化 find/save upsert | **CLOSED** | `find_sql`/`save_sql` 含 `$N` + ON CONFLICT |
| D-R7-03 | draft `PgEntity` 通用 codec | **DEFER** | 固定表形状，非通用 trait |
| D-R7-04 | version column / Conflict 并发控制 | **DEFER** | 简单 upsert |
| D-R7-05 | `TxRunner` 边界 | **CLOSED** | `PgTxRunner` |
| D-R7-06 | resiliencx `with_retry_*` / budget | **CLOSED** | `resilience.rs`；safety 文档诚实 |
| D-R7-07 | 池无自动 budget 接线 | **CODE_PASS**(诚实) | CHANGELOG 0.3.3：显式 helper，非隐式 |
| D-R7-08 | scaffold Record 隔离 | **CLOSED** | `#[cfg(feature = "scaffold")]` |

### 本轮结论

本仓 OBJECTIVE 扩展（Repository / SSL require 路径 / resiliencx）**已闭**。  
通用 `PgEntity`/version 并发 = **DEFER**（draft P2 理想，非 foundation 硬 DoD）。

---

# Round 8 — SSOT 11 层、对齐矩阵、gap-matrix

### 焦点

SSOT 完整性 vs 实现真相；禁止镜像 COMPLETE 冒充 ship。

### 条款对照

| 条款ID | 条款 | 状态 | 证据路径 |
|--------|------|------|----------|
| D-R8-01 | SSOT 树存在（goal/spec/test/gate/release/matrix/plan…） | **CLOSED** | `.agents/ssot/adapters/storage/postgres/**` |
| D-R8-02 | draft 快照入库 | **CLOSED** | `plan/infra-rs-draft-spec-goal.md` |
| D-R8-03 | landing 说明 | **CLOSED** | `plan/infra-rs-landing.md` |
| D-R8-04 | `spec.md` ↔ complete-spec 同构义务 | **CODE_PASS** | alignment 要求 `cmp`；本轮未重新执行 cmp（见 actionable） |
| D-R8-05 | matrix S-1..S-8 P0 | **CLOSED** | `matrix/matrix.md` |
| D-R8-06 | matrix S-9 package stable OPEN | **LIVE_OPEN**/正确 | 不得 PASS |
| D-R8-07 | matrix S-10 DEFER OPEN | **DEFER** | COPY/migrations/read-replica… |
| D-R8-08 | alignment POSTGRESX-1..12 | **CLOSED**/CODE_PASS | `docs/ssot/postgresx-ssot-alignment.md` |
| D-R8-09 | POSTGRESX-14/16 LIVE_OPEN 诚实 | **CLOSED**(声明) | alignment 表 |
| D-R8-10 | POSTGRESX-17 Release 候选 OPEN | **LIVE_OPEN** | 未冻结正式 SHA |
| D-R8-11 | gap-matrix postgresx 行 | **CODE_PASS** | `docs/ssot/gap-matrix.md`（版本写 0.3.3 落后 0.3.5 → R9） |
| D-R8-12 | 无 `postgresx` SSOT 目录 | **CLOSED**(正确) | 见路径裁定 |

### 本轮结论

SSOT 与 alignment **结构完整且诚实（含 LIVE_OPEN）**。  
版本号行滞后与 docs 版本漂移是 **文档 GAP**，不是能力空洞。

---

# Round 9 — 文档/版本一致性与静默遗漏扫描

### 焦点

零静默遗漏：版本、OPEN 项、API 命名。

### 条款对照

| 条款ID | 条款 | 状态 | 证据路径 |
|--------|------|------|----------|
| D-R9-01 | `Cargo.toml` version `0.3.5` | **CLOSED** | crate 根 |
| D-R9-02 | `docs/ssot/postgresx-ssot-alignment.md` version 行 `0.3.3` | **GAP** | 文件头 version=0.3.3；与 0.3.5 漂移 |
| D-R9-03 | `docs/README.md` 写 `0.3.4` | **GAP** | `crates/.../docs/README.md` |
| D-R9-04 | `adapters-ssot-alignment.md` 写 postgres `0.3.5` | **CLOSED** | 已对齐 package |
| D-R9-05 | gap-matrix 仍写 `0.3.3` | **GAP** | 快照日期 2026-07-22 |
| D-R9-06 | matrix S-10 含「SSL require-only 默认」 | **CODE_PASS***/澄清 | 远程已 force Require；loopback 可 Disable——「默认仅 Require」未全局强制，属边界澄清非空洞 |
| D-R9-07 | deprecated raw client/pool fail-closed | **CODE_PASS** / **LIVE_OPEN** | `conn`/`pool`；live-only 证据声明 |
| D-R9-08 | channel binding 合同单测 | **CLOSED** | `tls.rs` tests + const assert |
| D-R9-09 | `lib.rs` 导出面单测点名 | **CLOSED** | `default_exports_named` |
| D-R9-10 | examples/basic.rs 存在 | **CODE_PASS** | `examples/basic.rs`（未本轮执行） |

### 本轮结论

**能力面**无新空洞；**文档版本漂移** 3 处 GAP，应用文档补齐即可关闭静默遗漏。

---

# Round 10 — 收敛仲裁（前轮分歧只收敛、不互相否定）

### 焦点

合并 R1–R9：什么可标 foundation 100% 关闭条件，什么必须 DEFER/LIVE_OPEN。

### 仲裁原则

1. 后轮**只收敛**：同一事实取「更严证据等级」（LIVE_OPEN 优先于误标 PASS）。  
2. 命名简化（PoolStats vs PostgresPoolStats）**不**否决 CLOSED，记 DEFER/形状差。  
3. draft P2/P3 与量化平台 **默认 DEFER**。  
4. 未跑 live TLS / deadline 容器 **禁止 CLOSED-as-PASS**。

### 收敛条款总表（仲裁后唯一视图）

| 条款ID | 条款 | 最终状态 | 说明 |
|--------|------|----------|------|
| F-01 | workspace member + 生产默认导出 | **CLOSED** | |
| F-02 | from_env / FOUNDATIONX_* | **CLOSED** | |
| F-03 | Pool SQL/tx/health/stats/close | **CLOSED** | |
| F-04 | 参数化 + SQLSTATE + 双错误 | **CLOSED** | |
| F-05 | 远程 Require fail-closed + rustls 路径 | **CODE_PASS** | live TLS **LIVE_OPEN** |
| F-06 | deadline/取消/脱池 | **CODE_PASS** | conformance **LIVE_OPEN** |
| F-07 | PgRepository + PgTxRunner + resiliencx | **CLOSED** | |
| F-08 | scaffold 非默认；offline test；bench；docs 主体 | **CLOSED** | docs version 有 GAP |
| F-09 | package stable / crates.io | **LIVE_OPEN** | 禁止宣称 |
| F-10 | COPY / migrations / read-replica | **DEFER** | P2 |
| F-11 | soak / HA / 指标全套 / stream | **DEFER** | P3 |
| F-12 | 量化产品平台能力 | **DEFER** | 战役边界外 |
| F-13 | channel binding / mTLS / 自定义 CA | **DEFER** | 已诚实声明 |
| F-14 | SSOT 路径 `postgres` 非 `postgresx` | **CLOSED** | 正确 |
| F-15 | 对齐文档 version 漂移 | **GAP** | 文档补齐 |

### 本轮结论：foundation 可关闭条件

当且仅当：

1. F-01..F-08 保持（含文档 version GAP 修复）；  
2. F-09/F-10/F-11/F-12/F-13 显式 DEFER/LIVE_OPEN 写入 SSOT（已大体满足）；  
3. **不**把 LIVE_OPEN 改写成 PASS。

→ 在此定义下，**foundation DoD 可关**；**100% = 关闭 DoD + 缺口零静默**，不是 package stable。

---

# 收敛缺口清单（唯一）

| ID | 项 | 判定 | 动作 | 证据 |
|----|-----|------|------|------|
| G-01 | 真实 PostgreSQL TLS 握手（远程 Require） | **LIVE_OPEN** | **补齐** live 证据后才可升格；现阶段保持 OPEN | `tls.rs`；alignment POSTGRESX-11；禁止 PASS |
| G-02 | `deadline_conformance` / 固定镜像实验本轮结果 | **LIVE_OPEN** | **补齐**（可选门禁）：`node scripts/postgres-deadline-conformance.mjs` | `tests/deadline_conformance.rs` |
| G-03 | deprecated raw client/pool 隔离 live 证据 | **LIVE_OPEN** | **补齐** 或持续标注 live-only | spec §5；`conn`/`pool` |
| G-04 | package stable / Release 候选身份 | **LIVE_OPEN** | **已闭**(不宣称) / 未来 Lead 启动再补 | `release/release.md`；POSTGRESX-9/17 |
| G-05 | COPY | **DEFER** | **DEFER** | draft P2；matrix S-10 |
| G-06 | migrations | **DEFER** | **DEFER** | draft §2.9/P2 |
| G-07 | read-replica / target_session_attrs | **DEFER** | **DEFER** | goal Not in scope |
| G-08 | 24h soak / failover 演练 | **DEFER** | **DEFER** | draft P3/SLO |
| G-09 | metrics feature / 完整 RED 指标 | **DEFER** | **DEFER** | draft §2.10 |
| G-10 | stream/cursor/行数上限 / COPY 大结果 | **DEFER** | **DEFER** | draft §2.5 |
| G-11 | TxOptions isolation/savepoint/transaction_retry | **DEFER** | **DEFER** | draft §2.6 |
| G-12 | statement cache 公共 API / Statement+op name | **DEFER** | **DEFER** | draft §2.3/2.5 |
| G-13 | acquire_with / close(deadline) / PostgresHealth | **DEFER** | **DEFER**（形状差） | draft §2.3 |
| G-14 | channel binding SCRAM-PLUS | **DEFER** | **已闭**(诚实 none) / 未来增强 | `tls.rs` CHANNEL_BINDING |
| G-15 | 自定义 CA / mTLS | **DEFER** | **DEFER** | spec OPEN |
| G-16 | 通用 `PgEntity` codec / version 列 | **DEFER** | **DEFER** | 现有固定表 `PgRepository` 足够 foundation |
| G-17 | 量化数据平台 / HA Cluster 产品面 | **DEFER** | **DEFER** | landing；战役边界 |
| G-18 | `postgresx-ssot-alignment.md` version=0.3.3 | **GAP** | **补齐** → 0.3.5 | `docs/ssot/postgresx-ssot-alignment.md` |
| G-19 | crate `docs/README.md` version=0.3.4 | **GAP** | **补齐** → 0.3.5 | `crates/.../docs/README.md` |
| G-20 | gap-matrix postgresx 版本行 0.3.3 | **GAP** | **补齐** 快照（可选） | `docs/ssot/gap-matrix.md` |
| G-21 | 误建 `.../storage/postgresx` SSOT 目录 | **CLOSED** | **已闭** — 正确路径是 `postgres` | 目录列表 |
| G-22 | FK `23503`→Invalid vs draft Conflict 倾向 | **CODE_PASS** | **可选补齐** 映射表文档一行 | `error.rs` |
| G-23 | feature 名 draft 矩阵 vs `default=[]` | **DEFER** | **已闭**(本仓选择) / 文档可注 | `Cargo.toml` |
| G-24 | SSOT `cmp` spec 双文件 | **CODE_PASS** | **补齐** 验证命令执行（执行项） | gate/alignment 命令 |

---

# 生产默认公共 API 覆盖（对照 `lib.rs`）

来源：`crates/adapters/storage/postgres/src/lib.rs` 默认导出（**非** `scaffold`）。

| 导出符号 | 类别 | draft/goal 对应 | 状态 |
|----------|------|-----------------|------|
| `PostgresConfig` | config | draft §2.3 | **CLOSED** |
| `PostgresConfigBuilder` | config | draft §2.3 | **CLOSED** |
| `DEFAULT_MAX_POOL_SIZE` / `DEFAULT_PORT` | config | 本仓常量 | **CLOSED** |
| `SslMode` | config | TLS 策略 | **CLOSED** |
| `PostgresPool` | pool | draft 核心 | **CLOSED** |
| `PoolStats` | pool | ≈`PostgresPoolStats` | **CODE_PASS**（命名差） |
| `PgConnection` | conn | draft | **CLOSED** |
| `PgTransaction` | tx | draft | **CLOSED** |
| `TxStatus` | tx | draft 状态机 | **CLOSED** |
| `TxState` | tx | deprecated 兼容 | **CODE_PASS**（迁移期） |
| `PgTxRunner` | contracts | goal Acceptance | **CLOSED** |
| `PgRepository` / `PgRecord` | repository | 本仓 OBJECTIVE | **CLOSED** |
| `MakeRustlsConnect` / `build_client_config` | tls | draft TLS | **CODE_PASS** |
| `error_kind_from_sqlstate` / `xerror_from_sqlstate` | error | draft §2.8 | **CLOSED** |
| `map_pool_error` / `map_tokio_error` | error | 映射 | **CLOSED** |
| `TransactionRollbackFailure` | error | 双错误 | **CLOSED** |
| `with_retry_sync` / `with_retry_async` / `with_retry_async_no_wait` | resiliencx | OBJECTIVE | **CLOSED** |
| `with_budget*` 系列 / `PgRetryConfig` | resiliencx | budget | **CLOSED** |
| `Row` / `ToSql` re-export | types | 查询面 | **CLOSED** |
| `PostgresAdapter` / `Record` / mock 族 | scaffold | **非默认** | feature only |

**draft 有、本仓默认无（DEFER，非遗漏）**：

- `QueryOptions`、`TxOptions`、`PostgresHealth`、`acquire_with`、async `close(deadline)`
- `Statement` 类型、`PgEntity` trait、`transaction_retry`、savepoint、COPY、migrations API

**方法面（`PostgresPool`，实现核对）**：

| 方法 | 状态 |
|------|------|
| `connect` / `connect_from_env` | **CLOSED** |
| `acquire` | **CLOSED** |
| `execute` / `query` / `query_one` / `query_opt` | **CLOSED** |
| `with_transaction` / `begin` | **CLOSED** |
| `health` / `stats` / `close` / `summary` | **CLOSED** |
| `inner` (deprecated) | **CODE_PASS** fail-closed 隔离池 |

---

# 路径问题最终回答

> 本仓是否缺 `.agents/ssot/adapters/storage/postgresx`？

**答案：不缺，也不应创建。**

- **正确路径**：`.agents/ssot/adapters/storage/postgres/`
- **package 名**：`postgresx`
- **crate 路径**：`crates/adapters/storage/postgres`
- 新建 `postgresx/` 目录会造成 SSOT 双源，**禁止**

---

# 给 Executor 的精简 actionable gaps（≤15）

| # | 动作 | 优先级 | 备注 |
|---|------|--------|------|
| 1 | 更新 `docs/ssot/postgresx-ssot-alignment.md` 文件头 `version` → `0.3.5`，审计结论与 POSTGRESX 表保持 LIVE_OPEN 诚实 | P0 文档 | G-18 |
| 2 | 更新 `crates/adapters/storage/postgres/docs/README.md` 版本行 → `0.3.5` | P0 文档 | G-19 |
| 3 | （可选）刷新 `docs/ssot/gap-matrix.md` postgresx 版本/日期与「真实 TLS 实验 OPEN」措辞 | P1 文档 | G-20 |
| 4 | 跑 `cmp`：`spec/spec.md` vs `spec/xhyper-postgresx-complete-spec.md`；不一致则同步 | P0 校验 | G-24 |
| 5 | 离线门禁：`cargo test -p postgresx --all-targets` + `clippy -p postgresx --all-targets -- -D warnings` | P0 验证 | F-08 |
| 6 | **不要**把 G-01/G-02/G-03 标 PASS；若 Lead 要求证据，再跑 live / `postgres-deadline-conformance.mjs` | P0 纪律 | LIVE_OPEN |
| 7 | SSOT/goal/matrix 保持 COPY、migrations、read-replica、soak、package stable 为 DEFER/OPEN | P0 纪律 | G-05..G-09 |
| 8 | **禁止**创建 `.agents/ssot/adapters/storage/postgresx/` | P0 | G-21 |
| 9 | （可选）在 `docs/标准.md` 或 error 文档锚定 `23503→Invalid` 与 draft 差异 | P2 | G-22 |
| 10 | （可选）config/operations 补一句：无 `acquire_with`/`QueryOptions`，超时仅配置级 | P2 | G-13 |
| 11 | channel binding / mTLS / 自定义 CA 保持 OPEN 声明（已有 0.3.5 CHANGELOG） | P1 保持 | G-14/G-15 |
| 12 | 不实现 COPY/migrations/stream（除非 Lead 改 scope） | P0 范围 | DEFER |
| 13 | 不宣称 package stable / crates.io | P0 | G-04 |
| 14 | Release 身份：由 Release 对真实候选 commit 重算，不在 Code 阶段伪造 SHA | P0 | POSTGRESX-17 |
| 15 | 若只关 foundation goal：验收勾选 goal Acceptance 1–6 + 文档 version 对齐即可 | P0 关闭条件 | Round 10 |

---

# 总裁决（一句话）

**foundation 生产默认路径（Pool/SQL/Tx/Repository/TLS 代码路径/远程 Require/deadline 卫生/离线门禁/SSOT 诚实）已可关闭；真实 TLS live、deadline 容器 live、raw 隔离 live、package stable 仍为 LIVE_OPEN/未宣称；COPY·migrations·soak·量化平台默认 DEFER；文档 version 三处 GAP 须补；SSOT 正确路径是 `postgres`，不要 `postgresx` 目录。**

---

*Analyst 10× review 结束。未修改任何业务代码；本文件为唯一审查落盘产物。*
