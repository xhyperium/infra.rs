# adapters/storage/redis — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `redisx` |
| 当前版本 | `0.3.15` |
| 标题 | Redis KV |
| 实现 | `crates/adapters/storage/redis` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 |
| 状态 | **P0 生产入口已落地**（#188–#191）；package stable **未宣称** |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 Redis KV 生产默认客户端，满足 draft P0 DoD，且默认 `cargo test` 离线绿灯。

## Acceptance（本仓可验证）

1. workspace member `redisx` 可 `cargo test -p redisx --all-targets`
2. 生产默认面：`RedisPool / RedisClient / RedisConfig`
3. 环境注入：`FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS,MODE,NODES,SENTINEL_MASTER}`（密钥不入库）
4. live：`tests/live_kv.rs · tests/live_kv_conformance.rs` 默认 `#[ignore]`，真凭据可绿
5. bench：`benches/kv_hot_path.rs`（不得挂死 `--all-targets`）
6. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认

## 当前边界

- Cluster / Sentinel / TLS 命令代码路径存在，但真实拓扑、握手、切换和恢复证据保持 OPEN。
- Pub/Sub 仅 Standalone；Cluster / Sentinel 失败关闭，重连与必达 NO-GO。
- 配置 budget 后，GET/EXISTS/PTTL/MGET 按 `ReadOnly` 重试，无 TTL SET/MSET 按 `Idempotent` 重试。
- 相对 TTL SET、DEL、PEXPIRE 的多次尝试在 I/O 前拒绝；PUBLISH 不自动重试。单命令原子性不消除
  响应丢失后的结果歧义。
- `RedisOperation::Set` 是无法表达 TTL 参数的粗粒度查询面，保守保持 `AmbiguousWrite`；client 按
  实际 TTL 参数细分。

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/redisx-ssot-alignment.md](../../../../../../docs/ssot/redisx-ssot-alignment.md)
