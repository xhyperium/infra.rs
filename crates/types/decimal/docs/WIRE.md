# decimalx Wire / Storage 边界（生产政策）

**Status**：Policy · **非**跨版本 stable  
**Package**：`xhyper-decimalx` 0.1.x

| 边界 | 当前事实 | 稳定承诺 |
|------|----------|----------|
| serde | `{mantissa, scale}` 结构字段 | **无** |
| Display/FromStr | 文本；FromStr 强制 `scale≤MAX_SCALE(18)` | 非持久化协议 |
| SQL NUMERIC | 由上层 adapter/schema 映射 | 非本 crate 合同 |

生产规则：金额入口 `try_new`/`parse`/`validate`；运算 `checked_*` only；不得仅凭 derive 宣称 wire stable。
