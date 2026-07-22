# R3 — 声明收敛与仓库级验证

| 字段 | 值 |
|---|---|
| 输入 HEAD | `cc932231afafc4feff44965fe3d97facf19538cf`（R2 evidence） |
| 内容候选 | 待固定提交 SHA |
| 当前状态 | **VERIFYING** |
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

在上述条件全部满足前，本轮不写 GO，也不把 R1/R2 PASS 继承给最终候选。
