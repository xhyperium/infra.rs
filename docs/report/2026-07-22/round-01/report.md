# Round 1: Baseline Scan — Spec 完整性基线扫描

| 字段 | 值 |
|------|-----|
| 轮次 | 1/10 |
| 视角 | Baseline Scan |
| 日期 | 2026-07-22 |

## 1. 审查摘要

对 infra.rs 全部 24 个 workspace member 的 SSOT spec 存在性与覆盖度进行基线扫描。发现 **三极分化**：kernel/testkit/evidence/goalctl/verifyctl/decimalx/canonical 共 7 个域拥有深度完整 spec（评分 8-10），7 个 storage adapter 由 2026-07-22 当日统一填充达到中期水平（评分 5-6），而 configx/schedulex/bootstrap/observex/transport/contracts 及 2 个 exchange scaffold 共 8 个域仅有完全 stub 管道或空目录（评分 0-3）。**3 个 SSOT 域无对应 crate**（gate/testkitx/xtask），**1 个域 SSOT 目录完全为空**（configx），**1 个 crate 无独立 SSOT 域**（contract-testkit）。

## 2. 逐 crate 分析

### 2.1 kernel

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道目录 + 1 README + 3 spec 文件 + evidence 子目录（40+ evidence 文件） |
| 填充文件数 (>20 lines) | 14/14 管道文件 + 3 spec + plan/gap-matrix-v2 + design/DESIGN-KERNEL-002 |
| 7行占位数 | 0（所有管道文件均有实质内容） |
| Spec 深度分 | **10/10** |
| 对齐文档 | kernel-ssot-alignment.md（219 lines） — Yes |
| S1-S7 维度 | S1:5 S2:5 S3:4(gap-matrix) S4:4 S5:5(release/version) S6:5 S7:5(evidence) |
| PASS/DEFER 矩阵 | Yes（plan/gap-matrix-v2.md） |
| 匹配度 | **matched** — L0 错误分类/时钟/关停信号，crates.io 已发布 xhyper-kernel 0.1.1 |

发现:
- Spec 最深域之一（spec/spec.md 35K，design 47K，plan 25K）
- evidence 目录含完整的 10 轮审查记录、miri/cargo/coverage/mutants 原始证据
- residual-open.txt 维护已知未解决问题清单

### 2.2 testkit

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道目录 + 1 README + 3 spec + plan/archive（15 文件） |
| 填充文件数 (>20 lines) | 14/14 + plan archive 完整 10 轮审查记录 |
| 7行占位数 | 0 |
| Spec 深度分 | **10/10** |
| 对齐文档 | testkit-ssot-alignment.md（174 lines） — Yes |
| S1-S7 维度 | S1:5 S2:5 S3:4 S4:4 S5:5 S6:5 S7:3 |
| 匹配度 | **matched** — ManualClock 族，已验证与 crates.io 版本一致 |

发现:
- spec/spec.md 为全仓最大单项 spec（41K / 2244 lines）
- plan/archive 保留完整 10 轮审查记录（round-01 至 round-10，含 pass2 复查）

### 2.3 contract-testkit

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 0（无独立 SSOT 域） |
| 填充文件数 (>20 lines) | 0 |
| 7行占位数 | N/A |
| Spec 深度分 | **2/10** |
| 对齐文档 | 无独立对齐文档；受 testkit 和 contracts 对齐覆盖 — No |
| 匹配度 | **partial** — 由 SPEC-TESTKIT-002 §3.2 规范，但无独立 SSOT 域或对齐文档 |

发现:
- package `contract-testkit` 提供 Fake/Recording + per-trait conformance suite
- 内容精简（FakeKeyValueStore、FakeEventBus、FakeTxRunner + assert_* suite）
- 规范散落于 testkit 主 spec 和 contracts 相关文档中，缺少集中 SSOT

