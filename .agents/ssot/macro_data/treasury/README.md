# treasury — 美国财政数据域 SSOT

<!-- ssot:domain=treasury -->
<!-- ssot:provenance=status=unknown; source=UNKNOWN; as_of=UNKNOWN; fixture=UNKNOWN -->
<!-- ssot:spec_status=draft -->
<!-- ssot:implementation_status=not_started -->

当前只定义脱敏财政记录 fixture 的离线解析边界。来源、端点、认证、参数、许可、限流、缓存和修订语义均为 `UNKNOWN`，没有 Treasury provider 实现，不能把任何来源叙述当作已核验合同。

## 当前材料

- `spec/`：记录字段身份、单位、期间、修订和缺失语义的占位；
- `test/`：只允许本地 fixture 和纯数据桩；
- `matrix/`、`gate/`、`evidence/`：绑定条款、门禁和晋级证据；
- `plan/`、`tasks/`、`prompt/`：不得指导创建网络客户端或凭据流程。

实现前必须补齐官方/合同来源、版本、访问日期、认证、字段、状态码、分页、限流、缓存和再分发证据，并经人工审查与回滚演练。
