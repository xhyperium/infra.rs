# Round 4: Compatibility — 兼容性审查

| 字段 | 值 |
|------|-----|
| 轮次 | 4/10 |
| 视角 | 兼容性 — wire/DTO 版本、API 稳定性、迁移��破坏性变更 |
| 日期 | 2026-07-22 |
| 关注 crate | canonical、decimalx、contracts、kernel、所有 types-bearing crate |

---

## 1. 审查摘要

| Crate | Wire 兼容性 | API 稳定性 | 公共 API 门禁 | 总体判定 |
|-------|------------|-----------|--------------|---------|
| **canonical** | **L2 子集** — v1–v1.3 committed wire 全覆盖 | 纯 DTO，无业务表面；新字段/类型走版本常量分批晋升 | `[lints] workspace=true`; `publish=false`; API baseline 存在 | **良好** — 12 类型 committed wire + 完整 golden/N-1 门禁 |
| **decimalx** | **L1** — 字段 shape 为当前事实，**非**跨版本 stable | 字段私有；校验型 serde 拒绝非法输入；Price/Qty/Ratio newtype 透明 | `[lints] workspace=true`; `publish=false`; API baseline 存在 | **良好** — 类型安全有底线；wire stable 明确为 OPEN |
| **contracts** | trait 签名 Additive Only 策略 | VenueAdapter additive default 可运行时检测；ExecutionVenue 无 default | `[lints] workspace=true`; `publish=false`; API baseline 存在 | **部分** — 缺编译时 override 机控 |
| **kernel** | `#[non_exhaustive]` 枚举；Timestamp 私有字段 | L0 抽象，SemVer 未追踪（workspace internal） | `[lints] workspace=true`; `publish=false`; API baseline 存在 | **良好** — non_exhaustive + 私有构造 |

**总体评估**：canonical 的 wire 兼容性在四 crate 中最强，拥有完整的 committed wire 版本体系、双向 golden 测试、N-1 遗留 fixture。所有 crate 均已注册 workspace lint 与 API baseline，但均处于 `publish=false`（未宣称 crates.io）。contracts 的 Additive Only 策略缺少编译时强制门禁，仅运行时检测。

---

## 2. Wire 兼容性详细分析

### 2.1 canonical

**Committed Wire 清单**（来源：`crates/types/canonical/src/wire.rs`）

| 版本 | 类型 | 数量 | 证据 |
|------|------|------|------|
| v1 | CancelOrderRequest, OrderRef, OrderAck, OrderStatus, Side | 5 | `COMMITTED_WIRE_V1` |
| v1.1 | Order | 1 | `COMMITTED_WIRE_V1_1` |
| v1.2 | Tick, Trade | 2 | `COMMITTED_WIRE_V1_2` |
| v1.3 | Position, OrderBookSnapshot, PriceLevel, SymbolMeta | 4 | `COMMITTED_WIRE_V1_3` |
| **合计** | | **12** | |

**冻结策略实现度**（对照 `wire.rs:36–44` 声明的策略逐项核实）：

| 策略条款 | 状态 | 证据 |
|---------|------|------|
| 字段名 = JSON 键（serde 默认无 rename） | **PASS** | 所有 DTO 使用 derive `Serialize/Deserialize` 无 `#[serde(rename)]` |
| 枚举外部 tagging（`{"Exchange":"..."}` / `"Open"`） | **PASS** | `OrderRef`/`OrderStatus`/`Side` serde 重导出均为默认外部 tagging |
| `deny_unknown_fields` | **PASS** | 全部 12 个 committed 类型 + `Decimal`/`Money` 反序列化均启用 | 
| 未知 variant 拒绝 | **PASS** | `wire.rs:148–150`（OrderRef）、`149–150`（OrderStatus）、`167–168`（Side） |
| 缺字段失败 | **PASS** | `wire.rs:182`、`199–204`、`239–242`、`275–277` 等 |
| 非法 Decimal scale 失败 | **PASS** | `wire.rs:244–248`、`277–281`、`340–345` 等 |
| N-1 兼容（legacy fixture） | **PASS** | 每个 v1.x 版本有对应 `fixtures/market/canonical/v1{,.1,.2,.3}/*_legacy.json` |
| 双向 golden 测试 | **PASS** | 每个 committed 类型有 `_bidirectional_golden_and_rejects` 测试 |
| 时间 = Unix ns（CAN-TIME-001） | **PASS** | `ts: i64` 所有文档标注纳秒；`proposed_time` 模块 ms↔ns |

**额外加固**：
- `wire_commitment()` 函数将类型名映射到 `WireCommitment` 枚举，`committed_inventory_is_explicit` 测试锁定清单大小不变
- `fixtures/market/` 下有 11 个 JSON fixture 文件，v1–v1.3 全覆盖
- `validation_owners_table_covers_all_public_dtos` / `wire_commitment_matrix_covers_all_public_dtos` 测试交叉验证 SSOT 文档

