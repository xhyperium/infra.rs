# STATUS.md 全模块生产级就绪度深度审计报告

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| 输入看板 | 根目录 [`STATUS.md`](../../../STATUS.md)（生成 2026-07-21T11:24:53Z，平均完成度 **92%**） |
| 审计范围 | workspace **21** 成员（L0 / T0 / types / L1 / contracts / adapters） |
| 被审计快照 | `9174840`（`main` / `fix(hooks): address Codex P2 on nice/timeout gate (#155)`） |
| 执行方式 | **Agent Team** 六路并行：L0 · types · L1 · contracts · adapters · workspace 质量基线 |
| 报告性质 | 只读生产就绪度审计；**不是**发布批准、package stable 或 Production Ready 签字 |
| 关联计划 | [core-crates 生产修复方案](../../plans/2026-07-21-core-crates-production-readiness.md) · [核心五件套审计](./core-crates-production-readiness.md) |
| Follow-up | [status-modules-prod-followup.md](../../plans/2026-07-21-status-modules-prod-followup.md) · Beads epic **`infra-s9t`** |
| partial 证据 | [`_partials/`](./_partials/)（分平面明细，本文件为合成主报告） |
| 二次深审 | [workspace-production-readiness.md](./workspace-production-readiness.md)（增量正确性、安全、资源与治理阻断） |

---

## 0. 结论先行（Executive Summary）

### 0.1 一句话

**STATUS 平均 92% 与工程门禁全绿，不能推出「可上生产应用」。**  
完成度衡量的是**布局·测试·源码厚度**；生产级要求的是**语义闭合·真实后端·运维闭环·分层签字**。二者相关但不可互换。

### 0.2 总判定

| 层级平面 | 模块数 | 可作生产应用依赖？ | 说明 |
|----------|-------:|--------------------|------|
| L0 kernel | 1 | **有条件（库语义）** | L1 Internal + L4 Platform（声明范围内） |
| T0 testkit | 1 | **仅测试支持** | 明确非生产 runtime |
| types | 2 | **有条件（内部/wire 子集）** | decimalx L1；canonical L2 committed 子集 |
| L1 infra | 7 | **整体否** | 各有合同内就绪面；无一达运行时平台级 |
| contracts | 1 | **L3 子集；first-batch 否** | KV+Instr 有真入口；Tx/Bus/Venue 业务未 L3 |
| adapters | 9 | **默认否** | scaffold/mock；**例外** redis live KV + exchange 只读 time |
| **workspace 整体** | **21** | **否** | 禁止宣称整体 Production Ready |

### 0.3 STATUS 成熟度 vs 生产判定（全表）

| Package | STATUS 成熟度 | STATUS 完成度 | 建议目标层 | 生产判定 | 可安全使用范围（摘要） |
|---------|---------------|--------------:|------------|----------|------------------------|
| `kernel` | active | 98% | L1 + L4 | **有条件就绪** | L0 错误 / 时钟 / 关停原语 |
| `testkit` | active | 98% | L1（test-support） | **有条件就绪（测试）** | ManualClock；非 runtime |
| `decimalx` | active | 98% | L1 | **有条件就绪（内部）** | `try_new` + 仅 `checked_*` 资金路径 |
| `canonical` | active | 98% | L2 子集 | **部分就绪** | committed wire v1–v1.3 DTO |
| `bootstrap` | active | 100% | L1 | **有条件（装配）** | 进程内组合根；非完整 app 生命周期 |
| `configx` | partial | 90% | L1 内存合同 | **合同内就绪** | 进程内 String KV；非配置中心 |
| `evidence` | partial | 88% | L1 内存 | **开发默认可用** | 注入探测；非持久化审计 |
| `observex` | partial | 88% | L1 tracing 面 | **最小面就绪** | 重试/熔断 info 打点；无 OTEL |
| `resiliencx` | active | 100% | L1 | **接近 Internal Ready** | 同步弹性 + `retry_async`；budget 仍 DEFER |
| `schedulex` | partial | 88% | L1 registry | **登记合同就绪** | 任务 ID 集合；**不是**调度器 |
| `transportx` | active | 95% | L1 + 部分 I/O | **有条件就绪** | HTTP/WS 客户端边界；平台矩阵未闭合 |
| `contracts` | active | 100% | L3 | **L3 子集**（KV+Instr） | trait + Fake/conformance；redis live + observex；非 first-batch 全绿 |
| `binancex` | scaffold+mock | 89% | — | **不可用** | mock/trait 形状；无真实交易所 |
| `okxx` | scaffold+mock | 89% | — | **不可用** | 同上 |
| `postgresx` | scaffold+mock | 89% | — | **不可用** | 内存 + commit 边界 mock |
| `redisx` | scaffold+mock | 89% | — | **KV live 有条件** | feature `live` + conformance；默认 scaffold 仍非生产 |
| `kafkax` | scaffold+mock | 89% | — | **不可用** | 内存 EventBus |
| `natsx` | scaffold+mock | 88% | — | **不可用** | 同上 |
| `clickhousex` | scaffold | 83% | — | **不可用** | pure 内存 sink |
| `ossx` | scaffold | 83% | — | **不可用** | pure 内存 ObjectStore |
| `taosx` | scaffold+mock\* | 88% | — | **不可用** | pure 内存时序（无独立 Mock 类型） |

