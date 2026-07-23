# PLAN-TRANSPORT-MAINT-003

1. 锁定 active spec 与测试 seam。
2. 逐项执行 red → green：HTTP、WS、Debug/TLS、Pool、Retry-After。
3. 同步版本、直接消费者、README/API/CHANGELOG/release/alignment。
4. 执行 scoped 门禁并把退出码写入 evidence/review/release。

回滚：任何兼容性或安全语义无法满足时，不进入 release PASS；保留未提交 diff 交 root 审查。
