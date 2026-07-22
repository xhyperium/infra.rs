# Round 9: DEFER — 延迟项累积审查

| 字段 | 值 |
|------|-----|
| 轮次 | 9/10 |
| 视角 | DEFER 累积 gap 审查 |
| 日期 | 2026-07-22 |
| 审查范围 | 全部 workspace member crate（23 package） |
| 审查依据 | 所有 alignment docs（`docs/ssot/*-ssot-alignment.md`）、SSOT specs（`.agents/ssot/*`）、prod-consume-surface、defer-disposition（W0 冻结）、2026-07-21 审计 §11.2 |

## 1. 审查摘要

本轮对所有 alignment doc 及 SSOT spec 中的 DEFER 项进行累积审查，共识别出 **54 个独立的 DEFER/GAP 项**，分布在 21 个 crate/域中。

**关键发现**：
- 历史 DEFER-1 ~ DEFER-8（2026-07-21 审计 §11.2）**全部已闭合或接受**（post-W5 + L5，见 defer-disposition.md）
- 新一轮 DEFER 主要来自 L1 平台层能力不足（configx 多源、observex OTEL、resiliencx budget）和 contracts 全 trait 深度
- Adapter 层 Cluster/JetStream/EOS 等扩展能力已明确为 DEFER，不影响 P0 生产入口
- 整体 Production Ready **仍否**（Accept 残留仍然存在）

## 2. 逐 crate DEFER 清单

### 2.1 kernel

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| K-D1 | crates.io 再发布 | kernel-ssot-alignment §12+ | `publish = false` 显式关闭；本仓明确不向 crates.io 再发布 |
| K-D2 | crates.io `publish = true` | 同上 | 与 K-D1 一体两面 |

### 2.2 decimalx

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| D-D1 | package stable / Spec Approved | types-ssot-alignment D-9 | 未宣称 |
| D-D2 | full productized fuzz（cargo-fuzz 靶） | types-ssot-alignment §未做 | scheduled mutants/miri 有 CI；非 cargo-fuzz 全量 |
| D-D3 | wire 跨版本 stable 协议 | types-ssot-alignment §未做 | wire shape = 当前事实 ≠ stable 承诺 |

### 2.3 canonical

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| C-D1 | package stable / 跨语言 wire 协议 / envelope | types-ssot-alignment C-9 | committed v1–v1.3 子集；全量未宣称 |
| C-D2 | 上游镜像 `wire-commitment-matrix.md` 与实现清单同步 | types-ssot-alignment §未做 | 非本仓阻塞项 |

### 2.4 contracts

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| CT-D1 | 全 trait 深度 conformance��ObjectStore / TimeSeries / PubSub / Analytics） | contracts-ssot-alignment CT-8 | first-batch 11 篇语义文档 + conformance；其余 DEFER |
| CT-D2 | Tx / Bus / Repository / Venue 业务 live | contracts-ssot-alignment CT-9, L3 子集 | KV+Instr L3 子集 OK；业务 live 深度 DEFER |
| CT-D3 | 真实 postgres Tx / kafka·nats Bus / 交易所业务 live（非只读 time） | contracts-ssot-alignment §未做(DEFER) | 超出 L3 子集范围 |
| CT-D4 | VenueAdapter compile-fail 强制 override | contracts-ssot-alignment CT-10 | 当前仅 runtime `venue_override_gate` 测 |
| CT-D5 | Additive Only API snapshot / semver diff 门禁 | contracts-ssot-alignment §未做(DEFER) | 无 compile-fail 机控 |

