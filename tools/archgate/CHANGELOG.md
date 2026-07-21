# Changelog — archgate

遵循 [Keep a Changelog](https://keepachangelog.com/)，版本号见 `Cargo.toml`。

## [Unreleased]

### 变更

- **KERNEL-API-001/002 绑定源码生成**：先以固定版本 `cargo-public-api` 从当前
  `xhyper-kernel` 源码生成规范化 public API，再与 candidate snapshot / 冻结 baseline
  比较；工具缺失、版本不符、生成失败、stale snapshot、未授权 addition、baseline
  removal/signature change 一律 fail closed（不得用两份人工快照互比冒充绿）。
- **registry TOML 反序列化**：新增 `registry` 模块，以 `toml`+`serde` 解析
  `.architecture/workspace.toml` / `policies/dependency.toml`（`schema_version` 校验、
  `[defaults]` 合并、字段顺序无关、多行 `may_depend_on`）；行扫描解析器退役。
- 配套门禁：`publish_drift`、`status_edges`；JSON 增加 `schema_version`。

### 新增

- **KERNEL-API-002**：相对 `kernel-public-api.baseline.txt` 的新增公开行须在 `kernel-api-rfc.toml` 登记 Approved RFC。
- 负向单测：stale snapshot 描述、removal、unregistered addition、missing cargo-public-api、
  API-002 不回退 committed snapshot、baseline 指纹篡改。


### 新增

- SPEC-KERNEL-002 §12.2 命名规则模块 `kernel_rules`：KERNEL-DEP/FEATURE/API/TIME/ERR/SERDE/ASYNC/UNSAFE/LIFECYCLE；
  JSON 输出 `kernel_rules` + `kernel_internal_count`；失败阻断 archgate。

### 变更

- R3.1：`bootstrap →` 其他 L1 依赖与 `lint-deps` 对齐为唯一组装豁免；`dependency.toml` 不再放宽整层 `infra→infra`。
- 文档：README 补充「非职责」与「限制与安全」（crate-standard wave2）。


### 变更

- 文档：`main.rs` 补充 crate 级 `//!` 说明（crate-standard `crate-root-docs`）。

### 新增

- 补充工具职责、规则维护约束与设计入口文档。
- R7 kernel 门禁：`[dependencies]` 仅允许 `thiserror`；源码 use/pub 行禁止
  `anyhow`/`serde`/`tokio`/`chrono`/`tracing`；`crates/kernel/` 下 `public_api_leaks`
  计为失败；JSON 输出 `kernel_external_deps` / `kernel_forbidden_tokens` /
  `kernel_public_api_leaks`。
