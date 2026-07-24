# schedulex maintenance evidence（2026-07-23）

Baseline：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`；Beads：`infra-2d9.9.1`。

## 三轮索引

| 轮次 | 证据 | 结论 |
|---|---|---|
| Round 1 | 源码/测试/manifest/历史/SSOT 审计 | 找到治理冲突、非法状态、随机顺序、英文错误与未冻结时间语义 |
| Round 2 | Goal/Design/Spec/Matrix/Test/Gate + 双镜像 | 冻结 registry + explicit tick，保持 timer/distributed NO-GO |
| Round 3 | public seam Red→Green 与下列命令 | 代码切片完成；最终版本、CTK、全仓/PR 门禁仍待前序合并 |

## Red → Green

| Seam | Red exit / 事实 | Green |
|---|---|---|
| 非法 JobId | 101；空 ID 被 `add` 接受 | 0；插入前统一校验 |
| 非法 Schedule | 101；伪造 Cron parsed 被接受 | 0；重新解析并比对 |
| tick 顺序 | 101；`z,a,m` 按随机顺序执行 | 0；Job ID 字典序 |
| list_meta 顺序 | 101；metadata 随机顺序 | 0；Job ID 字典序 |
| 时间回退 | 101；回退 tick 执行到期 Job | 0；忽略且不推进 |
| 中文错误 | 101；详情为 `every_ms must be > 0` | 0；简体中文详情 |
| every interval | 101；off-grid 首次 tick 不执行并可能永久饥饿 | 0；首次执行，随后按 last-fire interval，跨度不补跑 |

失败替换原子性、错误继续、无补跑、重复 ID、cancel、Cron MinuteMatch 与 panic 为 public seam 回归，均 exit 0。

## Residual OPEN / NO-GO

- scoped crate gate、显式 API baseline、workspace build/test/clippy 与 deny 已通过；后续 diff 冻结后必须重跑，不能视为最终 verifier 证据。
- 根 `AGENTS.md` 仍误写 package/registry-only；父 `.9` writer 拥有同一 hunk，已通过 Beads `infra-2d9.9` 留下修正要求，child 在其合入并 rebase 前保持 BLOCKED。
- 默认 public API package 清单/workflow、版本 bump、lock/STATUS、独立最终 verifier 与发布记录仍待完成。
- contract-testkit 阶段必须等待 #256 人工合并并 rebase。
- 后台 timer、async、持久化、完整 cron、分布式调度与 package stable 保持 NO-GO。
