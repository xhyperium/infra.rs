<!-- ssot:clause=yield_curve.clause.001 -->
<!-- ssot:trace=yield_curve.spec.001 -->
<!-- ssot:spec-profile=kernel.domain.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion|algorithm|compatibility -->
<!-- ssot:req=yield_curve.req.identity.001,yield_curve.req.temporal.001,yield_curve.req.values.001,yield_curve.req.failures.001,yield_curve.req.compatibility.001,yield_curve.req.evidence.001 -->
<!-- ssot:ac=yield_curve.ac.identity.001,yield_curve.ac.temporal.001,yield_curve.ac.values.001,yield_curve.ac.failures.001,yield_curve.ac.compatibility.001,yield_curve.ac.evidence.001 -->
# yield_curve — 来源无关 kernel 规格草案

> 状态：`draft`。kernel 只定义来源无关的期限、利率惯例、曲线和值对象；provider 的 URL、字段、限流、授权和许可由对应 provider 域另行核验。该规格不宣称任何来源或算法已实现。

## 1. 范围与边界

输入是已由 provider 脱敏、校验并绑定来源身份的曲线点；输出是可比较、可审计的 canonical curve 或明确失败。kernel 不拥有 HTTP、认证、来源字段、缓存或再分发权利。财政收益率曲线可路由到本域，普通宏观观测路由到 `domain_macro`，路由不由数据值猜测。

## 2. 身份、期间与市场惯例

完整身份为 `source + series_id + currency + valuation_date + settlement_date + maturity + curve_kind + vintage + convention`。`valuation_date` 是业务估值日，`publication_time` 是 UTC instant，`vintage` 是来源可见版本；日期、结算日、节假日日历和时区必须显式。币种、day-count、复利频率、支付频率和报价口径是身份的一部分，不能把不同惯例的同期限值合并。

`maturity` 是值对象，不接受自由文本。标准化时保留原始 `(value, unit)` 和 canonical tenor key；`12M` 与 `1Y` 只有在同一 `TenorConvention` 明确允许且日历/计息规则一致时才视为相等，不能通过字符串或浮点近似自动合并。任一身份字段不同都不得碰撞。

## 3. 数值、单位与缺失

收益率必须明确是百分点还是小数，禁止隐式乘除 100；保存原始十进制精度、舍入模式、报价单位、币种和利率类型（par/zero/discount）。拒绝 NaN、∞、非法负期限、未知单位、精度溢出和不一致的 convention。缺失期限、节假日无报价或来源抑制必须带 `missing_reason`，不得转零或伪造曲线点。

## 4. 曲线聚合与确定性

一条 curve 必须声明 `curve_id`、valuation/settlement date、currency、convention、`curve_kind`、点集合和 source vintage；点按 canonical tenor key 严格排序且不得重复。缺少 required point、跨币种混合、重复 identity、期间不一致或 hash 不匹配时整条批次原子失败。相同 fixture 和算法版本必须得到相同点序列与结果。

## 5. 派生算法与 schema 兼容

官方发布点使用来源身份；插值、拟合和外推结果使用新的 `derived` identity，并记录算法版本、输入点 identity 集合、边界策略、舍入模式和 `interpolated/extrapolated` 标记。算法必须明确单调性、无套利约束（若适用）、超出边界的失败条件和误差界；不能把派生值覆盖官方值。schema 增加字段需有默认语义，语义变化、tenor/convention 改变必须版本化并保留 N-1 fixture。

## 6. 错误与安全

稳定错误至少区分 `invalid_tenor`、`tenor_convention_mismatch`、`invalid_currency`、`invalid_date`、`invalid_rate`、`unknown_unit`、`duplicate_point`、`curve_incomplete`、`convention_mismatch`、`algorithm_out_of_bounds`、`schema_mismatch` 和 `fixture_hash_mismatch`。错误不得包含 secret、完整 URL、原始响应或无界输入；失败不得产生部分曲线或把算法失败转为 Missing observation。

## 7. 证据、验收与晋级

离线验收覆盖 tenor 规范化（包括 12M/1Y 规则）、币种、day-count/compounding、valuation/settlement、点排序、重复、缺失、单位口径、官方与 derived 分离、插值/外推边界、算法版本、schema/N-1、坏数值、脱敏和回滚。fixture SHA-256、命令、退出码、reviewer、输入点集合和 commit-matched evidence 全部齐全后，才可将 kernel 规格晋级 `verified`。

## 8. 回滚与 provider 路由

provider 只负责来源 wire、许可和脱敏 fixture，并显式映射到本 kernel；kernel 不反向推断来源事实。算法或 convention 变更必须保留旧版本、旧 fixture 和回滚目标，支持停止摄取、隔离批次、重放输入和比较恢复结果。当前没有可发布实现，provider 的授权和许可不在本 kernel evidence 中声称。
