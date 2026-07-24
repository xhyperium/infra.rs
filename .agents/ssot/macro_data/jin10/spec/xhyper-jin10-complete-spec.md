<!-- ssot:clause=jin10.clause.001 -->
<!-- ssot:trace=jin10.spec.001 -->
<!-- ssot:spec-profile=provider.offline.v1 -->
<!-- ssot:spec-contract=identity|temporal|units|missing|determinism|errors|security|evidence|promotion -->
<!-- ssot:req=jin10.req.identity.001,jin10.req.temporal.001,jin10.req.values.001,jin10.req.failures.001,jin10.req.compatibility.001,jin10.req.evidence.001 -->
<!-- ssot:ac=jin10.ac.identity.001,jin10.ac.temporal.001,jin10.ac.values.001,jin10.ac.failures.001,jin10.ac.compatibility.001,jin10.ac.evidence.001 -->
# jin10 — 事件与行情消息 provider 规格草案

> 状态：`draft`/`not_started`。REST、WSS、MCP、认证、推送协议、限流和再分发权利全部为 `UNKNOWN`，不得当作公开 API 合同。

## 1. 范围与非目标

定义事件、日历和行情消息的离线规范化边界；当前不猜测 endpoint、Header、Query、WebSocket 地址、心跳、重连、登录态、缓存或再分发权限。不实现真实网络客户端、代理切换、自动重试或凭据注入。

## 2. 身份与时间语义

`source_event_id` 是来源事件身份，完整身份追加 `kind + subject + business_period + vintage`；只有消息显式提供可验证 `indicator`、主体和业务期间时，才可提议映射到 `domain_macro`，Flash/Quote/Search 不自动成为宏观观测。`observed_at` 和 `publication_time` 使用 UTC instant，业务期间与接收时间分离。未知事件不得静默映射为已知类型；重复事件必须拒绝或明确幂等，乱序不能覆盖既有事实。

## 3. 数值、单位与缺失

事件 payload 中的价格、数量、比例和金额必须声明单位、缩放、币种、精度与时区；拒绝 NaN、∞、控制字符、过长字符串和非法缩放。缺失字段保留 `missing_reason`，未知值不转空字符串或零。

## 4. Fixture 输入输出与确定性

输入为脱敏、严格 UTF-8 的 JSON/CSV 消息 fixture；输出保留原始来源身份、发布时间、业务期间、语言、未知字段和规范化事件。相同 fixture 必须按身份/时间稳定排序并得到相同结果；任一坏消息导致该批次原子失败，不产生部分输出。

## 5. 错误、schema 演进与原子性

稳定错误至少区分 `invalid_event_id`、`unknown_kind`、`invalid_timestamp`、`invalid_value`、`duplicate_event`、`identity_conflict`、`schema_mismatch` 和 `fixture_hash_mismatch`。坏 JSON、字段类型变化、非有限值和时间冲突必须整体拒绝。未知字段策略必须版本化，N-1 fixture 必须继续可读。

## 6. 安全与来源契约

认证材料使用 secret newtype，禁止 Debug/Serialize/Error/tracing 输出；完整 URL、query secret、Cookie、原始响应和用户输入不得进入日志或错误。供应商文档/合同 URL、版本、访问日期、媒体类型、状态码、游标、重试/限流、断线、缓存和再分发许可缺一项即保持 `UNKNOWN`。

## 7. 验收、证据与晋级

离线验收覆盖合法/未知消息、缺失字段、重复、乱序、坏 JSON、非有限值、身份冲突、schema 迁移、单位、时区、脱敏和确定性。真实网络测试默认不执行；fixture SHA-256、命令、退出码、reviewer、合规和 commit-matched evidence 齐全后才可提议 provider。

## 8. 回滚与运行边界

事件类型或字段映射变更必须保留旧 schema、fixture 和回滚目标；回滚后重新运行离线 parser 和错误门禁。权限未知时拒绝真实请求，不改变访问层级。