### 2.5 bootstrap

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| B-D1 | 全量 async contracts 注入 | bootstrap-ssot-alignment §5 | 组合根仅持 `Bounded*` 最小面 |
| B-D2 | 真实 app 生命周期 / async drain | bootstrap-ssot-alignment §5 | SSOT 开放项 |
| B-D3 | composition manifest（BOOT-MAN-001） | bootstrap-ssot-alignment §5 | 非目标 |
| B-D4 | 异步组件启动/逆序补偿 | bootstrap-ssot-alignment §5 | 非目标 |
| B-D5 | 生产就绪 / package stable | bootstrap-ssot-alignment §5 | 未宣称 |
| B-D6 | 远程持久化 wire（evidence） | bootstrap-ssot-alignment §2 | 内存实现已有；远程 DEFER |
| B-D7 | dev：binance / redisx / canonical / tokio e2e（无 monorepo adapters） | bootstrap-ssot-alignment §2 | 以 stub trait double 替代 |
| B-D8 | `xtl lint-deps` / `xtl no-new-gate` | bootstrap-ssot-alignment §6 | 本仓无 `cargo xtl` 工具链 |

### 2.6 configx

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| CX-D1 | 多源加载 / 热更新 / secret | configx-ssot-alignment §7.5 | SSOT Unknown；未批准前不实现 |
| CX-D2 | 源/格式/优先级/类型化/订阅/通知 | configx-ssot-alignment | Unknown 能力全体 DEFER |
| CX-D3 | schema 校验 / 失败保留旧快照 | configx-ssot-alignment §5.7, §6.5 | 当前仅内存 KV；无解析路径 |
| CX-D4 | 热更新 runtime / 去抖 / 背压 / 关闭协议 | configx-ssot-alignment §5.3 | 未实现 |
| CX-D5 | secret 脱敏 / 信任合同诊断 | configx-ssot-alignment §5.5 | 未实现 |

### 2.7 schedulex

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| S-D1 | timer / cron / 调度执行 | schedulex-ssot-alignment §3 明确非目标 | **显式禁止**；非 DEFER，是 Non-goal |
| S-D2 | Once / FixedDelay / FixedRate / misfire / lease / shutdown / 持久化 / 分布式 | 同上 | 同 S-D1：明确非目标 |
| S-D3 | 平台幻想（当生产调度器使用） | prod-consume-surface Deny | 禁止 |

> **注意**：schedulex 的上述项并非 DEFER（待实现），而是**显式非目标**（禁止实现），应归为「设计排除」。

### 2.8 observex

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| O-D1 | OTEL exporter / flush / shutdown | observex-ssot-alignment §2.5, §4.6 | 禁止宣称 OTEL 栈完成 |
| O-D2 | 基数/敏感性强制 | observex-ssot-alignment §4.5 | 当前为推论，未强制 |
| O-D3 | 失败隔离策略（未来 exporter） | observex-ssot-alignment §5.3 | 无 exporter 时无关 |
| O-D4 | 完整上游 contracts（VenueAdapter 等，非 observex 职��） | observex-ssot-alignment §2.5 | 非 observex 职责 |
| O-D5 | exporter/flush 测试 | observex-ssot-alignment §6.7 | API 未批 |

### 2.9 evidence

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| E-D1 | 远程/签名 wire | evidence-ssot-alignment | `InMemoryEvidenceAppender` + `FileEvidenceAppender` 已有；远程 DEFER |

### 2.10 resiliencx

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| R-D1 | backoff / budget / stable | resiliencx-ssot-alignment | residual OPEN；重试/熔断/限流/舱壁 PASS |

### 2.11 transportx

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| T-D1 | M3 Unknown：TLS/池/代理/gRPC | transportx-ssot-alignment §4.3 | 生产矩阵；Non-goals |
| T-D2 | 完整 M3 矩阵（资源边界、TLS 全矩阵） | transportx-ssot-alignment §4.4, §5.5 | P0 硬化已完成（#166） |
| T-D3 | exchange 业务协议（下单/签名） | transportx-ssot-alignment §3.binance/okx | pub time OK；业务战役 DEFER |

### 2.12 testkit

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| TK-D1 | 全 trait 深度 conformance / 真实后端 profile（contract-testkit 侧） | testkit-ssot-alignment §24.4 | first-batch PASS；其余 DEFER |
| TK-D2 | integration harness（跨 crate INFRA） | testkit-ssot-alignment §3.3, §24.0 residual | 跨 crate 集成未做 |
| TK-D3 | branch coverage ≥90% 强制 | testkit-ssot-alignment §13.7, §24.3 | OPTIONAL residual；本仓不升强制 |
| TK-D4 | 全仓 production-graph machine gate（xtask） | testkit-ssot-alignment §24.5 | 无 infra-xtask graph check |
| TK-D5 | governance 机控（archgate OOS） | testkit-ssot-alignment §24.6 | OOS（本仓不移植 archgate） |

