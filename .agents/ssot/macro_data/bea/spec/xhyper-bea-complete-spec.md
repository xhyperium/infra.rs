<!-- ssot:clause=bea.clause.001 -->
<!-- ssot:trace=bea.spec.001 -->
<!-- ssot:spec-profile=provider.offline.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion -->
<!-- ssot:req=bea.req.identity.001,bea.req.temporal.001,bea.req.values.001,bea.req.failures.001,bea.req.compatibility.001,bea.req.evidence.001 -->
<!-- ssot:ac=bea.ac.identity.001,bea.ac.temporal.001,bea.ac.values.001,bea.ac.failures.001,bea.ac.compatibility.001,bea.ac.evidence.001 -->
# bea — 脱敏宏观记录 provider 规格草案

> 状态：`draft`/`not_started`。来源合同、数据集、字段、授权、配额、许可和再分发语义均为 `UNKNOWN`；本文件不得指导联网、凭据注入、缓存或再分发。

## 1. 范围与非目标

当前只定义待批准来源的脱敏 fixture 解析：记录来源身份、数据集身份、地理主体、业务期间、单位、修订和缺失原因。当前不定义外部端点、请求流程、认证材料、抓取调度或 provider crate。

## 2. 身份与时间语义

规范身份为 `source + dataset + table_or_series + measure + subject + period + vintage`；原始表号或行号必须保留为 `source_series_id`，不得直接充当本仓 `IndicatorId`。宏观观测候选只可显式映射到 `domain_macro`，曲线类观测需另经 `yield_curve` 路由批准，不能使用旧的 `MacroPoint` 名称。`period` 是业务期间，`publication_time` 是 UTC instant，`vintage` 是可见版本；缺少版本语义时必须显式使用 `None`。同一完整身份重复输入必须拒绝或明确幂等，禁止后到值覆盖先到值。

## 3. 数值、单位与缺失

每个数值必须带原始单位、缩放因子、精度和币种/计量口径；金额、比例、指数和增长率不得共享无单位的数值类型。拒绝 NaN、∞、非法缩放、负精度和期间外日期。缺失值必须携带 `missing_reason`，不得静默转零、空字符串或删除记录。

## 4. Fixture 输入输出与确定性

输入 fixture 必须是严格 UTF-8、固定字节和脱敏字段，输出按规范身份稳定排序；解析同一输入必须得到相同结果和错误顺序。未知字段保留或以显式 `unknown_field` 错误处理，不能静默丢弃。输出不得产生部分成功结果，fixture 哈希必须在 evidence 中绑定。

## 5. 错误、schema 演进与原子性

错误至少区分 `invalid_identity`、`invalid_period`、`invalid_value`、`unknown_unit`、`duplicate_identity`、`schema_mismatch` 和 `fixture_hash_mismatch`，并保留有限字段路径。坏 JSON/CSV、字段类型变化、修订断链和排序冲突均稳定失败。schema 增加字段必须兼容读取；删除或改变语义必须通过版本迁移和 N-1 fixture，禁止静默降级。

## 6. 安全与来源契约

认证材料只能由运行时 secret 引用承载，禁止进入 SSOT、fixture、Debug、Display、Serialize、错误、日志、tracing、URL 或原始响应。来源、版本、访问日期、字段字典、状态码、分页、限流、缓存和再分发许可未逐项核验前保持 `UNKNOWN`，不得写入具体合同数字。

## 7. 验收、证据与晋级

离线验收必须覆盖合法记录、缺字段、单位/缩放、期间边界、缺失原因、修订、重复身份、未知字段、坏数值、schema 迁移、脱敏和确定性排序；每项绑定 fixture SHA-256、命令、退出码、reviewer 与 commit-matched evidence。只有官方或合同来源、合规审查、provider 路径和回滚证据齐全后，才可从 `draft` 晋级 `verified`。

## 8. 回滚与运行边界

provider 变更必须保留前一版规范和 fixture 解析结果，回滚后重新运行相同离线门禁。未经批准不得启用网络、自动重试、缓存、限流绕过或再分发；失败时停止且不保留部分结果。
