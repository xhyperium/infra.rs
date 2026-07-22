# verifyctl docs

| 文档 | 说明 |
|------|------|
| 本页 | crate 文档入口 |
| [../README.md](../README.md) | 用户入口与 CLI |
| [../../docs/ssot/tools-ssot-alignment.md](../../docs/ssot/tools-ssot-alignment.md) | SSOT 对齐 |

## 职责

`verifyctl`：Goal Contract + changed paths → VerificationPlan → execute → RunResult（`verification-plan/v1` / `verification-run/v1`）。

## 命令

```bash
cargo run -p verifyctl -- plan --contract /tmp/contract.json --changed tools/verifyctl -o /tmp/plan.json
cargo run -p verifyctl -- execute /tmp/plan.json -o /tmp/run.json
cargo run -p verifyctl -- report /tmp/run.json
```
