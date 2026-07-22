# adapters/storage/redis — Release

| 项 | 状态 |
|----|------|
| workspace member | 是 · `redisx` |
| 当前版本 | `0.3.4` 未发布候选；`0.3.3` 为 main 历史 |
| publish | `publish = false` |
| crates.io | **未发布** |
| SemVer package stable | **未宣称** |
| 内部可用 | P0 生产入口可用（#188+） |

当前候选曾冻结；治理修正后最终 SHA 待重冻。最终 reviewer/verifier、CI、PR/审批/合并仍 pending，
不得据此创建发布签名或宣称 package stable。

## 发布前清单（若未来 stable）

- 公共 API 冻结说明 + CHANGELOG；
- live 在 CI 可选 job 稳定；
- DEFER 项明确 out-of-scope 或落地；
- `publish = true` 与 Lead 批准。
