# SSOT 同步完整性报告

**日期**: 2026-07-21 02:03 UTC
**源**: `/home/workspace/xhyper.rs/.agent/SSOT/`
**目标**: `/home/workspace/infra.rs/.agents/ssot/`

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

## 结论

**全部通过** — 源和目标在文件数量、大小和内容上完全一致。无遗漏、无差异。

> **补充（2026-07-21）**：镜像同步成功 ≠ 本仓实现（R7）。本仓落地以 `Cargo.toml` members + 对齐文档为准。
>
> | 域 | 本仓路径 | 状态 | 对齐文档 |
> |----|----------|------|----------|
> | kernel | `crates/kernel` | **已落地** | [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) |
> | testkit | `crates/testkit` | **core 已落地**；contract-testkit DEFER | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) |
> | types | `crates/types/{decimal,canonical}` | **已落地**；wire/package stable OPEN | [types-ssot-alignment.md](./types-ssot-alignment.md) |
> | infra 八域 | `.agents/ssot/{bootstrap,configx,gate,observex,resiliencx,schedulex,testkitx,transport}` | **仅镜像**，未宣称 crate 落地 | 见下节 |
> | adapters 九域 | `.agents/ssot/adapters/*` + `crates/adapters/**` | **镜像已注册**；crate **scaffold** | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
>
> **总览**：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)  
> **非 SSOT 域**：`infra-core` 已从 workspace **移除**（见根 `CHANGELOG`）；它从未属于 kernel/testkit/types 镜像三域。

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

> 镜像 COMPLETE ≠ 本仓 crate 已落地。上述域当前仅为只读 SSOT；本仓 `crates/` 是否实现以 `Cargo.toml` members 为准。

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

> 镜像 COMPLETE ≠ 本仓业务实现。本仓 9 个 adapter crate 为 scaffold（#42）；状态见
> [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)。

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
| evidence | SSOT 已就位；crate 最小面已落地（`xhyper-evidence`） |
| goalctl | SSOT 已就位；**无** crate |
| xtask | SSOT 已就位；**无** crate |
| verifyctl | SSOT 已就位（本仓扩展）；**无** crate |

> 文档 COMPLETE ≠ 本仓业务实现。对齐见 [tools-ssot-alignment.md](./tools-ssot-alignment.md)。

## 本仓裁定更新（2026-07-21 · #164）

- **archgate / `.architecture`：OOS** — 本仓明确不移植；对齐见 `docs/ssot/kernel-ssot-alignment.md` 与 PR #164。
- 从外仓 `rsync -a --delete` 时**不得**无脑覆盖已落地的本仓 OOS / 对齐裁定；同步后须重跑对齐抽查。
