# L1 Infra 模块生产就绪度审计（partial）

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| 审计范围 | STATUS.md 全部 **L1** 模块（7 个） |
| 源码根 | `/home/workspace/infra.rs`（主仓只读） |
| 报告路径 | `docs/report/2026-07-21/_partials/l1-infra.md` |
| 判据 | 生产 7 维 + 分层 L1–L5（见 [core-crates-production-readiness.md](../../core-crates-production-readiness.md) §2 · [plans/2026-07-21-core-crates-production-readiness.md](../../../plans/2026-07-21-core-crates-production-readiness.md) §2） |
| 性质 | **只读审计**；不等同发布批准或 package stable |
| 测试抽样 | `cargo test -p bootstrap -p configx -p evidence -p observex -p resiliencx -p schedulex -p transportx --all-targets` → **全绿**（见 §测试证据） |

> **STATUS 完成度 ≠ Production Ready。**  
> STATUS 公式是「布局 50% + 有测试 25% + 内容 25%」的结构进度；scaffold 上限 0.55。  
> 本报告按 **正确性 / 契约完整性 / 兼容性 / 可运维性 / 安全性 / 可验证性 / 治理合规** 七维，并给出 **L1 Internal → L5 Release** 分层目标可达性。

---

## 0. 总览结论表

| # | 模块 | STATUS | 成熟度 | 实质面（一句话） | 生产判定（相对声明面） | 建议目标层 | 可安全使用 | 不可宣称 |
|---|------|--------|--------|------------------|------------------------|------------|------------|----------|
| 1 | `bootstrap` | **100%** active | 组合根较厚 | typed composition + 关停所有权 + 有界上下文 | **有条件就绪（Internal）** | L1 Internal | 进程内装配 / 关停信号 | 生产 app 生命周期 / async drain / 真实 venue 注入 |
| 2 | `configx` | **90%** partial | 内存 KV | `ConfigStore` 字符串 map | **合同内就绪 / 平台未就绪** | L1（内存合同） | 进程内配置字典 | 多源 / schema / 热更新 / secret |
| 3 | `evidence` | **88%** partial | 最小追加面 | trait + 内存 appender | **开发/测试默认就绪** | L1（内存） | 注入探测 / 单测 | 持久化审计 / 签名链 / 跨进程证据 |
| 4 | `observex` | **88%** partial | tracing 三方法 | `TracingInstrumentation` | **最小面就绪** | L1（tracing 面） | 重试/熔断事件打点 | OTEL / metrics / flush·shutdown |
| 5 | `resiliencx` | **98%** active | 弹性原语完整 | 重试+熔断+限流+舱壁（无墙钟） | **接近 Internal Ready** | L1 Internal | 同步弹性策略（确定性） | async wait / retry budget / 墙钟冷却 |
| 6 | `schedulex` | **88%** partial | 登记表 | 任务 **ID 登记** 非定时器 | **登记合同就绪 / 调度未就绪** | L1（registry） | 任务 ID 集合管理 | timer / cron / Job 执行 / 分布式 |
| 7 | `transportx` | **95%** active | 真实 HTTP/WS | reqwest + tungstenite + mock | **传输边界有条件就绪** | L1 + 部分 I/O | 客户端 HTTP/WS（自管超时/策略） | TLS 矩阵 / 池 / 体大小硬限 / gRPC |

**七模块整体 Production Ready：否。**  
相对各自 **active SSOT 最小面**，多数已「合同对齐 + 测试绿」；相对 **生产运行时系统**（配置中心、证据总线、完整可观测、真实调度、平台矩阵），均未到 L2–L5 签字条件。

---

## 1. 生产判据与分层（本报告采用）

### 1.1 七维

| # | 维度 | 本层关注点 |
|---|------|------------|
| 1 | **正确性** | 公开可达输入是否 panic / 静默错 / 不一致状态 |
| 2 | **契约完整性** | 失败、取消、超时、背压、幂等、关停是否可表达 |
| 3 | **兼容性** | API / wire 版本、演进、回滚 |
| 4 | **可运维性** | 错误分类、可追踪、并发与失败门禁 |
| 5 | **安全性** | 资源边界、超时、密钥/PII、反序列化 |
| 6 | **可验证性** | 合同测 vs 自往返；真实 I/O vs 纯内存 |
| 7 | **治理合规** | 中文错误、lint、MSRV、文档诚实、分层依赖 R3 |

