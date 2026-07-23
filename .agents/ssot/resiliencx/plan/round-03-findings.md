# resiliencx 第 3 轮候选准备记录

| 字段 | 值 |
|---|---|
| 日期 | 2026-07-23 |
| Beads | `infra-2d9.9` |
| 战役历史起点 | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| 最终 Review base | `origin/main@630f03d5db5739a89933fe921d7615841fde3789`（rebase 后固定基线） |
| 当前版本 | `0.1.2`；同一 PR 已从 0.1.1 bump 一次，按 R-C2 不重复 bump |
| 候选状态 | 治理修正后候选已重冻；本地 reviewer 完成，verifier 技术/证据初验完成 |

## 已闭合实现事实

- 第 2 轮已闭合 `RetrySafety`、整次 deadline、调用方 seeded jitter、budget reservation / RAII
  refund、attempt 观测一致性与 bulkhead poison 容量恢复。
- 第 2 轮独立代码/规格复审结论为通过；第 3 轮本地独立 reviewer 已完成实现/证据审查，独立
  verifier 已完成技术/证据初验。
- deadline 只提供 cooperative cancellation，不撤销已发生的外部副作用；预算、熔断、限流与舱壁
  均为单进程原语，不是分布式重试或协调平台。

## 第 3 轮机器证据

| 检查 | 结果 | 证据边界 |
|---|---|---|
| 当前 Round 3 三包 all-features/all-targets test | 退出码 0 | resiliencx 84 passed；postgresx 52 passed + 6 ignored；redisx 51 passed + 8 ignored |
| 当前 Round 3 三包 clippy / doc / fmt | 均退出码 0 | `-D warnings`；rustdoc `--no-deps`；三包 scoped fmt |
| active / complete spec | `cmp` 退出码 0 | 两份 current-state spec 一致 |
| workspace dependency / crate version gate | 均退出码 0 | 仅验证当前 manifest 合法，不替代本轮所需 PATCH bump |
| 相对 `e0dacd95c68a09d464dda97ed1e51e129c26a3cc` diff check | 退出码 0 | 限本任务 allowed paths |
| 当前 Round 3 首次行覆盖率 | `1106 / 1116`，99.1039%，失败 | 缺失 10 行；随后补真实行为测试 |
| 修复后 root 串行覆盖率 | `1156 / 1156`，zeros 0，100.0000%，退出码 0 | 共享工作树本地机器证据，不是固定 commit CI artifact |
| 最新固定 review 修复后覆盖率 | `1208 / 1208`，zeros 0，100.0000%，退出码 0 | root 串行最终结果；仍不是固定 commit CI artifact |

Round 1 的 `0.1.1` 未 bump 记录是当时执行者范围；root 已在发布准备阶段将当前版本更新为 `0.1.2`。

## 审查结论与外部待办

- Done（本地）：治理修正后候选已重冻；独立 reviewer 已完成实现/证据审查；独立 verifier 已完成
  技术/证据 AC 初验。本次纯状态 delta 不改变受审源码/测试。
- Pending（GitHub）：固定提交 CI artifact、PR、维护者审批与合并。
- Pending（发布）：合并后再判断 tag 或其他发布动作。

release 继续 BLOCKED；本记录不宣称分布式能力、Production Ready、发布批准或 package stable。

## Round 3 reviewer 阻断与当前修复候选

历史 Round 3 reviewer 在 `e0dacd9` 固定树指出：generic Adapter budget 入口与 Redis/Postgres 消费路径
仍可绕过 `RetrySafety`，且文档把未带 safety 的入口过度描述为安全。后续候选完成以下修复；重冻候选
现已完成本地 reviewer 审查与 verifier 技术/证据初验：

- resiliencx 新增 generic sync/async safe Adapter budget 入口，首次闭包/future 前校验 safety；
- 完整列举 unchecked compatibility API，不删除旧入口；
- Redis 真实 client budget 路径按操作语义迁移到 safe wrapper；
- Postgres 只新增显式 safety wrapper，不虚构 Pool budget 字段；0.3.2 错误发布叙述已更正；
- 相关用户可见英文错误中文化；
- `matrix/matrix.md` 从布局占位更新为本轮需求—实现—测试—文档追溯。

## 版本后续

版本裁定已完成：同一 PR 内 `resiliencx` 已从 `0.1.1` bump 至 `0.1.2`，按 R-C2 只 bump 一次；root
`postgresx` 与 `redisx` 的 `0.3.3` 已是 main 历史；当前 Cargo 均为 `0.3.4` 未发布候选，并已同步
path version。历史 `0.3.3` 发布记录不得被当前候选叙事覆盖。

## Coverage 失败与修复

root 首次串行门禁报告 `1106 / 1116`、缺失 10 行、99.1039%，缺口为 `budget.rs` 的零尝试校验、
async 最终错误返回，以及用于证明 unsafe 拒绝前不构造 operation 的 closure body。修复使用真实行为测试：

- `max_attempts == 0` 返回 `Invalid` 且 operation 调用数保持 0；
- async non-retryable 错误原样返回且不消费预算；
- sync/async 控制探针先执行一次覆盖其真实行为，清零后交给 safe API，断言 validation 未再次调用。

未使用 coverage 排除、不可达标注或空断言。root 修复后串行门禁结果为 instrumented `1156`、
hit `1156`、zeros `0`、`100.0000%`、退出码 `0`。

## 最新固定 review 修复

- RedisClient `with_retry_budget` 原样保存 0；GET/SET 路由测试证明 operation future 未构造且
  probe driver 调用数为 0；
- resiliencx 新增明确标注 unchecked compatibility 的 generic async budget core；safe 入口先校验
  safety 再委托，Postgres/Redis legacy async wrapper 也委托该 core；
- budget exhaustion 统一标准错误，`record_retry` 记录刚失败 attempt（从 1 起）；
- `retry_async` rustdoc 与 README 不再把 unchecked API 直接描述为生产路径/生产入口。

本节再次修改生产源码，故 `1156 / 1156` 是修复前基线；root 最终串行重跑为 `1208 / 1208`、
zeros 0、100.0000%、退出码 0。
