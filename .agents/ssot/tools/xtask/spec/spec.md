# xtask 实现契约

## 0. 文档状态与权威顺序

本文定义位于 `tools/xtask/` 的内部工具链 crate `xtask`；它细化但不替代仓库权威文档。

权威顺序由高到低：

1. [`CONSTITUTION.md`](../../../../../CONSTITUTION.md)，尤其 Article IV、VI、VII 与 §7.3；
2. [`docs/architecture/spec.md`](../../../../../docs/architecture/spec.md)，尤其 §2（依赖规则 R1–R6）、§3（路径约定）；
3. [`adr/`](../../../../../docs/architecture/adr) 下已批准的 ADR；
4. 本契约。

本文的 **必须**、**不得**、**应当**、**可以** 表示本层级规范要求。若与更高权威冲突，以更高权威为准。

### 0.1 主张标签

- **证据（Evidence）**：更高权威直接规定。
- **推论（Inference）**：满足证据所需的最小实现后果，不构成新的公开契约。
- **未知（Unknown）**：现有权威尚未裁定；不得将其静默实现为稳定公开 API。

## 1. 目的与非目标

### 1.1 目的

**证据（spec §2、§3）：** `xtask` 是 infra.rs workspace 的内部工具链 crate，负责可重复的仓库结构检查，包括校验 workspace 依赖图是否符合 spec §2 定义的 R1–R6 依赖方向铁律，以及 §3 的路径分层约定。

本 crate 必须提供：

- `lint-deps` 子命令：扫描 workspace 全部 member crate 的依赖关系，报告违规；
- `crate-standard` 子命令：只读盘点 workspace 与已发现的 legacy/quarantine package，并按 canonical crate standard 报告结构 finding；
- `approval-check` 子命令：只读验证 IG-1 唯一审批 registry 的决策/提案集合、subject SHA-256、
  角色、批准记录、依赖与 Evidence，并以 fail-closed 结果报告 gate readiness；
- crate 分层分类：按 `Cargo.toml` 路径前缀判定 crate 所属 Layer；
- 跨层允许矩阵：显式声明每个 Layer 允许依赖的目标 Layer 集合。

### 1.2 非目标

`xtask` 不得包含：

- 依赖许可证 / 安全漏洞检查（归 `cargo-deny`，见 CI `deny` job）；
- 未使用依赖检测（归 `cargo-machete`，见 CI `machete` job）；
- 代码 lint（归 `clippy`，见 CI `clippy` job）；
- 格式化（归 `rustfmt`，见 CI `fmt` job）；
- 版本号 bump / CHANGELOG 同步 / 发版自动化（归 `scripts/version.mjs`，见 ADR `cargo-release-adoption.md` 方案 B）；
- 推测性的便利命令、脚手架生成或通用任务运行器（Constitution Article IV）。

**推论：** `xtask` 是只读、可重复的仓库结构校验器，不演化成通用 task runner。

## 2. 分层模型（classify.rs）

### 2.1 Layer 枚举

**证据（spec §3）：** workspace crate 按路径分层。

| Layer | 路径前缀 | 语义 |
|---|---|---|
| `Kernel` | `crates/kernel/`、`crates/infra/evidence/`、`crates/testkit/` | L0 根与测试/证据边界；gate 为 OOS |
| `Types` | `crates/types/` | 跨层共享 DTO |
| `Contract` | `crates/contracts/` | 契约层 trait 出口 |
| `Infra` | `crates/infra/`（不含 gate） | L1 基础设施 |
| `Storage` | `crates/adapters/storage/` | 存储适配器 |
| `Exchange` | `crates/adapters/exchange/` | 交易所适配器 |
| `Domain` | `crates/domain/` | L2.5 领域值对象 |
| `Services` | `crates/services/` | 能力服务 |
| `Apps` | `apps/` | 组合根 |
| `XTask` | `tools/xtask/` | 工具链自身，不受业务规则约束 |
| `Legacy` | `legacy/` | ADR-008 过渡期豁免 xlib 规则 |
| `Unknown` | 不匹配以上 | 规避检测，应报告 |

### 2.2 分类规则

**证据（spec §3）：** 按 `manifest_path` 前缀判定。

- 路径含 `/tools/xtask/` 或以 `/tools/xtask/Cargo.toml` 结尾 → `XTask`；
- 路径含 `/crates/kernel/`、`/crates/infra/evidence/` 或 `/crates/testkit/` → `Kernel`；
- 以此类推；
- 路径兜底失败时，按 crate 名识别 `xtask` / `xlibgate` / `xlib_harness` / `xlib_evidence` / `quant`；
- 均不匹配 → `Unknown`。

