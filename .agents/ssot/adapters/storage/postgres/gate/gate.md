# adapters/storage/postgres — Gate

## 合并门禁（P0）

```bash
cargo fmt --all -- --check
cargo clippy -p postgresx --all-targets -- -D warnings
cargo test -p postgresx --all-targets
```

## Live 门禁（可选）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p postgresx -- --ignored
```

## 阻断条件

- 默认路径退化为仅 scaffold
- 硬编码密钥
- live 测试去掉 `#[ignore]` 导致 CI 依赖外网/本机服务
- 无证据宣称 package stable
