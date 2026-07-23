# adapters SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 域 | `adapters/`（exchange + storage） |
| Active SSOT | `.agents/ssot/adapters/**`（本仓权威；变更须同步 dual spec、alignment 与门禁） |
| 本仓路径 | `crates/adapters/{exchange,storage}/<name>` |
| 审计日期 | 2026-07-22 |
| 结论 | storage×7 声明面见专项对齐；exchange 已有签名 REST + 公共 WS 解析/注入，但精度/限流/时钟/私有 WS/重连/受控 live 交易证据未闭合，**交易 NO-GO**；未宣称 package stable / L5 / crates.io |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 历史 COMPLETE / Spec Approved 叙事 | 仅描述历史战役；禁止单独当作本仓交付证明 |
| current-state 权威 | 本仓 `.agents/ssot/adapters/**`；不再以外仓 0 diff 作为验收条件 |
| 本仓 adapter crates | storage OBJECTIVE 见专项证据；exchange 签名 REST + 公共 WS 行情解析已实现，但不是可交易声明；COPY/migrations/schema-registry/全量管理订单 WS 等仍 OPEN |
| `crates/AGENTS.md` 标准八项布局 | **已齐**（README / AGENTS / CHANGELOG / examples / docs / tests / benches） |
| `publish = false` | **已**在 adapter + contracts `Cargo.toml` 显式关闭 |
| package stable / crates.io | **未**宣称 |
| contracts trait crate | **已** workspace member（#43 `contracts`；历史产品名 `xhyper-contracts`，不可用于 `-p`）；trait scaffold，非业务实现 |

## SSOT 目录

保留上游 `adapters/` 层级（勿展平到 `.agents/ssot/` 根；`infra/` 已展平）。

```text
.agents/ssot/adapters/
├── exchange/
│   ├── binance/     # → crates/adapters/exchange/binance · package binancex
│   └── okx/         # → crates/adapters/exchange/okx     · package okxx
└── storage/
    ├── clickhouse/  # → crates/adapters/storage/clickhouse · package clickhousex
    ├── kafka/       # → crates/adapters/storage/kafka      · package kafkax
    ├── nats/        # → crates/adapters/storage/nats       · package natsx
    ├── oss/         # → crates/adapters/storage/oss        · package ossx
    ├── postgres/    # → crates/adapters/storage/postgres   · package postgresx
    ├── redis/       # → crates/adapters/storage/redis      · package redisx
    └── taos/        # → crates/adapters/storage/taos       · package taosx
```

每个域目录对齐 kernel 11 层布局：`goal/spec/design/plan/tasks/prompt/test/review/release/retrospective` + `matrix/gate/evidence` + `README.md`。

双镜像：`spec/spec.md` ≡ `spec/xhyper-*-complete-spec.md`（`cmp` 同构；镜像内已满足）。

## 本仓可观察事实

