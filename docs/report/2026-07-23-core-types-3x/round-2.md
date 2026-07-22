# R2 — 故障注入与边界对抗

| 字段 | 值 |
|---|---|
| 输入 gate HEAD | `e0cc99d716638663cd605da749cc1988026528d9`（R1 GO） |
| R2 最终候选 | `6c8da41ca0b9105b8e7fa9312fcceee793775b63` |
| 轮次目标 | 两个真实行为 RED 修复 + 两个公开 seam adversarial 回归加固 |
| 声明边界 | kernel L1/L4 已证面；testkit T0/L1；decimalx L1 checked；canonical L2 strict serde JSON subset |

## 对抗矩阵

| 域 | 类型 | R2 发现/加固 | 结果 |
|---|---|---|---|
| kernel | 真实 RED | 已触发信号仍先构造 `Duration::MAX` deadline，完成事实被 `DeadlineOverflow` 覆盖 | 完成状态改为优先返回 `Ok(true)` |
| decimalx | 真实 RED | 256 位小数经 `len as u8` 被诊断为 scale 0 | 先 checked 转换；超出 u8 时以 `Parse` 保留真实长度 |
| testkit | 绿色回归加固 | 既有并发测试只有单控制者 | 新增 8 控制者同时推进、最终值无丢失更新 |
| canonical | 绿色回归加固 | 公开 integration seam 未穷举 12 个 exact version 与 Envelope/time 负向 | 增加 12/12 table、负/重复/缺失/未知字段、显式版本错误与 exact time 边界 |

后两项在未改生产实现的 R1 输入上即通过，不能写成 RED→GREEN；它们是 adversarial regression evidence。

## RED → GREEN

- kernel RED：`cargo test -p kernel --lib wait_timeout_completed_state_precedes_deadline_validation` → exit 101，`Err(DeadlineOverflow) != Ok(true)`。
- decimalx RED：`cargo test -p decimalx --test boundary_matrix oversized_fraction_length_never_wraps_in_diagnostic` → exit 101，`Scale != Parse` 且旧诊断会窄化为 0。
- 两项最小实现后相同命令均 exit 0。
- testkit/canonical 回归命令在生产实现修改前已 exit 0，分别为 1/1 与 5/5。

## 明确拒绝

- 不实现 canonical bytes、自动 version router、codec 或跨语言协议。
- 不实现 external harness、真实服务/网络/进程 fault。
- 不把 domain allocator exhaustion 从诚实 residual 伪装为已闭合。
- 本 PR 不重复 bump 四 crate 版本，不扩张 package stable / crates.io / overall Production Ready 声明。

## Evidence

- RED 原因与退出码：[`evidence/r2-red.txt`](evidence/r2-red.txt)。
- GREEN 命令、工具链、计数与可复算指纹：[`evidence/r2-green.txt`](evidence/r2-green.txt)。
- 独立 reviewer 对候选 `6c8da41` 与 evidence HEAD `cc93223` 给出 **GO（0.97）**；原始终局输出见 [`reviews/r2-final-reviewer.md`](reviews/r2-final-reviewer.md)，并明确区分两个真实 RED 修复与两个绿色回归加固。
