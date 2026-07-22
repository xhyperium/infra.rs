# adapters/storage/clickhouse — Test 合同

## 离线（默认 CI）

```bash
cargo test -p clickhousex --all-targets
cargo clippy -p clickhousex --all-targets -- -D warnings
node scripts/clickhouse-https-conformance.mjs
```

覆盖期望：config 校验/脱敏、`HTTP_PORT`/`PORT` 冲突、远程 HTTP fail-closed、
pool close、固定错误码映射、错误正文不泄漏、公共 API 引用。

`tests/security_failures.rs` 只使用 loopback 临时 HTTP 服务；
`tests/https_conformance.rs` 由脚本生成临时 CA/证书。二者都不证明真实 ClickHouse 集群。

## Live（可选 · 真凭据）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p clickhousex -- --ignored --nocapture
```

- live 文件：`tests/live_smoke.rs`
- 端口由 `HTTP_PORT`（兼容 `PORT`）注入；远程环境必须 TLS
- 未在当前证据中运行；TLS/auth/deadline/并发 live 保持 OPEN

## Bench

```bash
cargo test -p clickhousex --bench '*' -- --nocapture   # 或 cargo bench -p clickhousex
```

`benches/hot_path.rs（3s 有界）` 必须有界，避免 `--all-targets` 挂死。
