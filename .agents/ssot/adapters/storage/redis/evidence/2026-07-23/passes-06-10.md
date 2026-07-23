# redisx 缺口审查 Pass 6–10（critic）

| 字段 | 值 |
|------|-----|
| Worktree | `/home/workspace/infra.rs/.worktrees/feat/redisx-p0-close` |
| 对照草稿 | `.agents/ssot/adapters/storage/redis/plan/infra-rs-draft-spec-goal.md`（原 `.cargo/draft/redisx_SPEC_GOAL.md`） |
| 实现 | `crates/adapters/storage/redis`（package `redisx` **0.3.5**） |
| 对齐 | `docs/ssot/redisx-ssot-alignment.md`、`docs/ssot/adapters-ssot-alignment.md`、`docs/ssot/gap-matrix.md`、`docs/ssot/draft-gap-matrix.md` |
| 前置 | `/tmp/.../reviews/gap-matrix-v0.md` **缺失**；本文件独立完成 6–10 轮 |
| 日期 | 2026-07-23 |
| 角色 | critic / 只读审查；不改源码 |

---

## Pass 6: Pub/Sub reliability claims

### Residual findings

1. **[P0 诚实边界 · 已正确标注但易被 surface live 误读]**  
   - 源码与文档一致拒绝“可靠投递”：`pubsub.rs` 结构体注释写「**不**提供可靠投递保证」；`operations.md` / `usage.md` / alignment REDISX-15 将「断线重订阅与消息必达」标 **OPEN / NO-GO**。  
   - 证据：`crates/adapters/storage/redis/src/pubsub.rs:19-21`；`docs/operations.md:54`；`docs/ssot/redisx-ssot-alignment.md:45`（REDISX-15 OPEN）。

2. **[P1 · 断连事件被静默吞掉 — 对照 draft §2.5]**  
   - Draft 要求：「断连事件不得被静默吞掉」；合同 stream 无 `Result` 时，**必须**另提供 `RedisMessageStream<Item=XResult<BusMessage>>`。  
   - 实现：`into_message_stream` 用 `filter_map` 只映射 payload → `BusMessage`，**无**错误项、**无** disconnect 事件、**无** `RedisMessageStream` 扩展类型。订阅连接挂掉时 stream 结束，调用方只能“收不到消息”，无法区分正常 idle 与故障。  
   - 证据：`pubsub.rs:120-134`；全 crate 无 `RedisMessageStream` 符号。

3. **[P1 · 无重连 / 无重订阅实现]**  
   - Publish 侧用 `ConnectionManager`（驱动可自动重连），但 **subscribe 流** 是一次性 `get_async_pubsub` → `into_on_message()`，无 resubscribe 状态机、无 lag/断线指标。  
   - `RedisPubSubFacade::sub_channel` 每次新建会话，失败后不会自动恢复原订阅。  
   - 证据：`pubsub.rs:77-102,184-189`；live 注释明确「不宣称必达」：`tests/live_pubsub_conformance.rs:1-3`。

4. **[P1 · 拓扑 fail-closed 正确，但能力面窄]**  
   - Cluster / Sentinel 在 I/O 前 `Invalid` 拒绝，不降级 Standalone —— **诚实且符合 gate 阻断条件**。  
   - 证据：`pubsub.rs:193-202` + 单测 `cluster_pubsub_fails_closed_before_connect` / `sentinel_pubsub_fails_closed_before_connect`。  
   - 状态校正：REDISX-14（配置一致性 / fail-closed）可维持 **PASS**；REDISX-15 必须保持 **OPEN**，不得因 surface live 上调。

5. **[P2 · PUBLISH 重试边界]**  
   - 预算路径对 PUBLISH 声明 `NeverAutomatic`（不自动重试）—— 与“避免重复投递”一致；但仍可能丢消息。文档已写清。无新冲突。  
   - 证据：`docs/ssot/redisx-ssot-alignment.md:55`；`operations.md:43`。

### Status corrections

