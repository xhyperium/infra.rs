# domain_exchange 设计文档

## 当前实现状态

实现位于 `crates/domain_exchange`（package/lib：`domain_exchange`）。当前提供 13 方法 `VenueAdapter`、DTO、结构化错误、能力矩阵、分页默认方法和 mock 契约；live 生命周期、真实 cursor 和 provider 错误映射仍待实现。

## 设计决策

### D1. VenueAdapter 为 async trait

交易所交互本质是 I/O 密集型，使用 `#[async_trait]` 避免手动 future 装箱。同步场景（如 mock 测试）可通过 `block_on` 适配。

### D2. 13 方法均返回 Result

交易所 API 调用的失败是常态（网络、限速、交易所宕机），不提供 infallible 方法。

### D3. 订阅与连接分离（目标）

`connect()` 建立底层传输层连接；`subscribe()` 订阅具体数据流。分离设计允许：
- 连接池复用
- 断线重连时不丢失订阅配置
- 测试时 mock 连接但验证订阅逻辑

### D4. RateLimited 结构化信息

遵循 HTTP 429 / WebSocket 限速最佳实践，让调用方（而非 adapter 内部）决定重试策略。当前 `AdapterError::RateLimit(String)` 仍是兼容错误变体；`RateLimitDetail`/`RateLimitDetailed` 已提供结构化 retry-after、scope、provider code、HTTP status 和 request id，adapter 映射仍待实现。

## crate 布局

```
crates/domain_exchange/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── venue_adapter.rs      # trait 定义
│   ├── types.rs              # StreamType, OrderAmend, AccountInfo, InstrumentMeta
│   ├── error.rs              # AdapterError
│   └── mock.rs               # MockVenueAdapter（测试用）
└── tests/
    └── adapter_contract.rs   # trait 契约测试
```
