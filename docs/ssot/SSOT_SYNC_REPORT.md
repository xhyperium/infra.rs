# SSOT 同步完整性报告

**镜像基线日期**: 2026-07-21 02:03 UTC（kernel/testkit/types 与上游 diff=0 时的快照）  
**实现对齐刷新**: 2026-07-22（`contract-testkit` 独立 crate 叙事纠偏 · #178；承接 2026-07-21 `infra-s9t` · #166–#168 · #172 · #174 · #175）  
**源（镜像）**: `/home/workspace/xhyper.rs/.agent/SSOT/`  
**目标**: `/home/workspace/infra.rs/.agents/ssot/`

> **读法**：上文「文件数/差异=0」只证明**镜像树**曾与上游一致；**crate 落地**以下方「实现落地」与各 `*-ssot-alignment.md` 为准（R7）。  
> 本轮 **未** 重跑外仓 `rsync`；只刷新本仓对齐叙事。

## 摘要

| 指标 | kernel | testkit | types | 合计 |
|------|--------|---------|-------|------|
| 文件数 | 69 | 50 | 63 | **182** |
| 目录数 | 17 | 14 | 32 | **63** |
| 大小 | 756 KB | 460 KB | 460 KB | **1,676 KB** |
| 差异 | 0 | 0 | 0 | **0** |

## 各目录明细

### kernel

| 子目录 | 说明 |
|--------|------|
| `design/` | 设计文档（DESIGN-KERNEL-002.md, design.md） |
| `evidence/` | 证据记录（2026-07-13, 2026-07-14, 含突变测试输出和覆盖率报告） |
| `gate/` | 门控规则 |
| `goal/` | 目标定义 |
| `matrix/` | 矩阵映射 |
| `plan/` | 计划文档（审批包、差距矩阵、计划正文） |
| `prompt/` | 提示词模板 |
| `release/` | 发布说明 |
| `retrospective/` | 回顾记录 |
| `review/` | 审查结果 |
| `spec/` | 规格文档（SPEC-KERNEL-001, spec.md, xhyper-kernel-complete-spec.md） |
| `tasks/` | 任务清单 |
| `test/` | 测试策略 |

### testkit

| 子目录 | 说明 |
|--------|------|
| `design/` | 设计文档（DESIGN-TESTKIT-002.md） |
| `evidence/` | 证据记录 |
| `gate/` | 门控规则 |
| `goal/` | 目标定义 |
| `plan/` | 计划文档（含 26 个档案文件，10 轮审查发现） |
| `spec/` | 规格文档（SPEC-TESTKIT-001, spec.md, xhyper-testkit-complete-spec.md） |

### types

| 子目录 | 说明 |
|--------|------|
| `canonical/` | canonicalx 类型规范（含 20260717 对齐计划、审批包、验证责任矩阵） |
| `decimal/` | decimalx 类型规范（含 20260717 对齐计划、消费者迁移证据、10x 门控脚本） |

## 验证方法

```bash
# 文件数量对比
/bin/ls -lR <src> | grep -c '^-'  # 源
/bin/ls -lR <dst> | grep -c '^-'  # 目标

# 内容差异对比
diff -rq <src> <dst>  # 返回空 = 完全一致

# 大小对比
du -sh <src> <dst>  # 字节级一致
```

## 同步命令

```bash
# 删除感知同步（推荐）；保留 adapters/ 层级（infra 已展平到 ssot 根）
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/kernel/   .agents/ssot/kernel/
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/testkit/  .agents/ssot/testkit/
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/types/    .agents/ssot/types/
# infra（8 子域，源为 upstream/infra/，目标已展平到 ssot 根）
for sub in bootstrap configx gate observex resiliencx schedulex testkitx transport; do
  rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/infra/$sub/ .agents/ssot/$sub/
done
rsync -a --delete /home/workspace/xhyper.rs/.agent/SSOT/adapters/ .agents/ssot/adapters/
```

## 本仓实现落地（≠ 镜像同步）

镜像完整性只证明 `.agents/ssot/**` 与上游一致或已本地化。  
**实现落地**以各域 `*-ssot-alignment.md` + `crates/` 为准。

2026-07-21（PR #98 **已合入 main**）核心五 crate 生产就绪闭合后，本仓实现对齐文已同步更新：

