# adapters/storage/oss — Prompt 约束

实现 / 修改 `ossx` 时：

1. 只改 scope 内 `crates/adapters/storage/oss` 与对应对齐/SSOT 文档
2. 默认路径必须可生产使用；scaffold 不得成为默认
3. 密钥仅 env；禁止提交 secrets
4. 新增 live 测试必须 `#[ignore]`，默认 CI 离线绿
5. 完成前：`cargo test -p ossx --all-targets` + fmt/clippy
6. 宣称完成必须附当前会话测试输出
7. 远程 HTTP 必须 fail-closed；loopback 开发端点可显式使用 HTTP
8. 对象、缓冲、错误体、并发、multipart part/count、重试次数与总 deadline 必须有硬上界
9. Initiate/Complete 不因响应不确定自动重放；abort 失败必须暴露 `orphan_risk=true`
10. STS、lifecycle、live 与 package stable 无证据保持 OPEN
11. 高层 multipart 必须共享单一剩余 deadline；drop future 后 key/UploadId 必须可发现并可补偿