| ID / 主张 | 应处状态 | 证据 |
|-----------|----------|------|
| Pub/Sub Standalone 接线 + ACL/TLS 复用 | PASS | `connect_config` + offline 单测 |
| Cluster/Sentinel Pub/Sub | 明确 **NO-GO**（非“部分支持”） | `pubsub_connection_info` fail-closed |
| 可靠投递 / 断线重订阅 / disconnect 可观测 | **OPEN**；禁止因 `live_pubsub_portable_surface` 暗示完成 | surface suite 仅 contract 可调用性 |
| Draft 扩展 `RedisMessageStream` | **未实现** | 全树无符号 |

### Pass 6 结论

Pub/Sub **可靠性主张总体诚实（NO-GO 写清）**；残余风险是：合同 stream 静默断流 + 缺 Result 扩展 + 仅 surface live，若对外话术写成“生产 Pub/Sub 就绪”会越界。建议话术限定为：**可选 Standalone Pub/Sub 入口，无可靠性合同**。

---

## Pass 7: Cluster / Sentinel honesty

### Residual findings

1. **[P0 · 代码路径 ≠ live PASS — alignment 已 OPEN，但 gap 矩阵口径冲突]**  
   - `RedisPool::connect` 确有 `connect_cluster` / `connect_sentinel`（`ClusterClient` / `Sentinel::async_master_for`）。  
   - 离线测试仅为 **连拒绝端口**（`127.0.0.1:1`），且接受 **外层 timeout 空分支当“通过”**，**不**验证 slot 路由、MOVED/ASK 有界、failover 后 master 再发现。  
   - 证据：`pool.rs:319-389,428-483`；`redisx-ssot-alignment.md:40-41` REDISX-10/11 **OPEN**。  
   - **冲突**：`docs/ssot/draft-gap-matrix.md:5` 写「production RedisPool + Cluster/Sentinel/TLS/resiliencx | **done** DEFER closed」—— 易被解读为拓扑能力已闭合；与同仓 `gap-matrix.md:8`「Cluster/Sentinel/TLS live … deferred」及 alignment **OPEN** 不一致。  
   - **校正**：Cluster/Sentinel **实现脚手架 PASS**；**真实拓扑 / 故障切换 live = OPEN**。禁止用 “DEFER closed” 覆盖 live OPEN。

2. **[P1 · Sentinel 语义诚实缺口]**  
   - 发现 master 后落入 `RedisBackend::Standalone(ConnectionManager)`（`pool.rs:35-43,389`）。  
   - **无** 运行时 Sentinel 再发现 / failover 跟随：master 切换后 `ConnectionManager` 重连旧地址行为未测、未文档化为“仅连接时发现一次”。  
   - 配置注释写「发现 master 后以 Standalone ConnectionManager 连 master」（`config.rs:17-18`）—— 实现匹配，但 `operations.md` 故障表只写 ConnectionManager 重连，**未**声明 Sentinel failover NO-GO。  
   - **校正**：文档应显式：**Sentinel = 建连时 master 发现；运行时 failover 重发现 OPEN/NO-GO**。

3. **[P1 · Cluster 与 draft §2.4 差距]**  
   - Draft：N 个 multiplexed lane、MOVED/ASK 重定向**有界**、blocking 独立 lane。  
   - 实现：P0 **单 lane**（`stats.open` 0/1；`pool.rs:27`）；依赖驱动 `ClusterConnection` 内部路由，本 crate **无** 有界重定向计数器/指标。  
   - MSET 跨 slot 原子性已诚实声明（README/alignment）—— 保留。  
   - 错误映射对 `Moved` → Transient（`error_map` 单测）属于分类层，**不**等于 Cluster 生产闭环。

4. **[P1 · TLS 与拓扑叠加]**  
   - `tls=true` → `TcpTls { insecure: false }` / Cluster `TlsMode::Secure` 代码路径存在；alignment REDISX-12 **OPEN**（无真实握手 live）。  
   - 默认 `RedisConfig.tls = false`（`config.rs:56`），与 draft §2.8「生产默认 TLS」冲突 —— 属配置策略 OPEN，不可标 production-secure-by-default。

