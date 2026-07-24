<!-- ssot:clause=treasury.clause.001 -->
<!-- ssot:trace=treasury.spec.001 -->
<!-- ssot:spec-profile=provider.offline.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion -->
<!-- ssot:req=treasury.req.identity.001,treasury.req.temporal.001,treasury.req.values.001,treasury.req.failures.001,treasury.req.compatibility.001,treasury.req.evidence.001 -->
<!-- ssot:ac=treasury.ac.identity.001,treasury.ac.temporal.001,treasury.ac.values.001,treasury.ac.failures.001,treasury.ac.compatibility.001,treasury.ac.evidence.001 -->
# treasury — 财政记录离线 provider 规格草案

> 状态：`draft`/`not_started`。来源表、端点、参数、字段、许可、限流、缓存和修订语义均为 `UNKNOWN`；本文件不得指导联网客户端或凭据实现。

## 1. 范围与非目标

只定义脱敏财政记录 fixture 的表、记录身份、期间、金额、单位、精度、修订和缺失解析。当前不定义下载流程、请求速率、缓存 TTL、认证或 provider crate。

## 2. 身份与时间语义

完整身份为 `source_table + record_key + indicator + subject + period + vintage`，财政收入/支出/债务等宏观记录只有在字段级映射获批后进入 `domain_macro`；收益率、拍卖或期限点必须单独判断是否路由到 `yield_curve`，不得混用。财政年度/季度等原始期间必须保留并映射到规范 `period`。发布批次使用 UTC instant，修订为追加版本。重复身份、期间倒序、年度与季度冲突必须返回稳定错误。

## 3. 数值、单位与缺失

金额必须带币种、单位、缩放、精度和舍入模式；数量、比例和金额不可隐式互转。拒绝非有限值、坏小数、非法负值（若合同不允许）和精度溢出。财政缺失符号保留为 `missing_reason`，不得静默转零。

## 4. Fixture 输入输出与确定性

输入为严格 UTF-8 脱敏 JSON/CSV，必须校验表身份、字段类型、期间和排序；输出按完整身份稳定排序并保留来源字段。未知字段保留或显式错误，同一 fixture 重复解析必须相同，任何坏记录都不产生部分结果。

## 5. 错误、schema 演进与原子性

稳定错误至少区分 `invalid_record_key`、`invalid_period`、`invalid_value`、`unknown_unit`、`duplicate_identity`、`precision_overflow`、`schema_mismatch` 和 `fixture_hash_mismatch`。字段删除或语义改变整体拒绝；新增字段和版本迁移必须提供 N-1 fixture，禁止静默兼容错误语义。

## 6. 安全与来源契约

认证材料只能由运行时 secret 引用承载，禁止进入 SSOT、fixture、Debug、Serialize、错误、日志或 tracing。晋级前必须补齐官方文档/合同、版本、访问日期、字段字典、状态码、分页、限流、缓存和再分发许可；未核验数字保持 `UNKNOWN`。

## 7. 验收、证据与晋级

离线验收覆盖表/记录身份、财政期间、金额精度、币种/缩放、缺失、修订、重复、排序、坏数值、未知字段、schema/N-1、脱敏和确定性。命令、退出码、fixture SHA-256、reviewer、许可和 commit-matched evidence 必须齐全后才可另提 provider crate。

## 8. 回滚与运行边界

字段或单位变更保留旧 schema、fixture 和回滚目标；回滚后重跑所有离线门禁。当前不创建网络客户端、认证、缓存、限流、重试或真实 E2E。
