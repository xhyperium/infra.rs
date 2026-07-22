# adapters — 本仓 SSOT

> **SSOT 根**：`.agents/ssot/adapters/`  
> **保留层级**：`adapters/{exchange,storage}/`（勿展平到 `.agents/ssot/` 根）  
> **R7**：规格 COMPLETE **≠** 本仓 ship；以 members + 测试为准

## 子域一览

### exchange

| 域 | SSOT | package | 本仓状态 |
|----|------|---------|----------|
| binance | `exchange/binance/` | `binancex` | scaffold + mock HTTP + `server_time` |
| okx | `exchange/okx/` | `okxx` | scaffold + mock HTTP + `server_time` |

### storage（P0 生产默认路径 · #188–#191）

| 域 | SSOT | package | 生产面 | landing |
|----|------|---------|--------|---------|
| redis | `storage/redis/` | `redisx` | RedisPool / RedisClient | [landing](storage/redis/plan/infra-rs-landing.md) |
| postgres | `storage/postgres/` | `postgresx` | PostgresPool / PgTransaction | [landing](storage/postgres/plan/infra-rs-landing.md) |
| kafka | `storage/kafka/` | `kafkax` | KafkaPool / Producer / Consumer | [landing](storage/kafka/plan/infra-rs-landing.md) |
| nats | `storage/nats/` | `natsx` | NatsPool / EventBus | [landing](storage/nats/plan/infra-rs-landing.md) |
| oss | `storage/oss/` | `ossx` | OssClient（OSS V1） | [landing](storage/oss/plan/infra-rs-landing.md) |
| clickhouse | `storage/clickhouse/` | `clickhousex` | ClickHousePool HTTP | [landing](storage/clickhouse/plan/infra-rs-landing.md) |
| taos | `storage/taos/` | `taosx` | TaosPool REST | [landing](storage/taos/plan/infra-rs-landing.md) |

每个 storage 域另有：

- `plan/infra-rs-draft-spec-goal.md` — 自 `.cargo/draft/*_SPEC_GOAL.md` 入库的只读快照
- 标准 11 层 + `README.md`

## 硬限制

1. 禁止在 SSOT 树写实现副本（`src/` / `Cargo.toml` / `*.rs`）。
2. **未**宣称 package stable / Cluster·JetStream·EOS / crates.io。
3. live 凭据仅进程 env：`scripts/live/build-foundationx-env.mjs`。

## 验证

```bash
test -f .agents/ssot/adapters/README.md
for d in redis postgres kafka nats oss clickhouse taos; do
  test -f .agents/ssot/adapters/storage/$d/plan/infra-rs-landing.md
  test -f .agents/ssot/adapters/storage/$d/plan/infra-rs-draft-spec-goal.md
  test -f .agents/ssot/adapters/storage/$d/spec/spec.md
done
cargo test -p redisx -p postgresx -p kafkax -p natsx -p ossx -p clickhousex -p taosx --all-targets
```

## 对齐

- [docs/ssot/adapters-ssot-alignment.md](../../docs/ssot/adapters-ssot-alignment.md)
- [docs/ssot/draft-gap-matrix.md](../../docs/ssot/draft-gap-matrix.md)
