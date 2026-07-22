# adapters/storage/nats — Test 合同

## 离线（默认 CI）

```bash
cargo test -p natsx --all-targets
cargo clippy -p natsx --all-targets -- -D warnings
```

覆盖期望：config 校验/脱敏、pool close、error map、消息/ID 编解码（若有）、公共 API 引用。

## Live（可选 · 真凭据）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p natsx -- --ignored --nocapture
```

- live 文件：`tests/live_event_bus.rs`
- 端口提示：4222

## Bench

```bash
cargo test -p natsx --bench '*' -- --nocapture   # 或 cargo bench -p natsx
```

`benches/hot_path.rs（3s 有界）` 必须有界，避免 `--all-targets` 挂死。
