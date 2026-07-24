# domainx Round-01 审查报告

**审查对象**：`crates/domainx`（worktree `feat-domain-ssot-implement`）  
**对照 SSOT**：`.agents/ssot/domainx/spec/spec.md` v0.2.1  
**角色**：Reviewer（只读）  
**日期**：2026-07-22  

## 结论

**ready with follow-ups**

本轮声明范围（公共类型契约、DX-VAL-001..005 纯校验、DX-API-002 fixture、门禁表诚实性、真实 API 测试）均已满足，证据可复现。无阻塞缺陷；下列 follow-ups 不阻碍合入本轮 domainx 实现。

---

## 证据列表

### 1. 公共类型与 spec 一致

| 检查项 | 结果 | 证据 |
|--------|------|------|
| ID / 时间别名 | ✅ | `OrderId`/`TradeId`/`ReportId`/`PositionId`/`PortfolioId` = `String`；`Timestamp = i64`（`lib.rs` L22–32） |
| `Decimal` re-export | ✅ | `pub use rust_decimal::Decimal`（L6） |
| 枚举 `#[non_exhaustive]` + `Debug+Clone+PartialEq+Eq+Hash+Serialize+Deserialize` | ✅ | `OrderSide`/`OrderType`/`OrderStatus`/`PositionDirection`/`PositionStatus`/`ExecType`/`TimeInForce`（L39–122） |
| 枚举 `#[serde(rename_all = "camelCase")]` | ✅ | 上述除 `TimeInForce` 外均有 |
| `TimeInForce` adjacently tagged | ✅ | `#[serde(tag = "type", content = "value")]`，变体 `Gtc`/`Ioc`/`Fok`/`Gtd(Timestamp)`（L75–87） |
| 结构体字段名/类型/Option 语义 | ✅ | `Commission`/`Order`/`Position`/`Trade`/`ExecutionReport`/`Portfolio` 与 spec §2.3 字段表一致；`Position` 无 `status`（符合 DX-POS-001 缺口说明） |
| 结构体 camelCase | ✅ | 各 struct 均 `#[serde(rename_all = "camelCase")]` |

### 2. DX-VAL-001..005 纯校验

| ID | 函数 | 失败类型 | 单元测试 |
|----|------|----------|----------|
| DX-VAL-001 | `validate_non_negative_quantities` | `ValidationError::Quantity` | `val001_rejects_negative_quantity` |
| DX-VAL-002 | `validate_quantity_balance` | `Quantity` | `val002_rejects_unbalanced_fill` / `val002_accepts_balanced` |
| DX-VAL-003 | `validate_order_prices`（Market/Limit/StopMarket/StopLimit 矩阵） | `Price` | `val003_*` 四组 |
| DX-VAL-004 | `validate_created_before_updated` | `Time` | `val004_rejects_created_after_updated` |
| DX-VAL-005 | `validate_gtd_deadline` | `Time` | `val005_rejects_gtd_before_created`（含 equal / Gtc 通过） |
| 组合 | `validate_order` 串联 001–005 | — | `validate_order_accepts_valid_limit` / `validate_order_rejects_bad_gtd` |

- 模块：`crates/domainx/src/validate.rs`  
- 公共导出：`lib.rs` L9–13  
- 无 I/O / 网络 / 下单副作用；纯 `Result<(), ValidationError>`  
- `ValidationError` 亦为 `#[non_exhaustive]` + `thiserror`

### 3. DX-API-002 fixtures

| Fixture | 路径 | 集成测试 |
|---------|------|----------|
| Limit 订单 + camelCase + validate | `tests/fixtures/order_limit.json` | `fixture_order_limit_round_trip_and_validate` |
| `TimeInForce::Gtd` adjacently tagged | `tests/fixtures/time_in_force_gtd.json` | `fixture_time_in_force_gtd_adjacently_tagged` |
| Decimal 尾随零 / 负数 / 大数 | `tests/fixtures/trade_decimal_edge.json` | `fixture_trade_decimal_trailing_zeros_negative_large` |
| ExecutionReport camelCase | `tests/fixtures/execution_report.json` | `fixture_execution_report_camel_case` |
| 枚举 wire 名 camelCase | 内联 `serde_json::to_value` | `enum_variants_use_camel_case_wire_names`（`buy`/`stopMarket`/`partiallyFilled`/`tradeCancel` 等） |

