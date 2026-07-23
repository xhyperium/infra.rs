# kafkax SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `kafkax` |
| SSOT | `.agents/ssot/adapters/storage/kafka/` |
| 实现 | `crates/adapters/storage/kafka` |
| 审计日期 | 2026-07-23 |
| version | `0.3.5` |
| 结论 | **AMO、单 owner 应用 ALO、TLS+CA+SASL/PLAIN 有隔离 broker 与真 secrets live 证据**；生产测试矩阵（离线+集成+bench+故障重建）已落地；group/rebalance/自动重连/native EOS **NO-GO**；Part2 量化栈 **OOS**；**未**宣称 package stable |

## 语义矩阵

| 入口 | 状态 | 证据与边界 |
|------|------|------------|
| `KafkaEventBus` | AMO PASS | `contracts::EventBus` 无 ack handle |
| `AtLeastOnceConsumer` | 有条件 PASS | 单分区、单 owner、应用自管 checkpoint；无 group/fencing |
| `ProduceThenCheckpointCoordinator` | 非原子 PASS | produce 成功/checkpoint 失败会重复；不称 EOS |
| `FileOffsetStore` | 有条件 PASS | 单进程同目录 temp+sync+rename+父目录 sync；不承诺多进程/绝对掉电持久 |
| TLS / CA | PASS | rskafka rustls transport；webpki roots + 可选 PEM CA；远程明文 fail-closed |
| SASL | 有条件 PASS | 仅 PLAIN；未知/不完整机制 fail-closed |
| deadline | PASS | connect（含 CA 装载）/metadata/topic/partition client/produce 均有界 |
| backpressure / close | PASS | consumer/EventBus 有界队列；close 取消等待并按 deadline 等待在途操作 |
| 错误边界 | PASS | 稳定 `ErrorKind`；驱动原文、SASL 用户名/密码不进入公开错误或 Debug |
| `KafkaConfigBuilder` | PASS | 链式构建 + `build()` 校验 |
| group/rebalance/自动重连/native EOS/DLQ | **NO-GO** | 没有对应实现或证据 |
| draft Part2 量化栈 | **OOS** | 见 `evidence/kafkax-10pass-matrix.md` |

旧 `EosCoordinator` / `EosSession` 仅为 deprecated 源码兼容别名。内置 checkpoint 单调前进，拒绝负 offset 与 `i64::MAX` 溢出。

## 十轮对照

条款级矩阵与 R1–R10 记录：

- [`.agents/ssot/adapters/storage/kafka/evidence/kafkax-10pass-matrix.md`](../../.agents/ssot/adapters/storage/kafka/evidence/kafkax-10pass-matrix.md)

「100%」= 矩阵全覆盖 + 可交付面闭合 + 不可交付面显式 NO-GO/OOS。

## 验证

```bash
cargo test -p kafkax --all-targets
cargo clippy -p kafkax --all-targets -- -D warnings
node scripts/kafka-prod-matrix.mjs
node scripts/kafka-prod-matrix.mjs --fault-restart
node scripts/kafka-broker-conformance.mjs
node scripts/kafka-tls-sasl-conformance.mjs
# 真 secrets（不入库）
node scripts/live/build-foundationx-env.mjs --env dev --out <private>/foundationx.env
set -a; source <private>/foundationx.env; set +a
cargo test -p kafkax --test live_event_bus -- --ignored --nocapture
```

### 本会话证据（2026-07-23）

| 项 | 结果 |
|----|------|
| 离线 `cargo test -p kafkax --all-targets` | PASS（含 prod_offline） |
| `kafka-prod-matrix.mjs --fault-restart` | **主场景 9 + 故障/重建 PASS** |
| 真 secrets live | **3/3 PASS**（#282 会话） |
| `kafka-broker-conformance.mjs` | **3/3 PASS** |
| TLS/SASL isolation harness | **2/2 PASS** |
| 密钥入库扫描 | 仅 env 注入；仓库无密码 |

单节点 / 真 secrets 结果不证明 group/rebalance/HA/native EOS。

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
