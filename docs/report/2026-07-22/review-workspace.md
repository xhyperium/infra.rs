# 集成审查：infra.rs workspace 全模块 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 审查范围 | 22 个 `crates/**` workspace package + `goalctl`、`verifyctl` |
| 视角 | 全模块 API、类型、错误、并发、契约、适配器、SSOT、门禁与量化端到端 |
| 执行依据 | [`review-prompt.md`](./review-prompt.md) v1.0 |
| 判据 | [`production-readiness-criteria.md`](./production-readiness-criteria.md) L1–L5、S1–S7、QT-1–QT-7、QT-Ship |
| 当前证据 | 当前 review worktree 源码、`cargo metadata`、测试/门禁输出、SSOT 对齐文档、历史 round-10 与 defer-close 综合 |
| 审查者 | AI Agent |

> **声明**：本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。审查结论仅代表代码基线在审查时刻的快照分析。

## 1. 结论先行

当前 workspace 的工程门禁为通过，但整体生产发布仍为 **NO-GO**。原因不是编译失败，而是生产条件尚未闭合：没有 Maintainer L5 签核，交易所私有交易链路和 WebSocket 长期运行证据不足，bootstrap 尚未证明从真实应用组合根闭合到交易所与存储，消息可靠语义还必须按具体部署选择并由 live 证据证明。

可以被当前证据支持的较窄结论是：kernel、decimalx、canonical、configx、schedulex、transportx、resiliencx、observex、evidence、contracts 及 storage 的部分声明面具备可复验的代码与测试入口；这不等于 package stable、L3 全量、可交易或 workspace Production Ready。

## 2. 审查方法与证据等级

审查按以下顺序执行：

1. 完整读取治理文件、生产判据、清单和本 Prompt；以 `cargo metadata --no-deps` 确认成员，而不是以文档数字推断成员。
2. 对每个 package 对照 SSOT、对齐文档、源码、测试、examples、live 测试和依赖声明；静态扫描 `panic`、`unwrap`、`expect`、`unsafe`、`f64`、serde 反序列化、锁和 tracing。
3. 执行当前可执行门禁；将 ignored live 测试、未配置的外部服务、历史报告和源码声明分别标为“未运行”“未知”“历史证据”“实现证据”。
4. 对生产结论采用最严格证据：mock/fixture 只能证明解析或分支行为，不能证明真实交易所、broker、数据库、TLS 握手或端到端组合根。

历史文档中存在更早的 DEFER 数字和结论；本报告采用 [`defer-close` 综合裁定](../2026-07-22-defer-close/synthesis/go-nogo-synthesis.md)作为声明层最新状态，同时以当前源码和本轮门禁纠正其不能直接证明 live/L5 的部分。

## 3. Workspace 清单与分层结论

