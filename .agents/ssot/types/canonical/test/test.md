# types/canonical — Test

| 字段 | 值 |
|---|---|
| 状态 | **current-state 测试合同** |
| 更新 | 2026-07-23 |
| 本次 writer | 仅更新文档；未执行测试 |

## 必须覆盖

| 面 | 断言 |
|---|---|
| Wire inventory | v1/v1.1/v1.2/v1.3 共 12 个类型；`committed_wire_version` 返回精确 `WireVersion` |
| 兼容查询 | 所有 committed 类型的 `wire_commitment` 仍返回 `CommittedV1`；未知类型与 `Money` 返回 `Uncommitted` |
| Strict serde | 双向 round-trip；未知字段/variant、缺字段与非法 decimal scale 均拒绝 |
| Golden / N-1 | 文件或穷举 inline golden 与序列化结果一致；有登记的 legacy/N-1 历史向量可读 |
| 枚举 | `OrderStatus` 六个 variants、`Side` 两个 variants、`OrderRef` 两个 variants 的 JSON 表示固定 |
| 时间 | ms→ns checked overflow；`unix_millis_from_ns` 与兼容 alias 向 0 截断；`unix_millis_from_ns_exact` 仅接受整毫秒纳秒值 |
| Envelope | shape round-trip、缺/未知字段拒绝、显式版本验证成功/失败；反序列化不自动路由 |
| 分层 | `Money` 与 `decimalx::Money` 同一；无上层反向依赖或 domain 行为 |

N-1 fixture 证明的是已登记历史 JSON 向量仍可读取，不等于通用 migration reader、跨语言协议或跨大版本兼容。

```bash
cargo test -p canonical -p decimalx
cargo check -p canonical --all-targets
cargo clippy -p canonical --all-targets -- -D warnings
cargo fmt -p canonical -- --check
node scripts/quality-gates/check-canonical-align.mjs
```
