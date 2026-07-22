# R1 — 权威与当前事实闭合

| 字段 | 值 |
|---|---|
| Beads | `infra-2d9.7` |
| 输入 SHA | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| 输入分支 | `feat/infra-2d9.7-core-types-3x` |
| 轮次目标 | active SSOT、公开 seam、实现、测试与声明一致；修复已证 fail-open |
| 声明边界 | kernel L1/L4 已证面；testkit T0/L1；decimalx L1 checked；canonical L2 committed subset |

## Clause / API / test / claim 矩阵

| 域 | R1 发现 | 修复后合同 | 公开 seam 证据 | R1 状态 |
|---|---|---|---|---|
| kernel | `ClockDomain`/共享 origin/`wait_timeout` 已实现但 active spec 未登记；不可表示 deadline 被降格为普通 timeout | active spec 登记当前时间域语义；`wait_timeout -> Result<bool, WaitTimeoutError>` | `Duration::MAX -> DeadlineOverflow`；常规 trigger/timeout 保持 `Ok(bool)` | 聚焦测试 PASS；全门禁待 R3 |
| testkit | spec 声称仅四类型；runner 将时钟错误变成 epoch 0，panic/re-run 可 fail-open | 区分进程内 deterministic runner 与外部 integration harness；消费型 builder + terminal report/error；step 前后 fault/snapshot 失败均 terminal | `harness_fail_closed` 覆盖 success/source/panic/pre/post fault 与 snapshot getter | 聚焦测试 PASS；全门禁待候选重跑 |
| canonical | active spec 仍称大部分 DTO uncommitted；coarse 查询无法区分 v1–v1.3；ns→ms 仅截断 | 12 个 committed 类型登记；`WireVersion` 精确查询；新增无损 ns→ms | public surface + unit 精确版本/精度损失拒绝 | 聚焦测试 PASS；全门禁待 R3 |
| decimalx | active spec仍描述公开字段/旧错误；`i128::MIN` 的 Display 文本无法读回；错误转换丢 source | 私有值类型、checked path、内部 JSON v1；全可表示值文本往返；source chain | `boundary_matrix` 覆盖 MIN scale 0/1/18、全域 property 与 `XError::source` | 15/15 PASS；全门禁待 R3 |

## Red → green 证据

| 切片 | Red（修复前） | Green（修复后） |
|---|---|---|
| decimalx | `cargo test -p decimalx --test boundary_matrix`：2 FAIL（MIN 解析 `MantissaOverflow`；`XError::source == None`） | 同命令：15 PASS |
| kernel | `cargo test -p kernel --lib wait_timeout_huge_timeout_checked_add_no_panic`：bool 无 `expect_err` 且无 `WaitTimeoutError` | 同命令：1 PASS |
| canonical wire | `cargo test -p canonical --test public_api_surface shape_and_time_and_wire_surface`：缺 `WireVersion`/`committed_wire_version` | 同命令：1 PASS |
| canonical time | `cargo test -p canonical --lib exact_nanos_to_millis_rejects_precision_loss`：缺函数 | 同命令：1 PASS |
| testkit | `cargo test -p testkit --test harness_fail_closed`：缺 typed report/outcome，旧 runner 仍用 `String`/借用记录 | 同命令：6 PASS（含首错停止、非文本 panic、step 前后 fault） |

R1 首次独立审查另发现：候选仅记录 step 后快照，且 fault 仍标 `Passed`，与 active spec 不一致。该候选被判 NO-GO；修复后同一测试扩为 6 PASS，并新增 step 前 fault、首错停止与非文本 panic。首次审查快照不得复用，须对新候选重审。

## 可重放证据

- 工具链：`rustc 1.97.0`、`cargo 1.97.0`、`node v24.14.0`。
- 实现候选内容指纹（写入 evidence 前的完整 diff SHA-256）：`0f7bddd7765971a6da3e0f1720d0fb0adf113d50a6337050f0ab554da08b490b`。
- Red：[`evidence/r1-red.txt`](evidence/r1-red.txt)，记录命令、退出码和失败 seam。
- Green：[`evidence/r1-green.txt`](evidence/r1-green.txt)，记录 focused 命令、退出码、关键计数与专项门禁。
- 首次独立审查：NO-GO；其候选在审查期间发生漂移，裁决不升级。修复候选必须以 commit SHA 冻结后重审。
- 固定 `fc201d7` 重审：旧 P0 均已关闭，但因 runner compile-fail/空场景/assert helper、组合失败 source 与 SSOT 漂移仍判 NO-GO。
- 修复提交 `6e4e599`：补 3 个 rustdoc compile-fail、8 个 runner fail-closed 场景、组合观测错误链与全部已报 SSOT/evidence 漂移；四域 focused 门禁及 kernel loom 重新通过。其后的 evidence-only 提交为新审查 HEAD。
- 固定 `241b055` 重审：仅剩 runner snapshot synchronization、成功后重跑 compile-fail 与 diff 指纹不可复算三项证据缺口。
- 合同证据提交 `e58976f`：新增 step 前/后 snapshot synchronization 两个 runner 单测、成功后重跑 rustdoc compile-fail，并记录可直接复算的 base→candidate diff SHA-256；同一 focused 闭包再次 PASS。

## 版本与影响

| crate | 版本 | 原因 |
|---|---|---|
| kernel | `0.3.0 → 0.3.1` | 公开返回签名与 typed error |
| testkit | `0.1.2 → 0.1.3` | runner 公开接口与 fail-closed 行为 |
| decimalx | `0.1.1 → 0.1.2` | 文本解析边界与错误链行为 |
| canonical | `0.1.1 → 0.1.2` | 新增精确 wire/time 查询 |

`scripts/version/crate-bump.mjs` 已同步所有 path dependency version；每个 crate 本 PR 只 bump 一次。

## R1 residual / 下一轮输入

| ID | 状态 | R2 验证方向 |
|---|---|---|
| R1-K-API | OPEN | 根 re-export/API baseline、所有 `wait_timeout` consumer 与 loom 配置差异 |
| R1-T-CONC | OPEN | ManualClock 并发测试必须用 barrier 且断言可验证的不撕裂关系 |
| R1-T-DOMAIN | DEFER | allocator 耗尽仍只声明 process-lifetime 未耗尽范围 |
| R1-C-WIRE | OPEN | 每个 exact version inventory 与 Envelope 显式校验负向路径 |
| R1-D-BOUND | OPEN | MIN/MAX × scale 边界、舍入表、serde duplicate/missing 分类 |
| R1-CROSS | OPEN | API baseline、文档、CHANGELOG、alignment 与版本一致性 |

R1 不宣称最终完成；R2 必须以本轮输出状态为输入执行新的故障/边界闭环。
