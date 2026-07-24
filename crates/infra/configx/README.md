# configx

L1 进程内字符串配置组件：线程安全 KV、多源合并、调用方手动 reload、进程内通知与 secret 诊断脱敏。

| 项 | 值 |
| --- | --- |
| package / lib | `configx` / `configx` |
| path | `crates/infra/configx` |
| version | `0.1.2` |
| publish | `false` |
| 生产依赖 | 仅 `kernel` |
| feature | `default = []` |

> 诚实边界：File / Env 只在调用方显式加载时读取；本 crate 不启动自动 watcher、后台轮询或异步
> runtime，也不是远端配置中心、类型化 schema 产品或 secret manager。

规范：[`../../.agents/ssot/configx/spec/spec.md`](../../.agents/ssot/configx/spec/spec.md)
对齐说明：[`../../docs/ssot/configx-ssot-alignment.md`](../../docs/ssot/configx-ssot-alignment.md)

## 最小用法

```rust
use configx::{ConfigStore, LayeredConfig, MemorySource};
use std::sync::Arc;

let store = ConfigStore::new();
let layered = LayeredConfig::new()
    .with_source(Arc::new(MemorySource::from_pairs([("host", "localhost")])));

layered.reload_into(&store)?;
assert_eq!(store.try_get("host")?.as_deref(), Some("localhost"));

let snapshot = store.try_snapshot()?;
assert_eq!(snapshot.get("host"), Some("localhost"));
# Ok::<(), kernel::XError>(())
```

## 读取与失败语义

| API | 语义 |
| --- | --- |
| `get` | 兼容路径；缺失或读锁中毒均为 `None` |
| `try_get` | `None` 只表示缺失；毒锁返回错误 |
| `ConfigSnapshot::capture` | 兼容路径；毒锁折叠为空快照 |
| `try_snapshot` / `try_capture` | 毒锁返回错误 |
| `get_secret` / `subset_snapshot` | 兼容辅助路径；毒锁折叠为缺失 / 空快照 |
| `try_get_secret` / `try_subset_snapshot` | Result 辅助路径；毒锁显式返回错误 |
| `require_keys` / `require_nonempty` | 基于单个 Result 快照校验 |
| `merge_into` | overlay 读取失败显式返回错误 |

生产校验和 merge 使用 Result 路径。需要跨多个 key 的一致视图时使用快照，不要串联多次独立 `get`。

## 原子批量与手动 reload

- `extend_pairs`、`LayeredConfig::apply_to`、`merge_into` 在锁外准备批次，以一次写锁提交。
- `reload_into` 完整加载并执行 key 校验后，以一次写锁替换全部 map。
- 加载、key 校验或写锁失败时，旧配置保持不变。
- `ConfigWatch::reload` 是调用方显式操作；成功替换后发送进程内 generation 通知。
- watch mutation 由独立 mutex 串行化；等待 store 写锁时不持有 watch state mutex。
- store 整图替换是配置线性化点；generation 在 mutation mutex 释放前发布。
- generation 溢出显式失败。
- `wait_outcome / wait_timeout_outcome` 区分 `Changed / TimedOut / Closed`。
- 兼容 `wait / wait_timeout` 仍返回 `Option`；timed wait 用 `try_lock` 保证 mutex 竞争受总 deadline 限界。
- timed wait 在接受 `Changed` 前再次检查 deadline；deadline 后 generation 增长仍返回 `TimedOut`。
- state 可立即观察时，已关闭 watch 即使零时限也返回 `Closed`；锁竞争仍受总 deadline 限界。

## 错误合同

- 用户可见 `XError` context 使用简体中文。
- 新 Result / wait API 的 rustdoc 通过 `# Errors` 说明失败条件。
- 关键失败测试精确断言 `ErrorKind::Invalid` 与完整 context。

## secret 诊断边界

`SecretString`、`ConfigSnapshot::Debug` 与 `MemorySource::Debug` 对 `secret:` 前缀值脱敏。
KEY=VALUE 解析错误不回显原始行。读取 API 仍返回原始字符串；这不是加密、权限控制或远端 secret
托管。调用方若未使用 `secret:` 前缀，组件无法自动识别敏感值。

## 验证

```bash
cargo fmt -p configx -- --check
cargo test -p configx --all-targets
cargo clippy -p configx --all-targets -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p configx --filter crates/infra/configx/src
cmp .agents/ssot/configx/spec/spec.md \
    .agents/ssot/configx/spec/xhyper-configx-complete-spec.md
```

## 非职责

- 自动文件 watcher、后台轮询、远端动态推送
- 分布式配置中心、多机一致性、服务发现
- 类型化 JSON / TOML / YAML schema 产品
- secret 加密、访问控制或远端 secret manager
