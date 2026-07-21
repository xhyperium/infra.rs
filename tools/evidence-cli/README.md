# evidence-cli

SPEC-EVIDENCE-002 §25 只读命令行工具。

```bash
cargo run -p xhyper-evidence-cli -- verify --root ./data
cargo run -p xhyper-evidence-cli -- head --root ./data
cargo run -p xhyper-evidence-cli -- inspect --root ./data --chain <64hex>
cargo run -p xhyper-evidence-cli -- export --root ./data --chain <64hex>
cargo run -p xhyper-evidence-cli -- vectors verify
```

## 退出码

| Code | 含义 |
|------|------|
| 0 | success / valid |
| 2 | invalid arguments |
| 3 | chain invalid |
| 4 | checkpoint/signature invalid |
| 5 | storage unavailable |
| 6 | unsupported version |
| 7 | repair required（incomplete tail） |

默认只读；`repair-tail` 需 `--confirm`。
