# Canonical wire 兼容矩阵（Accept 路径 · infra-s9t.10）

| 字段 | 值 |
|------|-----|
| 状态 | **Accept（无 envelope）** |
| 方案 | B — 对外兼容矩阵 + 升级路径，**暂不**引入 `schema_version` envelope |

## Committed

| 版本标签 | 覆盖 | 规则 |
|----------|------|------|
| `COMMITTED_WIRE_V1` … `V1_3` | 公开市场 DTO 子集 | `deny_unknown_fields` 在类型上声明时拒绝 extra |
| envelope | **无** | 破坏性变更走 major / 双读期（见 WIRE.md） |

## 升级路径

1. 新增字段：带默认 + 文档；旧消费者忽略未知（若类型未 deny_unknown）。
2. 删除/改名：major 或迁移期双字段。
3. envelope（方案 A）作为 follow-up ADR，不阻塞本 Accept。

## 变更

| 日期 | 说明 |
|------|------|
| 2026-07-21 | infra-s9t.10 方案 B 落盘 |
