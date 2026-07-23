# taosx 独立 Review（相对 origin/main）

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| 审查者角色 | **独立 Reviewer**（非本变更实现者） |
| 分支 / 工作区 | `feat/taosx-spec-goal-close` · `/home/workspace/infra.rs/.worktrees/feat/taosx-spec-goal-close` |
| 对照基线 | `origin/main` |
| 审查范围 | `crates/adapters/storage/taos/**` · `docs/ssot/taosx-ssot-alignment.md` · `docs/report/2026-07-23/taosx-ten-round-review.md` · `.agents/ssot/adapters/storage/taos/**` |
| 审查性质 | 只读审查 + 本报告落盘；**未**改生产代码 |

## Verdict: Approve

在六个强制审查维度上，**未发现**错误宣称 package stable / Native SQL / HA、密钥入库、SSOT `taosx/` 分叉、version 漂移，或 fail-closed 被削弱的阻塞问题。公开 API 的 crate-root 导出已被 `public_api_surface` 点名，行为面另有 conformance / unit / live 入口支撑。

可合并（仍须走仓库 PR + CI + maintainer 审批；本 Verdict **不**替代 Human L2/L3 批准）。

---

## Blocking Issues

**无。**

---

## 维度结论

### 1. 是否错误宣称 package stable / Native SQL / HA

| 检查点 | 结论 |
|--------|------|
| package stable | **未宣称**（OPEN / NO-GO / NOT CLAIMED / `publish = false`） |
| Native SQL / FFI | **NO-GO**，实现仅为 REST SQL + NativeWs 握手探测 |
| HA / Cluster / failover | **NO-GO**，无矩阵/无证据 |

关键措辞均为「受限 REST 生产默认入口 / 可生产默认使用（受限 REST）」并与 package stable **显式切割**，与 adapters 总对齐中的「P0 生产入口 ≠ package stable」一致。

**Evidence**

- `docs/ssot/taosx-ssot-alignment.md:9-10,25,32,37` — version `0.3.4`；结论与 TAOSX-9/14 NO-GO
- `docs/report/2026-07-23/taosx-ten-round-review.md:10,177-180,200-204` — package stable / Native SQL / HA 保持 NO-GO
- `crates/adapters/storage/taos/Cargo.toml:4,9` — `version = "0.3.4"` · `publish = false`
- `crates/adapters/storage/taos/src/native.rs:23-31,201`（client 注释）— WS 仅握手；SQL 仍 REST
- `.agents/ssot/adapters/storage/taos/{goal,matrix,spec,release,review}/**` — 多处 package stable / Native / HA NO-GO 一致

**Risk:** P2（措辞误读风险已由同文 NO-GO 消解；无升级）

---

### 2. 是否把密钥写入仓库

| 检查点 | 结论 |
|--------|------|
| crate 源码 / 测试 / docs | 未见硬编码密码、token、私钥块 |
| SSOT evidence | 脱敏摘要；声明密钥不入库 |
| 配置面 | 密码仅 `FOUNDATIONX_TAOSX_PASSWORD` / 运行时字段；`Debug` 脱敏为 `***` |

**Evidence**

- 对 `crates/adapters/storage/taos`、`.agents/ssot/adapters/storage/taos`、`docs/report/2026-07-23`、`docs/ssot/taosx-ssot-alignment.md` 检索 `AKIA` / `BEGIN … PRIVATE KEY` / 长字面 password 赋值：**无匹配**
- `crates/adapters/storage/taos/src/config.rs:143-163,400-405` — `Debug` 红acted；单测 `debug_redacts_password`
- `.agents/ssot/adapters/storage/taos/evidence/2026-07-23-real-dev-live.md:7,27-31` — 脱敏声明
- `docs/ssot/taosx-ssot-alignment.md:45` — 禁止密钥入库；export 脚本注入

**Risk:** P0 维度通过（未发现密钥入库）

---

### 3. 公开 API 测试是否覆盖 crate-root 导出

