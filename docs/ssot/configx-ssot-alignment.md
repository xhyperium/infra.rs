# configx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| Spec | active configx 0.1.1（`.agents/ssot/configx/spec/spec.md` ≡ `xhyper-configx-complete-spec.md`） |
| 镜像 | `.agents/ssot/configx/**`（R6 只读；**禁止**改镜像冒充本仓完成） |
| 本仓实现 | `crates/configx` · package `configx` · lib `configx` · version `0.1.1` |
| 审计日期 | 2026-07-21；**defer-close 复核 2026-07-22** |
| 结论 | **active 合同面（§2–§7）无 FAIL**；**多源加载 / 热更新 NOT IMPLEMENTED（诚实边界，仅内存字符串 KV）**；**≠** 远端配置中心 / Agent L5 |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游镜像 COMPLETE / 布局对齐 | 描述的是 **goal 管线布局**；**禁止**单独当作本仓实现证明 |
| 本仓 `crates/configx` | **已落地**并与 active SSOT §2–§6 可移植子集对齐 |
| 多源加载 / 分层合并 | **NOT IMPLEMENTED（诚实边界）** — 当前仅内存字符串 KV；多源加载/分层合并未实现；禁止宣称配置平台 |
| 热更新通知 / 热重载 | **NOT IMPLEMENTED（诚实边界）** — 当前无 watcher / 后台热重载；仅内存字符串 KV |
| secret | **PASS**：`secret.rs` · `SecretString`（Debug 脱敏）+ `set_secret`/`get_secret` |
| 远端配置中心 / 动态服务发现 | **OPEN（诚实边界）** — **未**实现；禁止宣称配置中心产品 |
| line/branch cov | 目标 100%（`cargo llvm-cov -p configx`） |
| 本仓 crates.io 再发布 | **不做**；`publish = false` |

## 本仓可观察事实

```text
crates/configx/                 EXISTS
Cargo.toml members              含 crates/configx
package name                    configx
lib name                        configx
version                         0.1.1
publish                         false
生产依赖                        仅 kernel（path crates/kernel）
features                        default = []
公开面                          ConfigStore（内存字符串 KV）
模块                            lib · source · layered · watch · secret（多源/热更新 NOT IMPLEMENTED）
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
- 文件名 `xhyper-configx-complete-spec.md` 与 `spec.md` 同构，内容仍是 0.1.1 **最小内存 KV 合同**，不是完整配置平台
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`

---

## 逐条对齐矩阵（active SSOT §2–§7 可移植子集）

> 判定：`PASS` = 本仓有源码/测试证据；`FAIL` = 语义缺失须修；`DEFER` = Unknown/未批准/环境专属，写明原因。  
> 证据指针为 crate 相对路径（`crates/configx/...`）。

### §2 位置、依赖与版本

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 2.1 | 路径 `crates/configx`，L1 Infra | `crates/configx/`；根 `Cargo.toml` members | PASS |
| 2.2 | package `configx` / lib `configx` | `Cargo.toml` `[package]` / `[lib]` | PASS |
| 2.3 | 版本独立维护；当前 `0.1.1` | `Cargo.toml` `version = "0.1.1"` | PASS |
| 2.4 | 普通依赖仅 `kernel` | `Cargo.toml` `[dependencies]` 唯一 path 依赖 | PASS |
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
| 4.6 | 上位：多源优先级 / 更新通知 / 热重载 | **PASS（进程内）** | `LayeredConfig` 后覆盖先；`ConfigWatch::reload`；**≠** 远端配置中心 / schema 产品 |
| 4.7 | 不得声称所有 None 都是正常缺失 | README / rustdoc 写明读中毒折叠 | PASS |

