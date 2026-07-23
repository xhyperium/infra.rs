# adapters/storage/clickhouse — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `clickhousex` |
| version | `0.3.3` |
| 标题 | ClickHouse Analytics |
| 实现 | `crates/adapters/storage/clickhouse` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 → 三轮加固（负向验收 + 对抗回归） |
| 状态 | **P0 生产入口已落地**（#188–#191）；三轮加固补齐单测锚点与边界回归；package stable **未宣称** |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 ClickHouse Analytics 生产默认客户端，满足 draft P0 DoD，且默认 `cargo test` 离线绿灯。

## Acceptance（本仓可验证）

1. workspace member `clickhousex` 可 `cargo test -p clickhousex --all-targets`
2. 生产默认面：`ClickHousePool / ClickHouseClient HTTP(S)`
3. 环境注入：`FOUNDATIONX_CLICKHOUSEX_{HOST,HTTP_PORT,PORT,USER,PASSWORD,DATABASE}`；
   `HTTP_PORT` 优先，双设不同值 fail-closed（密钥不入库）
4. live：`tests/live_smoke.rs` 默认 `#[ignore]`，真凭据可绿
5. bench：`benches/hot_path.rs（3s 有界）`（不得挂死 `--all-targets`）
6. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认
7. 远程 HTTP、CA/明文冲突与零 deadline 在连接前 fail-closed
8. HTTP 失败只暴露状态/数字错误码，不回显 SQL、payload 或认证正文
9. （三轮加固新增）`insert_json_each_row` / `insert_batch` 的表名与行结构校验
   必须先于网络请求生效，且空 `rows` 短路成功不占用 in-flight 许可
10. （三轮加固新增）`max_in_flight` 背压等待必须在 `acquire_timeout` 后返回
    `DeadlineExceeded`，不得无限阻塞或静默丢弃

## Not in scope

真实 ClickHouse TLS/auth/deadline/并发证据、native 9000 protocol、cluster、
ReplicatedMergeTree 运维面

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/clickhousex-ssot-alignment.md](../../../../../docs/ssot/clickhousex-ssot-alignment.md)
