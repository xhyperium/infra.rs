<!-- ssot:trace=eastmoney.prompt.001 -->
# eastmoney — Agent 提示词

在 provider crate 获批前，只实现脱敏 fixture 的离线解析、字段校验和 `domain_macro` 映射设计。

要求：

1. 先阅读本域 `spec/design/gate` 与 manifest，不猜测外部端点或授权。
2. 认证材料、完整 URL、原始响应和用户输入不得进入错误、Debug、序列化或 tracing。
3. 拒绝、挑战、配额不足和未知字段返回稳定、可测试的错误；不改变访问方式。
4. 使用中文用户可见文本，执行 `cargo fmt`、`clippy -D warnings`、离线 fixture 测试和 SSOT 门禁。