| Package | 路径 | 层级 | 当前判定 | 主要边界 |
| --- | --- | --- | --- | --- |
| `kernel` | `crates/kernel` | L0 | 有条件 GO（库语义） | L5、业务组合根不属于本 crate |
| `testkit` | `crates/testkit` | T0 | 有条件 GO（仅测试） | 非 runtime 依赖 |
| `decimalx` | `crates/types/decimal` | Types/L0 | 有条件 GO（内部） | 资金路径必须使用 checked API |
| `canonical` | `crates/types/canonical` | Types/L2 subset | 有条件 GO（wire subset） | committed schema 不是全产品协议 |
| `bootstrap` | `crates/bootstrap` | L1 | 声明面 GO；交易装配 NO-GO | 没有真实 adapter 端到端证据 |
| `configx` | `crates/configx` | L1 | 进程内有条件 GO | 不是远端配置中心 |
| `schedulex` | `crates/schedulex` | L1 | 进程内有条件 GO | `tick` 驱动，不是分布式调度 |
| `evidence` | `crates/evidence` | L1 | 声明面有条件 GO | 合规不可变审计仍 OPEN |
| `observex` | `crates/observex` | L1 | 进程内有条件 GO | 不是完整 OTEL/OTLP SDK |
| `resiliencx` | `crates/resiliencx` | L1 | 有条件 GO | adapter 统一接线和外部策略仍需部署证据 |
| `transportx` | `crates/transport` | L1 | 有条件 GO | TLS/池/代理实际部署矩阵未全证 |
| `contracts` | `crates/contracts` | Contracts/L3 subset | 子集有条件 GO | 全 trait live/conformance 未闭合 |
| `contract-testkit` | `crates/test-support/contracts` | T0 | 有条件 GO（仅 dev） | 不能替代真实后端 |
| `binancex` | `crates/adapters/exchange/binance` | L2 adapter | NO-GO 交易 | live 仅公共 server time |
| `okxx` | `crates/adapters/exchange/okx` | L2 adapter | NO-GO 交易 | live 仅公共 server time |
| `redisx` | `crates/adapters/storage/redis` | L2 adapter | 有条件 GO（KV） | live ignored；Cluster/Sentinel 未证 |
| `postgresx` | `crates/adapters/storage/postgres` | L2 adapter | 有条件 GO（SQL/Tx） | live ignored；Repository 产品语义有限 |
| `kafkax` | `crates/adapters/storage/kafka` | L2 adapter | 有条件 GO（AMO/应用层 ALO） | broker live 与 EOS 仍未运行/非原生 |
| `natsx` | `crates/adapters/storage/nats` | L2 adapter | 有条件 GO（Core） | Core 与 JetStream 语义必须分开签核 |
| `ossx` | `crates/adapters/storage/oss` | L2 adapter | 有条件 GO | OSS live ignored；合规/重试边界需部署验证 |
| `clickhousex` | `crates/adapters/storage/clickhouse` | L2 adapter | 部分 GO | 批量写与池强度不足以宣称分析产品 ready |
| `taosx` | `crates/adapters/storage/taos` | L2 adapter | 部分 GO | REST 部分路径；native/批量/池未全闭合 |
| `goalctl` | `tools/goalctl` | Tools | 有条件 GO（最小 CLI） | CLI 语义和英文错误文案需治理对齐 |
| `verifyctl` | `tools/verifyctl` | Tools | 有条件 GO（最小 CLI） | plan/execute/report 不是完整审计平台 |

## 4. 通用维度与专项观察

### 4.1 D1–D9 总体结果

| 维度 | 结果 | 证据与边界 |
| --- | --- | --- |
| D1 公开 API 正确性 | 有条件 | `cargo doc -D warnings`、public API baseline 和 all-target 测试通过；exchange/storage 仍有显式降级或占位路径 |
| D2 类型与不变量 | 局部强 | kernel 的时钟域、decimalx checked 算术、canonical wire shape 较强；adapter 输入校验和 metadata 精度仍需继续强化 |
| D3 错误处理 | 局部通过 | kernel/adapter 多数映射到 `XError`；`evidence`、`observex`、tools 仍有英文 Display 或通用错误分类不足 |
| D4 并发安全 | 代码路径通过 | loom、锁 poison 测试和 transport pool 测试通过；live 后端并发/重连行为未运行 |
| D5 泛型与 Trait | 子集通过 | contracts 语义文档与能力拆分存在；`VenueAdapter` legacy facade、EventBus at-most-once 和 live profile 仍有边界 |
| D6 依赖与版本 | 通过 | workspace dependency、crate version、deny 均通过；deny 配置有冗余 skip 警告 |
| D7 SSOT 对齐 | 多数对齐 | 22 package 映射存在；SSOT COMPLETE 不能升级为实现或 L5；evidence 对齐面仍偏薄 |
| D8 测试覆盖 | 代码面强、live 面未知 | workspace test、核心三包 LCOV 100%、loom、fixtures 通过；live 测试均未在本轮运行 |
| D9 可观测性 | 进程内通过 | tracing/instrumentation/export/flush 代码存在；完整 OTEL、远端 exporter 和运行告警策略未证 |

### 4.2 分层专项结论