- [workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- [types-ssot-alignment.md](./types-ssot-alignment.md)
- [contracts-ssot-alignment.md](./contracts-ssot-alignment.md)
- [kernel-ssot-alignment.md](./kernel-ssot-alignment.md)
- [testkit-ssot-alignment.md](./testkit-ssot-alignment.md)
- [bootstrap-ssot-alignment.md](./bootstrap-ssot-alignment.md)

审计报告：[docs/report/2026-07-21/core-crates-production-readiness.md](../report/2026-07-21/core-crates-production-readiness.md)。

### 2026-07-21 · infra-s9t 落地同步（≠ 再跑 rsync）

Epic **`infra-s9t` 18/18 closed**（#166–#168 · #172）。对齐文 #174；报告/计划 closeout #175。  
本仓**实现对齐文**已按源码刷新；**未**要求重跑外仓镜像 rsync。

| 域 | 落地增量 | 对齐文档 |
|----|----------|----------|
| contracts | L3 子集 KV+Instr；CT-9 部分 PASS | [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) |
| adapters / redisx | `RedisLiveKv` + live_kv_conformance | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| adapters / binancex·okxx | 公共 `server_time` 解析 + ignore live | 同上 |
| resiliencx | `AsyncWait` / `retry_async` | [resiliencx-ssot-alignment.md](./resiliencx-ssot-alignment.md) |
| transport | P0 Debug/上限硬化 | [transport-ssot-alignment.md](./transport-ssot-alignment.md) |
| bootstrap | `require_evidence` release fail-closed | [bootstrap-ssot-alignment.md](./bootstrap-ssot-alignment.md) |
| evidence | `FileEvidenceAppender` | [evidence-ssot-alignment.md](./evidence-ssot-alignment.md) |
| configx | `require_keys` | [configx-ssot-alignment.md](./configx-ssot-alignment.md) |
| observex | TracingInstrumentation（L3 Instr 入口） | [observex-ssot-alignment.md](./observex-ssot-alignment.md) |
| schedulex | registry only + 误用红线 | [schedulex-ssot-alignment.md](./schedulex-ssot-alignment.md) |

权威总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)。  
行动树 CLOSED：[status-modules-prod-followup.md](../plans/2026-07-21-status-modules-prod-followup.md)。  
**禁止**：把 epic closed 读成 workspace Production Ready / L5。

## 结论

**镜像完整性（历史快照）**：kernel/testkit/types 与源在文件数/内容上曾 diff=0（见文首表）。  
**实现落地（当前）**：以 members + 下表为准——**不是**「全部仅镜像」。

> **R7**：镜像 COMPLETE ≠ 本仓 ship。

| 域 | 本仓路径 | 实现状态 | 对齐文档 |
|----|----------|----------|----------|
| kernel | `crates/kernel` | **已落地** · L1+L4 内部发布 | [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) |
| testkit | `crates/testkit` | **core 已落地**（ManualClock 族） | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) |
| contract-testkit | `crates/test-support/contracts` | **独立 crate 已落地**（Fake + per-trait suite；仅 dev-dep）· #178 | 同上 · [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) |
| types | `crates/types/{decimal,canonical}` | **已落地**；package stable **OPEN** | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| bootstrap / configx / observex / resiliencx / schedulex / transport | 各 `crates/*` | **已落地**（各有 DEFER 边界） | 分域对齐文 |
| gate / testkitx | 仅 `.agents/ssot/*` | **仅镜像**，无 crate | — |
| adapters 九域 | `crates/adapters/**` | scaffold 为主 + **redis live / public time** | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| contracts | `crates/contracts` | trait 出口 + `venue_gate` + **L3 子集**；Fake/suite 在 `contract-testkit` | [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) |
| tools/evidence | `crates/evidence` | 最小面 + `FileEvidenceAppender` | [evidence-ssot-alignment.md](./evidence-ssot-alignment.md) |
| tools/goalctl·xtask·verifyctl | — | **无** workspace member | [tools-ssot-alignment.md](./tools-ssot-alignment.md) |

> **总览**：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)  
> **非 SSOT 域**：`infra-core` 已从 workspace **移除**；**archgate OOS**（#164）。  
> **禁止**：把「全 trait / live 后端仍 DEFER」读回成「独立 `contract-testkit` crate 未落地」。

### 2026-07-22 · contract-testkit 实现对齐刷新（≠ 再跑 rsync）

**触发**：本仓已有 workspace member `contract-testkit`（path `crates/test-support/contracts`，PR #178），
与 SPEC-TESTKIT-002 §3.2 一致；但本报告结论表曾误写「独立 contract-testkit **DEFER**」，
与 `workspace-ssot-alignment.md` / `testkit-ssot-alignment.md` / `SSOT.md` R7 冲突。

本轮 **未** 重跑外仓 `rsync`；只修正本仓实现对齐叙事。

| 项 | 裁定 |
|----|------|
| package | `contract-testkit`（`-p contract-testkit`） |
| path | `crates/test-support/contracts` |
| plane | test-support（仅 dev-dep；不得进 production graph） |
| 能力 | Fake/Recording + per-trait `assert_*` suite；`tests/suite_self_tests.rs` |
| 仍 DEFER | 全 trait 深度 conformance、真实后端 profile、integration harness（§3.3）；**不是**「独立 crate 未建」 |
| 权威规格 | `.agents/ssot/testkit/spec/spec.md` §3.2 |