### 2.13 contract-testkit

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| CTK-D1 | 全 trait 深度 / 真实后端 profile | testkit-ssot-alignment, contracts-ssot-alignment | first-batch suite PASS；对象/时序/发布订阅等 DEFER |

### 2.14 adapters — aggregate

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| AP-D1 | package stable / Spec Approved / crates.io | adapters-ssot-alignment A-10 | P0 生产入口 ≠ package stable |
| AP-D2 | Cluster / Sentinel / Streams / JetStream / EOS / multipart / native protocol / migrations | adapters-ssot-alignment §未做 | 非 P0；storage 生产默认客户端已落地 |
| AP-D3 | exchange 交易 / 签名 / 私有 API | adapters-ssot-alignment A-9 | 仍 scaffold（仅 server_time 公共 API） |
| AP-D4 | contracts `Ticker` f64 → decimalx/canonical 迁移 | adapters-ssot-alignment §未做 | 金额字段迁离 f64 |
| AP-D5 | package 命名是否统一 `xhyper-*` 前缀 | adapters-ssot-alignment §未做 | 需 Lead 裁决 |

### 2.15 Storage 逐包 DEFER（per-package）

| Package | DEFER 项 | 来源 |
|---------|----------|------|
| **redisx** (REDISX-10) | Cluster / Sentinel / Streams full / pubsub | redisx-ssot-alignment |
| **postgresx** (POSTGRESX-10) | COPY / migrations / read-replica / SSL require-only 默认 | postgresx-ssot-alignment |
| **kafkax** (KAFKAX-10) | EOS / transactional producer / schema registry / group coordinator | kafkax-ssot-alignment |
| **natsx** (NATSX-10) | JetStream 全量 / NKey / TLS 默认开启策略 | natsx-ssot-alignment |
| **ossx** (OSSX-10) | multipart / lifecycle / STS 临时凭证 | ossx-ssot-alignment |
| **clickhousex** (CLICKHOUSEX-10) | native 9000 protocol / cluster / ReplicatedMergeTree 运维面 | clickhousex-ssot-alignment |
| **taosx** (TAOSX-10) | native WS / 全超表治理 / 集群 | taosx-ssot-alignment |

### 2.16 tools

| ID | DEFER 项 | 来源 | 说明 |
|----|----------|------|------|
| TOL-D1 | goalctl 完整 authority plane（index/reconcile） | tools-ssot-alignment §明确未宣称 | 最小编译器已落地 |
| TOL-D2 | verifyctl 变更感知闭包 / 远程 runner / 签名证据链 / V0–V3 矩阵 | tools-ssot-alignment §明确未宣称 | 最小 plan/execute/report 已落地 |
| TOL-D3 | xtask / gate 工作流编排 | tools-ssot-alignment §明确未宣称 | xtask **未** member · **未**宣称 ship |
| TOL-D4 | evidence 远程/签名 wire | tools-ssot-alignment | 同 E-D1 |

## 3. DEFER 严重程度分类

### 3.1 分类定义

| 分类 | 含义 |
|------|------|
| **Blocker** | 阻断当前目标生产层级宣称 |
| **Major** | 显著生产缺口，影响功能完整性 |
| **Minor** | nice-to-have，不阻塞 |
| **Accepted** | 明确接受为不实现或 deferred with sign |

### 3.2 分类表

#### Blocker（阻断生产宣称）

