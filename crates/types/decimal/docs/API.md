# decimalx 公开 API

**角色**：十进制 / Money

## 公开消费面

| 类型 | 要点 |
|------|------|
| `Decimal` | `new`/`try_new`/`checked_*`/`rescale`/`normalize` |
| `Price`/`Qty`/`Ratio` | newtype 包装 |
| `Currency`/`Money` | 合法币种 + 金额 |
| `RoundingStrategy` / `DecimalError` | 舍入与错误 |

## 最小用法

```rust
use decimalx::{Currency, Decimal, Money, RoundingStrategy};

let a = Decimal::new(10, 0);
let b = Decimal::new(3, 0);
let q = a.checked_div(b, RoundingStrategy::HalfEven).unwrap();
let ccy = Currency::try_new(*b"USD").unwrap();
let m = Money::try_new(a, ccy).unwrap();
assert_eq!(m.currency().as_str(), "USD");
let _ = q;
```
