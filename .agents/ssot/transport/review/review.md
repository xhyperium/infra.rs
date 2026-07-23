# transport — Review

> 状态：独立终审为 **OPEN**。本文是 reviewer 输入，不是批准结论。

## 审查基线

- 合同：[`spec/spec.md`](../spec/spec.md)
- 候选：`transportx 0.1.4`
- 落地裁定：[`transport-ssot-alignment.md`](../../../../docs/ssot/transport-ssot-alignment.md)
- 追溯：[`matrix/matrix.md`](../matrix/matrix.md)

## 必审问题

1. URL 解析失败和不透明 URL 是否始终 fail-closed，且代理 Debug 是否复用同一规则。
2. HTTP chunk 与 WS decoder 是否在 payload 完整交付前拒绝超限数据。
3. `Retry-After` HTTP-date 是否使用确定性时钟入口，并正确处理过去日期与非法值。
4. `sni=false` 是否在构造阶段明确失败，而不是静默忽略。
5. pool 在 lease Drop、`into_inner`、锁中毒、factory error 和 panic unwind 后是否精确恢复许可。
6. 公开 API 与错误映射是否仍符合 L1 边界，未吸收重试、业务认证或调度职责。

## 已知边界

本地 test/clippy/doc/fmt/dependency gate 已运行，固定代码证据由 manifest 绑定。
PR CI、独立终审、人工批准与 merge 均为 OPEN。企业 PKI/mTLS、M3、
真实业务 live 与 package stable 为 **NO-GO**。

reviewer 应基于固定候选 diff 独立给出裁决；本文件不填写 PASS，也不代表 maintainer 批准。