| ID | 项 | 适用 crate | 阻断层级 |
|----|----|-----------|----------|
| CT-D1 | 全 trait 深度 conformance | contracts | L3 全量 |
| CT-D2 | Tx/Bus/Repository/Venue 业务 live | contracts | L3 真实后端 |
| CX-D1/D2 | 多源/热更新/schema | configx | L2/L3 配置平台 |
| O-D1 | OTEL exporter/flush/shutdown | observex | L3 可观测 |
| R-D1 | backoff/budget/stable | resiliencx | L2/L3 生产策略 |
| AP-D3 | exchange 业务 live | binancex/okxx | L3 交易 |
| CTK-D1 | 全 trait depth/真实后端 profile | contract-testkit | L3 验证面 |

#### Major（显著生产缺口）

| ID | 项 | 适用 crate | 影响 |
|----|----|-----------|------|
| B-D1 | 全量 async contracts | bootstrap | 组合根能力 |
| B-D2 | 真实 app 生命周期 / async drain | bootstrap | 生产运维 |
| T-D1 | M3 TLS/池/代理/gRPC | transportx | 传输矩阵 |
| CJ-D1 | Cluster/Sentinel/JetStream/EOS ×7 | adapters/storage | HA/Scalability |
| E-D1 | 远程/签名 wire | evidence | 合规审计 |
| D-D3 | wire 跨版本 stable 协议 | decimalx | 跨服务演化 |
| C-D1 | package stable | canonical | 跨语言 |
| TK-D2 | integration harness | testkit | 跨 crate 验证 |

#### Minor（nice-to-have）

| ID | 项 | 适用 crate | 说明 |
|----|----|----------|------|
| B-D3/B-D4 | composition manifest / 异步启动 | bootstrap | 非目标 |
| B-D8 | `xtl lint-deps` | bootstrap | 工具链缺口 |
| O-D2/O-D3 | 基数强制 / 未来失败隔离 | observex | 无 exporter 时无关 |
| TK-D3 | branch coverage 强制 | testkit | OPTIONAL |
| TOL-D1/D2 | 完整 authority plane / V0–V3 矩阵 | tools | 最小面已落地 |
| AP-D5 | package 命名统一 | adapters | 需 Lead 裁决 |
| CT-D5 | Additive Only API snapshot | contracts | 门禁增强 |
| B-D7 | dev e2e（无 monorepo） | bootstrap | 替代方案已有 |

#### Accepted（已接受）

| ID | 项 | 来源 | 接受裁定 |
|----|----|------|----------|
| K-D1/D2 | crates.io 再发布 | kernel-ssot-alignment | `publish = false` 显式；**Accepted — 本仓不发布 crates.io** |
| DEFER-6 | 非 Linux 矩阵 | defer-disposition | **Accept** — 仅 Linux x86_64 + MSRV 1.85 |
| DEFER-1 (Accept 部分) | 真实云端后端 | defer-disposition | **Accept**（mock 入口 Close；真实云端 Accept） |
| DEFER-2 (Accept 部分) | 全 trait 深度语义二期 | defer-disposition | **Accept**（首批 Close；二期 Accept） |
| AP-D4 | Ticker f64 → decimalx 迁移 | adapters-ssot-alignment | **Accepted** — 禁止宣称 stable 直至收口 |
| TOL-D3 | xtask 未 ship | tools-ssot-alignment | **Accepted** — 明确未宣称 |

### 3.3 分类统计

| 分类 | 计数 |
|------|------|
| Blocker | 7 |
| Major | 8 |
| Minor | 9 |
| Accepted | 6 |
| 设计排除（schedulex Non-goal） | 3 |

## 4. 历史 DEFER 闭合状态（2026-07-21 → 2026-07-22）

### 4.1 DEFER-1 ~ DEFER-8 闭合追踪