- L0：`ClockDomain`、`ShutdownSignal`、`ComponentState` 有 checked/矩阵/loom 证据；`testkit` 正确限定为 dev/test support。
- Types：decimalx 的 `WIRE_SCHEMA_VERSION`、serde 拒绝未知字段/非法 scale、checked 算术和 feature 门禁通过；canonical envelope、N-1/golden/拒绝样例通过。
- L1：configx 是内存字符串 KV + file/env layered/watch 的进程内面；schedulex 是外部注入时间的 `tick` runner；observex 是进程内 buffer/export；resiliencx 提供原语；transportx 提供 HTTP/WS、TLS、pool、proxy 面。
- Contracts：`KeyValueStore`、`EventBus`、`Repository`、`TxRunner`、Venue capability traits 和 `contract-testkit` 均存在；`LiveContractProfile` 是能力声明/辅助编排，不是实际环境证明。
- Storage：redis/postgres/kafka/nats/oss/clickhouse/taos 都有生产默认客户端或协议面及 ignored live 测试；本轮没有 broker/database/object-store live 运行证据。
- Exchange：Binance/OKX 已有 REST+WS、签名和 mock HTTP/WS fixture 路径，但只有公共 server-time live 入口；没有私有下单、撤单、账户、私有 WS 和重连的真实证据。
- Tools：goalctl 的 doctor/validate/compile 和 verifyctl 的 plan/execute/report 可运行；二者是最小 CLI，verifyctl **非生产 verifier**，也不是完整 goal/evidence 审计平台。

## 5. 发现明细

### P0：发布或量化端到端阻塞

| # | 文件/证据 | 问题 | 影响 | 建议 |
| --- | --- | --- | --- | --- |
| P0-1 | `docs/governance/prod-signoff-TEMPLATE.md`；[`production-readiness-criteria.md`](./production-readiness-criteria.md) L5 | 当前证据没有 Maintainer 人类 L5 signoff、冻结的 DEFER 清单和发布签核包 | 任何 workspace Production Ready 或可发布宣称都不成立 | 由 Maintainer 按模板填写并在发布流程中冻结证据；Agent 不代签 |
| P0-2 | `crates/adapters/exchange/{binance,okx}/tests/live_server_time.rs` | 交易所 live 证据只有公共 server time；没有真实签名下单/撤单/查单、账户和私有 WS 重连验证 | QT-1、QT-2 与 QT-Ship-1/6 未满足，不能宣称可交易 | 增加隔离 testnet/demo 凭据脚本、订单幂等/错误码/撤单/私有流和可回滚 live 证据 |
| P0-3 | `crates/bootstrap/src/store_set.rs`、`crates/bootstrap/src/lib.rs`；`contracts/src/live.rs` | 组合根提供 typed StoreSet/live helper，但没有从 bootstrap 到真实 exchange + storage 的端到端链路证据 | QT-Ship-1 未证明；声明层接口不能替代部署闭合 | 固化一个不依赖生产密钥的可复现实验组合，并补 adapter 到 bootstrap 的真实 wiring 验证 |
| P0-4 | `crates/adapters/storage/kafka/src/{bus.rs,at_least_once.rs,eos.rs}`、`crates/adapters/storage/nats/src/{bus.rs,jetstream.rs}` | Core/EventBus 默认面明确是 at-most-once；Kafka 应用层 ALO/EOS 与 NATS JetStream 是扩展面，未形成统一应用投递策略和运行证据 | QT-Ship-3 只能按部署配置有条件满足，不能全局宣称可靠消息 | 对每条生产链路选定 AMO/ALO/EOS，记录接受的丢失/重复语义，并运行 broker-backed conformance |

### P1：重要正确性或生产路径风险

