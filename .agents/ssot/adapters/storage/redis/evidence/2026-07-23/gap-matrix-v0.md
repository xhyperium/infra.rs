# redisx Gap Matrix v0 — Draft SPEC × SSOT × 实现

| 字段 | 值 |
|------|-----|
| 审计日 | 2026-07-23 |
| Worktree | `/home/workspace/infra.rs/.worktrees/feat/redisx-p0-close` |
| Draft 来源 | 原 `.cargo/draft/redisx_SPEC_GOAL.md`（gitignored）→ 已入库 SSOT：`.agents/ssot/adapters/storage/redis/plan/infra-rs-draft-spec-goal.md` |
| SSOT 根 | `.agents/ssot/adapters/storage/redis/` |
| 实现 | `crates/adapters/storage/redis` → package **`redisx` `0.3.5`** |
| 对齐文档 | `docs/ssot/redisx-ssot-alignment.md`（文中仍写 `0.3.4`，与 Cargo 有漂移） |
| 方法 | 只读对照 draft MUST（§2.2–2.12 + P0–P4）/ SSOT matrix / 源码+测试+日志 |
| 状态枚举 | **PASS** / **PARTIAL** / **OPEN** / **NO-GO** |

## 0. 结论摘要

| 维度 | 判定 |
|------|------|
| Draft **全量 100%** | **未达成**；P2–P4 + 多项 P0 硬化项仍 OPEN/NO-GO |
| Draft **P0 生产默认 bar**（本仓可宣称） | **基本达成（PARTIAL→接近 PASS）**：Standalone 生产 `RedisPool`/`RedisClient` + 超时/背压/脱敏/错误映射 + 离线绿灯 + Standalone KV live 入口已绿 |
| SSOT 当前合同 | 与实现 **一致**：生产默认已落地；Cluster/Sentinel/TLS **真实 live OPEN**；package stable **禁止宣称**；Pub/Sub 重连/必达 **NO-GO** |
| SSOT 路径是否迁 `redisx/` | **否** — 保持 `adapters/storage/redis/`（package 名 `redisx`） |

### 0.1 本轮证据基线（命令）

| 命令/产物 | 结果 |
|-----------|------|
| offline default `cargo test -p redisx --all-targets` | lib **49 passed**；live **5+2 ignored**（`/tmp/.../redisx-offline-default.log`） |
| offline + `pubsub` | lib **54 passed**；ignored live 含 pubsub（`redisx-offline-pubsub.log`） |
| live KV（`--ignored`） | **5 passed**（`redisx-live-kv.log`） |
| live conformance | **2 passed**（`redisx-live-conformance.log`） |
| live pubsub | **1 passed**（`redisx-live-pubsub.log`） |
| SSOT 自述 | `51 passed + 8 ignored`（与本树 49/54 略有偏差，属文档滞后） |

> 说明：live 日志总耗时 ~0.01s 属“已连上本地/容器 Redis”量级；**不**构成 Cluster/Sentinel/TLS 拓扑证据。

---

## 1. 路径与版本漂移（元问题）

| 项 | Draft | SSOT | 实现 | 状态 | 证据 | 残差风险 |
|----|-------|------|------|------|------|----------|
| Draft 文件位置 | `.cargo/draft/redisx_SPEC_GOAL.md` | 入库 `plan/infra-rs-draft-spec-goal.md` | N/A | **PASS**（迁移） | `plan/infra-rs-draft-spec-goal.md:1-5` | 勿再在 gitignored draft 当 SSOT |
| package 名 | `redisx` | `redisx` | `name = "redisx"` | **PASS** | `Cargo.toml:2` | — |
| SSOT 目录名 | N/A | `adapters/storage/redis/` | path `crates/adapters/storage/redis` | **PASS**（约定） | README + Cargo path | 见 §5 路径裁决 |
| 版本号 | 未钉 | goal/alignment 写 `0.3.4` | **`0.3.5`** | **PARTIAL** | `Cargo.toml:4`；`CHANGELOG.md:3`；`docs/ssot/redisx-ssot-alignment.md:9` | 文档/SSOT 未同步 PATCH |
| `redis` crate | 0.27 起点 | 依赖评审锁定 | workspace `redis = 0.27` + cluster/sentinel/tls features | **PASS**（P0） | 根 `Cargo.toml:64-71` | 未来 major 需评审 |
| package stable | DoD 要求门禁全绿才可 | **OPEN 禁止宣称** | `publish = false` | **OPEN** | `Cargo.toml:9`；matrix S-9 | 误标 stable 为治理违规 |

