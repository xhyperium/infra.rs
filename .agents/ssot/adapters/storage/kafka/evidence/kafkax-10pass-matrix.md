# kafkax 十轮深度对照矩阵（2026-07-23）

> 与 `.cargo/draft/kafkax_SPEC_GOAL.md`、本树 SSOT、`crates/adapters/storage/kafka` 对齐。  
> **「100%」裁定**：矩阵全覆盖 + **可交付面** 100% 闭合 + **不可交付面** 100% 显式 NO-GO/OOS。  
> 不是实现 draft 全文（rdkafka P1/P2 + Part2 量化栈）。

## R1–R10 审查记录

| 轮次 | 维度 | 关键裁定 |
|------|------|----------|
| R1 | 驱动边界 | draft `rust-rdkafka` → 本仓 **`rskafka`** 合法漂移；group/txn 改 NO-GO |
| R2 | EventBus AMO / 可靠 API | AMO PASS；`AtLeastOnceConsumer` 应用 ALO 有条件 PASS |
| R3 | Producer / delivery | publish 等确认 PASS；acks=all / 幂等 / native txn **NO-GO** |
| R4 | Consumer / ack | 手动分区 + 应用 checkpoint PASS；group/rebalance **NO-GO** |
| R5 | TLS / SASL | TLS+CA+PLAIN+远程明文 fail-closed PASS；SCRAM/OAuth/mTLS **NO-GO** |
| R6 | deadline / 背压 / close | PASS |
| R7 | 错误 / 可观测 | 脱敏 + kind PASS；pool stats（含 timeouts/topics）PASS；consumer lag 属 group **NO-GO** |
| R8 | 测试矩阵 | 离线 + 隔离 harness + 真 secrets live PASS；24h soak **NO-GO** |
| R9 | SSOT 完整性 | 11 层齐全；本轮同步 version/matrix/NO-GO/OOS |
| R10 | Part2 | 量化栈 **全套 OOS**；与本仓实现合同无矛盾 |

## 条款矩阵（收敛）

| ID | 条款摘要 | 状态 | 证据 |
|----|----------|------|------|
| A-0 | 真 broker 默认路径（非仅 scaffold） | PASS | `KafkaPool` + default features |
| A-CFG | `KafkaConfig` / `from_env` | PASS | `src/config.rs` |
| A-BLD | `KafkaConfigBuilder` | PASS | `KafkaConfigBuilder::build` + 单测 |
| A-POOL | connect/producer/consumer/health/stats/close | PASS | `src/pool.rs` + live health |
| A-PROD | publish 等 delivery report | PASS | `src/producer.rs` + live |
| A-MSG | `KafkaMessage`/`Delivery`/bus id | PASS | `src/message.rs`（含 timestamp） |
| A-EB | EventBus AMO | PASS | `bus` + broker_conformance + live |
| A-ALO | 单 owner 应用 ALO | PASS 有条件 | `at_least_once` + conformance |
| A-PTC | produce-then-checkpoint 重复窗口 | PASS 非原子 | `eos` + conformance |
| A-OFF | Memory/File offset store | PASS 有条件 | `offset.rs` 单测 |
| A-TLS | TLS + 可选 PEM CA | PASS | pool + tls harness |
| A-PLAIN | SASL PLAIN | PASS | config + live secrets |
| A-REMOTE | 远程明文 fail-closed | PASS | config 单测 |
| A-LIFE | deadline / 有界背压 / close | PASS | `lifecycle` 单测 |
| A-ERR | 错误脱敏 + kind | PASS | `error_map` |
| A-DOC | usage/config/operations | PASS | `docs/` |
| A-BENCH | hot_path 有界 | PASS | `benches/hot_path.rs` |
| GROUP | consumer group coordinator | **NO-GO** | rskafka 无 |
| REB | rebalance / generation fencing | **NO-GO** | — |
| RECON | 自动重连 | **NO-GO** | fetch 错结束流 |
| EOS-N | native transactional EOS | **NO-GO** | 仅 PTC 别名 |
| SCHEMA | schema registry | **NO-GO** | — |
| SCRAM | SCRAM / OAuth / mTLS | **NO-GO** | 仅 PLAIN |
| HA | multi-broker 故障证明 | **NO-GO** | 单节点证据 |
| STABLE | package stable / crates.io | **NO-GO** | 未宣称 |
| P2-* | Part2 量化（proto/embedded/io-uring/µs） | **OOS** | draft L160+ |

## 本会话验证摘要（脱敏）

| 命令 | 结果 |
|------|------|
| `cargo test -p kafkax --all-targets` | 离线全绿 |
| `cargo test -p kafkax --test live_event_bus -- --ignored` | **3/3 PASS**（`FOUNDATIONX_KAFKAX_*` 真配置） |
| `node scripts/kafka-broker-conformance.mjs` | **3/3 PASS** |
| `node scripts/kafka-tls-sasl-conformance.mjs` | **2/2 PASS** |
| secrets | 仅 scratch `0600`；仓库无密码 |

## 收敛结论

在「rskafka 可交付面 + 其余显式 NO-GO/OOS」定义下，**可交付面闭合**；draft 幻想能力不实现、不宣称。

## 增量 0.3.7（gap 清零）

| 条款 | 状态 |
|------|------|
| headers 读写 | PASS |
| key produce 公共 API | PASS |
| stats 扩展 | PASS |
| selfcheck 同源 produce | PASS |
| NO-GO 表 | 不变 CLOSED |
