# configx SSOT 对齐与本仓落地状态

| 字段 | 值 |
| --- | --- |
| Active spec | `.agents/ssot/configx/spec/spec.md` |
| Complete copy | `.agents/ssot/configx/spec/xhyper-configx-complete-spec.md`（必须 `cmp` 一致） |
| 本仓实现 | `crates/configx` · package/lib `configx` · version `0.1.2` |
| Round baseline | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| 最终 Review base | `origin/main@630f03d5db5739a89933fe921d7615841fde3789`（rebase 后固定基线） |
| 当前结论 | `f904ecd` 实现内容已审且无 finding，rebase 后等价为 `eba66fb`；fixed HEAD 完整门禁已通过；最终独立 verifier 待文档修正后复核；GitHub 新 HEAD CI/审批 pending，不宣称 Production Ready |

## 能力裁定

| 能力 | 状态 | 证据与边界 |
| --- | --- | --- |
| 内存字符串 KV | PASS | `ConfigStore` |
| Result 读取/快照 | PASS | `try_get / try_snapshot / try_capture / try_subset_snapshot` |
| Result secret | PASS | `try_get_secret`；兼容 `get_secret` 保留折叠 |
| 兼容折叠读取 | PASS | `get / capture` 保留 poison 折叠 |
| 原子批量提交 | PASS | `extend_pairs / apply_to / merge_into` 单写锁 |
| 多源优先级 | PASS | `LayeredConfig` 组合 `MemorySource` / `EnvSource` / `FileSource`；后源覆盖前源 |
| reload | PASS（进程内手动） | 完整 load + key 校验后单写锁替换 |
| 更新通知 | PASS（进程内手动） | mutation 串行；显式 wait outcome；无自动 watcher |
| 诊断脱敏 | PASS | `ConfigSnapshot` / `MemorySource` Debug 对 `secret:` 值输出 `***`；`SecretString` 的 Debug / Display 不泄露明文 |
| parse 错误脱敏 | PASS | 不回显原始配置行 |
| generation 溢出 | PASS | `checked_add`；溢出不替换 store |
| timeout 总 deadline | PASS | state `try_lock`；锁竞争与伪通知不重置时限 |
| deadline 后 generation | PASS | 接受 Changed 前二次判时；late notify 返回 TimedOut |
| closed 与 deadline 同时成立 | PASS | state 可立即观察时 Closed 优先；零时限回归测试 |
| 中文错误合同 | PASS | 用户可见 XError 中文；关键测试精确断言 kind/context |
| 类型化 schema | OPEN | 仅 key 形状校验，不校验 value 类型 |
| 远端配置中心 | NOT IMPLEMENTED | 无远端源、推送或多机一致性 |
| 自动文件 watcher | NOT IMPLEMENTED | `FileSource` / `EnvSource` 只在显式调用时加载 |
| secret manager | NOT IMPLEMENTED | 脱敏不是加密、权限控制或托管 |

## 原子性与失败矩阵

| 场景 | 合同 | 测试证据 |
| --- | --- | --- |
| 批量迭代器尚未收集完成 | store 保持旧状态 | `extend_pairs_does_not_expose_partial_commit` |
| reload 与 store 等待 | per-watch phase hook 精确证明 state 已释放、mutation 仍持有 | `reload_releases_state_lock_while_waiting_for_store_and_serializes_notify` ×100 |
| source load 失败 | store 不变 | `reload_preserves_on_source_error` |
| key 校验失败 | store 不变 | `reload_preserves_on_validation_error` |
| overlay poison | merge 返回错误，base 不变 | `merge_into_reports_when_overlay_read_poisons` |
| 校验 store poison | 返回 poison 错误，不伪装成 missing | `production_validation_reports_poison` |
| generation 溢出 | notify 报错；watch reload 不替换 store | watch 溢出两测 |
| reload 等待 store | state 锁可获取；notify 排为下一 generation | `reload_releases_state_lock_while_waiting_for_store_and_serializes_notify` |
| state 锁被占用 | timed wait 按 deadline 返回 | `wait_timeout_is_bounded_when_state_mutex_is_held` |
| 实际伪通知 | 握手且通知计数 > 1；不延长 deadline | `wait_timeout_deadline_survives_actual_spurious_notifications` |
| deadline 后通知 | 实际 notify 后仍返回 TimedOut，seen 不前移 | `wait_timeout_rejects_generation_arriving_at_deadline` |

## 公开语义边界

- `get` 的 `None` 仍可能表示读锁中毒；需要区分时使用 `try_get`。
- `ConfigSnapshot::capture` 仍可能把毒锁折叠为空；生产校验/merge 使用 Result 快照。
- `get_secret / subset_snapshot` 保留折叠；需要区分 poison 时使用对应 `try_*` API。
- 单次快照具有完整 map 视图；多次独立 `get` 可跨 reload，不是事务。
- `ConfigSnapshot::Debug` 只按 `secret:` 前缀脱敏；读取值仍为明文。
- `ConfigWatch::reload` 必须由调用方触发，不监控文件或远端变化。
- 兼容 `wait / wait_timeout` 的 `None` 仍有歧义；新调用方使用显式 outcome。

## 定向验证

```bash
cargo fmt -p configx -- --check
cargo test -p configx --all-targets
cargo clippy -p configx --all-targets -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p configx --filter crates/configx/src
cmp .agents/ssot/configx/spec/spec.md \
    .agents/ssot/configx/spec/xhyper-configx-complete-spec.md
```

完整验证结果记录在 `.agents/ssot/configx/plan/round-03-findings.md`；定向通过不扩大为全 workspace
可靠性声明。Round 1/2 执行者未改版本，root 已在发布准备阶段统一 PATCH bump 至 `0.1.2`。

## 非目标 / 开放项

- 自动 watcher、后台轮询、远端动态推送
- 分布式配置中心、服务发现、多机一致性
- 类型化 JSON / TOML / YAML schema
- secret 加密、访问控制与远端托管
- package stable、workspace Production Ready、Agent L5
