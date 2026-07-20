# AGENTS — crates/types/canonical

- Package：`xhyper-canonical` · lib：`canonical`
- 定位：跨层共享**纯 DTO**（ADR-001）；无业务方法、无 I/O、无 codec
- 依赖：仅 `xhyper-decimalx` + `serde`；禁止 contracts/domain/adapter 反向依赖
- `Money` 复用 `decimalx::Money`（ADR-007）；禁止金融 `f32`/`f64` 字段
- `OrderId` 类型已删；id 字段为 wire `String`；新接口优先 `OrderRef`
- DTO `ts: i64` = Unix epoch **纳秒**（CAN-TIME-001）；adapter 入口用 `ns_from_unix_millis`
- 形状检查：`shape::*`（adapter 入口，非 domain 校验）
- Active SSOT：`.agents/ssot/types/canonical/spec/spec.md`
- Wire 等级：`.agents/ssot/types/canonical/plan/wire-commitment-matrix.md`
- Residual OPEN/HUMAN/DEFER：`.agents/ssot/types/canonical/plan/residual-open.md`
- 门禁：`cargo test -p xhyper-canonical` · `node scripts/check-canonical-align.mjs`
