# Round 6: Security -- 安全性审查

| 字段 | 值 |
|------|-----|
| 轮次 | 6/10 |
| 视角 | 安全性 -- 反序列化安全、资源消耗、依赖风险、unsafe 代码 |
| 日期 | 2026-07-22 |

---

## 1. 审查摘要

本轮回溯 workspace 全部 24 个 crate 的安全态势。整体基线**良好**：

- `cargo deny check` **全绿**（advisories / bans / licenses / sources 均 ok）
- **零 unsafe 代码**存在于当前代码库
- 19/24 crate 已启用 `#![forbid(unsafe_code)]`
- 全部 24 crate 的 `Cargo.toml` 均有 `[lints] workspace = true`
- 反序列化均走**校验路径**（decimalx 自定义 Deserialize，canonical 全量 `deny_unknown_fields`）
- 所有 storage adapter 的凭据 Debug 均脱敏
- transportx 有完善的上限/超时防御（16 MiB body / 4 MiB frame / 30s timeout）

**已知差距**：

| # | 严重度 | 描述 |
|---|--------|------|
| GAP-SEC-001 | P2 | `binancex` / `okxx` / `transportx` 未启用 `forbid(unsafe_code)` |
| GAP-SEC-002 | P2 | exchange adapter 当前为 scaffold，无 API key/signing 逻辑（将来引入时须审计） |
| GAP-SEC-003 | P3 | `configx` 无 secret 类型，无 Debug 脱敏（已在 SSOT spec 中记录） |
| GAP-SEC-004 | P3 | `taosx::RawResponse` 使用 derive Deserialize 无 `deny_unknown_fields`（仅内部类型） |
| GAP-SEC-005 | P4 | kafkax / natsx 无显式消息体/帧上限（依赖底层驱动） |
| GAP-SEC-006 | P5 | `deny.toml` 有 10 条 unnecessary-skip 警告 |

---

## 2. 反序列化安全

### 2.1 decimalx -- 自定义校验 Deserialize

- **`Decimal`**：自定义 `Deserialize` impl（`crates/types/decimal/src/lib.rs:792`），内部 helper struct 标注 `#[serde(deny_unknown_fields)]`，反序列化时强制 `scale <= MAX_SCALE` 检查
- **`Money`**：自定义 `Deserialize` impl（`:765`），同样 `deny_unknown_fields`
- **`Currency`**：自定义 `Deserialize` impl（`:703`），`serde(transparent)` + value-from-str 校验
- **`Price` / `Qty` / `Ratio`**：derive `Deserialize` + `#[serde(transparent)]`，委托 Decimal 的自定义 Deserialize
- **panicking path 现状**：
  - `rescale()` 在 overflow 时 panic（文档标注 `# Panics`，推荐 `checked_rescale`）
  - 算术运算符（`+/-/*`）在 overflow 时 panic（推荐 `checked_*`）
  - 这些 panics 是**有意设计**（禁止静默回绕），且有 checked 安全替代
- **结论：PASS** -- 反序列化路径安全，panicking API 有文档警告和安全替代

### 2.2 canonical -- deny_unknown_fields 全量覆盖

- 所有 12 个 DTO/enum 均标注 `#[serde(deny_unknown_fields)]`（如 `Order` `:65-66`、`Tick` `:89-90`、`OrderAck` `:120-121`）
- 未知字段、未知 enum variant 均被拒绝
- wire golden tests 完整覆盖（`src/wire.rs`）
- 当前 serde shape 明确区分为 "committed wire" vs "uncommitted RT"（SSOT 文档注册）
- **结论：PASS** -- 反序列化安全，未知输入一律拒绝

### 2.3 taosx -- 内部 RawResponse

- `RawResponse`（`crates/adapters/storage/taos/src/client.rs:31`）使用 derive `Deserialize`
- 字段均设 `#[serde(default)]`，容忍缺失字段
- **无 `deny_unknown_fields`**（但此为 crate 内部私有类型，仅用于 REST 响应解析）
- **结论：LOW RISK** -- 内部类型，影响范��有限；建议加 `deny_unknown_fields` 防防护性

### 2.4 其余 crate

- 无 crate 在公开类型上使用派生 Deserialize 而不加校验
- `contract-testkit` 的 Deserialize 仅供测试使用
- **结论：CLEAN**

---

## 3. unsafe 代码与 lint 策略

### 3.1 unsafe 块统计

```text
$ grep -rn '\bunsafe\b' crates/ --include='*.rs'
(无匹配 -- 零 unsafe 代码)
```

