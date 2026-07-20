# decimalx 实现规范

状态：当前 `0.1.0` 实现的 active 验收合同（wire/生产边界未稳定）
权威来源：`CONSTITUTION.md`、XLib spec v0.2、Approved ADR-006/007
crate：`crates/types/decimal`

- Package / lib：`xhyper-decimalx` / `decimalx`
- Implementation snapshot：`b0934baa`（2026-07-15）
- Document commit：`e0b98df4`
- Verified at：`e0b98df4`（相关实现路径未变化）
- Candidate：[SPEC-TYPES-DECIMALX-002](../20260717/xhyper-decimalx-complete-spec.md)（Draft，非权威，不覆盖本文）

## 1. 定位、证据等级与职责

本文区分：**Evidence**（权威文档或当前代码直接证明）、**Inference**（维持既定合同所需的最小结论）、
**Unknown/Proposed**（尚未批准，不能当成现有 API）。冲突时按
`CONSTITUTION.md` → XLib spec → Approved ADR → 当前实现处理；代码偏离上位合同处必须修代码或走
XLib spec §9，而不能由本文静默批准。

**Evidence**：`decimalx` 位于 `/types/`，是 `Decimal`、`Money`、`Currency` 族的唯一定义点；
它只提供纯数值/值类型，不含业务规则（XLib spec §§1、4.2；ADR-007 Decision 1）。

非目标：领域定价/手续费/会计政策、市场数据 DTO、存储/网络、时钟、审计、配置和运行时装配。
业务层可包装自己的默认舍入政策，但本 crate 不提供隐式默认（ADR-006 Decision 3）。

## 2. 位置、依赖与版本

| 项目 | 当前事实 | 合同 |
|---|---|---|
| 路径 | `crates/types/decimal` | **Evidence**：固定于 `/types/`（ADR-007） |
| 版本 | `0.1.0` | **Evidence**：独立维护；每次只允许 `x.y.z → x.y.(z+1)`（XLib spec §5；Constitution §7.3） |
| 生产依赖 | `xhyper-kernel`, `serde` | **Evidence**：当前 Cargo；kernel 为架构依赖，`serde` 为已声明外部依赖 |

ADR-007 历史文本中的 `xlib_standard` 当前对应 package `xhyper-kernel` / lib `kernel`；`serde` 是第三方序列化依赖，
须继续满足 Constitution Article VII 的必要性、安全与许可证审查。不得新增 workspace 生产依赖；
新增第三方依赖需先走相同审查，若改变架构合同还须走 XLib spec §9。

## 3. 当前公开 API（代码事实）

**Evidence**：`src/lib.rs` 当前公开：

- `Decimal { pub mantissa: i128, pub scale: u8 }`、`Decimal::new`、`Decimal::ZERO`；
- `eq_value` / `cmp_value`（数值语义）；
- `checked_add` / `checked_sub` / `checked_mul` / `checked_div` / `div` / `checked_rescale` / `rescale`；
- `Add`、`Sub`、`Mul` 运算符（内部走 checked，溢出 panic）；
- `PartialEq` / `Eq` / `PartialOrd` / `Ord` / `Hash` 为数值语义（非结构字段）；
- `RoundingStrategy::{Floor, Ceiling, HalfUp, HalfDown, HalfEven}`；
- `Price(pub Decimal)`、`Qty(pub Decimal)`、`Ratio(pub Decimal)`；
- `Currency(pub [u8; 3])`、`as_str`、`FromStr<Err = XError>`；
- `Money { pub amount: Decimal, pub currency: Currency }`；
- 值类型带 serde derive（形状为字段默认；`Decimal` 比较语义独立于 wire 字段）。

公开字段意味着调用方可构造任意 `scale` 和任意三字节 `Currency`；`Currency::from_str` 才保证
“恰好三个大写 ASCII 字母”。`as_str` 对非法公开字段返回空串。**Evidence**：这是当前行为，
不是已证明合理的长期不变量。

## 4. 数值行为与不变量

### 4.1 表示与 scale

**Evidence**：数值含义为 `mantissa × 10^(-scale)`（XLib spec §4.2）。加减和 `eq_value`
将两侧补零到较大 scale；乘法直接相乘 mantissa 并相加 scale。金额/数量计算禁止 `f32`/`f64`。

**Evidence（规范）**：ADR-006 要求所有双目运算先无损对齐到较大 scale；除法或缩位必须显式选择
舍入策略，不存在默认 fallback。

### 4.2 舍入

五种枚举名称已由 ADR-006 批准。`Floor` 向负无穷取整，`Ceiling` 向正无穷取整；三个 half 模式
分别表达远离零、趋向零和 ties-to-even 的中点策略。

**实现状态（2026-07-14 INFRA-000 审计后）**：

1. 已实现 `checked_add` / `checked_sub` / `checked_mul` / `checked_div` / `checked_rescale` 与
   `rescale`；`div` 为 `checked_div` 别名。运算符在溢出时 panic（禁止静默回绕）。
2. scale 对齐与算术均走 checked 路径；溢出映射为 `XError::Invalid`（上下文含 `overflow` /
   `division by zero` 等字面量——错误码枚举尚未单独裁定）。
