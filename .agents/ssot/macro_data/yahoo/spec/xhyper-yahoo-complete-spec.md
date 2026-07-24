<!-- ssot:clause=yahoo.clause.001 -->
<!-- ssot:trace=yahoo.spec.001 -->
<!-- ssot:spec-profile=provider.offline.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion -->
<!-- ssot:req=yahoo.req.identity.001,yahoo.req.temporal.001,yahoo.req.values.001,yahoo.req.failures.001,yahoo.req.compatibility.001,yahoo.req.evidence.001 -->
<!-- ssot:ac=yahoo.ac.identity.001,yahoo.ac.temporal.001,yahoo.ac.values.001,yahoo.ac.failures.001,yahoo.ac.compatibility.001,yahoo.ac.evidence.001 -->
# yahoo — 行情与历史柱离线 provider 规格草案

> 状态：`draft`/`not_started`。非官方接口、Cookie/crumb 流程、限流、自动访问许可、字段和 SLA 全部为 `UNKNOWN`。

## 1. 范围与非目标

只允许脱敏 fixture 的行情、历史柱、汇率和搜索响应解析。没有书面许可和可审计来源前，不实现自动登录、Cookie/crumb 获取、代理轮换、挑战规避、持久缓存、再分发或真实网络测试。

## 2. 身份与时间语义

`MarketIdentity = source + symbol + exchange + currency`；历史身份追加 `indicator? + period + interval + adjusted_flag + vintage`。只有明确是宏观时间序列或曲线且完成 source-to-kernel 映射时才能进入 `domain_macro`/`yield_curve`；普通行情、搜索和市场产品不自动进入本仓 kernel。交易时间、业务期间、发布时间和 vintage 分离，时区必须显式。重复身份、interval 冲突或 adjustment 口径冲突必须稳定失败，不能后到值覆盖先到值。

## 3. 数值、单位与缺失

价格、收益率、数量、汇率和调整因子必须携带单位、币种、精度、缩放和调整口径；拒绝非有限值、时间戳单位未知、精度溢出和不明缩放。缺失原因和原始状态保留，不转零或静默删除。

## 4. Fixture 输入输出与确定性

每个响应 fixture 必须脱敏、固定 UTF-8、绑定来源版本和 SHA-256；解析先校验 schema、字段类型、时间戳单位、symbol/exchange/currency，再按身份和时间稳定排序。未知字段/枚举必须保留或显式错误，同一输入必须确定性输出且失败不产生部分结果。

## 5. 错误、schema 演进与原子性

稳定错误至少区分 `invalid_market_identity`、`invalid_period`、`invalid_value`、`unknown_unit`、`duplicate_identity`、`timestamp_unit_unknown`、`schema_mismatch` 和 `fixture_hash_mismatch`。schema 哈希变化、调整因子缺失和字段语义改变必须阻断；新增字段通过版本迁移和 N-1 fixture 验证。

## 6. 安全与来源契约

secret、Cookie、完整 URL、原始响应和高基数 symbol 不进入错误、Debug、序列化或 tracing。每个未来响应必须绑定来源文档/许可、访问日期、方法与非敏感参数、媒体类型、schema 哈希、错误/限流行为和再分发边界；未核验事实保持 `UNKNOWN`。

## 7. 验收、证据与晋级

离线验收覆盖行情、历史柱、汇率、搜索、交易时区、interval、adjusted/unadjusted、单位、缺失、重复、schema 迁移、坏数值、未知字段、脱敏和确定性。fixture、命令、退出码、reviewer、人工合规、回滚测试和 commit-matched evidence 全部完成后才可重新评估 `spec_status`。

## 8. 回滚与运行边界

字段、调整口径或 interval 变更保留旧 schema、fixture 和回滚版本；回滚后重跑同一组解析和安全门禁。不可用或无许可不能被自动跳过后计为通过。
