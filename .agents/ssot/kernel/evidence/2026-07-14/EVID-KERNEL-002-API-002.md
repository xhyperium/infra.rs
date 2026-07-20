# EVID-KERNEL-002-API-002 — KERNEL-API-002 机控

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Rule | **KERNEL-API-002** |
| Residual | RES-GATE-009 **CLOSED (implemented)** |

## 机制

1. **Baseline**：`.architecture/api/kernel-public-api.baseline.txt`（= kernel 0.1.1 冻结面）
2. **Current**：`.architecture/api/kernel-public-api.txt`
3. **Allowlist**：`.architecture/api/kernel-api-rfc.toml` 的 `[[allow]]` 块
4. **判定**：current \ baseline 的每一行须匹配 allow.pattern，且 `rfc` 解析到 Status=Approved 的文档

当前 baseline ≡ snapshot → **0 additions** → PASS。

## 验证

```bash
cargo test -p archgate
cargo run -p archgate -- --json   # KERNEL-API-002 ok
```

## 未来增补流程

1. 改代码 + `cargo public-api -p kernel --simplified > .architecture/api/kernel-public-api.txt`
2. 在 `kernel-api-rfc.toml` 登记 `[[allow]]` + Approved RFC 路径
3. PR 审阅；必要时更新 baseline（整批冻结）