当前可执行矩阵还包含 `Services` 与 `Apps` 的上层依赖方向；详细允许集合以
`tools/xtask/src/allowed_matrix.rs` 为准。架构漂移诊断在 monorepo 历史中以
`.architecture/policies/dependency.toml` 为声明源（**infra.rs 不维护 `.architecture`；本仓不以 archgate / 该路径为验收**）。

## 3. 跨层允许矩阵（allowed_matrix.rs）

**证据（spec §2 R1/R1.1/R2/R2.1/R4）：** normal/build 依赖必须满足允许矩阵。

| from \ → | Kernel | Types | Contract | Infra | Storage | Exchange | Domain | XTask |
|---|---|---|---|---|---|---|---|---|
| **Kernel** | ✓ | | | | | | | |
| **Types** | ✓ | ✓ | | | | | | |
| **Contract** | ✓ | ✓ | | | | | | |
| **Infra** | ✓ | ✓ | ✓ | ✓ | | | | |
| **Storage** | ✓ | ✓ | ✓ | ✓ | ✓ | | | |
| **Exchange** | ✓ | ✓ | ✓ | ✓ | | ✓ | | |
| **Domain** | ✓ | ✓ | | | | | ✓ | |
| **XTask** | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

空格表示禁止依赖。`XTask` 可依赖所有层（工具链自身不受业务规则约束）。

## 4. 规则组（lint_deps.rs）

### 4.1 R1 — 测试设施仅允许 dev-dependency

`xlib_harness` / `testkitx` 作为 normal 依赖引用时，除 `XTask` / `Kernel` / `Infra` 外均违规。

### 4.2 R1/R2/R4 — 跨层允许矩阵

normal 依赖不在 `allowed_targets(from)` 内 → 违规。

### 4.3 R3/R3.1 — L1 互依禁止

`Infra` 层 crate 之间禁止直接依赖（`bootstrap` 豁免，ADR-005）。

### 4.4 R2/R2.1 — 适配器同层互依禁止

`Storage` / `Exchange` 层内 crate 禁止互相依赖。

### 4.5 R5 — Domain 三平级互斥

`domain_market` / `domain_macro` / `domain_exchange` 互相独立，禁止依赖。

### 4.6 R1.2 — domainx 单向链

`domainx` 禁止反向依赖 `domain_market` / `domain_macro` / `domain_exchange`。

### 4.7 ADR-007 — decimalx 基础数值层

`decimalx` 禁止依赖 `canonical`。

### 4.8 R6 — `check_r6`：禁止跨层 `pub use` 具体实现

**证据（现状）**：`lint_deps.rs::check_r6` 已实现 R6 的源码级检查：递归扫描每个非
`Types`/`Legacy` 层 crate 的 `src/` 目录，逐行匹配以 `pub use` 加空格开头、紧跟
`crate_name::` 的语句；若 `crate_name` 解析到 `Infra`/`Storage`/`Exchange` 层且与当前
crate 不同层，判定为 R6 违规。`/types/` 层与 `Legacy`（`stdio`/`quant`，ADR-008 过渡期）豁免。

**已知局限（未知/未实现，ADR-009 记录）**：当前实现是最小的逐行文本匹配，不处理：

- glob 导入（`pub use foo::*`）；
- 别名导入（`pub use foo::Bar as Baz`）；
- 大括号分组导入（`pub use foo::{A, B}`）；
- `pub(crate) use` 等受限可见性重导出；
- 跨多行的 `use` 语句；
- "先在本 crate 内部 `use`，再从本 crate 重导出"的间接转发。

这些遗漏只会造成**假阴性**（漏报违规），不会造成假阳性；在补齐前，`lint-deps` 通过不能被
解释为"已完整证明 R6 合规"，只能证明"未检出已知形态的违规"。是否将 `check_r6` 拆分为独立
子命令 `lint-pub-use`、以及上述局限的具体解析方案与测试契约，见
[ADR-009](../../../../../docs/architecture/adr/009-r6-enforcement-boundary.md)（Proposed）。

### 4.9 `crate-standard`：crate 结构 inventory/checker

**证据：** [`docs/standards/CRATE_STANDARD.md`](../../../../../docs/standards/CRATE_STANDARD.md) 定义 workspace、legacy、target 类型及 ERROR/WARN/MANUAL 边界。本子命令只机械投影已批准且可判定的条款，不自行创造政策。

发现与分类：

1. 从仓库根运行 `cargo metadata --no-deps --format-version 1`，以 `workspace_members` 识别 workspace package；
2. 将仓库内发现但不属于 `workspace_members` 的 legacy/quarantine manifest 单独分类，不套用 workspace library ERROR；
3. 根据 metadata target `kind` 区分 library 与 binary；同时具有两类 target 时分别记录；
4. 空集合、非 workspace manifest、路径含空格和 metadata 失败必须有确定结果或非零错误，禁止 panic；
5. 输出按 scope、manifest path、crate 名和 finding rule id 稳定排序，重复运行必须逐字一致。

