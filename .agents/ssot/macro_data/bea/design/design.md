<!-- ssot:trace=bea.design.001 -->
# bea — 离线设计

当前 `draft`/`not_started`，来源合同、认证、配额、分页、许可和访问方式均为 `UNKNOWN`。

## 设计边界

- 只设计脱敏 fixture 的结构解析、来源身份保留、单位/频率校验和缺失语义；
- 不设计请求参数、API Key、HTTP 客户端、重试、限流、缓存或真实服务测试；
- 任何映射、派生指标和失败样本都必须绑定 manifest 条款、测试 ID 与离线证据；
- secret sentinel 不得进入 SSOT、fixture、Debug、Display、Serialize、错误、tracing、URL 或原始响应。

## 晋级条件

官方或合同来源、版本、访问日期、脱敏 fixture、失败样本、授权、合规审查、回滚目标和 commit-matched evidence 齐全后，才能另提独立 provider 设计与 Cargo 路径。
