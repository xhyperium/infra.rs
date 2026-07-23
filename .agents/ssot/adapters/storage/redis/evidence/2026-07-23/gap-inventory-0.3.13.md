# redisx gap inventory — 0.3.13（可行动 = 0）

| 类别 | 项 | 状态 | 证据 |
|------|-----|------|------|
| C1 Streams API | PASS | `streams.rs` + live/e2e/bench |
| C2 Hash/List/Set/ZSet | PASS | `structures.rs` + live/e2e |
| C3 MULTI/EXEC | PASS | `transaction.rs` + live |
| C4 cluster/sentinel feature 门 | PARTIAL→接受 | 代码默认编入；live 仍环境 OPEN |
| C5 multi-lane | PASS | `stats.open = max_in_flight` / `command_lanes` |
| C6 blocking lane 预算 | PASS | `with_conn_budget` + `blpop`/`xread_block` |
| C7 cluster redirect 上限 | PASS | `max_cluster_redirects` → `ClusterClient::retries` |
| C8 warmup/keepalive/reconnect | PASS | config 面 + warmup PING |
| C9 secret provider | PASS | `password_from_provider` / `has_password` |
| C10 默认 TLS true | 不做 | SSOT 诚实默认 false；运维强制 |
| D1 Prometheus exporter | OOS 框架 | 进程内 metrics_snapshot PASS |
| D2 tracing | PASS | `ping` instrument |
| D3 readiness/liveness | PASS | pool API + live |
| D5 Lua fixed SHA | PASS | `script_load_and_eval` / `eval_sha` |

## 允许残留

| ID | 状态 |
|----|------|
| REDISX-9 package stable | OPEN |
| REDISX-10/11/12 topology live | OPEN（env 无拓扑） |
| REDISX-15 Pub/Sub 必达 | NO-GO |
| REDISX-18 SelfValidator 框架 | OOS |
| REDISX-19 覆盖率 100% | OPEN（禁止刷线） |
