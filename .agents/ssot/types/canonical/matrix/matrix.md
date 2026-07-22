# types/canonical — Matrix

| 字段 | 值 |
|---|---|
| 状态 | **current-state 入口有效** |
| 更新 | 2026-07-23 |

| 关注面 | 当前裁定 | 权威入口 |
|---|---|---|
| v1–v1.3 committed 类型 | 12 个 strict serde JSON DTO/enum；精确版本可查询 | [wire-commitment-matrix.md](../plan/wire-commitment-matrix.md) |
| Unknown fields | committed DTO 与 Envelope 外层 deny；未知 enum variant 拒绝 | [spec.md](../spec/spec.md) §4 / §6 |
| Golden / N-1 | v1、v1.1、v1.2、v1.3 均有受测向量；不外推通用迁移 | [test.md](../test/test.md) |
| Validation owner | canonical 只管 shape；业务语义由 adapter/domain owner 承担 | [validation-owners.md](../plan/validation-owners.md) |
| Envelope | 运输包装；调用者显式 validate；不自动路由 | [design.md](../design/design.md) |
| Residual | package stable / newtype / layout / serde 与 consumer 迁移仍 deferred | [residual-open.md](../plan/residual-open.md) |
| 历史 infra 对齐 | 保留 2026-07-21 时点证据，不覆盖 current-state | [alignment-matrix-infra-2026-07-21.md](../plan/alignment-matrix-infra-2026-07-21.md) |

**边界**：本矩阵中的 committed 指 strict serde JSON DTO shape/L2 subset，不是 canonical bytes、通用 codec、跨语言协议或 package stable。