5. **[P2 · Pub/Sub × Cluster/Sentinel]**  
   - 已 fail-closed（Pass 6）；`operations.md:53` 诚实。无新问题。

### Status corrections

| 主张 | 应处状态 | 说明 |
|------|----------|------|
| Cluster 连接 API 存在 | PASS（代码） | `connect_cluster` |
| Cluster **生产可用 / live** | **OPEN** | 无真实 cluster fixture / CI |
| Sentinel 建连发现 | PASS（代码） | `async_master_for` |
| Sentinel **failover 跟随** | **OPEN / NO-GO** | 一次性发现 + Standalone CM |
| draft-gap-matrix「Cluster/Sentinel … done DEFER closed」 | **需收紧措辞** | 仅指 P0 默认路径/DEFER 票；≠ 拓扑 live |
| TLS secure 构造 | PASS（离线） | 真实握手 OPEN |

### Pass 7 结论

实现侧对 **模式 enum + 连接分支** 基本诚实；**最大不诚实风险在矩阵汇总层**（draft-gap-matrix 把 Cluster/Sentinel/TLS 写进 “done”）。审查建议：**以 `redisx-ssot-alignment.md` REDISX-10/11/12 为准**，汇总表降级为「代码路径存在 / live OPEN」。

---

## Pass 8: coverage / test theater risks

### Residual findings

1. **[P0 · 连接失败测试可“空过”]**  
   - `connect_refused` / `cluster_connect_refused` / `sentinel_connect_refused`：`Ok(Err(_))` 断言 kind；`Err(_)` 外层超时分支 **无 assert**（注释“也视为连接失败路径被驱动”）。  
   - 若 connect 挂起到 3–5s 超时，测试仍绿，**未证明**错误映射或 fail-fast。  
   - 证据：`pool.rs:410-424,439-452,467-482`。  
   - 性质：**test theater 风险（弱断言）**，非伪造日志，但会抬高“Cluster/Sentinel 已测”的虚假信心。

2. **[P1 · 依赖环境的 close 测试 silent skip]**  
   - `closed_pool_is_closed_flag`：无 `from_env` 或 connect 失败则 `return`，**默认 CI 恒绿且零断言**。  
   - 真正 close 行为在 `live_kv.rs` 的 `#[ignore]` 用例；离线套件对 close 状态机覆盖不足。  
   - 证据：`pool.rs:486-505` vs `tests/live_kv.rs:25-38,98-106`。

3. **[P1 · Live 矩阵窄于 draft §2.10]**  
   | Draft 要求 | 现状 |
   |------------|------|
   | ACL / TLS / Cluster / Sentinel 集成 | 仅 Standalone live（ignored） |
   | kill/restart、主从切换、丢包、池耗尽故障 | **无** |
   | Pub/Sub 断线 | **无**；仅 `assert_pub_sub_surface` |
   | 24h soak / loom | **无** |
   | 并发 10k future | **无** |
   - Live 入口质量：**KV 扩展 + trait + close + TTL0** 尚可；**不能**外推拓扑/故障。

4. **[P1 · 通过数话术易陈旧]**  
   - 多处写「51 passed + 8 ignored」（alignment / README / 标准.md）。`Cargo.toml` 已是 **0.3.5**；若后续增测未刷新数字，属证据漂移。  
   - 本轮 **未** 重新跑 `cargo test`（critic 只读）；数字不得当作本轮 fresh 证据。

5. **[P2 · surface / contract 测试偏“点名导出”]**  
   - `public_api_surface` 主要 `assert_type` + 符号存在；resilience 单测较扎实（budget / UnsafeSideEffect 拒 I/O）。  
   - Scaffold 测试在 `feature = "scaffold"` 下，**不得**计入生产客户端覆盖率叙事。

6. **[P2 · feature `live` 空壳]**  
   - `Cargo.toml`：`live = []` 仅兼容编译；文档/标准仍写「live 测试（feature live）」易误导。实际 live = `#[ignore]` 集成测试。

### Status corrections