每个 crate 记录至少包含：crate 名、manifest path、scope、target kind、publish、features、标准结构事实与 findings。每个 finding 包含 rule id、`ERROR`/`WARN`/`MANUAL`、证据路径（可定位时含行号）和 suggested action。

#### 已投影 rule_id（workspace package；legacy 非 member 仅 MANUAL）

| rule_id | level | 触发条件 |
|---|---|---|
| `crate-required-file` | ERROR | 缺 `README.md` / `CHANGELOG.md` / `AGENTS.md` / `docs/` 目录，或 lib/bin target 根文件不存在；`docs/` 仅 `.gitkeep` 仍满足存在性 |
| `crate-independent-version` | ERROR | package 的 `Cargo.toml` **文件字面量**使用 `version.workspace = true`（或等价 workspace 继承），或没有三段式 `version = "X.Y.Z"`（semver 粗匹配 `\d+\.\d+\.\d+`）。不得以 cargo metadata 解析出的版本代替字面量 |
| `crate-name-prefix` | ERROR（新 crate）/ WARN（现有 crate） | `[package] name` 不以 `xhyper-` 前缀。**现有 crate allowlist**：`EXISTING_NON_PREFIXED`（38 个：archgate（monorepo historical name, not infra.rs member） / binance / bootstrap / canonical / clickhousex / configx / contracts / decimalx / domain_exchange / domain_macro / domain_market / domainx / evidence / evidence-cli / evidence_file / evidence_legacy / evidence_memory / evidence_postgres / evidence_signer / gate / kafkax / ledger / market_data / marketd / natsx / observex / okx / ossx / postgresx / redisx / resiliencx / risk_engine / schedulex / schema_codegen / taosx / testkit / transportx / xtask），命中 → WARN；未命中且无前缀 → ERROR。证据 = `[package] name` 行号。lib/bin name 通过 `[lib] name` 解耦，不检查 lib/bin name。首次发布 crates.io 前必须重命名（`publish = false` 仅作内部包的不阻断） |
| `crate-changelog-unreleased` | WARN | `CHANGELOG.md` **存在**且内容（大小写不敏感）不含 `[Unreleased]`；文件缺失时不重复报（由 `crate-required-file` 覆盖） |
| `crate-root-docs` | WARN | 每个存在的 lib/bin 入口 `src_path` 中缺少行首（可空白）`//!` crate 级文档；文件不存在时不报此规则 |
| `crate-readme-fields` | WARN | `README.md` **存在**且启发式缺少「非职责」（`非职责` / `non-goals` / `out of scope`）或「限制」（`限制` / `limitations` / `constraints` / `安全说明`）字段；文件缺失时不重复报；**不得**升 ERROR |
| `legacy-quarantine-review` | MANUAL | `legacy/**/Cargo.toml` 且非 `workspace_members`；不套用 workspace ERROR/WARN 规则 |

行为契约：

- `--check` **仅**在存在 `ERROR` 时非零退出；`WARN`/`MANUAL` 完整输出但不阻断；
- 只读扫描；findings 按 `(evidence_path, rule_id)` 稳定排序，重复运行逐字一致；
- `error.rs`、`config.rs`、integration test、bench、example、evidence 不得因缺少空壳而报 ERROR；
- feature/API/运行时语义不做 blanket 判定；`taosx` 的 `rest`/`native` 互斥按 ADR-003 保留，不得误报；
- 不解析 Rust AST、不检查 public API baseline、不生成模板、不修改任何 crate 文件。

CLI 契约：

```bash
# 稳定 Markdown inventory
cargo run -p xtask -- crate-standard

# 稳定 JSON inventory
cargo run -p xtask -- --json crate-standard

# 仅存在 ERROR 时非零退出；WARN/MANUAL 仍完整输出
cargo run -p xtask -- crate-standard --check
```

实现必须复用现有 `cargo_metadata`、`serde_json` 与标准库，不新增第三方依赖。正常报告写 stdout，诊断写 stderr；输入/metadata 错误返回非零。测试至少覆盖 workspace/legacy、library/binary、空集合、路径空格、稳定排序与重复输出、缺失必需文件、`version.workspace` ERROR、CHANGELOG 无 Unreleased WARN、入口缺 `//!` WARN、README 缺非职责/限制 WARN、完整 README 字段不报该 WARN、WARN-only 不阻断及只读性。

### 4.10 `approval-check`：IG-1 审批 registry/verifier