```text
Cargo.toml members              含 crates/contracts + 9 个 crates/adapters/**
package 命名                    `contracts`（别名 `xhyper-contracts` 已废弃）；binancex / okxx / redisx / …（adapter 无 xhyper- 前缀）
lib 入口                        adapters: Error/Result；contracts: ExchangeAdapter/StorageAdapter
生产依赖                        adapters: thiserror；contracts: serde + thiserror
实现深度                        storage 声明面见专项；exchange **签名 REST + 公共 WS 解析/注入（交易 NO-GO）**
标准布局八项                    已齐
publish                         false（显式）
version                         storage 独立版本；见下表各 package 行（kafkax `0.3.5`）

| 镜像路径 | 本仓路径 | package | 本仓状态 |
|----------|----------|---------|----------|
| `.agents/ssot/adapters/exchange/binance` | `crates/adapters/exchange/binance` | `binancex` | **`0.3.2`** HMAC 签名 REST + 公共 WS 解析/注入；live 仅 server_time；交易 NO-GO |
| `.agents/ssot/adapters/exchange/okx` | `crates/adapters/exchange/okx` | `okxx` | **`0.3.3`** 四头签名 REST + 公共 WS 解析/注入；live 仅 server_time；交易 NO-GO |
| `.agents/ssot/adapters/storage/clickhouse` | `crates/adapters/storage/clickhouse` | `clickhousex` | **`0.3.3`** HTTP(S)+PEM CA+`insert_batch`+有界池；真实集群 TLS OPEN |
| `.agents/ssot/adapters/storage/kafka` | `crates/adapters/storage/kafka` | `kafkax` | **`0.3.5`** AMO/ALO + TLS/PLAIN + 生产测试矩阵（offline/reliability/bench/fault）；group/native EOS NO-GO；Part2 OOS |
| `.agents/ssot/adapters/storage/nats` | `crates/adapters/storage/nats` | `natsx` | **`0.3.3`** Core/JetStream；同客户端重启恢复 3/3，断线窗口无回放、Cluster/HA NO-GO |
| `.agents/ssot/adapters/storage/oss` | `crates/adapters/storage/oss` | `ossx` | **`0.3.3`** ObjectStore + 有界 multipart/retry/orphan 补偿；dev live PASS |
| `.agents/ssot/adapters/storage/postgres` | `crates/adapters/storage/postgres` | `postgresx` | **`0.3.12`** Pool/Tx/Repository/COPY/Migrator/mTLS/selfcheck；live + selfcheck Full；package stable OPEN |
| `.agents/ssot/adapters/storage/redis` | `crates/adapters/storage/redis` | `redisx` | **`0.3.7`** Standalone + 安全 PubSub 边界；Cluster/Sentinel/TLS live OPEN |
| `.agents/ssot/adapters/storage/taos` | `crates/adapters/storage/taos` | `taosx` | **`0.3.9`** Production-default 全 API + gap-zero register；非 crates.io package-stable |
验证（本仓权威命令）：

```bash
# 双镜像
for d in exchange/binance exchange/okx \
  storage/clickhouse storage/kafka storage/nats storage/oss \
  storage/postgres storage/redis storage/taos; do
  cmp .agents/ssot/adapters/$d/spec/spec.md \
      .agents/ssot/adapters/$d/spec/xhyper-*-complete-spec.md
done

# 默认生产路径（含 storage 生产客户端）
cargo check -p binancex -p okxx -p redisx -p kafkax -p natsx \
  -p postgresx -p taosx -p ossx -p clickhousex
cargo test -p redisx -p postgresx -p kafkax -p natsx \
  -p ossx -p clickhousex -p taosx --all-targets
cargo test --workspace --all-targets
node scripts/kafka-tls-sasl-conformance.mjs
node scripts/postgres-deadline-conformance.mjs
node scripts/clickhouse-https-conformance.mjs

