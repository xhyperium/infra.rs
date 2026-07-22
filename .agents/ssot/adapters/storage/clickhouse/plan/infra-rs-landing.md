# infra.rs 本仓落地说明 — clickhousex

| 字段 | 值 |
|------|-----|
| package | `clickhousex` |
| 实现路径 | `crates/adapters/storage/clickhouse` |
| version | `0.3.2` |
| 生产默认面 | ClickHousePool HTTP(S) + 可选 PEM CA |
| scaffold | `feature = "scaffold"`（可选 mock） |
| live | `tests/live_smoke.rs`（默认 `#[ignore]`） |
| 凭据 | `FOUNDATIONX_*` via `scripts/live/build-foundationx-env.mjs` |
| PR | #188 · #189 · #190 · #191 |
| 对齐 | [docs/ssot/adapters-ssot-alignment.md](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| package stable | **未宣称** |

## 硬限制

1. 本文件描述 **infra.rs 本仓 P0 生产入口**，不是 monorepo 战役 COMPLETE。
2. Cluster / JetStream / EOS / multipart 等 **DEFER**。
3. 无 live 证据不得宣称“全后端 Production Ready”。
4. 本地 HTTPS conformance 只证明客户端 CA/主机名 fail-closed，不等于真实 ClickHouse live。

## 验证

```bash
cargo test -p clickhousex --all-targets
node scripts/clickhouse-https-conformance.mjs
cmp .agents/ssot/adapters/storage/clickhouse/spec/spec.md \
  .agents/ssot/adapters/storage/clickhouse/spec/xhyper-clickhousex-complete-spec.md
# live:
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p clickhousex -- --ignored
```
