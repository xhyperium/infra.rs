# AGENTS — crates/types/canonical

> 父级规则：[`crates/AGENTS.md`](../../AGENTS.md)

- Package / lib / version：`canonical` / `canonical` / `0.1.2`；Cargo 选择器 `-p canonical`
- 生产层级：**L2 committed wire subset**（v1 / v1.1 / v1.2 / v1.3）
- 承诺边界：strict serde JSON DTO shape；**不是** canonical bytes、通用 codec、跨语言协议或 package stable
- 定位：跨层共享纯 DTO（ADR-001）；无业务状态机、I/O、授权、风控、重试或审计
- 依赖：仅 `decimalx` + `serde`；禁止 contracts/domain/adapter/service/kernel/evidence 反向依赖
- `Money` 复用 `decimalx::Money`（ADR-007）；禁止金融 `f32` / `f64` 字段
- `OrderId` 类型已删；id 字段为 wire `String`；新接口优先 `OrderRef`
- DTO `ts: i64` = Unix epoch 纳秒；ms→ns 用 checked helper；ns→ms 必须区分 exact 与向 0 截断
- Wire：精确查询用 `WireVersion` / `committed_wire_version`；旧 `WireCommitment` / `wire_commitment` 保持 coarse 兼容
- Envelope：仅 `schema_version` + `payload` 运输包装；反序列化后由调用者显式 validate，禁止宣称自动路由
- 形状检查：`shape::*` 供 adapter 入口使用，不是 domain validation
- Active SSOT：`.agents/ssot/types/canonical/spec/spec.md`
- Wire 清单：`.agents/ssot/types/canonical/plan/wire-commitment-matrix.md`
- Residual：`.agents/ssot/types/canonical/plan/residual-open.md`
- 门禁：`cargo test -p canonical -p decimalx` · `node scripts/quality-gates/check-canonical-align.mjs`
- 示例：`cargo run -p canonical --example basic`
- 版本纪律：本轮已从 `0.1.1` PATCH bump 到 `0.1.2`；不得再次 bump

## Committed inventory

| Wire | 类型 |
|---|---|
| v1 | `CancelOrderRequest` · `OrderRef` · `OrderAck` · `OrderStatus` · `Side` |
| v1.1 | `Order` |
| v1.2 | `Tick` · `Trade` |
| v1.3 | `Position` · `OrderBookSnapshot` · `PriceLevel` · `SymbolMeta` |

所有 committed 类型维持 `deny_unknown_fields`、必填字段、未知 variant 拒绝、文件或 inline golden 与非法 decimal scale 拒绝证据；有登记的 legacy/N-1 向量必须保持可读。`Money` 的 wire SSOT 在 `decimalx`，`Envelope<T>` 不属于上述 DTO 版本清单。

## 目录结构

```text
crates/types/canonical/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── envelope.rs
│   ├── proposed_time.rs
│   ├── shape.rs
│   └── wire.rs
├── examples/basic.rs
├── docs/
├── tests/
├── benches/hot_path.rs
├── CHANGELOG.md
├── AGENTS.md
└── README.md
```

## 文档版本

| 版本 | 日期 | 修订 |
|---|---|---|
| v1.2.0 | 2026-07-23 | 收敛 v1–v1.3 committed 清单、strict JSON、Envelope 与精确版本查询边界 |
| v1.1.0 | 2026-07-21 | package 名对齐 `canonical`；examples/docs/tests 落地 |
| v1.0.1 | 2026-07-21 | 对齐子模块标准布局 |
| v1.0.0 | 2026-07-21 | 初始规则 |
