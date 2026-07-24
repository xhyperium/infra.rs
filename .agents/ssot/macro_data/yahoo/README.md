# yahoo — 离线市场数据域规格 SSOT

<!-- ssot:domain=yahoo -->
<!-- ssot:provenance=status=unknown; source=UNKNOWN; as_of=UNKNOWN; fixture=UNKNOWN -->
<!-- ssot:spec_status=draft -->
<!-- ssot:implementation_status=not_started -->

当前只维护脱敏 quote、series、fx 和 search fixture 的离线解析边界。来源、字段、能力、协议、认证、访问许可、限流、缓存和再分发均为 `UNKNOWN`；没有 provider 实现，不进入 `domain_macro` L0。

## 当前范围

- 保留标的身份、时间、单位、市场标签、修订和缺失原因；
- 对合法、未知、缺失和坏输入返回稳定错误；
- 不定义客户端、访问路径、凭据流程、fallback 或真实服务测试。

实现前必须补齐官方/合同来源、授权、许可证、脱敏 fixture、离线正负样本、人工审查、回滚目标和 commit-matched evidence。
