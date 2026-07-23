# redisx 行覆盖率残余说明（2026-07-23）

## 测量

| 命令 | Lines Cover |
|------|-------------|
| `cargo llvm-cov -p redisx --lib --summary-only`（0.3.6 基线离线） | **67.41%** |
| `cargo llvm-cov -p redisx --lib --summary-only`（0.3.7 补测后离线） | **71.35%**（`error_map` **99.22%**） |
| `cargo llvm-cov -p redisx --lib --tests --features pubsub -- --include-ignored`（+ 真实 Redis live） | **79.49%** |

证据文件（scratch 会话）：`redisx-coverage.txt` / `redisx-coverage-with-live.txt`。

## 未达 100% 的原因（禁止刷线假绿）

| 残余类别 | 估算影响 | 处置 / OPEN 链接 |
|----------|----------|------------------|
| Cluster 成功路径 + MOVED/ASK 运行时 | 中 | REDISX-10：无真实 Cluster 拓扑，仅 refused 负向测 |
| Sentinel 发现成功 + failover 再发现 | 中 | REDISX-11：无 Sentinel live |
| TLS 握手成功路径 | 中 | REDISX-12：secure 构造测过，无 TLS live |
| 罕见 redis 错误文案分支（error_map） | 低 | 持续归零；已覆盖主 ErrorKind 映射 |
| Pub/Sub 断连/重订阅分支 | 中 | REDISX-15 NO-GO：不宣称必达 |
| config 解析冷路径 / 边界组合 | 低 | 已有大量负向单测；残余组合 OPEN |
| scaffold feature 代码 | N/A | 默认图不编入；不计入生产面 100% 目标 |

## 诚实结论

- **不可**宣称「行覆盖率 100%」。
- **可**宣称：Standalone P0 生产面在离线 + Standalone live 下有实质覆盖；残余与 Cluster/Sentinel/TLS/PubSub 可靠性 OPEN/NO-GO 一致。
- 刷 100% 的禁止项：对 Cluster 空壳 mock 刷绿、硬编码断言、不驱动真实 entrypoint。

## 关联

- [gap-matrix-v0.md](./gap-matrix-v0.md)
- [docs/ssot/redisx-ssot-alignment.md](../../../../../../docs/ssot/redisx-ssot-alignment.md)