\* STATUS 标 `scaffold+mock`，源码更接近 pure scaffold（对齐文为准）。

### 0.4 工作区工程基线（本会话实测）

| 门禁 | 结果 |
|------|------|
| `cargo test --workspace --all-targets` | **522 passed / 0 failed**（54 suites） |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **0 warning** |
| `cargo fmt --all -- --check` | **通过** |
| `cargo deny check` | **通过**（Zlib allowance 未命中提示，非阻断） |

> 门禁绿 = 可编译、可测、可 lint、供应链策略通过。  
> **≠** 业务语义完整、真实后端可用、可签 Production Ready。

---

## 1. 审计方法与判据

### 1.1 Agent Team 分工

| Agent | 平面 | 产出 partial |
|-------|------|--------------|
| A1 | L0 kernel + testkit | [`_partials/l0-kernel-testkit.md`](./_partials/l0-kernel-testkit.md) |
| A2 | types decimalx + canonical | [`_partials/types-decimal-canonical.md`](./_partials/types-decimal-canonical.md) |
| A3 | L1 七模块 | [`_partials/l1-infra.md`](./_partials/l1-infra.md) |
| A4 | contracts | [`_partials/contracts.md`](./_partials/contracts.md) |
| A5 | adapters ×9 | [`_partials/adapters.md`](./_partials/adapters.md) |
| A6 | workspace 质量基线 | [`_partials/workspace-baseline.md`](./_partials/workspace-baseline.md) |

主报告由 Lead 合成；判定以源码 + 本会话 `cargo test` 证据为准，并对照既有 core 审计 / SSOT / 生产计划。

### 1.2 生产七维

| # | 维度 | 问题 |
|---|------|------|
| 1 | **正确性** | 公开可达输入是否触发未声明 panic、静默错误或不一致状态？ |
| 2 | **契约完整性** | 失败、取消、超时、背压、事务、确认、顺序、幂等是否可表达？ |
| 3 | **兼容性** | API / wire 是否有版本、演进、拒绝与回滚策略？ |
| 4 | **可运维性** | 错误是否可分类追踪？关停 / 并发 / 失败是否有门禁？ |
| 5 | **安全性** | 反序列化、资源消耗、密钥/PII、TLS、依赖风险是否有边界？ |
| 6 | **可验证性** | 测试是否覆盖真实合同，而非仅行覆盖或自往返？ |
| 7 | **治理合规** | 中文错误、lint、MSRV、文档诚实、依赖方向 R3？ |

### 1.3 分层 Production Ready（本仓采用）

| 层级 | 含义 | 签字条件（摘要） |
|------|------|------------------|
| **L1 Internal Ready** | 进程内库；非法状态可控；checked 路径无 panic；持续 CI | 模块 owner |
| **L2 Wire Ready** | 跨进程/落盘类型有版本、兼容矩阵、拒绝策略 | 模块 + wire owner |
| **L3 Contract Ready** | trait 语义合同 + conformance + **至少一非 scaffold 验证入口** | contracts + adapter owner |
| **L4 Platform Ready** | OS/MSRV 矩阵声明并实测；API/semver 门禁 | platform owner |
| **L5 Release Ready** | 人签 + DEFER 冻结 + CHANGELOG 发布说明 | maintainer |

