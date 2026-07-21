# Cargo 短名 vs xhyper-* 文档对齐（infra-s9t.11）

| 文档/叙事常见名 | `cargo test -p` **权威短名** |
|-----------------|------------------------------|
| xhyper-kernel | `kernel` |
| xhyper-testkit | `testkit` |
| xhyper-configx | `configx` |
| xhyper-evidence | `evidence` |
| xhyper-observex | `observex` |
| xhyper-resiliencx | `resiliencx` |
| xhyper-schedulex | `schedulex` |
| xhyper-transportx | `transportx` |
| xhyper-contracts | `contracts` |
| xhyper-decimalx | `decimalx` |
| xhyper-canonical | `canonical` |
| xhyper-bootstrap | `bootstrap` |

**规则**：README / CI 示例命令使用短名；`Cargo.toml` `package.name` 以仓库为准。