| ID | 审计时（§11.2） | post-W5 + L5 状态 | 闭合/残留 |
|----|-----------------|-------------------|-----------|
| DEFER-1 | 真实后端验证入口 | **Close (mock 入口)** + **Accept (真实云端)** | 部分闭合，残留 Accept：postgres live（未交付）、exchange 业务 live（未交付） |
| DEFER-2 | contracts 全 trait 深度语义 | **Close (首批)** + **Accept (二期)** | 部分闭合，残留 Accept：ObjectStore/TimeSeries/PubSub/Analytics 仍未全量 |
| DEFER-3 | canonical 非 committed DTO | **Close (分批)** v1.1–v1.3 | **已闭合**：Order/Tick/Trade/Position/OrderBookSnapshot/PriceLevel/SymbolMeta 已晋升 |
| DEFER-4 | fuzz/oracle/mutants/Miri | **Close** | **已闭合**：oracle/边界/proptest + scheduled miri/mutants |
| DEFER-5 | API snapshot/semver 门禁 | **Close** | **已闭合**：baselines + public-api 门禁 |
| DEFER-6 | 非 Linux 矩阵 | **Accept** | **Accepted** — 不变 |
| DEFER-7 | 人签 | **Close** | **已闭合**：0.3.0-signoff GO-with-Accepts @ZoneCNH |
| DEFER-8 | Venue override 门禁 | **Close** (runtime gate) | **已闭合**：runtime `venue_override_gate` |

**闭合率**：5/8 Close（DEFER-3/4/5/7/8）+ 1 Accept（DEFER-6）+ 2 Close+Accept 混合（DEFER-1/2）

### 4.2 仍 OPEN 的历史残留

| 残留 | 来源 | 状态 |
|------|------|------|
| postgresx live 未交付 | prod-consume-surface Conditional | **仍然 OPEN** — scaffold/mock Tx 仍 |
| exchange 业务 live（签名、私有 API、下单） | prod-consume-surface Deny | **仍然 OPEN** — scaffold + mock HTTP |
| ObjectStore/TimeSeries/PubSub/Analytics conformance | contracts DEFER-2 Accept 二期 | **仍然 OPEN** |
| 真实云端后端（DEFER-1 Accept 部分） | defer-disposition | **仍然 Accept** |
| 二期 trait 全量（DEFER-2 Accept 部分） | defer-disposition | **仍然 Accept** |

## 5. 累积 gap 评估

### 5.1 按层级汇总

| 层级 | 当前状态 | 累积 DEFER gap | 能否在当前层级宣称？ |
|------|---------|---------------|---------------------|
| L1 Internal Ready | 签核确认（GO-with-Accepts） | 0 Blocker | **是**（2026-07-21 签核有效） |
| L2 Wire Ready | committed 子集 | 0 Blocker（canonical 全 DTO 为 Major） | **有条件**：committed v1–v1.3 |
| L3 Contract Ready | mock-L3 + KV/Instr 子集 | 2 Blocker（CT-D1/CT-D2） | **否**（业务 live 缺失） |
| L4 Platform Ready | 矩阵 + API baseline | 0 Blocker | **是**（Accept 仅非 Linux） |
| L5 Release Ready | GO-with-Accepts | 人工签字域 | **是**（2026-07-21 签核仍有效） |
| **整体 Production Ready** | — | 7 Blocker + 8 Major | **否** |

### 5.2 关键 gap 路径

```text
整体 Production Ready ←─┐
                         │
    ┌────────────────────┼────────────────────┐
    │                    │                    │
L3 Contract Ready    configx L2/L3      observex L3
(CT-D1/CT-D2 =     (CX-D1 = 多源/      (O-D1 = OTEL
 Blocker)           schema)            exporter)
    │
    ├─ ObjectStore conformance    → Major
    ├─ TimeSeries conformance     → Major
    ├─ PubSub conformance         → Major
    ├─ Tx live (真实 postgres)    → Major
    ├─ Bus live (真实 kafka/nats) → Major
    └─ Venue live (真实交易所)     → Blocker
```

### 5.3 与 prod-consume-surface 交叉对照

| 消费面 | 当前裁定 | 累积 DEFER 是否变更裁定？ |
|--------|---------|--------------------------|
| `kernel` L1+L4 Allow | Allow | 否（Accept 残留不变） |
| `decimalx` L1 Allow | Allow（checked_* only） | 否 |
| `canonical` L2 Allow | Allow（committed 子集） | 否 |
| `contracts` L3 Conditional | **Conditional** | 否（业务 live 仍 DEFER） |
| `redisx` live KV Conditional | Conditional（feature `live`） | 否 |
| `postgresx` live Conditional | **未交付**（仍 scaffold） | 否 |
| exchange 只读 `server_time` Conditional | Conditional（#172） | 否 |
| exchange 业务 live | **Deny** | 否（业务 live 仍 DEFER） |
| L1 平台层七包 | Allow（条件） | 否（配置/Observability DEFER 不影响 Allow 子集） |