**整体 Production Ready** = 各模块达到**各自声明目标层**，且 workspace 级 L5 对「应用可交付面」签字完成。  
当前：**不满足。**

### 1.4 关键命名事实（避免误用命令）

| 叙事名（文档/SSOT 常见） | 当前 Cargo package 名（`-p` 权威） |
|--------------------------|-------------------------------------|
| xhyper-kernel | **`kernel`** |
| xhyper-testkit | **`testkit`** |
| xhyper-decimalx | **`decimalx`** |
| xhyper-canonical | **`canonical`** |
| xhyper-contracts | **`contracts`** |
| xhyper-bootstrap 等 | **`bootstrap`** / `configx` / …（短名） |

`cargo test -p xhyper-kernel` 在 HEAD 上会 **package not found**。属 P1 文档漂移，不阻断语义，但损害可复现性。

### 1.5 STATUS 公式（提醒）

```text
completion = layout(8项)×50% + has_tests×25% + content×25%
scaffold   → content 上限 0.55
```

STATUS 自声明：**不是** Production Ready 签字，也不是 SSOT COMPLETE。

---

## 2. 分平面深度结论

### 2.1 L0 / T0 — kernel · testkit

**判定：声明范围内有条件就绪。**

| 模块 | 目标层 | 状态 |
|------|--------|------|
| kernel | L1 + L4 | 原报告 ClockDomain / loom CI / `wait_timeout` / 中文错误 **已闭合**；本会话 `cargo test -p kernel -p testkit --all-targets` 全绿 |
| testkit | L1 test-support | ManualClock 四类型闭合；**禁止**当生产 runtime |

仍须知悉：

- 无 Component 编排 / 健康检查（刻意）
- 异步组合根需自适配 `ShutdownSignal`（无 tokio 依赖）
- contract-testkit **不在** testkit（在 contracts 最小 Fake）
- P1：package 名文档漂移、CHANGELOG 与 0.3.0 签核对齐、examples/benches 多为 `.gitkeep`

明细：[`_partials/l0-kernel-testkit.md`](./_partials/l0-kernel-testkit.md)

---

### 2.2 types — decimalx · canonical

**判定：相对 core 审计 §1「未就绪」已过时；以 L1 / L2 子集与 §12 为准。**

| 模块 | 目标层 | 状态 |
|------|--------|------|
| decimalx | L1 Internal | 字段私有 + 校验 serde + `DecimalError` 中文；非法 scale 不可表示；资金路径须只用 `checked_*` |
| canonical | L2 Wire（committed） | `COMMITTED_WIRE_V1`…`V1_3` 覆盖公开市场 DTO；`deny_unknown_fields`；**无** wire envelope |

仍阻断 / 缺口：

| ID | 项 |
|----|-----|
| B-D1 | decimal wire **非**跨版本 stable（`docs/WIRE.md`） |
| B-D2 | panicking `+/-/*` 仍公开（靠门禁非类型消灭） |
| B-C1 | 无 schema_version envelope |
| B-C2 | `shape::*` 不自动挂在 Deserialize |

测试：`cargo test -p decimalx -p canonical --all-targets` → **144 passed**。

明细：[`_partials/types-decimal-canonical.md`](./_partials/types-decimal-canonical.md)

---

### 2.3 L1 infra — 七模块

**判定：七模块整体不可标 Production Ready；相对各自 active SSOT 最小面多数合同对齐。**

| 模块 | 实质 | 生产判定 | 最大陷阱 |
|------|------|----------|----------|
| bootstrap | typed 组合根 + 关停信号 | Internal **有条件** | `require_evidence` **release fail-closed**（#168；`build` panic） |
| configx | 内存 String KV | **合同内就绪** | 当成配置中心 / 唯一生产配置源 |
| evidence | 内存 appender | 开发默认可用 | 当成合规审计落盘 |
| observex | tracing 三方法 | 最小面就绪 | 宣称 OTEL/生产可观测完成 |
| resiliencx | 重试/熔断/限流/舱壁 | **最接近** L1 Internal | 默认 `thread::sleep`；熔断无墙钟 |
| schedulex | 任务 **ID 登记表** | 登记合同 OK | **名实落差最大**——不是定时调度器 |
| transportx | 真 HTTP/WS 驱动 | 传输边界有条件 | TLS/体限制/强制超时未闭合 |