### 2.4 configx

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 0（`.agents/ssot/configx/` 目录**完全为空**） |
| 填充文件数 (>20 lines) | 0 |
| 7行占位数 | N/A |
| Spec 深度分 | **1/10** |
| 对齐文档 | configx-ssot-alignment.md（156 lines） — Yes |
| 匹配度 | **gap** — 无 spec 但有 crate（L1 内存 KV 存储） |

发现:
- **`.agents/ssot/configx/` 为空目录** — 最严重的 SSOT gap
- 对齐文档存在��填充（156 lines），但无上游 spec 可对
- crate 实际提供 ConfigStore/ConfigDiff/snapshots/subset 等接口
- 对齐文档引用了 "Active SSOT" 概念但指向不存在

### 2.5 schedulex

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道文件 + 1 README + 2 spec |
| 填充文件数 (>20 lines) | 3（spec/spec.md 2.1K, xhyper-schedulex-complete-spec.md 2.1K, README 2.3K） |
| 7行占位数 | 12（design/evidence/gate/goal/matrix/plan/prompt/release/retrospective/review/tasks/test） |
| Spec 深度分 | **3/10** |
| 对齐文档 | schedulex-ssot-alignment.md（64 lines） — Yes |
| 匹配度 | **matched** — L1 任务 ID 登记表，spec 确认 "active SSOT：无真实定时器" |

发现:
- spec 仅有 2.1K，内容声明性 "登记表" 语义与实际代码一致
- pipeline 12 个文件均为 7 行占位

### 2.6 bootstrap

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道文件 + 1 README + 2 spec |
| 填充文件数 (>20 lines) | 3（spec/spec.md 4.6K, xhyper-bootstrap-complete-spec.md 4.6K, README 2.3K） |
| 7行占位数 | 12 |
| Spec 深度分 | **3/10** |
| 对齐文档 | bootstrap-ssot-alignment.md（115 lines） — Yes |
| S1-S7 维度 | S1:4 S2:5 S3:0 S4:3 S5:2 S6:4 S7:0 |
| 匹配度 | **matched** — composition root，实现 ADR-005/ADR-016 |

发现:
- spec 描述了 PlatformContext/AppContext/BootstrappedApp/ShutdownController
- ADR-016 豁免该 crate 的某些要求（启动期依赖组装特殊性）
- pipeline stubs 占位但 spec 有实质内容

### 2.7 evidence

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 3 spec core + plan 目录（26+ 文件） + evidence 目录 |
| 填充文件数 (>20 lines) | 全部管道 + plan 含 10 轮审查 + 2 辅助 spec（evidence-immutability-spec） |
| 7行占位数 | 0 |
| Spec 深度分 | **10/10** |
| 对齐文档 | evidence-ssot-alignment.md（36 lines） — Yes |
| 匹配度 | **matched** — L1 审计证据追加面（EvidenceAppender + InMemory/File 实现） |

发现:
- spec/spec.md 为全仓第二大单体 spec（53K / ~2910 lines）
- 位于 `.agents/ssot/tools/evidence/`（非独立域） — SSOT 组织结构特例
- evidence-immutability-spec.md 提供独立的不可变性保证规范
- correction-schema-v1.json 提供结构化修正元数据格式

### 2.8 observex

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec |
| 填充文件数 (>20 lines) | 2（spec/spec.md 5.9K, xhyper-observex-complete-spec.md 5.9K, README 2.3K） |
| 7行占位数 | 12 |
| Spec 深度分 | **3/10** |
| 对齐文档 | observex-ssot-alignment.md（147 lines） — Yes |
| 匹配度 | **matched** — TracingInstrumentation 包装，实现 contracts::Instrumentation |

发现:
- spec 描述 TracingInstrumentation + PrefixedInstrumentation + CountingInstrumentation
- 明确标注 "非目标：OTEL exporter/flush/shutdown"
- 对齐文档与 spec 一致但 pipeline stub 占位

