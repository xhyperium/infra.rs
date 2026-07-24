# Round-03 Review：domain_exchange

**角色**：Reviewer（只读）  
**对象**：`crates/domain_exchange` vs `.agents/ssot/domain_exchange/spec/spec.md` v0.2.1  
**工作树**：`/home/workspace/market_data.rs/.worktrees/feat-domain-ssot-implement`  
**日期**：2026-07-22  
**结论**：`ready with follow-ups`

---

## 1. 审查范围

对照 SSOT 门禁与实现，核查：

1. `VenueAdapter` 是否具备 13 个方法且签名语义一致  
2. `AdapterError` 是否包含 `Unsupported`，且有 Display 覆盖测试  
3. `tests/mock_lifecycle.rs` 中 `StatefulMock` 生命周期 / `RestOnlyMock` DE-REST-001  
4. `DE-ERR-001` / `DE-CAP-001` / `DE-PAGE-001` 是否仍为 pending  
5. 测试是否不依赖 live 网络  

**证据来源**：源码通读 + `/tmp/grok-goal-b7a9a210fee0/implementer/domain-test.log`（`cargo test` 域包全绿）。

---

## 2. 门禁对照

| 门禁 ID | SSOT 状态 | 审查结论 | 证据 |
|---|---|---|---|
| **DE-API-001** | verified | **通过** | trait 13 方法与 spec §2 一致；`AdapterError` 8 变体含 `Unsupported`；`de_api_adapter_error_display_all_variants` 覆盖全部 Display |
| **DE-LIFE-001** | verified（mock 级） | **通过（mock 级）** | `StatefulMock`：未连接门禁、connect/disconnect 幂等计数、断开后拒绝、13 方法可达 |
| **DE-REST-001** | verified | **通过** | `RestOnlyMock` 对 WS/交易返回 `Unsupported`，断言前缀 `Unsupported:` 且不匹配 Network/Internal；REST 路径 `get_instruments` / `get_order_book` 允许 |
| **DE-ERR-001** | pending | **仍 pending（正确）** | 无 `retry_after_ms` / scope / HTTP status 结构化字段；`RateLimit(String)` 仍为扁平 String |
| **DE-CAP-001** | pending | **仍 pending（正确）** | trait 无 `exchange_id()` / capabilities API |
| **DE-PAGE-001** | pending | **仍 pending（正确）** | `get_open_orders` / `get_instruments` 仍为单页 `Vec`，无 cursor/window 契约 |

说明：pending 三项未因 mock 通过被错误升格，与 spec §8 说明一致。

---

## 3. 分项审查

### 3.1 VenueAdapter 13 方法（DE-API-001）

源码 `crates/domain_exchange/src/lib.rs` 中 trait 为 `Send + Sync`，方法与 spec 分组一一对应：

| # | 方法 | 结果类型 | 匹配 |
|---|---|---|---|
| 1 | `connect()` | `Result<(), AdapterError>` | ✅ |
| 2 | `disconnect()` | `Result<(), AdapterError>` | ✅ |
| 3 | `subscribe_ticker(&InstrumentKey)` | `Result<(), AdapterError>` | ✅ |
| 4 | `subscribe_order_book(&InstrumentKey)` | `Result<(), AdapterError>` | ✅ |
| 5 | `subscribe_trades(&InstrumentKey)` | `Result<(), AdapterError>` | ✅ |
| 6 | `place_order(&Order)` | `Result<ExecutionReport, AdapterError>` | ✅ |
| 7 | `cancel_order(&OrderId, &InstrumentKey)` | `Result<(), AdapterError>` | ✅ |
| 8 | `amend_order(&OrderAmend)` | `Result<ExecutionReport, AdapterError>` | ✅ |
| 9 | `get_order(&OrderId, &InstrumentKey)` | `Result<Order, AdapterError>` | ✅ |
| 10 | `get_open_orders(&InstrumentKey)` | `Result<Vec<Order>, AdapterError>` | ✅ |
| 11 | `get_account_info()` | `Result<AccountInfo, AdapterError>` | ✅ |
| 12 | `get_instruments()` | `Result<Vec<InstrumentMeta>, AdapterError>` | ✅ |
| 13 | `get_order_book(&InstrumentKey, Option<u32>)` | `Result<OrderBook, AdapterError>` | ✅ |

辅助类型：

- `StreamType`：`Ticker|Level1|Trade|Level2|MiniTicker`，`camelCase` + `#[non_exhaustive]`；集成测试 `de_api_stream_type_serde_camel_case` 验证 `MiniTicker` ↔ `"miniTicker"`。  
- `OrderAmend` 字段与 spec §3.2 一致。  
- `AccountInfo` / `Balance` / `InstrumentMeta` 字段与 spec §3.3 一致（`symbol` 仍为 provider string，未强绑 `InstrumentKey`，与 spec 诚实表述一致）。

### 3.2 AdapterError + Unsupported + Display

源码变体（`#[non_exhaustive]` + `thiserror`）：

1. `InvalidRequest`  
2. `Authentication`  
3. `RateLimit`  
4. `Network`  
5. `WebSocket`  
6. `Parse`  
7. `Internal`  
8. **`Unsupported`** — Display：`"Unsupported: {0}"`；文档明确不得伪装 Network/Internal，并指向 DE-ERR-001  

`tests/mock_lifecycle.rs::de_api_adapter_error_display_all_variants` 对 8 个变体做精确 `to_string()` 断言，满足 DE-API-001 Display 要求。

