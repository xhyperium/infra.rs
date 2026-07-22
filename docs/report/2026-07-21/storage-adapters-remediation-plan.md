# 存储适配器 — 生产补救计划与工作量估算

> **日期:** 2026-07-21 | **依据:** [storage-adapters-production-readiness.md](storage-adapters-production-readiness.md)
> **总差距:** 3 个 P0 项，5 个 P1 项，5 个 P2 项 | **估算:** 10-14 天

## 1. 计划总览

| 阶段 | 优先级 | 任务数 | 影响 Crate | 天数 |
|------|:--------:|-------|:---------------:|:----:|
| A. Mock | P0 | 3 个 mock | taosx, ossx, clickhousex | 2 |
| B. 连接池 | P0 | 3 个 pool | taosx, ossx, clickhousex | 2-3 |
| C. JetStream | P0 | NATS 持久化 | natsx | 1-2 |
| D. 批量写入 | P0 | Batch insert | clickhousex, taosx | 1-2 |
| E. 重试集成 | P1 | 7 个断路器 | 全部 7 个 | 2-3 |
| F. TLS 强制 | P1 | 4 个适配器 SSL 配置 | postgresx, redisx, kafkax, natsx | 1 |
| G. 错误类型化 | P1 | 3 个适配器类型错误 | taosx, ossx, clickhousex | 1 |
| H. Repository Trait | P1 | PostgresPool 生产级 impl | postgresx | 1 |
| I. 打磨 | P2 | 测试、文档、scaffold 清理 | 全部 7 个 | 2-3 |
| **合计** | | | | **10-14** |

## 2. 详细任务分解

### 阶段 A：Mock 实现（P0，2 天）

**目标：** 提供内存 mock 以实现离线测试。

#### A.1 MockTaos（taosx）— 0.7 天

```rust
// crates/adapters/storage/taos/src/mock.rs
#[derive(Clone)]
pub struct MockTaosStore {
    tables: Arc<RwLock<HashMap<String, Vec<Tick>>>>,
}
```

新文件：`src/mock.rs`（~150 LOC），通过 `#[cfg(feature = "scaffold")]` 门控。

#### A.2 MockOss（ossx）— 0.7 天

```rust
// crates/adapters/storage/oss/src/mock.rs
pub struct MockObjectStore {
    objects: Arc<RwLock<HashMap<String, Bytes>>>,
}
```

新文件：`src/mock.rs`（~120 LOC）。

#### A.3 MockClickHouse（clickhousex）— 0.6 天

```rust
// crates/adapters/storage/clickhouse/src/mock.rs
pub struct MockAnalyticsSink {
    events: Arc<RwLock<Vec<(String, Bytes)>>>,
}
```

新文件：`src/mock.rs`（~100 LOC）。

**交付物：** 3 个新 `mock.rs` 文件（~370 LOC），scaffold feature 门控。

### 阶段 B：连接池（P0，2-3 天）

**目标：** 为当前使用单连接的适配器添加连接池。

#### B.1 TaosPool（taosx）— 1 天

当前 TaosPool 是直接 REST 客户端。需要添加：

- 可配置最大连接数的连接池
- 健康检查（`SELECT 1`）
- REST 客户端连接复用

修改文件：`src/client.rs`、`src/config.rs`（~200 LOC）

#### B.2 OssPool（ossx）— 0.7 天

通过 `reqwest::Client` 池添加 OSS HTTP 连接池。

修改文件：`src/client.rs`、`src/config.rs`（~120 LOC）

#### B.3 ClickHousePool（clickhousex）— 0.7 天

通过 `reqwest::Client` 池添加 ClickHouse HTTP 连接池。

修改文件：`src/client.rs`、`src/config.rs`（~100 LOC）

**交付物：** 3 个适配器连接池配置（~420 LOC）。

### 阶段 C：NATS JetStream（P0，1-2 天）

**目标：** 为 natsx 添加 JetStream 持久流支持。

当前 NatsEventBus 是 at-most-once（Core NATS）。量化交易生产需 JetStream 实现持久化。

```rust
// crates/adapters/storage/nats/src/jetstream.rs
pub struct JetStreamBus {
    jetstream: async_nats::jetstream::Context,
}
```

新文件：`src/jetstream.rs`（~300 LOC），feature gate: `jetstream`

**交付物：** JetStream consumer/producer（~300 LOC）。

### 阶段 D：批量写入（P0，1-2 天）

**目标：** 为分析型和时序型适配器增加批量/分块插入。

#### D.1 clickhousex 批量插入 — 0.5 天

当前：`sink()` 调用时逐行 `INSERT INTO ... FORMAT JSONEachRow`。
目标：缓冲累积行并在批次中刷新。

修改文件：`src/client.rs`（~80 LOC）

#### D.2 taosx 批量插入 — 0.5 天

当前：`write_series()` 调用时逐行 `INSERT INTO ... VALUES (...)`。
目标：每条 INSERT 语句多行写入。

修改文件：`src/client.rs`（~60 LOC）

**交付物：** 2 个适配器批量插入方法（~140 LOC）。

### 阶段 E：重试集成（P1，2-3 天）

**目标：** 集成 `resiliencx` 重试/断路器到全部 7 个适配器。

每个适配器添加 RetryConfig，配置如下：

- 最大重试：3
- 退避：指数型（1s、2s、4s）
- 断路器：连续 5 次失败 → 打开 30 秒

```rust
// 示例：postgresx
use resiliencx::RetryPolicy;

impl PostgresPool {
    pub async fn execute_with_retry<R>(&self, f: impl Fn() -> XResult<R>) -> XResult<R> {
        self.retry_policy.retry(f).await
    }
}
```

