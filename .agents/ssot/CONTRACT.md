# CONTRACT.md — 域合同数据模型

> 本文件是 **域合同（Domain Contract）的数据模型 SSOT**。  
> 定义合同是什么、元数据字段、命名规则、分层语义、兼容策略与当前合同目录。  
>
> - **验证规则**以 [`CONTRACT_SPEC.md`](./CONTRACT_SPEC.md) 为准（L1–L4 合规）。  
> - **域树与落地判定**以 [`SSOT.md`](./SSOT.md) 为准（R6 / R7）。  
> - **各域行为细节**以该域 `spec/spec.md` 为准（active current-state）。  
>
> 验证脚本：`scripts/quality-gates/check-contract-compliance.mjs`  
> CI：`.github/workflows/contract-compliance.yml`

---

## 1. 定义

**域合同（Domain Contract）** 是一份与 workspace package 绑定的、可机读对齐的权威声明面：

| 维度 | 含义 |
|------|------|
| **身份** | 稳定 `Spec ID` + 对应 package / 物理路径 |
| **边界** | 职责、非目标、依赖白名单、layer |
| **表面** | 公开 trait / 类型 / 方法 / 错误面（可被 L1 扫描） |
| **行为** | 前置/后置条件、不变量、状态转换（可被 L2 测） |
| **非功能** | 并发、资源、unsafe、供应链（可被 L3 查） |
| **生产证据** | 受控 live / 运行时长 / 错误分类（可被 L4 证） |

合同**不是**：

- 实现代码副本（禁止在 `.agents/ssot/` 写 `src/` / `Cargo.toml` / `*.rs`）
- STATUS 完成度分数（`STATUS.md` 是结构标尺，不是合同）
- 历史 draft / complete 战役叙事（仅当 `spec/spec.md` 声明为 active 时生效）
- package stable / Production Ready / crates.io 发布证明（须独立签核）

```text
active contract  =  .agents/ssot/<domain>/spec/spec.md  (+ 双镜像若存在)
implementation   =  crates/... 或 tools/...
verification     =  CONTRACT_SPEC.md + check-contract-compliance.mjs
```

---

## 2. 文档职责划分

| 文件 | 职责 | 可编辑内容 |
|------|------|------------|
| **本文件 `CONTRACT.md`** | 合同**数据模型**、字段 schema、命名、目录、兼容策略 | 模型与目录；不含 L1–L4 细则 |
| [`CONTRACT_SPEC.md`](./CONTRACT_SPEC.md) | 合同**验证规则**（L1–L4）、门禁、例外 | 规则 ID 与命令 |
| [`SSOT.md`](./SSOT.md) | SSOT 树规则、落地 vs 规格（R6/R7） | 树结构与清单 |
| 域 `spec/spec.md` | 该 package 的 **active 行为合同** | 公开面与验收边界 |
| `docs/ssot/*-ssot-alignment.md` | 本仓落地矩阵 | 实现/测试对照，不扩写合同语义 |
| `market_data/CONTRACT.md` 等子树 | **子平面**横向治理（仅该子树主题） | 不得覆盖本文件全局模型 |

冲突裁决（从高到低）：

1. 组织 Rust 规范 / 仓库宪章 / `AGENTS.md`
2. 本文件（跨域合同模型）
3. `CONTRACT_SPEC.md`（验证）
4. 域 `spec/spec.md`（active 行为）
5. `design/` / `goal/` / `review/`（解释与状态）
6. 源码与测试（**当前实现证据**；不得静默覆盖未决目标合同）

源码与 active spec 不一致时：标 `pending` 或同 PR 消除冲突；**禁止**把「编译通过」当作合同已满足。

---

## 3. 合同记录（Record）Schema

每份 active 合同由 **一条逻辑记录** 描述，权威正文在域 `spec/spec.md`。  
合规脚本通过 front-matter / 表头字段发现记录（见 `discoverContracts()`）。

### 3.1 必填字段

| 字段 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `spec_id` | string | 稳定合同 ID，见 §4 | `SPEC-KERNEL-002` |
| `domain` | string | SSOT 叶域名 | `kernel`、`configx` |
| `package` | string | Cargo package 名 | `kernel`、`transportx` |
| `lib` | string | `lib` crate 名（可与 package 同） | `kernel`、`transportx` |
| `path` | path | 实现物理路径 | `crates/kernel`、`crates/infra/configx` |
| `spec_path` | path | active spec 路径 | `.agents/ssot/kernel/spec/spec.md` |
| `layer` | enum | 见 §5 | `L0` / `L1` / `L2` / `Contract` / `T0` |
| `status` | enum | 见 §7 | `Active` / `Candidate` / … |
| `current_version` | semver | 与 `Cargo.toml` version **应对齐**（L1-SIG-007） | `0.3.1` |

