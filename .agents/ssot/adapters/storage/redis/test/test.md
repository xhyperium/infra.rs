# adapters/storage/redis — Test 合同

## 离线（默认 CI）

```bash
cargo test -p redisx --all-targets
cargo test -p redisx --all-targets --features pubsub
cargo clippy -p redisx --all-targets --features pubsub -- -D warnings
```

覆盖期望：config 校验/脱敏、pool close、error map、公共 API、Pub/Sub 配置同源与拓扑失败关闭、
只读/写入重试分类、原子性边界、non-retryable 失败只执行一次。

## Live（可选 · 真凭据）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p redisx -- --ignored --nocapture
```

- live 文件：`tests/live_kv.rs · tests/live_kv_conformance.rs`
- 端口提示：6379
- 真实 Cluster / Sentinel / TLS 未提供受控环境时保持 OPEN，不得运行生产端点

## Bench

```bash
cargo test -p redisx --bench '*' -- --nocapture   # 或 cargo bench -p redisx
```

`benches/kv_hot_path.rs` 必须有界，避免 `--all-targets` 挂死。
