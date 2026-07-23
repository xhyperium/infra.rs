# transport — Matrix

> 状态：`0.1.4` IMPLEMENTED CANDIDATE 追溯矩阵；本地 PASS，外部交付门禁 OPEN。

权威合同：[`spec/spec.md`](../spec/spec.md)。

| 合同 | 实现入口 | 测试证据 | 本地状态 |
|---|---|---|---|
| HTTP 请求/响应上限与 chunk 累计 | [`lib.rs`](../../../../crates/transport/src/lib.rs) | [`limits_and_debug.rs`](../../../../crates/transport/tests/limits_and_debug.rs) | PASS |
| URL/header/body 与代理 Debug 脱敏 | [`lib.rs`](../../../../crates/transport/src/lib.rs)、[`proxy.rs`](../../../../crates/transport/src/proxy.rs) | [`limits_and_debug.rs`](../../../../crates/transport/tests/limits_and_debug.rs) | PASS |
| RFC 9110 `Retry-After` | [`lib.rs`](../../../../crates/transport/src/lib.rs) | [`reqwest_driver.rs`](../../../../crates/transport/tests/reqwest_driver.rs) | PASS |
| TLS 默认 SNI 与 `sni=false` 拒绝 | [`tls.rs`](../../../../crates/transport/src/tls.rs)、[`lib.rs`](../../../../crates/transport/src/lib.rs) | [`limits_and_debug.rs`](../../../../crates/transport/tests/limits_and_debug.rs) | PASS |
| pool 校验、RAII、poison 与 factory unwind | [`pool.rs`](../../../../crates/transport/src/pool.rs) | [`pool_contracts.rs`](../../../../crates/transport/tests/pool_contracts.rs) 与内部故障注入测试 | PASS |
| WS decoder 单帧/碎片聚合上限 | [`lib.rs`](../../../../crates/transport/src/lib.rs) | [`websocket.rs`](../../../../crates/transport/tests/websocket.rs) | PASS |
| 既有 HTTP/WS/Mock 生命周期 | [`lib.rs`](../../../../crates/transport/src/lib.rs) | [`reqwest_driver.rs`](../../../../crates/transport/tests/reqwest_driver.rs)、[`websocket.rs`](../../../../crates/transport/tests/websocket.rs)、[`mock_http.rs`](../../../../crates/transport/tests/mock_http.rs) | PASS |
| `httpdate` 依赖评估 | [`design`](../design/design.md#r-dep-001httpdate-评估) | `cargo tree -p transportx -i httpdate`、`cargo deny check` | PASS；锁定 `1.0.3` |
| 企业 PKI/mTLS/M3/live | 无闭合实现或证据 | 无 | NO-GO |

## 交付矩阵

| 层级 | 状态 | 结论 |
|---|---|---|
| 本地实现与验证 | PASS | 由 manifest 绑定；支持 IMPLEMENTED CANDIDATE |
| PR CI | OPEN | 不得预判 |
| 独立终审 | OPEN | 不得预先批准 |
| 人工批准与 merge | OPEN | 尚未 released |

门禁命令见 [`gate/gate.md`](../gate/gate.md)，reviewer 输入见
[`review/review.md`](../review/review.md)。