**残余 GAP**：
- 无跨语言 wire 协议 envelope 定义（如 protobuf schema / JSON Schema）
- 无 semver diff 自动化检查（破坏性变更不会在 CI 中机械拦截）
- `Money` 未列入 committed 清单（wire SSOT 在 decimalx），canonical 仅 re-export

### 2.2 decimalx

**Wire 状态**（来源：`crates/types/decimal/docs/WIRE.md` + `src/lib.rs`）：

| 类型 | Wire 格式 | 稳定承诺 | 校验机制 |
|------|----------|---------|---------|
| `Decimal` | `{"mantissa": i128, "scale": u8}` | **无** | `deny_unknown_fields` + `try_new` |
| `Currency` | `[u8; 3]`（大写 ASCII） | **无** | 自定义 `Deserialize` → `try_new` |
| `Money` | `{"amount": Decimal, "currency": Currency}` | **无** | `deny_unknown_fields` + `try_new` |
| `Price`/`Qty`/`Ratio` | `#[serde(transparent)]`（透传 Decimal） | **无** | derive + 委托 Decimal 校验 |

**正确性评估**：
- 所有 serde 反序列化路径走 `try_new`，拒绝非法 scale（≥19）与非法 currency（非大写 ASCII）
- WIRE.md 明确声明「当前事实，**非**跨版本 stable」，诚实不越权承诺
- `Money` 的 `deny_unknown_fields` 在 inner `MoneyWire` 结构体上（`src/lib.rs:768`），正确隔离
- Price/Qty/Ratio 使用 `serde(transparent)`，wire 形状完全由 Decimal 决定

**残余 GAP**：
- wire shape 为 struct fields 非 decimal 字符串（`serde_shape_is_struct_fields_not_string` 测锁定），这对 human-readability 不利但正确性无影响
- 无 oracle 差分检验针对 serde 路径
- 未声明跨版本 wire stable（这是有意的 OPEN 项）

---

## 3. API 稳定性评估

### 3.1 kernel — non_exhaustive 策略

kernel 作为 L0 层，在枚举类型上系统使用 `#[non_exhaustive]`：

| 类型 | non_exhaustive? | 字段可见性 |
|------|:---:|------|
| `ErrorKind` | **是** | 9 个固定 variant，可未来扩展 |
| `ClockError` | **是** | 3 个 variant |
| `ComponentState` | **是** | 6 个 lifecycle 状态 |
| `Timestamp` | 不适用 | 私有字段 newtype；`from_unix_nanos`/`as_unix_nanos` 唯一构造/解构 |
| `XError` | 不适用 | 不透明结构体，不可直接构造（仅通过 `::invalid()` 等工厂方法）|
| `ClockDomain` | N/A | 私有字段 newtype；`from_raw`/`as_raw` 构造/解构 |

**评估**：non_exhaustive 策略正确防止下游 crate 做 exhaustive match，为新增 variant 提供向前兼容性。`Timestamp` 的私有构造确保纳秒语义不会意外绕开。

### 3.2 decimalx — 字段私有策略

| 类型 | 字段私有? | 构造/解构入口 | serde 透明? |
|------|:---:|------|:---:|
| `Decimal` | **是** | `new`/`try_new`/`FromStr`; `mantissa()`/`scale()` | 自定义（校验型） |
| `Currency` | **是** | `try_new`/`FromStr`; `as_str()`/`as_bytes()` | 自定义（校验型） |
| `Money` | **是** | `try_new`; `amount()`/`currency()` | 自定义（校验型） |
| `Price` | **是** | `new(Decimal)`; `as_decimal()`/`into_inner()` | transparent |
| `Qty` | **是** | `new(Decimal)`; `as_decimal()`/`into_inner()` | transparent |
| `Ratio` | **是** | `new(Decimal)`; `as_decimal()`/`into_inner()` | transparent |

所有数值类型的内部态不可外部访问，防止绕过 scale 约束。Price/Qty/Ratio 的透明 serde 确保了 wire 形状与 Decimal 一致。

### 3.3 canonical — 纯 DTO 无业务 API

- 所有 DTO 为 **公开字段** struct（例：`pub price: Price`），这是 DTO 层的设计意图
- 无 impl 业务方法（`no_business_methods_on_dto_surface` 测锁定）
- shape 辅助函数（`is_plausible_venue_slug` 等）为纯输入验证，无副作用
- `proposed_time` 为 ms↔ns 转换工具，代数上没有可伸缩性问题

### 3.4 contracts — Additive Only 政策

**声明**（`crates/contracts/src/lib.rs:6`）："一旦发布不可修改签名，只能新增（Additive Only）"

**实现**：
- `VenueAdapter::cancel_order_request` / `query_order_request` 有 additive default，返回中文 `XError::invalid`
- `is_default_cancel_order_request_error` / `is_default_query_order_request_error` 辅助函数可通过运行时检测
- `ExecutionVenue` **无** additive default，实现方必须提供完整方法
- `venue_override_gate.rs` 定义错误常量与检测函数

