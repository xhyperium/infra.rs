<!-- ssot:clause=fred.clause.001 -->
<!-- ssot:trace=fred.spec.001 -->
<!-- ssot:spec-profile=provider.offline.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion -->
<!-- ssot:req=fred.req.identity.001,fred.req.temporal.001,fred.req.values.001,fred.req.failures.001,fred.req.compatibility.001,fred.req.evidence.001 -->
<!-- ssot:ac=fred.ac.identity.001,fred.ac.temporal.001,fred.ac.values.001,fred.ac.failures.001,fred.ac.compatibility.001,fred.ac.evidence.001 -->
# fred — FRED 时间序列 provider 规格草案

> 状态：`draft`/`not_started`。端点、参数、配额、许可、修订语义和访问权限必须在实现前逐项绑定官方或合同来源；当前只允许离线 fixture。

## 1. 范围与非目标

本域定义美国宏观时间序列的脱敏解析和 `domain_macro` 映射，不宣称 workspace 有 FRED provider。网络 I/O、认证材料、缓存、调度和真实账户不进入当前实现边界。

## 2. 身份与时间语义

`FredSeriesId` 保留来源序列键；完整身份至少为 `source + series_id + indicator + subject + observation_period + vintage`，只有可证明无损的数值观测才能映射到 `domain_macro`。观测期间不是发布时间，`publication_time` 使用 UTC instant，realtime/vintage 窗口必须满足起点不晚于终点。重复身份必须明确拒绝或幂等，修订为追加版本。

## 3. 数值、单位与缺失

`FredValue` 区分 `Numeric`、`Integer`、`Text` 和 `Missing`，并携带单位、缩放、精度和调整口径。拒绝 NaN、∞、坏数值、未知单位和精度溢出；解析失败不得静默转零。来源缺失标记映射为 `missing_reason`，不丢失原始状态。

## 4. Fixture 输入输出与确定性

fixture 必须脱敏、固定 UTF-8、绑定 SHA-256、来源和访问日期；解析校验序列身份、期间、值、缺失和排序。输出按身份稳定排序，未知字段保留或显式报错；重复运行必须产生相同结果，失败不得产生部分结果。

## 5. 错误、schema 演进与原子性

`FredError` 至少区分 `invalid_series_id`、`invalid_period`、`invalid_value`、`unknown_unit`、`duplicate_identity`、`vintage_conflict`、`schema_mismatch` 和 `fixture_hash_mismatch`，只暴露有限字段路径。schema 新增字段可兼容读取，删除或语义变化必须版本化并用 N-1 fixture 验证。

## 6. 安全与来源契约

认证材料只能由运行时 secret 注入对象承载，禁止进入 SSOT、Debug、Serialize、URL、日志、错误或 tracing。端点、参数、配额、许可、访问日期、修订和再分发语义未核验前均为 `UNKNOWN`，不得写具体访问合同。

## 7. 验收、证据与晋级

离线验收覆盖正常值、缺失、日期边界、vintage、重复、坏数值、未知字段、单位、排序、错误脱敏和 schema/N-1 迁移。每项绑定 fixture SHA-256、命令、退出码、reviewer、许可审查和 commit-matched evidence；来源与 provider 根路径获批后才能晋级。

## 8. 回滚与运行边界

序列映射或修订逻辑变更保留旧 fixture、schema 和回滚目标，回滚后重跑离线门禁。当前不执行真实网络、读取 token、自动重试、缓存或限流改变。
