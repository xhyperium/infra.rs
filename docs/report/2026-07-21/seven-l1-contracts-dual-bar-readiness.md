# 七包双标尺深度分析：STATUS 100% vs 生产级发布

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-21 |
| 范围 | `configx` · `evidence` · `observex` · `resiliencx` · `schedulex` · `transportx` · `contracts` |
| 性质 | **只读分析报告**；不是代码实现、不是 Maintainer 签核、不是 package stable |
| 输入看板 | 根 [`STATUS.md`](../../../STATUS.md)（生成器 `scripts/docs/gen-crate-status.mjs`） |
| 上游审计 | [status-modules-production-readiness.md](./status-modules-production-readiness.md) · [`_partials/l1-infra.md`](./_partials/l1-infra.md) · [`_partials/contracts.md`](./_partials/contracts.md) |
| Follow-up | [status-modules-prod-followup.md](../../plans/2026-07-21-status-modules-prod-followup.md) · Beads epic **`infra-s9t`** |
| 签核模板 | [prod-signoff-TEMPLATE.md](../../governance/prod-signoff-TEMPLATE.md)（**禁止 Agent 代签**） |
| 本会话基线 | `cargo test -p configx -p evidence -p observex -p resiliencx -p schedulex -p transportx -p contracts --all-targets` → **171 passed / 0 failed** |

---

## 0. 结论先行

### 0.1 双标尺（不可互换）

| 标尺 | 名称 | 量什么 | 权威来源 | 到达「完成」的含义 |
|------|------|--------|----------|-------------------|
| **A** | **STATUS 结构完成度** | 布局 7 项 + 是否有测试 + 内容分（LOC 桶 / example / docs） | `gen-crate-status.mjs` → `STATUS.md` | 目录与厚度达标；**不是**语义生产就绪 |
| **B** | **Production Ready（分层）** | 声明 SSOT 合同面上的正确性 / 契约 / 运维 / 安全 / 验证 / 治理 | 审计报告 L1–L5 + SSOT 对齐文 | 达到**各自声明目标层**并经人签（L5） |

```text
STATUS 100%  ⇏  Production Ready
Production Ready  ⇏  必须 STATUS 100%（但通常已接近）
```

**禁止表述**：「某 crate 98%/100% ⇒ 可生产发布」。  
**允许表述**：「某 crate 在 **声明 SSOT 最小面** 上达到 L1 Internal Ready（有条件）」。

### 0.2 一句话总判

| 标尺 | 七包现状 |
|------|----------|
| **A · STATUS 100%** | **无一达到**；最接近 resiliencx/contracts **98%**（只差 example）；最远 evidence/observex/schedulex **88%** |
| **B · 生产级发布（集合）** | **不可宣称**。声明面内：configx/schedulex 合同内 L1 可用；resiliencx 同步面接近 L1；transportx 有条件 L1 I/O；contracts **未过 L3**；evidence/observex 名实易被误用。**L5 必须人类 Maintainer**，Agent 不能签 |

### 0.3 总表（双标尺并排）

| Package | path | STATUS % | 成熟度 | 声明 SSOT 面 | Bar A → 100% 主杠杆 | 诚实最大 Bar B 层 | 生产判定 |
|---------|------|----------:|--------|--------------|---------------------|-------------------|----------|
| `configx` | `crates/configx` | **90%** | partial | 内存 `String` KV | LOC≥500（已有 example） | **L1 Internal（合同内）** | 合同就绪；非配置中心 |
| `evidence` | `crates/evidence` | **88%** | partial | trait + 内存 appender | LOC≥500 + example | **L1 开发默认** | 不可作合规审计 |
| `observex` | `crates/observex` | **88%** | partial | Tracing 三方法 | LOC≥500 + example | **L1 tracing 最小面** | 非 OTEL 平台 |
| `resiliencx` | `crates/resiliencx` | **98%** | active | 重试/熔断/限流/舱壁 | **仅 example** | **L1 同步接近 Ready** | 默认阻塞 sleep；async DEFER |
| `schedulex` | `crates/schedulex` | **88%** | partial | 任务 **ID 登记表** | LOC≥500 + example（防虚胖） | **L1 registry only** | **不是** timer/cron |
| `transportx` | `crates/transport` | **95%** | active | HTTP/WS 客户端边界 | LOC≥500（已有 example） | **L1 + 部分 I/O 有条件** | TLS/池/体限 M3 DEFER |
| `contracts` | `crates/contracts` | **98%** | active | 15 trait + Fake/conformance | **仅 example** | **未达 L3** | 缺非 scaffold 真实后端 |

