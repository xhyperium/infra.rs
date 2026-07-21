# xtask

xhyper.rs 内部工具链。以 Cargo 二进制 crate 形式提供 workspace 级维护命令。

## 子命令

| 命令 | 作用 |
|------|------|
| `lint-deps` | 依赖图 R1–R6 |
| `no-new-gate` | PLAN-GATE-RETIRE-001：冻结 runtime `xhyper-gate` 新增使用 |
| `gen-structure` / `migration` / `crate-standard` | 结构与标准盘点（`gen-structure` 按架构分层写根 `STRUCTURE.md`） |
| `inventory-ssot` | INFRA-004 SSOT/拓扑只读漂移 |
| `evidence-check` | INFRA-003 Evidence schema/脱敏/自测 |
| `approval-check` | IG-1 审批 registry（`--registry-only` 只验结构） |
| `semver-check` | INFRA-008 cargo-semver-checks 探测（缺工具非 0） |
| `drift-detect` | INFRA-060 只读漂移 + 类别清单（禁止自动修复） |

## 用法

```bash
cargo run -p xhyper-xtask -- lint-deps --json
cargo run -p xhyper-xtask -- no-new-gate
cargo run -p xhyper-xtask -- no-new-gate --json
cargo run -p xhyper-xtask -- inventory-ssot --json
cargo run -p xhyper-xtask -- evidence-check --self-test --json
cargo run -p xhyper-xtask -- approval-check --registry-only --json
cargo run -p xhyper-xtask -- semver-check --json
cargo run -p xhyper-xtask -- drift-detect --json
```

全局 `--json` 输出机器可读报告。

## 内部结构

- `allowed_matrix` / `classify` / `lint_deps`：依赖纪律。
- `inventory_ssot` / `drift_detect`：SSOT 与只读漂移。
- `evidence_check` / `schema_lite`：Evidence 校验。
- `approval_check`：IG-1 registry。
- `semver_check`：API 兼容探测脚手架。

## 定位

仅 workspace 内部使用，不发布到 crates.io。修改校验规则时须同步更新 spec §2。

## 非职责

- 不发布到 crates.io；不承载业务域逻辑。
- 不自行创造架构政策（只机械投影已批准规则）。
- 不替代 clippy/test/coverage 等工程门禁。
- **不**自动修复漂移；**不**由 AI 批准 D-id 或标 WP ACCEPTED。

## 限制与安全

- `lint-deps` / `crate-standard` / `inventory-ssot` / `drift-detect` 为只读检查。
- `semver-check` 在工具或 baseline 缺失时必须非 0（禁止假 PASS）。
- 规则变更必须先改 `docs/standards` 或更高权威，再改本工具。
- Evidence 记录禁止明文 secret；见 `schemas/jsonschema/evidence-record.schema.json`。

