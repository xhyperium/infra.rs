# configx 当前实现规范

状态：`configx` `0.1.1` active current-state 合同；声明面已实现，**非配置平台 Production Ready**。

## 1. 权威与边界

- Package / lib / path：`configx` / `configx` / `crates/configx`。
- 生产依赖仅 `kernel`；`default = []`；`publish = false`。
- 本文只记录 Cargo/source/tests 可观察事实，不用历史 COMPLETE 或未来目标扩大实现声明。
- Candidate Draft 仅作输入，不覆盖本文。

## 2. 可观察实现

| 能力 | 当前实现 | 证据 |
|------|----------|------|
| 内存 KV | `ConfigStore`：线程安全 String key/value；读锁中毒折叠为缺失，写锁中毒返回 `XError::Invalid` | `src/lib.rs` |
| 配置源 | `ConfigSource`；`MemorySource`、`EnvSource`、`FileSource` | `src/source.rs` |
| 文件格式 | `parse_key_value_file` 解析受限 `KEY=VALUE` 文本 | `src/source.rs` |
| 分层 | `LayeredConfig` 按注册顺序加载，后源覆盖前源；失败时先保留旧 store | `src/layered.rs` |
| reload/通知 | `ConfigWatch::reload` 由宿主显式调用；`ConfigSubscription` 提供进程内等待/超时/关闭 | `src/watch.rs` |
| secret 最小面 | `SecretString` 的 Debug/Display 脱敏；`set_secret` / `get_secret` | `src/secret.rs` |
| 快照与差异 | `ConfigSnapshot`、`ConfigDiff`、subset/agreement helper | `src/{lib,diff,view}.rs` |
| 最小校验 | key 合法性、必填 key、非空值 | `src/lib.rs` |

多源在本文中指 Memory/Env/File 三类**本地拉取源**与确定性分层，不表示远端控制面。

## 3. 行为合同

1. `LayeredConfig` 后加入的源优先级更高；`reload_into` 在全部源加载成功后才清空并替换 store。
2. `EnvSource` 只采集指定前缀并剥离前缀；`FileSource` 只在调用 `load`/reload 时读取。
3. `ConfigWatch` 不创建后台任务；reload 与 notify 都由宿主触发。
4. `SecretString` 只约束自身格式化输出；底层 store 仍持有 String，调用方必须控制明文暴露和日志路径。
5. 当前 API 不承诺批量写原子性、公平锁、类型化 schema、分布式一致性或动态服务发现。

## 4. OPEN 与禁止声明

以下能力**未验证或未实现**：

- 远端配置中心、远端推送、多机一致性与服务发现；
- 自动 File watcher、后台轮询、去抖/背压和 runtime 生命周期编排；
- JSON/TOML/YAML 完整 schema、类型化配置与迁移；
- secret manager/KMS、静态或内存加密、访问审计与自动轮换；
- package stable、workspace Production Ready 或 Agent L5。

因此不得把 `ConfigWatch::reload` 写成自动文件热监听，也不得把 `SecretString` 写成 secret manager。

## 5. 验证与验收

```bash
cmp .agents/ssot/configx/spec/spec.md \
  .agents/ssot/configx/spec/xhyper-configx-complete-spec.md
cargo test -p configx --all-targets
cargo clippy -p configx --all-targets -- -D warnings
cargo fmt --all --check
```

通过条件：上述可观察实现与测试一致；OPEN 保持显式；文档不把本地多源/reload/脱敏 wrapper 扩大为远端配置产品。
