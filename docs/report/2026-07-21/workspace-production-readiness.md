# Workspace 模块生产就绪度二次深审补充报告

> **post-#178 勘误（2026-07-22）**：独立 `contract-testkit` **已交付**；Fake 不在 contracts 生产 API；postgres scaffold 为 `ScaffoldTxContext`（非 FakeTxContext）。正文历史句若写「未交付 / FakeTx」以本勘误为准。

| 字段 | 值 |
|---|---|
| 审计日期 | 2026-07-21 |
| 审计范围 | `STATUS.md` 列出的 21 个 Cargo workspace member |
| 审计基线 | commit `917484062dfeb4d5bf03e8a82d3252ab22f6edf2` |
| 主报告 | [`status-modules-production-readiness.md`](./status-modules-production-readiness.md)（PR #156） |
| 本报告定位 | 在主报告基础上独立复核，并补充可复现的正确性、安全、资源与治理阻断；不替代主报告 |
| 审计后主线变化 | PR #157 新增 12 个 core crate 的 hot-path bench、公开面测试与 API 文档；改善非功能证据，但不改变本报告的生产阻断结论 |
| 任务 | Beads `infra-sxq` |
| 环境 | Linux x86_64；Rust/Cargo 1.97.0；cargo-deny 0.20.2；Node.js 24.14.0 |
| 报告性质 | 当前源码的生产就绪候选审计；不是 Maintainer 发布批准或稳定性承诺 |

## 1. 结论先行

**当前 workspace 不能作为一个整体宣称满足生产级应用标准。**

`STATUS.md` 的 92% 是结构与可观察进度，不是生产成熟度。它将目录布局计为 50%，任意测试入口计为 25%，剩余部分主要来自 LOC、示例和 README；不评价真实后端、协议正确性、恢复、安全、资源边界、性能、发布责任或实际运行记录。文件自身也在 `STATUS.md:7` 明确排除了 Production Ready 含义。

本次二次审计没有发现可无条件签字为 Production Ready 的模块。相对成熟度如下：

- `kernel`、`decimalx`、`canonical` 的 committed 子集，以及 `testkit` 的 ManualClock 测试支持，具备较强的窄范围证据，但都有明确限制或阻断；只能按角色写成“有条件使用”。
- `configx`、`schedulex`、`bootstrap`、`evidence`、`observex`、`resiliencx`、`transportx` 和 `contracts` 有可用的最小实现或接口，但不足以承担其典型生产角色。
- 9 个 adapter 都未达到生产适配器门槛。7 个存储 adapter 没有相应外部系统客户端；两个交易所 adapter 虽可注入真实 HTTP driver，但核心协议仍是占位或 mock-first 语义。
- 关键生产链没有闭环：没有真实配置加载、真实 adapter、可替换的完整 contract、持久审计、可导出 observability、受 deadline 约束的启动/停止与故障恢复组合证据。

### 1.1 逐模块总判定

“当前窄实现”评价代码诚实声明的最小范围；“典型生产角色”评价模块名称通常代表的生产职责。

