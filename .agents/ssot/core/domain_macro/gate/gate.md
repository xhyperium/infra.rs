<!-- ssot:trace=domain_macro.gate.001 -->
# domain_macro 门禁

- DM-V01–DM-V07 每条必须有失败样本、稳定错误码、测试 ID 和 commit-matched evidence。
- `implemented`/`verified` 只能在 current code path 存在且测试真实运行后使用。
- 任何公共字段、派生反序列化绕过验证、`HashMap<struct, _>` 默认 JSON、NaN/∞、无时区时间、修订断链或 source identity 丢失均阻止晋级。
- 外部输入路径不得使用 panic/unwrap/expect/todo；错误不得包含秘密或无界原文。
- 未完成 N-1 fixture、回滚演练和发布渠道说明时不得发布。