### 2.9 resiliencx

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + plan 额外文件（alignment-matrix/residual-open） |
| 填充文件数 (>20 lines) | 4（spec 3.8K, README 2.3K, plan/alignment-matrix 1.4K, plan/residual-open 1.1K） |
| 7行占位数 | 11（plan 有额外内容但 plan.md 本体仍为 stub 342 bytes） |
| Spec 深度分 | **4/10** |
| 对齐文档 | resiliencx-ssot-alignment.md（61 lines） — Yes |
| 匹配度 | **matched** — RetryConfig/CircuitBreaker/RateLimiter/Bulkhead |

发现:
- plan 目录微有松动（alignment-matrix-infra + residual-open），但仍以 stub 为主
- spec 描述的 Package stable 未达成状态与 lib.rs "仍未交付：retry budget、package stable" 一致

### 2.10 transport

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec |
| 填充文件数 (>20 lines) | 2（spec 5.3K, README 2.3K） |
| 7行占位数 | 12 |
| Spec 深度分 | **3/10** |
| 对齐文档 | transport-ssot-alignment.md（97 lines） — Yes |
| 匹配度 | **matched** — HTTP/WS 传输边界（HttpDriver/WebSocketDriver） |

发现:
- spec 声明传输边界 "驱动无关" 语义
- pipeline 全 stub 占位

### 2.11 decimalx

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + 20260717/ 子目录（2 文件） + plan 扩展 |
| 填充文件数 (>20 lines) | 8+（spec 10K, 20260717 spec 7.9K, goal 4.9K, plan 6.7K, gap-matrix 2.5K, review 1.4K, matrix 1.3K, goal 1.1K, tasks 2.3K 等） |
| 7行占位数 | ~5（design/evidence/prompt/release/retrospective/test 等小文件） |
| Spec 深度分 | **8/10** |
| 对齐文档 | types-ssot-alignment.md（137 lines，与 canonical 共享） — Yes |
| 匹配度 | **matched** — Decimal 数学类型（ADR-006/007），P0 生产路径已定义 |

发现:
- 20260717/ 日期戳子目录含完整 executable goal + spec
- plan/evidence 目录含消费方库存 (consumer-migration-p0) 和 10x gate 脚本
- code 明确标注 "禁止 f32/f64 参与金额/数量运算；除法必须显式 RoundingStrategy"
- PASS/DEFER 矩阵存在（plan/gap-matrix.md）

### 2.12 canonical

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + 20260717/ 子目录（2 文件） + plan 扩展（15 文件） |
| 填充文件数 (>20 lines) | 10+（spec 5.9K, 20260717 spec 7.4K, goal 6.8K, plan 11K, gap-matrix 3.9K, tasks 4.4K, spec-inventory 5.1K 等） |
| 7行占位数 | ~3（design 226, prompt 175, release 242） |
| Spec 深度分 | **8/10** |
| 对齐文档 | types-ssot-alignment.md（共享） — Yes |
| 匹配度 | **matched** — 跨层 DTO（Order/Tick/Position/OrderBookSnapshot/Money） |

发现:
- Wire commitment 矩阵已明确（v1/v1.1/v1.2/v1.3 wire 版本）
- ts: i64 = Unix epoch 纳秒（CAN-TIME-001 Approved 2026-07-17）
- production-upgrade.md + m3-migration-checklist.md 维护回填过程
- Spec 标注 "已承诺 wire" 而非 "package stable"

### 2.13 contracts

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec |
| 填充文件数 (>20 lines) | 2（spec 5.4K, README 2.2K） |
| 7行占位数 | 12 |
| Spec 深度分 | **3/10** |
| 对齐文档 | contracts-ssot-alignment.md（125 lines） — Yes |
| 匹配度 | **matched** — 契约层 trait 出口（Additive Only），R4 白名单约束 |

发现:
- spec 声明 "只放 trait/type" 语义与代码一致
- lib.rs 明确 "一旦发布不可修改签名，只能新增（Additive Only）"
- pipeline 全 stub 占位

