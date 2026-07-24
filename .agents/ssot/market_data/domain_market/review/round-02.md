# Round-02 Review: domain_market

**角色**：Reviewer（只读）  
**范围**：`crates/domain_market` vs `.agents/ssot/domain_market/spec/spec.md`（v0.2.1）  
**工作区**：`/home/workspace/market_data.rs/.worktrees/feat-domain-ssot-implement`  
**日期**：2026-07-22  
**证据**：`implementer/domain-test.log`（domain_market 19 unit + 7 integration 全过）

---

## Verdict

**ready with follow-ups**

已实现类型字段、DM-BOOK 纯检查、DM-TIME 门禁、DM-SER fixture round-trip 与 spec 对齐；`DM-ENV-001` / `DM-CAN-001` 保持 non-verified，无伪 verified。遗留均为文档/契约已知限制或可后续补强，不阻断本轮 domain_market 落地。

---

## 1. Types：Tick / Quote / Bar / OrderBook / envelope 字段

| 类型 | Spec §2 | 源码 | 结论 |
|------|---------|------|------|
| `InstrumentKey { exchange, symbol }` | §2.1 | `lib.rs` 28–34 | 一致 |
| `ProductLine` 四变体 + `#[non_exhaustive]` | §2.1 | 41–49 | 一致 |
| `Tick`：instrument/price/quantity/side/trade_id/timestamp/received_at | §2.2 | 68–83 | 一致 |
| `Quote`：bid/ask 价量 + levels + 双时间戳 | §2.2 | 104–123 | 一致 |
| `PriceLevel { price, quantity, order_count }` | §2.2 | 92–99 | 一致；`order_count: Option<u64>` 合理 |
| `Bar`：interval/open/close 时间 + OHLCV + optional 扩展 | §2.2 | 145–172 | 一致 |
| `BarInterval` untagged 整数变体 | §2.2 注明 | 132–140 | 一致（见 follow-up） |
| `OrderBook`：bids/asks/sequence/first&last update id/timestamp/update_type | §2.2 | 190–208 | 一致；无 `received_at` 与 spec 一致 |
| `TickDirection` / `OrderBookUpdateType` non_exhaustive | §2.2 | 56–63 / 179–185 | 一致 |
| `MarketFactEnvelope`：instrument/source/fact_type/data/timestamp | §2.4 基线 | 237–248 | 一致；仍为 `String + Value` 骨架 |
| `DataSource` 13 个 provider | §2.4 | 218–232 | 一致 |
| 聚合：OI / Funding / Liquidation / LSR | §2.5 | 278–349 | 字段与语义注释一致；LSR 注释仍为 percentage |

**Decimal / Timestamp**：价格数量走 `domainx::Decimal`（`rust_decimal` re-export）；时间为 `Timestamp = i64` Unix ms。与 §3 一致。

**[AF] 文档漂移（非契约）**：`InstrumentKey` 注释仍写 “SSOT lives in `xhyper-canonical`”，与 spec §1 / `DM-CAN-001 blocked` 的诚实表述冲突。spec 已声明旧文档不一致；应改为「当前 workspace 唯一 instrument 类型；canonical 迁移见 DM-CAN-001」。

---

## 2. book.rs：DM-BOOK 排序 + update ids

| 门禁 | 实现 | 测试证据 | 结论 |
|------|------|----------|------|
| DM-BOOK-002 bid 降序 / ask 升序 | `bids_are_descending` / `asks_are_ascending`；相邻 `>=` / `<=`（允许同价） | `book002_*` unit | 通过 |
| DM-BOOK-001/003 update id 区间 | `validate_update_ids`：两侧皆有时要求 `first <= last` | `book001_rejects_inverted_update_ids` | 通过 |
| 缺失 ID 不得假设连续 | `deltas_are_contiguous` → `None` | `book001_contiguous_requires_both_ids` | 通过 |
| 组合入口 | `validate_order_book` | fixture + unit | 通过 |
| Snapshot fixture 排序与 id | `order_book_snapshot.json` + `dm_book_snapshot_fixture_*` | integration | 通过 |

**边界诚实**：

- Provider 跳号恢复 / checksum **未**在本 crate 实现，与 spec「verified（纯检查）」+ adapter 归属一致。
- `validate_update_type_shape` 当前对 Snapshot/Delta 恒 `Ok`（穷尽匹配脚手架），**不**构成 Snapshot/Delta 语义差分检查 → follow-up，不否决 DM-BOOK 纯检查主路径。
- 无 Delta JSON fixture；连续性仅有 unit 覆盖。spec 证据写 “snapshot fixture”，可接受。

---

## 3. time.rs：DM-TIME event vs received + bar bounds

