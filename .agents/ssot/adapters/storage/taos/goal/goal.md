# adapters/storage/taos — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `taosx` |
| 标题 | TDengine TimeSeries |
| 实现 | `crates/adapters/storage/taos` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 |
| 状态 | **P0 生产入口已落地**（#188–#191）；package stable **未宣称** |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 TDengine TimeSeries 生产默认客户端，满足 draft P0 DoD，且默认 `cargo test` 离线绿灯。

## Acceptance（本仓可验证）

1. workspace member `taosx` 可 `cargo test -p taosx --all-targets`
2. 生产默认面：`TaosPool / TaosClient REST`
3. 环境注入：`FOUNDATIONX_TAOSX_{HOST,PORT,USER,PASSWORD,DATABASE,TLS,PRECISION}`（密钥不入库）
4. live：`tests/live_smoke.rs` 默认 `#[ignore]`，真凭据可绿
5. bench：`benches/hot_path.rs（3s 有界）`（不得挂死 `--all-targets`）
6. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认

## Not in scope

native WS / 全超表治理 / 集群

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/taosx-ssot-alignment.md](../../../../../docs/ssot/taosx-ssot-alignment.md)
