# contracts SSOT 对齐

| 字段 | 值 |
|---|---|
| package / lib | `contracts` / `contracts`（Cargo 选择器 `-p contracts`） |
| path / version | `crates/contracts` / `0.1.2` |
| Active Spec | `.agents/ssot/contracts/spec/spec.md` |
| 本轮 | 2026-07-22 第2轮：事务生命周期、capability fail-closed、portable suites |
| 状态 | R4 trait 面；**不宣称**业务原子事务、live PASS、package stable 或 Agent L5 |

## 结论摘要

| 问题 | 当前裁定 |
|---|---|
| 16 trait 出口 | 已落地；`TxContext: Send`，其余当前为 `Send + Sync` |
| `TxRunner` 对象安全 | PASS：无 generic method，返回 `Box<dyn TxContext>` |
| 事务错误闭合 | PASS：`run_tx_lifecycle` + `TxRunError` 四分支 |
| 业务原子性 | NO-GO：context 没有 SQL/Repository/KV 操作面 |
| 旧事务 helpers | deprecated 兼容；保持旧签名/错误映射，不作为生产推荐 |
| Live capability | fail-closed：repo/account/venue_time 无句柄槽时拒绝 |
| contract-testkit | Fake + 15 个 suite 入口；portable 与 snapshot profile 分层 |
| 真实 adapter suite | OSS/TAOS/ClickHouse/Redis ignored target **可编译**；live **NOT RUN** |
| exchange 业务 live | NO-GO：本轮不提供交易签核证据 |

## 本仓可观察事实

```text
crates/contracts/
  src/lib.rs                    16 traits / TxRunError / run_tx_lifecycle
  src/live.rs                   LiveContractProfile / LiveHandles fail-closed
  docs/contracts/*.md           trait 语义与非保证边界

crates/test-support/contracts/
  src/fakes/                    Fake / Recording / Batch-2
  src/suite/                    per-trait suites
  portable                      不承诺 replay、必达、排序或后端专有语义
  snapshot                      仅进程内快照/回放 Fake profile

真实 adapter ignored 调用入口：
  oss/tests/live_object_store.rs
  taos/tests/live_smoke.rs
  clickhouse/tests/live_smoke.rs
  redis/tests/live_pubsub_conformance.rs（feature pubsub）
```

## 条款矩阵

| ID | 条款 | 状态 | 证据 |
|---|---|---|---|
| CT-1 | R4 依赖白名单 / 无 adapter 实现 | PASS | manifest + workspace deps gate |
| CT-2 | `TxRunner` / `TxContext` 对象安全 | PASS | public surface + dyn 单元测试 |
| CT-3 | Begin / Business / Rollback / Commit 分类 | PASS | `TxRunError` 分支测试 |
| CT-4 | business + rollback 双错误均保留 | PASS | kind/context 分别断言 |
| CT-5 | commit 失败不自动 rollback | PASS | rollback 计数为 0 |
| CT-6 | 旧 helper 兼容 | PASS | 局部 `allow(deprecated)` 兼容测试 |
| CT-7 | 外部业务操作不冒充原子 | PASS | helper 签名不传 context + spec 禁止声明 |
| CT-8 | 七个 capability flag 可验证 | PASS | 四槽缺失 + 三个无槽 flag 全部 fail-closed |
| CT-9 | Batch-2 四 suite | PASS | Fake self-test |
| CT-10 | portable / snapshot 分型 | PASS | EventBus surface 与 snapshot 入口分离 |
| CT-11 | 真实 adapter 调用证据 | COMPILE PASS | 四个 ignored test target；live 未运行 |
| CT-12 | public API additive | 待最终门禁 | API baseline + 统一 PATCH bump |

## 事务边界

`run_tx_lifecycle` 只表达以下顺序：

```text
begin ── work Ok  ── commit
      └─ work Err ── rollback
```

- work 闭包不接收 `TxContext`，不能通过类型取得事务绑定业务能力；
- `BusinessAndRollback` 同时保留两个 `XError`；
- `Commit` 表示结果可能未知，禁止自动 rollback 或无条件重试；
- 取消/panic 只会 drop context，本合同不保证异步 rollback 完成；
- `tx_kv_set` 的 KV 来自独立对象，因此不是原子 KV 事务。

真正的业务原子面必须新增 tx-bound resource，并同时提供 ADR、staged Fake、真实
PostgreSQL 同连接/不可见性/全有全无证据；本轮刻意不创建未验证 UoW seam。

## Conformance 边界

- ObjectStore：唯一 key 的 put/get 精确字节往返；cleanup 由 adapter 负责。
- TimeSeriesStore：后端精度已对齐的单点 write/query；不假定顺序或无历史数据。
- AnalyticsSink：只验证接受成功；持久化由 adapter 后端查询证明。
- PubSub/EventBus surface：先 subscribe 后 publish，但不 poll、不承诺必达/replay/order。
- `assert_event_bus` 是 snapshot/replay Fake profile，不能套用 Kafka/NATS 实时订阅。

ignored target 存在和编译成功只记为 `COMPILE PASS`。没有受控环境实际运行输出时，
live 状态保持 `NOT RUN`，不得计作 Production Ready 证据。

## 未闭合 / 后续

- tx-bound Repository/UoW 与 PostgreSQL 真实原子性、取消和 commit outcome 分类；
- Kafka/NATS delivery、ack、redelivery、崩溃矩阵与 broker conformance；
- ObjectStore/TimeSeries/Analytics/PubSub 的后端专有持久化、清理、重连和背压；
- 交易所交易业务 live、人工签核与 Agent L5；
- 整 PR 最终只对受影响 crate PATCH bump 一次并更新所有 path version。

## 验证

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

## 变更记录

| 日期 | 说明 |
|---|---|
| 2026-07-22 | 第2轮：冻结事务生命周期非原子边界、结构化双失败、capability fail-closed、portable suites |
| 2026-07-22 | 既有 Batch-2 Fake / BackendProfile 与 live helpers 落地 |
