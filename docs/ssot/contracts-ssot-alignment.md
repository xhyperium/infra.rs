# contracts SSOT 对齐矩阵

| 字段 | 值 |
|---|---|
| 审计日期 | 2026-07-23 |
| baseline | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| Active Spec | `.agents/ssot/contracts/spec/spec.md`（与 xhyper 镜像 byte-identical） |
| crate | `crates/contracts` · package/lib `contracts` |
| version | `0.1.2`（非 package stable；非整体 Production Ready） |
| 兼容 | 15 traits Additive Only；本轮无方法删除/签名变更 |

## 事实纠偏

contracts 禁止具体 backend adapter 实现，但允许不绑定 backend 的轻量 orchestration helper。`LiveContractProfile` 只是接线意图；`LiveHandles::validate` 只是引用形状校验，二者都不是 readiness attestation。

| ID | 要求 | 状态 | 证据 |
|---|---|---|---|
| CT-API-1 | 15 traits 保持 Additive Only | IMPLEMENTED | `src/lib.rs`；消费者/API ratchet 待最终门禁 |
| CT-LIVE-1 | kv/bus/tx/venue 声明需要对应句柄 | IMPLEMENTED | `LiveHandles::validate` |
| CT-LIVE-2 | repo/account/time 不得由其他句柄推断 | IMPLEMENTED | 三 flag fail-closed 公共测试 |
| CT-BUS-1 | publish 不暗示 E2E | IMPLEMENTED | `publish_without_delivery_attestation`；失败传播测试 |
| CT-TX-1 | KV+Tx 不暗示跨资源原子性 | IMPLEMENTED | `kv_set_then_commit_separate_resources`；失败传播测试 |
| CT-TESTKIT | Fake/suite 仍为独立 dev-only crate | CANDIDATE | contract-testkit 0.1.2：14 trait / 15 broken case；contracts trait 无签名变更 |

兼容名称 `bus_publish` / `tx_kv_set` 保留并转调准确命名 helper。成功分别只表示 producer call 返回 Ok、以及独立 KV set 后 TxContext commit 返回 Ok；不证明消费、确认、持久性或原子回滚。

## 生产消费者

直接消费者包括 bootstrap、observex、resiliencx、contract-testkit 以及 exchange/storage adapters；本轮只同步 `contracts = { path, version = "0.1.2" }`，不 bump 无逻辑变更的消费者。具体实现继续位于各 adapter crate。

## contract-testkit 交叉验证（0.1.2 候选）

- 新增 ObjectStore、TimeSeriesStore、AnalyticsSink、PubSub suite，并以 `FixtureNamespace` 隔离资源名。
- AnalyticsSink / Instrumentation observed suite 使用调用方观察 seam；不改变本 crate 15 个生产 trait。
- EventBus / PubSub suite 明确只做操作 smoke，不将 Fake buffered 行为外推为 delivery 合同。
- `check-test-support-graph.mjs` 从 cargo metadata 检查 default/all-features normal/build 闭包；dev edge 不计生产污染。

## OPEN / NO-GO

- ObjectStore/TimeSeries/PubSub/Analytics 的 Sandbox/Real 深度 conformance 与 live evidence。
- VenueAdapter 强制 compile-fail override gate。
- 交易所签名、下单、WS 行情等完整业务 live。
- 跨 backend 事务原子性与 EventBus E2E delivery。
- first-batch 全绿 L3、workspace Production Ready、Agent/Maintainer L5。

最终命令与退出码写入 `.agents/ssot/contracts/evidence/README.md`。Fake、helper 或 handles 通过不得升级为真实后端 readiness；maintainer/独立 reviewer 未签署前 release 保持 BLOCKED。
