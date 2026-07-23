# configx Round 01 — 原子快照与失败语义发现

> 历史版本说明：本轮执行者未修改版本；root 已在第 3 轮发布准备阶段完成 PATCH bump 至 `0.1.2`。

状态：实现与定向测试已完成，等待独立 review / gate
Base commit：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`

## 范围

本轮只加固 `crates/configx` 的进程内手动 reload、批量提交、读失败语义、快照诊断和 watch 边界。
不新增依赖、feature、版本号、自动 watcher、远端源或后台 runtime。

## 基线发现

1. `reload_into` 先 `clear` 再逐项 `set`，并发快照可见空表或半表。
2. `extend_pairs`、`apply_to` 与 `merge_into` 逐项拿写锁，批量语义不是原子提交。
3. `require_keys`、`require_nonempty` 与 `merge_into` 使用折叠型读取，毒锁会伪装成缺失或空 overlay。
4. `ConfigSnapshot` 派生 `Debug`，会输出 `secret:` 键对应明文。
5. watch generation 用 `saturating_add`，达到 `u64::MAX` 后重复同一代。
6. `wait_timeout` 每次 Condvar 伪唤醒都重新使用完整 timeout，可能无限延长。
7. active / complete spec 虽已 `cmp` 一致，但内容落后于现有多源与进程内 watch 实现。

## 本轮裁定

- 所有批量输入先在锁外完整准备，再以单写锁提交。
- reload 完整 load 并执行 `validate_key` 后，直接替换整张 map；失败保留旧 map。
- 保留 `get / capture` 兼容折叠语义，新增 `try_get / try_snapshot / try_capture`。
- 生产校验与 merge 必须基于单个 Result 快照。
- `ConfigSnapshot::Debug` 仅按 `secret:` 前缀脱敏；读取仍是明文。
- generation 用 `checked_add`；watch reload 在溢出时不替换 store。
- timeout 以开始时刻和剩余时长实现总 deadline，伪唤醒不得重置。
- 文档只声明进程内、调用方手动 reload。

## 证据

| 合同 | 测试 |
| --- | --- |
| 批量提交中点不可见 | `extend_pairs_does_not_expose_partial_commit` |
| reload 只见完整旧/新快照 | `reload_readers_only_observe_complete_snapshots` |
| load / key 校验失败保留旧值 | layered 两个失败测试 |
| poison 与 missing 可区分 | `poison_semantics`、`production_validation_reports_poison` |
| merge overlay poison 显式失败 | `merge_into_reports_when_overlay_read_poisons` |
| 快照 secret Debug 脱敏 | `snapshot_debug_redacts_secret_values` |
| generation 溢出显式失败 | watch 两个 overflow 测试 |
| 伪唤醒不重置 timeout | `wait_timeout_uses_total_deadline_across_spurious_wakes` |

已运行：

```text
cargo fmt -p configx -- --check
结果：exit 0

cargo test -p configx --all-targets
结果：exit 0；34 单元 + 3 并发集成 + 7 公开 API，bench/example target 同步通过

cargo clippy -p configx --all-targets -- -D warnings
结果：exit 0（首轮发现排序风格 lint，最小修复后第 2 轮通过）

cmp .agents/ssot/configx/spec/spec.md \\
    .agents/ssot/configx/spec/xhyper-configx-complete-spec.md
结果：exit 0
```

完整命令输出与退出码由 Executor 交付证据汇总；本文不扩大为全 workspace 门禁结论。

## 保留风险

- 多次独立 `get` 可跨 reload；需要一致多 key 读取时必须使用快照。
- `secret:` 是调用方分类约定，未加前缀的敏感值无法自动脱敏。
- `RwLock` / `Mutex` 不承诺公平性或无饥饿。
- 当前 key 校验不是类型化 value schema。
- 当前测试与门禁不证明远端配置、自动 watcher 或 Production Ready。