| # | 文件:行号 | 问题 | 建议 |
| --- | --- | --- | --- |
| P1-1 | `crates/adapters/exchange/binance/src/adapter.rs:393-401,408-417` | Binance `tickSize`、`minQty`、`stepSize` 先解析为 `f64`；`stepSize` 读取后未用于 `SymbolMeta` 校验。浮点往返可能改变交易精度，且输入缺失时使用默认 tick/qty。 | 直接以 decimal 字符串解析；把 step size 纳入不可违反的订单校验；缺失/非法过滤器返回 `Invalid`，不要静默默认。 |
| P1-2 | `crates/adapters/exchange/binance/src/adapter.rs:706-775`；`okx/src/adapter.rs:550-630` | 没有 WS connector 时订阅返回成功的空流；连接/读取错误直接结束流，没有重连、退避、序列/快照恢复策略。 | 缺 connector 应返回 `Unavailable` 或显式能力状态；生产流增加重连、重订阅、断线间隔和数据连续性策略。 |
| P1-3 | `crates/adapters/exchange/binance/src/adapter.rs:491-504,577-604`；`okx/src/adapter.rs:364-378,464-490` | 无凭证或 legacy facade 的 place/cancel/query 返回 mock/固定状态；这是文档化降级，但如果调用方只看 `Ok` 会把非交易路径误当成真实执行。 | 将 mock 构造和生产 adapter 类型分离，或要求显式 `MockMode`；公共 trait 返回值中保留 execution provenance/能力状态。 |
| P1-4 | `crates/adapters/storage/kafka/src/bus.rs:58-91`；`nats/src/bus.rs:36-61` | `contracts::EventBus` 默认没有 ack/redelivery，Kafka consumer 还有分区 0、无 group coordinator 的限制；NATS Core 无历史回放。 | 生产调用方必须不能把 `EventBus` 当成 ALO；为 ALO/EOS 使用独立 trait/类型并在 bootstrap 中按能力注入。 |
| P1-5 | `crates/evidence/src/remote.rs:92-100` | Remote append 先写本地序号/记录，再发送远端；发送失败会留下本地记录而调用方得到错误，重试可能造成远端重复或序号语义分叉。注释记录了事实，但没有事务/幂等协议。 | 固化 at-most-once/at-least-once 语义、远端幂等键和重试协议；报告中将该实现限制为声明层功能。 |

### P2：规范、治理和维护性

| # | 文件:行号 | 问题 | 建议 |
| --- | --- | --- | --- |
| P2-1 | `crates/evidence/src/lib.rs`；`crates/observex/src/export.rs:18-24`；`tools/goalctl/src/compile.rs:15-23` | 多个用户可见错误仍使用英文 Display（例如 `telemetry exporter unavailable`、`io:`、`parse:`），与中文语言治理不一致。 | 将用户可见层错误中文化，内部 source 保留技术细节；补充错误分类映射。 |
| P2-2 | `tools/verifyctl/src/main.rs:72,117`；`tools/verifyctl/src/plan.rs:135` | CLI 序列化使用 `expect` 或 `unwrap_or_default`。对当前闭合 serde 类型实际很难触发，但仍是公共 CLI 的未声明 panic/静默摘要降级。 | 将序列化错误显式映射为 CLI 错误退出码；digest 计算失败不得默认为空字符串。 |
| P2-3 | `cargo test --workspace` 输出 | 多个 package 的 example 均名为 `basic`，产生 Cargo output filename collision 警告。当前不失败，但可能在未来 Cargo 版本变为硬错误。 | 给 examples 使用 package 前缀或在 manifest 中避免同名产物。 |
| P2-4 | `cargo deny check` 输出；`deny.toml:35,39,43-52` | deny 通过但报告 12 条 unnecessary/unmatched skip 配置，治理配置已漂移。 | 清理无效 skip，避免真正的重复依赖/平台风险被噪声掩盖。 |

### P3：微优化

| # | 文件/范围 | 建议 |
| --- | --- | --- |
| P3-1 | contracts legacy `VenueAdapter` 文档 | 将少量英文 legacy API 说明迁移为中文，保留标准技术名词。 |
| P3-2 | 各 adapter 的 bench/example | 为重复 `basic`、`hot_path` 名称建立统一命名约定和索引。 |

## 6. SSOT 对齐状态