| 模块 | STATUS | 当前窄实现 | 典型生产角色 | 主要硬阻断 |
|---|---:|---|---|---|
| `kernel` | active / 98% | 有条件就绪 | 有条件就绪 | `Duration::MAX` timeout 可 panic；ClockDomain 可伪造；SSOT/package 漂移 |
| `testkit` | active / 98% | 有条件就绪（测试基础设施） | 不适用（不是 runtime） | 仅 ManualClock core；不得外推为生产运行时或完整 contract-testkit |
| `canonical` | active / 98% | committed 子集有条件使用 | 部分就绪 | 无 envelope；“N-1”多为首次冻结的同字段 fixture；资源上限与完整 wire 未闭合 |
| `decimalx` | active / 98% | checked 路径有条件使用 | 部分就绪 | `i128::MIN` 文本不往返；保留 panicking API；门禁为启发式扫描；无性能基线 |
| `bootstrap` | active / 100% | typed composition 部分就绪 | 未就绪 | release 下 required evidence 可绕过；无真实 app start/ready/rollback/drain/deadline |
| `configx` | partial / 90% | 内存 KV 有限可用 | 未就绪 | 无配置源/schema/snapshot/LKG/hot reload/secret；读毒锁静默当缺失 |
| `evidence` | partial / 88% | 开发/测试可用 | 未就绪 | 无 durable append、hash chain、幂等/CAS、恢复、签名与独立锚 |
| `observex` | partial / 88% | 最小 tracing adapter 部分就绪 | 未就绪 | 默认无 subscriber 可静默丢弃；subscriber panic/阻塞可穿透；无 exporter/flush/metrics |
| `resiliencx` | active / 98% | 同步原语部分就绪 | 未就绪 | 同步 sleep、无 deadline/cancel/budget；熔断/限流语义非生产化；错误分类不贯通 |
| `schedulex` | partial / 88% | 内存 ID registry 可用 | 未就绪 | 没有 timer/job/executor/persistence/recovery/shutdown/lease |
| `transportx` | active / 95% | loopback/受控集成部分就绪 | 未就绪 | Debug 泄漏请求；WS 无 timeout；响应/帧无界；TLS/错误/观测闭环不足 |
| `contracts` | active / 98% | 接口骨架部分就绪 | 未就绪 | 事务无法绑定仓储操作；stream 无运行期错误；无真实后端 conformance；SSOT 漂移 |
| `binancex` | scaffold+mock / 89% | scaffold + mock seam | 未就绪 | place/cancel/query 有假成功/占位；无签名、鉴权、WS 恢复、幂等、对账 |
| `okxx` | scaffold+mock / 89% | scaffold + mock seam | 未就绪 | 同上；私有 REST 认证、结构化 body、业务错误码均未实现 |
| `clickhousex` | scaffold / 83% | 接口烟测 | 未就绪 | 内存 Vec；无 ClickHouse client、schema、batch/flush、持久化与恢复 |
| `kafkax` | scaffold+mock / 89% | scaffold + mock | 未就绪 | 内存快照流；并发 ID 可重复；无 broker/ack/offset/group/rebalance/security |
| `natsx` | scaffold+mock / 88% | scaffold + mock | 未就绪 | 内存快照流；并发 ID 可重复；未裁定 Core NATS/JetStream 与 durable 语义 |
| `ossx` | scaffold / 83% | 接口烟测 | 未就绪 | 内存 HashMap；无 OSS/S3 SDK、校验和、multipart、鉴权与大对象边界 |
| `postgresx` | scaffold+mock / 89% | scaffold + mock | 未就绪 | 内存 HashMap；ScaffoldTx 与 Repository 写入无关联；无 SQL/pool/migration/isolation |
| `redisx` | scaffold+mock / 89% | scaffold + mock | 未就绪 | 主 adapter 静默忽略 TTL；PubSub 语义错误；无 Redis client/cluster/auth/failover |
| `taosx` | 被误标 scaffold+mock / 88% | pure scaffold | 未就绪 | 实际无 mock；内存 Vec；无 TDengine client、schema、retention、恢复 |

### 1.2 Workspace 级最终判定

整体判定采用关键路径最弱项原则，而不是 21 个分数的平均值：

```text
真实输入
  → configx（无生产配置加载/快照）
  → bootstrap（无完整启动、ready、rollback、drain）
  → contracts（事务/stream 语义不闭合）
  → adapters（0/9 真实生产后端）
  → observex/evidence（无生产导出与 durable audit）
```

任一关键节点都足以阻断整体 Production Ready；当前同时存在多处阻断。

## 2. 审计方法

### 2.1 证据优先级

从高到低使用以下证据：

1. 当前 commit 的源码、Cargo manifest 与 `cargo metadata`。
2. 本次可复现命令及退出码。
3. 直接验证行为的语义、并发、wire、loopback 或 conformance 测试。
4. workflow 定义；只有取得当前 run 记录时才视为运行证据。
5. 本仓 SSOT 对齐文、README、治理文档和历史签核。
6. `STATUS.md` 与只读镜像，仅用作范围和声明参考。

源码与文档冲突时，以源码和本次运行结果为准。没有证据的维度标为 Unknown，不因“未发现 bug”而判 PASS。

### 2.2 八个审计维度

| 维度 | 核验内容 |
|---|---|
| 能力与边界 | 声明能力是否真实存在；模块名称是否高于当前实现 |
| 正确性与不变量 | panic、静默错误、并发、事务、顺序、幂等、时间与数值边界 |
| API/wire/数据兼容 | API baseline、版本、schema、fixture、迁移、回滚与 package 身份 |
| 可靠性与恢复 | timeout、deadline、取消、重试、背压、故障隔离、恢复与 shutdown |
| 安全 | unsafe、依赖供应链、TLS/鉴权、secret/PII、输入与资源耗尽边界 |
| 可观测与运维 | 稳定错误、trace/metric/audit、健康、flush、runbook 与故障定位 |
| 性能与容量 | benchmark、SLO、延迟/吞吐、内存上限、长稳与资源模型 |
| 验证与治理 | 测试深度、持续门禁、MSRV、文档一致性、发布与人类签核 |

不使用加权总分。关键合同错误、真实后端缺失、安全泄漏或资源无界不能被高覆盖率抵消。

### 2.3 角色化硬门槛

- 核心类型必须证明不变量、无未声明 panic、数值边界、API/wire 演进。
- `testkit` 只按生产级测试基础设施评价，不能按生产 runtime 加减分。
- L1 组件同时评价其明确的窄合同和名称代表的典型生产能力。
- `contracts` 必须有唯一且可替换的语义，以及真实实现 conformance。
- adapter 必须有真实 I/O、鉴权/TLS、错误映射、幂等/一致性、恢复和真实集成证据；mock 只证明接口接线。