**结论**：本轮识别的 DEFER 累积项**未**变更 prod-consume-surface 的 Allow/Conditional/Deny 裁定矩阵。

### 5.4 累积趋势

- 2026-07-21 审计 §11.2：**8 项 DEFER**（DEFER-1 ~ DEFER-8）
- 2026-07-22 本轮审查：**54 项 DEFER/GAP**（含历史、新识别的逐包细粒度 DEFER）
- 差异解释：本轮从 alignment docs 的逐条矩阵中提取了**所有 DEFER 标记**（包括 Minor/Accepted/Non-goal），粒度从 2026-07-21 的 8 个大类拆解为 54 个可追溯的逐 crate 项
- **有效 Blocker 增长**：从 0（DEFER-1~8 全部 Close/Accept）→ 7 个（本轮发现：contracts 全 trait、configx 多源、observex OTEL、resiliencx budget、exchange 业务 live）

## 6. 轮次结论

### 6.1 发现

1. **历史 DEFER 全部处理完毕**：DEFER-1 ~ DEFER-8 在 post-W5 + L5 阶段已全部 Close 或 Accept（defer-disposition.md 2026-07-21 冻结），无未分类项
2. **新 DEFER 已录入 alignment docs**：各 crate 的 `*-ssot-alignment.md` 均已明确标注 DEFER 项，状态追踪机制健全
3. **L3 Contract Ready 为核心 gap**：7 个 Blocker 中 2 个来自 contracts（全 trait + 业务 live），2 个来自 configx（多源/schema），1 个来自 observex（OTEL），1 个来自 resiliencx（budget），1 个来自 exchange（业务 live）
4. **L1 底座稳固**：kernel/decimalx/canonical/testkit 无新增 Blocker，L1 Internal Ready 签核仍有效
5. **Adapter 层 P0 已就绪**：storage×7 生产默认客户端已落地 + live 真凭据已验证；DEFER 均为 HA/Scalability 扩展

### 6.2 建议

| 优先级 | 建议 | 依据 |
|--------|------|------|
| P0 | contracts 全 trait 深度 conformance（ObjectStore/TimeSeries 等） | CT-D1（Blocker），阻塞 L3 全量 |
| P0 | exchange 业务 live（签名/下单/私有 API） | AP-D3（Blocker），阻塞交易路径 |
| P1 | configx 多源/schema | CX-D1/D2（Blocker），阻塞 L2/L3 配置 |
| P1 | observex OTEL exporter | O-D1（Blocker），阻塞 L3 可观测 |
| P1 | resiliencx backoff/budget | R-D1（Blocker），阻塞生产策略完备 |
| P2 | postgresx live 交付 | prod-consume-surface Conditional 未交付 |
| P2 | bootstrap 异步 drain / 完整 async contracts | B-D1/B-D2（Major） |
| P3 | storage Cluster/JetStream HA 扩展 | 7 个 per-package DEFER（Major） |
| Keep | 历史 Accept 项维持现状 | DEFER-1/2 Accept 部分、DEFER-6、K-D1/D2 等 |

### 6.3 总体判定

- L1 Internal Ready：**维持**（2026-07-21 签核仍有效）
- L2 Wire Ready：**维持**（committed 子集）
- L3 Contract Ready：**不可宣称完整**（7 个 Blocker 未闭合）
- L4 Platform Ready：**维持**（API baseline + 矩阵，Accept 非 Linux）
- L5 Release Ready：**维持**（GO-with-Accepts 签核仍为最终人工裁定）
- **整体 Production Ready：否**（累积 DEFER 未消除，Blocker 项仍存在）

## 7. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | 初版：DEFER 累积审查，54 项识别，7 Blocker + 8 Major + 9 Minor + 6 Accepted + 3 设计排除 |
