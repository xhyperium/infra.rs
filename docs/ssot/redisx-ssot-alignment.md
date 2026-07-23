# redisx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `redisx` |
| SSOT | `.agents/ssot/adapters/storage/redis/` |
| 实现 | `crates/adapters/storage/redis` |
| 审计日期 | 2026-07-23 |
| version | `0.3.4`（`0.3.3` 为 main 既有历史） |
| 结论 | **生产默认客户端已落地**；参数化 retry safety 已对齐；Cluster / Sentinel / TLS 真实 live 保持 **OPEN**；未宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `RedisPool / RedisClient / RedisConfig` |
| 模式 | 三种命令连接代码路径存在；仅 Standalone 有既有 KV live 入口 |
| TLS | secure 构造路径 PASS；真实 TLS 握手 OPEN |
| resiliencx | budget 下只读 + 无 TTL SET/MSET 幂等重试；相对 TTL SET/DEL/PEXPIRE 多试前拒绝；PUBLISH 不自动重试 |
| contracts | `contracts::KeyValueStore`（+ 可选 pubsub） |
| 环境变量 | `FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS,MODE,NODES,SENTINEL_MASTER}` |
| live | `tests/live_kv.rs · tests/live_kv_conformance.rs`（`#[ignore]`） |
| bench | `benches/kv_hot_path.rs` |
| Pub/Sub | Standalone only，同源 ACL/TLS/deadline；Cluster/Sentinel 失败关闭 |
| 原 OBJECTIVE DEFER | 部分实现；Cluster / Sentinel / TLS live 仍 OPEN |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| REDISX-1 | workspace member | PASS | `cargo metadata -p redisx` |
| REDISX-2 | 生产默认导出 | PASS | `src/lib.rs` |
| REDISX-3 | from_env | PASS | config · `FOUNDATIONX_REDISX_*` |
| REDISX-4 | 离线测试 | PASS | 最终 51 passed + 8 ignored；ignored live 需要外部 Redis |
| REDISX-5 | live 入口 | PASS | `tests/live_kv.rs · live_kv_conformance.rs` |
| REDISX-6 | bench 有界 | PASS | `benches/kv_hot_path.rs` |
| REDISX-7 | crate docs | PASS | docs/usage · config · operations |
| REDISX-8 | SSOT 11 层 + landing/draft | PASS | `.agents/ssot/adapters/storage/redis/` |
| REDISX-9 | package stable | OPEN | 禁止宣称 |
| REDISX-10 | Cluster 模式 | OPEN | 代码/离线拒绝连接测试存在；无真实 Cluster live |
| REDISX-11 | Sentinel 模式 | OPEN | `async_master_for` 路径存在；无真实 Sentinel/failover live |
| REDISX-12 | TLS 强制路径 | OPEN | secure 构造测试 PASS；无真实 TLS 握手 live |
| REDISX-13 | resiliencx 接入 | PASS | client 按 ReadOnly/Idempotent/UnsafeSideEffect 分类；PUBLISH NeverAutomatic；`RedisOperation::Set` 粗粒度保持 AmbiguousWrite |
| REDISX-14 | Pub/Sub 配置一致性 | PASS | pool 保存并复用 config；非 Standalone 失败关闭 |
| REDISX-15 | Pub/Sub 重连/必达 | OPEN | 无断线恢复 live；Redis Pub/Sub 不承诺可靠投递 |
| REDISX-16 | 种子 URL 脱敏 | PASS | Debug / endpoint / 配置错误负向测试 |

## 当前重试与原子性矩阵

| 操作 | budget 配置后的合同 | 诚实边界 |
|---|---|---|
| GET / EXISTS / PTTL / MGET | `ReadOnly`；仅 Transient 失败消耗预算重试 | MGET 只承诺单节点/同 slot |
| 无 TTL SET / MSET | 固定输入 `Idempotent`；仅 Transient 失败消耗预算重试 | 响应丢失仍不等于未执行；MSET 跨 slot 不承诺原子性 |
| 相对 TTL SET / DEL / PEXPIRE | `UnsafeSideEffect`；`max_attempts > 1` 在 I/O 前拒绝 | 单次允许；避免 TTL 起点或返回值漂移 |
| PUBLISH | `NeverAutomatic`；不自动重试 | 消息仍可能丢失；自动重试可能重复投递 |

`RedisOperation::Set` 无法携带 TTL 参数，因此公开查询面保守保持 `AmbiguousWrite`；真实 client
依据 `ttl = None` / `Some(_)` 细分为 `Idempotent` / `UnsafeSideEffect`。

## 验证

```bash
cargo test -p redisx --all-targets
cargo test -p redisx --all-targets --features pubsub
cargo clippy -p redisx --all-targets --features pubsub -- -D warnings
```

候选曾冻结；治理修正后最终 SHA 待重冻。最终 reviewer/verifier、最终 SHA CI、PR/审批/合并仍 pending。

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- gap：[gap-matrix.md](./gap-matrix.md)