默认 feature 下 crate-root 导出（`src/lib.rs:15-20`）：

| 符号 | `public_api_surface` 点名 |
|------|---------------------------|
| `TaosClient` / `TaosPool` / `TaosConfig` / `TaosExecResult` / `TaosPoolStats` | 是（类型 + 构造/字段） |
| `build_insert_sql_chunks` | 是 |
| `HARD_MAX_*`（6 常量） | 是（求和引用） |
| `TransportMode` / `TsPrecision` | 是（变体 + parse/转换） |
| `build_native_ws_url` / `validate_mode` / `connect_native_ws` | 是 |
| `TaosAdapter` | 仅 `feature = "scaffold"`，不在默认面 |

池同步面：`connect_without_ping` / `client` / `config` / `precision` / `stats` / `is_closed` 已点名。  
异步行为面（`connect`、写查、close、远程 fail-closed）由 `tests/taos_conformance.rs`、模块单测与 `live_smoke`（`#[ignore]`）覆盖，**不**要求全部挤进 `public_api_surface`。

**非阻塞缺口（不构成 Request changes）**

- `public_api_surface::config_public_methods_exercised` 用 `TaosConfig::default()` 断言「不含 s3cret」，脱敏真实覆盖依赖 `config::tests::debug_redacts_password`（`config.rs:400-405`），表面测试本身偏弱。
- 公开方法 `connect_from_env` / `write_batch*` / `close` / `ping` 等未在 `public_api_surface` 逐一点名，但有行为测试。

**Evidence**

- `crates/adapters/storage/taos/src/lib.rs:15-20,27-129`
- `crates/adapters/storage/taos/tests/taos_conformance.rs:84-110` — 远程明文/空密 fail-closed
- matrix S-15：`.agents/ssot/adapters/storage/taos/matrix/matrix.md:25`

**Risk:** P2

---

### 4. SSOT 路径是否错误新建 `taosx/` 分叉

| 检查点 | 结论 |
|--------|------|
| 权威路径 | `.agents/ssot/adapters/storage/taos/` |
| 平行树 `adapters/storage/taosx/` | **不存在** |
| 树内 `taosx-spec.md` | SUPERSEDED 指针 → `spec/spec.md`，**不是**平行目录 |

文档多处显式禁止另建 `taosx/` SSOT 树；package 名 `taosx` 与目录 `taos/` 映射固定。

**Evidence**

- 目录枚举：`.agents/ssot/adapters/storage/` 仅有 `taos/` 等 storage 子树，无 `taosx/`
- `docs/ssot/taosx-ssot-alignment.md:6`
- `docs/report/2026-07-23/taosx-ten-round-review.md:7,17-20,160`
- `.agents/ssot/adapters/storage/taos/taosx-spec.md:1-9`
- `.agents/ssot/adapters/storage/taos/goal/goal.md:10`

**Risk:** 无（维度通过）

---

### 5. 代码/文档一致性（version `0.3.4`）

| 位置 | version |
|------|---------|
| `crates/adapters/storage/taos/Cargo.toml` | `0.3.4` |
| `CHANGELOG.md` / `releases/0.3.4.md` | `0.3.4` |
| `docs/ssot/taosx-ssot-alignment.md` | `0.3.4` |
| SSOT `goal` / `matrix` / `spec` / `landing` | `0.3.4` |
| 十轮审查 D-11 | 对齐 `0.3.4` |

十轮正文中「曾 0.3.2 / Cargo 0.3.3」为过程叙述；**最终交付态**与 Cargo 一致为 `0.3.4`。

**Evidence**

- `Cargo.toml:4`
- `docs/ssot/taosx-ssot-alignment.md:9`
- `.agents/ssot/adapters/storage/taos/matrix/matrix.md:5`
- `.agents/ssot/adapters/storage/taos/goal/goal.md:9`
- `.agents/ssot/adapters/storage/taos/spec/spec.md:3,20`
- `.agents/ssot/adapters/storage/taos/plan/infra-rs-landing.md:13`
- `CHANGELOG.md:3-18`

