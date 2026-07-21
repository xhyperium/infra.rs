# STATUS 全模块生产就绪 Follow-up 修复清单

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-STATUS-PROD-FOLLOWUP-001` |
| 日期 | 2026-07-21 |
| 输入审计 | [docs/report/2026-07-21/status-modules-production-readiness.md](../report/2026-07-21/status-modules-production-readiness.md) |
| 上游计划 | [PLAN-CORE-PROD-002](./2026-07-21-core-crates-production-readiness.md)（**W0–W5 DONE** · 五核心分层签核） |
| Beads epic | **`infra-s9t`** |
| 状态 | **IN PROGRESS** · 本 PR 批量收敛多项；非 L5 发布批准 |
| 性质 | 对照 W0–W5 语义，扩展 L1 / adapters 阻断收敛 |

---

## 0. 与 PLAN-CORE-PROD-002 的关系

```text
PLAN-CORE-PROD-002 (DONE)
  W0 基线冻结 · W1 decimal · W2 wire · W3 contracts 形状 · W4 mock 入口 · W5 签核
        │
        │  仍 Accept / DEFER：真实后端、二期 trait、非 Linux、应用级平台面
        ▼
本 follow-up (OPEN)  = 审计 STATUS 21 模块后的「应用可生产」缺口
  W0+ 消费面冻结 · W1+/W2+ 残留 · W3 L3 真闭合 · W4 真实后端 · L1 平台 P0 · W5+ 治理
