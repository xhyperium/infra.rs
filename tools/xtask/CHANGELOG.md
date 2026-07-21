# Changelog — xtask

## [Unreleased]

### Fixed
- **lint-deps fail-closed I/O**（xhyper-gmd agent-safe subset）：
  - `check_r6` 的 `collect_rs_files` 在 `read_dir` 失败时不再 `if let Ok` 静默跳过，而是返回 `Err`。
  - `check_r6` 的 `fs::read_to_string` 失败时不再 `let Ok(_) else { continue }`，而是返回 `Err`。
  - 上述两处历史上是 R6 假阴性源；现统一 fail-closed。
- **`approval-auto` apply 冻结**（xhyper-4do agent-safe subset）：
  - `--apply` 必须显式授权（CLI `--authorized-by <DECISION_ID>` 或 env
    `XHYPER_APPROVAL_AUTO_APPROVED`），且引用 registry 中已 APPROVED 的 decision；
    未授权 → fail-closed。
  - 移除命令对 automation policy 的强制自授（mode / standing_authorization /
    ai_may_invoke_auto）；registry 必须由人审批就位，否则 bail。
  - `rfc3339_now` / `rfc3339_plus_days` / `git_head` / owner detect 失败一律
    fail-closed；删除 `2026-07-14T00:00:00Z` / `2026-10-12T00:00:00Z` / `0*40` /
    `ZoneCNH` 固定 fallback。
  - apply 路径在最早阶段获取 `.approval-auto.lock.d/approval-auto.apply.lock`
    （`File::create_new` 原子语义）；并发 apply 第二个直接 fail-closed。
  - apply 写入产物全部走 `tempfile::NamedTempFile::persist` 原子写。

### Added
- **lint-deps R1-R6 + ADR-007 负向 fixture**（xhyper-gmd agent-safe subset）：
  - 新增 `tests/lint_deps_fixtures.rs`，覆盖 R1（testkit normal-dep 禁止）、R3（L1 互依）、
    R5（domain 三平级互斥）、R1.2（domainx 单向链）、ADR-007（decimalx → canonical）、
    R6（跨层 `pub use`）的正/负 fixture。
  - 新增 I/O fail-closed 测试：不可读 `.rs` 文件触发 `check_r6` 报错；空 workspace
    与损坏 Cargo.toml 均 fail-closed。

### Changed
- **architecture TOML 反序列化**：新增 `architecture_toml` 模块；`migration` /
  `inventory-ssot` 以 `toml`+`serde` 解析 `workspace.toml` / `migration.toml` /
  `dependency.toml`（schema_version、defaults、字段顺序无关），退役行扫描。

### Fixed
- `ci aggregate`：无 `--decisions-file` 时 **FAIL**（禁止默认全绿）；仅 `--synthetic-smoke` 可显式合成 RUN_PASS。
- `ci aggregate`：`NOT_APPLICABLE` 无 reason、`REUSED` 无 attestation、裸 `SKIP` → FAIL。
- `ci aggregate`：v2 输出统一为结构化 decision 并在返回前校验 shipped schema；未知状态/类型与未完整验证的 REUSED 全部失败关闭。
- `ci locks`：空 `msrv`/`primary` pin → FAIL；`tools.lock` 必须与 `install-cargo-tool` 真实 VERSION 对齐。
- `ci locks`：新增隔离验证用 `--root`；CLI 负测不再改写仓库内 lockfile，消除 nextest 并发污染。
- `ci drift`：缺 checked-in `policy-table.md` → DRIFT（与 hand-edit 并列 fail-closed）。
- `ci drift`：新增隔离验证用 `--root`；unit/CLI 负测不再改写仓库内 `.github/ci/generated/`（temp fixture），消除 nextest 多进程下 `drift_missing_policy_table_is_drift` 与 hand-edit 竞态假 MATCH/假 PASS。