### 1.2 分层 Production Ready

| 层 | 含义 |
|----|------|
| **L1 Internal Ready** | 进程内库；非法状态可控；checked 无 panic；持续 CI |
| **L2 Wire Ready** | 跨进程/落盘有版本与拒绝策略 |
| **L3 Contract Ready** | trait 语义合同 + conformance + 非 scaffold 入口 |
| **L4 Platform Ready** | OS/MSRV 矩阵实测 + API 棘轮 |
| **L5 Release Ready** | 人签 + DEFER 冻结 + CHANGELOG |

---

## 2. 逐模块分析

### 2.1 bootstrap（`crates/bootstrap` · package `bootstrap`）

#### 能力面

| 能力 | 证据 |
|------|------|
| ADR-016 唯一组合根 | `Bootstrap` / `PlatformContext` / `AppContext` / `BootstrappedApp`；无 Gate/register/resolve |
| ADR-005 注入 | 默认 `observex::TracingInstrumentation`；可 `NoopInstrumentation`；trait 权威 `contracts` |
| Evidence 注入 | `with_evidence` / `require_evidence`；re-export `evidence::*` |
| 关停 | `ShutdownController::trigger` 消费 self；基于 kernel `ShutdownGuard/Signal` |
| 有界上下文 | `MarketDataContext` / `ExecutionContext` + `Bounded*` 标签面 trait |
| 错误映射 | `BootstrapError` → Missing / Invalid / Unavailable → `XError` |
| 依赖 | kernel + contracts + observex + evidence（R3.1 豁免组合根） |

源：`src/lib.rs` · `error.rs` · `bounded.rs` · `traits.rs` · README · CHANGELOG · [bootstrap-ssot-alignment.md](../../../../docs/ssot/bootstrap-ssot-alignment.md)

#### 缺口 / 风险

| 类别 | 说明 | 严重度 |
|------|------|--------|
| 内存/替面 | `Bounded*` 仅 `label`/`venue_id`，**非** contracts 生产 async 能力 | P1 |
| require_evidence | `build`/`build_app` 仅 `debug_assert!(validate)`；**release 可静默跳过** | P0 |
| 关停深度 | 有 trigger 信号，**无** async drain / 逆序补偿 / 超时 deadline 编排 | P1 |
| 用户可见错误 | `Display` 为英文（`missing required dependency: …`） | P2（治理） |
| I/O | 组合根本身无网络 I/O（正确）；真实 adapter 接线 DEFER | — |
| panic 路径 | 生产路径无裸 unwrap；测试路径有 expect | OK |

#### 七维速评

| 维 | 评 |
|----|----|
| 正确性 | 声明面内良好；release `require_evidence` 缺口 |
| 契约完整性 | 装配+关停信号完整；无 drain 合同 |
| 兼容性 | `0.3.0` workspace；`publish=false`；无 API 棘轮 |
| 可运维性 | 错误可映射 ErrorKind；无启动 metrics |
| 安全性 | 无 secret；无 Service Locator 面 |
| 可验证性 | unit+集成 `public_api` + example；无 e2e 真 app |
| 治理 | 文档诚实；SSOT PASS 子集；中文错误未达标 |

#### 判定

- **相对 active SSOT 组合根最小面**：对齐，测试绿 → **L1 Internal 有条件**  
- **相对生产应用组合根**：**未就绪**（async 生命周期、真实依赖、require fail-closed 全路径）  
- STATUS **100%** 反映布局/测试/LOC，**放大**了生产语义完成度

---

### 2.2 configx（`crates/configx` · package `configx`）

#### 能力面

- 线程安全内存 `ConfigStore`：`new` / `get` / `set` / `Default`
- 锁失败不对称：读中毒 → `None`；写中毒 → `XError::Invalid("config lock poisoned")`
- 生产依赖 **仅** kernel；`default = []`
- 文档明确：**不是**多源/schema/热更新

#### 缺口