## 3. 为什么 STATUS.md 的 92% 不能外推

### 3.1 公式只评价工程结构

`STATUS.md:30-36` 的公式是：

```text
completion = layout(8项)×50% + has_tests×25% + content×25%
content    = LOC 桶 + 可运行 example + docs/README 实质
```

它产生的有效结论是：21 个 member 都有标准目录、都有某种测试入口，且若干 crate 已有一定源码和文档。它没有产生以下证据：

- 测试是否覆盖生产语义、外部服务或故障恢复；
- 代码是否真实连接名义后端；
- 兼容、迁移、TLS、鉴权、secret、资源边界是否成立；
- benchmark 目录中是否存在基准；
- workflow 是否为 required check，或最近一次是否成功。

在被审计基线 `9174840` 上，所有 `benches/` 均只有 `.gitkeep`；当时 `cargo bench --workspace --no-run` 只能编译 lib 测试壳。后续 PR #157 已为 12 个 core crate 增加 `hot_path` bench，但 9 个 adapter 仍无基准，且尚无与生产 SLO 绑定的持续回归阈值、长稳或故障负载证据。

### 3.2 `scaffold+mock` 标签存在 false-positive

生成器在 `scripts/docs/gen-crate-status.mjs:171-177,223-227` 主要根据 adapter 身份、测试存在和 LOC 判定 `scaffold+mock`，并不检查真正的 mock 实现。结果是 `taosx` 在 `STATUS.md:62` 被标为 `scaffold+mock`，但源码只有内存 scaffold，没有 `mock.rs`；`docs/ssot/adapters-ssot-alignment.md:63-69,176` 反而记录得更准确。

### 3.3 历史签核没有扩大范围

`docs/plans/releases/0.3.0-signoff.md:146-170` 的 GO-with-Accepts 只给核心五 crate 分层签核，并明确：

- adapters mock 不是生产 DB/MQ/交易所客户端；
- `canonical` 只对 committed 子集声明 L2；
- `contracts` 只有 mock-L3；
- 整体 Production Ready 仍为否。

该历史签核不能替代本次对当前 commit、全部 21 个模块的审计。

## 4. 本次验证基线

