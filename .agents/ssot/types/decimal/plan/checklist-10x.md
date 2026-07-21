# 10x Checklist — PLAN-TYPES-DECIMALX-002-agent-safe-v1

每轮全部 PASS 才计该 round PASS。`fail_rounds` 必须为 0。

1. **SSOT**：Active 仍为 `decimalx-spec.md`；Draft 在 `20260717/` 且 Status 含 Draft；plan 引用 `SPEC-TYPES-DECIMALX-002` / `GOAL-TYPES-DECIMALX-002`。
2. **REJECTED 不回流**：plan/residual 含 numeric 路径 / canonical 环 / 默认 Money\<U\> 禁止。
3. **todo 纪律**：每个 AGENT_SAFE 任务为 DONE 且有证据路径；HUMAN_ONLY/DEFERRED 不标 DONE。
4. **inventory**：`plan/evidence/m0-consumer-inventory-2026-07-17.txt` 存在且含 consumers。
5. **# Panics**：`src/lib.rs` 中 Add/Sub/Mul/rescale 有 `# Panics` 或等价 panic 文档。
6. **cargo test -p xhyper-decimalx** 成功。
7. **cargo check -p decimalx --all-targets** 成功。
8. **cargo clippy -p decimalx --all-targets -- -D warnings** 成功。
9. **cargo fmt -- --check**（触及路径）成功。
10. **对齐不越权**：alignment 不宣称 Approved/Achieved/wire stable/package stable。
11. **tip-stable**：round 内 `git rev-parse HEAD` 不变（或记录变更原因）。

解释边界：10x PASS ≠ Goal ACHIEVED ≠ Spec Approved ≠ Cutover。
