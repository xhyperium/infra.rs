# taosx — Agent 行为规则

> 适用 crate：`crates/adapters/storage/taos/`（包名 `taosx`）  
> 父级：[`crates/AGENTS.md`](../../../AGENTS.md)

## 规则

- scaffold 阶段禁止宣称真实 I/O / package stable
- 生产依赖当前仅 `thiserror`
- 本地 trait 与 `xhyper-contracts` 公共面尚未强制统一（follow-up）
