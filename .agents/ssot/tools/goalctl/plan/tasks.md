# Tasks — PLAN-GOALCTL-002-phase1.1-v1

| Task ID | GAP/AC | 描述 | Paths | Disposition | Evidence 约定 |
|---------|--------|------|-------|-------------|---------------|
| T-DOC-001 | analysis | 落盘 plan/gap/tasks/residual/CURRENT-STATE | `.agents/ssot/tools/goalctl/plan/**` | AGENT_SAFE | plan 文件存在 |
| T-DOC-002 | GAP-014 | 建立/更新 `.agents/ssot/goalctl/todo.md` | `.agents/ssot/goalctl/todo.md` | AGENT_SAFE | 全行终态 |
| T-P0-000 | MVA#1 | RepositoryView + snapshot helpers | `src/repo.rs` 或 `repository_view.rs` · lib | AGENT_SAFE | unit |
| T-P0-001 | GAP-001 · AC-P0-SNAPSHOT | artifact committed 读 + source_commit | `src/artifact.rs` · main · tests | AGENT_SAFE | dirty 不改变 index |
| T-P0-002 | GAP-002 · AC-P0-RECONCILE | 禁目录→VERIFIED/OK；Fact only | `src/reconcile.rs` · tests | AGENT_SAFE | evidence dir 不产 VERIFIED |
| T-P0-003 | GAP-003 · AC-P0-COMPILE | commit/tree 强制绑定 | `src/compile.rs` · tests | AGENT_SAFE | A+B tree fail |
| T-P0-004 | GAP-004 · AC-P0-COMPILE | 默认 compile 标明 template；不虚构 PASS | `src/compile.rs` · docs | AGENT_SAFE | task_pack 无假 VERIFIED |
| T-P0-005 | GAP-005 | ApprovalRecord 内容校验 | `src/compile.rs` · resolve fact · tests | AGENT_SAFE | bogous ref fail |
| T-P0-006 | GAP-006 | 防假 VERIFIED（无 Evidence Fact 不升 VERIFIED） | reconcile | AGENT_SAFE | 同 T-P0-002 |
| T-P0-T | MVA#9 | 负例集成测试 committed_subject / cli | `tests/**` | AGENT_SAFE | cargo test |
| T-P1-001 | GAP-007 · AC-P1-CONTRACT | `--trust-level` CLI+输出 | `main.rs` · output | AGENT_SAFE | help/CLI |
| T-P1-002 | GAP-008 · AC-P1-CONTRACT | 全命令 source-commit | main · artifact · reconcile · compile | AGENT_SAFE | CLI |
| T-P1-008 | GAP-014 | README / VERSION matrix / CHANGELOG / CURRENT-STATE | tools/goalctl · contracts · plan | AGENT_SAFE | 无「尚无 resolve」假话 |
| T-VER-001 | gates | cargo test/clippy/fmt/goal-check | — | AGENT_SAFE | SCRATCH logs |
| T-VER-002 | 10x | 十轮检查 fail_rounds=0 | plan 10x-verdict | AGENT_SAFE | 10x verdict |
| T-VER-003 | approve | PR + liukongqiang5 APPROVE | GitHub | AGENT_SAFE* | readback JSON |
| T-P1-003 | GAP-009 | Schema↔Rust conformance | — | DEFERRED | residual |
| T-P1-004 | GAP-010 · AC-P1-DETERMINISM | 跨语言 canonical golden | — | DEFERRED | residual |
| T-P1-005 | GAP-011 | Identity FULL | — | DEFERRED/POLICY | residual |
| T-P1-006 | GAP-012 | module filter 精确化 | — | DEFERRED | residual |
| T-P1-007 | GAP-013 | PathSpec 统一 | — | DEFERRED | residual |
| T-P2-001 | GAP-015 | Bootstrap Trust | — | HUMAN_ONLY | residual |
| T-P2-002 | GAP-016 | Harness/Evidence/Shadow | — | HUMAN_ONLY | residual |
| T-P2-003 | GAP-017 | SLO/Corpus/replay | — | HUMAN_ONLY | residual |
| T-POL-001 | Cutover | required CI 切 goalctl | — | POLICY | 独立 CR |
| T-HUM-001 | Goal §6.3 | 标记 GOAL ACHIEVED | — | HUMAN_ONLY | 独立审批 |

\* T-VER-003 依赖 token/GitHub；失败则 env-limit 诚实记录，不伪造。