**证据：** 基础设施执行计划 §5–§6 与治理权限矩阵要求自然人角色、不得自批、AI 不得批准，且批准
必须绑定具体内容和 Evidence。唯一输入为
`docs/plans/infra-ig1-decisions.json`；Markdown 登记/TODO 不是批准事实。

行为契约：

1. `--registry-only` 只验证 registry 本身：schema/gate、D-01..D-05/D-06a/D-06b/D-07..D-19
   精确集合、九份 required proposals、subject 相对路径和 SHA-256、已定义角色、decision dependency DAG、
   已存在的 approval/Evidence 记录；结构有效时返回 0，即使状态仍 AWAITING/DRAFT；
2. 默认模式只执行 **IG-1 核心设计退出集**：D-01..D-05、D-06a、D-10..D-16、D-19 与九份
   required proposals；D-06b/D-07..D-09/D-17/D-18 属后续 lane 决策，不得反向造成 IG-1→Spike
   死锁，但其记录仍由 registry-only 精确校验；
3. bot/AI、proposal author 自批、重复 handle 冒充多角色、role binding 不匹配、非法 GitHub Review URL、
   非 40-hex reviewed commit、非严格 UTC 时间、scope/reason/ticket/有效期/独立性无效均是 finding；
4. verifier 固化每个 decision/proposal 的 required roles 与 decision dependency policy，registry 删除角色、
   作者或 D-06b→D-06a 边必须失败；不信任 registry 自报策略；
5. approval 必须重复绑定当前 `subject_revision` 与 `subject_sha256`；subject 任一字节变化或 revision
   递增后，保留旧 approval 必须失败；
6. Evidence 引用是 `{path, sha256}`，须位于仓库、拒绝 symlink/`..`/绝对路径并匹配内容 hash；D-06b
   仅接受 schema-valid、`PASS`、`INFRA-012`、clean commit、TDengine image/client 与 precision oracle
   均绑定的 `.evidence.json`；批准时必须聚合 REST roundtrip、native roundtrip、native sanitizer/fuzz
   三类独立记录，command 必须匹配对应 runtime test，`cargo check`/`true` 明确拒绝；
7. JSON 输出稳定包含 `registry_valid`、`gate_ready`、`trusted_review_readback`、decision/proposal count、
   `findings` 与 `blockers`；
8. 当前版本没有可信 GitHub API/签名回读，因此固定输出 `trusted_review_readback=false` 与
   `external-review-readback-not-implemented:INFRA-009` blocker；即使手填记录格式正确也不得令 gate
   变绿。未来只能通过 INFRA-009 的 review dismissal、last-push、actor type、ruleset API readback
   或等价验签实现解除，不得通过修改 registry 布尔值解除；
9. CLI 固定读取当前 Cargo workspace 的 canonical JSON，拒绝 `--path` 与 gate override；测试通过
   临时 workspace 的相同 canonical 相对路径注入负向 fixture，不在生产 CLI 暴露替代输入。

测试必须从 CLI seam 覆盖：canonical AWAITING registry 的 registry-only PASS/gate not ready、subject hash
篡改、revision 解绑、D-06b 集合/固定依赖缺失、角色 policy 漂移、handle 归一化、bot/AI、自批、
依赖环、畸形 Review/时间、任意文件冒充 Evidence，以及 alternate registry/gate override 的失败路径。

## 5. 用法

```bash
# 直接调用
cargo run -p xtask -- lint-deps

# crate 标准 inventory / ERROR gate
cargo run -p xtask -- crate-standard
cargo run -p xtask -- crate-standard --check

# 审批 registry 结构（当前应 PASS）/IG-1 gate（当前应非零）
cargo run -p xtask -- approval-check --registry-only
cargo run -p xtask -- approval-check

# 通过 alias（.cargo/config.toml）
cargo xtl lint-deps
cargo xtl approval-check --registry-only
```

退出码：0 = 通过，非 0 = 有违规。

## 6. 依赖策略

**证据（Constitution Article VII）：** `xtask` 作为工具链 crate，允许引入 `cargo_metadata` / `clap` / `anyhow` 等工具依赖，不受 L0–L2.5 业务依赖策略约束。

`xtask` 不发布到 crates.io（`publish = false`），版本号独立于宪法 / stdio / quant 的同步体系。

## 7. 演进约束

新增子命令（如 `gen-docs` / `release`）必须：

1. 与现有 `lint-deps` 正交，不引入跨子命令状态；
2. 在本 spec 追加对应章节；
3. 遵守 Article IV 简单优先——不写未被要求的通用 task runner 能力。

**未知：** 是否引入 `fix-deps`（自动修复违规依赖）——目前无证据要求，不得静默实现。
