# configx 实现规范

状态：当前 `0.1.2` 进程内实现的 active 验收合同（非 Production Ready）

- Package / lib：`configx` / `configx`
- Implementation baseline：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`
- Round：03 reviewer 阻断修复；root 串行覆盖率 `1166 / 1166`（100.0000%）
- 双镜像：本文必须与 `xhyper-configx-complete-spec.md` 逐字节一致

## 0. 文档定位与裁定边界

本文是本仓 `crates/configx` 的 active 合同。状态必须由源码、测试和验证命令共同证明；文件名中的
`complete` 只表示合同副本完整，不表示 package stable、Production Ready 或远端配置平台已完成。

冲突时按“组织 Rust 规范 → 仓库宪章 / AGENTS.md → 本文 → 实现”裁定。实现不得用未来目标反向扩大
当前声明面。

## 1. 定位、职责与非目标

`configx` 是 L1、进程内的字符串配置组件，当前职责包括：

- 线程安全的 `ConfigStore`；
- `MemorySource` / `EnvSource` / `FileSource`（`KEY=VALUE`）三类调用时加载源；
- 后注册源覆盖先注册源的 `LayeredConfig`；
- 调用方显式触发的原子 reload；
- 进程内 `ConfigWatch` / `ConfigSubscription` 通知；
- `secret:` 键约定与诊断脱敏。

非目标：

- 自动文件 watcher、后台轮询、远端推送；
- 远端配置中心、分布式配置中心、多机一致性、动态服务发现；
- JSON / TOML / YAML 类型化 schema 产品；
- 远端 secret manager、静态加密或访问控制；
- 隐式全局 service locator。

`EnvSource` / `FileSource` 只有在调用方执行 `load_merged`、`reload_into` 或 `ConfigWatch::reload` 时才读取。
本 crate 不启动线程或异步 runtime。

## 2. 位置、依赖与版本

| 项目 | 合同 |
| --- | --- |
| 路径 | `crates/configx`，L1 Infra |
| 版本 | `0.1.2`；root 发布阶段已完成本次 PATCH bump |
| 普通依赖 | 仅 `kernel`；不得增加其他 L1 |
| feature | `default = []`，无额外 feature |
| 发布 | `publish = false` |

## 3. 公开 API 与兼容语义

### 3.1 存储与读取

| API | 语义 |
| --- | --- |
| `ConfigStore::new / Default` | 创建空 store |
| `get(key) -> Option<String>` | 兼容路径；缺失或读锁中毒都返回 `None` |
| `try_get(key) -> XResult<Option<String>>` | Result 路径；`None` 只表示缺失，毒锁返回错误 |
| `set / remove / clear` | 单次写操作；写锁中毒返回 `XError::Invalid` |
| `extend_pairs` | 锁外收集完整批次，以一次写锁提交 |
| `try_snapshot` / `ConfigSnapshot::try_capture` | 完整快照；毒锁返回错误 |
| `ConfigSnapshot::capture` | 兼容路径；毒锁折叠为空快照 |
| `try_get_secret` / `try_subset_snapshot` | Result 辅助路径；毒锁显式返回错误 |
| `get_secret` / `subset_snapshot` | 兼容辅助路径；毒锁分别折叠为缺失 / 空快照 |

`require_keys`、`require_nonempty` 和 `merge_into` 必须使用 Result 快照路径，禁止把毒锁误判成
正常缺失或空 overlay。

### 3.2 多源与 reload

- `LayeredConfig::load_merged` 完整加载每个源并校验所有 key；后源覆盖前源。
- `apply_to` 原子覆盖/新增批次，但保留 store 中未被覆盖的键。
- `reload_into` 在完整加载与键校验成功后，以一次写锁替换全部 map。
- 源加载、键校验或写锁失败时，旧 store 保持不变。
- 完整快照读取者只观察到旧 map 或新 map；跨多次独立 `get` 的调用方若需要一致视图，应改用快照。

这里的“校验”仅指 `validate_key`（非空、无控制字符、最大 512 字节），不是类型化 value schema。

### 3.3 watch

- `notify` 仅在 watch 未关闭且 generation 可用 `checked_add(1)` 递增时通知。
- generation 为 `u64::MAX` 时显式返回错误，不饱和、不重复通知。
- `notify / reload / close` 由独立 mutation mutex 串行化；state mutex 只用于短暂检查 / 提交，
  不得在等待 store 写锁时持有。
- `ConfigWatch::reload` 由调用方手动执行：完整 load / key 校验在锁外完成；mutation mutex 内先短暂
  检查 state，释放 state 后原子替换 store，再发布 generation。store 替换是配置线性化点，generation
  发布在 mutation mutex 释放前完成。
- `wait_outcome / wait_timeout_outcome` 显式返回 `Changed / TimedOut / Closed`。
- 兼容 `wait / wait_timeout` 保留 `Option`，其中 timeout 与 closed 都折叠为 `None`。
- `wait_timeout_outcome` 以整次调用的总时限为 deadline；使用 `try_lock` 观察 state，mutex 竞争
  不得产生无界阻塞，实际伪通知也不得延长 deadline；接受 `Changed` 前必须再次裁定 deadline，
  deadline 到达后即使 generation 已增长也返回 `TimedOut`。
- state 可立即观察且 watch 已关闭时，`Closed` 优先于同时到期的 deadline；锁竞争仍不得为确认
  closed 而越过 deadline 阻塞。
- `close` 唤醒订阅者；本 crate 没有后台 watcher 生命周期。

### 3.4 secret 诊断边界

- `SecretString` 的 `Debug` / `Display` 不输出明文。
- `ConfigSnapshot` 的自定义 `Debug` 对所有 `secret:` 前缀键的值输出 `***`。
- `MemorySource` 的自定义 `Debug` 对所有 `secret:` 前缀键的值输出 `***`。
- KEY=VALUE 解析错误只报告行号与错误类别，不回显可能含 secret 的原始行。
- 快照的 `get`、store 的 `get / try_get` 仍返回原始字符串；脱敏不等于加密或访问控制。
- 不带 `secret:` 前缀的敏感值无法自动识别，调用方必须正确分类。

## 4. 错误、并发与原子性不变量

1. 可恢复失败不得 panic。
2. 所有用户可见 `XError` context 使用简体中文；关键错误测试精确断言 kind 与完整 context。
3. 兼容折叠 API 与 Result API 的差异必须保留在 rustdoc 和 README。
4. `extend_pairs`、`apply_to`、`merge_into` 的写入不暴露部分批次。
5. `reload_into` 不得执行可观察的 `clear + N 次 set`。
6. `require_keys` / `require_nonempty` 基于单个完整快照校验。
7. `merge_into` 读取 overlay 失败必须返回错误，base 不变。
8. reload 等待 store 时不得持有 watch state；并发 notify 必须由 mutation mutex 排在 reload 前后。
9. timed wait 的 mutex 竞争和 generation 接受都必须受 deadline 限界。
10. `RwLock` / `Mutex` 不承诺公平性、无饥饿或跨多个独立 API 调用的事务性。

## 5. 测试合同

至少覆盖：

- 兼容 `get/capture` 与 Result `try_get/try_snapshot` 的毒锁差异；
- 生产校验与 merge 的显式失败；
- 阻塞迭代器证明批量写入提交前不可见；
- reload 测试须由 per-watch phase hook + Barrier 精确证明 state guard 已释放、mutation 仍持有且即将
  等待 store；不得依赖轮询、sleep 或调度概率；
- 加载失败与 key 校验失败保留旧快照；
- `ConfigSnapshot::Debug` 与 `MemorySource::Debug` 对 `secret:` 值脱敏；
- parse 错误不得回显原始行；
- generation 溢出不修改 generation，watch reload 溢出不替换 store；
- reload 等待 store 时 state 可获取，且并发 notify 排到下一 generation；
- state mutex 被长期持有时 timed wait 仍按 deadline 返回；
- generation 在 deadline 到达后出现或在接受前跨过 deadline 时必须返回 `TimedOut`；
- 已关闭 watch 的零时限等待必须返回 `Closed`；
- 就绪握手与通知计数证明实际伪通知不会延长总 deadline；
- 显式 wait outcome 与兼容 Option 路径；
- 新 Result / wait API rustdoc 包含 `# Errors`；
- 关键 `XError` 测试精确断言 `ErrorKind::Invalid` 与中文 context；
- 关闭、正常 notify、源优先级与公开消费者路径。