| 规则 | 实现 | 测试 | 结论 |
|------|------|------|------|
| 毫秒启发式 | `looks_like_unix_millis`（约 2001–2100 ms 窗） | unit + integration 拒绝秒级 | 通过 |
| `received_at >= timestamp` | `validate_event_vs_received` | 接受 after；拒绝 before | 通过 |
| Tick / Quote 门禁 | `validate_tick_time` / `validate_quote_time` | fixture 路径调用 | 通过 |
| Bar `open_time <= close_time` + ms | `validate_bar_bounds` / `validate_bar_time` | unit + `bar.json` | 通过 |

**说明**：

- 严格拒绝 `received_at < timestamp` 与模块注释（强制 ingestion 注入 wall clock）一致；允许相等。
- OrderBook 无 `received_at`，时间门禁不对其施加 event/received，与类型契约一致。
- 未实现「缺失 timestamp 保留 Option」——类型为必填 `i64`；spec 对缺失语义偏 adapter 层，本轮不判 fail。

---

## 4. tests/serde_and_time.rs + fixtures：DM-SER-001

| 检查项 | 证据 | 结论 |
|--------|------|------|
| camelCase wire | `receivedAt`/`tradeId`/`bidPrice`/`openTime`/`firstUpdateId`/`factType`/`updateType`；断言无 snake_case | 通过 |
| Decimal 无精度损失 | fixture 字符串价量；`123456789012345.123456789` 与 scale=4 trailing zeros | 通过 |
| Round-trip | tick/quote/bar/order_book/envelope 均 `to_value`/`from_value` 相等 | 通过 |
| 与时间/簿门禁串联 | fixture 反序列化后调用 `validate_*` | 通过 |

Fixture 清单：

- `fixtures/tick.json`
- `fixtures/quote.json`
- `fixtures/bar.json`
- `fixtures/order_book_snapshot.json`
- `fixtures/envelope.json`

`BarInterval` untagged：`interval: 5` 会落入首个整数变体 `Seconds(5)`；测试与 spec 均已明文承认，未伪造「单位可辨」契约。

---

## 5. Honesty：DM-ENV-001 / DM-CAN-001 保持 non-verified

| ID | Spec 状态 | 实现现状 | 诚实性 |
|----|-----------|----------|--------|
| DM-ENV-001 | **pending** | `MarketFactEnvelope` 仍为 `fact_type: String` + `data: Value`；无 typed fact / sequence / MarketSubject | 未伪称 verified |
| DM-CAN-001 | **blocked** | `InstrumentKey`/`ProductLine` 仍属本 crate；无 workspace canonical 迁移 | 未伪称 verified |
| DM-API-001 | verified | 字段与源码一致 | 支持 |
| DM-TIME-001 | verified | `time.rs` + fixtures | 支持 |
| DM-BOOK-001 | verified（纯检查） | `book.rs` + snapshot fixture | 支持（范围限定正确） |
| DM-SER-001 | verified | fixtures + `serde_and_time.rs` | 支持 |

未发现将 pending/blocked 标成 verified 的迹象。envelope fixture 仅覆盖兼容骨架，符合 §2.4。

---

## Follow-ups（不阻断）

1. **[AF]** 修正 `InstrumentKey` 文档注释，去掉 “SSOT in xhyper-canonical” 误导，与 DM-CAN-001 对齐。  
2. **[follow-up]** `BarInterval` untagged 无单位判别；后续若要 wire 可逆，需 tagged 或字符串 interval（spec 已声明未实现）。  
3. **[follow-up]** 充实 `validate_update_type_shape` 或删除/标注为 placeholder，避免调用方误以为有 Snapshot/Delta 差分校验。  
4. **[follow-up]** 可选增加 `order_book_delta.json` fixture，固化 `deltas_are_contiguous` 的 wire 路径（当前 unit 已够纯检查）。  
5. **[follow-up]** `OrderBook` 上冗余的 `#[serde(rename = "updateType")]`（`rename_all = "camelCase"` 已足够）可清理。

---

## 评分维度（简表）

| 维度 | 结论 |
|------|------|
| 需求合规（对照 spec） | 通过 |
| 门禁证据 | 通过（domain-test.log） |
| 诚实标注 | 通过 |
| 代码质量 / 可维护性 | 通过，带小 follow-up |
| Scope 控制 | 通过：未越界做 adapter 恢复/typed envelope/canonical 迁移 |

---

## 最终结论

**ready with follow-ups**

domain_market 本轮对照 spec 的 verified 项（API / TIME / BOOK 纯检查 / SER）已落地且有 fixture/test 证据；pending（typed envelope）与 blocked（canonical owner）保持 non-verified，诚实。建议实现侧优先改 `[AF]` 文档注释，其余记入 backlog 即可。
