# adapters/storage/nats — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `natsx` |
| 标题 | NATS Core EventBus |
| 实现 | `crates/adapters/storage/nats` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 |
| 状态 | **P0 生产入口已落地**（#188–#191）；package stable **未宣称** |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 NATS Core EventBus 生产默认客户端，满足 draft P0 DoD，且默认 `cargo test` 离线绿灯。

## Acceptance（本仓可验证）

1. workspace member `natsx` 可 `cargo test -p natsx --all-targets`
2. 生产默认面：`NatsPool / NatsEventBus / NatsSubscription`
3. 环境注入：`FOUNDATIONX_NATS_{URL,USER,PASSWORD} 或 FOUNDATIONX_NATSX_*`（密钥不入库）
4. live：`tests/live_event_bus.rs` 默认 `#[ignore]`，真凭据可绿
5. bench：`benches/hot_path.rs（3s 有界）`（不得挂死 `--all-targets`）
6. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认

## Not in scope

JetStream 全量 / NKey / TLS 默认开启策略

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/natsx-ssot-alignment.md](../../../../../docs/ssot/natsx-ssot-alignment.md)