| 测试叙事 | 校正 |
|----------|------|
| 「Cluster/Sentinel 有测试」 | 仅 **拒绝连接 / 超时** 弱测；≠ 功能 live |
| 「51+8」 | 历史摘要；交付前需 fresh 复跑刷新 |
| Pub/Sub live | surface only；REDISX-15 仍 OPEN |
| 离线 close / 池耗尽 | **薄弱**；依赖 ignored live 或 silent skip |
| resiliencx 安全路由 | 单测相对 **强**，可维持 PASS |

### Pass 8 结论

存在 **中度 test theater 风险**：弱失败分支、silent skip、surface live、通过数冻结。未发现“编造 CI 输出”类 ALL-F01 证据；但 **不得** 用默认 `cargo test` 绿推断 Cluster/Sentinel/TLS/故障恢复已验收。

---

## Pass 9: docs / ops readiness

### Residual findings

1. **[P1 · 版本与命名漂移]**  
   | 文档 | 声称 | 实际 |
   |------|------|------|
   | `Cargo.toml` | — | **0.3.5** |
   | `redisx-ssot-alignment.md` / `goal.md` / landing / 多处 README | 0.3.4 | 落后 |
   | `docs/标准.md` | v0.3.4；符号 `RedisKeyValueStore` / `redisx::Config` | **无此类型**；真实为 `RedisClient` / `RedisConfig` |
   | `标准.md` 架构图 `kv — RedisKeyValueStore` | 过时 | 实现无独立 kv 模块 |
   - 证据：`Cargo.toml:4`；`docs/标准.md:3-27,35-40`。

2. **[P1 · 标准.md 过度承诺 / 模板残留]**  
   - 写「tracing 日志」「连接健康检查、异常连接自动驱逐」「配置通过 configx 注入」「故障转移 — Cluster/Sentinel 命令连接路径已实现」+ 评分 4/5。  
   - 源码 **无** `tracing`/`metrics` 埋点（`src/` grep 仅 resiliencx instrumentation noop 语义）；池是 Semaphore + 单 backend clone，**无** 空闲连接驱逐。  
   - `operations.md` 指标仅为 **建议**（`redisx_inflight` 等），非已实现导出。  
   - **校正**：`标准.md` 不得作为 ops/生产就绪证据；以 `operations.md` + alignment 为准并应修漂移。

3. **[P1 · Draft 可观测性 §2.9 vs 实现]**  
   - 必须：请求计数/延迟、排队、in-flight、重连、重定向、Pub/Sub lag。  
   - 实现：`RedisPoolStats { open, in_flight, waiters }` 仅内存快照；无 metrics feature、无 span。  
   - readiness：`ping` + `stats().open` 文档齐全（`operations.md:10-16`）—— **应用侧可拼**，库未提供三级 health API 类型。

4. **[P2 · ops 正向点]**  
   - `operations.md` / README / usage 对重试矩阵、Pub/Sub 拓扑 NO-GO、Cluster live OPEN 较诚实。  
   - Live 命令、env 前缀、禁止密钥回显清晰。  
   - Gate 阻断条件（`gate/gate.md`）与代码 fail-closed 对齐良好。

5. **[P2 · 升级/回滚/故障演练手册]**  
   - Draft DoD 要求升级回滚指南与至少一次故障演练证据。  
   - 有 CHANGELOG / releases 短摘要；**无** 独立 runbook 级 failover 演练记录（本 worktree 内）。`releases/0.3.5.md` 仅“三轮加固摘要 + 未宣称 stable”。

### Status corrections

| 文档面 | 状态 |
|--------|------|
| usage / config / operations（核心运维边界） | **基本就绪（P0 单机）** |
| 标准.md | **需修订**（符号/版本/ tracing/configx/驱逐） |
| SSOT goal / alignment 版本行 | **落后 0.3.5** |
| 可观测性落地 | **OPEN**（stats 有；metrics/tracing 无） |
| 故障演练 / soak 手册 | **OPEN** |

### Pass 9 结论

