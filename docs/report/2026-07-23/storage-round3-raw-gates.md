# Storage 七域第 3 轮原始门禁记录

## 候选与采集约束

- 固定行为候选：`50743dc387d78c5bf8be72cef7528218eefa2ca7`
- 最终合同候选：`4de762eb7fea50b65d2732f969193c076b1323ee`；相对行为候选仅对齐 ClickHouse 标准文档，19–22 与质量门禁在该内容上复验。
- 基线：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`
- 日期：2026-07-23
- 日志采集：每条命令独立保存 stdout/stderr，记录字节数与 SHA-256；遇到非零立即停止。
- 安全边界：本轮只使用隔离容器/临时证书，不读取 `dev.md` 或 `prod.md`，日志不含凭据值。
- 本文件是候选测试后的证据提交，只增加证据，不改变被测 Rust、脚本、配置或 SSOT 行为。
- 完整日志归档：`storage-round3-raw-gates.tar.gz`，25345 字节，SHA-256
  `734d0c58e2afc440aed1bf70897b72fe6d2e7718299e80cabee71a33971fadf1`；归档内含下表 22 个原始日志。

独立复验：

```bash
sha256sum docs/report/2026-07-23/storage-round3-raw-gates.tar.gz
verify_dir=$(mktemp -d /tmp/storage-round3-verify.XXXXXX)
tar -xzf docs/report/2026-07-23/storage-round3-raw-gates.tar.gz -C "$verify_dir"
sha256sum "$verify_dir"/*.log
```

## 完整日志清单

| 序号 | 命令 | 退出码 | 字节 | SHA-256 |
|---|---|---:|---:|---|
| 01 | `cargo fmt --all --check` | 0 | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| 02 | `RUSTC_WRAPPER= cargo clippy --workspace --all-features --all-targets -- -D warnings` | 0 | 193 | `65ce14a080d4d5f06889952a24c4df318dc8028448716c538507cb0219ace401` |
| 03 | `RUSTC_WRAPPER= cargo test --workspace --all-features --all-targets` | 0 | 86980 | `350dc3b06e7ba036c9e3a4ca84f3f1e153f96545362df6ef5aa55da870a7be9d` |
| 04 | `RUSTC_WRAPPER= RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps` | 0 | 946 | `a674c82657da538d13f005cffcc7ddd936288d4b35ff8fb560deb9ff5395a911` |
| 05 | `cargo deny check` | 0 | 4405 | `9a5d8d8299c0887932b47b0d9f19d845f5869a894a479d1f661b65b15cb0638e` |
| 06 | `node scripts/quality-gates/check.mjs` | 0 | 1793 | `6f17fe741d1dbe91bc5451e7d44c9c19d926b39a70731327337d53ec34f5f766` |
| 07 | `node scripts/quality-gates/check-crate-versions.mjs` | 0 | 195 | `f435712052660547a04cd8a78315ad7e4ecfb7b113521d6c909829907cd7e295` |
| 08 | `node scripts/quality-gates/check-workspace-deps.mjs` | 0 | 218 | `8040aa1800473ab014cf3cdad7cbeca79ec6921b85d962775cefdd058fa19586` |
| 09 | `node --test scripts/live/build-foundationx-env.test.mjs` | 0 | 662 | `68b71d73b14f76c9d38e342fc45b84ba0e48658f49bbcab2072b02197eb5dd35` |
| 10 | `node scripts/clickhouse-https-conformance.mjs` | 0 | 1440 | `ef90ea09c310102781bf0e20b20c23c59a1bf7ff1388ad01958b7fab571e6a12` |
| 11 | `node scripts/kafka-broker-conformance.mjs` | 0 | 1857 | `4875fd17ce7fddf89048b9ab44cd2045e8531371650a0326d8d89fba8e28ce52` |
| 12 | `node scripts/kafka-tls-sasl-conformance.mjs` | 0 | 3027 | `120771da6c344982b78ccf1ac73c6c51f771109e4641564ee1be33a84ea76025` |
| 13 | `node scripts/broker-conformance.mjs` | 0 | 3621 | `bebcc2141d4e40acac619bf04436dfc86f5180e2eee62a6e085241f8a38debfb` |
| 14 | `node scripts/nats-reconnect-conformance.mjs` | 0 | 2341 | `73608ef1df287ed40a28f96257a882f8fa4f5518a71e2dc0d3de088fb5bd3ad3` |
| 15 | `node scripts/postgres-deadline-conformance.mjs` | 0 | 995 | `ce7fcbadee76b9a1278d651250d83476d73927687802e49f3be1357dfba3fdd5` |
| 16 | `node scripts/taos-live-conformance.mjs` | 0 | 493 | `1cb63127b1b4d890215370f53a1391238b2780d014e7d936a0591067ccef1427` |
| 17 | `node scripts/storage-composition-conformance.mjs` | 0 | 1333 | `3f47046b40b4822b7774af3f2f5066703a67fcbd18df6eb05a9b3e0f20155b80` |
| 18 | `git diff --check origin/main...HEAD` | 0 | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| 19 | `node scripts/quality-gates/check-decimal-no-panicking-ops.mjs` | 0 | 59 | `210adca481d684b051b2c9266820c01e51356eb12fe7dddbf12c9cf4b9f65ee6` |
| 20 | `python3 scripts/standards/check-fences.py` | 0 | 1133 | `1a8d744a4fc64468e05f372c8c9687fd1a89a3978d15db7f5b925bdb8e35daa2` |
| 21 | `npx --yes markdownlint-cli2@0.23.1 ...` | 0 | 178 | `b5e3ab6b5c732e81032b57a1749d01248fb841b4b74fe41aed8ae33ec44510cf` |
| 22 | `npx --yes cspell@10.0.1 ...` | 0 | 56 | `988ce6071107c495963b8e4705de3f7fac061796108d64da39145c8ec2d8affc` |

## 原始 stdout/stderr 尾部

以下内容按日志原样摘录；空输出门禁已由零字节哈希记录。

```text
02-clippy.log
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.63s

03-test.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
bench_verifyctl_plan: iters=200 total=15.842859ms last_digest_prefix=ba836026a736
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

04-doc.log
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.14s
   Generated /home/workspace/infra.rs/.worktrees/feat/infra-2d9.3-storage-3x/.cargo/target/doc/binancex/index.html and 23 other files

05-deny.log
advisories ok, bans ok, licenses ok, sources ok

06-quality.log
PASS: docs status matrix 新鲜
PASS: STATUS.md crates 看板新鲜
结果: PASS

07-versions.log
scanned Cargo.toml under crates/: 22
OK: 全部 crate 使用独立显式版本，path 依赖 version 对齐
PASS

08-workspace-deps.log
scanned Cargo.toml (crates/ + tools/): 24
OK: 全部第三方依赖经 { workspace = true } 引用，path 依赖版本对齐
PASS

09-live-runner.log
tests 7
pass 7
fail 0
skipped 0

10-clickhouse-tls.log
test trusted_ca_succeeds_and_wrong_ca_fails_closed ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
ClickHouse HTTPS/CA conformance 已通过
临时 TLS 证书已清理（result=passed）

11-kafka-broker.log
test result: ok. 3 passed; 0 failed; 0 ignored
Kafka broker AMO/ALO/重复窗口 conformance 已通过
清理 Kafka 容器

12-kafka-tls-sasl.log
test trusted_ca_and_plain_credentials_publish_to_real_broker ... ok
test wrong_ca_and_wrong_password_fail_closed ... ok
test result: ok. 2 passed; 0 failed; 0 ignored
Kafka TLS+SASL/PLAIN conformance 已通过
清理容器与临时证书

13-nats-broker.log
test result: ok. 6 passed; 0 failed; 0 ignored
Kafka/NATS broker conformance 已通过
清理容器

14-nats-reconnect.log
NATS 重连 conformance 轮次：3/3
test reconnect_restores_subscription_and_slow_consumer_is_observable ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
NATS 重连与慢消费者 conformance 已连续通过 3 轮
清理容器

15-postgres-deadline.log
test pool_and_query_deadlines_fail_closed_then_recover ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
PostgreSQL 截止时间与连接隔离 conformance 已通过
PostgreSQL 容器已清理（result=passed）

16-taos-live.log
test live_ping ... ok
test live_write_query_ticks ... ok
test result: ok. 2 passed; 0 failed; 0 ignored
taos REST/Decimal live conformance 通过

17-storage-composition.log
test real_storage_contracts_are_callable_through_bootstrap ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
bootstrap 正式 storage contracts E2E 已通过
清理容器

19-decimal-gate.log
decimal panicking-ops gate: OK (104 files scanned, 0 hits)

20-fences.log
结果：24 文件，24 OK，0 BROKEN

21-markdownlint.log
Linting: 410 files
Summary: 0 issues in 0 files

22-cspell.log
CSpell: Files checked: 409, Issues found: 0 in 0 files.
```

完整日志的 SHA-256 与字节数是复验标识；本文件只摘录不含随机端点、容器名和临时路径的尾部，避免把短生命周期运行细节误当稳定接口。