### 2.14 binancex

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec |
| 填充文件数 (>20 lines) | 2（spec 2.9K, README 2.4K） |
| 7行占位数 | 12 |
| Spec 深度分 | **2/10** |
| 对齐文档 | adapters-ssot-alignment.md（224 lines，共享） — Yes |
| 匹配度 | **matched** — VenueAdapter scaffold，内存占位非真实 HTTP 协议 |

发��:
- spec 声明 "scaffold" 状态与代码的 "默认仍为内存占位（非真实交易所协议）" 一致
- lib.rs 有 live-test feature gate 和 mainnet() 构造函数

### 2.15 okxx

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec |
| 填充文件数 (>20 lines) | 2（spec 2.6K, README 2.3K） |
| 7行占位数 | 12 |
| Spec 深度分 | **2/10** |
| 对齐文档 | adapters-ssot-alignment.md（共享） — Yes |
| 匹配度 | **matched** — OKX VenueAdapter scaffold |

### 2.16 redisx

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + 2 evidence 补充文件 |
| 填充文件数 (>20 lines) | 全部管道填充（design 917, evidence 651, gate 569, goal 1.4K, matrix 771, plan 736, prompt 440, release 469, retrospective 430, review 692, tasks 510, test 779）+ spec 3.8K + plan 额外 draft 文件 |
| 7行占位数 | 0（2026-07-22 填充） |
| Spec 深度分 | **6/10** |
| 对齐文档 | redisx-ssot-alignment.md（56 lines） — Yes |
| 匹配度 | **matched** — 生产 Redis 客户端（ConnectionManager + Semaphore 背压） |

发现:
- 2026-07-22 当日 pipeline 全面填充（goal/gate/matrix 等均有实质内容）
- evidence 目录额外包含 ssot-fill-10x-review.md + storage-7-ssot-10x-review.md
- feature pubsub 门控 PubSub 能力

### 2.17 postgresx

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + plan 扩展 |
| 填充文件数 (>20 lines) | 全部管道填充 + spec 3.4K + infra-rs-draft-spec-goal 8.9K |
| 7行占位数 | 0（2026-07-22 填充） |
| Spec 深度分 | **5/10** |
| 对齐文档 | postgresx-ssot-alignment.md（56 lines） — Yes |
| 匹配度 | **matched** — PostgresPool + deadpool-postgres + SQLSTATE→ErrorKind 映射 |

### 2.18 kafkax

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + plan 扩展 |
| 填充文件数 (>20 lines) | 全部管道填充 + spec 3.7K + infra-rs-draft-spec-goal 9.3K |
| 7行占位数 | 0（2026-07-22 填充） |
| Spec 深度分 | **5/10** |
| 对齐文档 | kafkax-ssot-alignment.md（56 lines） — Yes |
| 匹配度 | **matched** — 纯 Rust rskafka，KafkaPool + EventBus at-most-once |

### 2.19 natsx

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + plan 扩展 |
| 填充文件数 (>20 lines) | 全部管道填充 + spec 3.2K + infra-rs-draft-spec-goal 7.8K |
| 7行占位数 | 0（2026-07-22 填充） |
| Spec 深度分 | **5/10** |
| 对齐文档 | natsx-ssot-alignment.md（56 lines） — Yes |
| 匹配度 | **matched** — async-nats Core NATS，NatsPool + EventBus at-most-once |

### 2.20 ossx

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + plan 扩展 |
| 填充文件数 (>20 lines) | 全部管道填充 + spec 3.0K + infra-rs-draft-spec-goal 8.0K |
| 7行占位数 | 0（2026-07-22 填充） |
| Spec 深度分 | **5/10** |
| 对齐文档 | ossx-ssot-alignment.md（56 lines） — Yes |
| 匹配度 | **matched** — 阿里云 OSS Signature V1，OssClient 实现 ObjectStore |

