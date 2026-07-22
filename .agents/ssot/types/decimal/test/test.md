# decimalx 测试合同

> **状态**：Active test strategy
>
> 测试代码位于 `crates/types/decimal/src/lib.rs` 与 `crates/types/decimal/tests/`；本文件不保存测试副本。

## 必须验证的生产合同

| 维度 | 最低测试集合 | 失败条件 |
|---|---|---|
| 构造边界 | mantissa `0` / `i128::MAX` / `i128::MIN`；scale `0` / `1` / `18` / `19` | 合法值被拒绝，或 scale 19 被接受 |
| 私有字段入口 | `try_new`、访问器、自定义 serde 反序列化 | serde 绕过 `MAX_SCALE` / Currency 校验 |
| 文本往返 | 任意可表示 Decimal；显式覆盖 `i128::MIN` × scale `0/1/18` | `Display` 后无法 parse，或数值不相等 |
| 超长文本诊断 | 256 位小数输入拒绝；诊断保留真实长度且不出现 scale 0 窄化 | 长度经窄化回绕或诊断伪报 |
| checked 算术 | add/sub/mul/div/rescale 的成功、对齐溢出、mantissa 溢出、scale 溢出、除零 | panic、静默回绕或错误成功 |
| 舍入 | 五种策略；正负；小于/等于/大于 half；奇偶商 | 策略方向或中点规则错误 |
| 数值语义 | 尾随零不同的 Eq/Ord/Hash | 相等值 hash 不同或排序不等 |
| 错误分类 | 每个 `DecimalError` 对应的 `DecimalErrorKind` 与中文 Display | 依赖字符串判断分类 |
| 错误链 | `DecimalError -> XError` 后调用 `source()` | source 缺失或不是原错误 |
| feature 边界 | 默认 feature 不提供 panicking ops；显式 feature 才提供 | 默认生产构建暴露 `+/-/*` |
| serde v1 | Decimal/Currency/Money shape、未知字段、非法值拒绝 | shape 漂移或反序列化绕过不变量 |

## 新增回归合同

### Display → FromStr

对任意 `Decimal::try_new(mantissa, scale)` 成功的值：

```text
let text = value.to_string();
let parsed: Decimal = text.parse()?;
assert_eq!(parsed, value); // 数值相等
```

边界样本必须包含 `mantissa = i128::MIN` 与 `scale = 0`、`1`、`MAX_SCALE(18)`。除定向测试外，
property 测试应覆盖任意 `i128` 和 `0..=18`，防止只修一个样本。

### DecimalError → XError

将任意代表性 `DecimalError` 转成 `kernel::XError` 后：

- `kind() == kernel::ErrorKind::Invalid`；
- `std::error::Error::source(&xerror)` 返回 `Some`；
- source 的类型/内容可追溯到原 `DecimalError`，不得只复制 Display 字符串。

## serde v1 的测试含义

golden JSON 只证明内部 Rust serde shape 未漂移，并证明反序列化校验生效。它不证明跨语言精确
承载：JSON consumer 可能无法精确表示 `i128`。因此测试报告不得把 serde round-trip 写成
“跨语言 wire stable”。跨语言 `i128` 风险保留在 [residual](../plan/residual-open.md)。

## 建议命令

```bash
cargo test -p decimalx
cargo test -p decimalx --features panicking-ops
cargo clippy -p decimalx --all-targets --all-features -- -D warnings
cargo fmt --all --check
node scripts/quality-gates/check-decimal-no-panicking-ops.mjs
node scripts/quality-gates/check-crate-versions.mjs
node scripts/quality-gates/check-workspace-deps.mjs
```

全仓交付还须执行宪章与仓库级门禁。本轮定向与四域 focused 测试结果记录在
`docs/report/2026-07-23-core-types-3x/`；最终 PASS 仍只能由验证任务绑定候选 commit 记录。
