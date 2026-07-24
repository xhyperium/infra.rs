# transport — Goal

> 状态：`0.1.4` **IMPLEMENTED CANDIDATE**。本地实现与 workspace 门禁已运行，
> 固定代码证据由 manifest 绑定；PR CI、独立终审、人工批准与 merge 均为 OPEN。

## 目标

在不扩大 L1 transport 边界的前提下，收敛 HTTP/WS 客户端传输的安全、资源与生命周期
合同，使 [`spec/spec.md`](../spec/spec.md) 的当前声明面可由实现和确定性测试复验。

## 三轮收敛

| 轮次 | 目标 | 候选结果 |
|---|---|---|
| R1 安全审计 | 阻断 Debug 凭据泄漏与未兑现 TLS 配置 | URL fail-closed 脱敏；`sni=false` 构造时拒绝 |
| R2 资源设计 | 在完整缓冲前限制 HTTP/WS payload | HTTP 逐 chunk 累计；WS decoder 前置 frame/message 上限 |
| R3 实现审计 | 闭合限流提示与池许可生命周期 | RFC 9110 `Retry-After`；pool RAII、poison 恢复与 factory unwind 回滚 |

## 验收边界

- HTTP 请求/响应资源上限、URL/代理 Debug 脱敏、`Retry-After` 两种语法均有测试。
- TLS 默认开启 SNI，当前不能兑现的 `sni=false` 必须 fail-closed。
- 客户端池在 Drop、`into_inner`、锁中毒以及 factory error/panic 后保持许可守恒。
- WS 入站大小限制下沉到 decoder，碎片聚合超限不得交付给调用方。
- 本地测试、Clippy、Rustdoc、格式和 workspace 依赖门禁通过。

## 非目标

企业 PKI/mTLS、证书轮换、M3、真实业务 live、重连/订阅恢复和 package stable 均为
**NO-GO**。本目标不以本地 PASS 替代 PR CI、独立 reviewer、人工审批或 merge。

追溯见 [`matrix/matrix.md`](../matrix/matrix.md)；当前落地裁定见
[`transport-ssot-alignment.md`](../../../../docs/ssot/transport-ssot-alignment.md)。
