# configx 实现规范

状态：当前 `0.1.0` 最小实现的 active 验收合同（非生产就绪）

- Package / lib：`xhyper-configx` / `configx`
- Implementation snapshot：`b0934baa`（2026-07-15）
- Document commit：`e0b98df4`
- Verified at：`e0b98df4`（相关实现路径未变化）
- Candidate：[SPEC-INFRA-CONFIGX-002](../../../draft/configx-complete-spec.md)（Draft，非权威，不覆盖本文）

## 0. 文档定位与裁定边界

本文细化 XLib spec v0.2 的 `configx` 合同，并以当前源码校准已实现能力。

- **证据（Evidence）**：XLib spec、已批准 ADR 或当前仓库代码直接规定/证明的事实。
- **推论（Inference）**：为使证据可验收而收窄出的最低要求，不新增架构批准。
- **未知（Unknown）**：上位材料和代码均未裁定；实现前必须评审。

冲突时按“XLib spec → 已批准 ADR → 本文 → 实现”裁定；代码存在不完整实现时，本文如实记录差距，
不得把现状反向解释为修改上位合同。

## 1. 定位、职责与非目标

- **证据**：`configx` 位于 `crates/configx`，属于 L1，目标职责是多源配置加载与热更新
  （XLib spec §§3、4.4）。
- **证据**：当前普通依赖仅为 `xhyper-kernel`，且明确不依赖 `observex`。
- **证据**：当前只实现线程安全的内存字符串 key-value 存储，尚未实现多源加载、解析或热更新。
- **证据**：workspace 当前没有 owner 外的 `ConfigStore` 生产引用。

非目标：secret 管理、业务规则校验、全局 service locator、日志/指标实现、远程控制面，以及在没有
真实消费者前构建通用配置平台。

## 2. 位置、依赖与版本

| 项目 | 当前事实 | 合同 |
| --- | --- | --- |
| 路径 | `crates/configx` | L1 Infra |
| 版本 | `0.1.0` | 独立维护；每次只允许 `x.y.z → x.y.(z+1)` |
| 普通依赖 | `xhyper-kernel` | 与 spec §4.4 一致；不得增加其他 L1 依赖 |
| feature | 无 | 新增前须由真实配置源需求评审 |

serde、文件格式、watcher、远端客户端或异步 runtime 尚未批准；workspace 已声明依赖不等于本 crate
获准使用。

## 3. 当前公开 API（代码事实）

| API | 当前语义 |
| --- | --- |
| `ConfigStore` | `RwLock<HashMap<String, String>>` 的拥有型封装；内部字段私有 |
| `ConfigStore::new() -> Self` | 创建空存储 |
| `ConfigStore::get(&self, key) -> Option<String>` | 克隆返回值；缺失或读锁中毒均返回 `None` |
| `ConfigStore::set(&self, key, val) -> XResult<()>` | 插入或覆盖；写锁中毒返回 `XError::Invalid` |
| `Default for ConfigStore` | 等价于 `new()` |

当前没有 builder、类型化解析、批量更新、订阅、快照、删除或枚举 API。公开面扩展须先以真实消费者
场景裁定，不能把未来职责直接推导成已批准签名。

## 4. 当前行为与不变量

1. **证据——空状态**：新建或默认存储对任意 key 返回 `None`。
2. **证据——覆盖**：同一 key 再次 `set` 后只读取到新值；多个 key 相互独立。
3. **证据——拥有返回值**：`get` 克隆字符串，调用方不持有锁或内部引用。
4. **证据——锁失败不对称**：读锁中毒被折叠成 `None`，写锁中毒返回
   `XError::Invalid("config lock poisoned")`。
5. **推论——依赖边界**：不通过直接依赖 `observex` 观测更新；需要跨层观测时须先定义合法合同。
6. **未知——上位目标**：多源优先级、解析/校验、原子快照、更新通知和热重载尚未实现。

`get` 无法区分“key 缺失”和“锁中毒”是当前兼容性事实，不得在文档或调用方中声称所有 `None` 都是
正常缺失。若要改变该语义，须评审返回类型与迁移方式。

## 5. 错误、并发、生命周期与信任边界

- `ConfigStore` 通过标准库 `RwLock` 支持共享并发访问；当前不承诺读写公平性、批量原子性或无饥饿。
- 当前没有后台任务、watcher 或显式 shutdown；热更新的 runtime、去抖、背压和关闭协议均为
  **未知**。
- key/value 是未分类字符串；当前实现没有 secret 类型、脱敏 `Debug` 或日志保护。调用方不得把敏感值
  交给未建立信任合同的诊断路径。
- 可恢复失败不得 panic；未来解析/校验失败不得用半更新状态替换上一份有效配置。

## 6. 测试合同

当前内联测试覆盖：空存储、set/get、覆盖、Default 和多 key 隔离。当前版本至少运行：

```text
cargo test -p xhyper-configx
cargo check -p configx --all-targets
cargo clippy -p configx --all-targets -- -D warnings
cargo fmt -- --check
cargo xtl lint-deps
```

目标职责落地前还须补：锁中毒语义、并发读写、源优先级、解析/校验失败保留旧快照、更新通知、secret
脱敏、watch 关闭不泄漏任务，以及 lint-deps 证明没有 L1 横向依赖。

## 7. 验收标准与开放决策

- [ ] 当前 API、错误字符串、依赖和测试与 §2–§6 一致。
- [ ] 文档不把内存 key-value 存储描述成已完成的多源热更新系统。
- [ ] 新增配置源、格式、优先级、类型化 API、通知或 runtime 前完成评审。
- [ ] 每次版本更新仅执行 `x.y.z → x.y.(z+1)`，兼容性治理独立执行。

仍需裁定：配置源/格式/优先级；类型化 API；锁中毒是否继续折叠为缺失；热更新通知与 runtime；
secret 来源；未知键策略；schema 与迁移。

## 8. 可追溯性

| 合同 | 来源 |
| --- | --- |
| L1 职责、依赖、不依赖 observex | XLib spec §4.4 |
| L1 互依禁止 | XLib spec R3 |
| 当前 API/锁语义/测试 | `crates/configx/src/lib.rs` |
| 当前依赖与版本 | `crates/configx/Cargo.toml` |
| 版本更新规则 | Constitution §7.3；XLib spec §5 |
