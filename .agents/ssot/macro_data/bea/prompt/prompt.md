<!-- ssot:trace=bea.prompt.001 -->
# bea — 离线 Agent 提示词

在 `not_started`/`offline_fixture_only` 状态，只处理脱敏 fixture 的解析、来源身份、单位、期间、缺失语义和稳定错误。执行前读取 manifest、spec、gate 和 evidence；未知外部事实保持 `UNKNOWN`。

不得猜测 endpoint、参数、认证、API Key、配额、HTTP 客户端、重试、限流、缓存或再分发许可；不得运行真实服务测试。所有结果必须记录条款 ID、修改文件、命令、退出码和未决风险。
