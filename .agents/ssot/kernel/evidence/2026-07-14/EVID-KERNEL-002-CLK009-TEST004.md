# EVID-KERNEL-002 — RES-CLK-009 + RES-TEST-004 收口

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Branch | `feat/kernel-002-e2-migrate-banned-apis` |
| PR | [#235](https://github.com/xhyperium/xhyper.rs/pull/235) |
| §18 | **仍 OPEN** |

## 变更

### RES-CLK-009 → CLOSED

- 删除 `MonotonicInstant::from_std`（无调用方）。
- `gate` / `binance` / `okx` 测试 `FixedClock::monotonic` 改为  
  `from_clock_elapsed(Duration::from_millis(1))`，不再 `Instant::now`。
- archgate `KERNEL-TIME-002` allowlist 收窄为仅 `crates/kernel/`。

### RES-TEST-004 → CLOSED

- `timestamp_min_max_and_u64_edges` 单元测试
- proptest `timestamp_near_i64_bounds`（MIN/MAX/0/任意 + 小 Duration）

## 命令

```bash
cargo test -p kernel --test clock_contract   # 9 passed
cargo test -p gate -p binance -p okx --lib   # pass
cargo run -p archgate -- --json              # 13 KERNEL-* ok
cargo clippy -p kernel -p archgate -p gate -p binance -p okx --all-targets -- -D warnings
```

## 仍 OPEN

RES-API-007 · RES-TEST-005/014/015/016 · RES-DOWN-006 · §18 / Spec Proposed  
miri：stable 工具链无 miri 组件（本环境未装）。
