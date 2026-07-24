<!-- ssot:clause=ecb.clause.001 -->
<!-- ssot:trace=ecb.spec.001 -->
<!-- ssot:spec-profile=provider.offline.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion -->
<!-- ssot:req=ecb.req.identity.001,ecb.req.temporal.001,ecb.req.values.001,ecb.req.failures.001,ecb.req.compatibility.001,ecb.req.evidence.001 -->
<!-- ssot:ac=ecb.ac.identity.001,ecb.ac.temporal.001,ecb.ac.values.001,ecb.ac.failures.001,ecb.ac.compatibility.001,ecb.ac.evidence.001 -->
# ecb — SDMX 离线 provider 规格草案

> 状态：`draft`/`not_started`。ECB 服务、dataflow、DSD、指标映射、许可和访问合同尚未核验；本文件不代表 API 已可用。

## 1. 范围与非目标

只定义脱敏 SDMX 元数据、维度字典和观测 fixture 的解析与 `domain_macro` 映射。不猜测数据集代码、维度顺序、端点、认证、缓存或再分发；不在本域复制核心类型实现。

## 2. 身份与时间语义

`source_series_id = dataflow + DSD + 规范化维度键`；完整身份追加 `indicator + subject + period + vintage`，其中 `indicator` 必须是 `domain_macro` 的规范 ID 或显式拒绝映射。维度顺序和代码表必须来自同一元数据快照，不能手填。`period` 是业务期间，`publication_time` 是 UTC instant，`vintage` 是可见版本；相同身份重复输入返回冲突错误。

## 3. 数值、单位与缺失

保留原始精度、单位、缩放、季调/非季调标记、观测状态和国家/地区聚合口径。百分比、金额、指数和比率不能混用；拒绝非有限数、未知单位、非法缩放、维度缺值和错误时区。缺失值保持来源状态与原因，不转零。

## 4. Fixture 输入输出与确定性

元数据 fixture 与观测 fixture 必须成对、严格 UTF-8、固定哈希并按维度键和期间稳定排序。解析先校验 dataflow/DSD/代码表，再解析观测；任何未知维度、错位列、重复键或 hash mismatch 都不得产生部分输出。相同输入必须得到相同规范化结果。

## 5. 错误、schema 演进与原子性

稳定错误至少包括 `metadata_mismatch`、`unknown_dimension`、`invalid_period`、`invalid_value`、`unknown_unit`、`duplicate_identity`、`schema_mismatch` 和 `fixture_hash_mismatch`。非 2xx 只是未来已获批 provider 的边界，不在当前离线 spec 伪造数字。维度增加可通过版本迁移兼容，语义变化必须提供 N-1 fixture。

## 6. 安全与来源契约

日志只保留脱敏 endpoint/数据集身份，不输出认证材料、完整查询 secret 或原始敏感字段。晋级前必须逐项记录官方 URL、HTTP 方法、请求参数、响应媒体类型、dataflow/DSD、文档版本、访问日期、许可/再分发边界和 metadata/observation fixture SHA-256；未知事实保持 `UNKNOWN`。

## 7. 验收、证据与晋级

离线验收覆盖元数据/观测双解析、维度顺序、单位缩放、季调标记、期间和 vintage、缺失、重复、坏数值、未知维度、schema/N-1 迁移、脱敏和确定性。证据必须绑定 fixture、命令、退出码、reviewer、许可证据和 commit；全部完成并获人工审查后才可晋级 `verified`。

## 8. 回滚与运行边界

数据集映射或代码表变更保留旧 metadata/observation fixture 与版本号；回滚后必须恢复旧身份映射并重跑离线门禁。未获批前不创建网络客户端、代理、重试、缓存或真实 E2E。
