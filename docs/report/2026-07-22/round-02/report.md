# Round 2: Correctness — 公开 API 正确性审查

| 字段 | 值 |
|------|-----|
| 轮次 | 2/10 |
| 视角 | Correctness |
| 日期 | 2026-07-22 |

---

## 1. 审查摘要

本报告从公开 API 正确性视角审查所有 24 个 workspace member crate，重点关注：

- **a. 公开函数对合法输入的 panic 风险** — checked/fallible API 与 panicking API 的分离
- **b. 结构体字段可��性与构造器校验** — 私有字段 + 验证构造器的模式
- **c. 公开 API 路径中的 unwrap()/expect()** — 携带语义的 expect 与盲 unwrap
- **d. 不变量文档化与可执行性** — 类型系统强制执行 vs 运行时检查
- **e. serde Deserialize 安全性** — 派生 vs 自定义（验证）反序列化

### 总体评分

| 评级 | crate 数目 | crates |
|------|-----------|--------|
| **优秀** | 10 | kernel, decimalx, canonical, configx, schedulex, evidence, observex, contracts, contract-testkit, testkit |
| **良好** | 6 | bootstrap, resiliencx, transportx, redisx, postgresx, kafkax |
| **需改进** | 6 | natsx, ossx, clickhousex, taosx, goalctl, verifyctl |
| **有风险** | 2 | binancex, okxx |

---

## 2. 逐 crate 正确性分析

### 2.1 kernel — xhyper-kernel (L0)

**lint 属性**: `forbid(unsafe_code)` + `deny(missing_docs)` + `deny(unreachable_pub)`

**a. panic 风险**: 无。所有公开 API 为 fallible 或无 panic 路径。

- `XError` 所有构造器不 panic（`invalid`, `missing`, `conflict`, `transient`, `unavailable`, `cancelled`, `deadline_exceeded`, `invariant`, `internal`）
- `Timestamp::checked_add/sub/duration_since` — 全 checked，溢出返回 None
- `ShutdownSignal::is_triggered/wait/wait_timeout` — 锁中毒走 `into_inner` 恢复，不传播 panic
- `ComponentState::try_transition` — 非法转换返回 Err

**b. 字段可见性**: 全部私有，编译时防下游构造。

- `XError` 字段全私有（`kind`, `context`, `retry_after`, `source`）
- `Timestamp`(i64), `MonotonicInstant`(elapsed+domain), `ClockDomain`(u64) 字段私有
- `ShutdownInner` 完全内部，`#[derive(Debug)]`
- 有 `compile_fail` doctest 验证下��无法结构字面量构造 `XError`

**c. unwrap/expect**: 无。公开 API 不使用 unwrap/expect。

**d. 不变量**: 充分文档化且类型系统执行。

- `Timestamp` 无 Default（防零值哨兵），无 serde
- `MonotonicInstant` 跨 domain 比较返回 None（不可静默当可靠）
- `Clock` trait 的 `monotonic()` 无默认实现（编译期强制实现）
- `#[non_exhaustive]` on `ErrorKind`, `ClockError`, `ComponentState`
- `ErrorKind` 按"调用方应如何反应"分类

**e. serde**: 故意不实现（wire 在上层协议层版本化），有 `compile_fail` 测试验证。

### 2.2 testkit — ManualClock

**lint 属性**: `forbid(unsafe_code)` + `deny(missing_docs)` + `deny(unreachable_pub)`

**a-d**: ManualClock 实现 `Clock` trait，确定性测试替身。结构体字段私有，公开 API 无 panic。仅 dev-dep。

**评分**: 优秀。

### 2.3 configx — L1 内存 KV 配置存储

**lint 属性**: `forbid(unsafe_code)` + `deny(missing_docs)` + `deny(unreachable_pub)`

**a. panic 风险**: 无。锁中毒时优雅降级。

