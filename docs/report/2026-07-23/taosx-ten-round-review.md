# taosx 十轮审查与最终缺口矩阵

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| Draft | `.cargo/draft/taosx_SPEC_GOAL.md` |
| SSOT | `.agents/ssot/adapters/storage/taos/`（**非** `taosx/`） |
| 实现 | `crates/adapters/storage/taos` → package `taosx` |
| 审查范围 | 本仓 adapter 生产默认面 + 当前 crate-root 公开 API |
| 结论 | **可生产默认使用（受限 REST）**；package stable / HA / Native SQL **NO-GO** |

> 100% 锚定：本仓 adapter 生产默认路径 + 公开 API 测试点名 + 十轮矩阵中可闭合 GAP。  
> **不**宣称 draft 后半量化交易全底座、24h soak、crates.io 发布。

---

## 是否补齐 `.agents/ssot/adapters/storage/taosx`？

**否。** 仓库 SSOT 权威路径为 `.agents/ssot/adapters/storage/taos/`。  
另建 `taosx/` 会造成双轨漂移；package 名 `taosx` 与目录 `taos/` 的映射已在 alignment / goal / landing 中固定。

---

## 第 1 轮 — 生产默认面与公开导出

**视角**：crate-root 导出是否构成可交付生产入口。

| 发现 | 判定 |
|------|------|
| 默认 `TaosPool`/`TaosClient` REST `:6041`，非 HashMap scaffold | PASS（draft §2.1 过时） |
| `feature = "scaffold"` 才暴露内存 `TaosAdapter` | PASS |
| 公开：`TaosConfig`/`TaosPool`/`build_insert_sql_chunks`/`HARD_MAX_*`/`TransportMode`/`connect_native_ws` | PASS |
| Draft §2.3 `TaosConfigBuilder`/`TaosQueryStream`/`BatchWriteReport` 未实现 | OUT-OF-SCOPE / 非本仓默认面 |
| alignment 文档 version 曾写 `0.3.2`，Cargo 为 `0.3.3` | GAP → 本 PR 闭合 |

**结论**：生产默认面已落地；draft 幻想 API 不全实现属诚实边界。

---

## 第 2 轮 — 配置与安全 fail-closed

**视角**：`FOUNDATIONX_TAOSX_*`、TLS/auth、host 注入、密钥脱敏。

| 发现 | 判定 |
|------|------|
| `from_env` + `validate`：空白/非法 host、零 timeout、超 `HARD_MAX_*` fail-fast | PASS |
| 远程明文与无密码 TLS 在 connect 前拒绝 | PASS（conformance + unit） |
| host 禁止 scheme/userinfo/path；REST redirect 禁止 | PASS |
| `Debug` 密码脱敏 | PASS |
| secrets 自 `/home/workspace/ZoneCNH/sre/secrets/env/dev.md` 经 `export-foundationx-env.sh` 注入，**不入库** | PASS |
| draft 的 runtime/DDL role 分离 | NO-GO（运维约定，非 crate 职责） |

**结论**：安全默认面达标；密钥路径合规。

---

## 第 3 轮 — TimeSeriesStore 写查合同

**视角**：`contracts::TimeSeriesStore` 纳秒 epoch、批量写、范围查。

| 发现 | 判定 |
|------|------|
| `write_series` → `write_batch`；STABLE + symbol 子表 | PASS |
| `query_series` 有界 collect；超 `max_query_rows` 拒绝 | PASS |
| 可移植 suite `assert_time_series_store` 在 live 通过 | PASS |
| 精度探测 + 配置不一致 fail-closed | PASS |
| 参数绑定官方 API / 流式 `TaosQueryStream` | NO-GO / 后续里程碑 |
| 部分失败 `BatchWriteReport` | NO-GO（整批错误；幂等重试 NO-GO） |

**结论**：合同写查生产路径闭合；流式与批报告为 NO-GO。

---

