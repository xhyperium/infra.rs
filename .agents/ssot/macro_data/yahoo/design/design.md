<!-- ssot:trace=yahoo.design.001 -->
# yahoo — 设计边界

本域为 `draft`/`not_started`，只允许离线 fixture 解析设计。Yahoo 的接口、许可、访问权限、字段和服务水平均为 `UNKNOWN`，没有批准的 provider 根路径。

## 组件

| 组件 | 责任 | 状态 |
|---|---|---|
| `FixtureParser` | 解析脱敏 quote/chart/fx/search fixture | 草案 |
| `SchemaGuard` | 识别未知字段并返回可诊断错误 | 草案 |
| `DomainMapper` | 保留 source identity，映射到统一模型 | 草案 |
| `AccessPolicy` | 表达批准后的访问合同，不保存秘密 | 草案 |

未核验权限前不设计自动会话、凭据获取、后台访问或网络端到端测试。拒绝、挑战、配额和未知合同状态统一返回 `access_denied`；不采用替代访问方式。

## 不变量

- 原始响应、完整 URL、Cookie、token 和高基数 symbol 不进入日志、错误、Debug 或序列化。
- 缺失值、时间区间、重复身份和单位转换必须在离线 fixture 中可重放。
- 只有 manifest 中的获批路径才可成为实现证据。
