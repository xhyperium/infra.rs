# adapters/storage/kafka — Test 合同

## 离线（默认 CI）

```bash
cargo test -p kafkax --all-targets
cargo clippy -p kafkax --all-targets -- -D warnings
```

覆盖期望：config 校验/凭据脱敏、pool close deadline、关闭取消有界队列背压、error map
不回显驱动原文、消息/ID 编解码（若有）、公共 API 引用。

## 隔离 broker（可选 · 非默认 CI）

可复现单节点 conformance（优先）：

```bash
node scripts/kafka-broker-conformance.mjs
node scripts/kafka-tls-sasl-conformance.mjs
```

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p kafkax -- --ignored --nocapture
```

- conformance：`tests/broker_conformance.rs`
- 受控 live：`tests/live_event_bus.rs`
- 端口提示：9092
- 任一 harness 未在当前会话运行时不得补写当前 PASS；失败日志必须脱敏且清理临时凭据

## Bench

```bash
cargo test -p kafkax --bench '*' -- --nocapture   # 或 cargo bench -p kafkax
```

`benches/hot_path.rs（3s 有界）` 必须有界，避免 `--all-targets` 挂死。
