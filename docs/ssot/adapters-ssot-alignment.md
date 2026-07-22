# adapters SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 域 | `adapters/`（exchange + storage） |
| 镜像 | `.agents/ssot/adapters/**`（R6 只读；**禁止**改镜像冒充本仓完成） |
| 本仓路径 | `crates/adapters/{exchange,storage}/<name>` |
| 审计日期 | 2026-07-22 |
| 结论 | **7 个 storage 默认生产客户端已落地**（#188–#190）：RedisPool / PostgresPool / KafkaPool / NatsPool / OssClient / ClickHousePool / TaosPool + live `#[ignore]`（ZoneCNH 真凭据已验）+ 有界 benches；scaffold 改 `feature = "scaffold"`；exchange 仍只读 server_time；**未**宣称 package stable / Cluster·JetStream·EOS 全量 / crates.io |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游镜像 COMPLETE / Spec Approved 叙事 | 描述的是 **xhyper monorepo 战役**；**禁止**单独当作本仓交付证明 |
| 镜像同步 | **完整**：与 `xhyper.rs/.agent/SSOT/adapters/` `diff -rq` = 0（144 文件 / 1.1M） |
| 本仓 adapter crates | **storage 生产默认路径已落地**（见下表）；scaffold → `feature = "scaffold"`；exchange 可选 `HttpDriver` + **公共 `server_time` JSON 解析**；Cluster / Sentinel / JetStream / EOS / multipart / migrations **DEFER** |
| `crates/AGENTS.md` 标准八项布局 | **已齐**（README / AGENTS / CHANGELOG / examples / docs / tests / benches） |
| `publish = false` | **已**在 adapter + contracts `Cargo.toml` 显式关闭 |
| package stable / crates.io | **未**宣称 |
| contracts trait crate | **已** workspace member（#43 `xhyper-contracts`）；trait scaffold，非业务实现 |

## 镜像目录（只读）

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
package 命名                    xhyper-contracts；binancex / okxx / redisx / …（adapter 无 xhyper- 前缀）
lib 入口                        adapters: Error/Result；contracts: ExchangeAdapter/StorageAdapter
生产依赖                        adapters: thiserror；contracts: serde + thiserror
实现深度                        storage **生产默认客户端** + live/bench；exchange 公共 time；Cluster/JetStream 等 DEFER
标准布局八项                    已齐
publish                         false（显式）
version                         workspace 0.3.0
```

| 镜像路径 | 本仓路径 | package | 本仓状态 |
|----------|----------|---------|----------|
| `.agents/ssot/adapters/exchange/binance` | `crates/adapters/exchange/binance` | `binancex` | scaffold + mock HTTP + **`parse_binance_server_time`** + `#[ignore]` live |
| `.agents/ssot/adapters/exchange/okx` | `crates/adapters/exchange/okx` | `okxx` | scaffold + mock HTTP + **`parse_okx_server_time`** + `#[ignore]` live |
| `.agents/ssot/adapters/storage/clickhouse` | `crates/adapters/storage/clickhouse` | `clickhousex` | **生产 `ClickHousePool` HTTP** + `AnalyticsSink` + `FOUNDATIONX_CLICKHOUSEX_*` + live；scaffold feature |
| `.agents/ssot/adapters/storage/kafka` | `crates/adapters/storage/kafka` | `kafkax` | **生产 `KafkaPool`/`Producer`/`Consumer`** + `EventBus`（at-most-once）+ SASL + live；scaffold feature |
| `.agents/ssot/adapters/storage/nats` | `crates/adapters/storage/nats` | `natsx` | **生产 `NatsPool`** + `EventBus` + `FOUNDATIONX_NATS_*` + live；scaffold feature |
| `.agents/ssot/adapters/storage/oss` | `crates/adapters/storage/oss` | `ossx` | **生产 `OssClient`（OSS V1 签名）** + `FOUNDATIONX_OSSX_*` + live；scaffold feature；multipart **DEFER** |
| `.agents/ssot/adapters/storage/postgres` | `crates/adapters/storage/postgres` | `postgresx` | **生产 `PostgresPool`/`PgTransaction`** + SQLSTATE 映射 + `FOUNDATIONX_POSTGRESX_*` + live；scaffold feature |
| `.agents/ssot/adapters/storage/redis` | `crates/adapters/storage/redis` | `redisx` | **生产 `RedisPool`/`RedisClient`** + `KeyValueStore` + `FOUNDATIONX_REDISX_*` + live/bench；scaffold feature |
| `.agents/ssot/adapters/storage/taos` | `crates/adapters/storage/taos` | `taosx` | **生产 `TaosPool` REST** + `TimeSeriesStore` + `FOUNDATIONX_TAOSX_*` + live；scaffold feature |

验证（本仓权威命令）：

```bash
# 镜像完整性（相对上游）
diff -rq /home/workspace/xhyper.rs/.agent/SSOT/adapters .agents/ssot/adapters

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

# live（默认 ignore；凭据见 scripts/live/build-foundationx-env.mjs）
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p redisx --test live_kv -- --ignored
# cargo test -p postgresx --test live_postgres -- --ignored
# cargo test -p kafkax --test live_event_bus -- --ignored
# cargo test -p natsx --test live_event_bus -- --ignored
# cargo test -p ossx --test live_object_store -- --ignored
# cargo test -p clickhousex --test live_smoke -- --ignored
# cargo test -p taosx --test live_smoke -- --ignored
```

