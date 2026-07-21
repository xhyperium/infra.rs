# decimalx

`/types/` 十进制数值类型（ADR-006 / ADR-007，spec §4.2）。纯基础数值层，无业务逻辑。

| 项 | 值 |
|----|-----|
| package | `decimalx` |
| lib | `decimalx` |
| path | `crates/types/decimal` |
| version | `0.1.0` |
| publish | `false`（internal only） |
| **生产层级** | **L1 Internal Ready** |
| 支持矩阵 | Linux x86_64 · MSRV 1.85 |

> 进程内金额/数量计算可用；**不是** package stable / 跨版本 wire 协议 / crates.io。

## 主要内容

- `Decimal`：`mantissa × 10^(-scale)`；数值 `Eq`/`Ord`/`Hash`（非结构字段）
- **生产主路径**：`try_new` / `FromStr` / `checked_add` / `checked_sub` / `checked_mul` / `checked_div` / `checked_rescale`
- panicking：`+` / `-` / `*` / `rescale` 在溢出时 panic；**非**推荐生产错误处理
- `RoundingStrategy`：Floor / Ceiling / HalfUp / HalfDown / HalfEven
- Newtypes：`Price` / `Qty` / `Ratio`；`Currency` / `Money`

## 硬限制

- `MAX_SCALE = 18`；字段私有，非法 scale 不可在 crate 外表示
- 禁止 `f32` / `f64` 参与金额 / 数量运算
- 资金路径只用 `checked_*`；CI 门禁扫描 panicking 运算符
- serde 字段 shape 为**当前事实**，**不等于**跨版本 stable（见 `docs/WIRE.md`）
- 不提供汇率、跨币种运算、tick/step、会计/手续费政策

## 最小用法

```bash
cargo run -p decimalx --example basic
```

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

## 验证

```bash
cargo test -p decimalx --all-targets
cargo clippy -p decimalx --all-targets -- -D warnings
cargo bench -p decimalx --bench hot_path -- --quick
cargo test -p decimalx --test oracle_diff --test boundary_matrix --test adversarial_serde
node scripts/quality-gates/check-decimal-no-panicking-ops.mjs
```

公开 API：[docs/API.md](./docs/API.md) · Wire 边界：[docs/WIRE.md](./docs/WIRE.md) · 变更日志：[CHANGELOG.md](./CHANGELOG.md)
