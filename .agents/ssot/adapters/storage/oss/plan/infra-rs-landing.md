# infra.rs 本仓落地说明 — ossx

| 字段 | 值 |
|------|-----|
| package | `ossx` |
| 实现路径 | `crates/adapters/storage/oss` |
| 生产默认面 | OssClient (OSS V1) |
| scaffold | `feature = "scaffold"`（可选 mock） |
| live | `tests/live_object_store.rs`（默认 `#[ignore]`） |
| 凭据 | `FOUNDATIONX_*` via `scripts/live/build-foundationx-env.mjs` |
| PR | #188 · #189 · #190 · #191 |
| 对齐 | [docs/ssot/adapters-ssot-alignment.md](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| package stable | **未宣称** |
| 当前加固 | infra-2d9.3.4：HTTPS/资源上界/multipart orphan/retry deadline |

## 硬限制

1. 本文件描述 **infra.rs 本仓 P0 生产入口**，不是 monorepo 战役 COMPLETE。
2. multipart 基础面已落地并完成资源/完整性加固；lifecycle、STS、流式 TB 对象仍 **OPEN**。
3. 无 live 证据不得宣称“全后端 Production Ready”。
4. Initiate/Complete 不自动重放；外部 drop future 由 RAII 写入有界 orphan registry，调用方可
   取回 key/UploadId 后显式 abort。进程崩溃清理由 lifecycle 承担，仍 OPEN。

## 验证

```bash
cargo test -p ossx --all-targets
cargo clippy -p ossx --all-targets -- -D warnings
cmp .agents/ssot/adapters/storage/oss/spec/spec.md \
    .agents/ssot/adapters/storage/oss/spec/xhyper-ossx-complete-spec.md
# live:
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p ossx -- --ignored
```
