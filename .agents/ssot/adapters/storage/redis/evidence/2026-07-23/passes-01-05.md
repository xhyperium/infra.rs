# redisx Critic Review — Passes 01–05 / 10

| 字段 | 值 |
|------|-----|
| 审查角色 | critic / reviewer |
| worktree | `/home/workspace/infra.rs/.worktrees/feat/redisx-p0-close` |
| draft 来源 | `.agents/ssot/adapters/storage/redis/plan/infra-rs-draft-spec-goal.md`（原 `.cargo/draft/redisx_SPEC_GOAL.md` 入库快照；worktree 内 `.cargo/draft/` 不存在） |
| 实现 | `crates/adapters/storage/redis/src/{lib,client,config,pool,pubsub,error_map,resilience}.rs` |
| 测试 | `tests/live_kv.rs` · `live_kv_conformance.rs` · `live_pubsub_conformance.rs` |
| 对齐 | `docs/ssot/redisx-ssot-alignment.md` |
| gap-matrix-v0 | **未找到**（`/tmp/grok-goal-977017128a45/implementer/reviews/gap-matrix-v0.md` 不存在） |
| 日期 | 2026-07-23 |
| 范围 | Pass 1–5 only；不含 6–10 |

> 原则：每 pass 只写**本轮新发现**或 `no new findings`；状态纠正须带 `path:line`。  
> 不粘贴任何 secret / 完整凭据。

---

## 总览（对照 draft 阶段诚实边界）

- Draft §2.11 将 pipeline/Lua/锁/Streams/Cluster live 等放在 P2–P4；本批以 **P0 生产默认**（配置 + Pool + 单机 KV + 错误/超时/背压 + live ignore 入口）为主验收。
- 实现已具备：`RedisPool`/`RedisClient`/`RedisConfig*`、Semaphore 背压、错误映射、resiliencx 安全路由、Pub/Sub（Standalone + fail-closed 拓扑）、Debug/endpoint 脱敏、TLS secure 构造路径。
- 不得将 Cluster/Sentinel/TLS **真实 live**、package stable、N-lane 池、调用级总 deadline、secret provider 等未闭合项宣称为已交付。

---

### Pass 1: API / export surface

#### Residual findings（新）

1. **Feature 矩阵与 draft §2.2 漂移（诚实边界，非 P0 阻断）**  
   - Draft 期望：`default = [runtime-tokio, tls-rustls]`，另有 `cluster` / `sentinel` / `metrics` / `test-util` 独立 feature。  
   - 实现：`Cargo.toml:11-17` 仅 `default=["runtime-tokio"]` + `pubsub` + `scaffold` + 兼容空 `live`；Cluster/Sentinel **始终编入** `pool.rs`，无 feature 门控。  
   - 影响：调用方无法在编译期排除 Cluster/Sentinel 依赖面；与 draft「Cluster/Sentinel 作为独立 feature/里程碑」不完全一致。当前 active 合同（`spec/spec.md:8-10`）已接受「三种模式均有代码路径」，但 **不得**把「编入」等同「live 闭合」。

2. **公开导出 unchecked 重试包装，存在误用面**  
   - `lib.rs:32-37` 导出 `with_budget` / `with_budget_async` / `with_budget_async_noop` 等 **不校验 `RetrySafety`** 的兼容 API。  
   - `resilience.rs:120-123,173-176` 文档标明 unchecked，但 crate 根默认可见。  
   - 生产客户端路径（`client.rs` → `with_budget_async_safe_noop` / `with_automatic_budget`）正确；残差是 **库消费者可绕过安全路由**，与 draft §2.6「只对已知幂等操作自动重试」精神冲突。  
   - 建议（follow-up，非本批必改）：`#[deprecated]` 或降为 `pub(crate)` / feature `compat`，或 rustdoc 置顶警告。

3. **Draft §2.3 / §2.5 扩展面仍缺（P2+ 预期，须保持 OPEN）**  
   - 已落地 KV 扩展：`delete/exists/expire/ttl/mget/mset`（`client.rs`）。  
   - 仍缺：`pipeline` / `script` / CAS / 分布式锁原语 / `RedisMessageStream<Item=XResult<BusMessage>>`（draft §2.5：合同 stream 无 `Result`，断连需扩展流）。  
   - `pubsub.rs:121-134` 的 `into_message_stream` 仅产出 `BusMessage`，断连事件会被 `filter_map` 静默消化——与 draft「断连事件不得被静默吞掉」有差距；若 P0 不交付扩展流，**对齐文档必须继续 OPEN**（REDISX-15 已 OPEN，维持）。

