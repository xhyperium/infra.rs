# `clickhousex` 当前实现规范

状态：当前 `0.3.3` 实现合同（reqwest HTTP(S) 默认真实路径；`AnalyticsSink`、批量写与有界池；
三轮加固新增校验顺序、错误分类与背压边界的显式 fail-closed 条款）。
**未宣称 package stable。**

## 0. 权威与范围

`clickhousex` 位于 `crates/adapters/storage/clickhouse`。默认导出
`ClickHouseConfig`、`ClickHousePool`、`ClickHouseClient`；内存实现仅在
`scaffold` feature 下导出。

非目标：原生 9000 协议、通用 DDL/migration 治理、Cluster/ReplicatedMergeTree 运维、
跨节点 exactly-once 或查询 DSL。

## 1. 公开合同

| 入口 | 当前合同 |
|---|---|
| `ClickHouseConfig` | 严格 env 解析、HTTP(S)、认证、数据库、连接/请求/获取截止时间、池容量；`HTTP_PORT` 优先并兼容 `PORT` |
| `ClickHousePool` | Semaphore 约束 in-flight；close 后拒绝新请求；stats 可观察 |
| `ClickHouseClient` | ping、文本/行查询、JSONEachRow、分块批量写 |
| `AnalyticsSink` | 幂等确保固定 `ANALYTICS_TABLE` 后写入 event/payload；其他表由调用方管理 |

`insert_batch` 按 `max_rows_per_chunk` 有界分块；表名等结构化 SQL 标识符必须通过现有校验，
业务值经 HTTP body/参数路径传递。

**校验顺序合同（显式条款，原为隐性假设）**：`insert_json_each_row` /
`insert_batch` 必须在发出任何网络请求**之前**完成表名标识符校验、行结构
（必须为 JSON object）校验；`insert_batch` 的分块逻辑必须在表名校验**之后**
才执行。空 `rows` 输入必须直接短路成功，不发起网络请求、不占用 in-flight
许可。此顺序保证非法输入永远不会触达 Semaphore 或 HTTP 层。

`insert_batch` 的分块结果必须体现为**逐块独立的 HTTP 请求**：调用方可观察
到的请求次数等于 `ceil(rows.len() / max_rows_per_chunk)`，不得把多个 chunk
合并为一次请求或反之拆分同一 chunk 为多次请求。

## 2. 安全与 HTTPS

- loopback 可显式使用 HTTP；远程 HTTP 在配置校验阶段 fail-closed。
- HTTPS 使用 rustls 根证书；`tls_ca_file` 可追加 PEM CA，文件读取与解析在
  `spawn_blocking` 中执行。
- CA 文件仅在 TLS 模式允许；空 CA、非法 PEM、错误 CA 或主机名校验失败均返回带 source 的错误。
- 密码和完整 URL 不进入 Debug / 错误上下文。
- `HTTP_PORT` 与兼容别名 `PORT` 双设不同值时拒绝启动，避免端口配置漂移。
- 非成功 HTTP 响应最多读取 4096 字节前缀用于数字错误码分类；对外错误只保留
  HTTP 状态和可选 `server_code`，不回显 SQL、payload 或认证正文。该截断边界
  必须验证：超出 4096 字节的响应体，截断点之后的内容不得出现在错误信息中。
- 连接、请求和池获取均有非零截止时间。
- **背压边界合同（显式条款，原为隐性假设）**：`max_in_flight` 限制的
  Semaphore 许可耗尽时，等待中的请求必须在 `acquire_timeout` 到期后返回
  `ErrorKind::DeadlineExceeded`，错误上下文包含配置的 `max` 值；不得无限期
  阻塞，也不得静默丢弃请求。
- **HTTP 错误分类合同（显式条款，原为隐性假设）**：`map_http_error` 必须覆盖
  以下全部分支且分类结果稳定：HTTP 404 → `Missing`；`server_code=57`
  （TABLE_ALREADY_EXISTS）→ `Conflict`；`server_code=60`（UNKNOWN_TABLE）→
  `Missing`；`server_code=81`（UNKNOWN_DATABASE）→ `Missing`；HTTP 5xx →
  `Transient`；HTTP 403 → `Unavailable`；其他未知 4xx → `Invalid`。

## 3. 可复验证据

本地 TLS HTTP 协议实验使用临时 CA 与带 localhost SAN 的服务端证书，证明：

1. 受信 CA + 正确主机名能完成 HTTPS `SELECT 1`；
2. 错误 CA 在握手阶段 fail-closed 且保留 source；
3. 临时私钥与证书总会清理。
4. loopback 失败服务证明 SQL/payload/认证正文与异常 ping 正文不会进入错误。

该实验验证客户端 HTTPS/CA 传输合同，**不**证明真实 ClickHouse 集群、复制或故障切换。

三轮加固（R1 负向验收 → R2 对抗回归）新增以下可复验证据（均离线单测，不依赖外部服务）：

5. `insert_json_each_row` / `insert_batch` 对非法表名、非 object 行的拒绝，以及
   空 `rows` 的短路成功，均通过必然连接失败的端口（`http_port: 1`）构造 pool 验证——
   若校验被跳过则测试会因网络错误而非预期的 `Invalid` 失败，从而间接证明校验先于网络。
6. `query_rows` 的 TabSeparated 解析（跳过空行、按 tab 拆列）、`map_http_error`
   的全部分支、`read_error_prefix` 的 4096 字节截断边界均有专项单测锚定。
7. 背压边界：`max_in_flight=1` 时第二个并发请求在 `acquire_timeout` 后收到
   `DeadlineExceeded` 的对抗测试（`second_request_times_out_waiting_for_the_only_permit`）。
8. `insert_batch` 分块的 HTTP 层证据：5 行按 `max_rows_per_chunk=2` 产生 3 次
   独立 POST 的对抗测试（`insert_batch_sends_one_http_request_per_chunk`）。

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p clickhousex --all-features --all-targets
node scripts/clickhouse-https-conformance.mjs
cmp .agents/ssot/adapters/storage/clickhouse/spec/spec.md \
  .agents/ssot/adapters/storage/clickhouse/spec/xhyper-clickhousex-complete-spec.md
```

## 4. OPEN / NO-GO

真实 ClickHouse TLS/auth/deadline/并发 live、mTLS、证书热轮换、native 9000、
Cluster/HA、DDL/schema 治理、exactly-once 与 package stable 未承诺。

**本条款为历史声明的原样保留**：三轮加固仅补齐既有实现路径的单测锚点与新增
离线对抗性边界回归，**未**接入真实 ClickHouse 实例、**未**新增任何真实集群
证据；因此本节 OPEN/NO-GO 范围与 `0.3.2` 相比不变，不得因新增单测而清零或
弱化。

追溯：`crates/adapters/storage/clickhouse/{src,tests/https_conformance.rs,tests/security_failures.rs}`、
`scripts/clickhouse-https-conformance.mjs`、`docs/ssot/clickhousex-ssot-alignment.md`。