L1 平面 P0（节选）：

1. ~~bootstrap release 不强制 evidence~~ **已修**（#168 fail-closed）  

2. evidence 仅内存 / configx 无 schema / schedulex 无执行 / observex 无导出  
3. resiliencx 默认阻塞 sleep（async 服务路径）  
4. **跨模块：有关停信号，无统一 drain 编排**

明细：[`_partials/l1-infra.md`](./_partials/l1-infra.md)

---

### 2.4 contracts

**判定：L3 子集（KV+Instr）已闭合；first-batch 整体未达 L3 Contract Ready。**

| L3 条件 | 状态 |
|---------|------|
| 语义合同 | **部分**（first-batch 11 篇；ObjectStore/TimeSeries/PubSub/Analytics 等无独立文档） |
| conformance suite | **部分**（KV/Repository/Tx/EventBus/Instrumentation） |
| 非 scaffold 验证入口 | **部分 PASS**（`redisx::RedisLiveKv` + observex；Tx/Bus/Repo/Venue 业务仍 scaffold） |

**公开面**：15 trait + `BusMessage`/`MessageAck` + `run_tx_commit_on_ok` + Fake/Recording。

**已闭合**：Tx 对象安全、消息带 ID、Fake、venue override 门禁、bootstrap `Bounded*`、**redis live KV**、**L3 子集文档**（#168/#172）。

**主阻断（相对 first-batch 全绿）**：Tx/Bus/Venue 业务无真入口；二期 storage trait；无 compile-fail override / API snapshot 机控。

测试：`cargo test -p contracts --all-targets`；live：`cargo test -p redisx --features live --test live_kv_conformance -- --ignored`。

明细：[`_partials/contracts.md`](./_partials/contracts.md) · [`L3_FIRST_BATCH_STATUS.md`](../../../crates/contracts/docs/L3_FIRST_BATCH_STATUS.md)

---

### 2.5 adapters（9 包）

**判定：9/9 均不可作为生产应用对真实后端的依赖。**

| 分层 | 包 | 深度 |
|------|-----|------|
| Exchange | binancex, okxx | contracts 齐 + 可选 HttpDriver mock；**无**签名/协议/真下单 |
| Storage first-batch | postgresx, redisx, kafkax, natsx | 进程内 mock 验证入口（commit / TTL / BusMessage id） |
| Storage pure | clickhousex, ossx, taosx | HashMap/Vec 内存桩 |

**横切**：Cargo.toml **无** sqlx/redis/rdkafka/async-nats 等真实 SDK；无认证；无 TLS/连接池配置面；无 adapter 级重试。

**最危险误用**：类型名像生产客户端（`RedisAdapter` / `PostgresAdapter`），行为是测试桩。

明细：[`_partials/adapters.md`](./_partials/adapters.md)

---

## 3. 跨平面组合风险（生产应用视角）

若有人将 STATUS 高分 crate **直接拼成「生产应用」**，典型失败模式：

```text
应用
  ├─ bootstrap.build()          → require_evidence 时 release panic（#168）
  ├─ configx 唯一配置源         → 无 schema fail-fast
  ├─ evidence InMemory          → 审计空心
  ├─ observex Tracing only      → 无可导出管道
  ├─ resiliencx.retry_fn        → 默认阻塞线程
  ├─ schedulex.schedule         → 仅登记 ID，从不触发
  ├─ redisx / postgresx         → HashMap，非真实存储
  └─ binancex.place_order       → 本地占位 Ack，非交易所
```