4. **配置字段相对 draft §2.3 不完整（P1 诚实项）**  
   - 已有：addr/nodes/mode/username/password/db/tls/connect|command|acquire timeout/max_in_flight/client_name（`config.rs:23-45`）。  
   - 缺：预热数、重连 backoff、TCP keepalive、secret provider（draft §2.3 配置 MUST 列表）。  
   - 不构成 P0 API 形状失败（Pool/Client/Stats/Mode 已导出），但不得在 README/对齐文中写「配置面已对齐 draft 全量」。

#### Status corrections

| 声称 | 纠正 | 证据 |
|------|------|------|
| 对齐文 `version = 0.3.4` | crate 实际 `0.3.5` | `Cargo.toml:4`；`spec/spec.md:3` 写 0.3.5；`docs/ssot/redisx-ssot-alignment.md:9` 仍 0.3.4 |
| 生产默认导出齐全（P0） | **通过**（核心类型） | `lib.rs:29-45`：`RedisClient/Config/Builder/Mode/Pool/PoolStats/error_map/resilience`；`pubsub` feature 下 `RedisPubSub*` |
| draft 要求的 pipeline/script/锁 | **非本批交付** | draft §2.11 P2；实现无对应 `pub` API |

#### Verdict (Pass 1)

P0 导出面 **基本达标**；残差为 feature 粒度、unchecked 重试导出、P2 扩展与配置字段清单。对齐文档 version 行需修正。

---

### Pass 2: deadline / backpressure

#### Residual findings（新）

1. **无「调用级总 deadline」；acquire 与 command 时间分离累计可能超预期**  
   - Draft §2.4 / §2.6：`acquire` 与 `command` 独立配置，**同时**受调用级总 deadline 限制；「不得把等待 semaphore 的时间排除在总 deadline 之外」。  
   - 实现：`pool.rs:276-298` 仅 `acquire_timeout`；`pool.rs:268-273` 仅在获得 permit **之后** 对闭包套 `command_timeout`。  
   - 最坏墙钟 ≈ `acquire_timeout + command_timeout`，无单次调用总预算 API。  
   - 严重度：**P1 合同缺口**（行为有界但仍与 draft 字面不符）。若产品接受「两段独立超时」，须在 `docs/config.md` / SSOT 明确，避免调用方按「总 deadline」建模。

2. **池模型为单 lane，非 draft N 个 multiplexed lane**  
   - Draft §2.4：N 个可配置 command lane + 每 lane manager。  
   - 实现：`RedisPoolStats.open` 注释写明 P0 单 lane 0/1（`pool.rs:27-28,214`）；单 `RedisBackend` + 全局 Semaphore。  
   - 背压语义可用（`max_in_flight` 默认 256，`config.rs:61`），但 **不是** draft 描述的多 lane 资源池。对齐文应继续写「P0 单 lane + 全局 in-flight Semaphore」，禁止写「完整资源池模型已对齐 draft」。

3. **Pub/Sub publish 路径无池级 `with_conn` 外层超时包装**  
   - 命令路径：`pool.with_conn` → `timeout(command_timeout, …)`（`pool.rs:268`）。  
   - `RedisPubSub::publish`（`pubsub.rs:112-117`）与 facade `pub_message`（`pubsub.rs:176-181`）直接 `query_async`，依赖建连时 `ConnectionManagerConfig::set_response_timeout`（`pubsub.rs:77-79`）。  
   - 残差：publish **不计入** 池 in-flight/Semaphore（符合 draft「Pub/Sub 不占命令 lane」），但若底层 response_timeout 与配置漂移，缺少与 KV 对称的显式 `timeout(...)` 护栏。建议 follow-up 对 publish 再包一层 `timeout(cfg.command_timeout(), …)` 以双保险。

4. **close 排空仅轮询 in-flight，不取消底层 I/O**  
   - `pool.rs:227-241`：`closed=true` 后 sleep 5ms 轮询至 in-flight==0 或 deadline。  
   - 已在飞的命令仍跑满 `command_timeout`；无 abort/cancel token。  
   - Draft §2.6 close 语义（新请求拒绝 + deadline 内排空）**基本满足**；残差是无法强制打断慢 I/O（P2 可接受，记 OPEN 细项即可）。

#### Status corrections

