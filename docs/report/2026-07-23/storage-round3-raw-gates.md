# Storage 七域第 3 轮原始门禁记录

## 候选与采集约束

- 固定候选：`bbcc191f0cce9e1344f9cdbf70808167dd6fc7ea`
- 基线：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`
- 日期：2026-07-23
- 日志采集：每条命令独立保存 stdout/stderr，记录字节数与 SHA-256；遇到非零立即停止。
- 安全边界：本轮只使用隔离容器/临时证书，不读取 `dev.md` 或 `prod.md`，日志不含凭据值。
- 本文件是候选测试后的证据提交，只增加证据，不改变被测 Rust、脚本、配置或 SSOT 行为。

## 完整日志清单

| 序号 | 命令 | 退出码 | 字节 | SHA-256 |
|---|---|---:|---:|---|
| 01 | `cargo fmt --all --check` | 0 | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| 02 | `RUSTC_WRAPPER= cargo clippy --workspace --all-features --all-targets -- -D warnings` | 0 | 72 | `4e857e8ebea43159b6f1acbd4b11448ae36294d66a6d373b8d0b78f089a3bf01` |
| 03 | `RUSTC_WRAPPER= cargo test --workspace --all-features --all-targets` | 0 | 86854 | `61d5ce6f1b6844077c305f39ae852cfa8372d4e12f87a1d8e600c2a48e3fb157` |
| 04 | `RUSTC_WRAPPER= RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps` | 0 | 206 | `82f35a00a0811f588d4cf86adc0c9d0395dcaa4917f4ee9ed83944c71020cf79` |
| 05 | `cargo deny check` | 0 | 4405 | `9a5d8d8299c0887932b47b0d9f19d845f5869a894a479d1f661b65b15cb0638e` |
| 06 | `node scripts/quality-gates/check.mjs` | 0 | 1793 | `6f17fe741d1dbe91bc5451e7d44c9c19d926b39a70731327337d53ec34f5f766` |
| 07 | `node scripts/quality-gates/check-crate-versions.mjs` | 0 | 195 | `f435712052660547a04cd8a78315ad7e4ecfb7b113521d6c909829907cd7e295` |
| 08 | `node scripts/quality-gates/check-workspace-deps.mjs` | 0 | 218 | `8040aa1800473ab014cf3cdad7cbeca79ec6921b85d962775cefdd058fa19586` |
| 09 | `node --test scripts/live/build-foundationx-env.test.mjs` | 0 | 661 | `7743b08fd56feb6006c439052af43de5eee2524ea876ee44d8ab68eb6dd0d6f5` |
| 10 | `node scripts/clickhouse-https-conformance.mjs` | 0 | 1440 | `b02b61f7e36d0e5ba51aa5107352c65f9123b825da3ba970d72d22f874728f0d` |
| 11 | `node scripts/kafka-broker-conformance.mjs` | 0 | 1852 | `f8e1d05228d58170afa269f65dfa2243b486c37d1d0fe2d76fac3365649898a3` |
| 12 | `node scripts/kafka-tls-sasl-conformance.mjs` | 0 | 3147 | `48e368258e33d4a5e4e30649b6151653a5093df687dd689afe35726a46477ba7` |
| 13 | `node scripts/broker-conformance.mjs` | 0 | 3603 | `b55f98d3fbfbc707c31dd1081daa6808c36260afc4e85c2e02425fe3673d3738` |
| 14 | `node scripts/nats-reconnect-conformance.mjs` | 0 | 2337 | `5ef63dd905b1aeeff90d7c49e39fd63b2bded7cabfb38bbf9e13699d5254afc1` |
| 15 | `node scripts/postgres-deadline-conformance.mjs` | 0 | 992 | `51b6daab2b314f28cbbfbd6c24ea44b89b607b1ecfd06e044cdf524d3fba7189` |
| 16 | `node scripts/taos-live-conformance.mjs` | 0 | 493 | `05146cfdbd47181d60db0705a1d2097d5921f134e3428a5bbf747b4fcadd763b` |
| 17 | `node scripts/storage-composition-conformance.mjs` | 0 | 2208 | `eb066847cc0624e148203fe66115dfd29609409f9d584b0298cfc702499040fb` |
| 18 | `git diff --check origin/main...HEAD` | 0 | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## 原始 stdout/stderr 尾部

以下内容按日志原样摘录；空输出门禁已由零字节哈希记录。

```text
02-clippy.log
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.15s

03-test.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
bench_verifyctl_plan: iters=200 total=13.303783ms last_digest_prefix=ba836026a736
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

04-doc.log
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.70s
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
```

完整日志的 SHA-256 与字节数是复验标识；本文件只摘录不含随机端点、容器名和临时路径的尾部，避免把短生命周期运行细节误当稳定接口。
