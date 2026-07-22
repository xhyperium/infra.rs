# decimalx 交付门禁

> **状态**：Active gate definition · R3 内容候选 `387a1dc` 的全仓/专项门禁已通过；独立终审待闭合
>
> 本文件定义通过条件，不替代独立 reviewer 的裁决。

## G1 — 范围与层级

- 实现路径唯一为 `crates/types/decimal`；生产依赖仅 `kernel` + `serde`。
- 禁止 `decimalx → canonical`，禁止 `f32` / `f64` 金额运算。
- 生产声明只能写 **L1 checked path**；出现跨语言 wire stable、package stable、crates.io
  Production Ready 即失败。

## G2 — 封装与构造

- `Decimal`、`Currency`、`Money` 字段私有，读取使用公开访问器。
- `try_new`、parse、serde 反序列化都强制 `MAX_SCALE = 18` / Currency 大写 ASCII 不变量。
- `Decimal::new` 的 panic 条件有 `# Panics`，并明确仅供 const/test/兼容便利。

## G3 — checked 生产路径

- add/sub/mul/div/rescale 的生产入口均为 fallible checked API。
- checked 路径覆盖 `i128::MIN/MAX`、scale `0/1/18`、对齐/中间值溢出与除零，输入不能触发 panic。
- `rescale` 和 `new` 不得出现在资金生产路径；`+/-/*` 只在 default-off `panicking-ops` 下存在。
- `node scripts/quality-gates/check-decimal-no-panicking-ops.mjs` 必须通过。

## G4 — 文本与错误

- 所有可表示 Decimal 满足 `Display -> FromStr` 数值往返；定向覆盖 `i128::MIN` 与
  scale `0/1/18`，并有 property 覆盖。
- `DecimalError` 全变体可映射到 `DecimalErrorKind`，用户可见 Display 为中文。
- `DecimalError -> XError` 保持 `ErrorKind::Invalid` 并保留 `source()` chain。

## G5 — serde v1

- Decimal/Currency/Money 的内部 Rust serde JSON shape 有 golden/round-trip 测试。
- 非法 scale、非法 Currency、未知 Decimal/Money 字段反序列化失败。
- 文档与评审明确 JSON `i128` 跨语言精度风险仍开放；测试不得越权证明精确跨语言协议。

## G6 — 版本与依赖

- 当前交付为 `decimalx 0.1.2`；相对 `0.1.1` 只执行一次 PATCH +1。
- path 依赖版本、lockfile 与 crate 版本由实现/发布任务同步。
- `check-crate-versions.mjs` 与 `check-workspace-deps.mjs` 通过。

## G7 — 质量与证据

至少保存以下命令的退出码与对应提交：

```bash
cargo test -p decimalx
cargo test -p decimalx --features panicking-ops
cargo clippy -p decimalx --all-targets --all-features -- -D warnings
cargo fmt --all --check
node scripts/quality-gates/check-decimal-no-panicking-ops.mjs
node scripts/quality-gates/check-crate-versions.mjs
node scripts/quality-gates/check-workspace-deps.mjs
```

全仓门禁按宪章执行。缺少证据、只报告“看起来通过”、或在本文件直接宣称 reviewer PASS，均视为
门禁未满足。