| 声称 | 纠正 | 证据 |
|------|------|------|
| acquire/command 分离 | **通过** | `config.rs:39-41,260-276`；`pool.rs:109-110,268,282` |
| 池耗尽有界 | **通过** | acquire 超时 → `DeadlineExceeded`（`pool.rs:293-296`）；sem 关闭 → `Unavailable`（`pool.rs:292`） |
| 调用级总 deadline | **未实现** | 全 crate 无 per-call total deadline 参数 |
| 连接/命令超时进入 connect 路径 | **通过** | Standalone/Cluster/Sentinel 均有 `timeout(connect_timeout, …)`（`pool.rs:310-314,339-341,375-377,383-386`） |

#### Verdict (Pass 2)

背压与分段超时 **P0 可用且有界**；相对 draft 的「总 deadline 包含排队」与「N-lane」仍为诚实 OPEN。

---

### Pass 3: error mapping

#### Residual findings（新）

1. **`Cancelled` 几乎不被使用；close 后统一 `Unavailable`**  
   - Draft §2.7：调用取消/关停 → `Cancelled`；同时 §2.6 允许 `Cancelled/Unavailable`。  
   - 实现 close 后：`Unavailable("redis 连接池已关闭")`（`pool.rs:250-251,263-264,277-278,287-288`）。  
   - live 测试断言 `ErrorKind::Unavailable`（`tests/live_kv.rs:37-38,104-105`）。  
   - 残差：无协作式 cancel → `Cancelled` 的路径；与 draft 表项「Cancelled」列弱对齐。建议在文档固定「关停 = Unavailable；Cancelled 预留给上层 `select!/Drop`」——**非映射 bug**，属语义钉死缺口。

2. **超时不经 `map_redis_error`，而由池层直接 `deadline_exceeded`**  
   - `pool.rs:272` / acquire `pool.rs:293` / connect 超时路径。  
   - 与 draft §2.7「排队或命令超时 → DeadlineExceeded」**一致**，但若上层只对 `map_redis_error` 做单测，会漏掉超时分支。当前单元测覆盖了 refused connect 的 kind 集合（`pool.rs:412-418`），**未**离线单测「池满 acquire 超时」与「故意慢命令 command 超时」的精确 `DeadlineExceeded`（需 mock/probe 或 live）。  
   - 残差：**超时 kind 的针对性单测缺口**（P2 测试债）。

