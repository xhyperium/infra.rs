# adapters/storage/oss — Test 合同

## 离线（默认 CI）

```bash
cargo test -p ossx --all-targets
cargo clippy -p ossx --all-targets -- -D warnings
```

覆盖期望：HTTPS fail-closed、配置脱敏/硬上界、Semaphore acquire/close、chunked body
限额、retry deadline、multipart XML/part/count/abort/orphan、公共 API 引用。
loopback HTTP 状态机必须覆盖 initiate 成功后取消 → registry 记录 → 显式 abort 清理，以及多片
共享单一总 deadline；仅测 helper 不足以算 PASS。

禁止把 ignored live 当默认 PASS；默认测试不得读取凭据或访问生产环境。

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
