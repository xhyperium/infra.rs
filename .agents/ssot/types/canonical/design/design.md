# types/canonical — Design

| 字段 | 值 |
|---|---|
| 状态 | **current-state 有效** |
| 更新 | 2026-07-23 |
| 层级 | L2 committed wire subset（strict serde JSON DTO shape） |
| 非目标 | package stable、canonical bytes、通用 codec、跨语言协议 |

## 模块边界

| 模块 / 数据面 | 职责 | 明确不负责 |
|---|---|---|
| crate root DTO | 跨层共享纯数据形状；v1–v1.3 committed 清单 | 业务状态机、I/O、授权、风控 |
| `wire` | committed 类型清单、精确 `WireVersion` 查询、coarse 兼容查询 | 序列化执行、schema registry、迁移路由 |
| `proposed_time` | Unix ms↔ns 边界转换 | 时钟、时区、自动单位猜测 |
| `shape` | adapter 入口可复用的轻量形状检查 | domain/business validation |
| `envelope` | `schema_version` + `payload` 运输包装 | 自动版本协商、decoder 选择、业务校验 |
| `decimalx` re-export | 复用 `Money`；Price/Qty/Decimal shape 来源 | 在 canonical 重定义数值语义 |

## Wire 设计

```text
类型名 ── committed_wire_version ──> V1 / V1_1 / V1_2 / V1_3 / None
   └── wire_commitment（兼容）──────> CommittedV1 / Uncommitted
```

- `committed_wire_version` 是精确查询；`wire_commitment` 只保留既有二值接口，不得替代精确版本判断。
- committed 合同是 serde JSON DTO shape：字段/variant 名、必填性、未知输入拒绝及 decimal shape。
- committed 表示由文件或 inline golden 固定；有登记的 legacy/N-1 向量保持可读。该证据不生成 canonical bytes，也不声称跨语言或跨大版本协议。

## Envelope 设计

Envelope 仅包装 payload。`wrap_current` 不会验证或路由版本；反序列化后，调用者必须显式执行 `validate_version` 或 `into_payload_if_version`，成功后才消费 payload。外层 `deny_unknown_fields` 不替代 payload 自身的 strict serde 合同。

权威清单见 [wire-commitment-matrix.md](../plan/wire-commitment-matrix.md)，剩余裁定见 [residual-open.md](../plan/residual-open.md)。