### 2.21 clickhousex

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + plan 扩展 |
| 填充文件数 (>20 lines) | 全部管道填充 + spec 3.1K + infra-rs-draft-spec-goal 7.8K |
| 7行占位数 | 0（2026-07-22 填充） |
| Spec 深度分 | **5/10** |
| 对齐文档 | clickhousex-ssot-alignment.md（56 lines） — Yes |
| 匹配度 | **matched** — ClickHouse HTTP 客户端 端��� 8123，实现 AnalyticsSink |

### 2.22 taosx

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + plan 扩展 |
| 填充文件数 (>20 lines) | 全部管道填充 + spec 3.7K + infra-rs-draft-spec-goal 7.2K |
| 7行占位数 | 0（2026-07-22 填充） |
| Spec 深度分 | **5/10** |
| 对齐文档 | taosx-ssot-alignment.md（56 lines） — Yes |
| 匹配度 | **matched** — TDengine REST 客户端 端口 6041，实现 TimeSeriesStore |

### 2.23 goalctl

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 3 spec core + contracts 子目录（3 文件） + spec 子目录（12 文件） + 20260716（2 文件） + decisions（1 文件） |
| 填充文件数 (>20 lines) | 大量：spec/goalctl-production-spec.md 15K, 20260716/ 完整可执行 spec 31K, decisions/DECISION-PACK-001 21K, goal 9.8K, plan 7K, gap-matrix 4.2K, contracts/CLI-CONTRACT 4.8K |
| 7行占位数 | ~5（基础 stub 如 design 334, matrix 311 等） |
| Spec 深度分 | **9/10** |
| 对齐文档 | tools-ssot-alignment.md（112 lines） — Yes |
| 匹配度 | **matched** — Goal → Contract 编译器（compile/validate/lint），CLI+lib |

发现:
- 20260716 时间戳版本含完整可执行 goal（27K）+ spec（31K）
- decisions/DECISION-PACK-001.md 提供完整的开发决策上下文
- contracts/ 子目录含 CLI-CONTRACT、RUNTIME-STATE、VERSION-CAPABILITY-MATRIX
- spec/ 目录含 12 个中文治理摘要文件

### 2.24 verifyctl

| 维度 | 值 |
|------|-----|
| Spec 文件数 | 14 管道 + 1 README + 2 spec + plan 扩展（5 文件） |
| 填充文件数 (>20 lines) | 大量：spec 30K, goal 14K, plan/infra-rs-draft-spec 30K, plan/infra-rs-draft-verification 21K, plan/infra-rs-draft-goal 14K |
| 7行占位数 | ~8（基础 stub：design 270, gate 131, matrix 137 等短文件，但均有非占位内容） |
| Spec 深度分 | **8/10** |
| 对齐文档 | tools-ssot-alignment.md（共享） — Yes |
| 匹配度 | **matched** — 验证计划/执行/报告（plan/execute/report），可选 evidence 钩子 |

发现:
- goal/goal.md 14K + spec/spec.md 30K 组成坚韧 spec 基础
- plan/infra-rs-draft-* 三文件（2026-07-22 填充）合计 65K，提供了从 goal 到 verification 的端到端草案
- 与 evidence crate 通过 feature "with-evidence" 可选集成

## 3. 聚合统计

### 3.1 Spec 深度分分布

| 分数区间 | 数量 | Crate |
|---------|------|-------|
| 10/10 | 3 | kernel, testkit, evidence |
| 9/10 | 1 | goalctl |
| 8/10 | 3 | decimalx, canonical, verifyctl |
| 5-6/10 | 8 | redisx(6), postgresx(5), kafkax(5), natsx(5), ossx(5), clickhousex(5), taosx(5), contract-testkit(2) |
| 4/10 | 1 | resiliencx |
| 3/10 | 5 | schedulex, bootstrap, observex, transport, contracts |
| 2/10 | 2 | binancex, okxx |
| 1/10 | 1 | configx |
| 0-2/10 | 0 | (gate/testkitx/xtask: SSOT 存在但无 crate，不纳入 crate 统计) |

