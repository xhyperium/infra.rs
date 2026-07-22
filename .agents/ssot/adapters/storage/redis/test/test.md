# adapters/storage/redis — Test 合同

## 离线（默认 CI）

```bash
cargo test -p redisx --all-targets
cargo clippy -p redisx --all-targets -- -D warnings
```

覆盖期望：config 校验/脱敏、pool close、error map、消息/ID 编解码（若有）、公共 API 引用。

## Live（可选 · 真凭据）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p redisx -- --ignored --nocapture
```

- live 文件：`tests/live_kv.rs · tests/live_kv_conformance.rs`
- 端口提示：6379

## Bench

```bash
cargo test -p redisx --bench '*' -- --nocapture   # 或 cargo bench -p redisx
```

`benches/kv_hot_path.rs` 必须有界，避免 `--all-targets` 挂死。
