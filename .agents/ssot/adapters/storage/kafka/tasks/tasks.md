# adapters/storage/kafka — Tasks

## P0（已关闭 · #188–#191）

- [x] 生产默认客户端与配置
- [x] unit / lib 测试
- [x] live `#[ignore]` 测试：`tests/live_event_bus.rs`
- [x] bench：`benches/hot_path.rs`
- [x] crate docs：usage / config / operations
- [x] SSOT landing + draft 快照
- [x] docs/ssot/kafkax-ssot-alignment.md

## 2026-07-23 draft 十轮闭合

- [x] 十轮条款矩阵 + R1–R10 收敛
- [x] SSOT matrix/goal/spec/evidence/review 同步 NO-GO/OOS
- [x] `KafkaConfigBuilder` + `KafkaMessage.timestamp` + 公共 API 行为测试
- [x] 真 secrets live 3/3 + broker conformance 3/3
- [x] 对齐文档 / gap-matrix / version `0.3.4`

## 2026-07-23 生产测试矩阵

- [x] 离线 prod_offline（功能/安全 fail-closed）
- [x] 集成 prod_reliability + kafka-prod-matrix.mjs（含 --fault-restart）
- [x] bench 多 payload + p50/p95/p99
- [x] docs/测试矩阵-生产发布.md 清单对照
- [x] 可选有界 soak（非 24h 门禁）

## OPEN / DEFER（不混入本战役）

- [ ] group coordinator / rebalance / 自动重连（需驱动能力变更）
- [ ] native EOS / transactional producer
- [ ] SCRAM / OAuth / mTLS **成功路径**（fail-closed 已测）
- [ ] package stable 证据包（若 Lead 启动）
- [ ] Part2 量化栈（**OOS**）
- [ ] 7×24 默认 soak 门禁（可选 `KAFKAX_SOAK_SECONDS`）

## 2026-07-23 gap 清零（0.3.7）

- [x] headers 公共面 / PublishRecord / publish_with_key / partition_for_key
- [x] KafkaPoolStats 细项（timeouts/cancelled/topics_*）
- [x] selfcheck ordering_headers 同源 shipped 路径
- [x] 全 pub API offline + e2e 分层测试
- [x] NO-GO 项 CLOSED（Skipped / fail-closed / 文档）
- [x] SSOT version 对齐 0.3.7

## 2026-07-23 G-STATS-01 严格（0.3.9）

- [x] shipped `limited_produce_await` / `apply_limited_produce_outcome`
- [x] 单测严格 timeout/cancel 计数（禁止仅 record_*）
- [x] 集成 strict cancelled|timeouts（禁止 OR published|failed）
- [x] SSOT / adapters-ssot / matrix S-20 对齐 0.3.9