**单机 KV 运维文档可用**；跨文档版本与 `标准.md` 模板承诺是主要 ops 诚信风险。对外/对内交付应以 `operations.md` + `redisx-ssot-alignment.md` 为 SSOT 叙述，**降权** `标准.md` 直至纠偏。

---

## Pass 10: DoD / production-grade claim legality

### Residual findings

1. **[合法可宣称 — 窄口径]**  
   在同时满足以下限定时，**允许**说「**P0 生产默认 KV 客户端入口已落地**」：  
   - 默认面 `RedisPool` / `RedisClient` / `RedisConfig`（非 scaffold）  
   - Standalone + timeout/acquire 背压 + close  
   - 参数化 retry safety（ReadOnly / Idempotent / UnsafeSideEffect / NeverAutomatic）  
   - live `#[ignore]` 入口存在；密钥不入库  
   - **显式不宣称** package stable / crates.io  
   - 证据：`goal/goal.md:10-23`；`README.md:7-11`；`gate/gate.md:25`；`Cargo.toml:publish = false`。

2. **[非法 / 越界宣称 — 对照 draft 全文 DoD]**  
   Draft 标题「生产级开发库」与 §1.1/§2.12 成功标准包含：高并发有界、节点切换、24h soak、完整指标、Cluster/Sentinel 里程碑、故障演练、TLS 生产默认、pipeline/Lua/锁等。  
   **当前不得**宣称：  
   | 宣称 | 合法性 |
   |------|--------|
   | package stable / Production Ready 全能力 | **非法**（REDISX-9 OPEN；adapters 汇总 OPEN） |
   | Cluster / Sentinel / TLS **已验证生产** | **非法**（REDISX-10/11/12 OPEN） |
   | Pub/Sub 可靠 / 必达 / 断线恢复 | **非法**（REDISX-15 OPEN） |
   | 满足 draft 全文 DoD / SLO（10k future、soak、p99 门禁） | **非法**（无证据） |
   | draft-gap-matrix「Cluster/Sentinel/TLS … done」作为拓扑闭环 | **非法或误导**（与 alignment 冲突） |
   | 标准.md「评分 4/5」「自动驱逐」「configx 注入」 | **不得引用为验收** |

3. **[P0 战役 DoD vs draft 全库 DoD]**  
   - 本仓 goal 将成功定义为 **draft P0 DoD 子集**（Pool+KV+timeout+live），**不是** draft §2.11 P1–P4 全阶段。  
   - `gap-matrix.md` redisx Current = **done** +  defer 列表 —— 在「OBJECTIVE = P0 默认路径」语义下可成立。  
   - 风险：读者把 “done” 读成 draft §2.12 全量 DoD。  
   - **校正话术（强制）**：  
     > `redisx 0.3.5`：**Standalone P0 KV 生产入口**已落地；Cluster/Sentinel/TLS **live OPEN**；Pub/Sub 可选且 **无可靠性合同**；**未** package stable；**未**声称 draft 全文生产级 DoD。

4. **[Gate / 审批链]**  
   - `gate/gate.md:32`：最终 SHA 待重冻；reviewer/verifier / PR 审批 pending。  
   - 在缺少 fresh CI + 独立 review 落盘前，**不得**关闭“可合并生产稳定”叙事；仅可称 worktree 内实现进展。

5. **[与 adapters 总览一致性]**  
   - `adapters-ssot-alignment.md:180-183`：live 入口 PARTIAL/OPEN；package stable OPEN —— 与 redisx 分文档一致。  
   - storage×7「生产默认路径 P0」**≠** 单包 stable。

### Status corrections（DoD 合法性矩阵）

| 验收项 | 合法性结论 | 依据 |
|--------|------------|------|
| P0 Pool+KV+timeout+Standalone live 入口 | **可宣称（有条件）** | 代码 + ignored live + 文档边界 |
| resiliencx 参数化 safety | **可宣称** | client 路由 + 单测 |
| Cluster/Sentinel 代码路径 | **可宣称“存在”**；**不可**宣称生产验证 | pool + OPEN live |
| Pub/Sub 可选 Standalone | **可宣称入口**；**不可**宣称可靠 | feature + NO-GO 文档 |
| 生产级（draft 全文）/ package stable | **禁止** | REDISX-9 与多处显式禁止 |
| DEFER closed = 拓扑完成 | **禁止** | 与 REDISX-10/11/12 冲突 |

