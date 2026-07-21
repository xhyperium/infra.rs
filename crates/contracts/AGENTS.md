# infra-contracts — Agent 行为规则

> 适用 crate：`crates/contracts/`（包名 `infra-contracts`）  
> 父级规则：[`crates/AGENTS.md`](../AGENTS.md)  
> 本仓对齐：[`docs/adapters-ssot-alignment.md`](../../docs/adapters-ssot-alignment.md)

---

## 职责

Adapter trait 出口（Additive Only）。只放 trait / type，不放实现。

---

## 规则

### C1: 无实现

- 禁止在本 crate 放入具体 exchange/storage 客户端
- 实现落在 `crates/adapters/**`

### C2: 依赖

- 生产依赖白名单：`serde`、`thiserror`
- 禁止依赖具体 adapter crate（防环）

### C3: 金额

- 公开金额/价格字段禁止 `f32`/`f64` 作为终态合同；当前 `Ticker` 的 `f64` 为 **已知缺口**，收口前不得宣称 stable

### C4: 布局

- 遵循 `crates/AGENTS.md` 标准七项
