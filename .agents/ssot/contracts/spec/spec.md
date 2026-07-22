# `contracts` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.2` additive 契约；未宣称 package stable / Production Ready |
| Package / lib | `contracts` / `contracts`（Cargo 选择器 `-p contracts`） |
| Path | `crates/contracts` |
| Layer | Contract / R4 唯一跨层 trait 出口 |
| Authority | 本文件是 active current-state spec；源码、Cargo metadata 与测试结果优先 |
| Candidate | `draft/contracts-complete-spec.md`（Draft，非权威） |
| Implementation baseline | `350c1c2` + 本轮 contracts 变更 |
| Verified at | 2026-07-22 |

> `[KNOWN]` 仅表示当前 Cargo、源码与测试可直接证明。规格存在、ignored live
> 测试可编译或 profile flag 为 true，都不自动等于真实后端已通过生产验收。

## 1. 定位与依赖

`contracts` 只定义跨层 trait/type 与小型编排 helper，不放 adapter 实现。公开面受
Additive Only 约束。正常依赖仅为 `kernel`、`canonical`、`async-trait`、`bytes`、
`futures-core`、`thiserror`；禁止依赖 runtime、adapter、service、app 或动态实现 registry。

`contract-testkit` 位于 `crates/test-support/contracts`，只能作为 dev-dependency，
不得进入 production normal graph。

## 2. 当前 16 个公开 trait

| 领域 | Trait | 当前实现事实 | 已冻结边界 |
|---|---|---|---|
| Storage | `KeyValueStore` | redis | get/set/可选 TTL；原子性未统一 |
| Messaging | `EventBus` | kafka、nats | 无 ack handle；不承诺 replay/必达 |
| Storage | `Repository<T, Id>` | postgres | find/save；schema/一致性未统一 |
| Storage | `TxContext` | postgres、Fake | `Send`、`&mut` commit/rollback；非 `Sync` 要求 |
| Storage | `TxRunner` | postgres、Fake | `Send + Sync`、对象安全、只提供生命周期 |
| Storage | `TimeSeriesStore` | taos | write/query；顺序、精度、去重未统一 |
| Storage | `ObjectStore` | oss | put/get；metadata/overwrite/missing 未统一 |
| Analytics | `AnalyticsSink` | clickhouse | write-only 接受面；落盘需 adapter 查询证明 |
| Messaging | `PubSub` | redis 可选面 | at-most-once 最小面不等于必达 |
| Observability | `Instrumentation` | observex | retry/circuit 三方法 |
| Venue legacy | `VenueAdapter` | binance、okx | 迁移 facade；树内需覆盖 additive defaults |
| Venue | `MarketDataSource` | binance、okx | runtime stream error 未统一 |
| Venue | `InstrumentCatalog` | binance、okx | missing/cache 未统一 |
| Venue | `ExecutionVenue` | binance、okx | structured cancel/query；幂等未统一 |
| Venue | `AccountSource` | binance、okx | freshness/consistency 未统一 |
| Venue | `VenueTimeSource` | binance、okx | unit/accuracy 未统一 |

除 `TxContext: Send` 外，其余当前 trait 为 `Send + Sync`。异步 trait 使用
`#[async_trait]`；stream 为 `BoxStream<'static, T>`，item 没有运行期 `XResult`
错误通道。

## 3. 事务生命周期合同

### 3.1 对象安全与非原子边界

- `TxRunner::begin_tx(&self) -> XResult<Box<dyn TxContext>>` 没有 generic method，
  `dyn TxRunner` 与 `dyn TxContext` 均可用。
- `TxContext` 只有 commit/rollback，不暴露 SQL、Repository 或 KV 操作面。
- `run_tx_lifecycle` 的业务闭包不接收 context；它只证明
  begin → work → commit/rollback 的生命周期顺序。
- 闭包捕获的 Repository、KV、HTTP 或消息调用**不因此获得原子性**。
- Future 取消或 panic 时只 drop context；本通用合同不保证可异步等待 rollback。

真正的业务原子事务必须由后端提供 tx-bound resource，并经 ADR、Fake 与真实后端
原子性测试后 additive 新增；当前不创建未经证明的 TxRepository/GAT/UoW 公共面。

