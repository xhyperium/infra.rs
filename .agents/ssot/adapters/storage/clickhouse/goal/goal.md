# adapters/storage/clickhouse — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `clickhousex` |
| 标题 | ClickHouse Analytics |
| 实现 | `crates/adapters/storage/clickhouse` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 |
| 状态 | **P0 生产入口已落地**（#188–#191）；package stable **未宣称** |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 ClickHouse Analytics 生产默认客户端，满足 draft P0 DoD，且默认 `cargo test` 离线绿灯。

## Acceptance（本仓可验证）

1. workspace member `clickhousex` 可 `cargo test -p clickhousex --all-targets`
2. 生产默认面：`ClickHousePool / ClickHouseClient HTTP`
3. 环境注入：`FOUNDATIONX_CLICKHOUSEX_{HOST,HTTP_PORT/PORT,USER,PASSWORD,DATABASE}`（密钥不入库）
4. live：`tests/live_smoke.rs` 默认 `#[ignore]`，真凭据可绿
5. bench：`benches/hot_path.rs（3s 有界）`（不得挂死 `--all-targets`）
6. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认

## Not in scope

native 9000 protocol / cluster / ReplicatedMergeTree 运维面

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/clickhousex-ssot-alignment.md](../../../../../docs/ssot/clickhousex-ssot-alignment.md)
