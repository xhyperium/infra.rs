# configx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| Spec | active configx 0.1.0（`.agents/ssot/configx/spec/spec.md` ≡ `xhyper-configx-complete-spec.md`） |
| 镜像 | `.agents/ssot/configx/**`（R6 只读；**禁止**改镜像冒充本仓完成） |
| 本仓实现 | `crates/configx` · package `xhyper-configx` · lib `configx` · version `0.1.0` |
| 审计日期 | 2026-07-21 |
| 结论 | **active 合同面（§2–§7 可移植条款）无 FAIL 残留**；上位多源/热更新能力统一 **DEFER** |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游镜像 COMPLETE / 布局对齐 | 描述的是 **goal 管线布局**；**禁止**单独当作本仓实现证明 |
| 本仓 `crates/configx` | **已落地**并与 active SSOT §2–§6 可移植子集对齐 |
| 多源加载 / 热更新 / secret | **DEFER**（SSOT Unknown；未批准前不实现） |
| line/branch cov | 目标 100%（`cargo llvm-cov -p xhyper-configx`） |
| 本仓 crates.io 再发布 | **不做**；`publish = false` |

## 本仓可观察事实

```text
crates/configx/                 EXISTS
Cargo.toml members              含 crates/configx
package name                    xhyper-configx
lib name                        configx
version                         0.1.0
publish                         false
生产依赖                        仅 xhyper-kernel（path）
features                        default = []
公开面                          ConfigStore + new/get/set/Default
```

验证（本仓权威命令）：

```bash
cargo test -p configx --all-targets
cargo clippy -p configx --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p configx --example basic
cargo llvm-cov -p configx --summary-only
```

## 与镜像文档的关系

