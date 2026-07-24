# tools/evidence — 历史重定向入口

> **状态**：历史重定向；本目录不再持有 active current-state spec。
> **canonical SSOT**：`.agents/ssot/evidence/spec/spec.md`，镜像为
> `.agents/ssot/evidence/spec/xhyper-evidence-complete-spec.md`。
> **实现**：`crates/infra/evidence`（package `evidence` 0.1.1）。

PR #233 已将 evidence 的 active 规格迁移到顶层 `.agents/ssot/evidence/`。保留本入口仅用于兼容历史链接和战役材料；这里的 goal/plan/review/release 等文件只可作为历史上下文，不能覆盖 canonical current-state 规格，也不能单独证明 ship、package stable 或合规产品就绪。

权威落地边界见：

- `.agents/ssot/evidence/spec/spec.md`
- `docs/ssot/evidence-ssot-alignment.md`
- `docs/ssot/tools-ssot-alignment.md`

验证：

```bash
test -f .agents/ssot/evidence/spec/spec.md
cmp .agents/ssot/evidence/spec/spec.md \
  .agents/ssot/evidence/spec/xhyper-evidence-complete-spec.md
node scripts/quality-gates/check-ssot-current-state.mjs
```

禁止在本目录重新建立第二份 active spec；迁移期引用应直接改指 canonical 路径。
