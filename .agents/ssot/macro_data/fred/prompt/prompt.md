<!-- ssot:trace=fred.prompt.001 -->
# fred — Agent 提示词

在权限和 provider 路径获批前，只实现脱敏 fixture 的离线解析与统一映射设计。

要求：

1. 先核对 `spec.md`、manifest 和证据；未知外部事实保持 `UNKNOWN`。
2. 使用 secret 引用对象表达认证，不在代码、fixture、Debug、Display、Serialize/JSON、URL、原始响应、错误或 tracing 中保存秘密。
3. 覆盖 series identity、观测日、vintage、缺失值、单位、坏输入和重复输入。
4. 运行 workspace Rust 门禁、离线测试和 `check-ssot-current-state.mjs`。