## 第 4 轮 — 资源硬上限与 close drain

**视角**：OOM/背压/关闭语义。

| 发现 | 判定 |
|------|------|
| `HARD_MAX_{IN_FLIGHT,BATCH_*,RESPONSE,QUERY,CLOSE}` | PASS |
| Semaphore `max_in_flight` + acquire → `DeadlineExceeded` | PASS |
| close 置位拒绝新请求；RAII in-flight drain；可重复 close | PASS |
| 超大 Content-Length 不整缓冲 | PASS |
| 无界 channel / 无限子表自动创建 | PASS（有界；不自动无限 cardinality 治理） |
| draft 每 tenant 子表速率配额指标 | NO-GO |

**结论**：资源与关闭硬边界有测试证据。

---

## 第 5 轮 — Decimal / NCHAR schema

**视角**：价格无损与 schema gate。

| 发现 | 判定 |
|------|------|
| bid/ask `NCHAR(64+)` 文本；scale=18 大 mantissa live 相等 | PASS |
| DESCRIBE 拒绝 DOUBLE 存量 schema | PASS |
| 不自动迁移 DOUBLE→NCHAR | PASS（诚实；运维迁移） |
| 全超表治理（orderbook JSON 等） | OUT-OF-SCOPE |

**结论**：Decimal 门禁达标。

---

## 第 6 轮 — Transport / WS 边界

**视角**：REST vs NativeWs 能力矩阵诚实性。

| 发现 | 判定 |
|------|------|
| SQL 始终 REST | PASS（文档/代码一致） |
| `NativeWs` 仅握手/关闭探测 | PARTIAL / 诚实边界 |
| Native 6030 SQL / FFI | NO-GO |
| TMQ / schemaless feature | NO-GO / OUT-OF-SCOPE |
| 本地 6041/6030 可达；live REST 用 6041 | PASS |

**结论**：不得宣称 WS SQL 长会话等价 native。

---

## 第 7 轮 — 测试矩阵与 live

**视角**：离线全绿 + 真实配置 live。

| 发现 | 判定 |
|------|------|
| `cargo test -p taosx --all-targets` 离线全绿 | PASS |
| `live_smoke` ignored 默认；export 注入后 2/2 ok | PASS（本会话证据） |
| 隔离 Docker runner `scripts/taos-live-conformance.mjs` 有归档 | PASS |
| 24h soak / 节点滚动 / leader 迁移 | NO-GO |
| 错误分类基于 code 而非字符串（缺表 Missing） | PASS |

**结论**：live 与离线门禁满足 P0；故障矩阵未闭合。

---

## 第 8 轮 — Bench 与可观测

**视角**：有界基准与 metrics 面。

| 发现 | 判定 |
|------|------|
| `benches/hot_path.rs` 3s 有界 connect + ping 循环 | PASS |
| 无服务时 skip，不挂死 | PASS |
| draft pool/write/query RED 指标全量 | NO-GO（tracing debug 有限） |
| readiness/liveness 文档在 operations.md | PASS |

**结论**：有界 bench 达标；完整 metrics 面 NO-GO。

---

## 第 9 轮 — SSOT / docs 对齐

**视角**：goal/spec/matrix/landing/alignment/crate docs。

| 发现 | 判定 |
|------|------|
| SSOT 11 层 + landing 已存在于 `taos/` | PASS |
| **不需要** `adapters/storage/taosx/` 平行树 | PASS |
| crate `docs/{README,config,usage,operations}` | PASS（本 PR 补强） |
| alignment version 漂移 0.3.2 vs 0.3.3 | GAP → 闭合 |
| draft 量化全仓 workspace 结构 | OUT-OF-SCOPE |

**结论**：SSOT 路径正确；对齐文档版本需同步。

---

## 第 10 轮 — 发布 / 稳定 / DoD

**视角**：生产级发布 vs package stable。