---

## 2. Draft §2.2 crate 与 feature

| ID | MUST | Draft 期望 | SSOT | 实现 | 状态 | 证据 | 残差风险 |
|----|------|------------|------|------|------|------|----------|
| F-01 | `default = [runtime-tokio, tls-rustls]` | TLS 默认 feature | 生产 TLS 路径存在；默认明文允许 | `default = ["runtime-tokio"]` **无** `tls-rustls` feature 门 | **PARTIAL** | `Cargo.toml:11-17` | TLS 依赖在 workspace redis features 里常开，但 **配置默认 `tls=false`** |
| F-02 | `cluster` / `sentinel` feature 里程碑 | 独立 feature | 代码路径在默认图 | **无**独立 feature；Cluster/Sentinel 始终编译 | **PARTIAL** | `pool.rs:147-150`；Cargo 无 `cluster=` | 稳定面扩大；未完成能力仍可构造 |
| F-03 | `pubsub` feature | 可选 | 可选 Standalone | `pubsub = []` | **PASS** | `Cargo.toml:14`；`lib.rs:42-45` | — |
| F-04 | `metrics` feature | 指标门控 | 文档建议指标 | **无** metrics feature / 无导出 counter | **OPEN** | grep 无 metrics 实现 | 可观测缺口 |
| F-05 | `test-util` 禁止进默认图 | 隔离 | scaffold 隔离 | **无** `test-util`；`scaffold` 等价隔离 | **PASS**（语义） | `Cargo.toml:15`；`lib.rs:12-14` | — |
| F-06 | `scaffold` 改名/deprecated 防误用 | `InMemoryRedis` 等 | scaffold only | `InMemoryRedis`/`RedisAdapter` 在 scaffold 后 | **PASS** | `lib.rs:47-50`；`scaffold.rs` | 默认 feature 不含 scaffold |
| F-07 | 生产类型不得藏在模糊 `live` feature | 明确默认导出 | 默认生产面 | `live = []` 仅兼容空 feature；生产默认启用 | **PASS** | `Cargo.toml:16-17`；`lib.rs:3-9` | — |

---

## 3. Draft §2.3 公共 API

| ID | MUST | 状态 | 证据 | 残差风险 |
|----|------|------|------|----------|
| A-01 | `RedisConfig` 私有字段 + Builder | **PASS** | `config.rs:21-45,508+` | — |
| A-02 | `RedisPool` / `RedisClient` `Clone` 共享引用 | **PASS** | `pool.rs:93-97`；`client.rs:22-28` | — |
| A-03 | `connect` / `client` / `subscribe` / `ping` / `stats` / `close` | **PASS** | `pool.rs:145-254` | subscribe 需 `pubsub` |
| A-04 | `RedisPoolStats{open,in_flight,waiters}` | **PASS** | `pool.rs:24-33,211-218` | `open` P0 仅 0/1 单 lane |
| A-05 | `RedisMode::{Standalone,Cluster,Sentinel}` | **PASS** | `config.rs:9-19` | live 仅 Standalone 有证据 |
| A-06 | 配置含：端点/mode/用户密码/db/TLS/三超时/max并发/**预热**/重连 backoff/**TCP keepalive**/client name | **PARTIAL** | 有：addr/nodes/mode/user/pass/db/tls/3 timeouts/max_in_flight/client_name（`config.rs:23-44`）；**缺** warmup 数、reconnect backoff 配置、TCP keepalive、secret provider | ConnectionManager 内部重连不可配置/不可观测 |
| A-07 | Debug/错误/日志脱敏 | **PASS** | `config.rs:67-84,87-104,302-328` 单测脱敏 | — |
| A-08 | `RedisPubSub` Send；drop 取消 | **PARTIAL** | 结构体存在；stream take 后 drop 结束（`pubsub.rs:19-29,121-134`） | 无显式 cancel token / 后台任务监督证明 |
| A-09 | `#![deny(missing_docs)]` | **OPEN** | `lib.rs` 仅 `forbid(unsafe_code)` | 文档完整度门禁未钉 |

