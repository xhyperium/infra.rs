# transport — Release

> 状态：`0.1.4` 内部候选，**尚未发布或合并**。

## 就绪度

| 条件 | 状态 | 说明 |
|---|---|---|
| 规格与实现声明面一致 | 本地已运行 | 固定代码与结果由 manifest 绑定 |
| crate 测试、Clippy、Rustdoc | 本地已运行 | 命令见 [`gate/gate.md`](../gate/gate.md) |
| workspace 格式与依赖门禁 | 本地已运行 | 不替代 PR 环境 |
| PR CI | OPEN | 以远端必需检查为准 |
| 独立终审 | OPEN | 不在本文件预先批准 |
| 人工批准 | OPEN | 需 maintainer 明确批准 |
| Merge | OPEN | 合入 `main` 后方可更新发布事实 |

## 候选摘要

0.1.4 在 0.1.3 上继续收紧 URL Debug、兼容池构造和异常关闭语义，并补齐隐藏 public
hook、factory unwind、锁中毒与超时证据；资源上限、RFC 9110 `Retry-After`、SNI fail-closed、
pool RAII/poison/factory unwind 许可恢复，以及 WS decoder 前置限制。用户可见摘要见
[`crates/transport/releases/0.1.4.md`](../../../../crates/transport/releases/0.1.4.md)。

## 发布红线

- 不得把本地 PASS 写成 PR CI、独立审查或人工审批 PASS。
- 不得宣称企业 PKI/mTLS、M3、真实业务 live 或 package stable；这些能力仍 **NO-GO**。
- 未 merge 前不得把本候选描述为主干已发布版本。

最终 release 裁决必须由外部交付门禁产生，本文件不作预先批准。
