# adapters/storage/kafka — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `kafkax` |
| 标题 | Kafka EventBus |
| 实现 | `crates/adapters/storage/kafka` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 |
| 状态 | **P0 生产入口已落地**；十轮矩阵已收敛；package stable **未宣称** |
| version | `0.3.7` |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 Kafka EventBus 生产默认客户端，满足 draft **可交付面** DoD，且默认 `cargo test` 离线绿灯。不可交付面（group/EOS/Part2）100% 显式 NO-GO/OOS。

## Acceptance（本仓可验证）

1. workspace member `kafkax` 可 `cargo test -p kafkax --all-targets`
2. 生产默认面：`KafkaPool / KafkaProducer / KafkaConsumer / KafkaEventBus / KafkaConfigBuilder`
3. 环境注入：`FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}`（密钥不入库）
4. live：`tests/live_event_bus.rs` 默认 `#[ignore]`，真凭据可绿
5. bench：`benches/hot_path.rs` 有界（不得挂死 `--all-targets`）
6. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认
7. 外部 I/O 有 deadline；有界消费队列可被 close 打断，close 超时后保持关闭
8. 十轮条款矩阵落盘：`evidence/kafkax-10pass-matrix.md`

## Not in scope / NO-GO / OOS

- **NO-GO**：EOS / transactional producer / schema registry / group coordinator / 自动重连 / SCRAM·OAuth·mTLS / package stable
- **OOS**：draft Part2 量化栈（embedded broker / io-uring / µs 热路径 / kafkax-proto）

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 十轮矩阵：[../evidence/kafkax-10pass-matrix.md](../evidence/kafkax-10pass-matrix.md)
- 对齐：[docs/ssot/kafkax-ssot-alignment.md](../../../../../docs/ssot/kafkax-ssot-alignment.md)
