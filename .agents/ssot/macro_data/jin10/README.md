# jin10 — 离线消息数据域规格 SSOT

<!-- ssot:domain=jin10 -->
<!-- ssot:provenance=status=unknown; source=UNKNOWN; as_of=UNKNOWN; fixture=UNKNOWN -->
<!-- ssot:spec_status=draft -->
<!-- ssot:implementation_status=not_started -->

当前只维护脱敏事件、日历和行情消息 fixture 的离线解析边界。来源、能力、协议、认证、字段、实时性、限流、缓存和再分发均为 `UNKNOWN`；没有 provider 实现，不进入 `domain_macro` L0。

## 目录结构

```text
jin10/
├── README.md
├── goal/ spec/ design/ plan/ tasks/ prompt/ test/
├── review/ release/ retrospective/ matrix/ gate/ evidence/
```

## 晋级条件

实现前必须补齐来源合同、字段字典、访问授权、许可证、脱敏 fixture、失败样本、人工审查、回滚目标和 commit-matched evidence。未获批前不得把任何产品叙述写成实现目标、客户端设计或测试合同。