### 3.2 Stub 域表（仅含已落地 crate）

| 域 | SSOT Pipeline Stub 文件数 | Spec 文件行数 | 备注 |
|-----|--------------------------|-------------|------|
| configx | 0（目录完全为空） | 0 | 最严重 gap |
| schedulex | 12 | 2.1K | spec 存在但 pipeline 全 stub |
| bootstrap | 12 | 4.6K | spec 有内容但 pipeline 全 stub |
| observex | 12 | 5.9K | spec 有内容但 pipeline 全 stub |
| transport | 12 | 5.3K | spec 有内容但 pipeline 全 stub |
| contracts | 12 | 5.4K | spec 有内容但 pipeline 全 stub |
| binancex | 12 | 2.9K | spec 有内容但 pipeline 全 stub |
| okxx | 12 | 2.6K | spec 有内容但 pipeline 全 stub |
| resiliencx | 11 | 3.8K | plan 微有松动但仍以 stub 为主 |

### 3.3 SSOT-crate 匹配表

| 匹配度 | 数量 | 详情 |
|-------|------|------|
| matched | 20 | 所有有 crate 有 SSOT 的域均匹配 |
| partial | 1 | contract-testkit（无独立 SSOT 域） |
| gap | 1 | configx（SSOT 目录为空） |
| **SSOT 存在但无 crate** | 3 | gate（目录为空）、testkitx（spec 1.5K）、xtask（spec 16K） |

### 3.4 对齐文档覆盖

| 状态 | 数量 | 详情 |
|------|------|------|
| 独立对齐文档 | 19 | kernel(219), testkit(174), configx(156), observex(147), types(137), contracts(125), adapters(224), bootstrap(115), tools(112), transport(97), schedulex(64), resiliencx(61) + 7 个 storage(各 56) + evidence(36) |
| 共享对齐文档 | 2 | goalctl + verifyctl 共享 tools-ssot-alignment；decimalx + canonical 共享 types-ssot-alignment |
| 无对齐文档 | 1 | contract-testkit |

## 4. 轮次结论

### 4.1 关键发现

1. **configx SSOT 为空目录** — 最优先修复项。该 crate 作为 L1 内存配置存储已落地生产，但 `.agents/ssot/configx/` 完全不包含任何规范文件，仅有 `docs/ssot/configx-ssot-alignment.md` 充当事实上的代理 spec。

2. **三极分化** — 仓库呈现清晰的三级分布：深度审��域（kernel/testkit/evidence 等 7 个，评分 8-10）、中期填充域（7 storage adapter + contract-testkit，评分 2-6）、和 stub 域（8 个 L1/contracts/exchange，评分 1-4）。

3. **Storage adapter 批量填充** — 2026-07-22 对 7 个 storage adapter 的 pipeline 进行了当日统一填充（design/goal/gate/matrix/plan/prompt/release/retrospective/review/tasks/test 均有实质内容），但从 infrastructure 草案（infra-rs-draft-*）变为正式域 spec 仍需更多工作。

4. **3 个孤儿 SSOT 域** — gate（空目录）、testkitx（pipeline 仅 stub + spec 1.5K）、xtask（spec 16K 但 pipeline 全 stub）有 SSOT 规范但无对应 crate，需确认是否为有意延期。

5. **contract-testkit 无独立 SSOT** — 该 crate 作为 dev-dep only test-support 层，其规范散落于 testkit 主 spec（§3.2）和相关文档，未见独立 SSOT 域。

### 4.2 后续轮次建议

- **R2 正确性**应优先验证 configx、contract-testkit 和 orphan SSOT 域的公开 API 正确性
- **R3 契约完整性**应覆盖 contracts、binancex、okxx 的 trait 语义对齐
- **R9 DEFER 复查**应解决 configx SSOT 缺失 + 3 个 orphan SSOT 域的最终处置