---

## 4. Draft §2.4 “连接池”真实语义

| ID | MUST | 状态 | 证据 | 残差风险 |
|----|------|------|------|----------|
| P-01 | N 可配置 multiplexed command lane | **PARTIAL** | 单 `RedisBackend` + `max_in_flight` Semaphore（`pool.rs:99-111,163`）；注释写明 **P0 单 lane**（`pool.rs:27`） | 高并发下单 lane 热点；非 draft N-lane |
| P-02 | 每 lane 自动重连 manager | **PARTIAL** | Standalone/Sentinel 用 `ConnectionManager`（`pool.rs:301-316`）；Cluster 用 `ClusterConnection` | 无恢复状态/指标；无 reconnect live |
| P-03 | Semaphore 背压 in-flight | **PASS** | `pool.rs:276-298` | — |
| P-04 | acquire 计入总 deadline（不可排除排队） | **PARTIAL** | acquire/command **分离** timeout（`pool.rs:266-273,281-297`）；**无**调用级总 deadline API | 排队+命令可叠加超调用方预算 |
| P-05 | Pub/Sub 不占命令 lane | **PASS** | 专用连接（`pubsub.rs:1,23-24`） | — |
| P-06 | 阻塞命令独立 blocking lane | **OPEN** | 无 BLPOP/XREAD BLOCK API | 误用命令 lane 会占满 in-flight |
| P-07 | Cluster MOVED/ASK 有界重定向 | **PARTIAL** | 底层 cluster client 处理；错误映射 MOVED/ASK→Transient（`error_map.rs:30-33`） | **无**本仓重定向次数上限配置/live |

---

## 5. Draft §2.5 合同与扩展

| ID | MUST | 状态 | 证据 | 残差风险 |
|----|------|------|------|----------|
| C-01 | `KeyValueStore::get` 缺失 `Ok(None)`；二进制安全；空值≠缺失 | **PASS** | `client.rs:101-118,403-411`；live KV | — |
| C-02 | `set` TTL `Some(0)` → Invalid（固定） | **PASS** | `client.rs:413-419`；`CHANGELOG` 0.3.2；live_ttl_zero | — |
| C-03 | 扩展：delete/exists/expire/ttl/mget/mset | **PASS** | `client.rs:179-398` | — |
| C-04 | `get_bytes/set_bytes` 命名扩展 | **PARTIAL** | 以 `Vec<u8>` 实现同等语义；无独立方法名 | 命名漂移，功能等价 |
| C-05 | `pipeline` / `script` | **OPEN** | 无公共 API | P2 |
| C-06 | 分布式锁 + fencing token + Lua CAD/CAE | **OPEN** / 运维文档列未闭合 | `docs/operations.md:75` | P2；误用 SETNX 无 fencing 高危 |
| C-07 | `PubSub` 合同 + 断连不可静默吞 | **PARTIAL** | `RedisPubSubFacade: PubSub`（`pubsub.rs:174+`）；stream 无 `Result` item | 断连可能结束流而无错误事件 |
| C-08 | 生产 `RedisMessageStream<Item=XResult<…>>` | **OPEN** | 仅 `BoxStream<BusMessage>`（`pubsub.rs:121`） | 合同层无法区分断连 |
| C-09 | `BusMessage.id` 会话内单调序号 | **PASS** | `AtomicU64`（`pubsub.rs:27,128-130`） | 跨会话不唯一（已文档化预期） |

---

## 6. Draft §2.6 并发 / 超时 / 重试

| ID | MUST | 状态 | 证据 | 残差风险 |
|----|------|------|------|----------|
| R-01 | 全异步；禁止 std Mutex 跨 await（生产路径） | **PASS** | 生产用 Semaphore/原子；Mutex 仅 scaffold/test | scaffold 误用风险由 feature 隔离 |
| R-02 | acquire/command 独立 + 调用级总 deadline | **PARTIAL** | 双 timeout PASS；总 deadline **OPEN** | 见 P-04 |
| R-03 | 仅幂等自动重试；INCR/Lua/不透明写默认不重试 | **PASS**（已实现面） | client 参数化 safety（alignment 表）；无 INCR/Lua API | 粗粒度 `RedisOperation::Set=AmbiguousWrite` 与 client 细分并存 |
| R-04 | 相对 TTL SET / DEL / PEXPIRE 多试拒绝 | **PASS** | `client.rs:144-175`；resilience 合同；gate 阻断 | — |
| R-05 | PUBLISH 不自动重试 | **PASS** | `NeverAutomatic`；alignment REDISX-13 | — |
| R-06 | 指数退避 + full jitter + 上限 | **PARTIAL** | 委托 `resiliencx` budget；无 redis 专用 backoff 旋钮 | 依赖 resiliencx 默认 |
| R-07 | `close` 后新请求失败；排空 in-flight | **PASS** | `pool.rs:226-241,263-265`；live_close | Pub/Sub 不纳入 in-flight 计数（spec 已声明） |

