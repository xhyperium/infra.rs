# adapters/storage/redis — Prompt 约束

实现 / 修改 `redisx` 时：

1. 只改 scope 内 `crates/adapters/storage/redis` 与对应对齐/SSOT 文档
2. 默认路径必须可生产使用；scaffold 不得成为默认
3. 密钥仅 env；禁止提交 secrets
4. 新增 live 测试必须 `#[ignore]`，默认 CI 离线绿
5. Pub/Sub 必须复用显式/建池配置；不支持的拓扑必须失败关闭
6. 自动重试只允许已知安全的只读操作；写入原子性与响应歧义必须有代码合同和失败测试
7. 完成前：`cargo test -p redisx --all-targets --features pubsub` + fmt/clippy + workspace deps gate
8. 宣称完成必须附当前会话测试输出；未运行真实 Cluster/Sentinel/TLS 必须保持 OPEN