| 发现 | 判定 |
|------|------|
| `publish = false`；未 crates.io | PASS（诚实） |
| 生产默认路径可验证 + live + bench + 门禁 | PASS（本 PR） |
| package stable / SBOM 全量 / 24h soak DoD | NO-GO / OPEN |
| CI fmt/clippy/test 通过后 PR→merge | 交付中 |

**结论**：可宣称 **受限 REST 生产默认就绪**；**不可**宣称 package stable。

---

## 最终缺口矩阵

| ID | Draft / 条款 | 状态 | 证据 | 闭合动作 | 补 SSOT？ |
|----|--------------|------|------|----------|-----------|
| D-01 | §2.1 仅 scaffold | PASS（draft 过时） | `src/client.rs` REST | 文档标注陈旧 | 否 |
| D-02 | 默认 REST 生产入口 | PASS | `TaosPool::connect` | — | 否 |
| D-03 | TimeSeries 写+查 | PASS | live_smoke + suite | — | 否 |
| D-04 | 资源硬上限/close | PASS | conformance + unit | — | 否 |
| D-05 | TLS/auth fail-closed | PASS | config tests | — | 否 |
| D-06 | Decimal NCHAR | PASS | live scale=18 | — | 否 |
| D-07 | 公开 API 测试点名 | PASS* | `public_api_surface` 本 PR 扩面 | 扩表面测试 | 否 |
| D-08 | live 真实配置 | PASS | export-foundationx + live_smoke | 证据归档 | 证据层 |
| D-09 | 有界 bench | PASS | hot_path | — | 否 |
| D-10 | crate docs 运维 | PASS* | docs/* 本 PR 对齐 | 补 usage 细节 | 否 |
| D-11 | alignment 版本 | GAP→PASS | 对齐 `0.3.4` | 改 alignment/matrix | 是（版本行） |
| D-12 | SSOT 路径 taosx vs taos | PASS | 用 `taos/` | 禁止双轨 | 否 |
| D-13 | Native SQL/FFI | NO-GO | 无实现 | 保留 NO-GO | 否 |
| D-14 | WS SQL 长会话 | NO-GO | 仅握手 | 保留 | 否 |
| D-15 | 幂等自动重试 | NO-GO | 多 chunk 无报告 | 保留 | 否 |
| D-16 | HA/Cluster/failover | NO-GO | 无矩阵 | 保留 | 否 |
| D-17 | package stable | OPEN/NO-GO | publish=false | 禁止宣称 | 否 |
| D-18 | TMQ/schemaless | OUT-OF-SCOPE | — | 产品后续 | 否 |
| D-19 | 量化 Order/回测底座 | OUT-OF-SCOPE | draft 后半 | 产品仓 | 否 |
| D-20 | 24h soak | NO-GO | 无证据 | 保留 | 否 |
| D-21 | TaosConfigBuilder/Stream | OUT-OF-SCOPE | 非默认面 | follow-up | 否 |
| D-22 | 完整 RED metrics | NO-GO | 有限 tracing | follow-up | 否 |

\* 本 PR 闭合项。

---

## 本 PR 建议改动清单

1. 扩 `public_api_surface`：常量、`TaosConfig` URL/`from_env`、池同步面方法  
2. 对齐 `docs/ssot/taosx-ssot-alignment.md` 与 matrix/goal version → 交付版本  
3. crate docs 补 live 真实配置命令（export-foundationx，不写密钥）  
4. SSOT evidence 增加 2026-07-23 真实 dev live 脱敏摘要  
5. PATCH bump + CHANGELOG  
6. 提交 / PR / CI / 合并 / 清理  

## Agent team 记录

- Lead：规划 + 审查矩阵 + 交付  
- 分析子代理：十轮审查并行（本文件为 SSOT 结论）  
- Executor：实现/测试/文档在 worktree `feat/taosx-spec-goal-close`  
- 实现者 ≠ 人类 Maintainer 批准（合并前需 CI + 规则集审批）
