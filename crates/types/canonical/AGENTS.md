# AGENTS — crates/types/canonical

> 父级规则：[`crates/AGENTS.md`](../../AGENTS.md)

- Package：`canonical` · lib：`canonical` · path：`crates/types/canonical` · version：`0.1.0`
- 生产层级：**L2 committed wire subset**（v1 / v1.1 / v1.2 / v1.3）
- 定位：跨层共享**纯 DTO**（ADR-001）；无业务方法、无 I/O、无 codec
- 依赖：仅 `decimalx` + `serde`；禁止 contracts/domain/adapter 反向依赖
- `Money` 复用 `decimalx::Money`（ADR-007）；禁止金融 `f32`/`f64` 字段
- `OrderId` 类型已删；id 字段为 wire `String`；新接口优先 `OrderRef`
- DTO `ts: i64` = Unix epoch **纳秒**（CAN-TIME-001）；adapter 入口用 `ns_from_unix_millis`
- 形状检查：`shape::*`（adapter 入口，非 domain 校验）
- Active SSOT：`.agents/ssot/types/canonical/spec/spec.md`
- Wire 等级：`.agents/ssot/types/canonical/plan/wire-commitment-matrix.md`
- Residual OPEN/HUMAN/DEFER：`.agents/ssot/types/canonical/plan/residual-open.md`
- 门禁：`cargo test -p canonical` · `node scripts/quality-gates/check-canonical-align.mjs`
- 示例：`cargo run -p canonical --example basic`

## 目录结构

```text
crates/types/canonical/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── shape.rs
│   ├── proposed_time.rs
│   └── wire.rs
├── examples/
│   └── basic.rs          # 可运行最小消费者
├── docs/
│   ├── API.md            # 完整公开消费面
│   └── README.md
├── tests/
│   ├── public_api.rs
│   └── public_api_surface.rs
├── benches/
│   └── hot_path.rs
├── CHANGELOG.md
├── AGENTS.md
└── README.md
```

## 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.1.0 | 2026-07-21 | package 名对齐 `canonical`；examples/docs/tests 落地 |
| v1.0.1 | 2026-07-21 | 对齐子模块标准布局 |
| v1.0.0 | 2026-07-21 | 初始规则 |
