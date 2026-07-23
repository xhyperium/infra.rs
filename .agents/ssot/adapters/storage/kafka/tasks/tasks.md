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
