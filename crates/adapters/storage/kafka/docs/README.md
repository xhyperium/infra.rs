# kafkax docs

| 文档 | 说明 |
|------|------|
| [usage.md](usage.md) | 快速使用 |
| [config.md](config.md) | 配置与环境变量 |
| [operations.md](operations.md) | 运维与 live |
| [测试矩阵-生产发布.md](测试矩阵-生产发布.md) | 基准/功能/集成/可靠性/安全…完整清单与命令 |
| [../../../../docs/ssot/kafkax-ssot-alignment.md](../../../../docs/ssot/kafkax-ssot-alignment.md) | SSOT 对齐 |

## 生产默认面

KafkaPool / Producer / Consumer

密钥仅 `FOUNDATIONX_*` 环境变量；默认 `cargo test` 离线绿灯；live 测试 `#[ignore]`。

## 测试入口

```bash
cargo test -p kafkax --all-targets
cargo test -p kafkax --test prod_offline
node scripts/kafka-prod-matrix.mjs
node scripts/kafka-prod-matrix.mjs --fault-restart
KAFKAX_SOAK_SECONDS=60 node scripts/kafka-prod-matrix.mjs --soak
cargo bench -p kafkax --bench hot_path -- --quick
node scripts/kafka-broker-conformance.mjs
node scripts/kafka-tls-sasl-conformance.mjs
```
