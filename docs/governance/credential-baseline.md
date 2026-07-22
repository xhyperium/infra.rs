# infra.rs 安全基线 — 凭据复杂度规范

## 概述

所有数据库密码、API 密钥和服务凭据必须满足以下复杂度要求。
本规范适用于 `dev.md` 和 `prod.md` 中的所有凭据，
以及通过 `gh secret set` 上传的 GitHub Actions Secrets。

## 密码复杂度规则（强制）

| 规则 | 要求 | 说明 |
|------|------|------|
| **最小长度** | 24 字符 | 禁止短于 24 字符的密码 |
| **字符集** | 大写 + 小写 + 数字 | 至少包含 A-Z、a-z、0-9 |
| **禁止连续重复** | 同一字符连续出现 <= 3 次 | 防止 `aaaa` 模式 |
| **禁止常见模式** | 不包括 `password`、`admin`、`123` 等 | 防止字典攻击 |
| **API Key 例外** | 32 字符十六进制 | FRED、Qdrant 等外部服务 key 除外 |
| **令牌例外** | 允许 `+` 和 `-` | NATS/JWT 等令牌可含特殊字符 |

## 生成方式

```bash
# 24 位随机密码
python3 -c "import secrets,string; print(''.join(secrets.choice(string.ascii_letters + string.digits) for _ in range(24)))"

# 32 位十六进制 API Key
python3 -c "import secrets; print(secrets.token_hex(16))"
```

## 合规检查

`extract_all_secrets.sh` 在 dry-run 时自动验证所��凭据是否符合��杂度要求。
不符合要求的凭据将标记并列出。

### 快速使用

```bash
# 运行完整审计（范围 + 复杂度 + 强度报告）
scripts/sre/extract_all_secrets.sh dev

# dev + prod 环境
scripts/sre/extract_all_secrets.sh all
```

### 输出示例

```
=== dev.md Scope Check ===
  PASSWORD/TOKEN entries: 57
  ✅ No DATABASE/USER in scope

=== dev.md Complexity Check ===

  -------------------------------------------------
  Strength Report
  -------------------------------------------------
  Total checked:     57
  Pass:              36 (63.2%)
  Fail:              21 (36.8%)
  | By Category
  |-- Length < 24:    21 (36.8%)
  |-- Token/API Key:  0 (0.0%) [exempt]

  Overall rating: WARNING (63.2% pass rate)
```

### 评级阈值

| Rating | Pass Rate | Meaning |
|--------|:--------:|---------|
| HEALTHY | > 75% | Most passwords compliant |
| WARNING | 50-75% | Multiple passwords need rotation |
| IMPROVING | > 0% | Fixes in progress |
| CRITICAL | < 50% | Immediate attention required |

### 检查项

| 检查 | 规则 | 触发条件 |
|------|------|----------|
| 长度 | >= 24 字符 | < 24 chars |
| 大小写 | 至少 1 个大写 + 1 个小写 | 缺失 |
| 数字 | 至少 1 个数字 | 缺失 |
| 重复 | 无连续 4 个相同字符 | `aaaa` pattern |
| 常见模式 | 排除列表 | `password`, `admin123` |

## 密码轮换策略

| 触发条件 | 时限 | 操作 | 记录位置 |
|----------|------|------|----------|
| 认证失败 | 立即 | 生成新密码，更新文档，同步 GitHub Secrets | `dev.md` / `prod.md` password rotation log |
| 复杂度不合规 | 30 天内 | 生成符合 24 字符标准的密码，见下方 tracking table | This document |
| 定期轮换 | 每 90 天 | 生成新密码，更新所有副本 | `dev.md` / `prod.md` password rotation log |
| 泄露事件 | 立即 | 轮换全部受影响服务密码 | This document |
| 新服务 | 创建时 | 使用随机生成器创建密码 | `dev.md` / `prod.md` |

## 轮换跟踪

以下密码在 `extract_all_secrets.sh` 审计中标记为不符��复杂度要求（��于 24 字符）。
每条记录对应一个独立密码及其所有关联数据库。

### dev 环境：15 个不合规密码（影响 15 个数据库 × 2 引擎 = 30 处更新）

| # | Password (masked) | Length | Databases affected | Status |
|---|------------------|:-----:|--------------------|:------:|
| 1 | `Kt6***MkC` | 21 | market_binance (PG + TD) | todo |
| 2 | `h8y***lHW` | 22 | macro_global_cb (PG + TD) | todo |
| 3 | `keC***D2r` | 22 | market_bybit (PG + TD) | todo |
| 4 | `AVM***E3I` | 23 | market_htx (PG + TD) | todo |
| 5 | `5OJ***Vk8` | 23 | macro_treasury (PG + TD) | todo |
| 6 | `PKw***ykx` | 23 | market_hyperliquid (PG + TD) | todo |
| 7 | `s2c***Hey` | 23 | market_kucoin (PG + TD) | todo |
| 8 | `yyy***3AW` | 23 | macro_yahoo (PG + TD) | todo |
| 9 | `O5p***osH` | 23 | macro_japan_cb (PG + TD) | todo |
| 10 | `fQn***Zjm` | 23 | macro_uk_cb (PG + TD) | todo |
| 11 | `w3W***uve` | 23 | macro_yield_curve (PG + TD) | todo |
| 12 | `SqP***xue` | 23 | market_bitget (PG + TD) | todo |
| 13 | `pAS***S1I` | 23 | market_upbit (PG + TD) | todo |
| 14 | `9Nm***jjb` | 23 | market_aster (PG + TD) | todo |
| 15 | `55i***ki2` | 23 | market_lighter (PG + TD) | todo |

## 轮换流程

```bash
# 1. Generate new compliant password (24 chars)
NEW_PASS=$(python3 -c "import secrets,string; print(''.join(secrets.choice(string.ascii_letters+string.digits) for _ in range(24)))")

# 2. Update PostgreSQL
PGPASSWORD="<admin>" psql -h 127.0.0.1 -p 5432 -U postgres -c "ALTER USER <db_user> WITH PASSWORD '$NEW_PASS';"

# 3. Update TDengine (if applicable)
taos -s "ALTER USER <db_user> PASS '$NEW_PASS';"

# 4. Update dev.md password entry

# 5. Sync to GitHub Secrets
gh secret set "FOUNDATIONX_POSTGRESX_<DB>_PASSWORD" --body "$NEW_PASS" --repo xhyperium/infra.rs
gh secret set "FOUNDATIONX_TAOSX_<DB>_PASSWORD" --body "$NEW_PASS" --repo xhyperium/infra.rs

# 6. Log rotation
# Edit dev.md password rotation log section:
# | YYYY-MM-DD | PostgreSQL + TDengine | <db_name> | Complexity non-compliance (N->24 chars) |

# 7. Validate
scripts/sre/extract_all_secrets.sh dev
```

## 轮换审计日志

完整��史记录见各环境文件：

| Environment | Rotation Log Location | Most Recent Entry |
|-------------|----------------------|-------------------|
| dev | `secrets/env/dev.md` > password rotation log | market_okx (2026-07-21, auth failure) |
| prod | `secrets/env/prod.md` > password rotation log | None yet |