### 3.2 结构化失败

`run_tx_lifecycle` 返回 `TxRunResult<T>`：

| Variant | 语义 |
|---|---|
| `Begin` | begin 失败，业务未执行 |
| `Business` | 业务失败且 rollback 成功 |
| `BusinessAndRollback` | 同时保留 business 与 rollback 两个 `XError` |
| `Commit` | commit 失败，结果可能未知；不自动 rollback、不建议无条件重试 |

兼容符号 `run_tx_commit_on_ok`、`tx_kv_set`、`run_on_tx_context` 保留但 deprecated。
其中 `tx_kv_set` 把独立 KV 与事务生命周期顺序组合，**不是原子写入**；旧 helper
仍保持历史 `XResult` 行为并可能丢弃 rollback 错误。

## 4. Live capability fail-closed

`LiveContractProfile` 的七个 flag 是声明意图，不是运行证据。`LiveHandles` 当前只有
kv、bus、tx、venue 四个句柄槽：

- 对四个已表示槽，flag=true 且 handle=None 时返回 `Missing`；
- repo、account、venue_time 没有句柄槽，flag=true 时一律 fail-closed；
- 因此 `storage_stack()`、`venue_stack()`、`all()` 当前不能通过 `validate()`；
- 不得把 preset 或 ignored live test 的存在写成全量 live PASS。

## 5. Conformance 分层

`contract-testkit` 当前公开 15 个 suite 入口：原 10 个入口，加四个 Batch-2
portable suite 与 `assert_event_bus_surface`。分层如下：

- portable core：ObjectStore 精确字节往返、TimeSeries 单点 write/query、
  AnalyticsSink 接受面、PubSub/EventBus subscribe+publish surface；
- snapshot profile：`assert_event_bus` 的 publish→subscribe 同步回放，只适用于
  进程内快照 Fake，不得套用 Kafka/NATS 实时订阅；
- adapter-specific：持久化查询、清理、投递/超时、顺序、replay 与故障注入。

portable suite 使用调用方提供的唯一 key/table/event/channel，避免并发污染；
PubSub/EventBus portable surface 不 poll，at-most-once 不得被推导为必达。

四个真实 adapter 已有 ignored live target 接入对应 suite：OSS、TAOS、ClickHouse、
Redis PubSub。默认门禁只证明这些 target 可编译；只有受控执行后的日志才能记为
live PASS。

## 6. Additive Only 与验收

- 不修改/删除既有 trait 方法或类型；deprecated 不等于可删除。
- 新错误类型与 helper additive；旧 helper 签名和既有错误映射保持兼容。
- public API baseline 必须包含 `TxRunError`、`TxRunResult`、`run_tx_lifecycle`。
- 同一 PR 内发生行为/API 变化的 crate 最终只 PATCH bump 一次；本轮不重复 bump。

```bash
cargo test -p contracts -p contract-testkit --all-targets
cargo test -p ossx -p taosx -p clickhousex -p redisx --all-targets --all-features
cargo test -p postgresx --all-targets --all-features
cargo clippy -p contracts -p contract-testkit -p ossx -p taosx -p clickhousex -p redisx \
  --all-targets --all-features -- -D warnings
cargo clippy -p postgresx --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p contracts -p contract-testkit -p ossx -p taosx \
  -p clickhousex -p redisx --all-features --no-deps
cargo fmt --all --check
cmp .agents/ssot/contracts/spec/spec.md \
  .agents/ssot/contracts/spec/xhyper-contracts-complete-spec.md
node scripts/quality-gates/check-public-api.mjs
node scripts/quality-gates/check-ssot-current-state.mjs
node scripts/quality-gates/check-workspace-deps.mjs
git diff --check HEAD
```

通过仅表示当前声明范围闭合，不等于 workspace Production Ready、交易可用或 L5。

## 7. 追溯

- `docs/architecture/spec.md` R4/R6、§4.3、§5
- `docs/architecture/adr/001-venue-adapter-boundary.md`
- `docs/architecture/adr/005-resiliencx-observability-boundary.md`
- `crates/contracts/{src/lib.rs,src/live.rs,docs/contracts/**}`
- `crates/test-support/contracts/src/{suite,fakes}/**`
