# kafkax — Agent 行为规则

> 适用 crate：`crates/adapters/storage/kafka/`（包名 `kafkax`）  
> 父级规则：[`crates/AGENTS.md`](../../../AGENTS.md)  
> SSOT 镜像：[`.agents/ssot/adapters/storage/kafka`](../../../../.agents/ssot/adapters/storage/kafka/)  
> 本仓对齐：[`docs/adapters-ssot-alignment.md`](../../../../docs/adapters-ssot-alignment.md)

---

## 职责

kafka storage adapter（当前为 scaffold）。

---

## 规则

### A1: scaffold 边界

- 未获 Spec + 证据前，禁止宣称 Done / Stable / 真实 I/O 完成
- 禁止把上游镜像 COMPLETE 当作本仓交付证明

### A2: 依赖

- 当前生产依赖仅 `thiserror`
- 新增 SDK / 网络依赖必须走 RFC 或显式 PR 说明
- 禁止 kernel/types 反向依赖本 crate

### A3: 金额与精度

- 金额字段禁止 `f32`/`f64`；须经 `decimalx` / canonical 边界（实现阶段强制）

### A4: 布局

- 遵循 `crates/AGENTS.md` 标准七项