| 类别 | 说明 | 严重度 |
|------|------|--------|
| 能力面 | 无文件/env/远端加载、无 schema 校验、无类型化配置 | 设计 DEFER（生产配置平台 P0 缺口） |
| 安全 | 无 secret 类型/脱敏；任意 `String` value | P1（若进生产敏感配置） |
| 热更新 / 背压 / shutdown | 无后台 watcher（符合当前合同） | DEFER |
| 配置校验 | 无结构化校验 fail-fast | 生产配置路径 P0 若依赖本 crate 作唯一配置源 |

#### 七维速评

| 维 | 评 |
|----|----|
| 正确性 | 合同内强（含 poison / 并发 smoke） |
| 契约完整性 | 仅 KV；无版本化配置快照 |
| 兼容性 | 0.1.0 最小 API |
| 可运维性 | 写失败可分类；读中毒与缺失不可区分（文档已写） |
| 安全性 | 明文 String；无边界 |
| 可验证性 | unit + `public_api` + concurrency + example |
| 治理 | README/SSOT 诚实；错误英文 |

#### 判定

- **内存 KV 合同**：**L1 Internal Ready（合同内）**  
- **配置系统 / 生产配置面**：**未就绪**  
- STATUS **90%** vs 生产：落差在「把进度条当成配置平台完成度」

---

### 2.3 evidence（`crates/evidence` · package `evidence`）

#### 能力面

- `EvidenceAppender::append_named` → `AppendReceipt { name, seq }`
- `EvidenceError::{DurabilityFailure, Unavailable}`
- `InMemoryEvidenceAppender`：递增 seq、`fail_next`、`close`
- std-only；bootstrap 注入默认

#### 缺口

| 类别 | 说明 | 严重度 |
|------|------|--------|
| 持久化 | **仅内存**；进程退出即失 | P0（审计证据生产路径） |
| panic | `fail_next` / `close` / `names` / `len` 使用 `lock().expect`；`append_named` 对 poison 映射 Unavailable | P1（观测/测试钩子路径） |
| wire | 无远程/签名链/完整 AppendRequest | DEFER（文档明确） |
| 错误语言 | 英文 Display | P2 |

#### 判定

- **开发默认注入面**：可用  
- **生产审计总线**：**未就绪**  
- STATUS **88%** 与「最小面 + 测试」一致，**不等于**证据系统完成

---

### 2.4 observex（`crates/observex` · package `observex`）

#### 能力面

- `TracingInstrumentation`：`record_retry` / `record_circuit_open` / `record_circuit_close` → `tracing::info!`
- 实现 `contracts::Instrumentation`；别名 `ObservexInstrumentation`
- 无 subscriber 不 panic；`Copy` 零字段
- 依赖：kernel（信封）+ contracts + tracing

#### 缺口

| 类别 | 说明 | 严重度 |
|------|------|--------|
| 导出 | **无** OTEL exporter / metrics / 采样 / 有界缓冲 / flush / shutdown | P0（生产可观测栈） |
| 基数 | `op` 字符串无受控集合 | P1 |
| 生命周期 | 无 flush；进程退出依赖 tracing subscriber | DEFER |

#### 判定

- **ADR-005 最小 tracing 面**：**L1 Internal Ready（声明面）**  
- **生产可观测平台**：**未就绪**  
- STATUS **88%** 合理反映「薄实现 + 测全」

---

### 2.5 resiliencx（`crates/resiliencx` · package `resiliencx`）

#### 能力面

| 能力 | 行为要点 |
|------|----------|
| 重试 | `retry_fn` / `retry_fn_with_wait`；仅 `is_retryable`；`Backoff` + 确定性 jitter；可注入 `Wait` |
| 熔断 | 三态；**拒绝计数**推进 Open→HalfOpen（**无墙钟**）；配置阈值校验 |
| 限流 | 令牌桶；**显式 `refill`**（不按时间自动补） |
| 舱壁 | 并发上限；满载立即 `Unavailable`；RAII permit |
| 可观测 | 只依赖 `contracts::Instrumentation`；**禁止**直接依赖 observex |

#### 缺口

