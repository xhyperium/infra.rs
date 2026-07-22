# `clickhousex` 当前实现规范

状态：当前 `0.3.2` 实现合同（reqwest HTTP(S) 默认真实路径；`AnalyticsSink`、批量写与有界池）。
**未宣称 package stable。**

## 0. 权威与范围

`clickhousex` 位于 `crates/adapters/storage/clickhouse`。默认导出
`ClickHouseConfig`、`ClickHousePool`、`ClickHouseClient`；内存实现仅在
`scaffold` feature 下导出。

非目标：原生 9000 协议、DDL/migration 所有权、Cluster/ReplicatedMergeTree 运维、
跨节点 exactly-once 或查询 DSL。

## 1. 公开合同

| 入口 | 当前合同 |
|---|---|
| `ClickHouseConfig` | 严格 env 解析、HTTP(S)、认证、数据库、连接/请求/获取截止时间、池容量；`HTTP_PORT` 优先并兼容 `PORT` |
| `ClickHousePool` | Semaphore 约束 in-flight；close 后拒绝新请求；stats 可观察 |
| `ClickHouseClient` | ping、文本/行查询、JSONEachRow、分块批量写 |
| `AnalyticsSink` | 将 event/payload 写入调用方管理的表 |

`insert_batch` 按 `max_rows_per_chunk` 有界分块；表名等结构化 SQL 标识符必须通过现有校验，
业务值经 HTTP body/参数路径传递。

## 2. 安全与 HTTPS

- loopback 可显式使用 HTTP；远程 HTTP 在配置校验阶段 fail-closed。
- HTTPS 使用 rustls 根证书；`tls_ca_file` 可追加 PEM CA，文件读取与解析在
  `spawn_blocking` 中执行。
- CA 文件仅在 TLS 模式允许；空 CA、非法 PEM、错误 CA 或主机名校验失败均返回带 source 的错误。
- 密码和完整 URL 不进入 Debug / 错误上下文。
- `HTTP_PORT` 与兼容别名 `PORT` 双设不同值时拒绝启动，避免端口配置漂移。
- 非成功 HTTP 响应最多读取 4096 字节前缀用于数字错误码分类；对外错误只保留
  HTTP 状态和可选 `server_code`，不回显 SQL、payload 或认证正文。
- 连接、请求和池获取均有非零截止时间。

## 3. 可复验证据

本地 TLS HTTP 协议实验使用临时 CA 与带 localhost SAN 的服务端证书，证明：

1. 受信 CA + 正确主机名能完成 HTTPS `SELECT 1`；
2. 错误 CA 在握手阶段 fail-closed 且保留 source；
3. 临时私钥与证书总会清理。
4. loopback 失败服务证明 SQL/payload/认证正文与异常 ping 正文不会进入错误。

该实验验证客户端 HTTPS/CA 传输合同，**不**证明真实 ClickHouse 集群、复制或故障切换。

```bash
cargo test -p clickhousex --all-targets
cargo clippy -p clickhousex --all-targets -- -D warnings
node scripts/clickhouse-https-conformance.mjs
cmp .agents/ssot/adapters/storage/clickhouse/spec/spec.md \
  .agents/ssot/adapters/storage/clickhouse/spec/xhyper-clickhousex-complete-spec.md
```

## 4. OPEN / NO-GO

真实 ClickHouse TLS/auth/deadline/并发 live、mTLS、证书热轮换、native 9000、
Cluster/HA、DDL/schema 治理、exactly-once 与 package stable 未承诺。

追溯：`crates/adapters/storage/clickhouse/{src,tests/https_conformance.rs,tests/security_failures.rs}`、
`scripts/clickhouse-https-conformance.mjs`、`docs/ssot/clickhousex-ssot-alignment.md`。