**Risk:** 无（维度通过）

---

### 6. 安全 fail-closed 是否被削弱

| 控制 | 状态 |
|------|------|
| 远程非 loopback 强制 TLS | `validate` 拒绝明文 |
| 远程强制非空 password | 拒绝空/空白密码 |
| strict host（禁 scheme/userinfo/path 等） | `valid_host` + 单测 |
| REST redirect | `reqwest::redirect::Policy::none()`（connect / connect_without_ping） |
| 资源 `HARD_MAX_*` | validate fail-fast |
| 精度配置 vs 探测不一致 | connect fail-closed |
| TLS 跳过校验 | 代码中 **无** `danger_accept_invalid_certs` 等 |
| 密码 Debug | 固定 `***` |

相对「削弱 fail-closed」审查标准：**未削弱**；conformance 仍覆盖远程明文/空密在发网前拒绝。

**Evidence**

- `crates/adapters/storage/taos/src/config.rs:302-309,350-372,432-456,459-480`
- `crates/adapters/storage/taos/src/client.rs:174,180-185,234-240`
- `crates/adapters/storage/taos/tests/taos_conformance.rs:84-110`
- grep `danger|accept_invalid` in taos crate：**无匹配**

**Risk:** 无（维度通过）

---

## Non-blocking Notes（P2 follow-up，不挡 Approve）

| ID | 说明 | 建议 |
|----|------|------|
| N-1 | `public_api_surface` 脱敏断言未注入假密码，依赖 `config` 单测 | 可选：在表面测试构造 `password: "s3cret"` 再 assert |
| N-2 | SSOT `review/review.md` live 日期仍写 2026-07-22，与 2026-07-23 evidence 略不同步 | 合并后或下轮同步指针 |
| N-3 | 对外「可生产默认使用」须始终绑定「受限 REST + package stable NO-GO」 | 已满足；后续文案勿丢边界句 |
| N-4 | draft 快照 `plan/infra-rs-draft-spec-goal.md` 仍含 aspirational native/TMQ 等 | 已标注 draft/≠stable；保持只读合同快照即可 |

---

## Risk 汇总

| 级别 | 项 |
|------|-----|
| **P0** | 无 blocking |
| **P1** | 无 |
| **P2** | N-1～N-4（文档/表面测试加固，不阻合并） |

整体交付风险：**P2**（边界诚实、安全默认面完整、version 一致）。

---

## Evidence 索引（命令 / 路径）

| 类型 | 内容 |
|------|------|
| 路径 | `crates/adapters/storage/taos/src/{lib,config,client,native}.rs` |
| 路径 | `crates/adapters/storage/taos/{Cargo.toml,CHANGELOG.md,README.md,releases/0.3.4.md}` |
| 路径 | `docs/ssot/taosx-ssot-alignment.md` |
| 路径 | `docs/report/2026-07-23/taosx-ten-round-review.md` |
| 路径 | `.agents/ssot/adapters/storage/taos/{goal,matrix,spec,evidence,review,plan}/**` |
| 符号 | `TaosConfig::validate` · `TaosPool::connect` · `connect_native_ws` · `public_api_surface` |
| 命令 | 静态审查：目录列举 + ripgrep（密钥模式 / danger TLS / version / NO-GO 措辞） |
| 命令 | 本 Reviewer **未**重跑 `cargo test -p taosx`（只读职责；离线/live 证据引用实现方十轮与 SSOT evidence） |
| 结果 | 六维全部通过；Blocking Issues = 空；Verdict = **Approve** |

---

## 审查独立性声明

- 本文件由独立 Reviewer 角色产出，**未**参与本分支实现编码。
- 不构成 Human maintainer 的 L2/L3 合并批准。
- 与实现方 `taosx-ten-round-review.md` 结论交叉核对后一致：可宣称受限 REST 生产默认面；**不可**宣称 package stable / Native SQL / HA。