---

## 7. Draft §2.7 错误映射

| 场景 | Draft `ErrorKind` | 实现 | 状态 | 证据 |
|------|-------------------|------|------|------|
| 非法配置/TTL | Invalid | Invalid | **PASS** | `error_map.rs:14-16`；TTL 校验 |
| 要求存在的扩展缺失 | Missing | NoScript→Missing；key 缺失多返回 `Ok(false/None)` | **PARTIAL** | `error_map.rs:35`；API 语义差异 |
| CAS/锁 owner | Conflict | ExecAbort→Conflict；**无 CAS API** | **PARTIAL** | `error_map.rs:34` |
| LOADING/TRYAGAIN/IO | Transient | PASS | **PASS** | `error_map.rs:18-20,38-40` + 单测 |
| 无节点/认证 | Unavailable | PASS | **PASS** | `error_map.rs:17,21-28` |
| 取消/关停 | Cancelled | close→**Unavailable**（非 Cancelled） | **PARTIAL** | `pool.rs:251,264,278` |
| 排队/命令超时 | DeadlineExceeded | PASS | **PASS** | `pool.rs:272,293` |
| 不变量 | Invariant | PubSub stream 二次 take | **PARTIAL** | `pubsub.rs:123` |
| 未分类 | Internal | PASS | **PASS** | `error_map.rs:59` |
| ClusterDown / MOVED / … | 见 SSOT 0.3.5 锚点 | 已单测锁定 | **PASS**（映射） | `error_map.rs:97-143`；`CHANGELOG 0.3.5` |

---

## 8. Draft §2.8 安全

| ID | MUST | 状态 | 证据 | 残差风险 |
|----|------|------|------|----------|
| S-01 | 生产默认 TLS；明文 opt-in | **OPEN** / 与 SSOT 诚实 | 默认 `tls: false`（`config.rs:56`）；TLS 构造 secure only | 明文默认可连生产，需运维强制 TLS |
| S-02 | secret provider；禁 serde/Clone 扩散 | **PARTIAL** | 密码在 Config 内 `Option<String>`；无 provider；Debug 脱敏 | 内存中明文密码；Clone Config 复制 secret |
| S-03 | 拒绝 insecure TLS | **PASS** | `from_url` / seed 校验（`config.rs:201-206,399-402`） | — |
| S-04 | ACL 最小权限（多用户建议） | **OPEN**（文档级） | env username 支持；无强制分用户 | 运维责任 |
| S-05 | Lua 固定 SHA；禁拼接 | **OPEN** | 无 script API | — |
| S-06 | key/channel/password 不进 metrics label | **PASS**（无违规实现） | 尚无 metrics；ops 文档禁止 | 未来加指标时需守门 |

---

## 9. Draft §2.9 可观测性

| ID | MUST | 状态 | 证据 | 残差风险 |
|----|------|------|------|----------|
| O-01 | 低基数指标（请求/延迟/排队/重连/重定向…） | **OPEN** | 仅 `stats()` 快照；`operations.md:18-21` 为建议 | 生产排障盲区 |
| O-02 | tracing span 传播、不记 payload | **OPEN** | 生产路径无 `tracing::instrument` | — |
| O-03 | liveness / readiness / diagnostics 三级 | **PARTIAL** | docs 定义；`ping`+`stats` 可拼 readiness；**无**一等 API | 调用方自建 |

---

## 10. Draft §2.10 测试与验证

