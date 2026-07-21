# contracts SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 域 | `contracts/`（adapter trait 出口） |
| 镜像 | `.agents/ssot/contracts/**`（R6 只读；**禁止**改镜像冒充本仓完成） |
| 本仓路径 | `crates/contracts` · package `infra-contracts` |
| 审计日期 | 2026-07-21 |
| 结论 | **SSOT 镜像已注册**；**trait 出口已落地**（#43/#46）；**未**宣称 package stable / 全量 wire |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游镜像 COMPLETE 叙事 | 描述 xhyper monorepo 战役；**禁止**单独当本仓交付证明 |
| 镜像同步 | **完整**：与上游 `diff -rq` = 0（16 文件） |
| 本仓 `infra-contracts` | **已注册**；`ExchangeAdapter` / `StorageAdapter` + 共享类型 |
| 金额类型 | `Ticker` 使用 `decimalx::Price`（禁止 f64） |
| 业务实现 | **不在本 crate**；实现在 `crates/adapters/**` |
| package stable | **未**宣称 |

## 本仓可观察事实

```text
crates/contracts/               EXISTS · members 已注册
package                         infra-contracts
生产依赖                        serde + thiserror + xhyper-decimalx
publish                         false
Active SSOT                     .agents/ssot/contracts/spec/spec.md
```

验证：

```bash
diff -rq /home/workspace/xhyper.rs/.agent/SSOT/contracts .agents/ssot/contracts
cmp .agents/ssot/contracts/spec/spec.md \
    .agents/ssot/contracts/spec/xhyper-contracts-complete-spec.md
cargo test -p infra-contracts --all-targets
cargo clippy -p infra-contracts --all-targets -- -D warnings
```

## 对齐矩阵

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| C-1 | 镜像路径 `.agents/ssot/contracts/` | PASS | rsync + diff 0 |
| C-2 | dual-spec 同构 | PASS | `cmp` |
| C-3 | workspace member 已注册 | PASS | 根 `Cargo.toml` |
| C-4 | 仅 trait/type，无客户端实现 | PASS | `src/{exchange,storage,error,lib}.rs` |
| C-5 | 金额非 f64 | PASS | `Ticker::{bid,ask,last}: Price` |
| C-6 | package stable | OPEN | 未宣称 |

## 与 adapters 关系

- `binancex` / `okxx` 实现 `ExchangeAdapter`
- storage adapters 实现 `StorageAdapter`（多数仍为 stub）
- 依赖方向：`adapters/*` → `infra-contracts` → `decimalx` → `kernel`

## 未做（OPEN）

1. 完整 wire 承诺矩阵
2. storage trait 与真实后端对齐
3. package stable / Spec Approved 本仓宣称

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)（若已合入）
- [workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