- 读操作（`get`, `contains_key`, `len`, `keys`）在锁中毒时返回空/None/0
- 写操作（`set`, `remove`, `clear`, `extend_pairs`）在锁中毒时返回 `XError::invalid("config lock poisoned")`
- `ConfigSnapshot::capture` 锁中毒时返回空快照
- `get_or()` 使用 `unwrap_or_else`（安全，闭包不 panic）
- `validate_key` 检查空、控制字符、长度 >512

**b. 结构体**: `ConfigStore.data` 私有（RwLock<HashMap>），`ConfigSnapshot.entries` 私有。

**c. unwrap/expect**: 无公开 API 中使用 unwrap/expect。`merge_into` 内部使用 `.expect("lock")` 但已通过 snapshot-read 模式避免死锁。

**评分**: 优秀。

### 2.4 schedulex — L1 任务 ID 登记表

**lint 属性**: `forbid(unsafe_code)` + `deny(missing_docs)` + `deny(unreachable_pub)`

**a-d**: 无 panic。`schedule_checked` 和 `schedule_normalized` 返回 `Result`。`ScheduleError` 带 `#[non_exhaustive]`。所有公开方法不含 unwrap/expect。

**评分**: 优秀。

### 2.5 bootstrap — L1 组合根

**lint 属性**: `forbid(unsafe_code)` + `deny(missing_docs)`

**a. panic 风险**: **单处已知 panic** — `Bootstrap::build()` 在 `require_evidence` 未满足时 panic。

- **文档化**: 源码注释明确标注 `"PANIC: require_evidence 未满足时禁止静默成功（含 release）"`
- **可恢复路径**: `try_build()` 提供 Err 路径
- **设计理由**: fail-closed，infra-s9t.4

**c. unwrap/expect**: 无盲性 unwrap。`build()` 的 panic 是有意设计。

**评分**: 良好（单处有文档的 panic，不是正确性缺陷）。

### 2.6 evidence — L1 审计证据追加面

**lint 属性**: `forbid(unsafe_code)` + `deny(missing_docs)`

**a-d**: 
- `validate_event_name` 防止嵌入式换行
- `FileEvidenceAppender::open` 处理父目录创建失败
- `saturating_add` 防序号溢出
- `.expect("lock")` 在 Mutex 上可接受（Mutex 锁失败为进程 bug）

**评分**: 优秀。

### 2.7 observex — L1 可观测封装

**lint 属性**: `forbid(unsafe_code)` + `deny(missing_docs)`

**a-e**: 
- 原子操作 `Ordering::Relaxed` 适合计数器场景
- `normalize_op("")` 返回 `"_"`（无 panic）
- `policy_summary()` 坦诚声明 DEFER 项
- `CountingInstrumentation` 为非生产 metrics 的测试计数器

**评分**: 优秀。

### 2.8 resiliencx — L1 弹性库

**lint 属性**: `forbid(unsafe_code)` + `deny(missing_docs)`

**a-c**: 通过 `contracts::Instrumentation` 注入可观测性。模块化设计（bulkhead/circuit/rate_limit/retry 各自文件）。无公开 API panic。

**评分**: 良好。

### 2.9 transportx — L1 传输层

**lint 属性**: `deny(missing_docs)` + `deny(unreachable_pub)`（**无 `forbid(unsafe_code)`**）

**a. panic 风险**: 核心 HTTP/WS 驱动路径无 panic。

**b. 结构体**: 
- `HttpRequest` 字段为 pub（method, url, headers, body）— **可能被构造非法值**（如非法 HTTP method）
- `HttpResponse` 字段为 pub（status, body）— 无校验
- `ReqwestHttpDriver` 字段私有
- **Debug 脱敏**: `RedactedHeaders` 正确隐藏敏感 header

**c. unwrap/expect**: Mock 和测试钩子用了 `.expect("lock")`，生产路径无。

