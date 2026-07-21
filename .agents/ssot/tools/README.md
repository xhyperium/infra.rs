# tools — 本仓 SSOT

> **SSOT 根**：`.agents/ssot/tools/`  
> **保留层级**：`tools/`（勿展平到 `.agents/ssot/` 根）  
> **状态**：**R7** — 文档 COMPLETE / Phase Approved **≠** 本仓 ship；以 `crates/` + workspace members 为准

## 子域一览

| 子域 | SSOT 路径 | 本仓实现路径 | 本仓状态 |
|------|----------|--------------|----------|
| `evidence` | `.agents/ssot/tools/evidence/` | `crates/evidence`（`xhyper-evidence`） | **最小面已落地**；见 [evidence-ssot-alignment](../../../docs/ssot/evidence-ssot-alignment.md) |
| `goalctl` | `.agents/ssot/tools/goalctl/` | `tools/goalctl`（期望） | **未落地**；规格仅作实现输入 |
| `xtask` | `.agents/ssot/tools/xtask/` | `tools/xtask` / `infra-xtask`（期望） | **未落地**；`cargo xtask` alias 可忽略 |
| `verifyctl` | `.agents/ssot/tools/verifyctl/` | `tools/verifyctl`（期望） | **本仓扩展域**；**未落地** |

## 硬限制

1. 禁止在 SSOT 树写 `src/`、`Cargo.toml`、`*.rs` 实现副本（C-LINT-007）。
2. README / Review COMPLETE / Phase Approved **不得**单独当作本仓交付证明。
3. 无证据不得宣称 Done / package stable / ship。

## 验证

```bash
test -f .agents/ssot/tools/README.md
test -f .agents/ssot/tools/evidence/README.md
test -f .agents/ssot/tools/goalctl/README.md
test -f .agents/ssot/tools/xtask/README.md
test -f .agents/ssot/tools/verifyctl/README.md
test -f .agents/ssot/tools/evidence/spec/spec.md
# 本仓 evidence 实现
cargo test -p xhyper-evidence --all-targets
```

**布局对齐：是 · 全量实现：未宣称 · 禁止假 Done。**
