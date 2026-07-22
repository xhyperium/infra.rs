# adapters/storage/oss — Test 合同

## 离线（默认 CI）

```bash
cargo test -p ossx --all-targets
cargo clippy -p ossx --all-targets -- -D warnings
```

覆盖期望：config 校验/脱敏、pool close、error map、消息/ID 编解码（若有）、公共 API 引用。

## Live（可选 · 真凭据）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p ossx -- --ignored --nocapture
```

- live 文件：`tests/live_object_store.rs`
- 端口提示：443 (HTTPS)

## Bench

```bash
cargo test -p ossx --bench '*' -- --nocapture   # 或 cargo bench -p ossx
```

`benches/put_get.rs` 必须有界，避免 `--all-targets` 挂死。