**d. 不变量**: 
- `PayloadTooLarge` 错误包含 kind/limit/got 结构化信息
- 速率限制从 Retry-After header 解析
- **缺失 `forbid(unsafe_code)`** — 应添加

**评分**: 良好（缺失 forbid(unsafe_code) 扣分）。

### 2.10 decimalx — 十进制数值类型

**lint 属性**: `forbid(unsafe_code)` + `deny(missing_docs)` + `deny(unreachable_pub)`

这是正确性方面最强的 crate，评审详列。

**a. panic 风险**: 三处文档化的 panic，全部有 checked 替代 API。

| API | panic 条件 | checked 替代 |
|-----|-----------|-------------|
| `Decimal::new` | scale > MAX_SCALE | `Decimal::try_new` |
| `Decimal::rescale` | 溢出 | `Decimal::checked_rescale` |
| `Add/Sub/Mul` 运算符 | 溢出 | `checked_add/sub/mul` |

每处 panic 在文档中有 `# Panics` 章节说明。生产资金路径正确要求使用 `checked_*`。

**b. 结构体字段**: `Decimal { mantissa, scale }` — 全部私有。非法 scale 无法表示（`Decimal::new` 和 `try_new` 是唯一构造路径）。

**c. serde**: 全部自定义实现，验证性反序列化。

- `Decimal` Deserialize: `#[serde(deny_unknown_fields)]` + 调用 `try_new` 验证 scale
- `Currency` Deserialize: 调用 `try_new` 验证大写 ASCII 三字节
- `Money` Deserialize: 同时验证 amount 和 currency，`#[serde(deny_unknown_fields)]`
- `Price/Qty/Ratio`: `#[serde(transparent)]` — 依赖内层 Decimal 验证

**d. 不变量**: 
- `i128::MIN / -1` 以 `checked_div` 处理（不 panic）
- `Hash` 在 canonical 形式上计算（尾随零一致）
- `Eq` 基于数值比较（scale 对齐后比较），非字段结构比较
- `FromStr` 拒绝 NaN/Inf，拒绝非法字符

**评分**: 优秀（数据型 crate 的最佳实践参考）。

### 2.11 canonical — 跨层共享 DTO

**lint 属性**: `forbid(unsafe_code)` + `deny(unreachable_pub)` + `deny(missing_docs)`

**a-c**: 纯 DTO crate，无逻辑，无 panic 路径。

**e. serde**: 所有公开 DTO 全面 `#[serde(deny_unknown_fields)]`。

- 复用 `decimalx::Money`（类型身份，非拷贝），继承其验证反序列化
- Wire 承诺矩阵文档化（COMMITTED_WIRE_V1 / V1_1 / V1_2 / V1_3）
- Validation owners 表覆盖所有公开 DTO

**评分**: 优秀。

### 2.12 contracts — 契约 trait 出口

**lint 属性**: `forbid(unsafe_code)` + `deny(unreachable_pub)` + `deny(missing_docs)`

**a-c**: 
- `VenueAdapter` 新方法 `cancel_order_request` / `query_order_request` 有 additive default（返回中文错误），树内 adapter 必须覆��
- `ExecutionVenue` 无 additive default（推荐生产入口）
- `run_tx_commit_on_ok` 正确处理 commit/rollback 双路径
- 依赖白名单（R4）：kernel + canonical + async-trait/bytes/futures-core

**评分**: 优秀。

### 2.13 contract-testkit — 测试契约工具

**lint 属性**: `forbid(unsafe_code)` + `deny(unreachable_pub)` + `deny(missing_docs)`

**a-c**: 仅 dev-dep，非生产图。Fake + Recording + per-trait conformance suite。

**评分**: 优秀。

### 2.14–2.22 适配器 crates (binancex, okxx, redisx, postgresx, kafkax, natsx, ossx, clickhousex, taosx)

#### 存储适配器 (redisx, postgresx, kafkax, natsx, ossx, clickhousex, taosx)

