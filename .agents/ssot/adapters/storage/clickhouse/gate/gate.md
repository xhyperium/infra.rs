# adapters/storage/clickhouse — Gate

## 合并门禁（P0）

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p clickhousex --all-features --all-targets
node scripts/clickhouse-https-conformance.mjs
cmp .agents/ssot/adapters/storage/clickhouse/spec/spec.md \
  .agents/ssot/adapters/storage/clickhouse/spec/xhyper-clickhousex-complete-spec.md
```

## Live 门禁（可选）

```bash
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a
cargo test -p clickhousex -- --ignored
```

## 阻断条件

- 默认路径退化为仅 scaffold
- 硬编码密钥
- live 测试去掉 `#[ignore]` 导致 CI 依赖外网/本机服务
- 无证据宣称 package stable
- 远程 HTTP 可降级、端口别名冲突未拒绝或错误中出现 SQL/payload/认证正文
- 未运行真实集群却把 TLS/auth/deadline/并发 live 标为 PASS
- `insert_json_each_row` / `insert_batch` 的校验逻辑被移到网络请求之后执行
  （即非法输入需要真实网络往返才能被拒绝）
- 背压等待队列在 `acquire_timeout` 到期后未返回 `DeadlineExceeded`
  （无限阻塞或静默丢弃请求）
- `insert_batch` 分块结果与 HTTP 请求次数不一致（合并/拆分 chunk）

## 三轮加固已验证 GO 项（0.3.3，均离线单测，不构成真实集群证据）

- `insert_json_each_row` / `insert_batch`：非法表名、非 object 行拒绝、空 `rows`
  短路成功，且校验先于网络请求 — **GO**
- `query_rows` TabSeparated 解析边界（空行跳过、tab 拆列） — **GO**
- `map_http_error` 全分支覆盖（404/57/60/81/5xx/403/未知 4xx） — **GO**
- `read_error_prefix` 的 4096 字节截断边界 — **GO**
- 背压边界：`max_in_flight=1` 下第二请求超时收到 `DeadlineExceeded` — **GO**
- `insert_batch` 分块 HTTP 层证据（5 行/每块 2 行 = 3 次独立 POST） — **GO**
- scaffold `ClickHouseAdapter` 多事件累加与身份访问器 — **GO**

以上均为对**既有实现路径**补齐的单测锚点或新增离线对抗测试，**不**改变
下方 OPEN/NO-GO 范围。
