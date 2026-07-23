# resiliencx — Round 03 追溯矩阵

> 状态：候选已重冻；本地 reviewer 完成，verifier 技术/证据初验完成；GitHub 交付 pending。

| 需求 | 实现 | 测试 | 文档 | 当前状态 |
|---|---|---|---|---|
| Generic safety-aware sync Adapter budget | `call_with_retry_budget_safe` | `budget::tests::safe_adapter_sync_*` + public surface | active spec / API | 实现完成，本地复审通过 |
| Generic safety-aware async Adapter budget | `call_with_retry_budget_async_safe` | `budget::tests::safe_adapter_async_*` + public surface | active spec / API | 实现完成，本地复审通过 |
| 首次 operation 前拒绝 unsafe 多试 | safe generic validate 在闭包/future 前 | constructed/calls 保持 0 | spec §2/§3.1 | 实现完成，本地复审通过 |
| Unchecked compatibility 诚实清单 | legacy rustdoc 标注 | 旧合同测试保留 | spec / API / README | 文档完成，本地复审通过 |
| Redis 真实 client 迁移 | `client.rs` 仅调用 safe async wrapper | wrapper + 分类单测；live 仍 ignored | Redis README/usage/标准 | 实现完成，本地复审通过 |
| Postgres 诚实边界 | safe wrappers；Pool 无虚构 budget 接线 | safe wrapper 单测 | Postgres README/usage/0.3.4 release | 实现完成，本地复审通过 |
| 中文用户错误 | resiliencx + 相关 adapter 路径 | 定向测试 | CHANGELOG / findings | 实现完成，本地复审通过 |

## Redis 分类

| 操作 | RetrySafety | 理由 |
|---|---|---|
| GET / EXISTS / PTTL / MGET | `ReadOnly` | 不修改 Redis 状态 |
| 无 TTL SET / MSET | `Idempotent` | 同一参数重复执行得到同一存储状态 |
| DEL | `UnsafeSideEffect` | 重复执行返回值语义变化 |
| PEXPIRE / 相对 TTL SET | `UnsafeSideEffect` | 重复执行会延后绝对过期时刻 |

## 门禁状态

- 本地已通过：resiliencx / postgresx / redisx all-features tests、clippy、doc、fmt；
- 本地已通过：active / complete spec `cmp`、workspace dependency/version gates、base diff check；
- 版本已裁定：同一 PR 内 `resiliencx` 保持一次 bump 后的 `0.1.2`；两个 adapter 当前为 `0.3.4`，
  `0.3.3` 保留为 main 历史；
- Coverage：首次 `1106 / 1116`（99.1039%）失败；补行为测试后 root 串行复验
  `1156 / 1156`、zeros 0、100.0000%、退出码 0；该数字是最新固定 review 源码修复前基线，
  最终重跑为 `1208 / 1208`、zeros 0、100.0000%、退出码 0；
- GitHub 固定提交 CI artifact、PR、维护者审批、合并、tag/发布。