| 风险 ID | 描述 | 严重度 |
|---------|------|--------|
| COMP-1 | 假后端：adapter 名像生产、实为内存 | **P0** |
| COMP-2 | 假配置 / 假证据 / 假调度 | **P0** |
| COMP-3 | 有关停信号、无 drain 编排 | **P0** |
| COMP-4 | contracts L3 未过却依赖 dyn trait 上线 | **P0** |
| COMP-5 | 弹性×传输未编排（429 / sleep / 超时分层） | P1 |
| COMP-6 | 资金路径误用 panicking 运算符 | P1（有门禁） |
| COMP-7 | 用户可见错误大量英文 vs 中文规范 | P2 治理 |

---

## 4. 阻断项与改进路线图

### 4.1 Workspace 级 P0（禁止「生产应用就绪」宣称）

| ID | 项 | 平面 |
|----|-----|------|
| WS-P0-1 | ~~无任何~~ **部分闭合**：redis live KV；Tx/Bus/Venue **业务** live 仍缺 | adapters + contracts L3 |
| WS-P0-2 | 生产配置 / 审计 / 调度 / OTEL 平台面未交付 | L1 |
| WS-P0-3 | 组合根无统一 graceful drain | L1 跨模块 |
| WS-P0-4 | contracts 二期 trait 与 Venue 真实路径未闭合 | contracts |
| WS-P0-5 | 整体 L5 对「应用可交付面」未签 | 治理 |

### 4.2 建议优先级（与既有 W 波次对齐并扩展）

权威任务树与勾选清单：[`docs/plans/2026-07-21-status-modules-prod-followup.md`](../../plans/2026-07-21-status-modules-prod-followup.md) · epic **`infra-s9t`**。

```text
P0-A  冻结「可生产消费面」清单     → infra-s9t.1  (W0+)
P0-B  W4：至少一个真实后端入口     → infra-s9t.2  (W4)
P0-C  bootstrap require_evidence   → infra-s9t.4  (L1)
P0-D  scaffold ≠ 客户端警示        → infra-s9t.14
P1-A  resiliencx async Wait        → infra-s9t.6
P1-B  evidence / configx 边界      → infra-s9t.7
P1-C  decimal/canonical wire       → infra-s9t.9 / .10
P1-D  package 名文档对齐           → infra-s9t.11
P2    中文错误 / OTEL / 名实       → infra-s9t.8 / .12
```

### 4.3 分层签字建议（现状）

| 模块 | 可立即讨论签字？ | 条件 |
|------|------------------|------|
| kernel | **是（L1+L4）** | 维持 Accept：Linux x86_64、publish=false |
| testkit | **是（L1 test-support）** | 禁止 runtime 叙事 |
| decimalx | **是（L1 有条件）** | 资金路径 checked-only + wire 非 stable 声明 |
| canonical | **是（L2 子集）** | 仅 committed 清单；无 envelope Accept |
| resiliencx | **接近（L1 同步）** | 排除 async 默认路径 |
| transportx | **有条件（L1 I/O）** | 超时/TLS Accept 列表 |
| bootstrap / configx / evidence / observex / schedulex | **合同内 yes / 产品 no** | 严格按 SSOT 最小面 |
| contracts | **L3 子集 yes / first-batch 否** | KV+Instr 已闭合；业务 live 另战役 |
| adapters ×9 | **默认否** | redis live + 只读 time 有条件；业务 SDK 另战役 |

---

## 5. 与既有产物的关系

| 文档 | 关系 |
|------|------|
| [core-crates-production-readiness.md](./core-crates-production-readiness.md) | 五件套深度审计；本报告 **扩展至 STATUS 全部 21 模块**；types/kernel/contracts 结论与 HEAD 对齐后 **§1「未就绪」部分过时，以分层签字为准** |
| [plans/2026-07-21-core-crates-production-readiness.md](../../plans/2026-07-21-core-crates-production-readiness.md) | 生产定义与 W0–W5；本报告 L1/adapters 缺口为 **计划外扩展面**，建议 follow-up epic |
| `docs/ssot/*-alignment.md` | 镜像 COMPLETE ≠ 本仓 ship；本报告与对齐文一致：**adapters 未宣称业务实现** |
| `STATUS.md` | 结构进度输入；本报告输出 **生产语义判定** |

---

## 6. 证据索引

### 6.1 本会话命令（可复现）

