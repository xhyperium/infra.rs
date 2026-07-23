# 2026-07-23 — taosx 真实 dev live 证据（脱敏）

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| 命令 | `scripts/live/export-foundationx-env.sh --env dev --secrets-dir …/ZoneCNH/sre/secrets/env -- cargo test -p taosx --test live_smoke -- --ignored --nocapture` |
| 环境 | 本机 loopback TDengine REST :6041；凭据来自 dev.md 解析；**未**写入仓库 |
| package | `taosx`（交付版本见 Cargo.toml） |

## 结果

```text
running 2 tests
test live_ping ... ok
test live_write_query_ticks ... ok
test result: ok. 2 passed; 0 failed; 0 ignored
```

覆盖：

- `SELECT SERVER_VERSION()` ping
- `CREATE DATABASE` / STABLE / 批量写 / 范围查
- `assert_time_series_store` 可移植合同
- scale=18 大 mantissa Decimal 往返相等
- `close()` drain

## 脱敏声明

- 本文件 **不含** 密码、token 或完整 DSN
- 原始带密钥的进程环境仅存在于 export 脚本临时文件，命令结束即删除
- 未使用 prod 主机作为本轮必过路径

## 相关

- 隔离 Docker live：`evidence/2026-07-23-infra-2d9.3.7-live.md`
- 十轮审查：`docs/report/2026-07-23/taosx-ten-round-review.md`
