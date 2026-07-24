# orderbook 十轮重复复审

**日期**：2026-07-23
**对象**：`.cargo/draft/orderbook.md`、`.cargo/draft/orderbook/1.md` 与 `.agents/ssot/orderbook/`
**原则**：PASS 只表示文档层检查通过；runtime 缺失项必须保留 `pending/deferred`。

| 轮次 | 复审焦点 | 结论 | 残差/证据 |
|---|---|---|---|
| R01 | draft 文件覆盖与主题入口 | PASS | 两份 draft 均被 README/evidence 引用；`arch.md` 为空未虚构要求 |
| R02 | v1/v2 冲突裁决 | PASS | v2 为主契约，v1 降为 Binance profile；见 `design.md` D1 |
| R03 | workspace 实现边界 | PASS | 明确无 orderbook crate；只把 domain_market 纯检查标为 verified |
| R04 | 三种同步模型 | PASS | A/B/C 的初始化、连续性与恢复动作互斥且已写入 spec §3 |
| R05 | Binance 对齐规则 | PASS | spot `u<=L`/`U<=L+1<=u`/`U=prev+1`，合约 `<L`/`U<=L<=u`/`pu=prev` 均保留；fixture 仍 pending |
| R06 | OKX/Coinbase provider 边界 | PASS（纠偏后） | OKX 以 `seqId/prevSeqId` 为主且不使用废弃 checksum；Coinbase 不把 envelope `sequence_num` 伪造为严格每簿序列 |
| R07 | Hyperliquid 全量语义 | PASS | 明确 ReplaceAll，禁止 diff 合并残留；schema/fixture 缺失仍 pending |
| R08 | 状态机与异常路径 | PASS | buffer 有界、gap/integrity/crossed/STALE/HALTED 均有动作；实现测试 pending |
| R09 | 精度、时间、排序与输出 | PASS | Decimal、毫秒、缺失时间、排序和 `domain_market` owner 一致；symbol canonical owner 未冻结 |
| R10 | 门禁诚实性与最终残差 | PASS | 无 runtime、fixture、materializer、压测时未关闭对应门禁；见 spec §7 |

## 最终结论

SSOT 已覆盖两份订单簿 draft 的可追溯目标、设计和执行门禁。尚不能声称订单簿模块已实现；下一步必须先选择 Rust 实现落点，再补 Parser/Rule/Verifier、fixture、回放、service profile 和 benchmark。

## 可重复验证记录

- R01–R10：最终文档扫描命令全部返回 `PASS`；R06/R07 专门验证了 OKX checksum 纠偏与 Coinbase sequence 作用域，R08 验证了 Hyperliquid full refresh/reconnect 边界。
- 审查器初版 R03/R07 的正则曾因“主设计”和“不能仅凭字段名假定严格”的措辞产生误报；调整为语义等价匹配后重跑，R01–R10 全部通过。
- `cargo fmt --all --check`：通过。
- `cargo build --workspace`：通过。
- `cargo test --workspace`：通过；当前 workspace 测试全部通过，未覆盖本主题未建立的 runtime 门禁。
- `cargo clippy --workspace --all-features --all-targets -- -D warnings`：通过。
