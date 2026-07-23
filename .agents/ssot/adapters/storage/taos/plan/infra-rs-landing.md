# infra.rs 本仓落地说明 — taosx

| 字段 | 值 |
|------|-----|
| package | `taosx` |
| 实现路径 | `crates/adapters/storage/taos` |
| 生产默认面 | TaosPool REST :6041 |
| scaffold | `feature = "scaffold"`（可选 mock） |
| live | `scripts/taos-live-conformance.mjs` → 固定 digest 隔离 TDengine + ignored test |
| 凭据 | 隔离容器每次生成临时随机密码且不输出；外部服务仅 `FOUNDATIONX_*`，禁止入库 |
| PR | #188 · #189 · #190 · #191 |
| 对齐 | [docs/ssot/adapters-ssot-alignment.md](../../../../../docs/ssot/adapters-ssot-alignment.md) |
| package 版本 | `0.3.7` |
| package stable | **未宣称** |
| SSOT 路径 | `adapters/storage/taos/`（**不**另建 `taosx/`） |

## 硬限制

1. 本文件描述 **infra.rs 本仓 P0 生产入口**，不是 monorepo 战役 COMPLETE。
2. Native SQL / FFI / WS auth / HA / 自动幂等重试 **NO-GO**。
3. 无 live 证据不得宣称“全后端 Production Ready”。

## 验证

```bash
cargo test -p taosx --all-targets
# 隔离 live（非 prod）：
# node scripts/taos-live-conformance.mjs
```
