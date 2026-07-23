# adapters/storage/postgres — Test 合同

## 离线（默认 CI）

```bash
cargo test -p postgresx --all-targets
cargo clippy -p postgresx --all-targets -- -D warnings
```

覆盖期望：config 校验/脱敏、SQLSTATE、事务状态机、resiliencx、公共 API 引用、TLS 构造。

## Live（可选 · 真凭据）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p postgresx --test live_postgres -- --ignored --nocapture
node scripts/postgres-deadline-conformance.mjs
```

- live 文件：`tests/live_postgres.rs`（SELECT / tx / Repository / resiliencx / query_opt）
- deadline：`tests/deadline_conformance.rs`（固定镜像）
- 端口提示：5432（dev）

## Bench

```bash
POSTGRESX_BENCH_ITERS=200 cargo bench -p postgresx --bench query_hot_path
```

`benches/query_hot_path.rs` 必须有界，无环境时 exit 0 跳过，避免 `--all-targets` 挂死。
