# domain_exchange — 交易域模型

**SSOT 根**：`.agents/ssot/domain_exchange/`
**规格**：`spec/spec.md`
**crate 路径**：`crates/domain_exchange`
**package/lib**：`domain_exchange`
**当前状态**：trait/DTO、mock 生命周期、结构化错误、能力矩阵和默认分页已建立；live 生命周期与各 adapter 映射待实现。

## 概述

交易域模型定义交易所接入的统一异步抽象层。`VenueAdapter` 当前保留 13 个方法，覆盖连接、公开行情订阅、订单、账户、标的元数据和订单簿快照；具体交易所协议由 adapter 负责。

## 落地状态

- **Status**：契约冻结 — trait/DTO 与 mock/default 契约已存在，live adapter 行为待实现
- **crate**：`crates/domain_exchange`
- **门禁**：见 `spec/spec.md` 的 `DE-*` 表