| 平面 | 当前结论 | 证据 | 不能推出的结论 |
| --- | --- | --- | --- |
| kernel/testkit | 对齐 | `docs/ssot/kernel-ssot-alignment.md`、`testkit-ssot-alignment.md`、源码/loom/cov | 不推出应用组合根或 L5 |
| decimal/canonical | 对齐到 committed wire subset | `WIRE_SCHEMA_VERSION`、envelope、golden/N-1/拒绝样例及专项 gate | 不推出全业务协议稳定 |
| infra L1 | 对齐到各自声明面 | `docs/ssot/{configx,schedulex,bootstrap,evidence,observex,resiliencx,transport}-ssot-alignment.md` | 不推出远端配置、分布式调度、完整 OTEL 或合规审计 |
| contracts/testkit | 子集与辅助面存在 | `contracts/src/live.rs`、`contract-testkit` fakes/suite | `LiveContractProfile` 不等于真实 backend live |
| storage adapters | 生产默认客户端/协议面已本地化 | `docs/ssot/*x-ssot-alignment.md`、ignored live tests | 不推出 Cluster/JetStream/EOS/批量/native 全量或 package stable |
| exchange adapters | REST+WS 代码与 mock/fixture 存在 | `adapter.rs`、auth/market、公共 server-time live | 不推出签名交易可用、私有 WS 可用或可交易 |
| tools | goalctl/verifyctl 已为 workspace member | `cargo metadata`、CLI source/tests | 不推出完整审计平台或 xtask/gate 已落地 |

当前清单与 metadata 一致：`crates/**` 为 22 个 package，工具为 2 个 package。`archgate`/`.architecture` 按治理裁定为 OOS，不计作缺失实现。

## 7. 质量门禁结果

| 门禁项 | 状态 | 当前证据 |
| --- | --- | --- |
| `cargo build --workspace` | 通过 | workspace 全部编译完成 |
| `cargo test --workspace` | 通过 | exit 0；有同名 `basic` example 输出冲突警告；ignored live 未执行 |
| `cargo fmt --all --check` | 通过 | 无输出、exit 0 |
| `cargo clippy --workspace --all-features --all-targets -- -D warnings` | 通过 | exit 0 |
| `cargo deny check` | 通过（有警告） | advisories/bans/licenses/sources 均 ok；12 条 skip 配置警告 |
| `node scripts/quality-gates/check-workspace-deps.mjs` | 通过 | 24 个 manifest 扫描，全部 workspace 依赖 |
| `node scripts/quality-gates/check.mjs` | 通过 | Harness 44/44 |
| `node scripts/quality-gates/check-crate-versions.mjs` | 通过 | 22 个 crate 独立版本、path 版本对齐 |
| `node scripts/quality-gates/check-canonical-align.mjs` | 通过 | 文件结构、源码模式、test、clippy、fmt 全通过 |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` | 通过 | workspace docs 生成完成 |
| `node scripts/quality-gates/check-public-api.mjs` | 通过 | kernel/testkit/decimalx/canonical/contracts 五个 baseline 通过 |
| `node scripts/quality-gates/check-decimal-no-panicking-ops.mjs` | 通过 | 102 文件扫描，0 hits |
| `node scripts/quality-gates/cov-gate-100.mjs -p kernel --filter crates/kernel/src --all-features` | 通过 | 773/773，100% |
| `node scripts/quality-gates/cov-gate-100.mjs -p decimalx --filter crates/types/decimal/src` | 通过 | 878/878，100% |
| `node scripts/quality-gates/cov-gate-100.mjs -p canonical --filter crates/types/canonical/src` | 通过 | 679/679，100% |
| `RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release` | 通过 | 3/3 loom 模型测试 |
| `node scripts/quality-gates/run-kernel-loom.mjs` | 通过 | 同上，3/3 |
| ignored live tests | 未运行 | 需要外部 broker/DB/OSS/exchange 与凭据；不能标成通过 |

Prompt 原文中的 `node scripts/quality-gates/cov-gate-100.mjs kernel` 与当前脚本 CLI 不兼容；本审查按脚本实际要求的 `-p` 与 `--filter` 修正执行，并将这一差异视为 Prompt 维护项，而不是覆盖率通过证据的缺失。

## 8. 跨 crate 接口与端到端分析

```text
kernel (XError/Clock/Shutdown)
  ├── decimalx ── canonical (checked money + committed DTO/wire)
  ├── configx / schedulex / resiliencx / transportx / observex / evidence
  ├── contracts (KV / Bus / Tx / Repo / Venue capability traits)
  │     └── contract-testkit (Fake + conformance；仅 dev)
  ├── bootstrap (PlatformContext + StoreSet + AsyncDrain)
  └── adapters: redis/postgres/kafka/nats/oss/clickhouse/taos/binance/okx
