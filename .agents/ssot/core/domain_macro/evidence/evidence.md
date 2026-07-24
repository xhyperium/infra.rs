<!-- ssot:trace=domain_macro.evidence.001 -->
# domain_macro 验证证据

| 事实 | 当前状态 | 证据 |
|---|---|---|
| workspace member | 已核实 | `Cargo.toml` 仅含 `crates/macrox` |
| 现有实现 | 部分 | `crates/macrox/src/lib.rs` 仅有 `IndicatorCategory`、`DataPoint<T>` |
| DM-V01–DM-V07 | 未验证 | 尚无对应实现、测试和原始命令输出 |
| JSON/N-1/回滚 | 未验证 | 尚无 golden fixture |

本文件不把计划路径、规格代码片段或 `cargo` 未执行声明当作证据。晋级时必须追加命令、环境、退出码、测试 ID、commit SHA 和 fixture SHA-256。
