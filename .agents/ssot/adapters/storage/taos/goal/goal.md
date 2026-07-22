# adapters/storage/taos — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `taosx` |
| 标题 | TDengine TimeSeries |
| 实现 | `crates/adapters/storage/taos` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 |
| 状态 | **REST SQL 受限生产入口已落地**；Native SQL / HA / package stable **NO-GO** |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 TDengine TimeSeries 生产默认客户端，满足 draft P0 DoD，且默认 `cargo test` 离线绿灯。

## Acceptance（本仓可验证）

1. workspace member `taosx` 可 `cargo test -p taosx --all-targets`
2. 生产默认面：`TaosPool / TaosClient REST`
3. 远程 TLS/auth fail-closed；Decimal 以 NCHAR 文本往返且存量 DOUBLE schema 拒绝
4. 响应、SQL batch、query rows、in-flight 与 close drain 均有编译期硬上限
5. live：`scripts/taos-live-conformance.mjs` 可启动隔离固定 digest 服务；测试本体默认 `#[ignore]`
6. bench：`benches/hot_path.rs（3s 有界）`（不得挂死 `--all-targets`）
7. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认

## Not in scope

Native SQL / FFI / WS 认证长会话 / 幂等自动重试 / 全超表治理 / HA 集群

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/taosx-ssot-alignment.md](../../../../../docs/ssot/taosx-ssot-alignment.md)