```

接口形状总体兼容：workspace dependency gate、编译、trait surface 和 contract-testkit 通过。风险集中在语义而非类型：`EventBus` 默认 AMO、adapter 的 mock/降级 `Ok`、WS 断线结束、storage/exchange live ignored，以及 bootstrap 尚未由真实应用闭环证明。

### QT 场景判定

| 场景 | 判定 | 依据 |
| --- | --- | --- |
| QT-1 行情 | Gap | exchange 私有/长期 WS 与重连 live 证据缺失 |
| QT-2 下单 | Gap | 代码有签名 REST 路径，但无 testnet/demo 私有交易证据 |
| QT-3 风控 | Conditional | decimalx checked + resiliencx budget/redis/postgres 面；未证明完整业务接线 |
| QT-4 持久化审计 | Conditional | storage/evidence 代码与 ignored live 入口存在；消息/合规语义未全闭合 |
| QT-5 配置调度 | Conditional | configx 分层/watch 与 schedulex tick 存在；不是远端/分布式平台 |
| QT-6 可观测 | Conditional | tracing + in-process exporter/flush；不是完整 OTEL 产品 |
| QT-7 聚合分析 | Conditional | clickhouse/taos REST/HTTP 部分面；批量、池、native 未全证 |

### QT-Ship

| 条件 | 判定 | 说明 |
| --- | --- | --- |
| QT-Ship-1 端到端链路 | 未满足 | typed wiring 存在，真实 bootstrap→adapter→backend 未证明 |
| QT-Ship-2 资金安全 | 局部满足 | decimal checked 和 gate 通过；Binance metadata 仍有 f64 输入转换风险 |
| QT-Ship-3 投递语义 | 有条件 | Kafka 应用层 ALO/EOS 与 NATS JetStream 扩展存在；默认 EventBus 是 AMO |
| QT-Ship-4 可观测与关停 | 有条件 | kernel/bootstrap drain + observex 进程内面；完整生产 telemetry 未证明 |
| QT-Ship-5 密钥与 TLS | 有条件 | secret/Debug 脱敏和 TLS 配置存在；真实握手/部署策略未运行 |
| QT-Ship-6 Live 证据 | 未满足 | 当前 live tests 均 ignored，本轮未执行；exchange 仅公共 server-time 入口 |

## 9. 生产就绪裁定

| 维度 | 裁定 |
| --- | --- |
| workspace L 层 | 不能整体宣称 L5；工程门禁通过不等于 Release Ready |
| 规格平面 S | 多数 package 达到或超过 28/35；evidence/exchange 的规格与实现边界仍较薄 |
| storage 默认客户端 | 有条件 L1 工程面；不等于 package stable、L3 全量或 L5 |
| exchange | NO-GO 交易；代码路径存在，真实私有交易/WS 证据不足 |
| 量化端到端 | NO-GO |
| workspace 整体 Production Ready | NO-GO |
| 阻塞项 | P0-1 至 P0-4；P1-1 至 P1-5 需按目标产品处理 |

## 10. 建议顺序

1. Maintainer 先冻结目标产品和签核边界；把 workspace、storage、exchange、tools 的“声明层”与“生产层”分开。
2. 为 Binance/OKX 建立受控 testnet/demo live harness：签名、时间偏移、订单幂等、错误分类、撤单/查单、私有 WS、重连和清理。
3. 选择并记录消息语义：AMO、应用层 ALO 或 EOS；禁止通过通用 `EventBus` 名称掩盖能力差异。
4. 将 bootstrap typed StoreSet 与真实 storage/exchange 能力接线，补一条无生产密钥的端到端验证链。
5. 修复 Binance metadata 的 `f64`/默认值问题、WS 空流语义、tools 未声明 panic、中文错误和 example/deny 配置噪声。
6. 最后由 Maintainer 按 L5 模板填写签核、CHANGELOG 和可回滚发布证据。

## 11. 变更记录

| 日期 | 说明 |
| --- | --- |
| 2026-07-22 | 按 `review-prompt.md` v1.0 对当前 workspace 执行全模块审查；补充当前门禁、源码风险、SSOT 与 QT-Ship 裁定。 |
