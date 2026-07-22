# configx 公开 API

本文记录 `configx 0.1.2` 的公开消费面与失败语义。组件只提供进程内能力；reload 必须由调用方显式触发。

## ConfigStore

| API | 返回 | 合同 |
| --- | --- | --- |
| `new / Default` | `ConfigStore` | 空 store |
| `get` | `Option<String>` | 兼容：缺失或读锁中毒均为 `None` |
| `try_get` | `XResult<Option<String>>` | `None` 仅缺失；毒锁为错误 |
| `set / remove / clear` | `XResult<_>` | 写锁中毒为错误 |
| `extend_pairs` | `XResult<()>` | 锁外收集，以单写锁提交全部批次 |
| `try_snapshot` | `XResult<ConfigSnapshot>` | 获取一个完整 map 视图 |
| `len / keys / contains_key` | 折叠型返回 | 兼容 API；毒锁折叠为空语义 |

`store_from_pairs` 使用原子 `extend_pairs` 构建新 store。`set_checked` 只校验 key，不校验 value schema。

## ConfigSnapshot

| API | 合同 |
| --- | --- |
| `capture` | 兼容路径；读锁中毒返回空快照 |
| `try_capture` | Result 路径；读锁中毒返回错误 |
| `get / len / is_empty / keys` | 读取快照内的稳定视图 |
| `Debug` | `secret:` 前缀键的值显示为 `***` |

脱敏只作用于 `Debug`。快照 `get` 仍返回明文，因此不得把快照视为 secret 容器。

## 校验与合并

- `require_keys` 和 `require_nonempty` 各自基于一次 `try_snapshot`；poison 不会伪装成缺失。
- `merge_into` 先显式读取 overlay 快照，再以单写锁提交到 base；overlay poison 时 base 不变。
- `validate_key` 只检查非空、无控制字符和 512 字节上限，不是类型化 schema。
- `try_subset_snapshot` 显式报告 poison；兼容 `subset_snapshot` 将 poison 折叠为空快照。
- `try_get_secret` 显式报告 poison；兼容 `get_secret` 将 poison 折叠为缺失。

## LayeredConfig

| API | 合同 |
| --- | --- |
| `load_merged` | 完整加载所有源并校验 key；后源覆盖前源 |
| `apply_to` | 原子覆盖/新增一个批次，保留其他旧键 |
| `reload_into` | 完整加载/校验后原子替换整个 store |

读取者通过单次 `try_snapshot` 只会看到完整旧 map 或完整新 map。跨多次独立 `get` 不构成事务。

## ConfigWatch

| API | 合同 |
| --- | --- |
| `notify` | generation 以 `checked_add` 递增；关闭或溢出返回错误 |
| `reload` | 调用方手动加载；store 整图替换后发布 generation |
| `subscribe` | 从当前 generation 之后等待 |
| `wait_outcome` | 显式返回 `Changed / Closed` |
| `wait_timeout_outcome` | 显式返回 `Changed / TimedOut / Closed`；锁竞争受总 deadline 限界 |
| `wait / wait_timeout` | 兼容 Option 路径；timeout / closed 折叠为 `None` |
| `close` | 唤醒订阅者并禁止后续 notify |

`notify / reload / close` 使用独立 mutation mutex 排序。reload 完成 state 检查后释放 state mutex，
再等待 store 写锁；因此等待 store 时不会阻塞 state 读取。store 替换是配置线性化点，generation
在 mutation mutex 释放前发布。

`ConfigWatch` 不是自动 watcher，不监控文件、环境变量或远端配置。

`wait_timeout_outcome` 在读取 state 前和接受新 generation 前都检查 deadline。若 deadline 已到，返回
`TimedOut` 且不推进 subscription 的 `seen`，即使 generation 已在最后观察窗口增长。
state 可立即观察且 watch 已关闭时，`Closed` 优先于同时到期的 deadline；若 state 锁正在竞争，
实现不会为确认关闭而越过总 deadline 阻塞。

所有 Result / wait API 的 rustdoc 都包含 `# Errors`；用户可见 `XError` context 为简体中文。

## 诊断安全

- `ConfigSnapshot::Debug` 与 `MemorySource::Debug` 对 `secret:` 值输出 `***`。
- KEY=VALUE parse 错误只报告行号与类别，不回显原始行。
- 未标 `secret:` 的值不保证自动脱敏。
