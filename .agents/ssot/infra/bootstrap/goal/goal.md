# bootstrap — Goal

> 状态：`infra-2d9.9` 第 3 轮候选准备中；目标尚未闭合。

## 目标

把 `bootstrap 0.3.3` 收敛为可固定复验的 L1 组合根候选：四条成功 build 路径保有明确的 shutdown
所有权，graceful shutdown 遵守 signal-before-drain，ownerless 路径 fail-closed，drain poison 错误不被
误分类，同时持续拒绝 Service Locator 与超出同步进程内 drain 的能力声明。

## 可验证完成条件

| 条件 | 当前状态 |
|---|---|
| shutdown owner / drain / poison 缺陷闭合 | 已实现；第 2 轮独立代码/规格复审通过 |
| 行覆盖率 100% | 最终 root 串行确认 exit 0；`963 / 963`，zeros 0，100.0000% |
| active / complete spec 同构 | 本轮 writer 复验 |
| 本地独立 reviewer | 已完成实现/证据审查；纯状态 delta 不改变受审源码/测试 |
| 独立 verifier | 已完成技术/证据初验 |
| GitHub 固定提交 CI artifact | Pending |
| PR、维护者审批与合并 | Pending |

完成条件不包含 async drain/cancel、panic 隔离、生产关停 SLA、完整应用运行时或 package stable。