fixture 内容与 wire 契约对齐示例：
- `order_limit.json`：`orderId`、`timeInForce: { "type": "Gtc" }`、`quantity: "1.5000"`
- `time_in_force_gtd.json`：`{ "type": "Gtd", "value": 1700000001000 }`

### 4. 门禁表诚实性

spec §6 当前状态（摘录）：

| ID | 状态 | 审查判定 |
|----|------|----------|
| DX-API-001 | verified | 合理：`cargo test -p domainx` + 类型编译 |
| DX-API-002 | verified | 合理：fixtures + `serde_fixtures.rs` |
| DX-VAL-001（含 001–005） | verified | 合理：`validate.rs` 单测 |
| **DX-CAN-001** | **blocked** | ✅ 未静默升为 verified（instrument 仍为 `String`） |
| **DX-COMP-001** | **pending** | ✅ 未静默升为 verified（`Position` 无 status；`Portfolio.total_commission` 仍为单一 `Decimal`） |

goal.md 仍将 DX-CAN-001 / DX-COMP-001 列为未完成 checkbox，与门禁表一致。

### 5. 测试调用真实已交付 API（无 theater）

- 集成测试通过 `domainx::{Order, TimeInForce, Trade, ExecutionReport, validate_order, …}` 反序列化真实 fixture、再序列化 round-trip、并对 `Order` 调用 `validate_order`。
- 单测直接调用 `validate_*` 纯函数，断言 `ValidationError` 变体。
- 日志证据：`/tmp/grok-goal-b7a9a210fee0/implementer/domain-test.log`  
  - domainx lib：19 passed  
  - `tests/serde_fixtures.rs`：5 passed  
- clippy：`domainx` Checking 通过（`clippy.log` L93），无 warnings 阻断。

未发现「硬编码 expect true / 不调用公共符号」的假测试。

---

## 问题

**无 P0/P1 阻塞问题。**

以下为非阻塞观察（不构成 not ready）：

1. **文档漂移（低）**  
   - `Order.instrument` 注释写 “until domain_market is available”，spec 写的是待 `xhyper-canonical` 迁移。  
   - `TimeInForce::Gtd` 注释 “cut-off 00:00:00 UTC” 非 spec 语义，易误导。

2. **spec 内部措辞不一致（低）**  
   - §2.2：`Gtd`「必须晚于创建时间」  
   - DX-VAL-005：「不早于创建时间」  
   - 实现采用 VAL 表（允许 `deadline == created_at`），与单测 `equal ok` 一致。建议后续统一 spec 措辞。

3. **VAL-001 单测分支不全（低）**  
   - 仅覆盖 `quantity < 0`；`filled_quantity` / `remaining_quantity` 为负的路径有实现、无独立断言。不削弱门禁，但覆盖可补。

4. **`test_enum_non_exhaustive_match` 证据力弱（低）**  
   - 在定义 crate 内穷尽 match 不能证明 `#[non_exhaustive]` 对下游的约束；属性本身源码可见且派生齐全，功能上无缺口。

---

## Follow-ups

| ID | 项 | 建议优先级 |
|----|----|------------|
| FU-DX-01 | 修正 `instrument` / `Gtd` 误导注释，对齐 spec（canonical / 毫秒截止） | P2 |
| FU-DX-02 | 统一 spec §2.2 与 DX-VAL-005 关于 Gtd「晚于 vs 不早于」的表述 | P2 |
| FU-DX-03 | 补 VAL-001：`filled_quantity`/`remaining_quantity` 负值用例 | P2 |
| FU-DX-04 | DX-CAN-001：instrument `String` → 唯一 canonical owner（保持 **blocked** 直至迁移 PR） | 既有 blocked |
| FU-DX-05 | DX-COMP-001：`Position.status` + 多资产手续费表达（保持 **pending**） | 既有 pending |

---

## 审查清单回执

| # | 检查点 | 结果 |
|---|--------|------|
| 1 | 公共类型匹配 spec（non_exhaustive / TimeInForce adjacently tagged / camelCase） | PASS |
| 2 | DX-VAL-001..005 纯校验在 `validate.rs` | PASS |
| 3 | DX-API-002 fixtures 在 `crates/domainx/tests/` | PASS |
| 4 | Gate：DX-CAN-001 blocked、DX-COMP-001 pending，未伪 verified | PASS |
| 5 | 测试调用真实 shipped API，无 hardcoded theater | PASS |

**Verdict: ready with follow-ups**
