# redisx 实现规范

状态：当前 `0.1.0` 实现合同（Mock + 真实驱动代码已落地；真实集成测 `#[ignore]`，未达 M3）

## 0. 文档定位与裁定边界

本文细化 XLib spec v0.2 的 `redisx` 合同。**证据（Evidence）**来自 Constitution、XLib spec、
已批准 ADR 和当前代码；**推论（Inference）**只收窄验收要求；**未知（Unknown）**必须评审，不能
由本文批准。冲突按“Constitution → XLib spec → 已批准 ADR → 本文 → 代码”裁定。

## 1. 职责、范围与非目标

- **证据**：路径为 `crates/adapters/storage/redis`，属于存储适配器层，目标是实现
  `contracts::KeyValueStore`；可选 `PubSub` 仍是扩展目标。
- **证据**：当前同时提供：
  - `MockKvStore`：内存 `HashMap` + `RwLock`，无外部依赖；
  - `RedisKvStore`：基于 `redis` crate 的 `aio::ConnectionManager` 真实实现。
- 非目标：在本版本承诺集群路由、分布式锁、计数器 API、PubSub 生产合同，或把 `#[ignore]`
  真测当作 CI 已通过的生产证据。

## 2. 位置、依赖与版本

| 项目 | 当前事实 |
| --- | --- |
| 路径/版本 | `crates/adapters/storage/redis` / `0.1.0` |
| 普通依赖 | `kernel`、`contracts`、`async-trait`、`anyhow`、`redis` |
| dev-dependency | `tokio` |
| feature | 无（真实驱动始终编译） |

依赖符合 R2，且无同层适配器依赖。每次版本更新必须且只能为 `x.y.z → x.y.(z+1)`。

## 3. 当前公开 API 与行为

### 3.1 MockKvStore

`MockKvStore` 是 `Debug + Default` 的拥有型 `RwLock<HashMap<String, Vec<u8>>>`；`new()` 创建空库。
`KeyValueStore::get` 克隆值，缺失返回 `Ok(None)`；`set` 插入或覆盖并返回 `Ok(())`。
**TTL 参数被明确忽略**，带 TTL 的值不会过期。锁中毒处使用 `unwrap()`，会 panic。

### 3.2 RedisKvStore

`RedisKvStore` 持有可克隆的 `redis::aio::ConnectionManager`。
- `new(client: redis::Client) -> XResult<Self>`：连接失败映射为 `XError::Unavailable`。
- `pool()`：暴露内部 `ConnectionManager`。
- `get`：`GET`；IO 失败为 `XError::Transient`。
- `set`：`SET`；若 `ttl` 为 `Some` 且毫秒 > 0，附加 `PX`（毫秒）。

## 4. 差距、并发与信任边界

- **证据**：真实驱动代码与 `redis` 依赖已引入；生产 Redis 集群/HA/TLS 合同未裁定。
- **证据**：真实集成测试存在且全部 `#[ignore = "需要 redis 服务（设置 REDIS_URL）"]`；
  **不得**将其表述为 CI 默认通过或 M3 生产证据。
- **未知**：key 命名空间、序列化、超时重试、集群与 shutdown 合同。
- **未知**：`PubSub`、`CounterStore`、`DistLock` 仅为待评审提案，不是现有 API。
- mock 的 `RwLock` 只提供单进程 map 访问。key/value 是不可信字节边界，生产实现须限制大小并
  避免把凭据或 value 写入日志。

## 5. 测试与验收

Mock 内联测试覆盖 set/get、缺失、覆盖、TTL 被忽略及 trait object。真实测试覆盖 set/get、
缺失、TTL 到期、二进制值，但均 `#[ignore]`。运行：

```text
cargo test -p redisx
cargo test -p redisx -- --ignored   # 需 Redis；非 CI 默认
cargo clippy -p redisx --all-targets -- -D warnings
cargo fmt -- --check
cargo run -p xtask -- lint-deps
```

验收要求：API/Cargo/测试与本文一致；不得把 mock 称为完整 Redis 语义；不得把 ignored 真测
宣称为生产就绪；新增扩展 trait 前先评审；版本更新遵守精确 patch 规则。

## 6. 开放决策与追溯

待裁定：连接生命周期与错误分类细节、集群/TLS、TTL 边界、扩展 trait（PubSub 等）。
追溯：XLib spec §§2 R2/R6、4.3、4.5、5；
`crates/adapters/storage/redis/{Cargo.toml,src/lib.rs,README.md}`。
