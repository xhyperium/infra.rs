# configx — Agent 规则

> 父级：[`../AGENTS.md`](../AGENTS.md)

## 职责

- L1 进程内字符串 KV、多源合并、手动 reload 与进程内通知（active SSOT 0.1.2）
- 线程安全：`RwLock<HashMap<String, String>>`
- 错误经 `kernel::{XError, XResult}`

## 硬边界

1. 生产依赖 **仅** `xhyper-kernel`；禁止其他 L1（含 `observex`）
2. 无 feature（`default = []`）
3. 公开面须保留兼容折叠 API 与 Result / 显式 outcome API 的双路径
4. reload 只允许描述为调用方手动触发的进程内操作；不得写成自动 watcher 或远端配置中心
5. 禁止在 `.agents/ssot/**` 镜像内改 COMPLETE 叙事冒充落地

## 目录

```text
crates/configx/
├── Cargo.toml
├── src/lib.rs          # ConfigStore + 单元/毒锁测试
├── tests/              # 公开面 + 并发集成测试
├── examples/basic.rs   # 消费者路径
├── docs/
├── README.md
├── AGENTS.md
└── CHANGELOG.md
```

## 验证

```bash
cargo test -p configx --all-targets
cargo clippy -p configx --all-targets -- -D warnings
cargo llvm-cov -p configx --summary-only
```
