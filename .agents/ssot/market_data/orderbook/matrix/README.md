# orderbook 追溯与门禁矩阵

| draft 要求 | SSOT 入口 | 当前实现证据 | 状态 |
|---|---|---|---|
| 通用内核/三种模型 | `spec/spec.md` §3 | 无 orderbook crate | specified |
| Binance 12 簿 profile | `goal/goal.md` §2、`spec/spec.md` §3.1 | 仅 draft；adapter 为 skeleton | pending |
| OKX `seqId/prevSeqId` 与废弃 checksum 纠偏 | `spec/spec.md` §3.2、OB-OK-002 | 无 sequence parser/fixture | pending |
| Coinbase level2 delivery/sequence scope | `spec/spec.md` §3.2、OB-CB-001 | 无 level2 parser/fixture | pending |
| Hyperliquid 整簿替换 | `spec/spec.md` §3.3 | 无 orderbook runtime | pending |
| 公共排序/update-id 检查 | `domain_market/spec/spec.md` §2.3 | `crates/domain_market/src/book.rs` | verified |
| 统一快照与 service outputs | `spec/spec.md` §2.2、§6 | 无 materializer/infra crate | deferred |

完整门禁 ID 见 [`../spec/spec.md` §7](../spec/spec.md#7-可执行门禁)；跨主题矩阵见 [`../traceability-matrix.md`](../../traceability-matrix.md)。
