<!-- ssot:clause=japan_cb.clause.001 -->
<!-- ssot:trace=japan_cb.spec.001 -->
<!-- ssot:spec-profile=provider.offline.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion -->
<!-- ssot:req=japan_cb.req.identity.001,japan_cb.req.temporal.001,japan_cb.req.values.001,japan_cb.req.failures.001,japan_cb.req.compatibility.001,japan_cb.req.evidence.001 -->
<!-- ssot:ac=japan_cb.ac.identity.001,japan_cb.ac.temporal.001,japan_cb.ac.values.001,japan_cb.ac.failures.001,japan_cb.ac.compatibility.001,japan_cb.ac.evidence.001 -->
# japan_cb — SDMX/CSV 离线 provider 规格草案

> 状态：`draft`/`not_started`。统计代码、来源地址、协议、授权、缓存和再分发许可均为 `UNKNOWN`；本文件不得指导联网客户端。

## 1. 范围与非目标

只定义脱敏 SDMX/CSV fixture 的系列身份、语言、期间、单位、修订和缺失语义。当前不定义下载入口、客户端依赖、User-Agent、重试、限流、缓存或真实服务验收。

## 2. 身份与时间语义

原始 `stat_code + dataset + dimension_key + language` 保存为 `source_series_id`，完整身份追加 `indicator`、主体、`period` 和 `vintage`；只有字段级映射完整时才能进入 `domain_macro`。`period` 是业务期间，发布批次使用 UTC instant，修订版本不得覆盖旧事实。未知维度、重复身份、乱序期间和时间冲突必须稳定失败。

## 3. 数值、单位与缺失

每条观测必须声明单位、精度、缩放和语言/地区口径；金额、指数、比例不得共享无单位值。拒绝坏数值、NaN、∞、单位缺失和非法缩放。来源缺失标记必须映射为明确 `missing_reason`，不得转零或丢行。

## 4. Fixture 输入输出与确定性

输入 fixture 为严格 UTF-8、固定字段顺序和脱敏 JSON/CSV；解析应规范化列名、期间和语言后按身份稳定排序。重复解析必须确定性，未知列保留或显式报错，任何一条坏记录都不得留下部分成功结果。

## 5. 错误、schema 演进与原子性

稳定错误至少区分 `invalid_series_id`、`invalid_period`、`invalid_value`、`unknown_unit`、`duplicate_identity`、`order_conflict`、`schema_mismatch` 和 `fixture_hash_mismatch`。列顺序/字段语义改变必须版本化；新增可选列必须通过 N-1 fixture 验证，删除必需列整体拒绝。

## 6. 安全与来源契约

fixture、Debug、Display、错误和 tracing 不得包含认证材料、完整 URL、原始响应敏感字段或凭据。晋级前必须补齐来源机构、文档版本、数据集/代码表、访问日期、授权、分页、限流、修订、缓存和许可证据；未核验事实保持 `UNKNOWN`。

## 7. 验收、证据与晋级

离线验收覆盖 SDMX/CSV 双格式、语言与维度、期间边界、单位/精度、修订、缺失、重复、乱序、未知列、坏数值、schema 迁移、脱敏和稳定排序。每项绑定 fixture SHA-256、命令、退出码、reviewer 和 commit-matched evidence；全部获批后才可另提 Cargo member。

## 8. 回滚与运行边界

代码表或映射变更保留旧版本 fixture 和回滚目标；回滚后重新验证身份、期间和单位结果。未获批前不运行联网、缓存、重试或凭据测试。
