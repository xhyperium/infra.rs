# taosx 实现规范

状态：当前 `0.3.1` 实现合同（Mock + rest 默认 + native FFI；真测 `#[ignore]`，未达 M3）。**未宣称 package stable。**

## 0. 权威、职责与非目标

按 Constitution → XLib spec → 已批准 ADR → 本文 → 代码裁定。**Evidence** 是直接事实，
**Inference** 是最低验收收窄，**Unknown** 必须评审。`taosx` 位于 `crates/adapters/storage/taos`，
让 ADR-003 的 `native`/`rest` 驱动与 mock 实现同一 `TimeSeriesStore`。

非目标：把 ignored 真测描述为 CI 已通过的生产证据；不在本 crate 承担建库建表与超级表治理。

## 1. Cargo、版本与当前 API

版本 `0.3.1`（package `taosx`）。

| 项目 | 当前事实 |
| --- | --- |
| 普通依赖 | `kernel`、`contracts`、`canonical`、`async-trait`；按 feature：`decimalx`、`anyhow`、`reqwest`、`serde_json` |
| features | `default = ["rest"]`；`rest`；`native`；二者互斥（`compile_error!`） |
| dev-dependency | `tokio`、`decimalx` |

依赖符合 R2。更新仅允许 `x.y.z → x.y.(z+1)`。

### 1.1 MockTimeSeriesStore（始终可用）

`Debug + Default`；按 table 分桶 `RwLock<HashMap<String, Vec<Tick>>>`。
`write_series` 追加（空 vec no-op）；`query_series(table, start, end)` 按 `Tick.ts` 闭区间
`[start, end]` 过滤，缺失 table 返回空 vec。

### 1.2 TaosRestStore（feature `rest`）

- `new(host, port, user, pass)`：REST 端点默认端口 6041，Basic Auth。
- `POST /rest/sql`；`bid`/`ask` 以 NCHAR 十进制字符串存储 Decimal（ADR-006）。
- 表结构假设：`ts TIMESTAMP, symbol NCHAR(32), bid NCHAR(64), ask NCHAR(64)`。

### 1.3 TaosNativeStore（feature `native`）

- 手工 FFI 链接 `libtaos`（非 `taos-rust` crate）：`taos_connect` / `taos_query` /
  `taos_fetch_row` 等。
- `new(host, port, user, pass, db)`：原生端口默认 6030。
- 连接句柄经 `Mutex` 串行；C API 为同步阻塞，async 方法内直接 FFI（生产可外层 `spawn_blocking`）。
- 与 REST 相同表结构与 Decimal 字符串编码。

## 2. ADR/架构差距、生命周期与安全

- **证据**：ADR-003 互斥 feature、mock 始终可用、rest 默认、native 隔离链接——**代码侧已落地**。
- **证据**：`rest_real` / `native_real` 集成测均 `#[ignore]`；**不得**当作 CI 默认通过或 M3 生产证据。
- **证据**：现行 contracts API 使用 `Tick` 的 `write_series/query_series`。
- **推论**：行情金额/数量继续使用 Decimal 派生类型，不得降为 f64。
- **未知**：生产连接池、超时重试、保留策略、批量上限、native 专用 CI runner 是否强制。
- table/时间范围/Tick 是信任边界；真实实现必须防注入、限制批量/结果大小、保护凭据。

## 3. 测试、验收与追溯

Mock 测试覆盖写查、闭区间过滤、缺失表、追加、空写、表隔离和 trait object。
`rest` 有纯函数 Decimal 往返单元测试；REST/native 真测 `#[ignore]`。运行：

```text
cargo test -p taosx
cargo test -p taosx --features rest -- --ignored rest_real   # 需 TDengine REST；非 CI 默认
cargo test -p taosx --no-default-features --features native -- --ignored native_real
cargo clippy -p taosx --all-targets -- -D warnings
cargo fmt -- --check
cargo run -p xtask -- lint-deps
```

验收要求：mock 行为准确；feature 互斥与 Decimal 编码符合 ADR-003/006；不夸大生产就绪；
API/依赖/测试与精确 patch 规则一致。

开放决策：生产超时/连接池、错误映射细粒度、保留与重试策略、native CI 强制策略。
追溯：XLib spec §§2 R2/R6、4.3、4.5、5；ADR-003；ADR-006；
`crates/adapters/storage/taos/{Cargo.toml,src/lib.rs,README.md}`。
