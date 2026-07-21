# Gap Matrix — SPEC-TESTKIT-002

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-TESTKIT-002-v1-complete` |
| Source | `xhyper-testkit-complete-spec.md` §0–§25 |
| 日期 | 2026-07-14 |

图例：`OK` · `PARTIAL` · `WRONG` · `ABSENT` · `N/A`

---

## 1. 规范章节差距

| Spec | 要求摘要 | 现状 | 状态 | 关闭 Wave | DEF |
|------|----------|------|------|-----------|-----|
| §0 | T0 test-support 平面；非 L0 | 文档/layer 仍 kernel/L0 | WRONG | W5 | DEF-001 |
| §1 | 隐式输入显式化；准入八问 | 概念未机控 | PARTIAL | W0/W6 | — |
| §2.1 | ManualClock 方向保留并收紧 | Atomic 可控但未收紧 | WRONG | W1 | DEF-002/003 |
| §2.2 | 删 xlib_test/mock/FixtureBuilder/provider | 全部仍在 | ABSENT | W3/W4 | DEF-004/005 |
| §3.1 | testkit 仅 ManualClock* 公开面 | 宏+placeholder 并存 | WRONG | W3 | DEF-004 |
| §3.2 | contract-testkit 独立包 | 不存在 | ABSENT | W4 | DEF-006 |
| §3.3 | integration harness 边界 | scripts 存在但非本战役 | PARTIAL | DEFER INFRA | — |
| §3.4 | Fixture 所有权 | 无通用 builder 跨包 | OK（碰巧） | — | — |
| §4.1 | clock.rs + tests/* | 仅 lib.rs | WRONG | W1 | DEF-002 |
| §4.2 | contract suites 目录 | ABSENT | ABSENT | W4 | DEF-006 |
| §5.1 | deps 仅 kernel | **OK** | OK | 维持 | — |
| §5.2 | 无 feature | **OK** default=[] | OK | 维持 | — |
| §5.3 | 仅 dev-dep 消费 | binance/okx 正确 | OK | W6 机控 | DEF-007 |
| §5.4 | 生产图隔离 | 粗检有；无专用命令 | PARTIAL | W6 | DEF-007 |
| §6 | forbid/deny 属性 | 缺失 | ABSENT | W1 | — |
| §7 | ManualClock 完整 API | 旧 API；无 fault/snapshot/Result | WRONG | W1 | DEF-002/003 |
| §8 | 宏退役合同 | 未退役 | ABSENT | W3 | DEF-004/005 |
| §9 | contract suite 原则/profile/负测 | 无 | ABSENT | W4 | DEF-006 |
| §10 | Mock/Fake 术语 | 未审计下游 Mock* | ABSENT | W6 | — |
| §11 | 确定性规则 | 未机控 sleep 等 | PARTIAL | W6 | — |
| §12 | API 预算仅 clock | 超标 | WRONG | W3 | DEF-004 |
| §13.1–13.5 | 测试矩阵 | 极简 unit | ABSENT | W1 | DEF-010 |
| §13.6–13.8 | mutation/coverage/Miri | 无 | ABSENT | W6 | DEF-010 |
| §13.9 | contract 自测 | 无 | ABSENT | W4 | DEF-006 |
| §14 | 图隔离门禁 001–005 | lint-deps 子集 | PARTIAL | W6 | DEF-007 |
| §15 | Archgate TESTKIT-* | **OOS**（本仓不移植 archgate） | **OOS** | — | — |
| §16 | CI 命令集 | 无专用 | ABSENT | W6 | DEF-007 |
| §17 | README/AGENTS/CHANGELOG | 仍 L0+宏 | WRONG | W5 | DEF-001 |
| §18 | incubating + layer test-support | status OK；layer WRONG | PARTIAL | W5 | DEF-001 |
| §19 | 迁移流程 Phase0–6 | 仅文档 | ABSENT | W0–W6 | — |
| §20 | PR 切分 | 计划有；未执行 | ABSENT | 全战役 | — |
| §21 | Evidence 目录 | 无 testkit change evidence | ABSENT | W7 | — |
| §22 | 1/7/30 天 | I-SCHED 映射 W0 / W1–W4 / W5–W9 | PARTIAL | W0–W9 | — |
| §23 | 指标 | 未测量 | ABSENT | W7 | DEF-010 |
| §24 | 完成定义 | 全未勾 | ABSENT | W9 | DEF-009 |
| §25 | 最终裁定方向 | 方向 OK；未落地 | PARTIAL | — | — |

---

## 2. DEF 登记

| ID | 标题 | 严重度 | 状态 | 关闭 Task |
|----|------|--------|------|-----------|
| DEF-001 | 身份：layer/docs 仍 L0/kernel | P0 | OPEN | T-ARCH-* T-DOC-* |
| DEF-002 | ManualClock 状态模型错误（Atomic/unchecked） | P0 | OPEN | T-CLK-001…012 |
| DEF-003 | 无 fault/snapshot/Result 控制 API | P0 | OPEN | T-CLK-003…010 |
| DEF-004 | 无价值公开 API（宏/FixtureBuilder） | P0 | OPEN | T-DEL-001…003 |
| DEF-005 | provider 大宏隐藏依赖+硬编码 | P0 | OPEN | T-DEL-004 T-CTC-* |
| DEF-006 | 无 contract-testkit / 负测 | P0 | OPEN | T-CTC-* |
| DEF-007 | 图隔离/CI 不全（**archgate = OOS**，不计入缺口） | P1 | OPEN | T-GATE-*（除 T-GATE-003 CANCELLED） |
| DEF-008 | 多份 active-looking specs | P1 | OPEN | T-ARCH-007 T-DOC-002/003 |
| DEF-009 | Spec 未 Approved；§24 未闭合 | P1 | OPEN | T-HUM-* T-24-* |
| DEF-010 | 质量门槛 coverage/mutation/Miri | P1 | OPEN | T-GATE-008…010 T-CLK-014…017 |

---

## 3. 实现面细目（当前源码）

| 组件 | 文件 | 问题 |
|------|------|------|
| ManualClock | `crates/testkit/src/lib.rs` | AtomicI64/U64；`new(i64)`；`set/advance/advance_mono` 无 Result；无 fault |
| xlib_test! | 同文件 | 仅包装 `#[test]` |
| mock! | 同文件 | 空 Default Clone 结构体 |
| provider_capability_contract_tests! | 同文件 | 展开依赖 canonical/contracts/futures_util/tokio；硬编码空流/0 时间/Pending |
| FixtureBuilder | 同文件 | PhantomData 占位；Copy/Default |
| 测试 | 同文件 cfg(test) | 无 property/concurrency/compile_fail |
| Cargo.toml | crates/testkit | description 写 L0；无 publish=false 显式；deps 仅 kernel OK |
| layer | workspace.toml | `layer = "kernel"` |
| consumers | binance/okx | provider 宏；dev-dep OK |

