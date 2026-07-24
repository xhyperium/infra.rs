<!-- ssot:trace=fred.design.001 -->
# fred — 设计边界

本域为 `draft`/`not_started`，仅设计脱敏 fixture 的离线解析。外部端点、许可、配额、修订语义和权限均为 `UNKNOWN`，不得从旧文档或记忆推导运行时能力。

## 组件

| 组件 | 当前责任 |
|---|---|
| `FixtureParser` | 严格 UTF-8、字段校验、缺失值、单位和日期解析 |
| `RevisionMapper` | 保留 source/series/date/vintage 身份，追加修订 |
| `SecretRef` | 只表达运行时注入引用，不实现存储、序列化或日志输出 |
| `AccessPolicy` | 权限未知时返回 `access_denied`，不发出真实请求 |

不设计网络客户端、缓存、重试、真实 API 测试或具体 crate 路径。只有来源、合同和授权经人工审查后，才能在 manifest 原子切换为 `approved/authorized_standard_client` 并提交新设计。

## 不变量

- sentinel secret 不得出现在 Debug、Display、JSON、错误、tracing、URL 或原始响应中。
- 数值拒绝 NaN/∞；缺失值不转零；重复身份明确拒绝或幂等。
- 拒绝、挑战、配额或权限未知均是终止状态，不改变访问方式。
