# Residual Open — `canonical` current-state

| 字段 | 值 |
|---|---|
| 当前版本 | worktree package `canonical` 0.1.2 |
| 更新 | 2026-07-23 |
| 原则 | 仅记录真实剩余项；已闭合的 wire 合同不得继续标成 OPEN |

## 仍需 HUMAN / DEFER

| ID | 状态 | 摘要 | 当前边界 |
|---|---|---|---|
| DEFER-STABLE | HUMAN_ONLY | package stable / crates.io 发布 | 当前 `publish = false`；L2 subset 不等于 package stable |
| DEFER-ID-NEWTYPE | DEFER | `OrderRef` / venue newtype 二期 | 当前 `Order.id` / `OrderAck.id` 仍为 wire `String` |
| DEFER-LAYOUT | DEFER / RFC | types/core、types/protocol 大搬迁 | 不在本轮；不得破坏现有路径与清单 |
| DEFER-SERDE | DEFER / RFC | 移除或替换 serde | v1–v1.3 合同即 strict serde JSON shape，须先有迁移证据 |
| DEFER-CONSUMERS | DEFER | 非主路径 consumer 全量迁移 | 不扩大 current-state DTO 承诺 |
| HUMAN-NEXT-WIRE | 按需 | 未来破坏性 wire 版本或 migration reader | 只有出现具体变更提案时才建立版本化 RFC |

## 已 CLOSED，不再是 OPEN

| 旧 ID / 主题 | 当前裁定 |
|---|---|
| OPEN-WIRE-001 unknown-field | **CLOSED**：committed 类型 `deny_unknown_fields`；未知 variant 拒绝 |
| OPEN-WIRE-002 全清单 / golden | **CLOSED**：v1–v1.3 共 12 个 committed 类型，均有文件或穷举 inline golden；有登记的 legacy/N-1 向量保持可读 |
| OPEN-WIRE-003 enum 演进 | **CLOSED（当前版本）**：variant 名已冻结；未知 variant 拒绝；新增 variant 对 strict reader 是破坏性变化 |
| coarse 版本无法区分 | **CLOSED**：新增精确 `WireVersion` / `committed_wire_version`；旧 `WireCommitment` 保兼容 |
| ns→ms 无损语义 | **CLOSED**：`unix_millis_from_ns_exact`；`unix_millis_from_ns` 与兼容 alias 仍显式记录向 0 截断 |
| Envelope 缺版本检查入口 | **CLOSED**：提供显式 validate/consume API；仍由调用者主动调用，不自动路由 |

## 持续 REJECTED / OOS

| ID | 项 | 原因 |
|---|---|---|
| REJ-CANONICAL-BYTES | canonical bytes / 确定性二进制编码 | 当前承诺仅 serde JSON DTO shape |
| REJ-CODEC | 通用 codec / schema registry / 任意格式转换 | 超出纯 DTO 深模块边界 |
| REJ-CROSS-LANGUAGE | 将 v1–v1.3 宣称为跨语言协议 | 没有跨语言 conformance 证据 |
| REJ-AUTO-ROUTING | Envelope 自动协商、路由或选择 decoder | Envelope 只运输包装；版本由调用者显式 validate |
| REJ-HASH | hash/sign/evidence 链 | 归属其他层 |
| REJ-BIZ | 订单状态机、盘口 diff、风控/业务校验 | canonical 只负责数据形状 |

## 诚实完成面

| 项 | 状态 |
|---|---|
| v1 / v1.1 / v1.2 / v1.3 committed inventory | **DONE** |
| strict unknown-field / unknown-variant 策略 | **DONE** |
| 双向 golden / N-1 历史向量 | **DONE（受测向量范围）** |
| 精确 wire version 查询 | **本轮交付** |
| Envelope 显式版本校验 | **DONE；非自动路由** |
| package stable | **DEFER / HUMAN_ONLY** |
| canonical bytes / 通用 codec / 跨语言协议 | **OOS / REJECTED** |

权威清单见 [wire-commitment-matrix.md](./wire-commitment-matrix.md)。
