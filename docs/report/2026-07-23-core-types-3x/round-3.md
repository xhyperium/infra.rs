# R3 — 声明收敛与仓库级验证

| 字段 | 值 |
|---|---|
| 输入 HEAD | `cc93223161880fbeb41fe770e7785a89f8dadf72`（R2 evidence） |
| 初始内容候选 | `387a1dc550341767a24a1548dd9ae47b2c8b84ee` |
| 主干同步候选 | `70d402a9a8b7b796077cba33e30ddf0c069c5e03`（包含 `origin/main 5fe242c`） |
| 最终校正候选 | `ec2d938a031a659748d638902ea2d85a730335cd` |
| 终门禁阻断修复候选 | `f26e29cf8f76c6db14be7210e41bd72b04791493` |
| 最终内容候选 | `f62859b484dc3774a444623d8e2537e0204c4ca8` |
| 最终状态校正候选 | `c4604ceb6c79df310ebe91fe56c516f88b1c8a6e` |
| PR #258 coverage 修复候选 | `55b899788607bb175b47aedf4ff06dfdc2926ada` |
| 当前状态 | **REVIEW BLOCKERS FIXED / NEW CANDIDATE PENDING** |
| 轮次目标 | 当前权威、历史战役、Cargo package 真相与生成状态一致 |

## 基线 RED

R3 先在未改内容的输入 HEAD 上执行 workspace build/test/fmt/clippy/deny 与综合门禁。Rust 门禁全部通过，`check.mjs` 为 **43/44**；唯一失败是生成入库的 `STATUS.md` 与四域代码/测试规模不一致。原始结论见 [`evidence/r3-red.txt`](evidence/r3-red.txt)。

## 最小修复范围

- 由仓库生成器刷新 `STATUS.md`，不手改生成内容。
- 修正 kernel `wait_timeout` 文档的线性化顺序：持锁首次观察已完成时先返回；未完成才验证 deadline。
- 将旧版本、旧 package、Stable/COMPLETE/PASS 战役文档显式标为历史快照，指向当前 README/spec/design/test/gate/matrix。
- 修正 testkit workflow 触发事实、canonical 人工边界审查与 Envelope API、decimal 内部 serde shape 与跨语言 stable 的声明边界。
- 当前可复制命令只使用 `kernel`、`testkit`、`canonical`、`decimalx` Cargo package 选择器；历史 evidence 原文不改写。

## 停止条件

1. 固定内容候选 SHA A；
2. 在 SHA A 上执行 workspace 全量门禁、四域专项门禁与 loom；
3. 仅用证据/裁决提交形成 SHA B，并独立复核 `A..B` 为 evidence-only；
4. 独立 reviewer 审查 `origin/main...B` 全量差异并给出 GO。

内容候选上的机器门禁已闭合；独立 review 与 evidence-only diff 复核完成前，本轮仍不写 GO，也不把 R1/R2 PASS 继承给最终候选。GREEN 命令和结果见 [`evidence/r3-green.txt`](evidence/r3-green.txt)。

主干在初次终审后推进到 `5fe242c`。为保留既有三轮 Git object 与指纹，本分支以 merge commit 同步主干；冲突按“主干 storage/contracts 新版本 + 本分支四域版本”合并，kernel 设计保留 current-state 权威并吸收 `crates/evidence` 路径与新门禁约束。`70d402a` 上重新执行完整闭包，结果见 [`evidence/r3-main-sync-green.txt`](evidence/r3-main-sync-green.txt)。此前绑定 `6dbaa6f` 的 R3 GO 已按 failure conditions 失效，必须对新候选重新终审。

## 独立审查与 residual

