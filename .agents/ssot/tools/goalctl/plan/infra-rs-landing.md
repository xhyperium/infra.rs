# infra.rs 本仓落地说明 — goalctl

| 字段 | 值 |
|------|-----|
| package | `goalctl` |
| 实现路径 | `tools/goalctl` |
| workspace member | **是**（#188） |
| 最小 CLI | 见 crate README |
| 对齐 | [docs/ssot/tools-ssot-alignment.md](../../../../docs/ssot/tools-ssot-alignment.md) |
| package stable / 全量 authority | **未宣称** |

## 验证

```bash
cargo test -p goalctl --all-targets
cargo run -p goalctl -- --help
```
