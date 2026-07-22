# adapters — 本仓 SSOT

> **SSOT 根**：`.agents/ssot/adapters/`  
> **保留层级**：`adapters/{exchange,storage}/`（勿展平到 `.agents/ssot/` 根）  
> **R7**：规格 COMPLETE **≠** 本仓 ship；以 members + 测试为准

## 子域一览

### exchange

| 域 | SSOT | package | 本仓状态 |
|----|------|---------|----------|
| binance | `exchange/binance/` | `binancex` | 签名 REST + 公共 WS 解析/注入；交易 **NO-GO** |
| okx | `exchange/okx/` | `okxx` | 签名 REST + 公共 WS 解析/注入；交易 **NO-GO** |

签名请求与公共行情入口不等于可交易。精度 filters、限流、时钟偏移、私有 WS、重连和受控下单/清理证据仍 OPEN。

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
3. live 凭据仅进程 env：`scripts/live/build-foundationx-env.mjs`；除 Redis 外，当前 live 入口没有可复验 CI/留档，不得据此宣称生产证据闭合。

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

## docs/ssot 分 package

| package | 文档 |
|---------|------|
| redisx | [redisx-ssot-alignment.md](../../docs/ssot/redisx-ssot-alignment.md) |
| postgresx | [postgresx-ssot-alignment.md](../../docs/ssot/postgresx-ssot-alignment.md) |
| kafkax | [kafkax-ssot-alignment.md](../../docs/ssot/kafkax-ssot-alignment.md) |
| natsx | [natsx-ssot-alignment.md](../../docs/ssot/natsx-ssot-alignment.md) |
| ossx | [ossx-ssot-alignment.md](../../docs/ssot/ossx-ssot-alignment.md) |
| clickhousex | [clickhousex-ssot-alignment.md](../../docs/ssot/clickhousex-ssot-alignment.md) |
| taosx | [taosx-ssot-alignment.md](../../docs/ssot/taosx-ssot-alignment.md) |