**lint 属性**: 全部有 `forbid(unsafe_code)`。部分缺失 `deny(missing_docs)`。

**共同发现**:

- **`pool.rs` 中使用 `expect`/`unwrap`**: 连接池初始化、获取连接等路径使用 expect���合理性：连接池创建失败为致命错误）
- **`adapter.rs` 中 expect**: 部分适配器在内部映射中 expect，非实际网络 IO
- **脚手架路径**: 部分 storage adapter 仍标记为 scaffold（taosx, clickhousex），生产就绪程度需进一步评审
- **live/conformance 测试覆盖**: redisx, postgresx, kafkax 有 live 和 conformance 测试；natsx 有 live 测试；ossx, clickhousex, taosx 覆盖率较低

**存储适配器评分**:
- redisx: 良好（live_kv_conformance + live_kv 测试）
- postgresx: 良好（live 测试 + bench）
- kafkax: 良好（live_event_bus 测试）
- natsx: 需改进（live 测试存在但 conformance 待完善）
- ossx: 需改进（live 测试存在，但 expect 多处）
- clickhousex: 需改进（scaffold 状态，live_smoke 存在但覆盖率有限）
- taosx: 需改进（scaffold 状态，live_smoke 存在但覆盖率有限）

#### 交易所适配器 (binancex, okxx)

**lint 属性**: **两个 crate 均无 `forbid(unsafe_code)`、`deny(missing_docs)` 或 `deny(unreachable_pub)` 顶级属性！**

这是正确性高优先级项。对比 kernel/decimalx/canonical 的全量 lint 门禁，binancex/okxx 缺少基础安全属性。

**额外风险**:
- 两者均为 scaffold 状态（`binancex` 注释: "默认仍为内存占位"；`okxx`: "内存占位，非真实 HTTP"）
- 如果未来填充真实交易所协议实现时未添加上述 lint 属性，安全默认是弱化的

**评分**: 有风险（缺少核心 lint 属性）。

### 2.23 goalctl — L1 工具

**lint 属性**: `forbid(unsafe_code)`

**a-c**: 编译/执行路径存在 expect/unwrap（CLI 工具有合理性，但应从工具环境考虑）。缺失 `deny(missing_docs)`。

**评分**: 需改进。

### 2.24 verifyctl — L1 验证工具

**lint 属性**: `forbid(unsafe_code)`

**a-c**: plan/execute/dry 路径存在 expect/unwrap。缺失 `deny(missing_docs)`。

**评分**: 需改进。

---

## 3. 跨 crate 正确性风险

### 3.1 反序列化安全链条

**当前状态**: 良好，但存在信任边界。

```
serde_json → decimalx::Decimal (验证 scale) → canonical::Price/Qty (transparent) → canonical::Order (deny_unknown_fields)
```

- `Price/Qty/Ratio` 使用 `#[serde(transparent)]`，信任内层 Decimal 的 `Deserialize`
- `canonical::Money` 有自定义 Deserialize 且 `deny_unknown_fields`
- **链条完整** — 从 JSON 字节到类型安全构造全程验证

**风险**: 如果 adapter crate 直接 serde 到内部类型而未走 decimalx 验证路径，可能绕过 scale 校验。当前未发现此类绕过。

### 3.2 缺失 lint 门禁的规模

| lint | 当前门禁 | 缺失 crate |
|------|----------|-----------|
| `forbid(unsafe_code)` | 22/24 | **binancex**, **okxx** |
| `deny(missing_docs)` | 16/24 | binancex, okxx, 部分 adapters, goalctl, verifyctl |
| `deny(unreachable_pub)` | 14/24 | binancex, okxx, 部分 adapters, bootstrap, transportx, goalctl, verifyctl |

### 3.3 Panic 分类矩阵

