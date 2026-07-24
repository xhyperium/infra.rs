# ADR-001：Instrument canonical owner 迁移（DX-CAN-001 / DM-CAN-001）

**状态**：Accepted for planning；**运行时 blocked**（`xhyper-canonical` 未纳入 workspace）  
**日期**：2026-07-23  
**关联**：DX-CAN-001、DM-CAN-001

## 背景

- `domainx` 交易对象使用 `instrument: String` 兼容占位。
- `domain_market` 行情对象使用 `InstrumentKey { exchange, symbol }`。
- 历史文档曾引用 `xhyper-canonical`，但该 crate **不是**当前 workspace 成员。

双轨并存导致：跨域关联依赖字符串约定；adapter 必须自行维护双向映射。

## 决策

1. **在 `xhyper-canonical`（或组织选定的唯一 owner crate）以 path/workspace 依赖进入本仓之前**，不得在 domain 层再引入第三套 instrument 结构。
2. 唯一 owner 落地后：
   - `domain_market::InstrumentKey` 迁移为 re-export 或 newtype 包装 canonical 类型；
   - `domainx` 订单/成交/持仓字段从 `String` 改为同一 canonical 类型（或稳定 `InstrumentId`）；
   - 五个 exchange adapter + `domain_exchange` 同步替换，单次迁移 PR 或有序 stack；
   - 提供 `From`/`TryFrom` 与版本化 JSON fixture（camelCase 字段名冻结）。
3. 迁移完成前：
   - 门禁状态保持 **blocked**；
   - adapter 文档必须声明 `String` ↔ `InstrumentKey` 映射规则。

## 后果

- 正面：消除双轨、跨域 join 可类型化。
- 负面：依赖外部/并行仓发布时间；迁移为 breaking change，需 SemVer major 或 `0.x` 显式声明。
- 不在本 ADR 范围：产品线（ProductLine）是否并入 canonical（可在迁移 PR 二次确认）。

## 验收（解除 blocked 时）

- [ ] workspace 成员包含 canonical crate
- [ ] domainx/domain_market/domain_exchange 无第二套 instrument struct
- [ ] `cargo test --workspace` + 各 adapter fixture 通过
- [ ] SSOT 门禁 DX-CAN-001 / DM-CAN-001 标为 verified 并附命令证据
