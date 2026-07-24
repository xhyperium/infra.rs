# ssot/ — SSOT 对齐与同步

## 职责

记录本仓对 `.agents/ssot/**`（本仓域规格 SSOT）的**对齐状态**与**实现落地矩阵**。

> 规格写 COMPLETE / Stable **≠** 本仓已宣称可 ship。以 `crates/` + `cargo metadata` 为准。  
> **archgate / `.architecture`：OOS**（PR #164）— 本仓明确不移植。

## 收录标准

**应放入本目录：**

- 各域 `*-ssot-alignment.md` 对齐矩阵
- 工作区总览 `workspace-ssot-alignment.md`
- 同步操作手册与同步完整性报告

**不应放入本目录：**

- 通用治理条文 → `.agents/rules/`
- CI/配置状态快照 → `docs/status/`
- 域规格正文 → `.agents/ssot/`（本仓 SSOT；变更走 PR，勿与外仓 rsync 无脑覆盖）

## 阅读顺序

1. [workspace-ssot-alignment.md](workspace-ssot-alignment.md) — members 地图与总览
2. [SSOT_SYNC_OPS.md](SSOT_SYNC_OPS.md) — 如何从上游同步
3. [SSOT_SYNC_REPORT.md](SSOT_SYNC_REPORT.md) — 镜像是否完整（≠ 实现落地）
4. 各域 `*-ssot-alignment.md` — 本仓落地差距
5. 生产就绪审计（五核心 crate）：[../report/2026-07-21/core-crates-production-readiness.md](../report/2026-07-21/core-crates-production-readiness.md)

> **2026-07-21 跟进**：PR #98 P0/P1；L5 [`0.3.0-signoff`](../plans/releases/0.3.0-signoff.md)；四包 GO [#159](https://github.com/xhyperium/infra.rs/pull/159) · tag [`v0.3.0-four-crates`](https://github.com/xhyperium/infra.rs/releases/tag/v0.3.0-four-crates)；kernel 内部发布 [#163](https://github.com/xhyperium/infra.rs/pull/163)。  
> **STATUS-PROD epic `infra-s9t` 已闭合**（[#166](https://github.com/xhyperium/infra.rs/pull/166)–[#168](https://github.com/xhyperium/infra.rs/pull/168) · [#172](https://github.com/xhyperium/infra.rs/pull/172)）：L1 P0、redis live KV、contracts L3 子集、exchange `server_time` 入口。  
> 对齐同步文档：[#174](https://github.com/xhyperium/infra.rs/pull/174) · closeout [#175](https://github.com/xhyperium/infra.rs/pull/175) · 行动树 **CLOSED**。  
> **2026-07-22 draft 生产落地**（[#188](https://github.com/xhyperium/infra.rs/pull/188)–[#191](https://github.com/xhyperium/infra.rs/pull/191)）：storage×7 默认客户端 + live/bench；goalctl/verifyctl members；`build-foundationx-env.mjs`。见 [draft-gap-matrix.md](./draft-gap-matrix.md)。  
> 规格 COMPLETE / STATUS 结构分 / epic closed / P0 生产入口 **仍不等于** workspace 整体 Production Ready / L5 / crates.io。详见各 `*-ssot-alignment.md`、[SSOT_SYNC_REPORT](./SSOT_SYNC_REPORT.md)、[双栏报告](../report/2026-07-21/seven-l1-contracts-dual-bar-readiness.md)。

## 文档

| 文档 | 说明 |
|------|------|
| [workspace-ssot-alignment.md](workspace-ssot-alignment.md) | Workspace 总览 |
| [SSOT_SYNC_OPS.md](SSOT_SYNC_OPS.md) | 同步操作手册 |
| [SSOT_SYNC_REPORT.md](SSOT_SYNC_REPORT.md) | 同步完整性报告 |
| [kernel-ssot-alignment.md](kernel-ssot-alignment.md) | kernel |
| [testkit-ssot-alignment.md](testkit-ssot-alignment.md) | testkit |
| [configx-ssot-alignment.md](configx-ssot-alignment.md) | configx |
| [schedulex-ssot-alignment.md](schedulex-ssot-alignment.md) | schedulex |
| [types-ssot-alignment.md](types-ssot-alignment.md) | types |
| [bootstrap-ssot-alignment.md](bootstrap-ssot-alignment.md) | bootstrap |
| [adapters-ssot-alignment.md](adapters-ssot-alignment.md) | adapters 九域汇总 |
| [redisx-ssot-alignment.md](redisx-ssot-alignment.md) | redisx 生产 P0 |
| [postgresx-ssot-alignment.md](postgresx-ssot-alignment.md) | postgresx 生产 P0 |
| [kafkax-ssot-alignment.md](kafkax-ssot-alignment.md) | kafkax 生产 P0 |
| [natsx-ssot-alignment.md](natsx-ssot-alignment.md) | natsx 生产 P0 |
| [ossx-ssot-alignment.md](ossx-ssot-alignment.md) | ossx 生产 P0 |
| [clickhousex-ssot-alignment.md](clickhousex-ssot-alignment.md) | clickhousex 生产 P0 |
| [taosx-ssot-alignment.md](taosx-ssot-alignment.md) | taosx 生产 P0 |
| [contracts-ssot-alignment.md](contracts-ssot-alignment.md) | contracts |
| [observex-ssot-alignment.md](observex-ssot-alignment.md) | observex |
| [resiliencx-ssot-alignment.md](resiliencx-ssot-alignment.md) | resiliencx |
| [transport-ssot-alignment.md](transport-ssot-alignment.md) | transport |
| [evidence-ssot-alignment.md](evidence-ssot-alignment.md) | evidence |
| [tools-ssot-alignment.md](tools-ssot-alignment.md) | tools（evidence/goalctl/xtask/verifyctl） |
| [draft-gap-matrix.md](draft-gap-matrix.md) | `.cargo/draft` → 本仓 P0 落地快照 |
| [gap-matrix.md](gap-matrix.md) | 同主题（含 live 入口） |

上级索引：[docs/README.md](../README.md)。

## 同步流程（强制）

修改对齐文档前 **必须** 先拉取最新 origin，避免 origin 新提交覆盖本地更改：

```bash
git pull --rebase origin main
```

之后再编辑 `*-ssot-alignment.md` 或 `gap-matrix.md`，最后：

```bash
git add docs/ssot/
SKIP_VERSION_CHECK=1 git commit -m "docs(ssot): sync alignment to current state"
```

## 变更（近期）

| 日期 | 说明 |
|------|------|
| 2026-07-22 | **#178** 独立 `contract-testkit` 落地；Fake 在 `crates/test-support/contracts`；对齐/同步报告已纠偏 |
| 2026-07-22 | **#188–#191** storage×7 生产路径 + goalctl/verifyctl + live env；workspace/adapters/tools 对齐文刷新 |
| 2026-07-24 | 治理对齐修正：7 处版本漂移修正（bootstrap/clickhouse/nats/oss/postgres/taos/configx）；skills 投影同步 + check-skills-sync.mjs CI 校验；workspace-ssot-alignment v2.0 审计日期刷新 |
| 2026-07-22 | storage×7 分 package 对齐文 + SSOT 层实质化（非占位） |
