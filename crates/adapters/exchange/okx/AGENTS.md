# okxx — Agent 行为规则

> 适用 crate：`crates/adapters/exchange/okx/`（包名 `okxx`）  
> 父级规则：[`crates/AGENTS.md`](../../../AGENTS.md)  
> 合约：`infra-contracts` · `ExchangeAdapter`

## 规则

### A1: scaffold 边界

- 未获 Spec + 证据前，禁止宣称 Done / Stable / 真实 I/O 完成
- 金额字段禁止 `f32`/`f64`；使用 `decimalx::Price`

### A2: 依赖

- 生产依赖：`infra-contracts`、`xhyper-decimalx`（均 path+version）
- 禁止 kernel/types 反向依赖本 crate