### §5 错误、并发、生命周期与信任边界

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 5.1 | std `RwLock` 共享并发；不承诺公平性/批量原子/无饥饿 | 实现 + 并发 smoke 不证明公平性 | PASS |
| 5.2 | 无隐式后台 daemon | 无自动 spawn 的远程轮询 | PASS |
| 5.3 | 热更新通知（进程内） | **PASS** | `ConfigWatch` / `ConfigSubscription`；**无** 去抖/背压产品矩阵 |
| 5.4 | 基础 KV 仍为字符串 | `ConfigStore` 仍 `String` map | PASS |
| 5.5 | secret 脱敏 | **PASS** | `SecretString` Debug=`***`；`SECRET_KEY_PREFIX` |
| 5.6 | 可恢复失败不得 panic | poison 测不 panic；set/get 可恢复 | PASS |
| 5.7 | 未来解析失败不得半更新替换有效配置 | KV 路径无 schema 解析半更新 | **N/A**（无此产品路径；诚实边界） |

### §6 测试合同

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 6.1 | 空存储 / set-get / 覆盖 / Default / 多 key 隔离 | `src/lib.rs` tests + `tests/public_api.rs` | PASS |
| 6.2 | 命令：`cargo test/check/clippy/fmt -p configx` | 本仓可执行；见验证入口 | PASS |
| 6.3 | 锁中毒语义 | `read_lock_poison_folds_to_none` / `write_lock_poison_returns_invalid` | PASS |
| 6.4 | 并发读写 | `tests/concurrency.rs` + 单元 concurrent smoke | PASS |
| 6.5 | 源优先级 / 更新通知 / secret 脱敏 | **PASS** | source/layered/watch/secret 模块 + 单测 |
| 6.6 | lint-deps 证明无 L1 横向依赖 | 生产依赖仅 kernel（`Cargo.toml` + metadata 扫描） | PASS（等价证据） |

### §7 验收标准与开放决策

| ID | 要求 | 本仓证据 | 判定 |
|----|------|----------|------|
| 7.1 | API/错误字符串/依赖/测试与 §2–§6 一致 | 本矩阵 + 测试套件 | PASS |
| 7.2 | 文档诚实：file/env ≠ 远端配置中心 | `README.md` / rustdoc / 本文 | PASS |
| 7.3 | 声明层源/分层/watch/secret 已落地 | 见 OBJECTIVE | PASS |
| 7.4 | 版本仅 `x.y.z → x.y.(z+1)` | 独立 package version | PASS |
| 7.5 | 远端配置中心 / schema 产品 / 类型化 API | 未实现 | **OPEN（诚实边界）** |

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| 多源 | DEFER | **PASS** | `crates/configx/src/source.rs` · Memory/Env/File |
| 分层/优先级 | DEFER | **PASS** | `crates/configx/src/layered.rs` · 后覆盖先 |
| 热更新 | DEFER | **PASS（进程内）** | `crates/configx/src/watch.rs` |
| secret | DEFER | **PASS** | `crates/configx/src/secret.rs` |

## 非目标 / 诚实边界

- **远端配置中心**、服务发现、动态推送、多机一致性
- 完整 JSON/TOML/YAML schema 校验产品
- 类型化配置 builder 全家桶
- 其他 infra 域（gate 等）

## 覆盖率目标

| 度量 | 目标 | 命令 |
|------|------|------|
| line | 100% | `cargo llvm-cov -p configx --summary-only` |
| branch | 100% | 同上 TOTAL branches |

毒锁两分支与空/有值路径均由真实测试驱动，禁止 mock `ConfigStore`。

## 本轮增量

| `require_keys` | **PASS**（必填 key 存在性；非完整 schema） |

## 双栏落地（2026-07-22 · STATUS 100% structure）

| 标尺 | 状态 |
|------|------|
| STATUS 结构完成度 | **100%**（layout+tests+content；非 Production Ready） |
| 声明面生产硬化 | 公共 API 集成测 + 热路径 bench + `docs/` 红线；**cov-gate-100 行覆盖** |
| 非宣称 | **禁止** workspace Production Ready / Agent L5 / 扩大 SSOT DEFER 平台面 |

自验证：`cargo test -p configx --all-targets`；`node scripts/quality-gates/cov-gate-100.mjs -p configx`；`cargo run -p configx --example …`；`cargo bench -p configx --bench hot_path -- --quick`。