| 命令 | 结果 | 能证明什么 | 不能证明什么 |
|---|---|---|---|
| `cargo build --workspace --all-features` | PASS | 当前 workspace 可构建 | 运行时行为与生产部署 |
| `cargo fmt --all --check` | PASS | 当前代码格式一致 | 行为正确性 |
| `cargo test --workspace --all-features --all-targets` | PASS | 当前离线测试套件全绿 | 真实后端、长稳、跨版本、生产流量 |
| `cargo clippy --workspace --all-features --all-targets -- -D warnings` | PASS | 当前 toolchain 下无 clippy warning | 安全、语义和性能完备 |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps` | PASS | rustdoc 可生成 | 每个公开方法都有完整合同；多数 crate doctest 为 0 |
| `cargo test --workspace --doc` | PASS | kernel 12 个 compile-fail doctest 通过 | 其余 20 个 crate 的用法；它们 doctest 均为 0 |
| `cargo deny check` | PASS，有未命中 `Zlib` allowance 警告 | 当前 advisories/bans/licenses/sources 配置通过 | TLS、secret、输入安全、unsafe 全局强制 |
| `node scripts/quality-gates/run-kernel-loom.mjs` | PASS，3 项 | 已建模的 trigger/wait 交错 | 未建模状态、timeout 极值、平台差异 |
| `node scripts/quality-gates/check-public-api.mjs` | PASS，5 个 package | 五个核心包 baseline 未漂移 | 其余 16 个 package、wire/语义兼容 |
| `node scripts/quality-gates/check-canonical-align.mjs` | PASS | 当前脚本覆盖的结构/测试/格式 | fixture 真正来自历史版本、完整迁移策略 |
| `node scripts/quality-gates/check-decimal-no-panicking-ops.mjs` | PASS，扫描 33 个文件 | 已匹配模式未命中 | 完整类型分析；脚本存在目录与表达式漏报 |
| `cargo bench --workspace --no-run` | 基线 PASS；PR #157 后新增 12 个 hot-path bench | release/bench profile 与新增目标可构建 | 生产 SLO、adapter 性能、长稳与故障负载 |
| `node scripts/quality-gates/check.mjs` | 初次 FAIL 39/40；刷新生成矩阵后 PASS 40/40 | Harness 与生成文档在交付状态一致 | 真实服务与非功能属性 |

注意：本次 Rust 运行使用 1.97.0，不是声明的 MSRV 1.85。仓库有 MSRV workflow 定义，但本报告没有把 workflow 文件存在当作当前 commit 的 MSRV run 成功记录。

## 5. 核心与类型模块

### 5.1 kernel

正面证据：

- `forbid(unsafe_code)`、`missing_docs`、`unreachable_pub` 完整；生产依赖仅 `thiserror`。
- Timestamp checked 运算、错误 source 链、Shutdown 的 `Mutex<bool> + Condvar` 与 poison 恢复证据较强。
- 本次 workspace 测试、12 个 compile-fail doctest、3 个 loom 模型、clippy、rustdoc、API baseline 均通过。

阻断与限制：

- `crates/kernel/src/lifecycle.rs:163-166` 使用 `Instant::now() + timeout`；合法公开输入 `Duration::MAX` 可触发 overflow panic，与“生产路径无 panic”对齐声明冲突。
- `ClockDomain::PROCESS`、`ClockDomain::from_raw` 及 doc-hidden 公开构造器允许下游伪造 domain，跨域保护不是封闭不变量：`crates/kernel/src/clock.rs:95-107,148-163`。
- Active SSOT 的 package/publish/version 与实际 `kernel`、`publish=false`、workspace 0.3.0 冲突；完整 Stable/分发声明不成立。
- 审计基线无实质 benchmark；PR #157 后已增加 hot-path bench，但无生产阈值；Miri/mutants workflow 存在不等于本次 run 已验证。

结论：在约束 timeout 输入、同一受控 ClockDomain、Linux x86_64 内部使用时可有条件采用；修复 panic 和身份/时钟域治理前不能无条件签字。

### 5.2 testkit

正面证据：

- ManualClock 的 wall/monotonic 独立性、domain、故障注入、毒锁、并发与属性测试较完整。
- 生产依赖图只含 kernel；仓库测试确认它没有泄漏为其他 crate 的 normal dependency。
- `forbid(unsafe_code)`、文档 lint、公开面测试与 API baseline 已建立。

限制：

- README 已明确“不是生产 runtime”；本报告只评价其作为测试基础设施的可靠性。
- 当前只落地 ManualClock core；独立 `contract-testkit`、完整 fixture/harness 平面已交付（#178；本报告审计日曾写未交付）。
- 继承 kernel 的公开 domain 构造边界；Miri/mutation 只有 workflow 定义，当前 commit 未在本报告中复跑。
- 审计基线无实际 benchmark；PR #157 后已增加 hot-path bench，但无生产阈值；除 kernel 外 workspace doctest 基本为空。

结论：ManualClock core 可作为生产项目的有条件测试基础设施；不适用“生产 runtime”声明，也不能代表完整 testkit 平面。

### 5.3 decimalx

正面证据：

- 字段已私有化；`try_new`、checked 算术、验证型 serde、oracle/边界/property 测试明显强于普通基础 crate。
- `forbid(unsafe_code)`、文档/API baseline、100% 行覆盖 workflow、Miri/mutants scheduled workflow 均已设置。
- 本次 checked 调用门禁、workspace tests、clippy、rustdoc 通过。

阻断与限制：

- 解析器先解析无符号绝对值再加负号，导致合法 `Decimal::new(i128::MIN, scale)` 可以 Display 却不能 FromStr 往返：`crates/types/decimal/src/lib.rs:468` 附近。
- `new`、`rescale`、`Add/Sub/Mul` 明确保留 panic 路径：`src/lib.rs:153-159,327-335,506-543`。这是文档化能力，不是隐藏 bug，但资金路径必须由机器强制使用 checked API。
- `check-decimal-no-panicking-ops.mjs` 是基于目录与正则的启发式扫描，不是类型分析；无法覆盖所有变量表达式和未来消费目录。
- 当前 serde shape 不等于长期跨语言 wire；无实质性能基准或完整 cargo-fuzz target。

结论：受控输入、强制 checked API、同版本内部计算可有条件使用；极值文本往返、完整门禁与 wire/performance 未闭合前，不应声明 package 全面生产稳定。

### 5.4 canonical

正面证据：

- committed 清单、`deny_unknown_fields`、golden/拒绝样例、非法 Decimal scale 拒绝及 API baseline 已存在。
- committed 范围明确分为 v1、v1.1、v1.2、v1.3；未列入清单的类型保持 Uncommitted。
- 本次 canonical 专项 gate 通过。

阻断与限制：

- `crates/types/canonical/src/wire.rs:41` 明确没有协议 envelope，版本主要靠常量和 fixture 维护。
- 多个所谓 N-1 fixture 在测试注释中就是“historical-looking / 首次冻结”，字段集与当前版本相同，例如 `wire.rs:224-228`；这不是由真实历史 producer 生成的跨版本证据。
- DTO 中的字符串、Vec、JSON body 没有统一长度、深度或集合容量上限；受不可信外部输入时仍需外层资源门禁。
- Active SSOT、当前 committed 清单、package 名和验证命令存在漂移，不能把 alignment 的 PASS 当作独立第二份证据。

结论：仅对明确 committed 的同协议家族、受控输入和外层资源限制可有条件使用；不得外推为全部 DTO、完整迁移体系或 package stable。

## 6. L1 基础设施模块

### 6.1 configx

当前实现是 `RwLock<HashMap<String,String>>`，单操作线程安全、拥有型读取、覆盖和毒锁路径有测试；作为可信代码控制的易失内存 KV 有有限价值。

生产配置角色的阻断：

- `new()` 永远为空；没有文件/env/CLI/远端源、持久权威源或确定性重建。
- 读锁中毒被静默折叠为 `None`，和正常缺失不可区分：`crates/configx/src/lib.rs:23-26,40-46`。
- 没有 schema、未知键策略、原子多键快照、last-known-good、通知、热重载与回滚。
- 没有 secret 类型、脱敏、容量边界；`Into<String>` 还在写锁内执行，调用方 panic 可毒化 store。

结论：仅在非关键、非敏感、允许重启丢失、无多键一致性/热更新的受控场景有限使用；作为生产配置子系统未就绪。

### 6.2 schedulex

当前实现只是 `HashMap<String,()>` 的登记、取消和 list，重复 ID 幂等，窄行为有测试。

它没有 Job、payload、Clock、timer、cron、worker、执行器、持久化、恢复、misfire、timeout、取消令牌、重试、dead letter、shutdown、lease 或 fencing。`list()` O(n) 克隆全部 key，ID 和总量也无边界。

结论：可作为临时进程内 registry；作为 scheduler 未就绪。“登记任务 ID”不能解释为调度或执行。

### 6.3 bootstrap

正面证据是 typed、只读 `PlatformContext`/`AppContext`、无 Service Locator，且 Instrumentation/Evidence/Shutdown 注入形状清晰。

阻断：

- `require_evidence()` 只在 `try_*` fail-closed；`build()`/`build_app()` 使用 `debug_assert!`，debug 可 panic，release 会移除检查并构造无 evidence 上下文：`crates/bootstrap/src/lib.rs:153-156,188-218`。
- 没有组件拓扑、async start、readiness、启动 deadline、部分启动逆序补偿、drain/join、停止 deadline 或 OS signal 接线。
- `take_shutdown_guard()` 后 controller 可静默 no-op；bounded traits 主要是 label/venue-id seam，不是实际组件能力。
- 默认 observex 没有 subscriber/exporter，evidence 默认 None；组合根没有形成可观测与审计闭环。

结论：typed composition 形状部分就绪；作为生产应用唯一组合根未就绪。

### 6.4 evidence

当前 `InMemoryEvidenceAppender` 能在单个 Mutex 下递增序号并保存事件名，开发测试行为简单可复现。

生产审计角色缺少 durable storage/fsync/WAL/crash recovery、canonical event、actor/subject/time、digest/hash chain、幂等 EventId、expected-head/CAS、跨实例序列、防 fork/gap、签名 checkpoint、外部锚与业务事务/outbox。`saturating_add` 到 `u64::MAX` 还会重复最大序号；Vec 无界增长。

结论：Dev/Test Only；不能承担不可抵赖或可恢复的生产审计证据。

### 6.5 observex

当前实现仅将 retry、circuit open/close 三个调用映射到同步 `tracing::info!`，无 subscriber 时不 panic，字段捕获测试通过。

生产阻断：

- bootstrap 默认创建 recorder，但不安装 subscriber；默认路径可静默丢弃全部事件。
- 同步 subscriber 的 panic 或阻塞可穿透业务热路径；当前只测试“无 subscriber”。
- 任意 `op: &str` 原样输出，无低基数、PII/secret、长度与 redaction 治理。
- 无 metrics、span/context propagation、sampler、有界队列、drop policy、exporter、flush/shutdown、schema/version 与端到端导出测试。

结论：受控环境的最小 tracing adapter 部分就绪；生产 observability 子系统未就绪。

### 6.6 resiliencx

重试/退避、三态熔断、手工令牌桶和 RAII bulkhead 已实现，离线测试与行覆盖证据较强。

生产阻断：

- 默认 wait 调用 `std::thread::sleep`；API 是同步闭包，没有 async deadline、per-attempt timeout、取消、总预算或 retry budget。
- Open→HalfOpen 由“被拒绝调用次数”推进，不是冷却时间；低流量可能永久 Open，突发流量可能过快探测。
- 任意错误都计入熔断失败；参数、权限等永久错误也会跳闸。
- rate limiter 只有 capacity 和手工 refill；无 rate/window/time source，`&mut self` 也不便多任务共享。
- jitter 对同 attempt 完全确定，不能打散同配置实例；缺少写操作幂等策略。
- bulkhead 无异步 permit、公平、排队/等待 timeout；可观测合同不覆盖限流拒绝、饱和、延迟和失败原因。

跨模块阻断：transport 的连接拒绝映射为 `Unavailable`，但 retry 只重试 `Transient`；最常见网络故障默认不会进入重试。429 的 `retry_after` 也在 adapter 映射时丢失。

结论：可用于受控同步任务或测试原语；生产弹性层未就绪。

### 6.7 transportx

HTTP/WS trait、reqwest/tungstenite 真实 driver 与 loopback 测试是真实正面证据；它不是纯 mock crate。

生产阻断：

- `HttpRequest`/`HttpResponse` derive `Debug`，会输出完整 URL、headers 和 body，可能泄漏 Authorization、签名、订单与账户数据：`crates/transport/src/lib.rs:71-92`。
- WS connect/read/write/close 没有 timeout/deadline；HTTP 只有一个可选总 timeout，没有独立 connect/read/write/pool 预算。
- HTTP `response.bytes()` 与 WS 完整帧全量缓冲，无 body/frame/header/连接数边界，存在内存 DoS 面。
- HTTP 与 WS 使用不同信任根；没有自定义 CA、mTLS、proxy、TLS policy、WS auth header/subprotocol 的生产配置证据。
- `HttpResponse` 不保留 headers；错误映射丢 timeout source、Close code/reason、合法 HTTP-date Retry-After 等信息。
- 无 transport 层 span/metric/redaction hook、性能/长稳/慢读/半开/DNS/TLS rotation 测试，也没有 transport 专项 coverage workflow。

结论：本地 loopback、sandbox 或低风险受控集成可用；修复敏感 Debug、deadline、资源上限、TLS/错误/观测闭环前不能处理真实凭证和无界外部响应后宣称生产就绪。

## 7. contracts

正面证据：依赖边界基本合规，16 个 trait 可编译，多数为 `Send + Sync`，对象形状、五个 first-batch conformance、venue override 与 API baseline 有测试。本次该包相关测试全绿。

生产阻断：

1. **事务合同不成立。** `TxContext` 只有 commit/rollback，Repository 操作不能绑定 transaction/session；Postgres scaffold 的 save 直接写 HashMap，而 begin_tx 返回无关联 FakeTx。业务写没有被事务保护。rollback 错误还被吞掉：`crates/contracts/src/lib.rs:98-145`。
2. **流式错误不可表达。** EventBus、PubSub、Venue/MarketData stream item 是裸数据，不是 `XResult<T>` 或 control envelope。运行中断线、解码、鉴权、序列缺口只能表现为无原因结束/停滞。
3. **MessageAck 未接入方法。** 没有 ack/nack/redelivery/offset 语义，不能承载 durable delivery。
4. **没有真实 conformance。** 9 个 adapter 都是 scaffold/mock；first-batch suite 只覆盖少数内存 happy path。
5. **语义仍为实现定义。** insert/upsert、not-found kind、时间 epoch/单位、取消/资源释放等允许不同实现产生不兼容行为。
6. **公开 fake 违反自身热路径合同。** `RecordingInstrumentation` 使用 `Mutex::lock().expect(...)`；锁中毒会 panic，且 fakes 无 feature 地进入正式 API baseline。
7. **Active SSOT 漂移。** Spec 仍称 15 traits、TxRunner 非 object-safe、所有 trait Send+Sync、crate 不含实现；当前源码均已不同。

结论：接口形状与离线测试基础较好，但真实事务、流式错误、可替换语义和真实 adapter 证据均未闭合；生产合同层未就绪。

## 8. 交易所 adapters

### 8.1 共性判定

`binancex` 和 `okxx` 都是 scaffold + 进程内 mock transport seam。测试只证明 trait 分派、布尔占位状态、mock URL 路由和粗粒度错误映射。

最高风险是**假成功与真假混合模式**：

- `connect()` 只翻转内存 bool，无握手/认证/健康检查。
- `place_order()` 不发请求，却直接返回 `Open` 且固定 `ts=0`；该值虽是合法 epoch 形状，但没有真实时钟采样，不能证明回执时间与时效性。
- 未注入 HTTP 时 cancel 静默成功，query 返回固定 Open；行情 stream 立即结束，账户/仓位/时间多为固定占位。
- 同时又允许注入真实 reqwest driver，形成“部分路径真联网、资金路径假 ACK”的危险表象。
- 没有 mock/live 类型或 feature 强隔离，也没有对未实现路径 fail-closed。

两者均缺少凭据模型、签名、nonce/timestamp、secret redaction、venue 限流、异步 retry/circuit、ambiguous timeout 对账、client-order-id 幂等、结构化错误、WS heartbeat/reconnect、snapshot+delta/checksum、审计和 sandbox/fault/soak 证据。

### 8.2 Binance 专项

源码 cancel 使用 POST，而 Binance Spot 当前官方文档要求 signed DELETE `/api/v3/order`，并要求 API key、timestamp 与 signature；client ref 也应区分 `origClientOrderId`。当前实现没有这些协议要素。参考：[Binance Spot Trade API](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/trade)。

结论：禁止接入真实资金路径；生产角色未就绪。

### 8.3 OKX 专项

OKX 私有接口要求认证头、HMAC-SHA256/Base64 签名与结构化请求体；当前实现 headers 为空、cancel 参数放 query、body 为空，且不解析 HTTP 200 内的业务 `code/sCode`。参考：[OKX API Guide](https://www.okx.com/docs-v5/en/)。

结论：禁止接入真实资金路径；生产角色未就绪。

## 9. 存储 adapters

### 9.1 共性事实

7 个 manifest 都没有相应外部系统 client。`contracts-live.yml` 名称虽含 live，但注释和命令明确是手工触发的 offline mock verification，没有启动任何外部服务；ClickHouse、OSS、TAOS 甚至不在该 workflow 中。

它们共同缺少连接池、timeout/retry/reconnect、TLS/auth、health/shutdown、schema/migration、序列化/version、容量/背压、观测、真实集成、故障恢复和性能证据。

### 9.2 clickhousex

核心是 `Mutex<Vec<(String, Bytes)>>`；endpoint 未用于 I/O。无 client、table/schema、batch/flush、压缩、幂等、部分失败、容量上限或恢复。仅可作为 `AnalyticsSink` 接口烟测。

### 9.3 kafkax

核心是 `HashMap<String, Vec<BusMessage>>`，subscribe 克隆当前快照后结束，不接收后续消息。生成 ID 和插入分两次加锁，并发 publish 可得到重复 ID。无 producer ack/idempotence、partition、consumer group、offset commit、rebalance、DLQ、schema registry、SASL/TLS。

### 9.4 natsx

实现与 Kafka scaffold 类似，同样有快照而非实时订阅和并发重复 ID 风险。还未裁定 Core NATS 或 JetStream，因此 durable、ack、retention、queue group、drain/reconnect 语义都不存在。

### 9.5 ossx

核心是进程内 HashMap；重启丢失、实例不共享。无 OSS/S3 SDK、bucket/tenant、stream/multipart/range、ETag/checksum、conditional write、版本、加密、凭据轮换和大对象边界。

### 9.6 postgresx

核心是 HashMap；Repository 直接写入，ScaffoldTxContext 与 rows 无关联。没有 SQL、参数绑定、row mapping、migration、pool、隔离级别、statement/transaction timeout、deadlock/serialization retry、TLS/HA。mock 的 staged write 不能证明通用 Repository 合同具有事务性。

### 9.7 redisx

主 adapter 明确忽略 TTL；PubSub 保存历史并返回快照，与真实 Redis Pub/Sub 不同。并发 publish 也可能重复 ID。无 client、pool、cluster/Sentinel、AUTH/ACL/TLS、MOVED/ASK、pipeline/Lua/CAS、failover/reconnect。

### 9.8 taosx

核心是 `HashMap<String, Vec<Tick>>`，写入 append、查询线性过滤；没有实际 mock。无 TDengine client、native/REST feature、时间精度、stable/subtable/tag、retention、乱序/去重、TLS/auth、恢复。`STATUS.md` 的 `scaffold+mock` 是分类误报。

## 10. 仓库级治理与证据缺口

### 10.1 安全 lint 未全局强制

当前 11/21 crate 显式 `forbid(unsafe_code)`；`transportx` 与 9 个 adapter 没有。根 workspace lint 只有两个 warn 项，constitution 检查也没有把 unsafe 扫描变成可靠失败门禁。因此 README 的“禁止 unsafe”仍主要是治理声明，不是完整机器保证。

### 10.2 验证深度不均

- API baseline 只覆盖 kernel/testkit/decimal/canonical/contracts。
- 专项 coverage 只覆盖 10/21；workspace coverage 上传并不等于阈值门禁。
- property/Miri/mutation 主要集中于 kernel/testkit/decimal；kernel 另有 loom。
- 无 cargo-fuzz target；PR #157 已增加 12 个 core hot-path bench，但无 adapter benchmark、SLO 阈值、soak/chaos 或真实 adapter 故障矩阵。
- `contracts-live.yml` 是 offline mock，不是 live backend。

### 10.3 发布、版本与供应链治理漂移

- 多个实际 package 名为无 `xhyper-` 前缀，但 README/SSOT/命令仍使用 `xhyper-*`；相应 `cargo -p` 命令不可复现。
- VERSIONING 文档称统一继承 workspace version，但实际同时存在 0.3.0、0.1.1 和多个 0.1.0。
- toolchain 使用滚动 stable/nightly，部分组织 workflow 依赖可变引用；CI Cargo 命令普遍未显式 `--locked`。
- 没有 `SECURITY.md`、LTS/EOL、SBOM/provenance/签名制品与发布后验证闭环。

### 10.4 文档新鲜度曾漂移，交付时已修复

`node scripts/quality-gates/check.mjs` 初次为 39/40，唯一失败是 `docs/status/CI_WORKFLOW_MATRIX.generated.md` stale。报告交付前已重新生成矩阵，最终复验为 40/40。这个过程说明“文档存在”不能自动视为当前事实，生成内容仍须通过新鲜度门禁。

## 11. 建议的生产化顺序

### P0：先消除会造成错误生产行为的路径

1. 修复 kernel 超大 timeout panic；封闭 ClockDomain 伪造入口。
2. 修复 decimal `i128::MIN` 文本往返；将资金路径 checked 使用升级为可靠机器门禁。
3. transport 默认脱敏 Debug，给所有 HTTP/WS I/O 增加 deadline 与 body/frame/connection 上限。
4. 统一 transport→adapter→resiliencx 的 retryable error 与 Retry-After 语义。
5. 重构 contracts 事务与 stream v2；事务工作必须绑定 session，stream item 必须能表达运行期错误与控制事件。
6. bootstrap 所有 required 路径 fail-closed；建立 start/ready/rollback/drain/deadline 生命周期。
7. 交易所 adapter 在 live 实现完成前强制 fail-closed，并做 mock/live 类型或 feature 隔离，禁止假 ACK。

### P1：建立真实生产证据

1. 选择最小真实后端组合，例如 Postgres + Redis/Kafka/NATS 之一，建立容器化 conformance、故障注入与恢复测试。
2. 为 Binance/OKX 建 signed REST、结构化 error fixture、sandbox 和 WS 重连/序列恢复测试。
3. configx 增加权威源、schema、原子 snapshot、last-known-good、watch/shutdown 与 secret 合同。
4. evidence 增加 durable append、canonical event、hash chain/CAS、恢复与签名 checkpoint。
5. observex 增加受控 schema、metrics/span、redaction、bounded exporter、flush/shutdown 与端到端验证。
6. resiliencx 增加 async wait、deadline/cancel、time-based circuit、真实 rate 与 retry budget。

### P2：补齐持续治理

1. 统一 package/version/SSOT/CHANGELOG/验证命令，并使生成文档恢复新鲜。
2. 为所有生产 crate 加统一 unsafe/docs/public API 策略；对关键 workflow 设 required check。
3. 固定 toolchain、工具和 Action 版本，CI 使用 `--locked`；建立依赖更新 SLA。
4. 为热路径与网络/队列建立可解释 SLO、benchmark、容量、长稳和回归阈值。
5. 真实证据齐备后，由 Maintainer 按 `docs/governance/prod-signoff-TEMPLATE.md` 分模块签字；审计代理不代签。

### 11.1 Beads 跟踪映射

现有收敛 epic 为 `infra-s9t`。本次审计复用了已有子任务，并补充了未被覆盖的阻断：

| 发现 | Beads |
|---|---|
| 真实后端与 contracts L3 | `infra-s9t.2`、`infra-s9t.3` |
| bootstrap required evidence / lifecycle | `infra-s9t.4`、`infra-s9t.5` |
| resiliencx async 路径 | `infra-s9t.6` |
| configx/evidence、schedulex/observex 边界 | `infra-s9t.7`、`infra-s9t.8` |
| decimal 极值、wire 与 panicking gate | `infra-s9t.9` |
| canonical 兼容策略 | `infra-s9t.10` |
| package/命令与治理漂移 | `infra-s9t.11`、`infra-s9t.12`、`infra-s9t.14` |
| kernel timeout/domain | `infra-s9t.15` |
| transport 安全/deadline/资源上限 | `infra-s9t.16` |
| observex 故障隔离与导出闭环 | `infra-s9t.17` |
| STATUS scaffold+mock false-positive | `infra-s9t.18` |

## 12. 未知项与报告限制

- 未取得当前 commit 的 GitHub required-check 实时配置与所有 workflow run 记录；workflow 文件存在不等于成功。
- 未连接任何真实交易所、数据库、MQ、对象存储或 telemetry backend；这正是 adapter 判未就绪的核心原因之一。
- 未运行完整 cargo-fuzz、Miri、mutation、soak、chaos 或跨平台矩阵。
- 未知真实业务 SLO、数据量、RTO/RPO、合规域、secret 分类和发布支持责任；未知不能解释为 PASS。
- 外部协议专项只用于证明当前 adapter 缺失基本协议要素，不构成第三方 API 的完整认证审计。

## 13. 最终声明

本报告支持以下表述：

> 当前仓库具有良好的 Rust 基础门禁和若干高质量核心窄能力，但 `STATUS.md` 的 92% 不能表示生产就绪。21 个模块中没有可无条件签字为 Production Ready 的模块；核心四个窄范围可有条件使用，多个 L1 仍为最小能力，9 个 adapter 全部未达到生产后端门槛。Workspace 整体不满足生产级应用标准。

本报告不支持以下表述：

- “21 个模块平均完成度 92%，所以系统已接近 92% Production Ready”；
- “scaffold+mock 通过测试，所以 adapter 可接真实资金或数据”；
- “SSOT 镜像写 COMPLETE/Stable，所以本仓实现已完成”；
- “GO-with-Accepts 核心签核等于整个 workspace 发布批准”。
