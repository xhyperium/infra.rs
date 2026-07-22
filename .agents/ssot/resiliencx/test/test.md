# resiliencx — Test

> 状态：最新修复后三包测试通过；root 串行 coverage `1208 / 1208`、zeros 0、100.0000%、退出码 0。
> 候选已重冻，本地 reviewer 完成，verifier 技术/证据初验完成；GitHub CI artifact pending。

行为测试位于 `crates/resiliencx/src/**` 与 `crates/resiliencx/tests/**`，覆盖安全分类、seeded 实际退避、
sync/async budget、deadline/backoff cancellation、bulkhead poison 恢复与公开 API。
串行 LCOV 门禁要求 `crates/resiliencx/src` 行覆盖 100%，禁止 coverage 排除或空断言。当前
`cargo test -p resiliencx -p postgresx -p redisx --all-features --all-targets` 退出码 0：resiliencx
84 passed；postgresx 52 passed + 6 ignored；redisx 51 passed + 8 ignored。ignored 项需要外部
Postgres/Redis 服务，不作为默认 CI 通过证据。此前
`994 / 994` 是本轮安全补丁前基线，不能作为当前树覆盖率结论。新增测试真实执行 `max_attempts == 0`、
async 不可重试错误返回，并以先执行后清零的控制探针证明 unsafe validation 不再次调用 closure/future；
未添加 coverage 排除或空断言。root 修复后串行复验为 `1156 / 1156`、zeros 0；新增 unchecked
async core 后最终结果为 `1208 / 1208`、zeros 0。本地 reviewer 已完成实现/证据审查，独立 verifier
已完成技术/证据初验；本次纯状态 delta 不改变受审源码/测试。

最新新增行为覆盖：RedisClient 配置 0 attempts 后 GET/SET 在 operation future 和 probe driver 前返回
`Invalid`；unchecked generic async core 的预算耗尽返回标准错误，失败 attempt 观测为 `[1, 2]`；
Postgres/Redis legacy async wrapper 精确验证同一语义。

复验命令见 [`../evidence/README.md`](../evidence/README.md)。