| panic 类别 | crate 数 | 典型 crate | 判定 |
|-----------|---------|-----------|------|
| 无公开 API panic | 16 | kernel, configx, schedulex, evidence... | OK |
| 有文档的 panic + checked 替代 | 2 | decimalx, bootstrap | OK |
| 有 expect/unwrap 但仅内部/test | 4 | transportx, adapters | 需关注 |
| 无 lint 保护的 panic 风险 | 2 | binancex, okxx | 高风险 |

### 3.4 Serde 验证覆盖

| 验证模式 | crate 数 | crates |
|---------|---------|--------|
| 自定义 Deserialize 带验证 | 1 | decimalx |
| 依赖 decimalx 验证 | 1 | canonical |
| `deny_unknown_fields` 全量 | 1 | canonical |
| 无 serde（故意） | 1 | kernel |
| 未评估 serde 安全 | 5 | adapters（exchange） |
| 无 serde 依赖 | 15 | 其余 |

---

## 4. 轮次结论

### 4.1 优势

1. **核心数值类型防线坚固**: `decimalx` 的 checked API + 验证 serde + 私有字段 + 禁止 f32/f64 在金额路径的组合，是本仓库最强的正确性防线
2. **kernel 错误分类正确**: `ErrorKind` 按反应分类，`is_retryable()`/`is_bug()` 方法避免字符串匹配，XError 字段全私有
3. **Wire 类型全量 deny_unknown_fields**: `canonical` 所有 DTO 阻止未知字段解析
4. **锁中毒处理标准化**: `configx` 和 `evidence` 对锁中毒提供文档化的降级语义，`ShutdownSignal` 走 `into_inner` 恢复

### 4.2 需立即修复

| 优先级 | 项目 | 涉及 crate | 行动 |
|--------|------|-----------|------|
| P0 | 缺少 `forbid(unsafe_code)` | binancex, okxx | 添加 `#![forbid(unsafe_code)]` 到 lib.rs |
| P1 | 缺少 `deny(missing_docs)` | binancex, okxx, goalctl, verifyctl, 部分 adapters | 添加并补齐公开类型文档 |
| P1 | transportx 缺少 `forbid(unsafe_code)` | transportx | 添加 |
| P2 | 交易所 adapter scaffold 无 lint 基础 | binancex, okxx | 在填充真实协议前建立 lint 基线 |

### 4.3 建议改进

1. **为所有 adapter 添加 `deny(missing_docs)` 和 `deny(unreachable_pub)`** — 与 kernel/decimalx/canonical 对齐
2. **在 scaffold adapter 转为生产 adapter 前完成正确性审查 door-check** — 作为 PR 门禁
3. **为 transportx 公开字段（`HttpRequest.method`）添加构造器校验** — 目前可以直接设非法 HTTP method
4. **为 decimalx 的 panicking 运算符添加 clippy lint 禁止在生产代码中使用** — 类似 `clippy::disallowed_methods`
5. **建立 adapter serde 验证审计清单** — 确认所有 adapter 的反序列化关口走向了 decimalx/contracts 的验证路径

### 4.4 正确性成熟度概览

```
L0 kernel               [██████████] 全量 lint + compile_fail 守卫
L1 configx/schedulex    [██████████] 锁降级/校验/无 panic
L1 observex/resiliencx  [█████████░] 无 panic，仪表面完整
L1 evidence/transportx  [████████░░] evidence 优秀，transportx 缺 forbid
L1 bootstrap            [█████████░] 单处有文档 panic + 可恢复路径
   decimalx             [██████████] 最佳实践参考
   canonical            [██████████] wire 冻结 + deny_unknown_fields
   contracts            [█████████░] additive only + additive default
   contract-testkit     [██████████] dev-only，覆盖完整
   adapters (storage)   [███████░░░] 大部分有 forbid，部分 expect 存在
   adapters (exchange)  [██░░░░░░░░] 缺少基础 lint，scaffold 状态
   tools                [██████░░░░] forbid 存在但 doc 缺失
```