- `.agents/ssot/configx/**`：只读镜像；禁止本地改 Done/COMPLETE 叙事冒充同步
- 实现 SSOT 以 **源码 + 本仓测试输出** 为准
- 文件名 `xhyper-configx-complete-spec.md` 与 `spec.md` 同构，内容仍是 0.1.0 **最小内存 KV 合同**，不是完整配置平台
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`

---

## 逐条对齐矩阵（active SSOT §2–§7 可移植子集）

> 判定：`PASS` = 本仓有源码/测试证据；`FAIL` = 语义缺失须修；`DEFER` = Unknown/未批准/环境专属，写明原因。  
> 证据指针为 crate 相对路径（`crates/configx/...`）。

### §2 位置、依赖与版本

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 2.1 | 路径 `crates/configx`，L1 Infra | `crates/configx/`；根 `Cargo.toml` members | PASS |
| 2.2 | package `xhyper-configx` / lib `configx` | `Cargo.toml` `[package]` / `[lib]` | PASS |
| 2.3 | 版本独立维护；当前 `0.1.0` | `Cargo.toml` `version = "0.1.0"` | PASS |
| 2.4 | 普通依赖仅 `xhyper-kernel` | `Cargo.toml` `[dependencies]` 唯一 path 依赖 | PASS |
| 2.5 | 不得增加其他 L1 依赖 | 生产 deps 扫描无 observex/其他 L1 | PASS |
| 2.6 | feature 无；`default = []` | `Cargo.toml` `[features]` | PASS |
| 2.7 | 未批准 serde/watcher/async runtime | 生产 `Cargo.toml` 无上述依赖 | PASS |

### §3 当前公开 API

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 3.1 | `ConfigStore`：`RwLock<HashMap<String,String>>` 拥有型封装；字段私有 | `src/lib.rs` | PASS |
| 3.2 | `new() -> Self` 空存储 | `src/lib.rs` + `tests/public_api.rs` | PASS |
| 3.3 | `get(&self, key) -> Option<String>` 克隆；缺失/读中毒 → None | `src/lib.rs` + 单元/集成测 | PASS |
| 3.4 | `set(&self, key, val) -> XResult<()>` 插入或覆盖 | `src/lib.rs` + 测试 | PASS |
| 3.5 | 写锁中毒 → `XError::Invalid` 上下文含 `config lock poisoned` | `src/lib.rs` `tests` `write_lock_poison_returns_invalid` | PASS |
| 3.6 | `Default` 等价 `new()` | `src/lib.rs` + `default_equals_empty_new` | PASS |
| 3.7 | 无 builder / 类型化 / 批量 / 订阅 / 快照 / 删除 / 枚举 API | 公开面仅 re-export `ConfigStore`；`rg` 无上述符号 | PASS |

### §4 行为与不变量

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 4.1 | 空状态：任意 key → None | `empty_store_returns_none` / `public_api` | PASS |
| 4.2 | 同 key 覆盖后只读新值；多 key 隔离 | `set_overwrites_same_key` / `multi_key_isolation` | PASS |
| 4.3 | `get` 返回拥有字符串 | `set_then_get_returns_owned_clone` / `get_returns_owned_string_not_borrow` | PASS |
| 4.4 | 锁失败不对称：读→None，写→Invalid | poison 两测 | PASS |
| 4.5 | 不通过直接依赖 observex 观测 | `Cargo.toml` 无 observex | PASS |
| 4.6 | 上位：多源优先级 / 解析校验 / 原子快照 / 更新通知 / 热重载 | 未实现；本仓不宣称 | DEFER（SSOT Unknown） |
| 4.7 | 不得声称所有 None 都是正常缺失 | README / rustdoc 写明读中毒折叠 | PASS |

### §5 错误、并发、生命周期与信任边界

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 5.1 | std `RwLock` 共享并发；不承诺公平性/批量原子/无饥饿 | 实现 + 并发 smoke 不证明公平性 | PASS |
| 5.2 | 无后台任务 / watcher / 显式 shutdown | 源码无 spawn/watcher | PASS |
| 5.3 | 热更新 runtime / 去抖 / 背压 / 关闭协议 | 未实现 | DEFER（Unknown） |
| 5.4 | key/value 未分类字符串；无 secret 类型/脱敏 Debug | 仅 `String`；无自定义 Debug 脱敏 | PASS（诚实最小面） |
| 5.5 | secret 脱敏 / 信任合同诊断路径 | 未实现 | DEFER（Unknown） |
| 5.6 | 可恢复失败不得 panic | poison 测不 panic；set/get 可恢复 | PASS |
| 5.7 | 未来解析失败不得半更新替换有效配置 | 无解析路径 | DEFER（无此代码路径） |

### §6 测试合同

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 6.1 | 空存储 / set-get / 覆盖 / Default / 多 key 隔离 | `src/lib.rs` tests + `tests/public_api.rs` | PASS |
| 6.2 | 命令：`cargo test/check/clippy/fmt -p xhyper-configx` | 本仓可执行；见验证入口 | PASS |
| 6.3 | 锁中毒语义 | `read_lock_poison_folds_to_none` / `write_lock_poison_returns_invalid` | PASS |
| 6.4 | 并发读写 | `tests/concurrency.rs` + 单元 concurrent smoke | PASS |
| 6.5 | 源优先级 / 解析失败保留旧快照 / 更新通知 / secret 脱敏 / watch 关闭 | 未批准能力 | DEFER（Unknown；非 0.1.0 面） |
| 6.6 | lint-deps 证明无 L1 横向依赖 | 生产依赖仅 kernel（`Cargo.toml` + metadata 扫描） | PASS（等价证据） |

### §7 验收标准与开放决策

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 7.1 | API/错误字符串/依赖/测试与 §2–§6 一致 | 本矩阵 + 测试套件 | PASS |
| 7.2 | 文档不把内存 KV 描述成多源热更新系统 | `README.md` / rustdoc / 本文结论 | PASS |
| 7.3 | 新增源/格式/优先级/类型化/通知/runtime 前评审 | 未添加；开放决策见下 | PASS（未越界） |
| 7.4 | 版本仅 `x.y.z → x.y.(z+1)` | 首版 `0.1.0` | PASS |
| 7.5 | 开放决策：源/格式/优先级/类型化/中毒语义变更/热更新/secret/未知键/schema | 未裁定；实现前须评审 | DEFER |

## 非目标（明确不在本仓 0.1.0 完成面）

- 多源加载、文件/远端源、优先级合并
- schema 校验、热重载、订阅/通知
- 类型化配置 API、secret 管理
- 其他 infra 域（bootstrap/gate/observex/…）

## 覆盖率目标

| 度量 | 目标 | 命令 |
|------|------|------|
| line | 100% | `cargo llvm-cov -p configx --summary-only` |
| branch | 100% | 同上 TOTAL branches |

毒锁两分支与空/有值路径均由真实测试驱动，禁止 mock `ConfigStore`。
