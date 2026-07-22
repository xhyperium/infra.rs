# adapters/storage/redis — Prompt 约束

实现 / 修改 `redisx` 时：

1. 只改 scope 内 `crates/adapters/storage/redis` 与对应对齐/SSOT 文档
2. 默认路径必须可生产使用；scaffold 不得成为默认
3. 密钥仅 env；禁止提交 secrets
4. 新增 live 测试必须 `#[ignore]`，默认 CI 离线绿
5. 完成前：`cargo test -p redisx --all-targets` + fmt/clippy
6. 宣称完成必须附当前会话测试输出