---

## 补充：infra 平面镜像（2026-07-21，v0.3.18 最终确定）

infra 的 8 个子域已从 `.agents/ssot/infra/` 展平到 `.agents/ssot/` 根：

**源**: `/home/workspace/xhyper.rs/.agent/SSOT/infra/`（上游仍保留 `infra/` 层级）  
**目标**: `/home/workspace/infra.rs/.agents/ssot/`（本仓展平，各子域为直接子目录）  
**命令**: `for sub in ...; do rsync -a --delete $SRC/infra/$sub/ $DST/$sub/; done`

| 域 | 文件数 | 目录数 | 大小 | 与源 diff |
|----|--------|--------|------|-----------|
| bootstrap | 16 | 14 | 128K | 0 |
| configx | 16 | 14 | 128K | 0 |
| gate | 34 | 14 | 268K | 0 |
| observex | 16 | 14 | 128K | 0 |
| resiliencx | 16 | 14 | 120K | 0 |
| schedulex | 16 | 14 | 120K | 0 |
| testkitx | 16 | 14 | 120K | 0 |
| transport | 16 | 14 | 128K | 0 |
| **合计** | **146** | — | — | **0** |

> 镜像 COMPLETE ≠ 本仓 crate 已落地。上表统计的是 **镜像树**；bootstrap/configx/observex/resiliencx/schedulex/transport **已有** crate，gate/testkitx **仅镜像**。以 members + 分域对齐文为准。

---

## 补充：adapters 平面镜像（2026-07-21）

**源**: `/home/workspace/xhyper.rs/.agent/SSOT/adapters/`  
**目标**: `/home/workspace/infra.rs/.agents/ssot/adapters/`（**保留 `adapters/` 层级**）  
**命令**: `rsync -a --delete …/SSOT/adapters/ .agents/ssot/adapters/`

| 域 | 文件数 | 目录数 | 大小 | 与源 diff |
|----|--------|--------|------|-----------|
| exchange/binance | 16 | 14 | 120K | 0 |
| exchange/okx | 16 | 14 | 120K | 0 |
| storage/clickhouse | 16 | 14 | 120K | 0 |
| storage/kafka | 16 | 14 | 120K | 0 |
| storage/nats | 16 | 14 | 120K | 0 |
| storage/oss | 16 | 14 | 120K | 0 |
| storage/postgres | 16 | 14 | 120K | 0 |
| storage/redis | 16 | 14 | 120K | 0 |
| storage/taos | 16 | 14 | 120K | 0 |
| **合计** | **144** | — | **1.1M** | **0** |

> 镜像 COMPLETE ≠ 本仓业务实现。本仓 9 个 adapter 以 scaffold 为主；**redisx live KV** 与 exchange **只读 server_time**
> 为有限真路径（#168/#172）。状态见 [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)。

---

## 补充：contracts 平面镜像（2026-07-21）

**源**: `/home/workspace/xhyper.rs/.agent/SSOT/contracts/`  
**目标**: `/home/workspace/infra.rs/.agents/ssot/contracts/`  
**命令**: `rsync -a --delete …/SSOT/contracts/ .agents/ssot/contracts/`

| 指标 | 值 |
|------|-----|
| 文件数 | 16 |
| 与源 diff | 0 |

> 见 [contracts-ssot-alignment.md](./contracts-ssot-alignment.md)。

---

## 补充：tools 本仓 SSOT（2026-07-21）

**路径**: `.agents/ssot/tools/`（保留 `tools/` 层级）

| 子域 | 本仓状态 |
|------|----------|
| evidence | SSOT 已就位；crate 最小面 + `FileEvidenceAppender`（`xhyper-evidence`） |
| goalctl | SSOT 已就位；**无** crate |
| xtask | SSOT 已就位；**无** crate |
| verifyctl | SSOT 已就位（本仓扩展）；**无** crate |

> 文档 COMPLETE ≠ 本仓业务实现。对齐见 [tools-ssot-alignment.md](./tools-ssot-alignment.md)。

## 本仓裁定更新（2026-07-21 · #164）

- **archgate / `.architecture`：OOS** — 本仓明确不移植；对齐见 `docs/ssot/kernel-ssot-alignment.md` 与 PR #164。
- 从外仓 `rsync -a --delete` 时**不得**无脑覆盖已落地的本仓 OOS / 对齐裁定；同步后须重跑对齐抽查。


### 2026-07-22 · 七包双栏 STATUS 100% structure

本仓实现对齐刷新（**未**重跑外仓 rsync）：七包 STATUS 结构 100% + 行覆盖 100% 门禁；生产宣称仍受 `prod-consume-surface` 与 L3 子集边界约束。
