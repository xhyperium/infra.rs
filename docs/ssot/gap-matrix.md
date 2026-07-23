# Gap Matrix — .cargo/draft → infra.rs (2026-07-23)

> **权威快照**：storage×7 OBJECTIVE DEFER 已闭合（生产默认就绪）。  
> package stable / crates.io **仍未**宣称。

| Domain | Draft DoD P0 | Current | Deferred (not OBJECTIVE / not stable) |
|--------|--------------|---------|----------------------------------------|
| postgresx | Pool+query+tx+TLS | **done** + Repository + 远程 Require live（CA+SNI）+ deadline + Migrator+COPY+mTLS+**selfcheck §6.1**+live (`0.3.12`) | 无限流式 COPY / read-replica / package stable / 服务端强制 mTLS live / down migration |
| redisx | Pool+KV+structures+Streams+tx+live | **done** 全公开 API + selfcheck + live/E2E/bench (`0.3.14`) | Cluster/Sentinel/TLS live（无 env）/ package stable / PubSub NO-GO |
| postgresx | Pool+query+tx+TLS | **done** + Repository + 远程 Require live（CA+SNI）+ deadline + Migrator+COPY+mTLS+**selfcheck §6.1**+live + 合同文档对齐 (`0.3.13`) | 无限流式 COPY / read-replica / package stable / 服务端强制 mTLS live / down migration / channel binding |
| kafkax | Producer pool + EventBus | **done** + 生产测试矩阵 offline/reliability/bench/fault (`0.3.5`) | SCRAM 成功路径/group/rebalance/native EOS/DLQ/Part2 OOS/24h 默认 soak |
| natsx | Core NATS EventBus | **done** + JetStream durable pull/显式确认 + 同客户端重启恢复 3/3 (`0.3.2`) | 断线窗口无回放 / NKey / Cluster/HA / 自动 DLQ / KV·Object 全量 |
| ossx | ObjectStore put/get | **done** + 有界 multipart/retry/orphan 补偿 (`0.3.2`) | lifecycle / STS / TB 流式对象 |
| clickhousex | Analytics insert+select | **done** + HTTPS/PEM CA + insert_batch + 有界池 (`0.3.2`) | 真实集群 TLS / mTLS / native 9000 / cluster 运维 |
| taosx | TimeSeries write+query | **done** Production-default 全 API + selfcheck Full + e2e/bench (`0.3.10`)；gap 未完成=0 | —（见 taosx-gap-register SUPERSEDED 行） |
| goalctl | Goal→Contract digest | **done** | full multi-module authority plane |
| verifyctl | plan+execute+run-result | **done** | full V0–V3 gate matrix |

Freeze: production-default path per domain; scaffold behind `scaffold` feature; no secrets in git; **no** package-stable claim.

## Live 入口

```bash
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo test -p redisx -p postgresx -p kafkax -p natsx \
    -p ossx -p clickhousex -p taosx -- --ignored
```

## 相关 PR

| PR | 说明 |
|----|------|
| #188–#191 | storage 生产客户端 + live 凭据 |
| #195 | storage×7 SSOT layers |
| #211 | storage×7 OBJECTIVE DEFER 闭合 → `0.3.1`/`0.3.2` (redis/postgres) |
| #219 | redisx/postgresx SSOT version 行对齐 `0.3.2` |