### 3.2 推荐字段

| 字段 | 说明 |
|------|------|
| `owner` | 维护责任（如 `platform`） |
| `publish` | 是否计划 crates.io（本仓多数为 `false`） |
| `authority` | 明确「本文件是 active current-state」 |
| `complete_mirror` | 双镜像路径（须与 `spec.md` `cmp` 一致） |
| `compatibility` | `additive_only` / `internal` / `experimental` |
| `baseline` | 实现基线 commit（审计起点，非冻结证据） |
| `verified_at` | 最近人工/机控核验日期 |
| `supersedes` | 被替代的旧 Spec ID |
| `non_goals` | 明确非目标（防夸大） |

### 3.3 推荐 front-matter 形态

新写或大修 active spec 时，优先使用统一块（现有表格式仍可被脚本宽松解析）：

```text
Spec ID:         SPEC-<DOMAIN>-<NNN>
Status:          Active
Owner:           platform
Physical Path:   crates/<path>
Package / lib:   <package> / <lib>
Current Version: x.y.z
Publish:         false
Layer:           L0 | L1 | L2 | Contract | T0
Compatibility:   additive_only | internal | experimental
```

或等价 Markdown 表：

| 字段 | 值 |
|------|-----|
| Spec ID | `SPEC-…` |
| Package / lib | `foo` / `foo` |
| Path | `crates/…` |
| Layer | L1 |
| Current Version | `0.1.2` |

### 3.4 脚本解析约定

`check-contract-compliance.mjs` 当前发现路径见 §8。解析启发式：

| 键 | 正则 / 来源（摘要） |
|----|---------------------|
| `spec_id` | `Spec(?:\s+ID)?\s*[：:]\s*(SPEC-[\w-]+)`；缺省时 `SPEC-<DOMAIN>-???` |
| `package` / `lib` | `Package / lib …` 行 |
| `path` | `Path` / `Physical Path` 行；缺省由 SSOT 相对路径推导 |
| `version` | `Version` / `Current Version` 与 `Cargo.toml` 比对（L1-SIG-007） |

新增受管合同域时：**必须**同步更新本文件 §8 与脚本 `srcDirs`。

---

## 4. Spec ID 命名

```text
SPEC-<DOMAIN>-<NNN>
```

| 段 | 规则 | 示例 |
|----|------|------|
| `SPEC` | 固定前缀 | — |
| `DOMAIN` | 大写，可含 `_` 或多段 | `KERNEL`、`TESTKIT`、`INFRA_TRANSPORTX`、`SCHEDULEX` |
| `NNN` | 三位或更长序号；**递增不复用** | `002`、`003` |

规则：

1. 同一 package 的 active 合同同时只有 **一个** Spec ID。  
2. 大修可升序号并写 `Supersedes:`；旧文移入 `*.superseded.md` 或历史目录。  
3. 禁止用文件名中的 `complete` / `xhyper-*` 冒充 Spec ID。  
4. 工具域可用日期式 ID（如 `SPEC-2026-VERIFYCTL-001`），但须在目录表登记。  
5. `contract_id` 在例外文件（`gate/exemptions.yaml`）中等于 `spec_id`。

---

## 5. 分层（Layer）语义

| Layer | 含义 | 典型 package | 合同侧重 |
|-------|------|--------------|----------|
| **L0** | 语义信任根 | `kernel` | 全局唯一语义；禁止 unsafe；强 ErrorKind 面 |
| **T0** | 测试支持（非生产 graph） | `testkit`、`contract-testkit` | 仅 dev-dep；确定性 / Fake |
| **L1** | 进程内基础设施 | `configx`、`schedulex`、`bootstrap`、`evidence`、`observex`、`resiliencx`、`transportx` | 声明面内行为；非完整平台产品 |
| **L2** | 跨层纯类型 / wire 子集 | `canonical`、`decimalx` | 序列化与类型合同 |
| **Contract** | 跨层 trait 出口（R4） | `contracts` | **Additive Only**；无 adapter 实现 |
| **Adapter** | 存储 / 交易所适配 | `redisx`、`binancex`… | 客户端入口合同；交易/集群另标 NO-GO |
| **Tool** | CLI / 验证工具 | `goalctl`、`verifyctl` | CLI 与 exit 合同；非生产 verifier 须标明 |

**合规层级（L1–L4）** 与 **架构 layer（L0/L1/…）** 是不同轴：

- 架构 layer：包在依赖图中的位置  
- 合规层级：`CONTRACT_SPEC` 的验证深度  

勿混写「L1 Internal Ready」与「L1-SIG-001」。

---

## 6. 兼容与演进策略

