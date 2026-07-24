# transport — Test

> 状态：`0.1.4` 声明面的本地测试与静态门禁已运行，固定代码证据由
> [`manifest.json`](../../../../evidence/testkit/2026-07-23-infra-2d9.10/manifest.json) 绑定；
> PR CI、独立终审、人工批准与 merge 均为 OPEN。

## 合同覆盖

| 合同 | 主要测试证据 |
|---|---|
| URL/header/body Debug 脱敏与 SNI fail-closed | [`limits_and_debug.rs`](../../../../crates/infra/transport/tests/limits_and_debug.rs) |
| HTTP body 上限与 chunk 累计首次越界 | [`limits_and_debug.rs`](../../../../crates/infra/transport/tests/limits_and_debug.rs) |
| RFC 9110 `Retry-After` 与真实 429 路径 | [`reqwest_driver.rs`](../../../../crates/infra/transport/tests/reqwest_driver.rs) |
| pool 配置、RAII Drop 与 `into_inner` | [`pool_contracts.rs`](../../../../crates/infra/transport/tests/pool_contracts.rs) |
| pool poison、factory error/panic 许可恢复 | [`pool.rs`](../../../../crates/infra/transport/src/pool.rs) 内部故障注入测试 |
| WS decoder 单帧/碎片累计上限与生命周期 | [`websocket.rs`](../../../../crates/infra/transport/tests/websocket.rs) |
| Mock 与公开消费面 | [`mock_http.rs`](../../../../crates/infra/transport/tests/mock_http.rs)、[`public_api_surface.rs`](../../../../crates/infra/transport/tests/public_api_surface.rs) |

所有网络路径默认使用本地 loopback 或受控故障注入，不把公网、企业 PKI 或业务 live
作为本地测试替代品。

## 复验命令

```bash
cargo test -p transportx --all-targets
cargo clippy -p transportx --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p transportx --no-deps
cargo fmt --all --check
node scripts/quality-gates/check-workspace-deps.mjs
```

上述本地门禁已完成。其结果仅支持 IMPLEMENTED CANDIDATE，不支持 released、M3、
package stable、企业 PKI/mTLS 或真实业务 live 声明。
