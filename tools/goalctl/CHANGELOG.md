# Changelog — xhyper-goalctl

## [Unreleased]

### Fixed

- 移除未使用依赖 `anyhow`（R0 machete `--with-metadata` 真实扫描检出；随 xhyper-dqg/#820 一并落地）

## [0.1.1] - 2026-07-16

### Fixed

- Phase 1.1 Truth Hardening: committed-view artifact/reconcile/compile; no directory→VERIFIED/OK; compile commit/tree bind; ApprovalRecord content validation; `--trust-level` CLI flag

### Fixed

- `artifact inspect` 强制仓库边界：拒绝 absolute / `..` / symlink 逃逸（`GC-ARTIFACT-PATH` → POLICY）
- envelope 严格 schema（enum / pattern / additionalProperties）、module 一致性、重复/畸形 control block、非法 legacy status
- `compile` fail-closed：source_commit / tree_id 仅接受 40 位小写 hex；拒绝仓外 scope、
  归一化后 allowed∩prohibited、空 validation command、无审批的 protected asset；
  TaskPack 输出剥离 schema 禁止字段（`verification` / `covers`）；
  默认 `--module` 的 allowed_paths 来自 repository index 的 `implementation_root`。
- `resolve` / `index` 绑定 committed subject：`resolve` 仅从 `git ls-tree`/`git show commit:path` 读 authority；禁止 dirty worktree 污染却盖 HEAD `source_commit`
- `index` 在 `Cargo.toml`/`Cargo.lock` dirty 时 fail-closed（`GC-DIRTY-CARGO-MANIFEST`）；非 HEAD subject 拒绝 live cargo metadata
- authority policy：拒绝 unsupported major 与 `unknown_fields: deny` 下的未知字段
- `APPROVED_CR` 晋升要求 ApprovalRecord（`approval_id` + `subject_digest`）；仅 Markdown Status 不得晋升
- status / module 过滤改为精确匹配，消除 substring 误晋升

## [0.1.0] — 2026-07-16

### Added

- Phase 1 complete read-only MVA:
  - `version` / `doctor` / `index`
  - `resolve` (Authority Snapshot；rank 仅来自 `authority-policy.yaml`)
  - `artifact inspect|index` (strict/mixed/legacy)
  - `reconcile` (五维 ModuleStatus + verdict；同强度冲突不伪成功)
  - `compile` (Task Pack；P0/P1 无验证方式 / allowed∩prohibited 失败)
- CLI-CONTRACT exit codes 与 `GC-*` diagnostics
- state-dir：XDG 默认或 `--state-dir`；拒绝 Cargo target
- 禁止 `.config/goal`

### Non-goals (unchanged)

- Agent Writer / auto PR / Cutover / required CI / `.config/goal` 控制面

## [0.1.0-dev] — 2026-07-16

- 开发过程标签；已由 0.1.0 取代
