# 生产就绪判据框架 — 十轮审查基准文档

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-22 |
| 用途 | 十轮 spec 审查的统一判据基准 |
| 性质 | 只读分析框架，不等同于 Maintainer 签核 |

---

## 1. 生产就绪分层（L1–L5）

基于现有治理体系（见 `docs/report/2026-07-21/core-crates-production-readiness.md` §2），每轮审查对每个 crate 按以下五层打分：

| 层 | 含义 | 判据 |
|----|------|------|
| **L1 Internal Ready** | 进程内库，可被 workspace 内部 crate 安全依赖 | CI 持续绿（test/clippy/fmt/doc）；cov-100 行覆盖门禁（`scripts/cov-gate-100.mjs`）；公开 API 无未声明 panic；依赖治理 `cargo deny check` 通过 |
| **L2 Wire Ready** | 跨进程/落盘/wire 格式已版本化 | committed wire 类型清单冻结；`deny_unknown_fields`；双向 golden 测试；N-1 兼容门禁；拒绝样例 |
| **L3 Contract Ready** | trait 语义闭合，有非 scaffold 验证入口 | 每个生产 trait 有：语义文档 + conformance suite + 至少一个非 scaffold 验证入口（真 DB/真 MQ/真交易所） |
| **L4 Platform Ready** | 平台矩阵与 API baseline | MSRV CI 通过；支持矩阵文档；public API snapshot 门禁；semver diff 检查 |
| **L5 Release Ready** | 人工 Maintainer 签核 | 维持 `prod-signoff-TEMPLATE.md`，Agent 禁止代签 |

## 2. Spec 完整性维度

每轮审查对每个 crate 的 SSOT spec 按以下维度打分（每项 0-5）：

| 维度 | 说明 |
|------|------|
| **S1. 域规格存在** | `.agents/ssot/{domain}/` 是否有 SPEC-*.md 或规格文档 |
| **S2. 对齐文档** | `docs/ssot/*-ssot-alignment.md` 是否存在且最新 |
| **S3. PASS/DEFER 矩阵** | 是否明确列出已实现能力（PASS）与延迟项（DEFER） |
| **S4. 禁止表述** | 是否明确列出禁止宣称的能力 |
| **S5. 版本/成熟度标注** | 是否有 L1/L2/L3/partial/active 等成熟度标签 |
| **S6. 源码对齐** | 规格描述是否与 `crates/{crate}/src/` 实际接口一致 |
| **S7. 变更记录** | 是否有日期标注的变更历史 |

## 3. 量化交易应用场景分类

每轮审查评估每�� crate 在以下量化交易场景的适用性：

| 场景 | 说明 | 涉及 crate |
|------|------|-----------|
| **QT-1. 市场数据接入** | WebSocket/HTTP 实时行情、深度、K 线 | `transportx`、`binancex`、`okxx` |
| **QT-2. 订单执行** | 下单、撤单、改单、批量操作 | `contracts`（Venue）、`binancex`、`okxx` |
| **QT-3. 仓位与风险管理** | 仓位计算、风控限额、熔断 | `decimalx`、`resiliencx` |
| **QT-4. 持久化与审计** | 订单/成交/Tick 落库、审计证据 | `canonical`、`evidence`、adapters/storage/* |
| **QT-5. 配置与调度** | 策略参数、定时任务、热更新 | `configx`、`schedulex` |
| **QT-6. 可观测性** | 链路追踪、指标采集、告警 | `observex`、`kernel`（错误/关停） |
| **QT-7. 数据聚合与分析** | Tick→K线、指标计算、回测数据 | `decimalx`、`canonical`、`clickhousex`、`taosx` |

每轮对每个场景评定：**Ready**（满足生产需求）、**Conditional**（有条件可用）、**Gap**（存在缺口）、**N/A**（不适用）。

## 4. 代码质量基线

每轮审查对每个 crate 评估以下代码质量指标：

| 指标 | 门禁 |
|------|------|
| `cargo test --all-targets` | 必须全绿 |
| `cargo clippy -- -D warnings` | 必须全绿 |
| `cargo fmt --check` | 必须全绿 |
| `scripts/cov-gate-100.mjs` | 必须通过（适用于核心 crate） |
| `cargo deny check` | 必须通过 |
| `[lints] workspace = true` | 鼓励 |
| `forbid(unsafe_code)` | 鼓励 |
| STATUS 完成度 | 参考指标，非判据 |

## 5. 审查轮次设计

| 轮次 | 视角侧重 |
|------|----------|
| R1 | 基线扫描 — 统计所有 crate spec 存在性与覆盖 |
| R2 | 正确性 — 公开 API panicking、不变量、边界 |
| R3 | 契约完整性 — trait 语义、conformance、真入口 |
| R4 | 兼容性 — wire/DTO 版本、diff、migration |
| R5 | 可运维性 — 错误分类、监控、关停、drain |
| R6 | 安全性 — 反序列化、资源消耗、依赖风险 |
| R7 | 量化交易场景逐一评估 |
| R8 | 跨 crate 集成风险 |
| R9 | DEFER 项复查与累积 gap |
| R10 | 最终综合裁定 |

## 6. 报告模板

每轮报告结构：

```markdown
# Round N: [视角名称] — 模块 Spec 审查

| 字段 | 值 |
|------|-----|
| 轮次 | N/10 |
| 视角 | [视角名称] |
| 日期 | 2026-07-22 |

## 1. 审查摘要

## 2. 逐 crate 分析

### 2.1 crate-name
- spec 完整性: [S1–S7 打分]
- 生产就绪分层: [L1–L5 判定]
- 量化交易场景: [QT-1–QT-7 判定]
- 发现/缺口:

## 3. 跨 crate 观察

## 4. 轮次结论
```

## 7. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | 初版：十轮审查框架定义 |
