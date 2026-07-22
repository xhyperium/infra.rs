# R2 正确性

- gap: OrderAck.ts 用 ms → **fixed** ns_from_unix_millis
- gap: OKX now_iso 纯毫秒整数 → **fixed** seconds.millis
- gap: okx query 子串猜状态 → **fixed** code/data 信封 + state 映射
- tests: signed_place_cancel_query_* 内容断言 PASS