### Pass 10 结论

**合法叙事收口为「Standalone P0 生产默认客户端」**；任何「生产级库已完成 / Cluster 已就绪 / Pub/Sub 可靠 / package stable」均 **不合法**。汇总矩阵若继续写 Cluster/Sentinel/TLS “done”，构成 **文档层 overclaim**，应在合并前纠正。

---

## 跨 Pass 汇总（6–10）

### 必须保持 OPEN / NO-GO（禁止上调）

1. REDISX-9 package stable  
2. REDISX-10 Cluster live  
3. REDISX-11 Sentinel live / failover  
4. REDISX-12 TLS 真实握手 live  
5. REDISX-15 Pub/Sub 重连与必达  
6. Draft 可观测 metrics/tracing 落地  
7. Draft 故障演练 / soak / 10k 并发 SLO  

### 可维持 PASS（窄证据）

1. 生产默认导出与 from_env  
2. Pub/Sub 配置复用 + 非 Standalone fail-closed  
3. 重试 safety 合同（含 PUBLISH NeverAutomatic）  
4. 种子 URL / Debug 脱敏（文档与负向测试叙事；本轮未重跑）  
5. Standalone live **入口存在**（非本轮 fresh 执行证明）  

### 文档/矩阵优先修复（审查建议，非本 critic 改码）

1. 统一版本叙事 → **0.3.5**  
2. 收紧 `draft-gap-matrix.md` redisx 行：Cluster/Sentinel/TLS 标 **代码有 / live OPEN**  
3. 修订 `docs/标准.md` 幽灵类型与 tracing/configx/驱逐承诺  
4. 强化 connect_refused 类测试：禁止空 `Err(_)` 绿通  
5. `operations.md` 补 Sentinel「仅建连发现、无运行时 failover」  

### 总评（6–10）

| Pass | 焦点 | 裁决 |
|------|------|------|
| 6 | Pub/Sub 可靠性 | 边界诚实；缺 disconnect 可观测与 Result 扩展；无可靠性 |
| 7 | Cluster/Sentinel 诚实 | 代码路径有；live/failover OPEN；**矩阵 overclaim** |
| 8 | 测试剧场 | 中度风险（弱断言/silent skip/surface live） |
| 9 | 文档运维 | 核心 ops 可用；标准.md/版本漂移拖后腿 |
| 10 | DoD 合法性 | **仅 P0 Standalone KV 入口可宣称**；全文生产级非法 |

**最终 critic 立场**：当前 worktree 的 redisx **不构成** draft「生产级开发库」DoD 闭合；构成 **有边界的 P0 Standalone 生产默认客户端**。任何向 Cluster/Sentinel/TLS/可靠 Pub/Sub/package stable 的升格，在现有证据下均应 **Block / Request changes（文档与宣称）**。

---

## 证据索引（路径）

- Draft：`.agents/ssot/adapters/storage/redis/plan/infra-rs-draft-spec-goal.md`  
- Alignment：`docs/ssot/redisx-ssot-alignment.md`  
- Gap：`docs/ssot/gap-matrix.md`、`docs/ssot/draft-gap-matrix.md`  
- 实现：`crates/adapters/storage/redis/src/{lib,pool,pubsub,config,resilience}.rs`  
- 测试：`tests/live_kv.rs`、`tests/live_pubsub_conformance.rs`、`pool.rs` 内 tests  
- Ops：`crates/adapters/storage/redis/docs/operations.md`、`docs/标准.md`、`README.md`  
- Gate/Goal：`.agents/ssot/adapters/storage/redis/{gate/gate.md,goal/goal.md}`  

*本报告不含密钥；未执行写入生产代码；未宣称本轮 fresh `cargo test` 通过。*