---

## 1. 标尺 A — STATUS 100% 公式与路径

### 1.1 公式（可复算）

源码：`scripts/docs/gen-crate-status.mjs` 中 `computeCompletion` / `contentScore`。

```text
completion = round( (layoutOk / 7) × 50% + has_tests × 25% + contentScore × 25% )

contentScore = locPart + exPart(0|0.1) + docPart(0|0.05)   # ≤ 1.0
  loc ≥ 500 → 0.85
  loc ≥ 200 → 0.65
  loc ≥  80 → 0.45
  loc ≥  30 → 0.25
  loc >   0 → 0.10
  exampleRs > 0 → +0.10
  docs/README.md trim length > 80 → +0.05
  isScaffold → content 封顶 0.55
```

**布局七项**（生成器权威；OBJECTIVE 表写「8/8」已过时）：  
`src/` · `tests/` · `docs/` · `benches/` · `README.md` · `review/` · `releases/`。

本七包均为 **7/7 + hasTests** → 结构底分 **75%**；余下 **25%** 全由 `contentScore` 决定。

### 1.2 本会话实测重算

| Package | src LOC | examples/*.rs | docs/README | contentScore | 完成度 |
|---------|--------:|--------------:|:-----------:|-------------:|-------:|
| configx | 190 | 1 | ✅ (>80) | 0.45+0.10+0.05=**0.60** | **90%** |
| evidence | 175 | 0 | ✅ | 0.45+0+0.05=**0.50** | **88%** |
| observex | 189 | 0 | ✅ | **0.50** | **88%** |
| schedulex | 109 | 0 | ✅ | **0.50** | **88%** |
| transportx | 416 | 1 | ✅ | 0.65+0.10+0.05=**0.80** | **95%** |
| resiliencx | 914 | 0 | ✅ | 0.85+0+0.05=**0.90** | **98%** |
| contracts | 1117 | 0 | ✅ | **0.90** | **98%** |

与 `STATUS.md` 一致。workspace 摘要：`node scripts/docs/gen-crate-status.mjs --summary` → 平均 **92%**（21 members）。

### 1.3 到达 STATUS 100% 的结构杠杆（便宜 vs 诚实）

| Package | 距 100% | 最小结构动作 | 诚实性提醒 |
|---------|---------|--------------|------------|
| resiliencx | −2pp | 增加 **1 个** `examples/*.rs` | 语义已厚；example 有文档价值 |
| contracts | −2pp | 增加 **1 个** Fake/conformance 可运行 example | 同上；勿伪装 L3 |
| transportx | −5pp | LOC 416→**≥500**（或再加实质模块） | 勿空注释灌水；优先硬化 timeout/limit 文档+测 |
| configx | −10pp | LOC 190→**≥500** | **灌水到 500 行 KV 无生产价值**；更优：保持 partial 或扩 schema 边界文档后由生产波次带厚 |
| evidence / observex / schedulex | −12pp | LOC≥500 **且** example | schedulex 灌水最危险（名实陷阱）；优先 example + README 红线，**不要**为 100% 伪造调度器 |

**关键结论（标尺 A）**：  
100% **主要是 content-score 问题**（布局/测试已满）。  
但对 thin 合同面，**用 LOC 虚高刷 100% 被本目标 Non-goals 明确禁止**；推荐「诚实 partial + 生产 follow-up」优先于虚胖。

---

## 2. 标尺 B — 分层 Production Ready

采用与全模块审计一致的分层（见 [status-modules-production-readiness.md §1.3](./status-modules-production-readiness.md)）：

| 层 | 含义 | 本七包谁相关 |
|----|------|--------------|
| **L1 Internal Ready** | 进程内库；非法态可控；CI 持续绿 | 全部（声明面内） |
| **L2 Wire Ready** | 跨进程/落盘版本与拒绝策略 | 本七包 **基本不适用**（无 wire 主业；canonical 在范围外） |
| **L3 Contract Ready** | 语义合同 + conformance + **非 scaffold 验证入口** | **contracts** 目标层；依赖 adapters W4 |
| **L4 Platform Ready** | OS/MSRV 矩阵 + API 棘轮 | transportx 超时/TLS 矩阵相关；整体未签 |
| **L5 Release Ready** | **人类** Maintainer 签 + DEFER 冻结 + CHANGELOG | **Agent 禁止代签**（`prod-signoff-TEMPLATE.md`） |

另：七包 **`publish = false`**，无 crates.io 发布路径（本目标 Non-goal）。

---

## 3. 逐包分析（双标尺 + PASS/DEFER + 阻断）

### 3.1 `configx` — 90% · partial · 内存 KV

| 项 | 内容 |
|----|------|
| **声明面** | `ConfigStore`：`new` / `get` / `set` / `Default`；`RwLock<HashMap<String,String>>`；仅依赖 kernel |
| **SSOT** | [configx-ssot-alignment.md](../../ssot/configx-ssot-alignment.md)：§2–§6 可移植条款 **无 FAIL**；多源/热更新/secret **DEFER** |
| **DEFER / P0（已有证据）** | §4.6 多源优先级/解析/快照/热重载 DEFER；L1-P0-3：不得作唯一生产配置源（`_partials/l1-infra.md`） |
| **Bar A → 100%** | LOC≥500（已有 `examples/basic.rs` + docs） |
| **Bar B 诚实层** | **L1 Internal（合同内）**；配置**平台**未就绪 |
| **可安全使用** | 进程内字符串字典、单测注入 |
| **禁止宣称** | 配置中心 / schema fail-fast / 热更新 / secret 管理 |

### 3.2 `evidence` — 88% · partial · 内存追加

| 项 | 内容 |
|----|------|
| **声明面** | `EvidenceAppender` / `InMemoryEvidenceAppender` / `AppendReceipt`；bootstrap 可注入 |
| **SSOT** | [evidence-ssot-alignment.md](../../ssot/evidence-ssot-alignment.md)：trait+内存 **PASS**；远程/签名 wire **DEFER** |
| **DEFER / P0** | 远程/签名 DEFER；L1-P0-2：仅内存、进程退出即失，**不得**作合规审计落盘 |
| **Bar A → 100%** | example + LOC≥500 |
| **Bar B 诚实层** | **开发/测试默认可用**；非 L2 证据总线 |
| **可安全使用** | 注入探测、单测 fail_next |
| **禁止宣称** | 持久化审计链 / 跨进程证据 / 已签生产证据系统 |

### 3.3 `observex` — 88% · partial · tracing 三方法

| 项 | 内容 |
|----|------|
| **声明面** | `TracingInstrumentation` → `record_retry` / `record_circuit_open` / `record_circuit_close`；实现 `contracts::Instrumentation` |
| **SSOT** | [observex-ssot-alignment.md](../../ssot/observex-ssot-alignment.md)：core GAP=0；OTEL exporter/flush/shutdown **DEFER** |
| **DEFER / P0** | OTEL DEFER；L1-P0-6：不得宣称生产可观测完成；`infra-s9t.17` subscriber 隔离 |
| **Bar A → 100%** | example + LOC≥500 |
| **Bar B 诚实层** | **L1 tracing 最小面 Ready**；非观测平台 |
| **可安全使用** | resiliencx/bootstrap 事件 info 打点 |
| **禁止宣称** | OTEL 完成 / 生产 metrics 管道 / 有界导出闭环 |

### 3.4 `resiliencx` — 98% · active · 四原语

| 项 | 内容 |
|----|------|
| **声明面** | 重试（可注入 `Wait`）· 熔断（拒绝计数推进，**无墙钟**）· 令牌桶（**显式 refill**）· 舱壁 RAII；只依赖 `contracts::Instrumentation` |
| **SSOT** | [resiliencx-ssot-alignment.md](../../ssot/resiliencx-ssot-alignment.md)：四能力 **PASS**；async wait/backoff/budget **DEFER** |
| **DEFER / P0** | async Wait DEFER；L1-P0-5：默认 `ThreadSleepWait` **阻塞线程**（async 服务路径危险）；`infra-s9t.6` |
| **Bar A → 100%** | **只差 example**（最便宜、且有演示价值） |
| **Bar B 诚实层** | **同步弹性接近 L1 Internal Ready**；async 生产框架未就绪 |
| **可安全使用** | 同步批处理重试/舱壁；显式 `Wait`；知悉无墙钟冷却 |
| **禁止宣称** | async 默认安全 / 墙钟熔断冷却 / 完整 retry budget |

### 3.5 `schedulex` — 88% · partial · ID 登记表（名实落差最大）

| 项 | 内容 |
|----|------|
| **声明面** | `Scheduler::{schedule, cancel, list}`；`HashMap` 任务 ID 集合；**std-only** |
| **SSOT** | [schedulex-ssot-alignment.md](../../ssot/schedulex-ssot-alignment.md)：登记合同 **PASS**；Clock/timer/Job/cron **SSOT §3 禁止** |
| **DEFER / P0** | 生产 timer **不是 DEFER 待做**，是 **永久 out of declared surface**；L1-P0-4 禁止当 production scheduler |
| **Bar A → 100%** | 结构上 example+LOC；**强烈建议不虚胖**，用 example 展示「只登记不触发」+ README 红线（`infra-s9t.8`） |
| **Bar B 诚实层** | **L1 registry only** — 生产级 = **诚实登记库**，不是「未来调度器」 |
| **可安全使用** | 任务 ID 集合管理 |
| **禁止宣称** | 任何定时执行 / cron / 分布式调度 |

### 3.6 `transportx` — 95% · active · 真 HTTP/WS

| 项 | 内容 |
|----|------|
| **声明面** | `HttpDriver` / `WsConnector` + Reqwest/Tungstenite 驱动 + Mock；仅依赖 kernel + 网络信封 |
| **SSOT** | [transport-ssot-alignment.md](../../ssot/transport-ssot-alignment.md)：portable scope **FAIL=0**；M3 TLS/池/体限 **DEFER** |
| **DEFER / P0/P1** | M3 资源边界 DEFER；L1-P1-1 默认超时未强制；`infra-s9t.16` 敏感 Debug/deadline/上限 **P0 follow-up** |
| **Bar A → 100%** | LOC≥500（已有 `examples/mock_ping.rs`） |
| **Bar B 诚实层** | **有条件 L1 Internal + 部分 I/O**；未到 L4 平台矩阵 |
| **可安全使用** | 客户端 HTTP/WS（自管超时/重试策略，重试归 resiliencx） |
| **禁止宣称** | 生产 TLS 矩阵完成 / 无界 body 安全 / 平台级池与代理 |

### 3.7 `contracts` — 98% · active · trait 出口（未过 L3）

| 项 | 内容 |
|----|------|
| **声明面** | 15 trait + `BusMessage`/`MessageAck` + `run_tx_commit_on_ok` + Fake/Recording + first-batch 语义文档 + conformance |
| **SSOT** | [contracts-ssot-alignment.md](../../ssot/contracts-ssot-alignment.md)：CT-1–7 PASS；CT-8/10 **部分**；**CT-9 非 scaffold 入口 DEFER（W4）** |
| **DEFER / P0** | CT-9；ObjectStore/TimeSeries/PubSub/Analytics 无独立文档；adapters 全内存；**L3 三条件未齐**（`_partials/contracts.md`） |
| **Bar A → 100%** | **只差 example** |
| **Bar B 诚实层** | **未达 L3 Contract Ready**；内部 trait 探索 / Fake 编排可用 |
| **可安全使用** | Fake/conformance、Instrumentation、形状验证 |
| **禁止宣称** | L3 Contract Ready / Production Ready / 真实后端语义已证 |

**L3 闭合条件（摘录）**：

1. 语义合同（first-batch 已部分）  
2. conformance suite（部分）  
3. **至少一个非 scaffold 验证入口** ← **在 `crates/contracts` 外**（`infra-s9t.2` redis/postgres 真入口）

---

## 4. Gap → Action 路线图

### 4.1 标尺 A — STATUS 100%（结构，便宜优先）

| 优先级 | 动作 | 影响包 | 波次/Bead | 备注 |
|--------|------|--------|-----------|------|
| A1 | 为 resiliencx / contracts 各加 **1 个** 可运行 example | 98%→100% | 任意小 PR | **最高 ROI**；不改语义 |
| A2 | transportx 实质硬化顺带过 500 LOC，或诚实模块扩展 | 95%→100% | 对齐 `infra-s9t.16` | 优先有价值代码，非注释 |
| A3 | evidence / observex 加 example + 边界文档；LOC 仅随真实硬化增长 | 88%→… | `infra-s9t.7` / `.8` / `.17` | 不强制本迭代 100% |
| A4 | configx：若坚持 100%，应绑定 schema 边界机检/文档，而非空 LOC | 90%→100% | `infra-s9t.7` | 虚胖 = Non-goal 违规 |
| A5 | schedulex：**优先** 红线 example（登记≠执行），**拒绝** 伪造 timer 冲 100% | 88% | `infra-s9t.8` | 名实优先于进度条 |

### 4.2 标尺 B — 生产级（声明面，有序）

对齐 [status-modules-prod-followup.md](../../plans/2026-07-21-status-modules-prod-followup.md) / epic **`infra-s9t`**：

```text
并行 A（文档/红线/本地硬化）:
  .1  冻结可生产消费面（allow/deny）
  .6  resiliencx async Wait（默认非阻塞路径）
  .7  evidence 持久化合同边界 + configx schema 红线
  .8  schedulex/observex 误用红线
  .14 adapters scaffold 警示
  .16 transport 脱敏 / deadline / 资源上限
  .17 observex subscriber 隔离

串行关键路径（L3）:
  .1 W0+ ──► .2 W4 非 scaffold 真实后端 ──► .3 contracts L3 闭合

L5（仅人类）:
  W3+W4+L1-P0 闭合后 → Maintainer 填 prod-signoff 模板
  Agent 禁止填写签核人 / 「已签核」
```

| 生产杠杆 | 对应包 | Bead | 是否在声明面内 |
|----------|--------|------|----------------|
| 语义闭合（Fake 真路径） | contracts first-batch | `.3` | 是 |
| 非 scaffold 后端 | contracts L3 + adapters | `.2` | **跨包**；非 contracts-only |
| async Wait | resiliencx | `.6` | 是（当前 DEFER residual） |
| 资源/超时 | transportx | `.16` | 部分（M3 全矩阵仍 DEFER） |
| 名实红线 | configx/evidence/observex/schedulex | `.7` `.8` | **声明面治理**；平台能力仍 DEFER |
| 人签 L5 | 全集 | 人工 | **Agent 不能完成** |

### 4.3 明确 **out of declared surface**（勿当 FAIL）

| 项 | 归属 | 说明 |
|----|------|------|
| 多源配置中心 / 热更新 | configx SSOT DEFER | 不是 0.1.0 FAIL |
| 签名证据链 / 远程 wire | evidence DEFER | 不是内存面 FAIL |
| 完整 OTEL 栈 | observex DEFER | tracing 三方法面已 PASS |
| 墙钟熔断 / 自动 refill | resiliencx 设计 | 确定性合同；非 bug |
| timer/cron/Job 执行 | schedulex **禁止** | 永久 out of surface |
| TLS/池/gRPC 全矩阵 | transport M3 DEFER | portable FAIL=0 |
| 真实交易所/DB 业务适配 | adapters + W4 | contracts L3 依赖，非本分析实现范围 |
| crates.io / `publish=true` | 治理 Non-goal | 七包皆 `publish=false` |

---

## 5. 跨包风险（集合「生产级发布」阻断）

| ID | 风险 | 严重度 | 说明 |
|----|------|--------|------|
| X-1 | **STATUS 100% ≠ 生产发布** | P0 认知 | 进度条与语义签字不可互换 |
| X-2 | **contracts L3 依赖 adapters W4** | P0 | 无法在 `crates/contracts` 内单独 GO |
| X-3 | **resiliencx 默认阻塞 sleep × transport async I/O** | P0/P1 | async 服务误用 `retry_fn` 会堵线程；429 与重试分层未编排 |
| X-4 | **名实误用集** | P0 | evidence/configx/schedulex/observex 名称像平台、合同是最小面 |
| X-5 | **假关停组合** | P0 | 本范围外 bootstrap 有信号无 drain；与 transport 连接/resiliencx sleep 无统一编排 |
| X-6 | **Agent 不能 L5 签 / 禁止自批准** | P0 治理 | `prod-signoff-TEMPLATE.md`：Maintainer only |
| X-7 | **`publish=false` / 无 crates.io** | P1 发布面 | 「生产级发布」指**内部声明面可用 + 人签**，不是 crates.io 发版 |
| X-8 | **SSOT DEFER 被当成 FAIL** | P1 | 扩大平台幻想会破坏 active 合同诚实性 |

**拼装失败模式（若把 STATUS 高分直接当生产应用）**：

```text
app
  ├─ configx 唯一配置源     → 无 schema，带病运行
  ├─ InMemory evidence      → 审计空心
  ├─ observex Tracing only  → 无导出
  ├─ resiliencx.retry_fn    → 默认 thread::sleep
  ├─ schedulex.schedule     → 只登记 ID，永不触发
  ├─ transportx 无硬超时    → 资源/ TLS 矩阵未签
  └─ contracts + *Adapter   → scaffold 内存，非真后端
```

---

## 6. 推荐目标态（务实）

### 6.1 若目标是「STATUS 表七包全 100%」

| 包 | 建议 | 估计工作量 |
|----|------|------------|
| resiliencx, contracts | 各 1 example | 极小 |
| transportx | 有价值硬化 + LOC 过桶 | 小–中 |
| configx, evidence, observex, schedulex | **不要**为 100% 灌水；或仅加 example 接受 90–93% 诚实区间 | — |

### 6.2 若目标是「声明面生产级（可讨论签字）」

| 包 | 目标层 | 前置闭合 |
|----|--------|----------|
| configx | L1 合同 + 红线 | README/消费面冻结（`.1` `.7`） |
| evidence | L1 开发默认 + 红线 | 持久化合同文档；实现可另 epic |
| observex | L1 tracing + 红线 | `.8` `.17` |
| resiliencx | L1 同步 **可签**；async 另列 Accept | `.6` 或 Accept「仅同步」 |
| schedulex | L1 registry **可签**（名实） | `.8` 红线 |
| transportx | L1 I/O 有条件 + Accept 列表 | `.16` |
| contracts | **L3** | **`.2` + `.3`**（跨 adapters） |

### 6.3 若目标是「七包集合可生产发布 / L5」

**当前不可达成**（本分析目标也 **不能** 代签）。最短讨论路径：

1. `.1` 消费面冻结  
2. `.2` 至少一个真实后端入口  
3. `.3` contracts L3  
4. resiliencx/transport/名实 P0/P1 闭合或 Accept  
5. **人类** Maintainer 签 `prod-signoff`  

---

## 7. 证据索引

### 7.1 仓库内既有（可点查）

| 资源 | 用途 |
|------|------|
| `STATUS.md` | 七包 % / 成熟度 |
| `scripts/docs/gen-crate-status.mjs` | 公式 SSOT |
| `docs/ssot/{configx,evidence,observex,resiliencx,schedulex,transport,contracts}-ssot-alignment.md` | PASS/DEFER 矩阵 |
| `docs/report/2026-07-21/status-modules-production-readiness.md` | 全仓生产审计 |
| `docs/report/2026-07-21/_partials/l1-infra.md` | L1 P0/P1 明细 |
| `docs/report/2026-07-21/_partials/contracts.md` | L3 三条件 |
| `docs/plans/2026-07-21-status-modules-prod-followup.md` | 行动树 `infra-s9t` |
| `docs/governance/prod-signoff-TEMPLATE.md` | L5 人签；禁 Agent 代签 |

### 7.2 本会话命令

```bash
# 基线健康（声明合同可运行）
cargo test -p configx -p evidence -p observex -p resiliencx \
  -p schedulex -p transportx -p contracts --all-targets
# → 171 passed, 0 failed（2026-07-21）

# STATUS 摘要
node scripts/docs/gen-crate-status.mjs --summary
# → 平均 92%；布局 21/21 齐全
```

| Package | 本会话测试摘要（passed） |
|---------|--------------------------|
| configx | unit 8 + concurrency 1 + public_api 5 |
| evidence | unit 5 + public_api 2 + surface 1 |
| observex | unit 8 + public_api 3 + surface 1 |
| resiliencx | unit 26 + public_api 6 + surface 3 + retry_contract 13 |
| schedulex | unit 5 + public_api 2 + surface 1 |
| transportx | mock 18 + reqwest 14 + websocket 9 + surface 2 |
| contracts | unit 22 + conformance 5 + public_surface 5 + surface 2 + venue_override 4 |

> 全绿 = 声明合同与实现一致。**≠** 生产平台闭合、**≠** L5 签字。

### 7.3 抽查引用（实现侧）

| 包 | 具体 DEFER/P0 指针 |
|----|-------------------|
| configx | SSOT §4.6 多源/热更新 DEFER；alignment「禁止把内存 KV 描述成多源热更新」 |
| evidence | alignment「远程/签名 wire DEFER」；l1-infra L1-P0-2 |
| observex | alignment §4.6 OTEL DEFER；l1-infra L1-P0-6 |
| resiliencx | alignment「async wait … DEFER」；l1-infra L1-P0-5 默认阻塞 sleep |
| schedulex | alignment「非 production timer scheduler」；§3 禁止 Clock/timer |
| transportx | alignment §4.3 M3 DEFER；§5.5 资源边界 DEFER |
| contracts | alignment CT-9 DEFER；partial「非 scaffold 入口未满足」 |

---

## 8. 禁止与允许的表述（本七包）

### 禁止

- 「STATUS 90–98% / 刷到 100% ⇒ 生产级发布」
- 「contracts 已 L3 Contract Ready」
- 「schedulex 可作生产定时调度」
- 「evidence / observex 完成审计 / OTEL」
- 「configx 是生产配置中心」
- 「Agent 已 L5 签核 / 自批准 GO」
- 「七包可 crates.io 生产发布」

### 允许（有条件）

- 「configx / schedulex 在 **active SSOT 最小面** 合同内 L1 就绪」
- 「resiliencx 同步弹性原语接近 L1 Internal Ready」
- 「transportx 客户端传输有条件可用（Accept 超时/TLS）」
- 「observex ADR-005 tracing 最小面就绪」
- 「evidence 开发默认注入可用」
- 「contracts trait 出口 + Fake/conformance 可用；L3 待 W4」
- 「工程测试七包本会话全绿」
- 「STATUS 100% 对 98% 包 = 加 example；对 thin 包勿虚胖」

---

## 9. 元数据

| 项 | 值 |
|----|-----|
| 报告路径 | `docs/report/2026-07-21/seven-l1-contracts-dual-bar-readiness.md` |
| 作者角色 | Agent 只读分析 |
| 非交付 | 代码变更 · Maintainer 签名 · crates.io · 扩大 SSOT DEFER 平台 |

---

*本报告合成七包双标尺路径；细节以 SSOT 对齐文与 `_partials/` 源码证据为准。若 STATUS 生成器公式变更，应重跑 §1 重算表。*

---

## 10. Follow-through（本分支实现 · 结构 A1/A5 + 红线）

| 日期 | 分支 | 性质 |
|------|------|------|
| 2026-07-21 | `docs/infra-seven-status-prod-follow` | 有价值 example + README 生产误用红线；**非** L5 签核、**非** vanity LOC |

### 10.1 STATUS 变化（`gen-crate-status.mjs` 刷新后）

| Package | 分析时 % | 现 % | 动作 |
|---------|----------:|-----:|------|
| resiliencx | 98% | **100%** | `examples/retry_sync.rs`（NoWait 同步重试 + ThreadSleep 红线） |
| contracts | 98% | **100%** | `examples/fake_surface.rs`（Fake KV/Tx + L3 红线） |
| evidence | 88% | **90%** | `examples/append_memory.rs` |
| observex | 88% | **90%** | `examples/trace_events.rs` |
| schedulex | 88% | **90%** | `examples/registry_only.rs`（明示不触发 timer） |
| configx | 90% | **90%** | 已有 example；剩余需 LOC≥500（拒绝虚胖） |
| transportx | 95% | **95%** | 已有 example；剩余需 LOC≥500（优先 `infra-s9t.16` 硬化） |

### 10.2 生产标尺 B

- README 七包均补 **生产误用红线** 表。
- **仍未** contracts L3、async Wait、真实后端、L5 Maintainer 签。
- 绿测试 / 可运行 example **≠** Production Ready。

### 10.3 验证

```bash
cargo run -p resiliencx --example retry_sync
cargo run -p contracts --example fake_surface
cargo run -p evidence --example append_memory
cargo run -p observex --example trace_events
cargo run -p schedulex --example registry_only
cargo test -p resiliencx -p contracts -p evidence -p observex -p schedulex --all-targets
node scripts/docs/gen-crate-status.mjs --summary
```
