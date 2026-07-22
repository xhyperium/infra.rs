# adapters/storage/oss — Gate

## 合并门禁（P0）

```bash
cargo fmt --all -- --check
cargo clippy -p ossx --all-targets -- -D warnings
cargo test -p ossx --all-targets
```

## Live 门禁（可选）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p ossx -- --ignored
```

## 阻断条件

- 默认路径退化为仅 scaffold
- 硬编码密钥
- 远程 endpoint 允许明文 HTTP
- 对象/缓冲/错误体/in-flight/retry/multipart 任一可配置为无界
- multipart abort 失败被静默吞掉，或 ETag 未经 XML escaping
- live 测试去掉 `#[ignore]` 导致 CI 依赖外网/本机服务
- 无证据宣称 package stable