- 首轮 Standards 轴：GO；仅发现 testkit terminal report 重复组装这一项 P2 判断性气味，已登记 follow-up `infra-1j3`。
- 首轮 Spec 轴：NO-GO；要求把 allocator residual 登记 Beads、统一 test/gate/matrix 状态，并澄清 stdout 的持久证据边界。本提交已逐项修复，等待固定新 HEAD 重审。
- `R-CLK-DOMAIN-EXHAUSTION` 保持 OPEN，已登记 `infra-lip`；本轮不实现、不关闭，也不扩张唯一性声明。
- 主干同步后的最终 Standards 与 Spec reviewer 均对 `3d34082` 给出 **GO（0.99）**；原始终局输出见 [`reviews/r3-final-standards-reviewer.md`](reviews/r3-final-standards-reviewer.md) 与 [`reviews/r3-final-spec-reviewer.md`](reviews/r3-final-spec-reviewer.md)。
- 最终聚合门禁发现 decimalx active spec 的三项验收勾选仍停留在实现前状态。实现、测试和双轴终审证据均已存在，因此本轮将其作为声明状态漂移校正；校正后的固定 SHA 必须重新通过全量机器门禁、双轴复审与最终聚合门禁后，才能恢复本轮 GO。
- 校正候选 `ec2d938a` 的全仓与四域机器门禁已全部退出 0；固定条件、命令、摘要及一次无效路径转录失败见 [`evidence/r3-final-candidate-green.txt`](evidence/r3-final-candidate-green.txt)。
- 对 `c27b7ce` 的最终聚合审查发现五项真实缺口：crate 专用错误未用 `thiserror`、HAR-04 未用 marker 证明 panic 后停止、decimal source 未断言类型身份、三个公开方法缺少行为测试、并发 reader loop 可能零次重叠。工作树已按测试先行闭合这些缺口；旧 evidence 与 reviewer GO 均不继承，新内容 SHA 固定前保持 NO-GO。
- 阻断修复候选 `f26e29c` 的全仓与四域机器门禁已全部退出 0，见 [`evidence/r3-hardening-green.txt`](evidence/r3-hardening-green.txt)；独立复审完成前仍不恢复 GO。
- 最终 verifier 随后发现 testkit alignment 的事实摘要仍残留 `prod deps kernel only`；已按 Cargo 与 active spec 校正为 `kernel, thiserror`。该文档改动使 `f26e29c` 不再是最终内容候选，必须重新固定 SHA 与复验。
- 同轮 Standards 复核还发现 `IntegrationHarness::run` 缺少组织 Rust P1 要求的 `# Errors`；修复扩展到 testkit 全部公开 fallible API，逐项写明 typed error 条件。
- 最终内容候选 `f62859b` 已重新通过完整机器闭包，见 [`evidence/r3-final-green.txt`](evidence/r3-final-green.txt)；独立双轴与聚合门禁仍待裁决。
- Spec 复审发现 canonical current-state test 的“本轮证据”仍继承 R1 reviewer GO；已改为最新机器 PASS / REVIEW PENDING，R1/R2 只保留为历史回归输入。
- 状态校正候选 `c4604ce` 已再次通过完整机器闭包，见 [`evidence/r3-final-state-green.txt`](evidence/r3-final-state-green.txt)。
- 最终 Standards / Spec / 聚合门禁均对 `c4604ce..fff07ea` 给出 GO；原始裁决见
  [`reviews/r3-final-state-standards-reviewer.md`](reviews/r3-final-state-standards-reviewer.md)、
  [`reviews/r3-final-state-spec-reviewer.md`](reviews/r3-final-state-spec-reviewer.md) 与
  [`reviews/r3-final-state-gate-reviewer.md`](reviews/r3-final-state-gate-reviewer.md)。
- PR #258 首次远端 CI 发现 `Testkit Coverage` 仅 95.9481%（25 行未覆盖），使 `bf904e3` 的最终 GO 失效。修复补齐 owned `String` panic、panic+终态观测失败、Debug/source 占位与缺失 snapshot getter 等真实路径，并以已覆盖的时钟推进步骤替代按合同不得执行的 marker 闭包；固定候选 `55b8997` 的本地同一 coverage 命令达到 100.0000%，全套 19 项机器门禁均退出 0，见 [`evidence/pr258-testkit-coverage-green.txt`](evidence/pr258-testkit-coverage-green.txt)。独立重审完成前保持 REVIEW PENDING。
- `55b8997` 的 Spec reviewer 随后发现 HAR-13 只断言 source 存在、未验证观测错误身份；最终 Verifier 发现测试直接调用私有 formatter 覆盖不可达的 `Passed` 错误状态，属于 coverage 驱动的白盒伪测试。修复将 `HarnessRunError` 内部状态收窄为只含三种失败类型，以公开 error Display 路径覆盖文案，精确 downcast `WallFaultObservation`，并用可复用 marker step 同时保留 pre-step poison 的“不执行”直接 oracle 与真实成功路径覆盖。该私有表示收窄不改变公开 API/行为，且同一 PR 已执行一次 PATCH bump，不二次 bump；新候选与全部证据必须重建。