# live（仅 dev；默认 ignore；安全 runner 在子进程退出后清理临时文件）
# scripts/live/export-foundationx-env.sh --env dev -- \
#   cargo test -p redisx -p postgresx -p kafkax -p natsx \
#     -p ossx -p clickhousex -p taosx -- --ignored
```

## exchange named DEFER（#210 / #214）

| DEFER 项 | 状态 | 证据 |
|----------|------|------|
| binancex 签名 HMAC-SHA256 | **PASS** | `auth::sign_*` 向量 + 签名 REST mock |
| binancex 下单 place/cancel/query | **PASS** | 签名路径 + 4xx 业务错误映射 |
| binancex WS 行情 | **PASS（公共最小面）** | bookTicker/trade/depth 解析 + `with_ws` |
| okxx 签名四头 | **PASS** | `OkxApiKey::sign*` + Debug 脱敏 |
| okxx 业务协议信封 | **PASS** | `code`/`data` + cancel `sCode` |
| 全量私有/管理订单 WS、OCO、L5/stable | **OPEN / 非目标** | CHANGELOG + README |
| 精度 filters / 限流 / 时钟偏移 / WS 重连 | **OPEN / 交易 NO-GO** | 当前源码/测试无闭合证据 |
| 受控 testnet/demo 下单-查单-撤单 live | **OPEN / 交易 NO-GO** | 当前 live 仅 server_time |

## 对齐矩阵（本仓证据，非镜像勾选）

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| A-1 | 镜像路径保留 `adapters/` 层级 | PASS | `.agents/ssot/adapters/{exchange,storage}/` |
| A-2 | 本仓 current-state 不依赖外仓 0 diff | PASS | `.agents/ssot/SSOT.md` R6/R7 + 本轮门禁 |
| A-3 | 9 域均有 README + 11 层 + dual-spec | PASS | 镜像树；`cmp` 全 OK |
| A-4 | 本仓 crate 路径与 SSOT Code 列一致 | PASS | `crates/adapters/...` 与镜像 README Code 列同构 |
| A-5 | workspace members 已注册 9 package | PASS | 根 `Cargo.toml` |
| A-6 | scaffold 可 `cargo check` / `cargo test` | PASS | feature `scaffold` 可选；默认生产路径可编译测试 |
| A-7 | 标准八项布局 | PASS | 9 adapters + contracts 均已补齐（含 benches/） |
| A-8 | `publish = false` | PASS | 各 `Cargo.toml` 显式关闭 |
| A-9 | 实现真实 I/O / adapter 业务 | **部分（声明范围有证据）** | storage 真实客户端已落地；Kafka 为 AMO/单 owner ALO、NATS 为 Core AMO/JetStream durable pull；不得用历史“应用级 EOS”或薄封装冒充 native EOS/自动 DLQ；exchange 交易仍 NO-GO |
| A-10 | package stable / Spec Approved 本仓宣称 | OPEN | **禁止**用镜像 COMPLETE 代替；P0 生产入口 ≠ package stable |
| A-11 | contracts workspace 注册 | PASS | #43 `crates/contracts` → `contracts`（别名 `xhyper-contracts` 已废弃） |
| A-12 | `FOUNDATIONX_*` 环境注入 + 密钥不入库 | PASS | `from_env` + live tests；`scripts/live/build-foundationx-env.mjs`（#191）；secrets 仅进程 env |
| A-13 | 有界 benches（`cargo test --all-targets` 不挂） | PASS | #190：kafka/nats/clickhouse/taos hot_path 3s 超时；redis/postgres/oss 既有 |

## 与 active spec 的关系

- `.agents/ssot/adapters/**`：本仓 active spec；不得只改 CLOSED/COMPLETE 叙事冒充实现证据
- 本仓只接受 `.agents/ssot/adapters/...` 与 `crates/adapters/...` 路径；历史外仓路径不构成验收入口
- 实现 SSOT 以 **源码 + 本仓测试输出** 为准
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`

## 依赖与边界（本仓意图）

```text
adapters/*  →  contracts / kernel（+ 外部 SDK：redis/tokio-postgres/rskafka/async-nats/reqwest…）
               禁止  kernel/types 反向依赖 adapters
               禁止  把 adapter 当 L0
```

- **storage 生产路径**依赖对应驱动（见各 crate `Cargo.toml`）；`scaffold` feature 保留 mock 面
- exchange：注入 `HttpDriver`+凭证走签名 REST；注入 `WsConnector` 解析公共行情。未注入能力时存在成功占位/空流 fail-open，且无受控 live 下单证据；维持交易 NO-GO

## `crates/contracts` 评估结论

| 问题 | 结论 |
|------|------|
| 是否应并入 workspace？ | **已并入**（#43），无需再次注册 |
| package | `contracts`（别名 `xhyper-contracts` 已废弃） · path `crates/contracts` |
| 角色 | adapter trait 出口；**不是**上游 `.agents/ssot` 独立域 |
| 实现深度 | trait + 共享类型 scaffold |
| 标准布局 / publish | 本 PR 补齐 |
| 风险 | `Ticker` 使用 `f64` 金额字段 — 与 decimalx 禁令冲突；**禁止**宣称 stable 直至收口 |

## 未做（follow-up / OPEN）

> **Current-state 说明**：历史 #211 的 DEFER 裁定不能替代当前能力证据。Kafka/NATS 只按专项 alignment 中的 AMO、单 owner ALO、JetStream durable pull 与 NO-GO 边界验收。

1. Redis Streams full / Kafka schema registry / broker 事务协议 EOS / JetStream KV full / CH native 9000 / taos full WS SQL session — **非 OBJECTIVE 残留**（生产默认路径已有替代面）
2. adapters 全量实现 contracts trait 的业务深度（当前生产客户端 + 部分 trait 绑定）
3. contracts：`Ticker` 等金额字段迁离 `f64`（改 decimalx / canonical）
4. package stable / crates.io — **禁止**宣称
5. package 命名是否统一 `xhyper-*` 前缀 — 需 Lead 裁决；当前保留 #42 / #43 命名

## 相关索引

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- gap：[draft-gap-matrix.md](./draft-gap-matrix.md) · [gap-matrix.md](./gap-matrix.md)
- 同步报告：[SSOT_SYNC_REPORT.md](./SSOT_SYNC_REPORT.md)
- live 凭据：`scripts/live/build-foundationx-env.mjs`（#191）
- 规则：[.agents/ssot/SSOT.md](../../.agents/ssot/SSOT.md)
- 初始化 commit：`b586260` feat: initialize adapter crates (#42)

## 跟进（2026-07-22 / #188–#191 · draft 生产落地）

| 项 | 状态 | 证据 |
|----|------|------|
| storage×7 默认生产客户端 | **PASS（P0）** | #188 源码 + `cargo test -p <pkg> --all-targets` |
| live `#[ignore]` 入口 | **PARTIAL / OPEN** | Redis 有可复验 CI；其余入口存在但当前无 CI 工件/脱敏留档，禁止据此宣称生产就绪 |
| benches 有界 | **PASS** | #190 超时包装；`--all-targets` 不挂死 |
| `FOUNDATIONX_*` env 构建 | **PASS** | #191 `scripts/live/build-foundationx-env.mjs` |
| package stable / 全业务 Production Ready | **OPEN** | 禁止宣称 |

```bash
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo test -p redisx -p postgresx -p kafkax -p natsx \
    -p ossx -p clickhousex -p taosx -- --ignored
```

## 跟进（历史 · 2026-07-21 / PR #98 · asa.5 · s9t）

| 项 | 状态 |
|----|------|
| scaffold 签名适配 | **PASS**：EventBus/PubSub 流项为 `BusMessage`；TxRunner `begin_tx` |
| mock-first 离线 CI | **PASS**：默认无外部服务 |
| redis live KV（s9t 子集） | **PASS**（后由 #188 升级为默认生产 `RedisPool`） |
| exchange 早期 server-time 入口 | **PASS（历史阶段；当前已有签名 REST + 公共 WS）** |

## SSOT 树补充（2026-07-22）

| 路径 | 说明 |
|------|------|
| `adapters/README.md` | 九域索引 + storage P0 表 |
| `storage/*/plan/infra-rs-landing.md` | 本仓生产落地说明 |
| `storage/*/plan/infra-rs-draft-spec-goal.md` | draft SPEC_GOAL 入库快照 |

## storage 分 package 对齐（2026-07-22）

| package | 对齐文档 | SSOT |
|---------|----------|------|
| redisx | [redisx-ssot-alignment.md](./redisx-ssot-alignment.md) | `.agents/ssot/adapters/storage/redis/` |
| postgresx | [postgresx-ssot-alignment.md](./postgresx-ssot-alignment.md) | `.agents/ssot/adapters/storage/postgres/` |
| kafkax | [kafkax-ssot-alignment.md](./kafkax-ssot-alignment.md) | `.agents/ssot/adapters/storage/kafka/` |
| natsx | [natsx-ssot-alignment.md](./natsx-ssot-alignment.md) | `.agents/ssot/adapters/storage/nats/` |
| ossx | [ossx-ssot-alignment.md](./ossx-ssot-alignment.md) | `.agents/ssot/adapters/storage/oss/` |
| clickhousex | [clickhousex-ssot-alignment.md](./clickhousex-ssot-alignment.md) | `.agents/ssot/adapters/storage/clickhouse/` |
| taosx | [taosx-ssot-alignment.md](./taosx-ssot-alignment.md) | `.agents/ssot/adapters/storage/taos/` |

各域 `goal/design/plan/tasks/test/review/release/matrix/gate/evidence` 已从布局占位改为 **infra.rs P0 实质合同**。

## STATUS 成熟度（2026-07-22）

`scripts/docs/gen-crate-status.mjs`：`scaffoldSignal` 对 **default features 不含 scaffold 且存在 pool/client 生产模块** 的 storage adapter **不再**因文档中的 scaffold 字样封顶为 `scaffold+mock`。
storage×7 STATUS 完成度 **100%** · 成熟度 `active`（结构进度，≠ package stable）。

## storage×7 三轮加固（2026-07-23 / feat infra-2d9.11）

- 每 adapter 完成 R1 合同诚实化 → R2 对抗/离线边界 → R3 文档与版本锚点。
- OPEN/NO-GO（Kafka EOS、NATS 断线回放/Cluster、Redis Cluster·Sentinel·TLS live、CH 真实集群 TLS、package stable）**未静默升级**。
- live 入口：`scripts/live/export-storage7x-env.sh`（或既有 `export-foundationx-env.sh`）+ `#[ignore]`。