### Changed
- `ci flake`：按 Spec §13.1 严格解析 typed TOML registry；缺表/空表为 SAFE_OFF，非法字段或日期失败关闭，默认日期改取系统 UTC。
- `ci fingerprint`：改为必需完整 typed JSON 输入，并以仓库 V1 canonical encoding 生成候选摘要；删除会制造 `empty-plan` hash 的旧快捷参数，输出明确标记 provenance 未验证且不可 reuse。
- `ci reuse` / Aggregate：统一为单一 structural candidate validator；v1 Attestation 可选 `reuse_inputs`，九项 predicate 仅作未验证诊断，所有路径继续回退 `RUN`，不启用生产 reuse。
- `ci chaos`：新增 Spec §26 精确 20 项 typed manifest 与闭集隔离 driver；分别报告 `gate_ok`、`coverage_complete`、11 个 EXECUTABLE 与 9 个可见 STUB，禁止测试名或混合故障冒充执行证据。
- Shadow workflow `ci-required-shadow`：从 `ci run` 生成 decisions_file 再 aggregate；接入 locks/drift/verify-runner/chaos。Runner P0 止血后 `verify-runner` 只接受 root-owned attestation + live resource/tool observation，PLACEHOLDER/空工具/通用或禁用 labels 均 `INFRA_FAILURE`；workflow 改为 manual-only exact-label，外部 runner 合规前不自动执行 PR。
- `tools.lock.toml` 版本对齐 install-cargo-tool（nextest 0.9.140 / cargo-deny 0.20.2 / machete 0.9.2 / llvm-cov 0.8.7）。
- Spec §26 可执行负向 fixtures 扩容（invalid_na / reused / skip / cancelled / unknown 等）。

### Added
- `ci`：补齐 `run` / `aggregate` / `reconcile` / `metrics` 命令面；实现接线前以
  `NOT_IMPLEMENTED` 非零退出，禁止 stub 假绿。`doctor` 同时要求 Lane 与 Aggregate Evidence schema。
- `crate-standard`：新增 `crate-name-prefix` rule_id（CRATE_STANDARD §3.1.1 / §7.5）。新 crate `[package] name` 缺 `xhyper-` 前缀 → ERROR；现有 crate allowlist（`EXISTING_NON_PREFIXED`，33 个）命中 → WARN。lib/bin name 不检查。现有测试 fixture 同步改用前缀以保持原本规则测试聚焦。
- `approval-auto`：IG-1 single_accountable_owner + machine co-sign
- `semver-check`：接线 `cargo semver-checks check-release`（baseline tag）

### Changed
- `approval-check`：支持 schema v2 `approval_automation` 策略


本文件记录 xtask 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。
版本号以对应 Cargo.toml 的 [package] version 为准；递增规则见根 AGENTS.md「版本管理」。

## [Unreleased]

### 新增

- `inventory-ssot`：INFRA-004 SSOT/拓扑只读漂移检测（architecture↔cargo members、dependency 策略、SQL 浮点、target-dir）。
- `evidence-check`：INFRA-003 Evidence 最小字段校验；`--self-test` 验证篡改后必须失败；并加载 `schemas/jsonschema/fixtures/negative/*.json` 断言每个样本失败。
- `approval-check`：只读校验 IG-1 唯一 JSON registry、20 个决策、九份 required proposals、subject
  revision/SHA-256、固定角色/依赖策略、批准 provenance 与 D-06b Evidence；默认 fail-closed，
  `--registry-only` 只验登记，INFRA-009 可信外部 Review 回读落地前 gate 永不误绿。
- `semver-check`：INFRA-008 脚手架；`cargo-semver-checks` 缺失或无 baseline 时 fail-closed（不伪绿）；文档见 `docs/semver-checks.md`。
- `drift-detect`：INFRA-060 只读漂移检测，输出故意类别 taxonomy；`auto_repair` 恒 false。
- `schema_lite`：Evidence JSON Schema 子集匹配（required/type/enum/pattern/additionalProperties）。

### 修复

- `approval-check` 的 SHA-256 依赖固定为兼容仓库 MSRV 1.80 的 `sha2 0.10.9`；D-15 未批准前不借草案抬高工具链下限。

### 变更

- `evidence-check`：接入 schema 子集校验、扩展 secret 模式、自测覆盖缺字段/篡改 hash/secret；可选扫描 `evidence/infrastructure`。
- `inventory-ssot`：unit path/id 唯一性、scripts/workflows `./target/` 硬编码扫描、intentional categories 字段。
- `approval-check`：`APPROVED` 且无自然人 approver / 缺 provenance 字段 → fail（`machine-approved-forbidden` / `approved-without-approvals`）。
- `crate-standard` 新增 `crate-readme-fields`（WARN）：README 须启发式覆盖「非职责」与「限制」字段。


### 变更

- 文档：README 补充「非职责」与「限制与安全」（crate-standard wave2）。


### 新增

- 建立初始文档骨架（CHANGELOG / AGENTS / README / docs）。
- 新增只读 `crate-standard` inventory/checker，并提供 `--check` 渐进 ERROR 门禁。
- `crate-standard` 新增规则投影：
  - `crate-independent-version`（ERROR）：禁止 `version.workspace`，要求三段式字面量版本；
  - `crate-changelog-unreleased`（WARN）：CHANGELOG 存在时须含 `[Unreleased]`；
  - `crate-root-docs`（WARN）：存在的 lib/bin 入口须有 crate 级 `//!` 文档。