### 6.1 `additive_only`（强制于 `contracts`）

- 不得在未声明的情况下 **删除** 公开项或 **收窄** 签名  
- trait 新增方法须有 default 或同步迁移全部实现方  
- 公开 enum 默认 `#[non_exhaustive]`（L1-SIG-006）  
- API baseline 变更须走 PR + 对齐文，不得静默

### 6.2 `internal`

- 默认 `publish = false`  
- 可在 major/约定 bump 下做 breaking，但须写进 active spec + CHANGELOG  
- 仍受 L1 签名/版本对齐约束

### 6.3 `experimental`

- 可隔离在 feature 或 `experimental` 模块  
- 不得写入 Production Ready 签字面  
- 合规可运行，但 L4 默认不适用

### 6.4 版本对齐

- `spec` 声明的 `Current Version` 与目标 `Cargo.toml` `version` 一致（L1-SIG-007）  
- crate 独立版本：交付默认 PATCH +1（见 `.agents/rules/VERSIONING.md`）  
- 规格变更若改公开行为：同一 PR 更新 spec + 测试 + 版本（或明确 `pending`）

---

## 7. 状态与证据词汇

### 7.1 合同 `status`（文档态）

| 值 | 含义 |
|----|------|
| `Draft` | 非权威；不得驱动合规阻断期望 |
| `Candidate` | 候选 active；待 CI / 审查 / 合并 |
| `Active` | 当前权威合同 |
| `Approved` | 规格/维护批准（≠ package stable） |
| `Superseded` | 已被新 Spec ID 替代 |
| `OOS` | 本仓明确不落地 |

### 7.2 落地 `maturity`（实现对齐文）

| 值 | 含义 |
|----|------|
| `specified` | 仅有规格 |
| `skeleton` | 有类型骨架，行为未闭合 |
| `pending` | 规格与源码已知分歧 |
| `implemented` | 声明面有源码 + 单测证据 |
| `verified` | 有可重复命令 / fixture / mock 证据 |
| `blocked` / `NO-GO` | 明确禁止宣称（如交易执行） |

### 7.3 禁止假阳性

以下 **不得** 单独推出「合同已满足 / 可生产」：

- 文件名含 `complete`、目录 11 层齐全、STATUS 100%  
- `cargo check` / 单测绿（必要但不充分）  
- `#[ignore]` live 测试可编译  
- feature flag 为 true  
- 历史 review COMPLETE / Phase Approved  

完整生产宣称须走签核模板与 Maintainer 人工签核（Agent 不得代签）。

---

## 8. 受管合同目录（Compliance Catalog）

下列记录由 `check-contract-compliance.mjs` **当前**发现与校验。  
`spec_id` 以各 `spec.md` 正文为准；未写出时脚本回退为 `SPEC-<DOMAIN>-???`。

| domain | package | path | SSOT spec | layer | 兼容 | 备注 |
|--------|---------|------|-----------|-------|------|------|
| `kernel` | `kernel` | `crates/kernel` | `.agents/ssot/kernel/spec/spec.md` | L0 | internal | `SPEC-KERNEL-002` |
| `testkit` | `testkit` | `crates/testkit` | `.agents/ssot/testkit/spec/spec.md` | T0 | internal | `SPEC-TESTKIT-002`；仅 dev-dep |
| `contracts` | `contracts` | `crates/contracts` | `.agents/ssot/contracts/spec/spec.md` | Contract | **additive_only** | R4 trait 出口；Fake 在 `contract-testkit` |
| `configx` | `configx` | `crates/infra/configx` | `.agents/ssot/infra/configx/spec/spec.md` | L1 | internal | 本地多源；非远端配置中心 |
| `schedulex` | `schedulex` | `crates/infra/schedulex` | `.agents/ssot/infra/schedulex/spec/spec.md` | L1 | internal | 宿主 `tick`；非分布式调度 |
| `bootstrap` | `bootstrap` | `crates/infra/bootstrap` | `.agents/ssot/infra/bootstrap/spec/spec.md` | L1 | internal | 唯一组合根 |
| `evidence` | `evidence` | `crates/infra/evidence` | `.agents/ssot/infra/evidence/spec/spec.md` | L1 | internal | 追加面；非合规审计平台 |
| `observex` | `observex` | `crates/infra/observex` | `.agents/ssot/infra/observex/spec/spec.md` | L1 | internal | 非 OTEL/OTLP 产品 |
| `resiliencx` | `resiliencx` | `crates/infra/resiliencx` | `.agents/ssot/infra/resiliencx/spec/spec.md` | L1 | internal | 进程内弹性 |
| `transport` | `transportx` | `crates/infra/transport` | `.agents/ssot/infra/transport/spec/spec.md` | L1 | internal | HTTP/WS 传输边界 |

