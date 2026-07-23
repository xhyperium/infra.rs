# taosx 残余对齐修复（#284 后）

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| 原因 | #284 合入后 `workspace-ssot-alignment.md` members 表仍写 taosx `0.3.2` |
| 修复 | 同步为 `0.3.4`；`draft-gap-matrix.md` 补十轮/live/NO-GO 措辞 |

## 验证

- 与 `crates/adapters/storage/taos/Cargo.toml` version `0.3.4` 一致
- 与 `docs/ssot/taosx-ssot-alignment.md` / adapters 表一致
- **不**新建 `.agents/ssot/adapters/storage/taosx/`
