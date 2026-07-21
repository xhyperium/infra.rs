# Consumer migration — decimalx production P0

| 字段 | 值 |
|------|-----|
| Date | 2026-07-17 |
| Branch | `feat/decimalx-prod-p0-hardening` |
| PR | #509 |

## 本轮已迁移（生产入口）

| 路径 | 变更 |
|------|------|
| `adapters/exchange/binance/src/parser.rs` | `parse_decimal` → `FromStr`（`MAX_SCALE`） |
| `adapters/exchange/okx/src/parser.rs` | 同上；`Money::try_new` |
| `adapters/storage/taos` util | `parse_decimal` → `FromStr` |
| `domain/ledger` | `balance`/`balance_checked` checked_add；prop 累加 checked_add |
| `domain/exchange/venue_safety` | 已是 checked_*（无需改） |

## 仍可后续迁移（非阻塞）

| 路径 | 说明 |
|------|------|
| 测试/fixture 中 `Decimal::new(...)` | 测试字面量，scale 均 ≤18；可保留 `new` |
| `canonical` 测试辅助 `Decimal::new` | 同上 |
| 全量删除 panicking operators | DEFERRED：需 consumer=0 |

## 门禁

```bash
bash .agents/ssot/types/decimal/plan/scripts/check-prod-money-paths.sh
cargo test -p xhyper-decimalx -p xhyper-ledger -p xhyper-binance -p xhyper-okx
# taos: cargo test -p xhyper-taosx
```
