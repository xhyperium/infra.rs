# adapters/storage/taos — Test 合同

## 离线（默认 CI）

```bash
cargo test -p taosx --all-targets
cargo clippy -p taosx --all-targets -- -D warnings
cmp .agents/ssot/adapters/storage/taos/spec/spec.md \
  .agents/ssot/adapters/storage/taos/spec/xhyper-taosx-complete-spec.md
```

覆盖期望：远程明文/空认证拒绝、strict host、Decimal NCHAR schema、响应与 batch bytes、
query rows、in-flight RAII、close deadline/重复 close、WS deadline、公共 API 引用。

## Live（可选 · 真凭据）

```bash
node scripts/taos-live-conformance.mjs
```

- live 文件：`tests/live_smoke.rs`
- 固定镜像 digest、动态 loopback 6041、全局 timeout 与 finally cleanup；不使用 prod
- 覆盖 scale=18、大 mantissa、正负 Decimal 的 REST 写查完全相等
- Native SQL / WS auth / HA 未覆盖，保持 NO-GO

## Bench

```bash
cargo test -p taosx --bench '*' -- --nocapture   # 或 cargo bench -p taosx
```

`benches/hot_path.rs（3s 有界）` 必须有界，避免 `--all-targets` 挂死。
