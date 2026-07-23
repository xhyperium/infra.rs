# PLAN-CONTRACTS-MAINT-003

1. 固化 active spec、Additive Only 与 NO-GO。
2. 先写公共 live helper/validation 红灯，再最小实现。
3. 同步版本、直接消费者、README/API/CHANGELOG/release/alignment。
4. 检查 contracts、contract-testkit 与全部生产消费者。

若兼容性 ratchet 显示 removal/signature change，则发布 BLOCKED；不以 bump 消解 breaking。
