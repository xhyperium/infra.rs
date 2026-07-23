## clickhousex PR

> 使用本模板：创建 PR 时 URL 加 `?template=clickhousex.md`
>
> 版本：`0.3.6` · dep：`contracts 0.1.4` · `xhyper-kernel 0.3.1`

## 类型

- [ ] feat — 新功能 / 新 API
- [ ] fix — 缺陷修复
- [ ] test — 测试补充
- [ ] refactor — 重构（无功能变化）
- [ ] docs — 文档 / CHANGELOG / README / 示例
- [ ] chore — 构建 / CI / 依赖 / 版本

## 影响范围

- [ ] `src/client.rs` — ClickHousePool / Client / AnalyticsSink / validate_ident
- [ ] `src/config.rs` — ClickHouseConfig / from_env / validate
- [ ] `src/adapter.rs` — scaffold `ClickHouseAdapter`（`feature = "scaffold"`）
- [ ] `src/lib.rs` — 公开导出 / doc tests
- [ ] `tests/schema_integrity.rs` — DDL 约束 / Config::validate / validate_ident / 端口别名
- [ ] `tests/security_failures.rs` — HTTP 失败路径脱敏验证
- [ ] `tests/https_conformance.rs` — TLS CA 实验（`#[ignore]`）
- [ ] `tests/live_smoke.rs` — 真实 ClickHouse 烟测（`#[ignore]`）
- [ ] `benches/hot_path.rs` — SELECT 1 基准测试
- [ ] `Cargo.toml` — 依赖 / 版本 / feature / workspace
- [ ] `.github/` — CI/CD / 模板
- [ ] 其他（请说明）

## 变更摘要

<!-- 简要说明做了什么、为什么这样做 -->

## 测试验证

### 质量门禁

- [ ] `cargo fmt -p clickhousex -- --check` 通过
- [ ] `cargo clippy -p clickhousex --all-targets --all-features -- -D warnings` 通过
- [ ] `cargo test -p clickhousex` 通过（32 lib + 2 scaffold）
- [ ] `cargo test -p clickhousex --features scaffold --test schema_integrity` 通过
- [ ] `cargo test -p clickhousex --test schema_integrity -- --test-threads=1` 通过（29 tests）
- [ ] `cargo test -p clickhousex --doc` 通过（5 doc tests）
- [ ] `node scripts/quality-gates/check-workspace-deps.mjs` 通过
- [ ] `node scripts/fix-encoding.mjs --check crates/adapters/storage/clickhouse/` 通过
- [ ] `cargo bench -p clickhousex --bench hot_path` 编译通过

### 测试矩阵

<!-- 列出受影响的测试文件及其结果 -->

| 测试文件 | 测试数 | 类型 | 结果 |
|----------|--------|------|------|
| `src/lib.rs` + `src/client.rs` + `src/config.rs` + `src/adapter.rs` | 32 | lib 单元（3 scaffold） | ... / 32 |
| `tests/schema_integrity.rs` | 29 | DDL / config / ident / port | ... / 29 |
| `tests/security_failures.rs` | 3 | HTTP 脱敏 | ... / 3 |
| `tests/https_conformance.rs` | 1 | TLS CA（`#[ignore]`） | ... / 1 |
| `tests/live_smoke.rs` | 2 | 真实 CH 烟测（`#[ignore]`） | ... / 2 |
| **总计** | **67** | **64 active + 3 ignored** | |

### 安全合同

<!-- 如改动涉及 HTTP 请求路径、错误脱敏、配置校验、标识符处理，确认以下不变式 -->

- [ ] 错误响应不含 SQL / payload / 认证正文（4096 字节截断有效）
- [ ] 密码 / 完整 URL 不进入 Debug / 错误上下文
- [ ] 远程明文 HTTP 在 `validate()` 阶段 fail-closed
- [ ] `validate_ident` 拒绝 SQL 注入模式（`;`, `--`, `0` 数字前缀）
- [ ] 校验在所有网络请求**之前**执行（插入路径）
- [ ] 空 `rows` 短路成功，不占用 in-flight 许可
- [ ] 背压等待 `acquire_timeout` 后返回 `DeadlineExceeded`

### Schema 约束（schema_integrity.rs 覆盖）

- [ ] 7 表均使用 `MergeTree` 引擎
- [ ] klines_* 表按月分区（`PARTITION BY toYYYYMM`）
- [ ] 排序键为 `(symbol, open_time)`
- [ ] 12 列名称与类型正确（`DESCRIBE TABLE` 验证）
- [ ] 6 张 klines 表列数一致
- [ ] 排序列数据非空 / OHLCV 非负
- [ ] `Config::validate` 10 项全部失败路径可用
- [ ] `validate_ident` 8 项（7 reject + 4 accept）
- [ ] 端口别名 4 种组合（冲突 / 一致 / HTTP_PORT / PORT）

## 依赖变更

<!-- 如 `Cargo.toml` 有依赖变更，在此说明 -->

- [ ] 无依赖变更
- [ ] 新增依赖：___
- [ ] 移除依赖：___
- [ ] 版本升级：___ → ___

## 破坏性变更

<!-- 如有破坏性变更，说明影响范围和迁移步骤；无则勾选 -->

- [ ] 无破坏性变更

## 审查聚焦

<!-- 指出需要 reviewer 特别关注的部分 -->
