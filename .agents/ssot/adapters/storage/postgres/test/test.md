# adapters/storage/postgres — Test 合同

## 离线（默认 CI）

```bash
cargo test -p postgresx --all-targets
cargo clippy -p postgresx --all-targets -- -D warnings
```

覆盖期望：config 校验/脱敏、pool close、error map、消息/ID 编解码（若有）、公共 API 引用。

## Live（可选 · 真凭据）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p postgresx -- --ignored --nocapture
```

- live 文件：`tests/live_postgres.rs`
- 端口提示：5432

## Bench

```bash
cargo test -p postgresx --bench '*' -- --nocapture   # 或 cargo bench -p postgresx
```

`benches/query_hot_path.rs` 必须有界，避免 `--all-targets` 挂死。
