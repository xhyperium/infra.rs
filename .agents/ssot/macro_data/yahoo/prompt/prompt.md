<!-- ssot:trace=yahoo.prompt.001 -->
# yahoo — Agent 提示词

在授权与 provider 路径获批前，只处理脱敏 fixture 的离线解析和 `domain_macro` 映射。

要求：

1. 将外部事实标为 `UNKNOWN`，不得凭记忆补全端点、配额或许可。
2. 设计稳定错误、缺失值和时间区间；拒绝、挑战和未知权限响应必须停止，配额语义保持 `UNKNOWN`。
3. secret、Cookie、完整 URL、原始响应和高基数标识不得出现在日志、Debug、错误或序列化。
4. 执行 Rust 质量门禁、fixture 测试和 `check-ssot.mjs`。