3. **认证失败统一 `Unavailable`，无 `Unauthenticated` 细分**  
   - `error_map.rs:17,41-42,51-52`：`AuthenticationFailed` / `NOAUTH` / `WRONGPASS` → `unavailable`。  
   - Draft 表：无可用节点/**认证依赖不可用** → Unavailable —— **字面符合**。  
   - 残差仅运维可观测性：认证与节点宕机同 kind，依赖消息字符串区分。可接受；勿宣称有独立 auth kind。

4. **未分类驱动错误落入 `Internal`（`error_map.rs:59`）**  
   - 符合 draft「未分类 → Internal（须计数并持续归零）」。  
   - 残差：crate 内 **无** Internal 错误计数/metrics（draft §2.9 可观测性 P1+）。记 OPEN，不阻塞 P0 映射正确性。

#### Status corrections

| 声称 | 纠正 | 证据 |
|------|------|------|
| LOADING/TRYAGAIN/IO → Transient | **通过** | `error_map.rs:18-19,38-40,53-54` + 单测 `76-80,122-126` |
| 认证 → Unavailable | **通过** | `error_map.rs:17,41-42` + 单测 `84-88,129-135` |
| ClusterDown → Unavailable | **通过** | `error_map.rs:21-28` + 单测 `98-101` |
| MOVED/ASK → Transient | **通过** | `error_map.rs:30-33` + 单测 `104-107` |
| ExecAbort → Conflict / NoScript → Missing | **通过** | `error_map.rs:34-35` + 单测 `110-119` |
| Invalid 配置 | **通过** | `error_map.rs:14-16`；TTL0 客户端 `client.rs:416` |
| PTTL 缺失 key → Missing | **通过** | `client.rs:286-288` |

#### Verdict (Pass 3)

§2.7 主表映射 **扎实**；残差为 Cancelled 语义钉死、超时单测、Internal 计数可观测性。

---

### Pass 4: live test honesty

#### Residual findings（新）

1. **对齐文 REDISX-4 计数过时**  
   - `docs/ssot/redisx-ssot-alignment.md:34`：`51 passed + 8 ignored`。  
   - 证据日志（implementer）：  
     - default features：`49 passed` unit + **7** ignored live（pubsub 集成测在无 feature 时 0 tests）（`/tmp/.../redisx-offline-default.log:160-185`）。  
     - `--features pubsub`：`54 passed` + **8** ignored（5+2+1）（`redisx-offline-pubsub.log:61-87`）。  
   - 纠正：应按 feature 矩阵分别写清，或统一写「默认 ~49 单测 + 7 ignore；pubsub +5 单测与 +1 ignore」。

2. **`live_kv_conformance` 名大于实**  
   - 文件头称 first-batch KeyValueStore 合同（`live_kv_conformance.rs:1-2`），但仅 2 个手写用例：get/set/missing 与 key 隔离（`19-49`），**未**调用 `contract-testkit` 全量 suite。  
   - 对比 pubsub：`live_pubsub_conformance.rs:8,17` 真实调用 `assert_pub_sub_surface`。  
   - 残差：命名暗示「conformance suite」，实为 L3 子集手写测。应对齐文写「KV L3 subset / 非全量 contract suite」，或接入 `contract-testkit` 正式 suite。

3. **`pool::tests::closed_pool_is_closed_flag` 静默跳过**  
   - `pool.rs:487-494`：`from_env` 或 connect 失败时 `return`，**不是** `#[ignore]`。  
   - 在无 Redis / 错误凭据的 CI 中记为 **passed 空跑**，夸大 close 状态机覆盖。  
   - 诚实建议：改为 `#[ignore]` 并入 live，或 connect 失败时 `return` 前 `eprintln!` 明确 skip（更稳妥：ignore）。

4. **Cluster / Sentinel / TLS live 诚实性 — 维持 OPEN，无夸大**  
   - 仅有离线 `*_connect_refused_returns_error`（`pool.rs:398-484`）与 TLS 构造测（`config.rs:792-804`）。  
   - `redisx-ssot-alignment.md:40-42` REDISX-10/11/12 OPEN —— **正确**。  
   - live 日志显示 ignored 入口在显式 `--ignored` 下可过 Standalone KV（`redisx-live-kv.log:4-11`，5 passed）——可证明 Standalone live 可达，**不得**外推 Cluster/TLS。

5. **ACL 专项 live 仍缺**  
   - `spec/spec.md:17` 已写「ACL 专项仍 OPEN」。  
   - live 使用 env 通用认证，无最小权限 / 错误 ACL 负向 live。维持 OPEN。

#### Status corrections

| 声称 | 纠正 | 证据 |
|------|------|------|
| live 默认 ignore | **通过** | `live_kv.rs:26,41,80,89,99`；conformance/pubsub 同 |
| REDISX-5 live 入口 PASS | **通过（入口存在）** | 三个 tests/* 文件；非「全拓扑 live 已绿」 |
| 51+8 测试账 | **过时** | offline-default 49+7；offline-pubsub 54+8 |
| package stable | **仍 OPEN** | alignment REDISX-9；Cargo `publish = false`（`Cargo.toml:9`） |

#### Verdict (Pass 4)

live **入口诚实**（ignore + Standalone 可跑）；残差为计数漂移、conformance 命名、close 单测静默 skip。禁止用 refused-connect / ignore 入口替代真实拓扑证据——当前文档立场正确。

---

### Pass 5: security / TLS / secrets Debug

#### Residual findings（新）

1. **生产默认 TLS 与 draft §2.8 相反**  
   - Draft：`生产默认 TLS；明文连接需显式 opt-in`。  
   - 实现：`tls: false` 默认（`config.rs:57`；`docs/config.md:12` `FOUNDATIONX_REDISX_TLS` 默认 false）。  
   - Feature 亦无 `tls-rustls` 默认开启（`Cargo.toml:12`）。  
   - 残差：**安全默认值政策冲突**。当前更像「本地/内网明文 opt-out TLS」。须在 SSOT/对齐明确「本仓偏离 draft：TLS 显式 opt-in」，或后续改为默认 true（可能破坏本地 dev 体验，需迁移说明）。

2. **`RedisConfig: Clone` 会克隆 password 明文进多份所有权**  
   - `config.rs:22` `#[derive(Clone)]`；密码在 `Option<String>`（`config.rs:33`）。  
   - Draft §2.8：`secret … 禁止 … Clone 到非必要对象`。  
   - 池在 pubsub feature 下保存完整 `RedisConfig`（`pool.rs:101-103,171-172`）；`subscribe` clone config（`pool.rs:253`）。  
   - 残差：无 secret provider / 零化类型；属 draft 全量安全模型未交付。P0 可接受但须 OPEN「凭据轮换 / secret provider」。

3. **username 在 Debug 中明文**  
   - `config.rs:73`：`username` 原样输出；仅 `password → ***`。  
   - 通常可接受（ACL 用户非 secret）；若组织政策把 username 当敏感，需加严。当前测试期望 username 可见（`config.rs:697`）。**非缺陷**，记观察项。

4. **seed URL / endpoint / 错误路径脱敏 — 强**  
   - `redact_seed_url`（`config.rs:88-104`）；Debug/endpoint/非法 URL 错误测（`config.rs:688-735`）。  
   - TLS：`insecure: false` 强制（`config.rs:335-337,201-206,399-402`）；`from_url` 拒绝 insecure；Cluster 节点 insecure 拒绝（`config.rs:656-658`）。  
   - `password_opt` 仅 `pub(crate)`（`config.rs:296-299`）。  
   - Pub/Sub 复用 ACL/TLS 且 Cluster/Sentinel fail-closed（`pubsub.rs:193-202` + 单测）。  

5. **`RedisClient`/`RedisPool` Debug 不泄密**  
   - `RedisPool` 自定义 Debug 仅 endpoint + stats + closed（`pool.rs:114-121`）。  
   - `RedisClient` `#[derive(Debug)]`（`client.rs:22`）依赖 pool 的 Debug；budget 无密钥。  
   - `RedisPubSub` Debug 仅 endpoint（`pubsub.rs:31-34`）。

#### Status corrections

| 声称 | 纠正 | 证据 |
|------|------|------|
| Debug 脱敏 password | **通过** | `config.rs:67-84,688-697` |
| 种子 URL 脱敏 | **通过** | `config.rs:715-735`；alignment REDISX-16 |
| TLS insecure 拒绝 | **通过** | `config.rs:201-206,792-804` |
| 真实 TLS 握手 | **OPEN** | 无 live；alignment REDISX-12 |
| 生产默认 TLS | **未实现（明文默认）** | `config.rs:57` vs draft §2.8 |
| secret provider | **未实现** | 配置仅 `Option<String>` 密码 |

#### Verdict (Pass 5)

P0 脱敏与 insecure 拒绝 **硬通过**；相对 draft 的「默认 TLS + secret provider」仍 OPEN。不得宣称安全模型完整对齐 draft §2.8。

---

## Pass 1–5 汇总

| Pass | 主题 | 结论 | 阻塞 P0？ |
|------|------|------|-----------|
| 1 | API/export | 核心面达标；unchecked 重试导出与 feature/扩展缺口 | 否（follow-up） |
| 2 | deadline/backpressure | 分段超时+Semaphore 有界；无总 deadline；单 lane | 否（须诚实文档） |
| 3 | error mapping | §2.7 主路径扎实 | 否 |
| 4 | live honesty | ignore 诚实；计数/conformance 命名/静默 skip 需修文档或测 | 文档/测试债 |
| 5 | security/TLS/Debug | 脱敏+secure TLS 构造 OK；默认明文 TLS、无 secret provider | 政策 OPEN，非实现回归 |

### 建议优先修正（不扩 scope 编码）

1. **文档**：`redisx-ssot-alignment.md` version → `0.3.5`；测试账按 feature 更新；明确 TLS 默认明文偏离 draft。  
2. **文档**：写清「总墙钟 = acquire + command」与「P0 单 lane」。  
3. **测试债**：`closed_pool_is_closed_flag` 改为 ignore 或明确 skip 日志；可选补 acquire/command 超时单测。  
4. **API 卫生（follow-up）**：收敛 unchecked `with_budget*` 可见性。  

### 明确不得宣称

- package stable / crates.io  
- Cluster / Sentinel / TLS **真实 live** 通过  
- draft 全量 feature 矩阵、N-lane 池、调用级总 deadline、secret provider、pipeline/Lua/锁  
- Pub/Sub 可靠投递或断线恢复  

---

## 证据索引（本批主要路径）

| 区域 | 路径 |
|------|------|
| draft | `.agents/ssot/adapters/storage/redis/plan/infra-rs-draft-spec-goal.md` |
| active 合同 | `.agents/ssot/adapters/storage/redis/spec/spec.md` |
| 导出 | `crates/adapters/storage/redis/src/lib.rs` |
| 配置/脱敏/TLS | `…/src/config.rs` |
| 池/超时/背压 | `…/src/pool.rs` |
| 错误映射 | `…/src/error_map.rs` |
| 客户端/重试细分 | `…/src/client.rs` · `…/src/resilience.rs` |
| Pub/Sub | `…/src/pubsub.rs` |
| live | `…/tests/live_*.rs` |
| 对齐 | `docs/ssot/redisx-ssot-alignment.md` |
| 离线日志 | `/tmp/grok-goal-977017128a45/implementer/redisx-offline-{default,pubsub}.log` |

---

*Pass 01–05 完成。Pass 06–10 不在本文件范围。*
