# redisx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `redisx` |
| SSOT 路径（唯一 canonical） | `.agents/ssot/adapters/storage/redis/` |
| 路径裁决 | **不**新增 `.agents/ssot/adapters/storage/redisx/`；目录名 `redis` 对齐 storage×7，package 名 `redisx` |
| 实现 | `crates/adapters/storage/redis` |
| 审计日期 | 2026-07-23 |
| version | `0.3.10`（selfcheck §6.5；`0.3.9` metrics/stream） |
| 结论 | **Standalone P0+ 生产默认客户端** + **自验证 catalog**；Cluster / Sentinel / TLS live **OPEN**；**禁止** package stable / Draft 全文 DoD |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `RedisPool / RedisClient / RedisConfig` |
| 模式 | 三种命令连接代码路径存在；仅 Standalone 有 KV/PubSub live 证据 |
| TLS | secure 构造路径 PASS；真实 TLS 握手 OPEN；默认配置 `tls=false`（dev 明文 opt-in） |
| resiliencx | budget 下只读 + 无 TTL SET/MSET 幂等重试；相对 TTL SET/DEL/PEXPIRE 多试前拒绝；PUBLISH 不自动重试 |
| contracts | `contracts::KeyValueStore`（+ 可选 pubsub） |
| 环境变量 | `FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS,MODE,NODES,SENTINEL_MASTER}` |
| live | `tests/live_kv.rs` · `live_kv_conformance.rs` · `live_pubsub_conformance.rs`（`#[ignore]`，需真实 Redis） |
| bench | `benches/kv_hot_path.rs` |
| Pub/Sub | Standalone only；重连/必达 **NO-GO** |
| Draft 全量 100% | **未达成**（P2–P4 + secret provider / Cluster live 等 OPEN；池 metrics 与 result stream 已在 0.3.9） |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| REDISX-1 | workspace member | PASS | `cargo metadata -p redisx` |
| REDISX-2 | 生产默认导出 | PASS | `src/lib.rs` |
| REDISX-3 | from_env | PASS | config · `FOUNDATIONX_REDISX_*` |
| REDISX-4 | 离线测试 | PASS | default + pubsub lib 离线全绿（live ignored） |
| REDISX-5 | live Standalone KV | PASS | live_kv 5 + conformance 2 在真实 Redis 下 `--ignored` 通过（2026-07-23） |
| REDISX-6 | bench 有界 | PASS | `benches/kv_hot_path.rs` |
| REDISX-7 | crate docs | PASS | docs/usage · config · operations |
| REDISX-8 | SSOT 目录 | PASS | `.agents/ssot/adapters/storage/redis/`（非 `redisx/`） |
| REDISX-9 | package stable | OPEN | 禁止宣称 |
| REDISX-10 | Cluster 模式 live | OPEN | 代码路径 + 拒绝连接测试；无真实 Cluster 拓扑 |
| REDISX-11 | Sentinel 模式 live | OPEN | 建连路径；无 failover live |
| REDISX-12 | TLS 握手 live | OPEN | secure 构造测试 PASS；无真实 TLS live |
| REDISX-13 | resiliencx 接入 | PASS | ReadOnly / Idempotent / UnsafeSideEffect / NeverAutomatic |
| REDISX-14 | Pub/Sub 配置一致性 | PASS | Standalone only；非 Standalone fail-closed |
| REDISX-15 | Pub/Sub 重连/必达 | NO-GO | Redis Pub/Sub 不承诺可靠投递；无 resubscribe live |
| REDISX-16 | 种子 URL 脱敏 | PASS | Debug / endpoint 负向测试 |
| REDISX-17 | Draft P0 可宣称 bar | PASS | Standalone 生产默认 KV 客户端（有界） |
| REDISX-18 | Draft 全文 DoD | OPEN | 见 evidence gap-matrix-v0 |
| REDISX-19 | 行覆盖率 100% | OPEN / 残余已文档 | 离线 lib 约 **71%**；+live 约 **79.5%**；`error_map` 约 **99%**；见 [coverage-residual.md](../../.agents/ssot/adapters/storage/redis/evidence/2026-07-23/coverage-residual.md) |
| REDISX-20 | 交付可追溯 | PASS | [delivery-record.md](../../.agents/ssot/adapters/storage/redis/evidence/2026-07-23/delivery-record.md) · PR #281 / #285 |
| REDISX-21 | 调用级总 deadline | PASS | `with_call_deadline`；acquire 计入总预算 |
| REDISX-22 | get_bytes/set_bytes | PASS | 别名 → get/set |
| REDISX-23 | pipeline_set | PASS | 管道批量 SET；跨 slot 非原子 |
| REDISX-24 | Lua + fencing 锁 | PASS | `eval_script` / `lock_*`；关键写须 fence；非 package stable |
| REDISX-25 | 池累计 metrics | PASS | `metrics_snapshot`；非 OTel exporter |
| REDISX-26 | Pub/Sub result stream | PASS | `into_result_message_stream`；断线一次 Err；无重连 |
| REDISX-27 | 自验证 selfcheck | PASS | `redisx::selfcheck` §6.5 11 项；短路/skip/catalog；非 tools/verifyctl |

## 10 轮审查与 gap 证据

| 产物 | 路径 |
|------|------|
| Gap matrix v0 | [evidence/2026-07-23/gap-matrix-v0.md](../../.agents/ssot/adapters/storage/redis/evidence/2026-07-23/gap-matrix-v0.md) |
| Review pass 1–5 | [evidence/2026-07-23/passes-01-05.md](../../.agents/ssot/adapters/storage/redis/evidence/2026-07-23/passes-01-05.md) |
| Review pass 6–10 | [evidence/2026-07-23/passes-06-10.md](../../.agents/ssot/adapters/storage/redis/evidence/2026-07-23/passes-06-10.md) |

## 当前重试与原子性矩阵

| 操作 | budget 配置后的合同 | 诚实边界 |
|---|---|---|
| GET / EXISTS / PTTL / MGET | `ReadOnly`；仅 Transient 失败消耗预算重试 | MGET 只承诺单节点/同 slot |
| 无 TTL SET / MSET | 固定输入 `Idempotent`；仅 Transient 失败消耗预算重试 | 响应丢失仍不等于未执行 |
| 相对 TTL SET / DEL / PEXPIRE | `UnsafeSideEffect`；`max_attempts > 1` 在 I/O 前拒绝 | 单次允许 |
| PUBLISH | `NeverAutomatic` | 消息仍可能丢失 |

## 验证

```bash
# 离线
cargo test -p redisx --all-targets
cargo test -p redisx --all-targets --features pubsub
cargo clippy -p redisx --all-targets --features pubsub -- -D warnings

# live（私有 env，勿提交）
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx.env
set -a && source /tmp/foundationx.env && set +a
cargo test -p redisx --test live_kv -- --ignored
cargo test -p redisx --test live_kv_conformance -- --ignored
cargo test -p redisx --features pubsub --test live_pubsub_conformance -- --ignored
cargo bench -p redisx --bench kv_hot_path
```

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- Draft 入库快照：`.agents/ssot/adapters/storage/redis/plan/infra-rs-draft-spec-goal.md`