## 对齐矩阵（本仓证据，非镜像勾选）

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| A-1 | 镜像路径保留 `adapters/` 层级 | PASS | `.agents/ssot/adapters/{exchange,storage}/` |
| A-2 | 与上游 `diff -rq` 无差异 | PASS | 同步报告 2026-07-21 |
| A-3 | 9 域均有 README + 11 层 + dual-spec | PASS | 镜像树；`cmp` 全 OK |
| A-4 | 本仓 crate 路径与 SSOT Code 列一致 | PASS | `crates/adapters/...` 与镜像 README Code 列同构 |
| A-5 | workspace members 已注册 9 package | PASS | 根 `Cargo.toml` |
| A-6 | scaffold 可 `cargo check` / `cargo test` | PASS | feature `scaffold` 可选；默认生产路径可编译测试 |
| A-7 | 标准八项布局 | PASS | 9 adapters + contracts 均已补齐（含 benches/） |
| A-8 | `publish = false` | PASS | 各 `Cargo.toml` 显式关闭 |
| A-9 | 实现真实 I/O / adapter 业务 | **部分（storage P0）** | 7 storage 生产客户端 + live 内容断言（真凭据 2026-07-22 已验）；exchange 交易 / Cluster / JetStream / EOS / multipart **OPEN** |
| A-10 | package stable / Spec Approved 本仓宣称 | OPEN | **禁止**用镜像 COMPLETE 代替；P0 生产入口 ≠ package stable |
| A-11 | contracts workspace 注册 | PASS | #43 `crates/contracts` → `xhyper-contracts` |
| A-12 | `FOUNDATIONX_*` 环境注入 + 密钥不入库 | PASS | `from_env` + live tests；`scripts/live/build-foundationx-env.mjs`（#191）；secrets 仅进程 env |
| A-13 | 有界 benches（`cargo test --all-targets` 不挂） | PASS | #190：kafka/nats/clickhouse/taos hot_path 3s 超时；redis/postgres/oss 既有 |

## 与镜像文档的关系

- `.agents/ssot/adapters/**`：只读镜像；禁止本地改 CLOSED/COMPLETE 叙事冒充同步
- 镜像内验证命令仍写 `.agent/SSOT/...`（上游 monorepo 路径）；**本仓**以 `.agents/ssot/adapters/...` 与 `crates/adapters/...` 为准
- 实现 SSOT 以 **源码 + 本仓测试输出** 为准
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`

## 依赖与边界（本仓意图）

```text
adapters/*  →  contracts / kernel（+ 外部 SDK：redis/tokio-postgres/rskafka/async-nats/reqwest…）
               禁止  kernel/types 反向依赖 adapters
               禁止  把 adapter 当 L0
```

- **storage 生产路径**依赖对应驱动（见各 crate `Cargo.toml`）；`scaffold` feature 保留 mock 面
- exchange 仍以 mock + 可选 HTTP 为主；真实交易 / 签名 **默认禁止**，直至 Spec + 证据批准

## `crates/contracts` 评估结论

| 问题 | 结论 |
|------|------|
| 是否应并入 workspace？ | **已并入**（#43），无需再次注册 |
| package | `xhyper-contracts` · path `crates/contracts` |
| 角色 | adapter trait 出口；**不是**上游 `.agents/ssot` 独立域 |
| 实现深度 | trait + 共享类型 scaffold |
| 标准布局 / publish | 本 PR 补齐 |
| 风险 | `Ticker` 使用 `f64` 金额字段 — 与 decimalx 禁令冲突；**禁止**宣称 stable 直至收口 |

## 未做（follow-up / OPEN）

1. Cluster / Sentinel / Streams / JetStream / EOS / multipart / native protocol — **DEFER**（非 P0）
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
| live `#[ignore]` 真凭据 | **PASS（本地）** | ZoneCNH `secrets/env/dev.md` + `/etc/nats/nats.conf` overlay；7 包全绿 |
| benches 有界 | **PASS** | #190 超时包装；`--all-targets` 不挂死 |
| `FOUNDATIONX_*` env 构建 | **PASS** | #191 `scripts/live/build-foundationx-env.mjs` |
| package stable / 全业务 Production Ready | **OPEN** | 禁止宣称 |

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p redisx --test live_kv -- --ignored --nocapture
cargo test -p postgresx --test live_postgres -- --ignored --nocapture
cargo test -p kafkax --test live_event_bus -- --ignored --nocapture
cargo test -p natsx --test live_event_bus -- --ignored --nocapture
cargo test -p ossx --test live_object_store -- --ignored --nocapture
cargo test -p clickhousex --test live_smoke -- --ignored --nocapture
cargo test -p taosx --test live_smoke -- --ignored --nocapture
```

## 跟进（历史 · 2026-07-21 / PR #98 · asa.5 · s9t）

| 项 | 状态 |
|----|------|
| scaffold 签名适配 | **PASS**：EventBus/PubSub 流项为 `BusMessage`；TxRunner `begin_tx` |
| mock-first 离线 CI | **PASS**：默认无外部服务 |
| redis live KV（s9t 子集） | **PASS**（后由 #188 升级为默认生产 `RedisPool`） |
| exchange 只读 `server_time` | **PASS（入口）** |

## SSOT 树补充（2026-07-22）

| 路径 | 说明 |
|------|------|
| `adapters/README.md` | 九域索引 + storage P0 表 |
| `storage/*/plan/infra-rs-landing.md` | 本仓生产落地说明 |
| `storage/*/plan/infra-rs-draft-spec-goal.md` | draft SPEC_GOAL 入库快照 |
