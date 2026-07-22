# types/decimal — 本仓 SSOT 入口

| 项 | 当前事实 |
|---|---|
| 实现 | `crates/types/decimal` |
| package / lib / version | `decimalx` / `decimalx` / `0.1.2` |
| Active Spec | [spec/spec.md](spec/spec.md) |
| 声明边界 | L1 checked path；不是跨语言 wire/package stable |

## 当前合同

- 私有 `Decimal` / `Currency` / `Money` 字段与校验型 serde。
- 资金路径只用 `try_new` / parse / `checked_*`；舍入策略必须显式。
- `new` / `rescale` 与 `panicking-ops` 只属于 const/test/兼容便利，不是生产资金路径。
- 所有可表示 Decimal（含 `i128::MIN`）的 Display→FromStr 数值往返必须闭合。
- `DecimalError → XError` 必须保留 source chain。
- wire v1 只承诺内部 Rust serde JSON shape；JSON `i128` 跨语言精度仍是 residual。

## 管线入口

[design](design/design.md) · [test](test/test.md) · [gate](gate/gate.md) ·
[matrix](matrix/matrix.md) · [residual](plan/residual-open.md)

历史 complete/campaign 文件只作来源，不与 active spec 做 `cmp`，也不继承 PASS。
