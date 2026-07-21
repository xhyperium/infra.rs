# kafkax — Agent 行为规则

> 适用 crate：`crates/adapters/storage/kafka/`（包名 `kafkax`）  
> 父级：[`crates/AGENTS.md`](../../../AGENTS.md)  
> 合约：`infra-contracts::StorageAdapter`

## 规则

- scaffold 阶段禁止宣称真实 I/O / package stable
- 生产依赖仅 `infra-contracts`（path+version）
- 新增真实 SDK 依赖须 RFC / 显式 PR 说明
