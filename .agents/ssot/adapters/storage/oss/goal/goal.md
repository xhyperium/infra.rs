# adapters/storage/oss — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `ossx` |
| 标题 | Aliyun OSS ObjectStore |
| 实现 | `crates/adapters/storage/oss` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 |
| 状态 | **P0 生产入口已落地**（#188–#191）；package stable **未宣称** |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 Aliyun OSS ObjectStore 生产默认客户端，满足 draft P0 DoD，且默认 `cargo test` 离线绿灯。

## Acceptance（本仓可验证）

1. workspace member `ossx` 可 `cargo test -p ossx --all-targets`
2. 生产默认面：`OssClient / OssConfig + sign_v1`
3. 环境注入：`FOUNDATIONX_OSSX_{ENDPOINT,BUCKET,ACCESS_KEY_ID,ACCESS_KEY_SECRET,REGION}`（密钥不入库）
4. live：`tests/live_object_store.rs` 默认 `#[ignore]`，真凭据可绿
5. bench：`benches/put_get.rs`（不得挂死 `--all-targets`）
6. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认

## Not in scope

multipart / lifecycle / STS 临时凭证

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/ossx-ssot-alignment.md](../../../../../docs/ssot/ossx-ssot-alignment.md)
