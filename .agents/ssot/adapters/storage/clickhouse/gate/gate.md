# adapters/storage/clickhouse — Gate

## 合并门禁（P0）

```bash
cargo fmt --all -- --check
cargo clippy -p clickhousex --all-targets -- -D warnings
cargo test -p clickhousex --all-targets
node scripts/clickhouse-https-conformance.mjs
cmp .agents/ssot/adapters/storage/clickhouse/spec/spec.md \
  .agents/ssot/adapters/storage/clickhouse/spec/xhyper-clickhousex-complete-spec.md
```

## Live 门禁（可选）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p clickhousex -- --ignored
```

## 阻断条件

- 默认路径退化为仅 scaffold
- 硬编码密钥
- live 测试去掉 `#[ignore]` 导致 CI 依赖外网/本机服务
- 无证据宣称 package stable
- 远程 HTTP 可降级、端口别名冲突未拒绝或错误中出现 SQL/payload/认证正文
- 未运行真实集群却把 TLS/auth/deadline/并发 live 标为 PASS