| 类别 | 说明 | 严重度 |
|------|------|--------|
| async | 默认 `ThreadSleepWait` **阻塞线程**；无 async wait / tokio sleep 合同 | P0（async 服务默认路径） |
| 墙钟语义 | 熔断冷却/限流 refill 非时间驱动——生产需调用方自管时钟 | P1（语义差异，非 bug） |
| retry budget | 未实现 | P1 |
| panic | `Bulkhead::in_flight` 用 `expect`；`try_enter` 已 map poison；`RecordingWait` 测试用 expect | P2 |
| 组合 | 与 transport 超时/429 未内建策略编排（边界正确，需上层组合） | — |

#### 七维速评

| 维 | 评 |
|----|----|
| 正确性 | 状态机与配置校验扎实；单测+合同测丰富 |
| 契约完整性 | 同步面完整；async/budget 缺口 |
| 兼容性 | 0.1.0；无 wire |
| 可运维性 | 经 Instrumentation 打点；错误 ErrorKind 清晰 |
| 安全性 | 有界舱壁/限流；无无界重试（受 max_attempts） |
| 可验证性 | 最强（retry_contract + public_api + 多模块 unit） |
| 治理 | 文档诚实；依赖 R3 正确 |

#### 判定

- **同步弹性原语**：**接近 L1 Internal Ready**（L1 七模块中语义最厚）  
- **async 生产弹性框架**：**未就绪**（sleep/async wait/budget）  
- STATUS **98%** 与实现厚度匹配度最高；仍须防「98% = 生产 async 弹性完成」误读

---

### 2.6 schedulex（`crates/schedulex` · package `schedulex`）

#### 能力面

- `Scheduler::{new, schedule, cancel, list, Default}`
- 内部 `HashMap<String, ()>`；std-only；**无** Clock/timer/Job/async

#### 缺口

| 类别 | 说明 | 严重度 |
|------|------|--------|
| 调度语义 | **不是**定时器；登记 ≠ 触发执行 | 设计诚实；生产调度 **P0 缺口** |
| 并发 | 非 `Sync` 共享（`&mut self` API）；无锁 | 多线程调度不适用 |
| 关停 / misfire / lease | 均未实现（SSOT §3 禁止） | DEFER |

#### 判定

- **ID 登记合同**：**L1 Internal Ready（极小面）**  
- **生产调度器**：**未就绪（明确非目标）**  
- STATUS **88%** 对「登记表」合理；对「scheduler」名严重误导风险 → 文档已警告，消费方仍易误用

---

### 2.7 transportx（`crates/transport` · package `transportx`）

#### 能力面

| 能力 | 证据 |
|------|------|
| HTTP 边界 | `HttpRequest/Response` + `HttpDriver` |
| WS 边界 | `WsConnector` / `WsConnection` 帧级 |
| 真实驱动 | `ReqwestHttpDriver`、`TungsteniteWsConnector`（类型 private） |
| 错误语义 | `TransportError`：Connect/ReadTimeout、ConnectionClosed、RateLimited(Retry-After)、Protocol、Io |
| 行为 | 429→RateLimited；其他 4xx/5xx→Ok(status,body)；WS Ping/Pong 跳过 |
| Mock | `MockHttpTransport` 双实现 legacy+Driver |
| 测试 | mock 18 + reqwest 14 + websocket 9（loopback） |

#### 缺口

| 类别 | 说明 | 严重度 |
|------|------|--------|
| 超时默认 | `new()` → `with_timeout(None)`：**未强制**本 crate 默认 deadline（依赖 reqwest 默认） | P1 |
| 资源边界 | **无** body size limit / absolute multi-phase deadline（SSOT 4.4/5.5 DEFER） | P1 |
| WS 连接超时 | `connect_async` 无显式 connect timeout 封装 | P1 |
| TLS / 池 / 代理 | M3 DEFER；不可宣称生产 TLS 矩阵 | P1 |
| Mock panic | `set_get`/`set_post` 与 legacy GET/POST 用 `expect` 锁 | P2（mock 路径） |
| 错误语言 | thiserror 英文 | P2 |
| 重试 | **故意**不内建（归 resiliencx）— 组合责任在上层 | — |

#### 真实 I/O 边界

- **是** L1 中唯一带生产网络栈的 crate（reqwest + tungstenite）。  
- 适配器（binancex/okxx）可注入 `HttpDriver`；业务解析仍 DEFER。  
- 测试以 loopback 为主，符合「外网 ignore」约定。