| ID | MUST | 状态 | 证据 | 残差风险 |
|----|------|------|------|----------|
| T-01 | 单元：配置/脱敏/错误/deadline/重试/关闭 | **PASS** | config/error_map/client/pool 单测；offline 49/54 | — |
| T-02 | 合同 suite KV/PubSub | **PARTIAL** | live_kv_conformance + pubsub live；默认 ignore | 非默认 CI 全绿依赖 |
| T-03 | 集成：standalone/ACL/TLS/Cluster/Sentinel 容器 | **PARTIAL** | Standalone live **PASS**；ACL 专项 SSOT **OPEN**；TLS/Cluster/Sentinel live **OPEN** | 拓扑假绿风险 |
| T-04 | 故障：kill/restart/主从/丢包/DNS/池耗尽 | **OPEN** | 仅 connect_refused 离线；无 chaos live | P1 |
| T-05 | loom / 取消泄漏 | **OPEN** | 无 loom | — |
| T-06 | 基准 多并发/多 payload + p99 门槛 | **PARTIAL** | `benches/kv_hot_path.rs` 有界；无 CI 回归阈值锁 | offline 日志：无 auth 时 bench skip |
| T-07 | Miri/ASan/deny/MSRV/feature 矩阵为发布门禁 | **PARTIAL** | workspace 有 deny/clippy；crate 无独立 miri | package stable 前必须齐 |

---

## 11. Draft §2.11 / §2.12 交付阶段与 DoD

### 11.1 阶段门（P0–P4）