每个适配器约 60 LOC 重试配置 + 20 LOC 池集成 × 7 = ~560 LOC total。

**交付物：** 每个适配器一个 RetryPolicy 类（~560 LOC），通过 `RetryConfigBuilder` 可配置。

### 阶段 F：TLS 强制（P1，1 天）

**目标：** 生产部署时增加 SSL/TLS 配置强制。

| 适配器 | 当前 | 目标 | LOC |
|---------|---------|--------|:--:|
| postgresx | `SslMode::Disable` 默认 | 生产环境 `SslMode::Require` 或 `VerifyFull` | 50 |
| redisx | 无 TLS 配置 | `use_tls: bool`，含 `rustls` feature | 50 |
| kafkax | SASL_PLAINTEXT 默认 | SASL_SSL 强制 | 50 |
| natsx | 无 TLS 配置 | `tls: bool` 配置选项 | 50 |

**交付物：** 4 个适配器 TLS 配置选项（~200 LOC）。

### 阶段 G：错误类型化（P1，1 天）

**目标：** 用 `thiserror` 类型化错误替换通用错误类型。

| 适配器 | 当前 | 目标 | LOC |
|---------|---------|--------|:--:|
| taosx | 通用 XError | `TaosError`（ConnectionFailed, InsertFailed, QueryTimedOut 等变体）| 80 |
| ossx | 通用 XError | `OssError`（BucketNotFound, SignatureInvalid, UploadFailed 等变体）| 80 |
| clickhousex | 通用 XError | `ClickHouseError`（TableNotFound, InsertFailed, QueryError 等变体）| 80 |

**交付物：** 3 个适配器类型化错误枚举（~240 LOC）。

### 阶段 H：Repository Trait（P1，1 天）

**目标：** 在生产环境 PostgresPool 上实现 `Repository<T, Id>`。

当前：Repository 仅在 scaffold PostgresAdapter 上实现。
目标：PostgresPool 增加 Repository impl，支持 `find(id)` 和 `save(entity)`。

新文件 `src/repository.rs` 或修改 `src/pool.rs`（~150 LOC）。

**交付物：** 生产级 Repository impl（~150 LOC）。

### 阶段 I：打磨（P2，2-3 天）

#### I.1 测试覆盖 — 1 天

- 每个适配器从 1 个扩展到 3+ 集成测试
- 增加连接失败测试
- postgresx 增加事务边界测试

#### I.2 文档 — 0.5 天

- 每个适配器迁移指南
- 配置参考完整性
- 增加量化交易使用示例

#### I.3 Scaffold 清理 — 0.5 天

- 验证所有 scaffold 均在 feature gate 后
- 移除泄漏到生产路径的 scaffold 模块

## 3. 依赖图

```text
阶段 A (Mock) ───────────────────────────────┐
阶段 B (Pool) ───────────────────────────────┤
阶段 C (JetStream) ─────────────────────────┤  均独立（可并行）
阶段 D (Batch) ──────────────────────────────┤
                                              │
阶段 E (重试) ← 依赖 B (pool 存在) ──────────┘
阶段 F (TLS)  ← 独立
阶段 G (错误) ← 独立
阶段 H (Repo) ← 独立

阶段 I (打磨) ← 依赖 A-H
```

A-D 阶段可并行执行（不同 crate）。E-H 阶段可在 B 完成后启动。

## 4. 单人与双人资源计划

### 单人：10-14 个日历日

| 周 | 一 | 二 | 三 | 四 | 五 |
|------|-----|-----|-----|-----|-----|
| 1 | A1 (MockTaos) | A2 (MockOss) | B1 (TaosPool) | B2 (OssPool) | C (JetStream) + D1 (CH batch) |
| 2 | D2 (Taos batch) + E (重试 ×4) | E (重试 ×3) + F (TLS) | G (错误) + H (Repo) | I (打磨) | I (打磨) |

### 双人：7-10 天

| 开发者 1 | 开发者 2 |
|-------------|-------------|
| A (Mock) + C (JetStream) | B (Pool) + D (Batch) |
| E (重试 ×4) + H (Repo) | E (重试 ×3) + F (TLS) + G (错误) |
| I (打磨 — 测试) | I (打磨 — 文档) |

## 5. 验收标准

个阶段**完成**的标志：

| 阶段 | 标准 |
|-------|----------|
| A | `cargo test -p {crate} --features scaffold` 通过，含 mock |
| B | 健康检查端点响应，pool 大小可配置 |
| C | `cargo test -p natsx --features jetstream` 通过，使用本地 NATS |
| D | 批量插入吞吐量 > 单行插入吞吐量的 10 倍 |
| E | 连续 5 次失败后断路器打开 |
| F | `SslMode::Require` 为默认，CI 中无 TLS 连接失败 |
| G | 所有错误变体含文档注释，错误链完整保留 |
| H | `impl Repository<T, Id> for PostgresPool` 编译并测试通过 |
| I | `cargo test --workspace` 通过，`cargo clippy -D warnings` 通过 |

## 6. 风险登记

| 风险 | 概率 | 影响 | 缓解措施 |
|------|:---------:|:------:|------------|
| NATS JetStream API 变更 | 低 | 中 | 锁定 async-nats 版本 |
| TDengine REST API 变更 | 低 | 低 | 使用稳定 v3.x API 端点 |
| Aliyun OSS 签名兼容性 | 低 | 中 | 生产前用 dev bucket 测试 |
| 连接池配置冲 | 中 | 低 | 使用 ConfigBuilder 模式，附合理默认值 |
| Scaffold feature gate 断裂 | 低 | 低 | CI 中验证 `#[cfg(feature = "scaffold")]` |