#### 判定

- **客户端传输抽象 + 默认驱动**：**有条件 L1 Internal Ready**  
- **平台/TLS/资源矩阵**：**未到 L4**  
- STATUS **95%** 较接近实现实质；仍非「生产网络栈签字」

---

## 3. 跨模块依赖与组合风险

```text
                    contracts::Instrumentation
                              ▲
              ┌───────────────┼────────────────┐
              │               │                │
         observex        resiliencx         bootstrap
      (TracingImpl)   (消费 trait)     (默认注入 Tracing)
              │               │                │
              └───────────────┴───────┬────────┘
                                      │
                                 evidence
                              (可选注入 bootstrap)

configx ──仅──► kernel
schedulex ──std only──
transportx ──仅──► kernel + 网络信封（R3 不依赖其他 L1）
```

| 风险 | 说明 | 级别 |
|------|------|------|
| **假关停** | bootstrap 只发 `ShutdownSignal`；transport 连接、resiliencx sleep、schedulex「任务」均无统一 drain | P0 组合 |
| **假配置** | 若 app 只靠 configx 内存 KV，无 fail-fast 校验 → 带病运行 | P0 组合 |
| **假证据** | 默认 `InMemoryEvidenceAppender` 不可作合规审计 | P0 组合 |
| **假观测** | 仅 info 事件；无导出/关联 trace id 强制 | P1 组合 |
| **弹性×传输未编排** | 429/timeout 在 transport；重试在 resiliencx；默认阻塞 sleep | P1 组合 |
| **熔断无墙钟** | 生产若假设「N 秒冷却」会误配；须按拒绝次数或外层时钟 | P1 语义 |
| **限流无自动 refill** | 调用方忘记 refill → 永久限流 | P1 语义 |
| **调度名陷阱** | schedulex 名像调度器，实为 ID set | P0 误用 |
| **require_evidence release** | 生产用 `build()` 而非 `try_build()` 可绕过 | P0 |
| **Bounded* vs contracts** | 命名已收敛（Bounded 前缀），但注入后仍无业务能力 | P1 |
| **错误语言统一** | L1 用户可见错误普遍英文，与仓库中文错误规范冲突 | P2 治理 |

**组合根诚实结论**：bootstrap 能「装起来」，不能单独证明「能关干净、可审计、可观测导出、可配置校验、可调度执行」。

---

## 4. P0 / P1 清单（L1 平面）

### P0（生产路径阻断或高误导）

| ID | 项 | 模块 |
|----|----|------|
| L1-P0-1 | `Bootstrap::build` / `build_app` 在 release 不强制 `require_evidence` | bootstrap |
| L1-P0-2 | evidence 仅内存，无耐久/远程；不得作生产审计落盘 | evidence |
| L1-P0-3 | configx 无 schema/多源/校验；不得作唯一生产配置源 | configx |
| L1-P0-4 | schedulex 无定时执行；禁止当 production scheduler | schedulex |
| L1-P0-5 | resiliencx 默认 `thread::sleep`；async runtime 路径未交付 | resiliencx |
| L1-P0-6 | observex 无 OTEL/flush；不得宣称生产可观测完成 | observex |
| L1-P0-7 | 组合无统一 graceful drain（信号有，编排无） | 跨模块 |

### P1（有条件使用须知 / 硬化项）

| ID | 项 | 模块 |
|----|----|------|
| L1-P1-1 | transport 默认超时未本 crate 强制；WS connect 超时/体大小限 DEFER | transportx |
| L1-P1-2 | 熔断/限流无墙钟；生产需外层时钟或显式 refill | resiliencx |
| L1-P1-3 | retry budget / async wait | resiliencx |
| L1-P1-4 | Bounded* 升级为 contracts 生产 trait 注入 | bootstrap |
| L1-P1-5 | evidence 观测路径 `expect` 锁；统一 poison→Error | evidence / bulkhead |
| L1-P1-6 | secret 配置与脱敏 | configx |
| L1-P1-7 | 用户可见错误中文化 | 全 L1 |
| L1-P1-8 | API/semver 棘轮、平台矩阵（L4） | 全 L1 |

---

## 5. STATUS 完成度 vs 生产级落差