| 阶段 | Draft 内容 | 状态 | 说明 |
|------|------------|------|------|
| **P0** | 配置、Pool、单机 KV、错误/超时/**指标**、真实集成 | **PARTIAL** | 核心 API+超时+Standalone live **齐**；指标/总 deadline/多 lane/默认 TLS **缺口** |
| **P1** | 专用 Pub/Sub、优雅关闭、故障注入、24h soak | **PARTIAL** | Pub/Sub Standalone + close **有**；故障注入/soak **OPEN**；Pub/Sub 重连 **NO-GO** |
| **P2** | pipeline/Lua/锁/Streams | **OPEN** | 明确未做（operations 未闭合列表） |
| **P3** | Cluster | **PARTIAL** | 代码路径+拒绝连接测试；**真实 Cluster live OPEN** |
| **P4** | Sentinel | **PARTIAL** | `async_master_for` 路径；**failover live OPEN** |

### 11.2 Definition of Done（draft §2.12）

| DoD 项 | 状态 | 证据 |
|--------|------|------|
| 默认生产入口无 scaffold | **PASS** | default features |
| README 最小/安全/故障示例 | **PARTIAL** | README 有最小示例；故障演练不全 |
| 公共 API 文档 + deny missing_docs | **PARTIAL** | 模块/类型有中文 docs；无 deny lint |
| 测试矩阵+基准阈值+安全审计+许可证门禁 | **PARTIAL** | 离线/部分 live；无完整发布审计包 |
| 升级/回滚/CHANGELOG/SemVer/运行手册 | **PARTIAL** | CHANGELOG+operations 有；无完整 rollback runbook |
| 故障演练：断线恢复/池耗尽/关停不丢已确认写 | **OPEN** | 无正式演练证据 |

---

## 12. SSOT Matrix 对照（权威本仓合同）

> 来源：`.agents/ssot/adapters/storage/redis/matrix/matrix.md` + `docs/ssot/redisx-ssot-alignment.md`  
> 与 draft **全量**不对齐是预期：SSOT 是 **本仓可宣称** 合同，draft 是 **上限愿景**。

| SSOT ID | 条款 | SSOT 状态 | 实现复核 | 备注 |
|---------|------|-----------|----------|------|
| S-1 / REDISX-1 | workspace member | PASS | **PASS** | `cargo metadata -p redisx` |
| S-2 / REDISX-2 | 生产默认导出 | PASS | **PASS** | `lib.rs` |
| S-3 / REDISX-3 | from_env | PASS | **PASS** | `FOUNDATIONX_REDISX_*` + `REDIS_URL` |
| S-4 / REDISX-4 | 离线测试 | PASS | **PASS** | 49/54 offline |
| S-5 / REDISX-5 | live ignore 入口 | PASS | **PASS** + 本环境已跑绿 ignored | 仍保持 ignore 防 CI 外依赖 |
| S-6 / REDISX-6 | bench 有界 | PASS | **PASS** | `kv_hot_path` |
| S-7 / REDISX-7 | crate docs | PASS | **PASS** | usage/config/operations |
| S-8 / REDISX-8 | SSOT 11 层 | PASS | **PASS** | 树完整 |
| S-9 / REDISX-9 | package stable | OPEN | **OPEN** | 禁止宣称 |
| S-10 / REDISX-10 | Cluster | OPEN | **OPEN** | 代码≠live |
| S-11 / REDISX-11 | Sentinel | OPEN | **OPEN** | 无 failover live |
| S-12 / REDISX-12 | TLS live | OPEN | **OPEN** | secure 构造 PASS |
| S-13 / REDISX-14 | Pub/Sub 配置同源 | PASS | **PASS** | pool 存 config；非 SA fail-closed |
| S-14 / REDISX-13 | 重试/原子性 | PASS | **PASS** | 参数化 safety |
| REDISX-15 | Pub/Sub 重连/必达 | OPEN | **NO-GO**（能力） | Redis Pub/Sub 天然不可靠；实现亦无重订阅 |
| REDISX-16 | 种子 URL 脱敏 | PASS | **PASS** | 负向单测 |

**判定**：实现与 **SSOT 当前合同** 对齐；与 **draft 全量** 差距大。不得用 draft 100% 反推本仓 ship。

---

## 13. 实现能力速查（相对 draft 扩展表）

| 能力 | 状态 | 路径 |
|------|------|------|
| Standalone KV + pool + close | **PASS** | `pool.rs` / `client.rs` / live |
| Cluster 命令连接代码 | **PARTIAL** | `connect_cluster`；live OPEN |
| Sentinel 发现 master 代码 | **PARTIAL** | `connect_sentinel`；failover OPEN |
| TLS secure 构造 | **PARTIAL** | `TcpTls{insecure:false}`；握手 live OPEN |
| resiliencx budget 路由 | **PASS** | `client.rs` + `resilience.rs` |
| Pub/Sub Standalone | **PARTIAL** | feature；无可靠投递 |
| scaffold 内存 | **PASS**（非生产） | feature scaffold |
| Streams / 锁 / pipeline / Lua | **OPEN** | — |
| 多 lane / warmup / keepalive 配置 | **OPEN** | — |
| 内置 metrics / tracing | **OPEN** | — |

---

## 14. 裁决问题回答

### 14.1 SSOT 是否应迁到 `redisx/` 路径？

**结论：保持 `.agents/ssot/adapters/storage/redis/`（package 名 `redisx`），不需要 `redisx/` 物理路径。**

| 理由 | 说明 |
|------|------|
| 布局 SSOT | 与 storage×7 一致：`adapters/storage/{redis,postgres,kafka,…}` 按 **协议/后端目录** 命名 |
| package 映射 | 目录 `redis` → crate `redisx` 已在 README/alignment/Cargo 全链路绑定 |
| 迁移成本 | 改路径会打断 `cmp` 双镜像、对齐文档、历史 PR 引用与 gate 脚本 |
| 何时才改 | 仅当全 adapters 统一改为 package 名目录（全局 RFC），禁止单点改 redisx |

### 14.2 现实 P0 生产 bar vs draft 100%

| Bar | 包含 | 本仓状态 | 可否宣称 |
|-----|------|----------|----------|
| **现实 P0（推荐 ship bar）** | Standalone 默认客户端；Config/env；Semaphore 背压；双 timeout；错误映射；脱敏；KV 扩展；可选安全重试；scaffold 隔离；离线 CI 绿；Standalone live 可跑；文档诚实 OPEN | **已基本满足** | 可宣称 **「生产默认 Standalone KV 客户端」**；**不可** package stable |
| **SSOT 当前合同** | 上表 + Cluster/Sentinel/TLS/PubSub 重连 OPEN/NO-GO 诚实 | **对齐** | 以 matrix 为准 |
| **Draft 100%** | N-lane 池、默认 TLS、secret provider、全量指标 tracing、ACL/TLS/Cluster/Sentinel 容器矩阵、故障注入+soak、pipeline/Lua/锁/Streams、调用级 deadline、deny missing_docs、完整 DoD 演练 | **远未达成** | **禁止**用 draft DoD 标 Done |

**推荐措辞（对外/PR）**：

> redisx `0.3.5`：Standalone 生产默认 `RedisPool`/`RedisClient` 已落地并通过离线测试与 Standalone live；Cluster/Sentinel/TLS 仅有代码路径与构造/拒绝连接测试，真实拓扑 live 仍 OPEN；Pub/Sub 仅 Standalone 且不承诺重连/必达；未宣称 package stable。draft SPEC_GOAL 为上限愿景，非本仓当前验收全集。

### 14.3 P0 残余缺口（若要把「现实 P0」收成严格 PASS）

按优先级（不扩 scope 到 P2+）：

1. **文档版本对齐** `0.3.4`→`0.3.5`（alignment/goal/README）— 低成本  
2. **Standalone 故障类 live**（进程重启恢复、池耗尽返回）— 增强 P0 信心  
3. **调用级总 deadline**（排队+命令）— draft §2.4/2.6 硬语义  
4. **最小 readiness API 或示例** + 可选低基数 metrics hook — draft §2.9 缩水版  
5. **默认 TLS 政策**：保持默认明文但文档/校验在 `mode=prod` 警告，或配置 profile — 与 draft 冲突需显式 ADR  

**明确不进 P0**：Cluster/Sentinel live、Pub/Sub 可靠投递、pipeline/锁/Streams、24h soak、loom。

---

## 15. 总表：Draft MUST 密度总览

| 章节 | PASS | PARTIAL | OPEN | NO-GO |
|------|------|---------|------|-------|
| §2.2 features | 3 | 3 | 1 | 0 |
| §2.3 API | 5 | 3 | 1 | 0 |
| §2.4 pool | 2 | 4 | 1 | 0 |
| §2.5 contracts | 4 | 3 | 2 | 0 |
| §2.6 concurrency | 4 | 2 | 0 | 0 |
| §2.7 errors | 5 | 4 | 0 | 0 |
| §2.8 security | 2 | 1 | 3 | 0 |
| §2.9 observability | 0 | 1 | 2 | 0 |
| §2.10 tests | 1 | 3 | 3 | 0 |
| §2.11 phases | 0 | 3 | 1 | 0* |
| §2.12 DoD | 1 | 3 | 1 | 0 |
| SSOT matrix 对齐 | 13 | 0 | 4 | 1 (REDISX-15 能力) |

\*P1 Pub/Sub 可靠投递记 **NO-GO**（协议+实现双否）。

---

## 16. 风险与治理

| 风险 | 级别 | 缓解 |
|------|------|------|
| 将 draft 100% 误认为已 ship | P0 | 以 SSOT matrix + alignment 为宣称边界 |
| Cluster/Sentinel 配置可连但无 live → 假生产 | P1 | 文档/gate：无拓扑证据不得 PASS |
| 默认明文 TLS | P1 | 运维强制 `TLS=true`；后续 ADR |
| 版本文档漂移 0.3.4/0.3.5 | P2 | 同步 alignment/goal/README |
| unchecked `with_budget*` 兼容 API 被误用 | P1 | README 已警告；新代码只用 `*_safe` |
| Pub/Sub stream 吞断连 | P1 | 标 NO-GO；需要则上 Streams |

---

## 17. 源路径索引

| 角色 | 绝对路径 |
|------|----------|
| Draft 快照 | `/home/workspace/infra.rs/.worktrees/feat/redisx-p0-close/.agents/ssot/adapters/storage/redis/plan/infra-rs-draft-spec-goal.md` |
| SSOT README | `…/adapters/storage/redis/README.md` |
| Spec | `…/adapters/storage/redis/spec/spec.md` |
| Matrix | `…/adapters/storage/redis/matrix/matrix.md` |
| Goal | `…/adapters/storage/redis/goal/goal.md` |
| 实现 | `…/crates/adapters/storage/redis/` |
| Alignment | `…/docs/ssot/redisx-ssot-alignment.md` |
| Offline/Live 日志 | `/tmp/grok-goal-977017128a45/implementer/redisx-*.log` |

---

**报告版本**：gap-matrix-v0  
**作者角色**：explore（只读）  
**下一步建议**（非本任务执行）：(1) 文档版本钉到 0.3.5；(2) 冻结「现实 P0」验收清单；(3) Cluster/Sentinel/TLS 各自独立 live epic，禁止混入 P0 close。
