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
cp -rf /home/workspace/xhyper.rs/.agent/SSOT/kernel  .agents/ssot/
cp -rf /home/workspace/xhyper.rs/.agent/SSOT/testkit .agents/ssot/
cp -rf /home/workspace/xhyper.rs/.agent/SSOT/types   .agents/ssot/
```

## 结论

**全部通过** — 源和目标在文件数量、大小和内容上完全一致。无遗漏、无差异。

> **补充（2026-07-21）**：镜像同步成功 ≠ 本仓实现（R7）。本仓落地以 `Cargo.toml` members + 对齐文档为准。
>
> | 域 | 本仓路径 | 状态 | 对齐文档 |
> |----|----------|------|----------|
> | kernel | `crates/kernel` | **已落地** | [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) |
> | testkit | `crates/testkit` | **core 已落地**；contract-testkit DEFER | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) |
> | types | `crates/types/{decimal,canonical}` | **已落地**；wire/package stable OPEN | [types-ssot-alignment.md](./types-ssot-alignment.md) |
>
> **总览**：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)  
> **非 SSOT 域**：`infra-core` 已从 workspace **移除**（见根 `CHANGELOG`）；它从未属于 kernel/testkit/types 镜像三域。
