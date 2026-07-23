# MATRIX-CONTRACTS-MAINT-003

| Requirement | Public seam | Test evidence | Condition |
|---|---|---|---|
| CT-LIVE-1 profile 非证明 | `LiveHandles::validate` | repo/account/venue_time 单旗标 | 全部 fail-closed |
| CT-LIVE-2 已有句柄 | `validate` | kv/bus/tx/venue 缺失 | 缺失时报 Missing |
| CT-BUS-1 无 E2E 假承诺 | publish helper | publish 成功/失败传播 | 不 subscribe/ack |
| CT-TX-1 无原子性假承诺 | KV+Tx helper | set/commit/rollback 失败 | 只描述真实顺序 |
| CT-API-1 Additive Only | 15 traits/API ratchet | consumer check/tests | 无 removal/signature change |

全 trait conformance、交易业务 live、跨 backend 原子性与 E2E delivery 固定 OPEN。
