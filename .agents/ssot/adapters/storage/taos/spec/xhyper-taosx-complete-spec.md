# taosx 实现规范

状态：当前 `0.3.3` 实现合同（REST SQL + WS 可达性探测；真实后端测试默认 `#[ignore]`）。**未宣称 package stable。**

## 0. 权威、职责与非目标

按 Constitution → 已批准 Goal/Design → 本文 → 代码裁定。`taosx` 位于
`crates/adapters/storage/taos`，实现 `contracts::TimeSeriesStore`；`Tick.ts` 在合同侧始终是纳秒 epoch。

当前真实边界：

- SQL、建库、建表、写入与查询全部经 HTTP(S) `POST /rest/sql`。
- `TransportMode::NativeWs` 只对 `/rest/ws` 做有 deadline 的握手及关闭探测；不执行 SQL，
  不证明 WS 认证、长会话、TMQ 或原生 6030 能力。
- scaffold 仅为可选进程内测试实现，不是默认生产路径。
- Native SQL、FFI、HA/Cluster、迁移治理、自动幂等重试与 package stable 均为 **NO-GO / OPEN**。

## 1. Cargo、版本与公开面

版本 `0.3.2`，package `taosx`，默认 feature 为空；可选 feature 仅 `scaffold`。

公开生产面：

- `TaosConfig` / `TransportMode` / `TsPrecision`
- `TaosPool`（别名 `TaosClient`）/ `TaosPoolStats`
- `build_insert_sql_chunks`
- `build_native_ws_url` / `connect_native_ws`（仅 reachability probe）
- 编译期硬上限常量 `HARD_MAX_*`

## 2. 安全与资源合同

### 2.1 TLS 与认证

- 仅精确 `localhost`、`localhost.` 或 `IpAddr::is_loopback()` 可使用明文 HTTP/WS。
- 远程地址必须启用 TLS，且用户名与密码均非空；host 中的 scheme、userinfo、路径、查询和片段拒绝。
- REST 客户端禁止 redirect，避免 Basic Auth 跨端点传播或 HTTPS 降级。
- 密码仅从配置/环境注入，`Debug` 固定脱敏；错误和日志不得输出密码。
- Native WS 探测不证明认证成功；远程 WSS 生产认证仍为 OPEN。

### 2.2 Decimal 精度

- 新建 stable 固定 `ts TIMESTAMP, bid NCHAR(64), ask NCHAR(64)`，symbol 为 tag。
- 写入使用 `Decimal::to_string()` 的带引号文本；查询使用 `Decimal::from_str`。
- 每次写入和查询前以 `DESCRIBE` 校验 bid/ask 为 `NCHAR(64+)`；存量 `DOUBLE` schema
  必须返回 Conflict，禁止静默精度降级。
- 离线测试覆盖 i128 大 mantissa、scale=18、正负值；live test 覆盖 REST JSON 完整往返。
- 时间戳仍按数据库 ms/us/ns precision 量化；该量化不属于 Decimal 金额精度声明。

### 2.3 硬上界

| 资源 | 默认 | 编译期硬上限 |
|---|---:|---:|
| in-flight | 64 | 1024 |
| batch rows | 500 | 10000 |
| 单条 SQL / batch bytes | 1 MiB | 8 MiB |
| REST response bytes | 8 MiB | 64 MiB |
| query rows | 10000 | 100000 |
| close drain | 5 s | 30 s |

- Content-Length 预检与逐 chunk `checked_add` 同时保护成功和错误响应。
- 公开 `exec_sql` 与批写共用 SQL byte cap；自定义 chunk 不得超过配置及硬上限。
- symbol 最多 48 UTF-8 bytes，并以完整十六进制编码映射子表，避免清洗碰撞。
- acquire 通过 RAII guard 计数；任务取消归还计数。close 原子关闭入口、拒绝新 I/O，
  并在配置 deadline 内等待在途请求排空；超时返回 DeadlineExceeded，重复 close 可继续排空。

## 3. 重试与一致性边界

本 crate 不做内部自动重试。多 chunk 写入发生部分成功时，调用方不得把整批盲目重试视为已证明幂等。
TDengine 更新/去重策略、operation-id 与故障后重复写证据均未闭合，因此幂等重试为 **NO-GO**。

## 4. 测试与证据

离线门禁：

```bash
cargo test -p taosx --all-targets
cargo clippy -p taosx --all-targets -- -D warnings
cargo fmt --all -- --check
node scripts/quality-gates/check-workspace-deps.mjs
cmp .agents/ssot/adapters/storage/taos/spec/spec.md \
  .agents/ssot/adapters/storage/taos/spec/xhyper-taosx-complete-spec.md
```

隔离 live conformance（固定镜像 digest、动态 loopback 端口、全局 timeout、finally 清理）：

```bash
node scripts/taos-live-conformance.mjs
```

人工外部服务入口仍保留 `tests/live_smoke.rs` 且默认 ignored；ignored/未运行不得记为 PASS。

## 7. 三轮加固（0.3.3）类型化错误与边界

- `query_series` 缺表空集仅依赖 `ErrorKind::Missing`（`map_taos_code`），禁止依赖驱动文案子串。
- 配置精度与探测精度不一致时 `connect` 必须 `Invalid` fail-closed。
- 离线 `tests/taos_conformance.rs` 覆盖明文远程拒绝、响应上界、schema 冲突、close/背压。
- **未宣称** package stable。
