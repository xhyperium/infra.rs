# 可执行负向 fixtures（Spec §26 子集）

| 文件 | 覆盖 | 驱动 |
|------|------|------|
| missing_lane.json | Missing Lane | `ci aggregate --decisions-file ... --expected fast,build_test` → 非 0 |
| invalid_na.json | Invalid N/A（无 reason） | 同上 → 非 0 |
| missing_reused_attestation.json | REUSED 缺 attestation | 同上 → 非 0 |
| unexpected_skip.json | Unexpected Skip（SKIP ≠ PASS） | 同上 → 非 0 |
| cancelled_lane.json | Cancelled Lane | 同上 → 非 0 |
| aggregate_unknown_state.json | Aggregate Unknown State | 同上 → 非 0 |
| aggregate_invalid_state.json | 非白名单 `TYPO_PASS` 状态 | 归一 `UNKNOWN` → 非 0 |
| all_pass.json | 正向对照（Expected 全 RUN_PASS） | → 0 |
| flake_expired.toml | Spec §13.1 登记项 `expires < today` | `ci flake --today 2026-07-16 --registry-file ...` → 非 0 |
| fingerprint_valid.json | typed FingerprintInputV1 正向候选（不证明 provenance/reuse） | `ci fingerprint --input ...` → 0；report `reusable=false` |
| reuse_attestation_valid.json / reuse_context_valid.json | 九项 structural reuse candidate 对照（不证明 attestation/runner trust） | `ci reuse --want-reused ...` 最终仍为 `RUN` |
| no_lookahead_violation.json | available_at > decision_as_of | `ci no-lookahead --fixture ...` → 非 0 |

其余清单项见 `tools/xtask/tests/ci_negative_fixtures.md`（逐步可执行化）。
20 项 maturity/driver/seam 的机器 SSOT 为上级目录 `manifest.toml`。

**Aggregate 无 decisions_file**：默认 FAIL（禁止全绿）；仅 `--synthetic-smoke` 可本地合成。
