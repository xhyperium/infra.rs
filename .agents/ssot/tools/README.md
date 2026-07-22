# tools — 本仓 SSOT

> **SSOT 根**：`.agents/ssot/tools/`  
> **保留层级**：`tools/`（勿展平到 `.agents/ssot/` 根）  
> **状态**：**R7** — 文档 COMPLETE / Phase Approved **≠** 本仓 ship；以 `crates/` + workspace members 为准

## 子域一览

| 子域 | SSOT 路径 | 本仓实现路径 | 本仓状态 |
|------|----------|--------------|----------|
| `evidence` | `.agents/ssot/tools/evidence/` | `crates/evidence` | **最小面已落地**；见 [evidence-ssot-alignment](../../../docs/ssot/evidence-ssot-alignment.md) |
| `goalctl` | `.agents/ssot/tools/goalctl/` | `tools/goalctl` | **workspace member**（#188）；最小 Goal→Contract CLI；[landing](goalctl/plan/infra-rs-landing.md) |
| `xtask` | `.agents/ssot/tools/xtask/` | `tools/xtask`（期望） | **未落地**；`cargo xtask` alias 可忽略 |
| `verifyctl` | `.agents/ssot/tools/verifyctl/` | `tools/verifyctl` | **workspace member**（#188）；最小 plan/execute/report；[landing](verifyctl/plan/infra-rs-landing.md) |

### draft 入库（只读快照）

| 文件 | 说明 |
|------|------|
| `goalctl/plan/infra-rs-draft-goal.md` | 自 `.cargo/draft/goalctl-goal.md` |
| `goalctl/plan/infra-rs-draft-spec.md` | 自 `.cargo/draft/goalctl-spec.md` |
| `verifyctl/plan/infra-rs-draft-goal.md` | 自 `.cargo/draft/verifyctl-goal.md` |
| `verifyctl/plan/infra-rs-draft-spec.md` | 自 `.cargo/draft/verifyctl-spec.md` |
| `verifyctl/plan/infra-rs-draft-verification.md` | 自 `.cargo/draft/verification.md` |

## 硬限制

1. 禁止在 SSOT 树写 `src/`、`Cargo.toml`、`*.rs` 实现副本（C-LINT-007）。
2. README / Review COMPLETE / Phase Approved **不得**单独当作本仓交付证明。
3. 无证据不得宣称 Done / package stable / ship。
4. goalctl 完整 authority plane / verifyctl 全 V0–V3 **未**宣称。

## 验证

```bash
test -f .agents/ssot/tools/README.md
test -f .agents/ssot/tools/evidence/README.md
test -f .agents/ssot/tools/goalctl/README.md
test -f .agents/ssot/tools/xtask/README.md
test -f .agents/ssot/tools/verifyctl/README.md
test -f .agents/ssot/tools/goalctl/plan/infra-rs-landing.md
test -f .agents/ssot/tools/verifyctl/plan/infra-rs-draft-spec.md
test -f .agents/ssot/tools/evidence/spec/spec.md
cargo test -p evidence -p goalctl -p verifyctl --all-targets
cargo run -p goalctl -- doctor
```

**布局对齐：是 · 最小生产 CLI：goalctl/verifyctl 已 member · 禁止假 Done。**

## 对齐

- [docs/ssot/tools-ssot-alignment.md](../../../docs/ssot/tools-ssot-alignment.md)
- [docs/ssot/workspace-ssot-alignment.md](../../../docs/ssot/workspace-ssot-alignment.md)
