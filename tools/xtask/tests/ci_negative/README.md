# CI Negative Fixtures（Spec §26 manifest）

> `manifest.toml` 是 20 项稳定 ID、maturity、driver、seam 与预期终态的机器清单。
> 当前为 11 `EXECUTABLE` + 9 `STUB`；`gate_ok=true` **不等于** `coverage_complete=true`。
> Markdown 投影见 `../ci_negative_fixtures.md`；全量 20/20、生产 Runner/Ruleset 与 Goal Achieved 均未成立。

`expected_logical_outcome = "rejected"` 表示 Rust driver 调用目标 seam 后观察到场景特异拒绝；它不是 CLI/OS 进程退出码证据。

```bash
cargo xtl ci chaos --json
```