单元测试 `test_adapter_error_display` 仅覆盖 `InvalidRequest` / `RateLimit`（冗余但无害）；完整覆盖依赖集成测试。

### 3.3 StatefulMock 生命周期（DE-LIFE-001 mock）

| 场景 | 行为 | 测试 |
|---|---|---|
| 未连接订阅/查询/交易 | `InvalidRequest("not connected")` | `de_life_connect_disconnect_idempotent_and_gate` |
| 重复 connect | 幂等成功，计数 +2 | 同上 |
| 连接后 place / get_order_book | 成功 | 同上 |
| 重复 disconnect | 幂等成功，后调用拒绝 | 同上 |
| 13 方法可达 | connect 后逐一调用全部方法 | `de_life_all_thirteen_methods_reachable` |

**边界诚实性**：未覆盖 live 重连恢复订阅、并发压力、Drop 资源释放——spec 已声明这些仍 pending，不因 mock 升格。符合「状态诚实」。

### 3.4 RestOnlyMock（DE-REST-001）

- WS 三订阅 + 交易相关方法返回 `AdapterError::Unsupported(...)`  
- 测试断言：`to_string()` 以 `Unsupported:` 开头，pattern 为 `Unsupported`，不接受 Network/Internal 伪装  
- REST 允许：`get_instruments`、`get_order_book`  

**小缺口（非阻断）**：`de_rest_only_returns_unsupported_not_network` 循环只覆盖 ticker / book_ws / trades / place / cancel / account，**未**显式断言 `amend_order` / `get_order` / `get_open_orders`。实现侧三者已返回 `Unsupported`，建议 follow-up 补入负测循环以防回归。

### 3.5 Pending 门禁未越界

| ID | 代码现状 | 状态标注 |
|---|---|---|
| DE-ERR-001 | 无结构化 retry_after/scope/provider code | pending ✅ |
| DE-CAP-001 | 无 exchange id / capability matrix API | pending ✅ |
| DE-PAGE-001 | 单页 `Vec`，无分页契约类型 | pending ✅ |

### 3.6 无 live 网络

- `Cargo.toml` 依赖：`serde` / `async-trait` / `thiserror` / path `domainx` / `domain_market`  
- dev：`tokio`（rt+macros）、`serde_json`  
- 无 reqwest/hyper/websocket 客户端；mock 仅内存 `AtomicBool`/`AtomicUsize`  
- **不依赖 live 网络** ✅  

### 3.7 测试证据

`domain-test.log`：

```
domain_exchange unittests: 7 passed
mock_lifecycle: 5 passed
  - de_life_connect_disconnect_idempotent_and_gate
  - de_api_stream_type_serde_camel_case
  - de_life_all_thirteen_methods_reachable
  - de_rest_only_returns_unsupported_not_network
  - de_api_adapter_error_display_all_variants
```

---

## 4. 发现列表

### 阻断（blocker）

无。

### 应跟进（follow-up，非阻断）

| ID | 严重度 | 描述 |
|---|---|---|
| F-DE-01 | P2 | REST-only 负测循环漏测 `amend_order` / `get_order` / `get_open_orders`（实现已正确，建议补断言） |
| F-DE-02 | P2 | `crates/exchange/coinglass` skeleton 仍对不适用方法返回 `Internal(...)`，与 DE-REST-001 精神（应用 `Unsupported`）及 spec §6 对 REST-only 的要求不一致。**当前门禁证据仅要求 RestOnlyMock**，故不阻断 domain_exchange 本轮；但后续 adapter 落地时应切换为 `Unsupported`，不得继续 `Internal` 伪装 |
| F-DE-03 | P3 | `lib.rs` 多数 `///` 仍为英文，组织 Rust 规范要求公共 API 文档中文；可在文档轮次统一 |
| F-DE-04 | info | DE-ERR/CAP/PAGE 三项 pending 保持，勿在无 ADR+契约测试时升格 verified |

### 观察（observational）

- 单元内 `MockAdapter` 与集成 `StatefulMock` 职责重叠：前者仅证明 trait 可编译/连接，后者才是门禁证据；可接受。  
- `get_open_orders` / `get_instruments` 的 `Vec` 语义已在 spec 标明「单页 skeleton」，实现未过度承诺，良好。  
- 各 exchange skeleton 统一返回 `Internal`「not implemented」符合 spec §5「入口存在 ≠ 状态机通过」。

---

## 5. 验收清单（本轮）

- [x] VenueAdapter 13 方法与 spec 一致  
- [x] AdapterError 含 Unsupported  
- [x] Display 全变体测试存在且通过  
- [x] StatefulMock 生命周期门禁（mock 级）  
- [x] RestOnlyMock DE-REST-001 负能力  
- [x] DE-ERR-001 / DE-CAP-001 / DE-PAGE-001 仍 pending  
- [x] 无 live 网络依赖  
- [x] `cargo test -p domain_exchange`（含 mock_lifecycle）通过  

---

## 6. Verdict

```text
verdict: ready with follow-ups
```

**理由**：domain_exchange 契约骨架、`Unsupported` 语义、mock 级生命周期与 REST-only 负测均与 SSOT v0.2.1 对齐；已标 verified 的 DE-API-001 / DE-LIFE-001(mock) / DE-REST-001 有可审计测试证据；三项 pending 未越界升格；无 live 网络。  
剩余 F-DE-01~03 为测试覆盖/下游 adapter 语义/文档风格跟进，不阻断本轮 domain_exchange 交付。
