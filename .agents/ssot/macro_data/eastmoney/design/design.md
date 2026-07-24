<!-- ssot:trace=eastmoney.design.001 -->
# eastmoney — 设计边界

本域仍为 `draft`/`not_started`。在获得书面授权、稳定合同和获批 crate 根目录前，只设计离线 fixture 解析器；`macrox` L0 不包含网络 I/O。

## 决策

1. 输入是版本化、脱敏的 JSON/CSV fixture；解析结果转换为 `domain_macro` 的统一类型。
2. 认证材料、完整请求 URL、原始响应和高基数标识不进入日志、错误、Debug 或序列化结果。
3. 标准客户端只允许在 `authorization_status=approved` 后实现；收到拒绝、挑战或配额响应时返回稳定的 `access_denied`，不改变访问方式。
4. 端点、字段、许可、配额和再分发规则当前均为 `UNKNOWN`；核验完成后必须逐项绑定官方或合同来源并通过人工合规审查。

## 组件边界

| 组件 | 当前状态 | 责任 |
|---|---|---|
| `FixtureParser` | 规格草案 | 严格 UTF-8、字段校验、单位和缺失值映射 |
| `DomainMapper` | 规格草案 | 映射到 `domain_macro`，保留 source identity |
| `AccessPolicy` | 规格草案 | 只表达已批准的访问权限，不保存秘密 |
| `ProviderClient` | 未开始 | 需先有批准路径和授权证据 |

## 取舍

离线 fixture 降低了规格早期的可用性，但能使测试可重复、避免把未核验事实写成生产承诺；在合同确认前不增加运行时传输层。