### 8.1 有规格、暂未纳入默认合规扫描

| domain | 说明 |
|--------|------|
| `infra/gate`、`infra/testkitx` | 规格镜像；本仓未宣称对应 member 落地 |
| `adapters/**` | 九 adapter 有域规格；合规 catalog 扩展须单独立项（交易 NO-GO 不变） |
| `types/**` | decimal / canonical 规格在 types 树；可后续加入 `srcDirs` |
| `tools/**` | goalctl / verifyctl / xtask；CLI 合同独立（如 `CLI-CONTRACT.md`） |
| `market_data/**`、`macro_data/**`、`core/**` | 子平面；横向规则见各树 `CONTRACT.md` / `SSOT.md`，不替代本文件 |

### 8.2 登记新合同的最小清单

1. 域树 `spec/spec.md`（+ 双镜像若域要求）  
2. 本表 §8 增加一行  
3. `check-contract-compliance.mjs` 的 `srcDirs` 增加路径  
4. 若影响落地叙事：更新 `docs/ssot/*-ssot-alignment.md`  
5. PR 中跑：`node scripts/quality-gates/check-contract-compliance.mjs --level L1`

---

## 9. 合同正文应覆盖的内容块

active `spec.md` 推荐结构（可裁剪，但公开面与边界不可空）：

1. **身份头**（§3 字段）  
2. **职责与非目标**  
3. **依赖与编译合同**（生产依赖、feature、unsafe）  
4. **公开 API / trait 面**（可用 Rust 签名块，供 L1-SIG-001）  
5. **错误面**（`ErrorKind` 子集或映射表，供 L2-BEH-002）  
6. **行为不变量** 与状态机（若有）  
7. **并发 / 取消 / 超时**（若有 I/O）  
8. **测试与证据要求**（指向 `test/`、`matrix/`、命令）  
9. **NO-GO / OPEN / DEFER** 显式列表  
10. **变更与版本**  

Code 列永远指向 `crates/` 或 `tools/`，不在 SSOT 内复制实现。

---

## 10. 与合规层级的映射

| 合规层 | 主要消费本模型的字段 | 权威细则 |
|--------|----------------------|----------|
| L1 签名 | 公开项、`#[non_exhaustive]`、version、依赖边界 | `CONTRACT_SPEC` §2 |
| L2 行为 | ErrorKind、前置条件、不变量、conformance | `CONTRACT_SPEC` §3 |
| L3 非功能 | 性能边界、unsafe、CVE | `CONTRACT_SPEC` §4 |
| L4 生产 | live 证据、运行时长、错误采样 | `CONTRACT_SPEC` §5 |

例外（`gate/exemptions.yaml`）必须引用 `contract_id = spec_id`，并含 ADR、到期日、批准人（见 `CONTRACT_SPEC` §7）。

---

## 11. 变更协议

| 变更类型 | 要求 |
|----------|------|
| 改数据模型（本文件） | worktree + PR；同步 `CONTRACT_SPEC` / 脚本若破坏兼容 |
| 改验证规则 | 改 `CONTRACT_SPEC.md` + 脚本/CI；本文件只调交叉引用 |
| 改某域行为合同 | 改该域 `spec/spec.md`（+ 双镜像）；对齐 version 与测试 |
| 新增受管域 | 走 §8.2 |
| 废除合同 | `Superseded` + R5 重定向；目录表标注 |

提交前建议：

```bash
test -f .agents/ssot/CONTRACT.md
test -f .agents/ssot/CONTRACT_SPEC.md
node scripts/quality-gates/check-contract-compliance.mjs --level L1 --fail-level L1
```

---

## 12. 相关路径速查

| 用途 | 路径 |
|------|------|
| 数据模型（本文件） | `.agents/ssot/CONTRACT.md` |
| 验证规则 | `.agents/ssot/CONTRACT_SPEC.md` |
| SSOT 树规则 | `.agents/ssot/SSOT.md` |
| 域操作说明 | `.agents/ssot/AGENTS.md` |
| 合规脚本 | `scripts/quality-gates/check-contract-compliance.mjs` |
| Stop Hook | `.claude/hooks/contract-compliance-guard.mjs` |
| CI | `.github/workflows/contract-compliance.yml` |
| 落地总览 | `docs/ssot/workspace-ssot-alignment.md` |
| market_data 子平面合同 | `.agents/ssot/market_data/CONTRACT.md` |

---

## 13. 版本

| 版本 | 日期 | 修订 |
|------|------|------|
| v1.0.0 | 2026-07-24 | 初始：补齐 CONTRACT_SPEC 引用的数据模型 SSOT；登记当前合规 catalog 十域 |
