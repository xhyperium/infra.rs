# decimalx

`/types/` 十进制数值类型（ADR-006 / ADR-007，spec §4.2）。纯基础数值层，无业务逻辑。

Package：`xhyper-decimalx` · lib：`decimalx` · path：`crates/types/decimal` · version：**0.1.0**

## 主要内容

- `Decimal`：`mantissa × 10^(-scale)`；数值 `Eq`/`Ord`/`Hash`（非结构字段）。
- **生产主路径**：`try_new` / `FromStr` / `checked_add` / `checked_sub` / `checked_mul` / `checked_div` / `checked_rescale`。
- panicking：`+` / `-` / `*` / `rescale` 在溢出时 panic（见 rustdoc `# Panics`）；**非**推荐生产错误处理。
- `RoundingStrategy`：除法/缩位必须显式指定（Floor / Ceiling / HalfUp / HalfDown / HalfEven）。
- Newtypes：`Price` / `Qty` / `Ratio`；`Currency` / `Money`。

## 生产路径

1. 构造：`Decimal::try_new` 或 `"1.25".parse()`（强制 `scale ≤ MAX_SCALE(18)`）
2. 运算：仅 `checked_*`；不要在资金路径依赖 `+/-/*` / `rescale`
3. 校验：公开字段可绕过时，入口处 `validate()`
4. wire：serde 字段 shape 为当前事实，**不**等于跨版本 stable（见 `docs/WIRE.md`）

## 定位

- **Decimal 族唯一定义点**（ADR-007）。
- 禁止 `f32` / `f64` 参与任何金额 / 数量运算（ADR-006）。
- Active 实现合同：`.agents/ssot/types/decimal/spec/spec.md`。
- 候选完整规范（**Draft，非权威**）：`.agents/ssot/types/decimal/20260717/`。

## 限制与安全

- 字段当前 `pub`：可构造任意 `scale`；`MAX_SCALE` 治理层正式批准仍开放（residual T-HUM-001）。
- `Currency::from_str` 校验 3 位大写 ASCII；公开字段仍可绕过。
- 不提供汇率、跨币种运算、tick/step、会计/手续费政策。

## 测试

```bash
cargo test -p xhyper-decimalx
cargo check -p xhyper-decimalx --all-targets
cargo clippy -p xhyper-decimalx --all-targets -- -D warnings
```

## 版本

0.1.0（见 `Cargo.toml`）。未宣称 package stable。未宣称 Spec Approved / Goal Achieved。
