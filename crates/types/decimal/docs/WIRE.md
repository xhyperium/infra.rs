# decimalx Wire / Storage 边界（生产政策）

**Status**：Policy · **非**跨版本 stable  
**Package**：`decimalx` 0.1.x

| 边界 | 当前事实 | 稳定承诺 |
|------|----------|----------|
| serde `Decimal` | `{mantissa, scale}` 结构字段；反序列化走 `try_new` | **无** 跨版本协议 |
| serde `Currency` | 3 字节数组 `[u8;3]`（大写 ASCII） | **无** |
| serde `Money` | `{amount, currency}` | **无** |
| Display/FromStr | 文本；FromStr 强制 `scale≤MAX_SCALE(18)` | 非持久化协议 |
| SQL NUMERIC | 由上层 adapter/schema 映射 | 非本 crate 合同 |
| 中间值能力 | `i128` 溢出 → `Err`（即使约分后可表示） | 正式合同，非 silent bug |

生产规则：

1. 金额入口 `try_new` / `parse` / `validate`；运算 **仅** `checked_*`。
2. 资金路径禁用 panicking `+` / `-` / `*` / `rescale`（门禁：`scripts/quality-gates/check-decimal-no-panicking-ops.mjs`）。
3. 不得仅凭 derive 或 100% 行覆盖宣称 wire stable / Production Ready。
4. 正确性证据：`tests/oracle_diff.rs`（BigDecimal 差分，仅 `Ok` 路径）+ `tests/boundary_matrix.rs` + scheduled mutants/miri。
