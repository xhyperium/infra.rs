# clickhousex 实现规范

> 状态：当前 `0.1.0` 实现合同（Mock + clickhouse HTTP 真实驱动已落地；真测 `#[ignore]`，未达 M3）。
> 权威顺序为 `CONSTITUTION.md` → canonical spec → Approved ADR → 本文 → 代码。

## 1. 证据边界与范围

- **Evidence**：`crates/adapters/storage/clickhouse/{Cargo.toml,src/lib.rs}` 提供：
  - `MockAnalyticsSink`：内存有序缓冲 + `snapshot`；
  - `ClickhouseSink`：基于 `clickhouse` crate 的 HTTP 客户端真实实现。
- **Inference**：生产表 schema、批量/背压、交付语义仍可收紧，但不否定现有代码事实。
- **Unknown**：exactly-once、批量 flush API、认证/TLS 生产合同未裁定。

目的：约束当前分析事件 sink。非目标：把 ignored 真测当作 CI 已通过的生产证据；不在本 crate
承担 DDL 治理或查询 DSL。

## 2. 位置、依赖、版本

路径 `crates/adapters/storage/clickhouse`，版本 `0.1.0`，无 features。
普通依赖：`kernel`、`contracts`、`async-trait`、`bytes`、`clickhouse`、`anyhow`；
dev 依赖 `tokio`。符合 R2。独立版本每次仅允许 `x.y.z → x.y.(z+1)`。

## 3. 当前公开 API 与行为

### 3.1 MockAnalyticsSink

- `new() -> Self`；`snapshot() -> Vec<(String, Bytes)>` 克隆当前序列（**不属于** trait，仅测试断言）。
- `AnalyticsSink::sink(&str, Bytes)`：追加 `(event, payload)` 并保序；允许空 payload 和重复 event。

### 3.2 ClickhouseSink

- `new(url, database)`：`clickhouse::Client::default().with_url().with_database()`。
- `client()`：暴露内部客户端（DDL/查询等高级用法）。
- `sink`：`INSERT INTO analytics (event, payload) VALUES (?, ?)`；payload 以 UTF-8 lossy 写入
  `String` 列（非 UTF-8 替换为 U+FFFD）。
- 假定表 schema 为 `analytics(event String, payload String)`，调用方负责建表。

## 4. 错误、并发、生命周期与信任边界

Mock 使用 `RwLock<Vec<_>>`；锁中毒会 panic。真实路径错误映射为 `XError::Transient` 等。
输入不验证且 mock 无容量上限。生产须裁定背压/内存、事件名与 payload schema、敏感数据、认证/TLS。

**证据**：真实测试均 `#[ignore = "需要 ClickHouse 服务（设置 CLICKHOUSE_URL/CLICKHOUSE_DB）"]`；
**不得**当作 CI 默认通过或 M3 生产证据。

## 5. 测试与验收

Mock 单元测试覆盖单次写入、顺序、重复事件、空 payload、trait object。
真实测试 `#[ignore]`。运行：

```bash
cargo test -p clickhousex
cargo test -p clickhousex -- --ignored   # 需 ClickHouse；非 CI 默认
cargo check -p clickhousex --all-targets
cargo clippy -p clickhousex --all-targets -- -D warnings
```

验收要求当前 API/行为匹配、依赖合规、默认检查通过；Unknown 与 ignored 真测不构成生产授权。

## 6. 可追溯性与开放决策

追溯 `docs/architecture/spec.md` §2 R2、§4.3 `AnalyticsSink`、§4.5.1、§5、§8。
`crates/adapters/storage/clickhouse/{Cargo.toml,src/lib.rs,README.md}`。
开放决策：批量/flush、交付语义、认证/TLS、二进制 payload 列类型。