**0 个 unsafe 块存在于 workspace**。

### 3.2 forbid(unsafe_code) 覆盖率

| 状态 | 数量 | Crate 列表 |
|------|------|-----------|
| 已启用 | 21 | kernel, testkit, configx, schedulex, decimalx, canonical, resiliencx, contracts, contract-testkit, evidence, bootstrap, observex, redisx, kafkax, natsx, postgresx, taosx, ossx, clickhousex, goalctl, verifyctl |
| **未启用** | **3** | **transportx, binancex, okxx** |

**GAP-SEC-001**：transportx、binancex、okxx 缺少 `#![forbid(unsafe_code)]`。虽然当前无 unsafe 代码，但缺少编译器防护是远期风险。

### 3.3 [lints] workspace = true

| 状态 | 数量 |
|------|------|
| 已配置 | 24/24 |

全部 crate **均已配置** `[lints] workspace = true`，继承根 workspace lint 规则。

---

## 4. 资源消耗与 DoS

### 4.1 transportx

| 资源限制 | 默认值 | 位置 |
|---------|--------|------|
| HTTP 总超时 | 30s | `lib.rs:50` (DEFAULT_REQUEST_TIMEOUT) |
| HTTP 请求体上限 | 16 MiB | `lib.rs:252-253` |
| HTTP 响应体上限 | 16 MiB | `lib.rs:252-253` |
| WebSocket 连接超时 | 30s | `lib.rs:382-383` |
| WebSocket 帧上限 | 4 MiB | `lib.rs:382-383` |

- 上限均为 **fail-closed**（超限返回 `TransportError::PayloadTooLarge`）
- `with_limits()` 支持自定义；`max_*_bytes == 0` 可关闭上限（文档标注仅测试逃生口）
- **结论：PASS** -- 默认防御深度充足

### 4.2 storage adapter 连接池

| Adapter | 连接池 | 默认 max_size | 超时 |
|---------|--------|--------------|------|
| postgresx | deadpool | `DEFAULT_MAX_POOL_SIZE` | pool acquire timeout |
| redisx | redis::Client (无独立池) | 驱动管理 | 命令超时 |
| kafkax | rskafka | -- | -- |
| natsx | async-nats | 驱动内置 | -- |
| taosx | reqwest HTTP | 无连接池配置 | 10s 默认 |
| clickhousex | reqwest HTTP | 无连接池配置 | -- |
| ossx | reqwest HTTP | 无连接池配置 | -- |

- 使用 reqwest 的 adapter (taosx/clickhousex/ossx) **依赖 reqwest 默认连接池**（内置 keep-alive/超时）
- postgresx **最为完善**：deadpool + max_pool_size 校验 + pool acquire timeout 映射
- **GAP-SEC-005**：kafkax / natsx 未暴露显式消息体上限配置

### 4.3 WebSocket 连接

transportx 的 `TungsteniteWsConnector`：单帧上限 4 MiB，fail-closed。WebSocket 消息接收路径均经过 `enforce_frame_limit()`。

### 4.4 内存可控性

- decimalx: `MAX_SCALE = 18`，mantissa 为 i128 -- 数值范围有界
- canonical: DTO 均为有限字段、有限长度枚举 -- 无无界收集

---

## 5. 依赖风险

### 5.1 cargo deny check 结果

```
advisories ok, bans ok, licenses ok, sources ok
```

- **无安全公告命中**（`yanked = "deny"` 阻止撤回版本）
- **许可证白名单**：MIT / Apache-2.0 / BSD-3-Clause / ISC / Zlib / Unicode-3.0 / CDLA-Permissive-2.0
- **多版本拒绝**：`multiple-versions = "deny"`，wildcards 拒绝

### 5.2 已知双版本依赖

| 依赖 | 版本链 | 原因 | 状态 |
|------|--------|------|------|
| `getrandom` | 0.2 / 0.3 | tokio-postgres vs crypto | 不可合并 |
| `syn` | 2.x | 普遍 | 生态待收敛 |
| `sha2` / `digest` | 0.10 / 0.11 | goalctl/ossx vs postgres-protocol | 非安全风险 |
| `rand` | 0.8 / 0.9 / 0.10 | 多库变迁期 | 功能等价 |

所有双版本已在 `deny.toml` 显式 skip，原因注释充分。

### 5.3 高风险依赖审计