---

## 4. 与旧 SSOT 冲突矩阵

| 源 | 冲突 | 处置 |
|----|------|------|
| testkit-spec.md | 职责强制宏 | Superseded |
| testkitx-spec.md | L1 路径 | Superseded |
| architecture/spec.md | L0 含 testkit | W5 修订 |
| ADR-010 | 宏最小批准 | W5 退役备注 |
| lint_deps classify | Layer::Kernel | TestSupport |
| CLAUDE/AGENTS 摘要 | L0 列表 | W5 修订 |

---

## 5. 可保留 vs 必须删除

| 保留方向 | 必须删除 |
|----------|----------|
| ManualClock 独立 wall/mono 控制 | xlib_test! |
| 仅依赖 kernel | mock! |
| dev-only 消费模式 | FixtureBuilder |
| incubating + publish false 方向 | provider 大宏在 core |
| | Atomic 多字段「伪一致」模型 |
| | L0 runtime 身份声明 |

---

## 6. 关闭顺序（最小风险）

```text
1. W0 冻结扩散（文档+清单）
2. W1 Clock V2（无外部调用点 → 低风险）
3. W4 contract-testkit 先行可测入口
4. W3/W4 删 core 宏 + 迁 Binance/OKX
5. W5 文档/layer SSOT
6. W6 机控防回流
7. W7–W9 验收与人审
```
