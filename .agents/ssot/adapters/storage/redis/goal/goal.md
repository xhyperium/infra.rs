# adapters/storage/redis — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `redisx` |
| 标题 | Redis KV |
| 实现 | `crates/adapters/storage/redis` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 |
| 状态 | **P0 生产入口已落地**（#188–#191）；package stable **未宣称** |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 Redis KV 生产默认客户端，满足 draft P0 DoD，且默认 `cargo test` 离线绿灯。

## Acceptance（本仓可验证）

1. workspace member `redisx` 可 `cargo test -p redisx --all-targets`
2. 生产默认面：`RedisPool / RedisClient / RedisConfig`
3. 环境注入：`FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS}`（密钥不入库）
4. live：`tests/live_kv.rs · tests/live_kv_conformance.rs` 默认 `#[ignore]`，真凭据可绿
5. bench：`benches/kv_hot_path.rs`（不得挂死 `--all-targets`）
6. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认

## Not in scope

Cluster / Sentinel / Streams full / pubsub 默认关闭

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/redisx-ssot-alignment.md](../../../../../docs/ssot/redisx-ssot-alignment.md)
