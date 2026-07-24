<!-- ssot:clause=eastmoney.clause.001 -->
<!-- ssot:trace=eastmoney.spec.001 -->
<!-- ssot:spec-profile=provider.offline.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion -->
<!-- ssot:req=eastmoney.req.identity.001,eastmoney.req.temporal.001,eastmoney.req.values.001,eastmoney.req.failures.001,eastmoney.req.compatibility.001,eastmoney.req.evidence.001 -->
<!-- ssot:ac=eastmoney.ac.identity.001,eastmoney.ac.temporal.001,eastmoney.ac.values.001,eastmoney.ac.failures.001,eastmoney.ac.compatibility.001,eastmoney.ac.evidence.001 -->
# eastmoney — 离线 provider 规格草案

> 状态：`draft`/`not_started`。东方财富端点、字段、授权、限流、许可和再分发边界均待核验；当前只允许脱敏 fixture，不形成访问合同。

## 1. 范围与非目标

覆盖行情、宏观观测、汇率和日历事件的脱敏解析与 `domain_macro` 映射。明确不实现联网采集、UA/Referer 伪装、IP/代理轮换、Cookie/Challenge 获取、验证码规避、无头浏览器或持久缓存。

## 2. 身份与时间语义

原始身份至少保留 `source_market + source_symbol_or_series + field + subject`，规范身份追加 `period + vintage`；原始代码不得直接当作 `IndicatorId`。宏观和汇率观测只有在字段级映射获批后才能进入 `domain_macro`；行情、搜索和日历产品不自动进入 kernel。`period` 表示业务期间，`publication_time` 使用 UTC instant，`vintage` 表示可见修订版本。重复完整身份必须返回 `duplicate_identity` 或定义幂等规则，禁止静默覆盖。

## 3. 数值、单位与缺失

价格、金额、指数、汇率、比例、成交量和日历数值分别携带单位、币种、缩放、精度与调整口径。拒绝 NaN、∞、非法缩放、时区不明和不支持的单位。缺失值保留缺失原因和原始标记，不转换为零或空字符串。

## 4. Fixture 输入输出与确定性

输入为严格 UTF-8 的脱敏 JSON/CSV fixture；必须校验字段类型、schema 版本、Content-Type 元数据（若 fixture 提供）、身份和排序。输出按身份稳定排序，未知字段必须保留或显式报错；同一 fixture 重复解析必须字节级确定，禁止部分结果和隐式网络补全。

## 5. 错误、schema 演进与原子性

稳定错误至少区分 `invalid_identity`、`invalid_period`、`invalid_value`、`unknown_unit`、`duplicate_identity`、`schema_mismatch`、`pagination_gap` 和 `fixture_hash_mismatch`。字段编号/含义变化、分页缺页、哈希不匹配和部分解析失败必须整体拒绝。新增字段可兼容读取，语义变化必须版本化并提供 N-1 fixture。

## 6. 安全与来源契约

secret、Cookie、完整 URL、原始响应、请求参数和高基数 symbol 不进入 Debug、序列化、错误或 tracing。每个未来 endpoint 必须单独绑定来源 URL、方法、参数、媒体类型、字段字典、错误体、分页、限流、缓存/再分发许可、访问日期和 fixture SHA-256；未核验项保持 `UNKNOWN`。

## 7. 验收、证据与晋级

离线验收覆盖合法行情/宏观/汇率/日历、单位缩放、期间和时区边界、缺失、重复、排序、未知字段、分页、坏数值、schema 迁移、脱敏和确定性。每项绑定 fixture、命令原文、退出码、reviewer、许可状态和 commit-matched evidence；只有书面授权和合规审查完成后才可晋级。

## 8. 回滚与运行边界

任何映射或解析变更必须保留上一版 schema、fixture 和回滚目标，并重新执行离线门禁。限流、重试和标准客户端只有在书面契约批准后才能另建任务；当前失败立即停止，不切换访问层级。
