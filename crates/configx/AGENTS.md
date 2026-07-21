# configx — Agent 规则

> 父级：[`../AGENTS.md`](../AGENTS.md)

## 职责

- L1 内存字符串 KV 存储（active SSOT 0.1.0）
- 线程安全：`RwLock<HashMap<String, String>>`
- 错误经 `kernel::{XError, XResult}`

## 硬边界

1. 生产依赖 **仅** `xhyper-kernel`；禁止其他 L1（含 `observex`）
2. 无 feature（`default = []`）
3. 公开面仅 `ConfigStore` + `new` / `get` / `set` / `Default`；扩展前须真实消费者 + 评审
4. 不把本 crate 文档写成「多源热更新系统」
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
cargo test -p xhyper-configx --all-targets
cargo clippy -p xhyper-configx --all-targets -- -D warnings
cargo llvm-cov -p xhyper-configx --summary-only
```
