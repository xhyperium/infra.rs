# orderbook 目标

## 1. 使命

提供交易所无关的订单簿构建与维护内核，把“每家交易所一套状态机”收敛为“一套参数化内核 + 每家一个薄适配器”，向下游输出同构的快照、质量指标和恢复状态。

## 2. 首期覆盖

| provider | 市场 | 同步模型 | 关键凭证 |
|---|---|---|---|
| Binance | spot / UM / CM | 外部快照 + diff | spot `U/u`；合约 `pu` |
| OKX | spot / swap / futures | 流内快照 + diff | `seqId/prevSeqId`；当前官方文档将 checksum 标为废弃 |
| Coinbase | Advanced Trade spot | 流内快照 + diff | `level2` 保证更新交付；`sequence_num` 仅在 raw fixture 证明可用时作连续性凭证 |
| Hyperliquid | perp / spot | 全量快照流 | 无增量连续性；每条消息整簿替换 |

Binance v1 草稿中的 12 个状态簿（spot、UM、CM 各 4 个）是 service profile，不限制通用内核容量；具体 symbol 和运行规模由部署配置决定。

## 3. 目标

### 3.1 抽象质量

- **OB-A1**：内核不得依赖任何 provider 专有包、名称或 wire 字段。
- **OB-A2**：新增 provider 只实现 Parser、ContinuityRule、必要的 IntegrityVerifier/SnapshotSource 及契约测试，不修改内核状态机。
- **OB-A3**：四家输出使用同一 `BookSnapshot`/`BookMetrics` 语义，下游不按 provider 分支。
- **OB-A4**：crossed、新鲜度、状态和 resync 指标在所有同步模型中语义一致。

### 3.2 正确性

- **OB-C1**：每种协议的断裂、重复、过期、乱序和恢复边界均可检测；不得静默产出脏簿。没有 provider 连续性凭证时必须使用连接/订阅恢复边界，不能伪造严格序列。
- **OB-C2**：Binance spot 使用 `U/u` 区间与连续规则；UM/CM 使用 `pu` 链规则。
- **OB-C3**：OKX 必须按 `seqId/prevSeqId` 检测连续性；当前官方文档明确 checksum 已废弃且固定为 0，不得把它作为 CRC32 门禁。若未来频道重新启用完整性字段，必须先更新 evidence/spec 再实现 verifier。
- **OB-C4**：Hyperliquid 全量消息必须替换整簿，不能合并后残留已消失档位。
- **OB-C5**：所有已物化簿检查 bid/ask 排序、负数量、crossed 和事件时间滞后。

### 3.3 性能与可用性目标

目标值沿用 draft，最终以压测环境和实现语言重新基准化：diff 应用 P99 < 1ms，全量替换 P99 < 2ms；service profile 的 Redis 镜像 P99 < 50ms；resync 风暴必须按簿隔离并受并发预算控制。

## 4. 非目标

- 不负责 provider WebSocket 连接、认证、订阅编排和重连调度；宿主/adapter 负责。
- 不负责撮合模拟、成交推演或跨交易所聚合订单簿。
- 不把期权无状态完整深度强行送入 Binance spot/futures 状态机。
- 不把 Kafka、Redis、ClickHouse、NATS 作为当前 Rust workspace 已存在的能力；它们是 service profile 的待实现输出。

## 5. 验收定义

- 参数化内核的状态转移矩阵和三种同步模型均有纯测试。
- 四个 adapter 的契约 fixture 覆盖首帧、正常更新、重复、断裂、恢复和错误输入；Coinbase 必须先证明 `sequence_num` 的作用域。
- OKX sequence gap/reset、Hyperliquid 缺档替换、Binance snapshot 对齐均有故障注入测试；废弃 checksum 字段不得被误用。
- lib 形态与 service 形态复用同一内核；输出 schema、状态和指标一致。
- 真实流回放、压测、72 小时运行和基础设施连通性只能在实现完成后关闭对应门禁，不能以当前骨架编译通过替代。

## 6. 当前未完成

- [ ] 选择 Rust crate/service 落点并实现内核。
- [ ] 将各 adapter 的 wire DTO 转成统一 `BookEvent`。
- [ ] 建立 provider 原始 fixture、回放器、契约测试和外部证据快照。
- [ ] 确定 `BookSnapshot` 与 `domain_market::OrderBook` 的边界及 symbol canonicalization owner。
- [ ] 实现 Kafka/Redis/ClickHouse/NATS/Prometheus service profile。
