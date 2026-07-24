<!-- ssot:clause=uk_cb.clause.001 -->
<!-- ssot:trace=uk_cb.spec.001 -->
<!-- ssot:spec-profile=provider.offline.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion -->
<!-- ssot:req=uk_cb.req.identity.001,uk_cb.req.temporal.001,uk_cb.req.values.001,uk_cb.req.failures.001,uk_cb.req.compatibility.001,uk_cb.req.evidence.001 -->
<!-- ssot:ac=uk_cb.ac.identity.001,uk_cb.ac.temporal.001,uk_cb.ac.values.001,uk_cb.ac.failures.001,uk_cb.ac.compatibility.001,uk_cb.ac.evidence.001 -->
# uk_cb — 时间序列离线 provider 规格草案

> 状态：`draft`/`not_started`。来源、Series ID、端点、User-Agent、许可、限流、重试和缓存语义均为 `UNKNOWN`；本文件不得指导联网采集。

## 1. 范围与非目标

只定义脱敏 JSON/CSV fixture 的系列身份、期间、单位、精度、修订、发布批次和缺失语义。当前不定义请求方法、客户端依赖、User-Agent 内容、重试次数、请求速率或实时任务。

## 2. 身份与时间语义

`source_series_id` 保留原始系列键，完整身份为 `source + series_id + indicator + subject + period + vintage`；只有可审计的字段映射才能进入 `domain_macro`，利率曲线需另行评估 `yield_curve` 路由。`period` 是业务期间，发布批次和 `publication_time` 使用 UTC instant，修订不覆盖既有事实。重复身份、时间冲突、乱序或不完整期间返回稳定错误。

## 3. 数值、单位与缺失

每个值必须带单位、精度、缩放、币种/地区和调整口径；百分点、比例、指数和金额不可无标签混用。拒绝 NaN、∞、坏数值、未知单位和非法缩放。缺失标记映射为明确 `missing_reason`，不转零、不删除观测。

## 4. Fixture 输入输出与确定性

fixture 必须严格 UTF-8、脱敏、固定哈希；解析校验字段、系列、期间、单位和排序，输出按身份稳定排序。未知字段保留或显式错误；同一输入结果和错误顺序必须确定，失败不得产生部分输出。

## 5. 错误、schema 演进与原子性

稳定错误至少区分 `invalid_series_id`、`invalid_period`、`invalid_value`、`unknown_unit`、`duplicate_identity`、`publication_conflict`、`schema_mismatch` 和 `fixture_hash_mismatch`。schema 新增可选字段需 N-1 fixture，删除必需字段或改变语义必须版本化并整体拒绝。

## 6. 安全与来源契约

secret、完整 URL、原始响应、凭据和高基数输入不得进入 SSOT、fixture、Debug、错误或 tracing。晋级前必须逐项绑定来源文档/合同、版本、访问日期、字段字典、认证、状态码、分页、限流、缓存和再分发许可；未知事实保持 `UNKNOWN`。

## 7. 验收、证据与晋级

离线验收覆盖合法序列、单位/精度、期间和发布批次、修订、缺失、重复、乱序、坏数值、未知字段、schema 迁移、脱敏和确定性。每项绑定 fixture SHA-256、命令、退出码、reviewer、回滚目标和 commit-matched evidence，才能另提获批 Cargo member。

## 8. 回滚与运行边界

系列映射或修订逻辑变更保留旧版本 fixture 和回滚目标；回滚后重新验证身份、期间和单位。未获批前不执行联网采集、缓存、重试或凭据测试。