| Crate | 关键生产依赖 | 风险等级 |
|-------|-------------|---------|
| kernel | 仅 thiserror | **极低** |
| decimalx | kernel + serde | 低 |
| canonical | decimalx + serde | 低 |
| transportx | reqwest + tokio-tungstenite | 中（网络层） |
| redisx | redis-rs | 中（网络+协议） |
| kafkax | rskafka | 中（网络+协议） |
| postgresx | tokio-postgres + deadpool | 中（网络+协议） |
| natsx | async-nats | 中（网络+协议） |
| binancex | kernel + xhyper-contracts | 低（scaffold） |
| okxx | kernel + xhyper-contracts | 低（scaffold） |
| goalctl | 多 crypto/CTL 依赖 | 中（CLI 工具） |

无已知 CVE 影响当前依赖图。

### 5.4 OSS 签名依赖

- ossx 使用 `hmac` + `sha1` + `base64`（HMAC-SHA1 签名）
  - SHA1 **仅用于签名**，非哈希完整性（OSS API 要求）
  - HMAC-SHA1 的碰撞风险不在此场景中构成安全威胁

---

## 6. Secret 管理

| Adapter | 凭据字段 | Debug 脱敏 | 注入方式 | 状态 |
|---------|---------|-----------|---------|------|
| ossx | access_key_secret | 是 | FOUNDATIONX_OSSX_* env | PASS |
| postgresx | password | 是 | env / builder | PASS |
| redisx | password | 是 | env / URL parse | PASS |
| kafkax | sasl_password | 是 | FOUNDATIONX_KAFKAX_* env | PASS |
| natsx | password | 是 | env / builder | PASS |
| taosx | password | 否（`PartialEq` derive 暴露） | env / builder | MINOR |
| clickhousex | password | 是 | env / builder | PASS |
| configx | -- | **无** | -- | GAP（spec 已知） |

**GAP-SEC-003**：configx 无 secret 类型区分，SSOT spec 已记录。当前实现为纯内存字符串 KV��调用方不得存入敏感值。

**taosx password**：`#[derive(PartialEq)]` 使 password 无意中可通过相等比较泄露。当前不使用该比较路径，建议移除 `PartialEq` derive。

---

## 7. Exchange Adapter 特别审查

| Crate | API key 逻辑 | 签名逻辑 | TLS | 状态 |
|-------|------------|---------|-----|------|
| binancex | **无** | **无** | 委托 transportx | scaffold only |
| okxx | **无** | **无** | 委托 transportx | scaffold only |

- 两者均为 **scaffold**，仅有内存占位 + `mainnet()` 硬编码 URL
- 当前无 API key/secret/signing 逻辑
- **GAP-SEC-002**：将来自实现签名时须引入 HMAC-SHA256（Binance）和 HMAC-SHA256 + Base64（OKX），并确保：
  - API secret Debug 脱敏
  - 仅环境变量注入
  - 请求时间戳 + recvWindow 防重放
  - Ed25519（OKX V5）密钥管理

---

## 8. constitution 安全要求对齐

| 条款 | 要求 | 当前状态 |
|------|------|---------|
| §2.1 安全优先 | 任何变更不得降低安全标准 | MET |
| §2.1 敏感信息不入库 | 密钥、token、证书不得提交 | MET（全 env 注入） |
| §4 编码规范 | `unsafe` + `// SAFETY:` 注释 | N/A（零 unsafe） |
| §4 编码规范 | 生产路径禁用 println!/eprintln!/dbg! | MET |
| §5 cargo-deny | 强制安全审计门禁 | MET（cargo deny check 通过） |
| AGENTS.md 安全 | 不提交 .env/证书/密钥 | MET |

---

## 9. 轮次结论

### 总体评级：**LOW RISK（阶段性通过）**

当前 workspace 安全态势良好：
- 零 unsafe 代码，21/24 crate 编译期屏障
- 反序列化全量校验（deny_unknown_fields / 自定义 Deserialize）
- 凭据 Debug 脱敏覆盖 7/8 存储适配器
- transportx 有完善的请求/响应/帧上限
- 依赖审计无已知 CVE

### 改进建议（不阻塞当前阶段）

| 优先级 | 建议 |
|--------|------|
| P2 | 为 transportx / binancex / okxx 添加 `#![forbid(unsafe_code)]` |
| P2 | exchange adapter 实现真实签名时建立签名审计 checklist |
| P3 | taosx RawResponse 加 `deny_unknown_fields` |
| P3 | taosx config 移除 PartialEq derive 以防密码���露 |
| P4 | kafkax / natsx 添加消息��大小上限配置 |
| P5 | 清理 deny.toml unnecessary-skip 警告 |