**残余 GAP**：
- **无编译时 override 门禁** — 树外 adapter 不覆盖 `cancel_order_request` 会编译通过，仅运行时收到 Invalid 错误
- 未使用 sealed trait / 外部实现禁止模式
- Additive Only 的 API snapshot / semver diff 门禁列为 DEFER（`contracts-ssot-alignment.md:92`）

---

## 4. 公共 API 门禁状态

### 4.1 `[lints] workspace = true`

| Crate | 状态 | 来源 |
|-------|:---:|------|
| kernel | **PASS** | `crates/kernel/Cargo.toml:34` |
| decimalx | **PASS** | `crates/types/decimal/Cargo.toml:24` |
| canonical | **PASS** | `crates/types/canonical/Cargo.toml:22` |
| contracts | **PASS** | `crates/contracts/Cargo.toml:21` |

### 4.2 `publish` 状态

所有四个关注 crate 均为 `publish = false`，未宣称 crates.io — 符合当前 L1–L2 内部 GO 叙事。

### 4.3 公共 API 基线（Snapshot）

| Crate | 基线文件 | 大小 |
|-------|------|------|
| kernel | `docs/api-baselines/kernel.txt` | 535 行 |
| decimalx | `docs/api-baselines/decimalx.txt` | 314 行 |
| canonical | `docs/api-baselines/canonical.txt` | 367 行 |
| contracts | `docs/api-baselines/contracts.txt` | 105 行 |

所有基线文件通过 `cargo public-api` 生成（由 `scripts/quality-gates/check-public-api.mjs` 控制）。这些基线可用于检测公共 API 的变化，但与 `cargo-semver-checks` 未集成。

### 4.4 `forbid(unsafe_code)` / `deny(missing_docs)`

| Crate | forbid(unsafe_code) | deny(missing_docs) | deny(unreachable_pub) |
|-------|:---:|:---:|:---:|
| kernel | 无 explicit 但配置中 | N/A | N/A |
| decimalx | **是** | **是** | **是** |
| canonical | **是** | **是** | **是** |
| contracts | **是** | **是** | **是** |

### 4.5 缺失项

- **无 `cargo-semver-checks` / semver diff CI** — 破坏性 API 变更不会被机械拦截
- **无 MSRV CI 矩阵**（criteria §1 L4 要求）
- **无包级 `version` 管理策略** — 所有 crate 硬编码 `0.1.0` 或从 workspace 继承 `0.3.0`，未与 semver 惯例绑定

---

## 5. 轮次结论

### 强项

1. **canonical wire 兼容性是本仓最强亮点** — 12 个 committed 类型覆盖执行路径（v1）、订单（v1.1）、行情/成交（v1.2）、持仓/OrderBook/元数据（v1.3），每层均有：
   - `deny_unknown_fields` 硬拒绝
   - 双向 golden JSON fixture
   - N-1 遗留 fixture 兼容
   - 拒绝样例（未知字段/variant/缺字段/非法 scale）

2. **decimalx 的安全姿态清晰** — 私有字段 + 校验型 serde + 声明 wire 为「当前事实非承诺」，避免过早锁定

3. **kernel 的 non_exhaustive 策略**正确应用在所有公开枚举上

4. **所有 crate 的 `[lints] workspace = true`** 和 API baseline 已到位

### 弱项 / GAP

| 项目 | 严重度 | 位置 | 建议 |
|------|:---:|------|------|
| contracts 缺编译时 Additive Only 机控 | **中** | 无强制 | 考虑 sealed trait 或 macro 生成必须 override 的 compile-fail 标记 |
| 无 semver diff / cargo-semver-checks CI | **低** | 全局 | 待 L4 platform ready 时引入 |
| 无 MSRV CI matrix | **低** | 全局 | 待 L4 平台就绪 |
| 缺跨语言 wire schema（protobuf/JSON Schema） | **低** | canonical | 非当前优先级，作为 L2→L3 升级时可做 |
| decimalx wire 非正式稳定承诺 | **信息** | WIRE.md | 有意的 OPEN — 记录即可 |
| `publish = false` 全 crate | **信息** | Cargo.toml | 与当前内部 GO 一致，记录即可 |

### 对照 SSOT 对齐文

| 对齐文 | 关键兼容性条款 | 状态 |
|-------|------|:---:|
| types-ssot-alignment.md（canonical C-9） | 全 wire Production Ready / package stable | **OPEN**（已记录） |
| types-ssot-alignment.md（decimalx D-8） | wire shape = 当前事实 ≠ stable 承诺 | **PASS**（已记录） |
| contracts-ssot-alignment.md（CT-10） | VenueAdapter override compile/run gate | **部分**（仅运行时） |
| contracts-ssot-alignment.md（DEFER） | Additive Only API snapshot / semver diff | **DEFER**（已记录） |

### 审查结论

**canonical 的 wire 兼容性在本仓中可评级为 L2 Ready（committed wire 子集）**，覆盖执行、订单、行情、持仓四类 DTO，拥有完整的 golden/N-1/拒绝门禁。其他 crate 处于 L1（内部可用），API 门禁基础已到位但缺 semver 自动化。本次审查未发现需要立即修复的兼容性缺陷。
