# adapters/storage/clickhouse — Prompt 约束

实现 / 修改 `clickhousex` 时：

1. 只改 scope 内 `crates/adapters/storage/clickhouse` 与对应对齐/SSOT 文档
2. 默认路径必须可生产使用；scaffold 不得成为默认
3. 密钥仅 env；禁止提交 secrets
4. 新增 live 测试必须 `#[ignore]`，默认 CI 离线绿
5. 远程 HTTP、端口别名冲突与无效 CA 必须 fail-closed
6. 错误与日志不得包含 SQL、payload、认证正文或完整 URL
7. 完成前：`cargo test -p clickhousex --all-targets` + fmt/clippy + HTTPS conformance + 双镜像 cmp
8. 宣称完成必须附当前会话测试输出；未运行真实集群不得把 live 标为 PASS