验证命令：

```bash
cargo fmt -p configx -- --check
cargo test -p configx --all-targets
cargo clippy -p configx --all-targets -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p configx --filter crates/configx/src
cmp .agents/ssot/configx/spec/spec.md \
    .agents/ssot/configx/spec/xhyper-configx-complete-spec.md
git diff --check 3cd29a942710c0fb42f3f6bc05e3c31570acad47 -- \
    crates/configx .agents/ssot/configx docs/ssot/configx-ssot-alignment.md
```

## 6. 验收标准

- Result 读取/快照/secret/subset 路径须区分缺失与 poison，兼容 API 语义不变。
- 所有批量写入与 reload 须在单写锁内提交，不暴露部分 map；失败保留上一份完整有效配置。
- 快照与 MemorySource Debug 不得泄露 `secret:` 值，parse 错误不得回显原始行。
- watch mutation/state 锁边界须明确，generation 溢出显式失败，timed wait 受总 deadline 限界。
- 用户可见错误须中文化，新 Result / wait API 须记录 `# Errors`。
- 显式 outcome 区分 Changed / TimedOut / Closed，兼容 Option API 保留。
- active / complete spec 须 `cmp` 一致，文档只声明进程内手动 reload。

上述实现与本地机器证据已形成；治理修正后候选已重冻，本地独立 reviewer 已完成实现/证据审查，
独立 verifier 已完成技术/证据初验。本次纯状态 delta 不改变受审源码/测试。GitHub 固定提交 CI
artifact 与发布流程仍 pending，因此本规范不构成发布 PASS。

## 7. 开放决策

仍未批准：远端源、自动 watcher、异步 runtime、去抖/背压、类型化 schema、未知 key 产品策略、
secret 加密/托管、跨进程或多机一致性、package stable / Production Ready。