| 模块 | STATUS | STATUS 在量什么 | 生产级实际 | 落差一句话 |
|------|--------|-----------------|------------|------------|
| bootstrap | 100% | 布局+测+厚组合根代码 | 装配骨架有；生命周期/真依赖薄 | **进度条封顶 ≠ 可上线 app 根** |
| configx | 90% | 内存 KV 合同齐 | 非配置中心 | **差一个「产品」** |
| evidence | 88% | trait+内存+测 | 非证据系统 | **差持久化与信任** |
| observex | 88% | 三方法+测 | 非观测平台 | **差导出与生命周期** |
| resiliencx | 98% | 四原语+合同测 | 同步面近就绪；async 缺口 | **落差最小但仍非 async 生产** |
| schedulex | 88% | 登记表五测 | 零调度能力 | **名实落差最大** |
| transportx | 95% | 真驱动+loopback 测 | 传输可用；平台矩阵缺 | **最接近真 I/O，缺平台硬化** |

### 汇总判语

1. **七个 L1 模块均不应整体标 Production Ready / package stable。**  
2. 相对 **各自 active SSOT 最小面**：configx / schedulex / observex / evidence / bootstrap 多为 **合同内 PASS**；resiliencx / transportx 语义更接近 **可用基础设施原语**。  
3. 相对 **生产运行时**（配置、审计、观测、调度、关停编排）：**全部未闭合**。  
4. STATUS 是 **结构可观测进度**，本报告七维才是 **生产语义**；二者相关但不可互换。

---

## 6. 测试证据（本会话抽样）

命令（注意：Cargo **package 名**为短名 `bootstrap` 等，非 `xhyper-*`）：

```bash
cargo test \
  -p bootstrap -p configx -p evidence -p observex \
  -p resiliencx -p schedulex -p transportx \
  --all-targets
```

| Package | 结果（摘要） |
|---------|----------------|
| bootstrap | unit 31 + 集成 6 + example 0 → **ok** |
| configx | unit 8 + concurrency 1 + public_api 5 → **ok** |
| evidence | unit 5 + public_api 2 → **ok** |
| observex | unit 8 + public_api 3 → **ok** |
| resiliencx | unit 26 + public_api 6 + retry_contract 13 → **ok** |
| schedulex | unit 5 + public_api 2 → **ok** |
| transportx | mock 18 + reqwest 14 + websocket 9 → **ok** |

> 全绿证明 **声明合同与实现一致**，不证明 **生产语义闭合**（与 core 审计 §1.2 同理）。

---

## 7. 建议使用矩阵（给集成方）

| 场景 | 建议 |
|------|------|
| 单测 / harness 组合 | ✅ bootstrap + Noop/Tracing + InMemory evidence |
| 同步批处理中的重试/舱壁 | ✅ resiliencx（显式 Wait；知悉无墙钟） |
| HTTP mock / loopback 客户端 | ✅ transportx Mock / Reqwest |
| 生产配置加载 | ❌ 勿只靠 configx；另建校验层或等多源合同 |
| 生产审计落盘 | ❌ 勿用 InMemory evidence |
| 生产 OTEL | ❌ 勿宣称 observex 完成 |
| 定时任务执行 | ❌ 勿用 schedulex |
| async 服务默认重试 | ⚠️ 勿直接 `retry_fn`（会 block）；需 async Wait 或外层 |
| 关停 | ⚠️ 消费 `ShutdownSignal` 后须自建 drain |

---

## 8. 追溯

| 资源 | 路径 |
|------|------|
| STATUS | `/home/workspace/infra.rs/STATUS.md` |
| 七维 / 分层 | `docs/report/2026-07-21/core-crates-production-readiness.md` · `docs/plans/2026-07-21-core-crates-production-readiness.md` §2 |
| SSOT 对齐 | `docs/ssot/{bootstrap,configx,evidence,observex,resiliencx,schedulex,transport}-ssot-alignment.md` |
| 实现 | `crates/{bootstrap,configx,evidence,observex,resiliencx,schedulex,transport}/` |

---

*本文件为 modules-prod-audit 的 L1 partial；可与 L0/types/contracts/adapters partials 汇总为完整 STATUS 生产落差报告。*
