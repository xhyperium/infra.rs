# Goal — `decimalx` 金融数值安全与兼容性闭合

| 字段 | 值 |
|---|---|
| Goal ID | `GOAL-TYPES-DECIMALX-002` |
| Status | **Draft / Non-normative candidate** |
| Package / lib | `xhyper-decimalx` / `decimalx` |
| Path / version | `crates/types/decimal` / `0.1.0` |
| Candidate Spec | [SPEC-TYPES-DECIMALX-002](./xhyper-decimalx-complete-spec.md) |
| Active SSOT | [decimalx-spec.md](../decimalx-spec.md) |
| Snapshot | `95102919`（2026-07-15） |
| Supersedes | `none while Draft` |

> `[KNOWN]` / `[INFERRED]` 为证据标签；`PROPOSED` / `OPEN` 为需求状态。`[KNOWN] HIGH` 的失败条件是源码、Cargo、ADR-006/007 或消费方已变化。

## 1. 当前基线

- `[KNOWN] HIGH` `decimalx` 是 `/types/` 的 `Decimal`、`Price`、`Qty`、`Ratio`、`Currency`、`Money` 唯一定义点；路径和依赖方向由 ADR-007 批准。
- `[KNOWN] HIGH` ADR-006 已批准五种 `RoundingStrategy`、scale 对齐和显式舍入；当前 checked add/sub/mul/div/rescale 已落地。
- `[KNOWN] HIGH` `Decimal`、newtypes、`Currency`、`Money` 的字段公开，可绕过解析和未来 limits；合法最大 scale 尚未批准。
- `[KNOWN] HIGH` `Add/Sub/Mul` 和 `rescale` 在溢出时 panic；checked API 返回 `XError::Invalid`。公开注释已明确这一现状，但它不是理想生产主路径。
- `[KNOWN] HIGH` serde 当前暴露结构字段 shape；domain、ledger、exchange adapters、taos、schema_codegen、canonical 等有广泛消费，任何私有化/迁移/移除 serde 都是高影响变更。

关键反例：若字段已私有、panicking API 已移除/改义、serde shape 已版本化，或定向消费扫描不再命中上述模块，对应风险和迁移范围必须重新计算。

## 2. 目标结果

在保持 `/types/decimal` 与 `canonical → decimalx` 无环依赖的前提下，把数值内核收紧为：

1. 金融计算不经过 `f32/f64`，不 silent wrap，不 implicit rounding。
2. 合法表示边界（scale、构造、Currency）明确；非法值不能从推荐 API 进入。
3. checked API 是生产主路径；panicking operator/`rescale` 有清晰兼容、弃用或受限使用策略。
4. Eq/Ord/Hash 始终按数值一致，尾随零表示差异不破坏 map/set key。
5. serde、文本、数据库和协议边界分别定义，不把 derive 等同稳定 wire。
6. 所有破坏性收紧先通过 consumer inventory、迁移 API 和真实数据兼容验证。

## 3. 对原草案方向的裁定

| 候选 | 裁定 |
|---|---|
| 迁移到 `crates/types/numeric` | `REJECTED`：违反当前批准路径且无收益证据 |
| `decimalx → canonical` | `REJECTED`：形成 `canonical → decimalx → canonical` 循环 |
| 删除 `kernel`、serde | `OPEN`：当前错误和 wire 消费未迁移，不能预设 |
| 字段私有化、`MAX_SCALE` | `PROPOSED`：先裁定值与兼容构造 |
| `Money<U>` 泛型单位 | `REJECTED` 作为默认方案：当前 `Currency` 已表达货币，缺少真实泛型单位消费者 |
| `RoundingStatus` / `DecimalContext` / 新 rounding modes | `OPEN`：需真实 inexact/precision use case |
| checked arithmetic 主路径 | ADR-006 要求为 `APPROVED`；当前实现为 `[KNOWN] HIGH` |

## 4. 非目标

- 不实现汇率、跨币种运算、tick/step 市场规则、会计或手续费政策。
- 不拥有 canonical encoding、JSON/Protobuf/PostgreSQL driver；只提供经过批准的值转换边界。
- 不以 BigInt 或新的数值后端替换 i128，除非边界证据证明必要。
- 不因文档追求“完整”一次性破坏所有公开字段和 wire shape。

## 5. 分阶段结果

### M0 — Consumer 与风险基线

固定 API/wire/consumer inventory；补齐最大 scale、Display、比较、解析与 panicking API 的边界测试。

### M1 — Limits 与安全构造

批准 `DecimalLimits` 或等价边界；新增 fallible 构造/accessor，迁移内部与下游，不立即关闭字段。

### M2 — Panic 面收敛

统计 operators/`rescale` 调用；生产路径迁移到 checked API；按兼容政策 deprecate、限制或保留明确 panic 合同。

### M3 — Wire 与存储

分别批准 serde 文本/结构兼容、DB 精度映射和 protocol 表示；建立 golden/differential/property 证据。

## 6. 完成定义

- [ ] ADR-006 的舍入、scale、正负中点和溢出合同有 property/differential 测试。
- [ ] `MAX_SCALE`/构造/Currency 不变量已批准，推荐 API 无法产生非法值。
- [ ] 生产 consumer 不依赖未说明的 panic 或 float 转换。
- [ ] Eq/Ord/Hash 对全边界一致，比较不因对齐溢出失真。
- [ ] serde/text/storage/protocol 的稳定范围与迁移分别登记。
- [ ] 路径和依赖 DAG 保持批准状态；下游 adapters/domain/tools 全部验证。

本 Goal 以金融正确和可迁移为先，不以“新类型数量”或一次性重写为完成标准。