```bash
# 质量基线
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo deny check

# 分平面抽样（注意短 package 名）
cargo test -p kernel -p testkit --all-targets
cargo test -p decimalx -p canonical --all-targets
cargo test -p bootstrap -p configx -p evidence -p observex \
  -p resiliencx -p schedulex -p transportx --all-targets
cargo test -p contracts --all-targets
cargo test -p binancex -p okxx -p redisx -p postgresx -p kafkax --all-targets
```

### 6.2 Partial 文件

| 文件 | 行数（约） | 内容 |
|------|-----------:|------|
| [`_partials/workspace-baseline.md`](./_partials/workspace-baseline.md) | 178 | 522 tests / clippy / fmt / deny |
| [`_partials/l0-kernel-testkit.md`](./_partials/l0-kernel-testkit.md) | 257 | kernel/testkit 七维 + checklist |
| [`_partials/types-decimal-canonical.md`](./_partials/types-decimal-canonical.md) | 279 | 金额不变量 + wire 矩阵 |
| [`_partials/l1-infra.md`](./_partials/l1-infra.md) | 445 | 七 L1 模块逐项 |
| [`_partials/contracts.md`](./_partials/contracts.md) | 332 | 15 trait + L3 对照 |
| [`_partials/adapters.md`](./_partials/adapters.md) | 255 | 9 adapter 总表 |

### 6.3 未在本会话重跑（边界声明）

- llvm-cov / miri / mutants 全量（以 CI schedule / 既有签核证据为准）
- loom（需 `RUSTFLAGS='--cfg loom'`；kernel 有专用 workflow）
- 外网 live 集成测试（仓库约定默认 ignore / DEFER）

---

## 7. 禁止与允许的表述

### 禁止

- 「STATUS 92% / 某 crate 98% ⇒ Production Ready」
- 「adapters 可对接真实 Binance/Postgres/Redis」
- 「contracts 已 L3 Contract Ready」
- 「schedulex 可作生产定时调度器」
- 「evidence / observex 完成生产审计 / OTEL」
- 「workspace 整体 Production Ready / package stable」

### 允许（有条件）

- 「kernel L1+L4 有条件就绪（L0 语义库）」
- 「testkit ManualClock 测试支持 L1」
- 「decimalx L1 内部就绪（checked 纪律）」
- 「canonical L2 committed 子集 wire 就绪」
- 「resiliencx 同步弹性原语接近 L1」
- 「transportx 客户端传输有条件可用」
- 「工程门禁（test/clippy/fmt/deny）全绿」
- 「adapters 适合 contracts 形状验证与 mock 编排」

---

## 8. 签字清单（生产应用视角 · 未完成）

以下全部勾选前，**不得**宣称「infra.rs **workspace** 满足生产级应用标准」。  
（`infra-s9t` 子项多已 DONE；清单保留为**应用面**视角。）

- [x] W0：生产消费面冻结（`prod-consume-surface.md` · s9t.1）
- [x] 至少一个 **非 scaffold** 验证入口（redis live · s9t.2 · #168）
- [ ] contracts **first-batch** L3 三条件全绿（当前仅 **子集** KV+Instr）
- [x] bootstrap `require_evidence` release fail-closed（s9t.4 · #168）
- [x] 配置 / 证据 / 观测 / 调度 README 红线（s9t.7/.8/.14）
- [x] decimal 门禁 + wire 矩阵文档（s9t.9/.10）
- [x] L1 中文错误抽查（s9t.12）
- [ ] L5 maintainer 签字 + 应用面 DEFER 列表冻结（**人工**）

---

## 9. 元数据

| 项 | 值 |
|----|-----|
| 分支 / worktree | `docs/infra-status-modules-prod-audit` · `.worktrees/docs/infra-status-modules-prod-audit` |
| 合成时间 | 2026-07-21 |
| rustc（基线会话） | 1.97.0 |
| 主报告路径 | `docs/report/2026-07-21/status-modules-production-readiness.md` |

---

*本报告由 Agent Team 六路 partial 合成。分平面细节以 `_partials/` 为准；若与主表冲突，以 partial 源码证据与更新时间较新者为准，并应回写本文件。*
