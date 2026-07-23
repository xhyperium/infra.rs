# Rust 开发规则补充（本仓落地）

> **定位**：本仓对组织《[Rust 编码规范（完整版）v2.1.1](https://github.com/xhyperium/.github/blob/main/rulesets/rust/RULES.md)》的**加严与落地**。  
> **上位**：组织 `rulesets/rust/`（P0 不可削弱）· 宪章 [§4.0](../constitution/04-code-standards.md#40-rust-全局编码规范强制上位)  
> **冲突裁决**：组织 P0 > 本文件加严条款 > 局部实现选择  
> **语言**：人类可读文本强制中文（见 [编码与语言约定.md](./编码与语言约定.md)）

本文档将常用工程约束收敛为可审查条目，供 Code Review、Agent 与质量门禁引用。完整规则 ID 与专项细则仍以上位文档为准。

---

## 1. 错误处理

| 规则 | 级 | 说明 | 上位 |
|------|----|------|------|
| 公共库禁止裸 `unwrap()` / 无注释 `expect()` | P0 | 必须返回类型化 `Result`；测试模块除外 | R-SEC-001 / R-ERR-001 |
| 业务 / 应用层可用 `expect()` | P0 | 仅限不可恢复或启动 fail-fast，且必须附带明确原因 | R-ERR-002 / R-ERR-003 |
| 统一错误类型 | P0 | 每 crate 自有 `Error` + `Result` 别名；库用 `thiserror` + `source` 链 | R-ERR-004 / R-API-010 |
| 禁止裸 `Box<dyn Error>` 扩散 | P0 | 公共 API 禁止 `String` / `anyhow::Error` / 未约束的 `Box<dyn Error>` 作为错误类型；`anyhow` 仅限 bin 组合根 | R-API-013 / R-API-014 |

### 落地要点

```rust
// ✅ 库：类型化错误
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConnectError {
    #[error("连接失败: {endpoint}")]
    Io {
        endpoint: String,
        #[source]
        source: std::io::Error,
    },
}

// ✅ 应用启动 fail-fast
// PANIC: 缺少运行必需配置，进程无法安全启动
let cfg = load_config().expect("缺少运行必需配置 INFRA_CONFIG");

// ❌ 公共库
let v = map.get(k).unwrap();
```

- 错误文案使用中文；字段名 / 机器码可用英文
- 捕获后必须处理：传播 / 转换 / 记录；禁止吞错

---

## 2. Unsafe 使用

| 规则 | 级 | 说明 | 上位 |
|------|----|------|------|
| 默认禁止 `unsafe` | P0 | 无 unsafe 的库 crate 推荐 `#![forbid(unsafe_code)]` | R-SEC-002 |
| 必须使用时写安全不变量 | P0 | `unsafe` 块紧邻 `// SAFETY:`，说明为何满足不变量（非复述代码） | R-SEC-002 |
| 优先封装为安全接口 | P0 | 禁止在业务层直接暴露 / 调用裸 `unsafe`；unsafe 收敛在最小封装模块 | 本仓加严 |

### 落地要点

```rust
// SAFETY: `idx` 已由 `check_bounds` 验证；`slice` 与调用方生命周期绑定且不重叠写入。
let v = unsafe { slice.get_unchecked(idx) };
```

- 含 `unsafe` 的 crate 须在模块文档写明安全契约
- 建议 CI / 定期审计：`rg -n "unsafe\\s*\\{" --glob '*.rs'`

---

## 3. 并发与异步

| 规则 | 级 | 说明 | 上位 |
|------|----|------|------|
| 共享状态明确锁类型与粒度 | P0 | 优先 `Arc<Mutex<_>>` / `Arc<RwLock<_>>`（或 `tokio::sync` 等价物）；文档化保护的不变量 | R-RT-011 |
| 禁止持锁期间执行 IO / 耗时操作 | P0 | 禁止持锁跨 `.await`；临界区只做内存态读写 | R-RT-011 / R-SEC-003 |
| 异步任务必须处理取消与超时 | P0 | 外部调用有 timeout；后台任务有取消策略（token / channel / abort 所有者） | R-RT-013 / R-RT-030 |
| 运行时统一 | P0 | workspace 统一 `tokio`；库内禁止擅自 `block_on` | R-RT-001 / R-RT-003 |
| 通道与缓存有界 | P0 | 禁止默认无界 channel / 无界缓存 | R-RT-020 / R-RT-022 |

### 落地要点

```rust
// ✅ 缩小临界区，不持锁 await
let snapshot = {
    let guard = state.lock().await;
    guard.clone_snapshot()
};
do_io(snapshot).await?;

// ❌ 持锁跨 await
let guard = state.lock().await;
do_io(guard.x).await?;
```

- `JoinHandle` 必须有生命周期所有者；禁止无监督 fire-and-forget
- 重试须区分可重试 / 不可重试；非幂等写默认不重试

---

## 4. 性能

| 规则 | 级 | 说明 | 上位 |
|------|----|------|------|
| 热路径控制分配与格式化 | P1 | 热路径禁止无必要动态分配、`format!` 字符串、默认开启的高频日志 | §16 性能 |
| 集合预分配容量 | P1 | 已知规模时使用 `with_capacity` / `reserve` | 本仓加严 |
| 关键路径提供 benchmark | P1 | 性能敏感 / 交易 / 协议解析路径须有 `criterion`（或项目统一）基准 | quant-dev-spec |
| 性能优化须有证据 | P1 | 不得以主观「更快」合并；附 bench 或指标 | 组织性能条款 |

### 落地要点

- 热路径日志使用 `trace!` / 采样 / feature 门控，默认可关闭
- 量化领域另见 [quant-dev-spec.md](./quant-dev-spec.md)（有界通道、SoA、`bytes::Bytes` 等）
- 微优化不得破坏可读性与正确性；先正确再快

---

## 5. 依赖管理

| 规则 | 级 | 说明 | 上位 |
|------|----|------|------|
| 新增外部依赖须评审 | P1 | 说明原因、替代方案、维护状态、许可证、体积影响 | R-DEP-001 |
| 禁止功能重叠的多 crate | P1 | 同能力只保留一条技术栈（如 HTTP 客户端、序列化框架） | R-DEP-008 |
| 版本集中锁定 | P0 | 第三方依赖写入根 `[workspace.dependencies]`；成员 `workspace = true`；`Cargo.lock` 入库 | R-DEP-004 |
| 定期升级与审计 | P0 | CI `cargo deny check`；高危漏洞阻断 | R-SEC-007 / R-SEC-008 |

### 落地要点

```bash
# 本地依赖集中管理门禁
node scripts/quality-gates/check-workspace-deps.mjs
cargo deny check
```

- 新增依赖流程：查 workspace 已有项 → 无则写入根表 → 成员仅 `workspace = true` → deny 通过
- 优先标准库；能用内部 crate 则禁止重复造轮（见 [项目开发规则.md](./项目开发规则.md) DEV-001）

---

## 6. 测试

| 规则 | 级 | 说明 | 上位 |
|------|----|------|------|
| 公共模块必须有单元测试 | P0 | 与源码同文件 `#[cfg(test)] mod tests`；覆盖核心逻辑与错误路径 | R-TEST / §15 |
| 核心交易逻辑必须有集成测试 | P0 | exchange / 订单 / 余额等路径放在 `tests/` 或专项 suite；无真实密钥时用 fake / recording | 本仓加严 |
| 禁止提交「失败却被 ignore」的测试 | P0 | 不得用 `#[ignore]` 掩盖红测；flaky 须 `#[ignore = "原因; owner; 期限"]` 并限期修复 | R-TEST flaky |
| 断言要具体 | P1 | 错误路径断言类型 / 码，禁止只 `is_ok()` / `is_err()` | R-TEST-001 / R-TEST-002 |

### 落地要点

```bash
cargo test --workspace --all-features
# 推荐
cargo nextest run --workspace
```

- 测试独立、无顺序依赖；时间与随机可注入
- 交易相关：**未**宣称 package stable 的 adapter 不得因「有集成测」外推生产闭合
- `contract-testkit` 仅 dev-dep；禁止 production graph 依赖

---

## 7. 日志与可观测性

| 规则 | 级 | 说明 | 上位 |
|------|----|------|------|
| 统一日志框架 | P0 | 使用 `tracing`；生产路径禁止 `println!` / `eprintln!` / `dbg!` | R-OBS-001 |
| 日志含上下文、禁敏感数据 | P0 | 字段化（英文 key + 中文消息）；禁止完整 token / 密码 / 私钥 | R-OBS-003 / R-SEC-006 |
| 热路径日志可关闭或降级 | P0 | 默认不用无采样 `info!` 打满热路径；用级别 / 采样 / feature 控制 | R-OBS-004 |
| 外部 I/O 可追踪 | P0 | HTTP / DB / MQ 等须有 span（系统、操作、关键 ID） | R-OBS-010 |

### 落地要点

```rust
error!(error = %err, operation = "connect", host = %host, "数据库连接失败");
info!(order_id = %id, "订单处理完成");
// 敏感：只记是否存在或指纹，不记原值
debug!(has_token = !token.is_empty(), "已加载凭证");
```

- 级别语义：`error` 需处理失败 · `warn` 可恢复 · `info` 生命周期 · `debug`/`trace` 排障
- 指标 label 禁止高基数（原始 user_id、完整 URL）

---

## 8. 序列化

| 规则 | 级 | 说明 | 上位 |
|------|----|------|------|
| 对外接口显式定义序列化结构 | P0 | wire / API 使用专用 DTO，显式 `serde` 属性与 rename 策略 | R-API / §8.4 |
| 禁止直接序列化内部类型 | P0 | 内部领域类型不得默认作为公共 wire 出口；需映射到稳定 DTO | 本仓加严 |
| 版本变更兼容旧数据 | P0 | 新增字段带默认；删除 / 改名走迁移或 major；枚举 wire 变更视为 breaking | release / P-4 |
| 金融字段校验反序列化 | P0 | `Decimal` / `Money` 等非法 scale/currency 必须反序列化失败 | quant / wire 基线 |

### 落地要点

```rust
// ✅ 对外 DTO
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OrderWireV1 {
    pub client_order_id: String,
    #[serde(with = "decimal_serde")]
    pub price: Decimal,
}

// ❌ 直接把含内部实现细节的类型塞进公共 API
#[derive(serde::Serialize)]
pub struct InternalOrderEngine { /* 私有状态 */ }
```

- 公共 enum 优先 `#[non_exhaustive]`；wire 格式变更记 CHANGELOG 并标 `BREAKING`
- 已承诺的 wire 字符串 / error code 变更视为 breaking

---

## 9. 提交前检查清单

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
node scripts/quality-gates/check-workspace-deps.mjs
# 推荐
cargo deny check
```

| 项 | 自检 |
|----|------|
| 错误 | 无裸 unwrap；库返回 Result；错误类型统一 |
| unsafe | 无业务层裸 unsafe；有则 SAFETY 注释 |
| 并发 | 无持锁 await；有 timeout / 取消 |
| 性能 | 热路径无滥分配 / 滥日志；关键路径有 bench |
| 依赖 | workspace 集中；无重叠栈；deny 通过 |
| 测试 | 公共模块有单测；交易核心有集成测；无掩盖红测的 ignore |
| 日志 | tracing；无敏感明文；热路径可降级 |
| 序列化 | 对外 DTO；不泄内部类型；兼容旧数据 |

---

## 10. 与相关文档的关系

| 文档 | 关系 |
|------|------|
| 组织 [RULES.md](https://github.com/xhyperium/.github/blob/main/rulesets/rust/RULES.md) | 上位完整版；专项见 security / async / testing / observability 等 |
| 宪章 [§4.0](../constitution/04-code-standards.md#40-rust-全局编码规范强制上位) | 本仓采纳上位 + 加严入口 |
| [项目开发规则.md](./项目开发规则.md) | 工程总览（worktree / 提交 / 复用 / 门禁） |
| [quant-dev-spec.md](./quant-dev-spec.md) | 量化领域（精度、时间戳、bench） |
| [VERSIONING.md](./VERSIONING.md) | crate 独立版本；交付 PATCH +1 |

---

## 变更日志

| 日期 | 变更 |
|------|------|
| 2026-07-23 | 初始版本：错误 / unsafe / 并发异步 / 性能 / 依赖 / 测试 / 日志 / 序列化 八项本仓落地 |
