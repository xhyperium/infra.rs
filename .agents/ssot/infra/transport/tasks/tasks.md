# transport — Tasks

> 状态：`0.1.4` 实现切片与本地验收已完成；本页用于规格追溯，持久任务状态以 Beads 为准。

## 已完成切片

| 切片 | 可复验产出 | 状态 |
|---|---|---|
| T1 安全 | 请求/代理 URL 脱敏；敏感 header/body 隐藏；`sni=false` 拒绝 | 已完成 |
| T2 HTTP 资源 | 请求体预检、`Content-Length` 预检、chunk 累计首次越界中止 | 已完成 |
| T3 429 语义 | delay-seconds 与 HTTP-date 解析，过去日期钳零 | 已完成 |
| T4 pool 生命周期 | 配置校验、RAII lease、`into_inner`、poison 恢复、factory error/panic 回滚 | 已完成 |
| T5 WS 资源 | 出站预检、入站 decoder frame/message 上限、碎片累计拒绝 | 已完成 |
| T6 本地验收 | crate test/clippy/doc、workspace fmt 与依赖门禁 | 已完成 |
| T7 依赖审计 | `httpdate 1.0.3` 用途/替代/许可/上游与 `cargo deny` 评估 | 已完成 |

## 外部交付里程碑

| 里程碑 | 完成条件 | 当前状态 |
|---|---|---|
| PR CI | PR 上的必需检查通过 | OPEN |
| 独立终审 | reviewer 对固定候选 diff 给出结论 | OPEN |
| 人工批准 | maintainer 明确批准 | OPEN |
| Merge | 候选合入 `main` | OPEN |

这些里程碑不在本文预先判定为 PASS。企业 PKI/mTLS、M3 与真实业务 live 没有实现或证据
任务，继续 **NO-GO**，不得由本轮切片外推。

测试映射见 [`test/test.md`](../test/test.md)，完整追溯见
[`matrix/matrix.md`](../matrix/matrix.md)。
