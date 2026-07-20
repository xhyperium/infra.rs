# Production operators inventory — 2026-07-17

| 字段 | 值 |
|------|-----|
| Branch | `feat/decimalx-prod-p0-hardening` |
| PR | #509 |
| Scope | domain ledger/exchange · adapters binance/okx/taos · canonical（prod 树） |

## 结论

**生产资金路径已无 panicking Decimal 运算符依赖。**

| 检查项 | 结果 |
|--------|------|
| `.rescale(` 生产树 | 0 |
| `sum = sum +/ -` 累加 | 0（ledger 已 checked_add） |
| venue_safety 价格保护 | `checked_add/sub/mul/div` |
| binance/okx parse | `FromStr`（MAX_SCALE） |
| panicking `+/-/*` | 仅 decimalx 自身测试 + 文档示例；生产 crate 未依赖 |

## 仍保留（兼容，非生产主路径）

- `impl Add/Sub/Mul` / `rescale`：rustdoc 标明生产请用 checked_*
- 测试/fixture 中 `Decimal::new` 字面量（scale ≤ 18）

## 门禁

`check-prod-money-paths.sh` 第 1/5/6 项 + P0 `decimal-money-path-gate` 持续防回流。

## 删除 panicking API

**DEFERRED**：需 workspace 外消费者迁移完成后再 deprecate/remove。
