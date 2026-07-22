# 生产就绪判据框架 — 十轮审查基准文档

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-22 |
| 用途 | 十轮 `crates/` SSOT/spec 审查的**统一生产条件标准** |
| 性质 | 只读分析框架；**不等于** Maintainer 签核；**不等于** workspace Production Ready |
| 执行 | Agent Team 两路并行证据盘点 + 十轮分视角重审 |
| 继承 | `docs/report/2026-07-21/*` · `docs/governance/prod-signoff-TEMPLATE.md` · `docs/plans/2026-07-21-core-crates-production-readiness.md` §2 |

---

## 1. 生产就绪分层（L1–L5）— 可度量标准

本仓采用五层判据。**某一层「达成」必须同时满足该层全部硬条件**。Agent **禁止**代签 L5。

| 层 | 名称 | 硬条件（全部满足才算达成） | 证据位置 |
|----|------|---------------------------|----------|
| **L1** | Internal Ready | ① `cargo test --all-targets` 绿 ② `clippy -D warnings` 绿 ③ `fmt --check` 绿 ④ 适用包 `cov-gate-100` 绿 ⑤ 公开 API 无未声明 panic ⑥ `cargo deny` 无未处理 CRITICAL ⑦ 中文用户可见错误 | CI + crate tests + scripts |
| **L2** | Wire Ready | ① committed wire 类型清单冻结 ② `deny_unknown_fields`（或等价拒绝） ③ 双向 golden / fixture ④ N-1 兼容策略文档化 ⑤ 拒绝样例测试 | types/canonical 等 |
| **L3** | Contract Ready | ① trait 语义文档 ② conformance suite ③ **至少一个非 scaffold 验证入口**（真 DB/MQ/交易所或等价） | contracts + adapters + observex |
| **L4** | Platform Ready | ① 支持矩阵文档（`support-matrix.md`） ② MSRV CI ③ public API snapshot 棘轮 ④ 声明平台范围内行为可复现 | docs/api-baselines · governance |
| **L5** | Release Ready | ① Maintainer 人类填写 `prod-signoff-TEMPLATE` ② DEFER 列表冻结 ③ CHANGELOG 发布说明 ④ 不得由 Agent 勾选「已签核」 | docs/plans/releases/*-signoff.md |

**整体 Production Ready** = 各 crate 达到**各自声明目标层**，且应用可交付面完成 L5 人签。  
**STATUS.md 结构完成度不等于生产就绪。**

### 1.1 量化交易「可上线」附加硬条件（QT-Ship）

在 L1–L5 之外，量化交易生产路径还必须满足 **QT-Ship-1…QT-Ship-6**（全部满足才可宣称 quant 可部署面 Ready）：

| ID | 条件 | 说明 |
|----|------|------|
| **QT-Ship-1** | 端到端链路闭合 | bootstrap 能注入 storage + exchange 生产客户端（非仅 Bounded 占位） |
| **QT-Ship-2** | 资金路径安全 | 金额/价格仅 `decimalx` + `checked_*`；禁止 f64 金额 |
| **QT-Ship-3** | 投递语义明确 | 消息路径至少 at-least-once **或** 显式 Accept at-most-once 并有补偿 |
| **QT-Ship-4** | 可观测与关停 | 生产 tracing/metrics 策略 + graceful drain |
| **QT-Ship-5** | 密钥与 TLS | 无明文密钥入仓；外网 TLS 策略可强制 |
| **QT-Ship-6** | Live 证据 | 关键路径存在可复现 live/集成证据（可 `#[ignore]` 但需凭据脚本） |

当前（2026-07-22）**QT-Ship 整体 = NO-GO**（见 synthesis：QT-Ship-1/3/4 等未满足）。

---

## 2. Spec 完整性维度（S1–S7）

每项 0–5 分；**Σ ≥ 28/35 视为「规格平面完整」**，仍可因 DEFER 未达生产层。

| 维度 | 说明 | 5 分锚点 |
|------|------|----------|
| **S1** | 域规格存在 | `.agents/ssot/{domain}/` 有完整 goal/spec/design 或 complete-spec |
| **S2** | 对齐文档 | `docs/ssot/*-ssot-alignment.md` 存在且含本仓结论 |
| **S3** | PASS/DEFER 矩阵 | 明确 PASS 与 DEFER/OPEN 条目 |
| **S4** | 禁止表述 | 明确禁止宣称（如禁止 Production Ready / OTEL 完成） |
| **S5** | 版本/成熟度 | active / partial / scaffold 等与 STATUS 一致 |
| **S6** | 源码对齐 | 规格描述与 `crates/**/src` 接口一致（抽样可证） |
| **S7** | 变更记录 | CHANGELOG / release / 对齐变更表有日期 |

---

## 3. 量化交易应用场景分类（QT-1…QT-7）

每轮审查评估每个 crate 在以下量化交易场景的适用性：

| 场景 | 说明 | 主相关 crate |
|------|------|--------------|
| **QT-1** | 市场数据接入 | transportx · binancex · okxx · canonical |
| **QT-2** | 订单执行 | contracts Venue · binancex · okxx · decimalx |
| **QT-3** | 仓位与风险管理 | decimalx · resiliencx · kernel |
| **QT-4** | 持久化与审计 | postgresx · redisx · kafkax · natsx · evidence · ossx |
| **QT-5** | 配置与调度 | configx · schedulex |
| **QT-6** | 可观测性 | observex · kernel |
| **QT-7** | 数据聚合与分析 | decimalx · canonical · clickhousex · taosx |

取值：**Ready** / **Conditional** / **Gap** / **N/A**。

---

## 4. 代码质量基线

| 指标 | 门禁 |
|------|------|
| `cargo test --all-targets` | 必须绿 |
| `cargo clippy -- -D warnings` | 必须绿 |
| `cargo fmt --check` | 必须绿 |
| `scripts/cov-gate-100.mjs` | 核心 crate 必须 |
| `cargo deny check` | 无未处理 CRITICAL |
| `forbid(unsafe_code)` | 鼓励 |
| STATUS 完成度 | **仅参考，非判据** |

---

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

---

## 6. 审查范围（22 packages under `crates/`）

与 `Cargo.toml` workspace members 中所有 `crates/**` 对齐（不含 `tools/goalctl` · `tools/verifyctl` 作为主体，仅作依赖上下文）：

kernel · testkit · decimalx · canonical · bootstrap · configx · schedulex · evidence · observex · resiliencx · transportx · contracts · contract-testkit · binancex · okxx · redisx · postgresx · kafkax · natsx · ossx · clickhousex · taosx

---

## 7. 报告模板

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

---

## 8. 禁止宣称清单（Agent）

- 「workspace 整体 Production Ready」
- 「Agent 已完成 L5 / 代签 prod-signoff」
- 「STATUS 99% 即等于可生产发布」
- 「exchange 可交易 / first-batch L3 全绿」（除非证据更新）
- 「SSOT COMPLETE 镜像即等于本仓已交付」

---

## 9. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | 初版：十轮审查框架定义 |
| 2026-07-22 | 终版：补 QT-Ship-1…6、22-package 范围、可度量 L1–L5；修复 U+FFFD 乱码 |
