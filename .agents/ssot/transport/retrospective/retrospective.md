# transport — Retrospective

> 状态：IMPLEMENTED CANDIDATE 阶段复盘；不是发布完成声明。

## 三轮所得

| 轮次 | 关键发现 | 可复用做法 |
|---|---|---|
| R1 安全 | Debug 派生不是天然安全边界；未接线配置会制造虚假能力 | 集中脱敏并 fail-closed；不能兑现的开关在构造时拒绝 |
| R2 资源 | “读取后检查”不能防止内存资源耗尽 | 在 `Content-Length`、chunk 累计与 WS decoder 三层前置检查 |
| R3 生命周期 | 手动归还不足以覆盖 error、panic 与 poison | 用 RAII lease/回滚守卫表达许可所有权，并以故障注入验证 |

`Retry-After` 的经验是：协议字段应覆盖标准允许的两种语法，并提供显式 `now` 入口，
避免时间相关测试依赖墙钟。

## 保留风险

- PR CI、独立终审、人工批准与 merge 均为 OPEN；候选仍可能因外部门禁反馈返工。
- 上限设为零会关闭对应保护，调用方必须显式承担风险。
- 企业 PKI/mTLS、证书轮换、M3、真实业务 live、重连与订阅恢复没有闭合证据，继续
  **NO-GO**。

发布后复盘只能在外部门禁与 merge 完成后补充；当前记录不替代 reviewer 或 maintainer
裁决。
