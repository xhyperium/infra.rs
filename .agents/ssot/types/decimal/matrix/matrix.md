# decimalx 当前态追溯矩阵

> **状态**：Active current-state matrix · 非 Goal Achieved / Spec Approved

## 合同到实现与测试

| 合同 | Active spec | 实现位置 | 主要测试位置 | 当前判定 |
|---|---|---|---|---|
| 私有 Decimal 字段 + 访问器 | §2–3 | `src/lib.rs::Decimal` | `tests/public_api_surface.rs` | 已实现 |
| `MAX_SCALE = 18` | §2 | `MAX_SCALE`、`try_new`、Deserialize | `boundary_matrix.rs`、`adversarial_serde.rs` | 已实现 |
| 私有 Currency/Money 字段 | §2 | `Currency`、`Money` | `public_api_surface.rs`、`boundary_matrix.rs` | 已实现 |
| DecimalError / Kind | §6 | `DecimalError`、`DecimalErrorKind` | `public_api_surface.rs` | 已实现 |
| checked 四则 / rescale | §3–4 | `checked_*`、`div` | `entry_checked_ops.rs`、`boundary_matrix.rs`、`oracle_diff.rs` | 已实现 |
| default-off panicking ops | §3 | Cargo feature + cfg impl | `public_api_surface.rs` + 生产路径脚本 | 已实现 |
| Display → FromStr 全表示往返 | §5 | `Display`、`FromStr` | `boundary_matrix.rs` + property 测试 | `f26e29c` PASS / REVIEW PENDING |
| DecimalError → XError source | §6 | `From<DecimalError> for XError` | `boundary_matrix.rs`（含类型 downcast） | `f26e29c` PASS / REVIEW PENDING |
| 内部 Rust serde JSON v1 | §7 | `WIRE_SCHEMA_VERSION` + 自定义 serde | unit wire tests、`adversarial_serde.rs` | 有限承诺 |
| JSON i128 跨语言精确承载 | §7 | 不在当前实现范围 | 尚无协议一致性套件 | residual |

以上实现路径均相对 `crates/types/decimal/`。

## 权威与来源

| 事实 | 当前权威 | 说明 |
|---|---|---|
| 架构、错误链、语言、门禁 | `CONSTITUTION.md` + `docs/constitution/` | 本仓宪章正文 |
| decimalx 当前合同 | `spec/spec.md` | Active current-state SSOT |
| API 与行为事实 | `crates/types/decimal/src/lib.rs`、`Cargo.toml` | 必须由测试证明 |
| 独立 crate 版本 | `docs/governance/VERSIONING.md` + crate Cargo | 行为交付 PATCH +1 |
| “ADR-006/007” | 历史来源记录 | 本仓无可解析原件；不作当前权威 |

## 声明边界

| 能力 | 可声明 | 不可声明 |
|---|---|---|
| 生产就绪 | L1 checked path | 整个金融领域模型 / package stable |
| serde v1 | 内部 Rust JSON shape + 校验 | 跨语言精确协议 / canonical wire |
| panic | default-off ops 与便利 API 的明确 panic | checked 资金路径可 panic |
| 完成状态 | `f26e29c` 全量门禁 PASS；独立终审待裁决 | Goal Achieved / package stable |

开放项只在 [residual-open.md](../plan/residual-open.md) 维护；测试与门禁分别见
[test.md](../test/test.md) 和 [gate.md](../gate/gate.md)。
