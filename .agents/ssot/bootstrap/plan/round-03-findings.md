# bootstrap 第 3 轮候选准备与错误语言加固记录

| 字段 | 值 |
|---|---|
| 日期 | 2026-07-23 |
| Beads | `infra-2d9.9` |
| 战役历史起点 | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| 最终 Review base | `origin/main@5fe242cefc873117d024f0d09f8ad5cbf449d2ec` |
| 当前版本 | `0.3.3` |
| 候选状态 | 治理修正后候选已重冻；本地 reviewer 完成，verifier 技术/证据初验完成 |

## 已闭合实现事实

- 第 2 轮已闭合 shutdown owner 保留、signal-before-drain、ownerless fail-closed、drain mutex
  poison 错误映射与同步 drain 边界。
- 第 2 轮独立代码/规格复审结论为通过；第 3 轮本地独立 reviewer 已完成实现/证据审查，独立
  verifier 已完成技术/证据初验。
- 能力仍限于进程内 typed composition 与同步 drain；不提供 async drain/cancel、panic 隔离、
  生产关停 SLA 或完整应用运行时。

## Fixed-SHA reviewer P0 与修复

固定 SHA reviewer 发现 ownerless `MissingDependency("shutdown_guard")` 经
`BootstrapError::Display` 暴露英文前缀，违反仓库人类可读文本简体中文规则。

- 三类 `BootstrapError` Display 分别固定为 `缺少必需依赖：{name}`、
  `bootstrap 配置无效：{reason}`、`依赖不可用：{name}`；
- `DependencyUnavailable` 顶层 Display 与 `XError` context 均不内插任意下层 source 文本；
  `#[source]` 链继续结构化保留；
- bootstrap 自有 drain 文本统一为 `关停钩子锁中毒`、`排空步骤失败`、`未知错误`；
- `examples/minimal.rs` 的断言和终端输出改为中文；
- re-export 类型、下层 source 与 hook opaque context 由定义方负责，bootstrap 不为
  翻译而包装、分叉或吞掉下层原因。

shutdown owner、ownerless fail-closed、signal-before-drain 与 LIFO 语义未修改。

## 第 3 轮机器证据

| 检查 | 结果 | 证据边界 |
|---|---|---|
| root 串行行覆盖率门禁（最终错误文本修复后） | exit 0；`963 / 963`，zeros 0，100.0000% | 共享工作树本地机器证据，不是固定提交 CI artifact |
| `cargo test -p bootstrap --all-targets` | exit 0；60 passed + 1 ignored | main `ContractStoreSet`、shutdown 与错误语言断言全绿 |
| `cargo clippy -p bootstrap --all-targets -- -D warnings` | exit 0 | 无 warning |
| `cargo fmt -p bootstrap -- --check` | exit 0 | package 格式通过 |
| `cargo doc -p bootstrap --no-deps` | exit 0 | rustdoc 生成成功 |
| active / complete spec | 本轮文档收敛后要求 `cmp` 一致 | 由 writer 交付检查复验 |
| 版本一致性 | Cargo 当前为 `0.3.3` | manifest 当前事实；未改写 Round 1/2 历史版本 |

此前 `975 / 975` 与 `961 / 961` 分别是 thiserror 修复前、最终错误文本修复前的中间树基线，
不作为当前候选覆盖率结论。

Round 1/2 记录保留当时执行者未修改 `0.3.1` 的历史事实；此前 `0.3.2` 记录保留对应阶段事实，
当前候选版本以 Cargo `0.3.3` 为准。

## 审查结论与外部待办

- Done（本地）：治理修正后候选已重冻；独立 reviewer 已完成实现/证据审查；独立 verifier 已完成
  技术/证据 AC 初验。本次纯状态 delta 不改变受审源码/测试。
- Pending（GitHub）：固定提交 CI artifact、PR、维护者审批与合并。
- Pending（发布）：合并后再判断 tag 或其他发布动作。

本记录不宣称 Production Ready、发布批准或 package stable；release 继续 BLOCKED。
