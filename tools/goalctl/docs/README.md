# goalctl docs

| 文档 | 说明 |
|------|------|
| 本页 | crate 文档入口 |
| [../README.md](../README.md) | 用户入口与 CLI |
| [../../docs/ssot/tools-ssot-alignment.md](../../docs/ssot/tools-ssot-alignment.md) | SSOT 对齐 |

## 职责

`goalctl`：Goal YAML/JSON → Contract JSON（`goal-contract/v1`）+ 稳定 sha256 digest。

## 命令

```bash
cargo run -p goalctl -- doctor
cargo run -p goalctl -- validate tools/goalctl/tests/fixtures/good_goal.yaml
cargo run -p goalctl -- compile tools/goalctl/tests/fixtures/good_goal.yaml -o /tmp/contract.json
```