3. 除法数值合同：`round(m1 · 10^(s_r − s1 + s2) / m2)`，`s_r = max(s1, s2)`；与“对齐后定点除”
   等价。除法**无**独立 `target_scale` 参数（缩位用 `rescale`）。
4. Half\* 中点用 `2·|r| ? |d|`（`unsigned_abs`），避免奇数分母误判与 `i128::MIN.abs()` 问题。
5. `PartialEq` / `Ord` / `Hash` 为**数值语义**（对齐比较 / 尾随零规范），serde 仍为结构字段形。

生产 fallible 路径已强制 `MAX_SCALE = 18`（`try_new` / `FromStr` / `checked_*` 结果）。
`Decimal::new` 仍可构造越界 scale（兼容）。serde wire 跨版本、字段私有化、独立 DecimalError 仍开放。

### 4.3 Currency 与 Money

`Currency::from_str` 接受且仅接受长度 3 的大写 ASCII；错误为
`XError::Invalid("currency must be 3 uppercase ASCII letters")`。`Money` 只组合数值与币种，
不批准跨币种运算、汇率、格式化或币种小数位规则。`Price`/`Qty`/`Ratio` 仅为 newtype，
当前没有额外校验或运算。

## 5. 错误、并发、序列化与兼容性

- **Evidence**：除零、币种解析与 checked 算术溢出当前均返回 `XError::Invalid`；上下文字面量已有测试，
  但独立错误类型/变体与跨版本兼容尚未裁定。
- **Evidence**：全部类型只含拥有的值；当前 API 无共享可变状态、I/O、异步或锁。
- **Inference**：值类型可跨线程移动不等于 crate 承诺任何全局并发/原子性语义。
- **Evidence**：serde 派生当前暴露结构字段/枚举名称的默认数据形状。
- **Unknown**：持久化 wire format、schema version、人类可读十进制文本、兼容迁移和 canonical encoding
  均未裁定。crate 政策见 `crates/types/decimal/docs/WIRE.md`。不得仅凭 derive 宣称跨版本格式稳定。

首个公开发布后的破坏性 Rust API 或序列化形状调整须记录 CHANGELOG、提供迁移说明并走 RFC；
此兼容/RFC 政策独立于“版本号只增 patch 位”的编号规则（XLib spec §§5、9）。XLib spec §5 的
Additive Only 仅适用于 `contracts` trait 层，不适用于本 crate。

## 6. 测试合同

当前 35 个内联测试 + 12 个 proptest 测试已覆盖（2026-07-15）：

1. 五种策略的正/负与精确中点；奇数分母非中点；
2. scale 不一致的加减、数值 `Eq`/`Ord`/`Hash`、proptest 交换/结合/单位元；
3. `i128` 对齐/加减乘 scale 溢出 → `Err`；禁止 `wrapping_mul`；
4. 除法 scale 合同（含 `1.00/0.50`）、除零；
5. serde 结构字段 round-trip（**非**跨版本 wire 兼容政策——后者仍开放）；
6. proptest 无 panic（checked 路径）与值相等不变量。

仍建议补强（非本轮阻塞）：property 级舍入表生成、最大 scale 策略、canonical 编码往返。

聚焦命令：

```text
cargo test -p xhyper-decimalx
cargo check -p xhyper-decimalx --all-targets
cargo clippy -p xhyper-decimalx --all-targets -- -D warnings
cargo fmt -- --check
cargo xtl lint-deps
```

最终仍须通过 XLib spec §8 与 Constitution §4.5 的全仓库 build/test/clippy/fmt、
`node scripts/check.mjs` 和 `cargo deny check`。

## 7. 验收标准与开放决策

- [ ] 路径、Cargo 依赖和独立 patch-only 版本规则符合 §2。
- [ ] 公开 API/rustdoc 与 §3 的实际导出一致，未重复定义 `Money`/`Decimal`。
- [ ] ADR-006 的 checked 算术、对齐、显式舍入和 rescale 合同已实现或经 RFC 正式修订。
- [ ] 所有溢出和舍入边界有确定、无静默回绕的测试。
- [ ] `Currency` 的安全构造边界明确；若保留公开字段，文档不得声称类型始终有效。
- [ ] serde 格式兼容范围获批并测试。
- [ ] §6 的聚焦与仓库门禁通过。

实现/发布前仍需裁定：溢出错误映射；最大合法 scale；除法目标 scale；是否封闭公开字段；
规范文本/二进制编码；serde 兼容承诺。implementation-plan 中超出当前源码与 Approved ADR 的草图均为
**Proposed**，不得作为已实现事实。

## 8. 可追溯性

| 合同 | 来源 |
|---|---|
| `/types/` 层、唯一定义点、依赖方向 | XLib spec §§1–4.2；ADR-007 |
| 十进制表示、显式舍入、对齐、边界测试 | XLib spec §§4.2、6；ADR-006 |
| 当前 API、错误与 serde | `crates/types/decimal/{Cargo.toml,src/lib.rs}` |
| 依赖治理、测试门禁 | Constitution Articles IV、VII、VIII |
| 版本与 RFC | Constitution §7.3；XLib spec §§5、9 |
