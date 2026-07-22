# Storage 七域第 3 轮证据包

| 字段 | 值 |
|---|---|
| 日期 | 2026-07-23 |
| 对比基线 | `origin/main` = `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| 实现提交 | `435774f`（`feat(storage): 三轮加固七类存储适配器`） |
| 审查修复候选 | `1729f0a5585dfd634a83e5aa37982ebb744e3afd`（含 Standards/Spec 首轮阻断修复） |
| NATS 监督修复 | `b4ce23ca2db2c11c61f75e783a8f14a80efa409b`（保留并等待 `JoinHandle`，panic fail-closed） |
| 首轮固定测试候选 | `bbcc191f0cce9e1344f9cdbf70808167dd6fc7ea`（仅比 NATS 修复多生成式 `STATUS.md` 刷新） |
| CI 门禁修复 | `50743dc387d78c5bf8be72cef7528218eefa2ca7`（Decimal 误报、文档结构、Markdown/CSpell） |
| 固定行为测试候选 | `50743dc387d78c5bf8be72cef7528218eefa2ca7` |
| 最终合同候选 | `4de762eb7fea50b65d2732f969193c076b1323ee`（仅追加 ClickHouse 标准文档所有权对齐） |
| 配置来源 | `/home/workspace/ZoneCNH/sre/secrets/env/dev.md`；仅由安全 runner 读取 |
| 禁止范围 | 未读取或运行 `prod.md`；未在日志、命令行或仓库写入凭据值 |

## 全局门禁

行为与隔离 conformance 命令均在固定候选 `50743dc387d78c5bf8be72cef7528218eefa2ca7` 上重新执行；
最终合同候选 `4de762eb7fea50b65d2732f969193c076b1323ee` 又执行 Decimal/fence/Markdown/CSpell/质量门禁，退出码均为 0。
原始摘要与输出哈希见同目录的 `storage-round3-raw-gates.md`：

- `cargo fmt --all --check`
- `RUSTC_WRAPPER= cargo clippy --workspace --all-features --all-targets -- -D warnings`
- `RUSTC_WRAPPER= cargo test --workspace --all-features --all-targets`
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps`
- `cargo deny check`（只有既有 unnecessary/unmatched skip 警告）
- `node scripts/quality-gates/check.mjs`（46/46）
- `node scripts/quality-gates/check-crate-versions.mjs`
- `node scripts/quality-gates/check-workspace-deps.mjs`
- `node --test scripts/live/build-foundationx-env.test.mjs`（7/7）
- `node scripts/quality-gates/check-decimal-no-panicking-ops.mjs`（104 文件，0 命中）
- `python3 scripts/standards/check-fences.py`（24/24）
- `npx --yes markdownlint-cli2@0.23.1 ...`（0 issue）
- `npx --yes cspell@10.0.1 ...`（0 issue）

## 七域 live / conformance

所有外部 dev 命令都通过 `scripts/live/export-foundationx-env.sh --env dev -- ...` 启动；
runner 使用 0700 随机目录、0600 独占 env 文件、非 shell 注入和 `trap` 清理。执行后
`/tmp/foundationx-live.*` 残留目录数为 0。

| 域 | 场景 | 结果 | 证据边界 |
|---|---|---|---|
| clickhousex | dev 认证连接 + `live_ping`；隔离 CA/错误 CA conformance | PASS | dev 为 loopback HTTP；真实集群 TLS、mTLS、HA 仍 OPEN |
| kafkax | dev 只读 cluster health；隔离 broker AMO/ALO/重复窗口 3/3；TLS/SASL 2/2 | PASS | 不证明 group、rebalance、自动重连或 native EOS |
| natsx | dev Core roundtrip；隔离 broker 同 client 重启/原订阅/慢消费者连续 3/3 | PASS | `dev.md` 首次认证失败；显式本机 dev NATS 配置后通过。Core 断线窗口仍可能丢消息，Cluster/HA 不在范围 |
| ossx | dev 随机 key put/get/delete，失败前先登记结果、断言前执行 delete | PASS | 不证明 STS、lifecycle、TB 流式对象或 package stable |
| postgresx | dev `SELECT 1` + health；固定 digest 隔离 deadline/cancel/连接隔离 conformance 1/1 | PASS | 自定义 CA、HA、COPY、迁移仍 OPEN |
| redisx | dev ping/stats/close | PASS / PARTIAL | 只证明 standalone；Cluster/Sentinel/TLS/failover/PubSub 重订阅仍 OPEN |
| taosx | dev REST ping；固定 digest 单节点 REST/NCHAR Decimal live 2/2 | PASS | Native SQL、WS 长会话认证、HA、自动幂等重试仍 OPEN |

NATS 的首次失败属于配置漂移证据：`dev.md` 凭据返回 authorization violation，安全 runner
没有回显值；显式读取本机 dev NATS 配置后同一测试退出码为 0。该事实不得改写为 prod 证据。

## 风险登记

| 风险 | 等级 | 当前控制 | 残余状态 |
|---|---|---|---|
| 未验证拓扑/协议被误报为 package stable | P0 | active SSOT、alignment 与 release 文案保留 NO-GO | OPEN，阻止统一 stable 声明 |
| NATS 超出重连预算后 channel 关闭 | P1 | 有限预算、事件统计、3 轮重启实验；调用方重建 client | OPEN，文档化 |
| Kafka group/rebalance/native EOS 缺失 | P1 | AMO/单 owner ALO/非原子重复窗口明确分离 | OPEN |
| OSS 进程崩溃丢失 orphan registry | P1 | 进程内 1024 条上限、取消可发现、显式 abort | OPEN，需 lifecycle/外部审计 |
| Postgres 自定义 CA、HA、COPY 与迁移未覆盖 | P1 | RAII 脱池、结构化双错误；固定镜像 deadline/cancel conformance 1/1 | OPEN，发布声明不得外推 |
| Redis Cluster/Sentinel/TLS 无本轮真实拓扑证据 | P1 | 非 standalone PubSub fail-closed，文档保留 OPEN | OPEN |
| TAOS 存量 DOUBLE schema 精度丢失 | P1 | `DESCRIBE` 后 fail-closed，只接受 NCHAR(64+) | 需受控迁移 |
| dev secret 文档与运行配置漂移 | P1 | 默认不读宿主配置；显式 dev override；不允许 prod | OPEN，需 secrets owner 轮换/同步 |

## 回滚方案

1. PR 合并前：停止并删除分支，无生产部署或数据迁移需要回滚。
2. PR 合并后、发布前：对 squash merge 执行单次 `git revert <merge-sha>`，重新运行
   fmt/clippy/test/doc/deny/quality gates；不得部分回退 active SSOT 或 Cargo 版本。
3. 已部署但未迁移 schema：回退服务二进制与配置，保留新版本 NO-GO 边界。
4. TAOS 已创建 NCHAR schema：不自动降回 DOUBLE；应用回滚与数据 schema 回滚分离，后者走人工审批迁移。
5. OSS multipart：回滚前导出进程内 orphan snapshot 并执行受控 abort；不得直接丢弃补偿记录。

本证据包只证明表中明确场景，不替代 Maintainer 审批、最终 CI 或生产变更授权。
