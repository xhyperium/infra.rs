# adapters/storage/redis — Gate

## 合并门禁（P0）

```bash
cargo fmt --all -- --check
cargo clippy -p redisx --all-targets --features pubsub -- -D warnings
cargo test -p redisx --all-targets --features pubsub
node scripts/quality-gates/check-workspace-deps.mjs
```

## Live 门禁（可选）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p redisx -- --ignored
```

## 阻断条件

- 默认路径退化为仅 scaffold
- 硬编码密钥
- live 测试去掉 `#[ignore]` 导致 CI 依赖外网/本机服务
- 无证据宣称 package stable
- Pub/Sub 从环境变量重建配置或把 Cluster/Sentinel 静默降级为 Standalone
- 让相对 TTL SET、DEL、PEXPIRE 在多次尝试配置下进入 I/O，或让 PUBLISH 自动重试
- 阻止无 TTL SET/MSET 按已声明 `Idempotent` 合同使用预算重试
- 把粗粒度 `RedisOperation::Set` 误写成可表达 TTL 参数，或把命令原子误写为“超时即未执行”
- 无真实拓扑证据把 Cluster / Sentinel / TLS 标为 live PASS

当前候选曾冻结；治理修正后最终 SHA 待重冻，reviewer/verifier 与最终 SHA CI 均 pending。
