# GATE-SCHEDULEX-003

状态：LOCAL IMPLEMENTATION GATES PASS；RELEASE GATES BLOCKED

- G1 active 双镜像 `cmp`。
- G2 public seam tests 与全 crate all-targets。
- G3 fmt、clippy `-D warnings`、rustdoc。
- G4 canonical LCOV 行覆盖率门禁。
- G5 public API ratchet无 removal/signature change。
- G6 std-only、workspace deps、crate version/path version。
- G7 独立 Standards/Spec review。
- G8 PR required checks、最后一次 push 后 CODEOWNER 人工审批。

任一 breaking API、非目标能力、伪造 coverage/live 或双镜像漂移均 BLOCKED。

2026-07-23 scoped 证据：50 tests、fmt、clippy、rustdoc、doc-test、LCOV 768/768、显式 schedulex API baseline 均通过。发布仍被根 `AGENTS.md` 治理漂移、默认 API CI 接线、版本/lock/STATUS、contract-testkit、最终全仓 verifier、PR CI 与人工审批阻断。
