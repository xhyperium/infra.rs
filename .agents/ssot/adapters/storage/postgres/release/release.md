# adapters/storage/postgres — Release

| 项 | 状态 |
|----|------|
| workspace member | 是 · `postgresx` |
| publish | `publish = false` |
| crates.io | **未发布** |
| SemVer package stable | **未宣称** |
| 内部可用 | P0 生产入口可用（#188+；`0.3.6` foundation DoD 闭合） |
| 本轮交付 | 内部 foundation 闭合 + live/bench/deadline 证据；**非** crates.io package stable |

## 当前质量门禁 (2026-07-24)

| 检查 | 结果 |
|------|------|
| `cargo build -p postgresx` | PASS |
| `cargo test -p postgresx --lib` | 66 passed, 0 failed |
| `cargo clippy -p postgresx --lib -- -D warnings` | PASS (0 warnings) |
| `cargo fmt -p postgresx -- --check` | PASS |
| SSOT 对齐 (matrix.md) | 22/24 PASS |
| Beads | 无 open issues |
| live 测试 (ignored) | 存在，需人工凭据 |

## 发布前清单（若未来 stable）

- [ ] 公共 API 冻结说明 + CHANGELOG
- [ ] live 在 CI 可选 job 稳定
- [ ] 远程 TLS live 握手证据
- [ ] DEFER 项明确 out-of-scope 或落地
- [ ] `publish = true` 与 Lead 批准

## crates.io 发布阻塞项

| # | 阻塞 | 说明 |
|---|------|------|
| 1 | `publish = false` | Cargo.toml 显式关闭 |
| 2 | package stable 未宣称 | SSOT 全部标注 NOT CLAIMED |
| 3 | 依赖链未发布 | 依赖 `xhyper-kernel` / `xhyper-contracts` / `xhyper-resiliencx` (已发布) |
| 4 | 名称冲突 | `postgresx` 需重命名为 `xhyper-postgresx`（与 kernel/contracts 同模式） |

## crates.io 发布路径

```bash
# 1. 重命名
sed -i 's/name = "postgresx"/name = "xhyper-postgresx"/' crates/adapters/storage/postgres/Cargo.toml
sed -i 's/publish = false/publish = true/' crates/adapters/storage/postgres/Cargo.toml

# 2. 更新 workspace 依赖（所有引用 postgresx 的 crate）
fd -t f Cargo.toml crates/ tools/ | xargs grep -l 'postgresx.*=.*{.*path' | \
  xargs sed -i 's|^postgresx = { path|postgresx = { package = "xhyper-postgresx", path|'

# 3. 验证
cargo test -p xhyper-postgresx --lib

# 4. 发布
CARGO_REGISTRY_TOKEN=... cargo publish -p xhyper-postgresx --allow-dirty
```

## DEFER 项明细 (out of scope)

| 项 | 说明 | 影响 |
|----|------|------|
| 不受限流式 COPY | 当前 COPY 有界 (16 MiB) | 大文件导入需外部切分 |
| read-replica 路由 | 未实现读写分离 | 仅单节点 |
| down migration | 仅有 up migration | 需手动回滚 |
| channel binding | SCRAM channel binding | 仅 TLS verify-full |
| 服务端 mTLS live | 远端强制 mTLS 未 live 验证 | 客户端 mTLS 已实现 |
| package stable | 未宣称 API 稳定 | crates.io 不可发布 |