```

| 原波次 | CORE 计划状态 | 本 follow-up 动作 |
|--------|---------------|-------------------|
| **W0** | 已冻结五核心范围 | **W0+**：扩展「可生产消费面」到全 STATUS 模块 + 误用红线 |
| **W1** | decimal L1 签字候选已合入 | **W1+**：deny_unknown / panicking 门禁复核 |
| **W2** | committed v1–v1.3 已合入 | **W2+**：envelope 或兼容矩阵 Accept 文档 |
| **W3** | Fake + conformance 部分 | **W3**：L3 三条件闭合（依赖 W4） |
| **W4** | mock 验证入口 | **W4**：至少一个**非 scaffold** 真实后端；**W4+** exchange 只读 |
| **W5** | L5 GO-with-Accepts | **W5+**：包名漂移、中文错误、持续门禁 |
| — | 五核心范围外 | **L1 平面 P0**：bootstrap / evidence / configx / resiliencx / schedulex / observex / drain |

---

## 1. Beads 树（权威任务源）

Epic：`infra-s9t` — *[STATUS-PROD] 全模块生产就绪阻断收敛*

| Bead | 波次 | P | 标题 | 依赖 |
|------|------|---|------|------|
| `infra-s9t.1` | W0+ | P1 | 冻结可生产消费面清单（trait/DTO/crate 红线） | — |
| `infra-s9t.2` | W4 | **P0** | 至少一个非 scaffold 真实后端验证入口（redis 或 postgres） | ← `.1` |
| `infra-s9t.3` | W3 | P1 | contracts first-batch L3 闭合 | ← `.2` |
| `infra-s9t.4` | L1 | **P0** | bootstrap `require_evidence` release fail-closed | ← `.1` |
| `infra-s9t.5` | L1 | P1 | 关停 drain 合同 + bootstrap 编排钩子 | — |
| `infra-s9t.6` | L1 | P1 | resiliencx async Wait + 默认非阻塞路径 | — |
| `infra-s9t.7` | L1 | P1 | evidence 持久化合同 + configx schema 边界 | — |
| `infra-s9t.8` | L1 | P2 | schedulex / observex 误用红线 | — |
| `infra-s9t.9` | W1+ | P2 | decimalx deny_unknown + panicking 门禁复核 | — |
| `infra-s9t.10` | W2+ | P2 | canonical wire envelope 或兼容矩阵 | — |
| `infra-s9t.11` | W5+ | P2 | 文档 package 名与 Cargo 短名对齐 | — |
| `infra-s9t.12` | W5 | P3 | L1 用户可见错误中文化抽查 | — |
| `infra-s9t.13` | W4+ | P2 | exchange testnet 只读 `server_time` | ← `.2` |
| `infra-s9t.14` | Docs | P1 | adapters scaffold 生产误用警示统一 | — |
| `infra-s9t.15` | Core | **P0** | kernel `wait_timeout` 溢出与 ClockDomain 封闭 | — |
| `infra-s9t.16` | L1 | **P0** | transport 敏感 Debug、deadline 与资源上限 | — |
| `infra-s9t.17` | L1 | P1 | observex subscriber 故障隔离与默认导出闭环 | — |
| `infra-s9t.18` | Docs | P1 | 修正 STATUS `scaffold+mock` false-positive | — |

```bash
bd show infra-s9t
bd children infra-s9t
bd ready
bd update <id> --claim
bd close <id> --reason="..."
```

---

## 2. 按波次执行清单

### 2.1 W0+ — 基线扩展（不可跳过）

| 勾选 | Bead | 任务 | 完成物 | 验收 |
|:----:|------|------|--------|------|
| [ ] | `infra-s9t.1` | 冻结可生产消费面 | `docs/plans/artifacts/prod-consume-surface.md`（新建或扩 inventory） | 列出 allow/deny crate·trait；对照 STATUS 标签 |
| [ ] | （文档） | 链到主报告 §0.3 表 | 本文件 §4 使用矩阵保持同步 | PR 描述引用 epic |

**W0+ 完成标准**：应用集成方能回答「哪些 crate 可以 import，哪些绝对不能当生产后端」。

---

### 2.2 W1+ — decimalx 残留

| 勾选 | Bead | 任务 | 验收 |
|:----:|------|------|------|
| [ ] | `infra-s9t.9` | `Decimal`/`Money` serde `deny_unknown_fields` **或** 文档冻结「忽略 extra」策略 | 测 + WIRE.md |
| [ ] | `infra-s9t.9` | 复核 `check-decimal-no-panicking-ops.mjs` 覆盖生产路径 | CI 仍绿 |

**对照 CORE W1**：不变量 P0 已闭；本波次只硬化 wire 与门禁。

---

### 2.3 W2+ — canonical wire 残留

| 勾选 | Bead | 任务 | 验收 |
|:----:|------|------|------|
| [ ] | `infra-s9t.10` | 方案 A：wire envelope（`schema_version`）**或** 方案 B：对外兼容矩阵 + 升级路径 Accept | ADR 或 `docs/` + 测 |

**对照 CORE W2**：v1–v1.3 committed 已合入；本波次闭合 B-C1。

---

### 2.4 W3 — contracts L3（依赖 W4）

| 勾选 | Bead | 任务 | 验收 |
|:----:|------|------|------|
| [ ] | `infra-s9t.3` | first-batch 语义文档齐全 | `docs/contracts/*` 覆盖签字子集 |
| [ ] | `infra-s9t.3` | conformance 对 Fake **与** 真实入口（W4）跑通 | CI 证据 |
| [ ] | `infra-s9t.3` | 二期 trait 标 `experimental` 或补文档 | README 诚实 |

**L3 三条件**（全部满足才可讨论签字）：

1. 语义合同  
2. conformance suite  
3. **至少一非 scaffold 验证入口**（=`infra-s9t.2`）

---

### 2.5 W4 / W4+ — 真实后端

| 勾选 | Bead | 任务 | 建议路径 | 验收 |
|:----:|------|------|----------|------|
| [ ] | `infra-s9t.2` | 非 scaffold 入口 | **优先** `redisx` KV 或 `postgresx` Tx | feature 门控 SDK + optional CI job |
| [ ] | `infra-s9t.13` | exchange 只读 | testnet `server_time` + 真 `HttpDriver` | `#[ignore]` live；默认离线绿 |
| [ ] | `infra-s9t.14` | scaffold 红线 | 9 adapter README 统一警示 | 类型名≠客户端 |

**对照 CORE W4**：原 W4 交付了 **mock** 入口；本波次闭合审计 **DEFER-1 / WS-P0-1**。

---

### 2.6 L1 平面 P0/P1（CORE 范围外）

| 勾选 | Bead | 严重度 | 任务 | 验收 |
|:----:|------|--------|------|------|
| [ ] | `infra-s9t.4` | **P0** | `require_evidence` release fail-closed | release 测：缺 evidence → Err |
| [ ] | `infra-s9t.5` | P1 | drain 合同 + 编排钩子 | docs + 可选 API |
| [ ] | `infra-s9t.6` | P1 | resiliencx async Wait | feature 测；禁止默认 block async |
| [ ] | `infra-s9t.7` | P1 | evidence 持久化合同 / configx schema 边界 | 文档红线 + 最小实现或机检 |
| [ ] | `infra-s9t.8` | P2 | schedulex 名实 / observex OTEL 边界 | README/AGENTS |
| [ ] | `infra-s9t.15` | **P0** | kernel timeout 溢出与 ClockDomain 伪造入口 | 极值/跨域回归测试 + SSOT |
| [ ] | `infra-s9t.16` | **P0** | transport 脱敏、deadline、资源上限 | HTTP/WS 故障与超限测试 |
| [ ] | `infra-s9t.17` | P1 | observex subscriber 隔离与导出闭环 | panic/阻塞/flush 端到端测试 |

---

### 2.7 W5+ — 治理与诚实性

| 勾选 | Bead | 任务 | 验收 |
|:----:|------|------|------|
| [ ] | `infra-s9t.11` | `xhyper-*` vs Cargo 短名对齐 | 文档命令可复制运行 |
| [ ] | `infra-s9t.12` | L1 中文错误抽查 | 清单或豁免 |
| [ ] | `infra-s9t.18` | STATUS adapter 分类误报 | 生成器回归测试 + STATUS 刷新 |
| [ ] | （人工） | 应用面 L5 再签 | 仅当 W3+W4+L1-P0 闭合后 |

---

## 3. 建议执行顺序

```text
并行批次 A（无依赖 / ready）:
  .1 W0+ 消费面 · .5 drain 文档 · .6 async Wait · .7 evidence/config ·
  .8 名实红线 · .9 decimal · .10 wire · .11 包名 · .12 中文 · .14 adapter 警示

串行关键路径:
  .1 W0+ ──► .2 W4 真实后端 ──► .3 W3 L3
                     │
                     └──► .13 W4+ exchange 只读

并行关键路径:
  .1 W0+ ──► .4 bootstrap require_evidence
```

**最短路径到「可讨论应用级 L3」**：`.1` → `.2` → `.3` + `.4` + `.14`。

---

## 4. 使用矩阵（集成方 · 冻结前草稿）

| 场景 | 允许 | 禁止 |
|------|------|------|
| 单测 / harness | bootstrap + Fake/InMemory + ManualClock | 把 mock 当集成测完成 |
| 同步批处理弹性 | resiliencx + 显式 Wait | async 服务默认 `retry_fn`（block） |
| HTTP 客户端 | transportx（自管超时） | 宣称 TLS 平台矩阵完成 |
| 金额 | decimalx `checked_*` | panicking 运算符资金路径 |
| Wire DTO | canonical committed 清单 | 未 committed 当跨服务契约 |
| 配置 | configx 仅进程内 KV | 唯一生产配置源 |
| 审计 | 非 InMemory 实现出现前 | InMemoryEvidence 当合规 |
| 调度 | — | schedulex 当 timer/cron |
| 存储 / 交易所 | W4 真实入口合入前 | `*Adapter` 当生产客户端 |

正式冻结后以 `infra-s9t.1` 完成物为准。

---

## 5. 非目标（本 follow-up 仍不做）

- 把 9 个 adapter 做成完整产品（只需验证入口 + 红线）
- crates.io 公开发布
- 全量 monorepo archgate（**OOS**，#164）/ domain_exchange 移植
- 性能 / 长稳 / 集群故障注入基准（另计划）
- 在未签应用面 L5 前改 README 为「Production Ready」

---

## 6. 完成定义（Epic `infra-s9t`）

全部满足：

1. [ ] `infra-s9t.2` + `infra-s9t.3` 闭合（L3 可讨论）  
2. [ ] `infra-s9t.4` 闭合（release fail-closed）  
3. [ ] `infra-s9t.1` + `infra-s9t.14` 闭合（消费面 + 误用红线）  
4. [ ] 其余 P1 子任务 closed 或显式 Defer-with-sign  
5. [ ] 主报告 §8 签字清单可更新为「应用 follow-up 进行中 / 完成」  
6. [ ] `bd epic status infra-s9t` 进度可解释  

---

## 7. 追溯

| 资源 | 路径 |
|------|------|
| 主审计报告 | [status-modules-production-readiness.md](../report/2026-07-21/status-modules-production-readiness.md) |
| partials | [report/2026-07-21/_partials/](../report/2026-07-21/_partials/) |
| 五核心计划（DONE） | [2026-07-21-core-crates-production-readiness.md](./2026-07-21-core-crates-production-readiness.md) |
| 签核 | [releases/0.3.0-signoff.md](./releases/0.3.0-signoff.md) |
| Beads | `bd show infra-s9t` |

---

*创建：2026-07-21 · Agent Team 审计后落地 · epic `infra-s9t`*
