# 核心 crate 生产级修复方案

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-CORE-PROD-002` |
| 日期 | 2026-07-21 |
| 输入审计 | [docs/report/2026-07-21/core-crates-production-readiness.md](../report/2026-07-21/core-crates-production-readiness.md) |
| 已合入基线 | PR [#98](https://github.com/xhyperium/infra.rs/pull/98) squash `76c56d7`（批次 A–E **子集**） |
| W0–W5 实现合入 | PR [#120](https://github.com/xhyperium/infra.rs/pull/120) [#121](https://github.com/xhyperium/infra.rs/pull/121) [#124](https://github.com/xhyperium/infra.rs/pull/124) [#125](https://github.com/xhyperium/infra.rs/pull/125) [#127](https://github.com/xhyperium/infra.rs/pull/127) [#128](https://github.com/xhyperium/infra.rs/pull/128)（2026-07-21 squash → `main`） |
| 目标 | 五个核心 crate 达到可签字的 **Production Ready**（按模块分轨，不假装整体一次过线） |
| 范围 | `xhyper-contracts` · `xhyper-decimalx` · `xhyper-canonical` · `xhyper-kernel` · `xhyper-testkit`；以及阻断生产语义的 adapters 验证入口 |
| 性质 | **可执行修复计划**（含顺序、验收、门禁、签字）；**不是**发布批准本身 |
| 状态 | **W0–W5 已合入 main**（2026-07-21）· **验收勾选已回写** · **L5 / DEFER-7 待 Maintainer 签核** · 整体 Production Ready：**否** |

---

## 0. 一句话结论

PR #98 闭合了审计 **P0/P1 可机器验证子集**；PLAN-CORE-PROD-002 **W0–W5 实现已于 2026-07-21 合入 `main`**（见上表 PR）。  
**§8.8 / DEFER-7 人工签核尚未完成**，因此：

> 证据链与门禁已显著加强（decimal oracle、wire v1.1–v1.3、contracts 合同测、adapter mock 验证入口、public-api / 支持矩阵）。  
> **五个 crate 整体 Production Ready：否**（待 Maintainer 签核 + 真实后端仍 Accept）。

本计划把报告 §11.2 的 8 个 DEFER 与 §8 的 8 条门槛映射为 **6 个可交付波次（W0–W5）**；实现波次已闭合，剩余为签核与真实后端 Accept 项。

---

## 1. 深度分析：审计结论复盘

### 1.1 审计当初的问题（§4–§6）vs 现状

| 审计阻断/改进 | PR #98 状态 | 仍缺什么（生产级） |
|--------------|-------------|-------------------|
| **P0 decimalx** 字段公开 / serde 绕过 / checked panic | **已闭合**：字段私有、`try_new`/serde 校验、`DecimalError` 中文、中间值溢出合同 | 独立 oracle、fuzz、mutants/Miri 证据；资金路径强制 `checked_*` 的调用侧门禁 |
| **P0 canonical** wire/演进未闭合 | **部分闭合**：`wire::COMMITTED_WIRE_V1` 五类型 + deny_unknown + golden/N-1/拒绝样例 | `Order`/`Tick`/`Trade` 等 Uncommitted；无协议 envelope；ID/ts newtype 未上类型层 |
| **P0 contracts** 仅接口形状 | **部分闭合**：`TxContext`/`begin_tx` 对象安全、`BusMessage`/`MessageAck`、Fake/Recording testkit、bootstrap `Bounded*` | 全 trait 深度语义、真实后端入口、VenueAdapter override 编译门禁、独立 contract-testkit crate |
| **P1 kernel** Clock domain / loom CI / 关停 deadline | **已闭合**（源码 + `kernel-loom.yml`） | 非 Linux 矩阵；MSRV 已在 CI 过线但仍须持续证明；archgate 未移植 |
| **P1 testkit** ManualClock 评级 | **有条件就绪**；domain/中文/lint 已跟随 | poison 三入口语义需对外合同硬化；contract-testkit 不在本 crate |
| **P2 lint / API / fuzz / 平台** | lint workspace 已统一；覆盖率 100% 门禁在 | API snapshot / semver-diff、fuzz、非 Linux、人工签字 **均未做** |

### 1.2 为何「测试全绿 + 100% 行覆盖」仍不等于 Production Ready

审计 §3 已写明，当前代码基线再次确认：

1. **行覆盖率**只保证插桩行至少执行一次，不保证恶意输入、跨版本 wire、trait 生产语义。
2. **同实现自往返 serde** 不证明 N-1 兼容；committed 仅 5 个类型有 golden/N-1。
3. **adapters 是内存 scaffold**：`PostgresAdapter::begin_tx` 直接 `FakeTxContext`，不能证明事务原子性。
4. **Additive default**（`VenueAdapter::cancel_order_request` 等）允许旧实现编译通过却在运行时返回 `Invalid`——生产路径需要编译期/CI 强制 override。
5. **无公开 API 棘轮**：`publish = false` 不保护内部消费者免受破坏性变更。

### 1.3 模块诚实评级（2026-07-21 · main 后）

| 模块 | 评级 | 可安全使用 | 不可宣称 |
|------|------|------------|----------|
| `decimalx` | 有条件就绪（内部） | 受控入口 + `checked_*` 资金路径 | package stable / 任意 wire 跨版本 |
| `canonical` | 部分就绪 | committed v1 五类型跨进程边界 | 全 DTO wire / envelope 演进 |
| `contracts` | 部分就绪 | Tx/消息合同测 + in-memory fake | 生产 backend 语义 / 全 trait 套件 |
| `kernel` | 接近就绪 | L0 错误 / 时间 / 关停 | 跨平台未经声明的假设 |
| `testkit` | ManualClock core 有条件就绪 | 确定性测试支持 | 生产 runtime / 全 testkit 平面 |

### 1.4 §8 门槛 ↔ DEFER 映射（生产签字闸）

| §8 门槛 | 对应 DEFER / 缺口 | 本计划波次 |
|---------|-------------------|------------|
| 1. decimalx 无公开非法状态 panic/藏错 | 已基本闭合；补 DEFER-4 证据 | W1 |
| 2. committed wire 版本/矩阵/旧 fixture/迁移 | DEFER-3（扩面）+ 现有 v1 硬化 | W2 |
| 3. 每生产 trait 语义 + 合同测 + 非 scaffold 入口 | DEFER-1 · DEFER-2 | W3 · W4 |
| 4. bootstrap/adapters 同一权威 trait | 命名已收敛；真实实现仍缺 | W3 · W4 |
| 5. 单调 domain 无静默误比较 | 已闭合；回归门禁保持 | W0 守门 |
| 6. loom/MSRV/clippy/test/doc/fuzz 等持续门禁 | DEFER-4 · DEFER-5 · DEFER-6 | W1 · W5 |
| 7. 用户可见错误中文 | 核心路径已闭合；contracts 默认文案抽查 | W0 · W3 |
| 8. 发布/回滚人签字 + 剩余 DEFER 显式 | DEFER-7 | W5 |

---

## 2. 生产级定义（本计划采用）

避免模糊「生产级」。本计划采用**分层 Production Ready**：

| 层级 | 含义 | 签字条件 |
|------|------|----------|
| **L1 Internal Ready** | 进程内库；非法状态不可表示；checked 路径无 panic；中文错误；持续 CI | 模块 owner |
| **L2 Wire Ready** | 跨进程/落盘/回放类型有版本、兼容矩阵、拒绝策略 | 模块 owner + wire owner |
| **L3 Contract Ready** | trait 有语义合同 + conformance suite + 至少一非 scaffold 验证入口 | contracts owner + adapter owner |
| **L4 Platform Ready** | 支持矩阵（OS/MSRV）声明并实测；API/semver 门禁 | platform owner |
| **L5 Release Ready** | 回滚责任人签字；剩余 DEFER 列表冻结；CHANGELOG 发布说明 | maintainer（§8.8） |

**整体 Production Ready** = 五个 crate 均达到各自声明的目标层级，且 L5 签字完成。

建议目标层级（可在 W0 确认时调整）：

| crate | 目标层级 | 说明 |
|-------|----------|------|
| `kernel` | L1 + L4 | 无 wire；平台矩阵声明即可 |
| `testkit` | L1（测试支持） | 明确 **非** 生产 runtime |
| `decimalx` | L1 +（可选 L2 若存在外部 wire 消费者） | 金额核心；优先 L1 证据硬化 |
| `canonical` | L2（committed 清单） | Uncommitted 保持显式；可分批升格 |
| `contracts` | L3（生产用 trait 子集） | 非生产 trait 可标 `experimental` |

---

## 3. 非目标（明确不在本计划）

- 把 adapters 做成完整交易所/数据库产品（只需 **验证入口** 证明 contract 语义）。
- crates.io 公开发布（`publish = false` 可保持；内部 Production Ready 不依赖 crates.io）。
- 移植 monorepo archgate / 全量 domain_exchange。
- 性能/长稳/集群故障注入基准（另开性能计划）。
- 在未签字前把 README 改成「Production Ready」。

---

## 4. 修复波次总览

```text
W0  基线冻结与范围裁定  ──►  不可跳过
W1  decimalx 证据硬化     ──►  L1 签字候选
W2  canonical wire 扩面   ──►  L2 签字候选（分批 DTO）
W3  contracts 语义深化    ──►  L3 子集
W4  真实后端验证入口      ──►  闭合 DEFER-1（可与 W3 重叠）
W5  治理门禁 + 人工签字   ──►  L4/L5 · §8 全过
```

依赖：

- W1 不依赖 W2–W4。
- W2 依赖 decimalx 校验边界（已满足）；扩 committed 时继承 decimal serde。
- W3 合同套件可先对 Fake 跑；**L3 签字**依赖 W4 至少一个真实入口。
- W5 可与 W1–W4 并行推进 API/MSRV 门禁，但 **签字必须在 W1–W4 目标子集完成后**。

---

## 5. W0 — 基线冻结与范围裁定

### 5.1 目的

防止「全量 Production Ready」变成无限范围。先冻结：哪些 trait/DTO 进本轮生产面。

### 5.2 任务

| ID | 任务 | 完成物 | Owner |
|----|------|--------|-------|
| W0-1 | 冻结 **生产 trait 子集**（建议首批） | `docs/plans/artifacts/prod-trait-inventory.md` | Lead |
| W0-2 | 冻结 **committed wire 升格候选** | 扩展 `wire-commitment-matrix` 或本仓 `docs/plans/artifacts/` | Lead |
| W0-3 | 声明 **支持矩阵**（OS / MSRV / arch） | `docs/governance` 或 crate README 补丁草案 | Lead |
| W0-4 | 为每个 DEFER 标 **Close / Accept / Defer-with-sign** | 本文件 §11 更新 | Maintainer |
| W0-5 | 创建 beads epic + 子任务 | `bd create` 树 | Executor |

### 5.3 建议首批生产 trait 子集

| Trait | 首批？ | 理由 |
|-------|--------|------|
| `KeyValueStore` | 是 | 语义简单；redis 可做真实入口 |
| `TxRunner` / `TxContext` | 是 | 已有 Fake；需 postgres 真实入口 |
| `EventBus` | 是 | 已有 BusMessage；需 kafka/nats 其一 |
| `Instrumentation` | 是 | observex 已实现；合同测补齐 |
| `ExecutionVenue` / `MarketDataSource` 等能力 trait | 是（优先于整包 `VenueAdapter`） | 避免双平面；可渐进 |
| `VenueAdapter` 整包 | 条件 | 仅当能力 trait 迁移期结束后作为 facade 或删除 |
| `Repository` | 是 | 需分页/not-found/幂等合同 |
| `ObjectStore` | 可选二期 | put/get 语义即可 |
| `TimeSeriesStore` | 可选二期 | 依赖 Tick wire |
| `AnalyticsSink` / `PubSub` | 可选二期 | 可标 experimental |

### 5.4 验收

- [x] 生产 trait 清单与 wire 升格清单有文件落盘：[`artifacts/prod-trait-inventory.md`](./artifacts/prod-trait-inventory.md)、[`artifacts/wire-promotion-candidates.md`](./artifacts/wire-promotion-candidates.md)
- [x] DEFER 处置表无「未分类」项：[`artifacts/defer-disposition.md`](./artifacts/defer-disposition.md)
- [x] 支持矩阵声明：[`artifacts/support-matrix.md`](./artifacts/support-matrix.md)（Accept 仅 Linux）
- [x] beads epic 可 `bd ready`：`infra-asa` + `infra-asa.1`…`.6`（epic 已 close；6/6 子任务 complete）
- [x] PR 评审通过并合入：计划 [#120](https://github.com/xhyperium/infra.rs/pull/120)；实现 [#121](https://github.com/xhyperium/infra.rs/pull/121) [#124](https://github.com/xhyperium/infra.rs/pull/124) [#125](https://github.com/xhyperium/infra.rs/pull/125) [#127](https://github.com/xhyperium/infra.rs/pull/127) [#128](https://github.com/xhyperium/infra.rs/pull/128)；收尾 [#138](https://github.com/xhyperium/infra.rs/pull/138)

---

## 6. W1 — `decimalx` 证据硬化（DEFER-4）

### 6.1 问题

不变量已在类型层闭合，但 **缺少独立正确性与异常输入证据**。资金路径若只靠 100% 行覆盖，不足以支撑 L1 签字。

### 6.2 任务

| ID | 任务 | 细节 | 完成判据 |
|----|------|------|----------|
| W1-1 | 差分 oracle | 选 `rust_decimal` 或 `bigdecimal` **仅 dev-dep**；对 `checked_add/sub/mul/div/rescale` 做同输入对比 | property/oracle 测试绿；文档声明 oracle 边界 |
| W1-2 | 完整边界策略 | `i128` 极值、`MAX_SCALE`、`TECH_MAX_POW10_EXP`、除零、对齐溢出 | 表驱动测试覆盖矩阵 |
| W1-3 | fuzz 入口 | `cargo fuzz` 或 `proptest` 扩展：任意字节 → Deserialize + 运算 | CI 可选 job 或 scheduled；失败可复现 seed |
| W1-4 | mutants / Miri 证据 | 参考 kernel 的 weekly workflow；decimal 增加 scheduled mutants/miri | workflow 绿；报告链到 `docs/status` 或 run 日志 |
| W1-5 | 调用侧约定文档 | 资金路径禁用 panicking `+/-/*`；lint 或 grep 门禁（workspace 内 adapters/examples） | script 或 CI step 拒绝生产路径使用 panicking ops |
| W1-6 | wire 事实声明 | `docs/WIRE.md` / README：serde shape = 当前事实 ≠ 跨版本协议 | 文档诚实，与 types 对齐文一致 |

### 6.3 验证命令

```bash
cargo test -p decimalx --all-targets
cargo clippy -p decimalx --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p decimalx --no-deps
# 覆盖率
node scripts/quality-gates/cov-gate-100.mjs decimalx   # 按仓库实际入口调整
# fuzz / mutants / miri：按新增 workflow
```

### 6.4 验收 → L1 Internal Ready（decimalx）

> **实现回写（2026-07-21）**：PR [#121](https://github.com/xhyperium/infra.rs/pull/121)。L1 **候选就绪**；整体/L5 仍否。

- [x] 无公开可构造非法状态导致 panic（字段私有 + `try_new`/serde 校验；PR #98 + 回归）
- [x] oracle 差分与边界矩阵在 CI 或 scheduled 有证据（`tests/oracle_diff.rs`、`boundary_matrix.rs`；`decimal-miri.yml` / `decimal-mutants.yml` scheduled）
- [x] 资金路径 panicking ops 有门禁或明确 exclude 列表（`scripts/quality-gates/check-decimal-no-panicking-ops.mjs`）
- [x] 模块 README 仍不写整体 Production Ready，直到 L5（诚实措辞；L5 人签前保持）

**备注（W1-3）**：完整 `cargo fuzz` 靶场未建；以 `adversarial_serde.rs` + proptest 作为轻量 fuzz 入口（与 DEFER-4 Close 口径一致）。

---

## 7. W2 — `canonical` wire 扩面（DEFER-3）

### 7.1 问题

仅 5 类型 Committed v1。交易关键路径上的 `Order`/`Tick`/`Trade` 仍 Uncommitted，一旦跨进程使用即无兼容承诺。

### 7.2 策略

**分批升格**，禁止一次宣称全量 wire stable。

| 批次 | 升格类型（建议） | 依赖 |
|------|------------------|------|
| v1（已有） | `CancelOrderRequest` `OrderRef` `OrderAck` `OrderStatus` `Side` | 已合入 |
| v1.1 | `Order`（下单/回报最小字段集） | 字段冻结评审 |
| v1.2 | `Tick` `Trade` | 行情回放 |
| v1.3 | `Position` `OrderBookSnapshot` `PriceLevel` `SymbolMeta` | 账户/盘口 |
| 可选 | 协议 envelope（`schema_version`） | 破坏性迁移时再上，不必阻塞 v1.x |

### 7.3 每个升格类型的强制清单

对每一类型：

1. 字段名 / 类型 / 缺省策略写入 `wire` 模块与 fixture
2. `#[serde(deny_unknown_fields)]`（或显式 `deny` 策略文档 + 测试）
3. 双向 golden（serde 往返 + 手写 JSON 字节相等）
4. N-1 旧 fixture（至少 1 个历史样例）
5. 未知字段 / 未知 variant / 缺字段 **拒绝样例**
6. 含 `Decimal`/`Money` 字段：非法 scale/currency **必须**反序列化失败
7. 更新 `COMMITTED_WIRE_V1` 或引入 `COMMITTED_WIRE_V1_1` 清单（版本递增规则写死）
8. `check-canonical-align.mjs` 与 wire 矩阵同步

### 7.4 ID / 时间类型化（降风险，非阻塞）

| 项 | 建议 | 优先级 |
|----|------|--------|
| `VenueId` / `InstrumentId` | newtype + `shape` 校验构造；保持 serde 为 string | P1 |
| `ts: i64` | 保持 ns 语义；可选 `DtoTimestamp` newtype 防单位混用 | P1 |
| adapter 边界 | wire DTO → 已校验 domain 的转换函数统一入口 | P0（文档+测试） |

### 7.5 验收 → L2 Wire Ready（committed 子集）

> **实现回写（2026-07-21）**：PR [#124](https://github.com/xhyperium/infra.rs/pull/124)。L2 仅限 **committed 清单**（`COMMITTED_WIRE_V1`…`V1_3`）；非全 DTO。

- [x] 本轮声明的 committed 清单 100% 满足 §7.3 八项（v1 + v1.1 `Order` + v1.2 `Tick`/`Trade` + v1.3 账户/盘口类型）
- [x] Uncommitted 类型 rustdoc 仍显式标注
- [x] 无「全 wire Production Ready」措辞
- [x] `node scripts/quality-gates/check-canonical-align.mjs` 绿（合入 CI / 本地门禁）

---

## 8. W3 — `contracts` 语义深化（DEFER-2 · DEFER-8）

### 8.1 问题

当前只有 Tx/消息/少量 trait 有行为测试。`ObjectStore`/`Repository`/`TimeSeriesStore` 等方法面无法表达分页、not-found、取消、幂等、并发写。`VenueAdapter` additive default 无编译门禁。

### 8.2 每 trait 语义合同模板（强制）

对 W0 冻结的每个生产 trait，落盘一节（可在 `crates/contracts/docs/contracts/`）：

```markdown
## TraitName

- 输入所有权 / 生命周期
- 成功语义
- 失败分类（映射 XError kind）
- 幂等：是/否/条件
- 取消与超时
- 顺序与一致性
- 资源释放
- not-found / 空结果约定
- 分页 / 游标（如适用）
- 对象安全要求
- 参考 fake
- 合同测试入口
- 真实验证入口（W4）
```

### 8.3 任务包

| ID | 任务 | 完成判据 |
|----|------|----------|
| W3-1 | `KeyValueStore` 合同：TTL、覆盖写、缺失键 | Fake + suite |
| W3-2 | `Repository` 合同：find 缺失、save 幂等、并发 last-write 文档 | Fake + suite |
| W3-3 | `EventBus` 扩展：可选 `ack` 扩展 trait 或明确 at-most-once 冻结 | 文档 + FakeEventBus 测 |
| W3-4 | `ObjectStore`：put 覆盖、get 缺失 → 明确 `NotFound`/`Invalid` | Fake + suite |
| W3-5 | `TimeSeriesStore`：时间窗、空结果、乱序点策略 | Fake + suite（可依赖 Tick wire） |
| W3-6 | `Instrumentation` 计数可观测（Recording 实现） | 已有 observex；补 contracts 侧 Recording |
| W3-7 | **能力 trait 优先**：文档标明 `ExecutionVenue` 等为生产入口；`VenueAdapter` 迁移期 | ADR 或 contracts rustdoc |
| W3-8 | **DEFER-8**：in-tree adapter 强制 override 门禁 | 见 §8.4 |
| W3-9 | 独立 `contract-testkit` 路径决策 | 保持 in-crate **或** `crates/test-support/contracts`；二选一写 ADR |
| W3-10 | 错误文案中文抽查 | default 实现 / fake 用户可见字符串 |

### 8.4 VenueAdapter additive override 门禁（DEFER-8）

可选方案（择一，推荐 A）：

| 方案 | 做法 | 优点 | 缺点 |
|------|------|------|------|
| **A. 清单 + 测试** | `tests/venue_override_gate.rs` 对 in-tree adapter 调用 `cancel_order_request`/`query_order_request`，断言 **不是** default Invalid 文案 | 简单、无新依赖 | 不拦 out-of-tree |
| **B. 拆 trait** | 新方法只放在无 default 的 `ExecutionVenue`；`VenueAdapter` 逐步降级 | 编译期安全 | 迁移成本 |
| **C. 宏/登记** | `assert_impl_all` + 强制封装 | 中等 | 复杂 |

**生产建议**：短期 A + 文档；中期 B 完成双平面收敛。

### 8.5 验收 → L3 Contract Ready（首批 trait）

> **实现回写（2026-07-21）**：PR [#128](https://github.com/xhyperium/infra.rs/pull/128)。L3 指 **首批 trait + Fake/conformance**；二期 trait（ObjectStore/TimeSeries 等）为 Accept。  
> **验证入口**见 W4（mock-first，非真实云端）。

- [x] 首批 trait 均有语义文档 + Fake + conformance suite（`crates/contracts/docs/contracts/*` + `tests/conformance_first_batch.rs`）
- [x] in-tree venue adapter override 门禁绿（`tests/venue_override_gate.rs`；方案 A 运行时门禁）
- [x] `public_surface` 驱动真实路径（禁止占位 `assert_eq!(15,15)` 类；`tests/public_surface.rs`）
- [x] bootstrap 仅 re-export / `Bounded*`，无静默同名冲突

**备注**：W3-4/W3-5（ObjectStore/TimeSeries）属二期 Accept，不阻塞首批 L3 候选。

---

## 9. W4 — 真实后端验证入口（DEFER-1）

### 9.1 问题

adapters 全为内存 scaffold（例：postgres `begin_tx` → `FakeTxContext`）。没有真实后端，L3 不能签字。

### 9.2 策略：**验证入口 ≠ 完整产品**

每个首批 trait **至少一个**非 scaffold 验证路径即可：

| Trait | 建议入口 | 形态 |
|-------|----------|------|
| `TxRunner` | `postgresx` feature `live` + testcontainers/CI service | `#[ignore]` 本地 + CI optional job |
| `KeyValueStore` | `redisx` live | 同上 |
| `EventBus` | `kafkax` 或 `natsx` 其一 live | 同上 |
| `ExecutionVenue` | `okxx`/`binancex` **sandbox/mock HTTP**（transportx 注入） | 录制 fixture 回放优先于真实下单 |
| `ObjectStore` | 本地文件系统或 minio feature | 二期可接受 |

### 9.3 硬约束

1. **默认 `cargo test --workspace` 不依赖外部服务**（live 测 `ignore` 或 feature）。
2. scaffold 与 live 实现 **分离类型或 feature**，禁止 silent 假装。
3. 合同 suite 对 Fake 与 live **复用同一测试函数**（generic over trait object / 宏）。
4. 密钥仅 CI secrets / 本地 `.env`（gitignore）；禁止写入仓库。
5. 交易所 live 默认 **只读或 sandbox**；真实下单需人工 flag。

### 9.4 任务

| ID | 任务 | 完成判据 |
|----|------|----------|
| W4-1 | postgres live Tx commit/rollback 可观察 | 与 `RecordingTxRunner` 同断言语义 |
| W4-2 | redis live KV get/set/ttl | suite 复用 |
| W4-3 | kafka 或 nats live publish/subscribe | 至少 at-most-once 证明 |
| W4-4 | venue HTTP mock：cancel/query structured path | fixture 驱动；override 非 default |
| W4-5 | CI workflow `contracts-live.yml`（可选/scheduled） | 文档说明触发条件 |
| W4-6 | adapters 对齐文更新：哪些 live、哪些仍 scaffold | `docs/ssot/adapters-ssot-alignment.md` |

### 9.5 验收

> **实现回写（2026-07-21）**：PR [#125](https://github.com/xhyperium/infra.rs/pull/125)。交付为 **离线 mock 验证入口**（`MockRedisAdapter`、exchange `MockHttpTransport` 等）；**非** testcontainers/真实云凭证。  
> DEFER-1：**Close（首批 mock）+ Accept（真实后端）**——见 [`artifacts/defer-disposition.md`](./artifacts/defer-disposition.md)。

- [x] 首批 trait 每个都有「Fake 合同绿 + live/**mock** 入口绿」证据路径（mock-first；`contracts-live.yml` = offline mock / `workflow_dispatch`）
- [x] scaffold 文档标明 mock ≠ 生产实现（adapter rustdoc / 对齐文口径）
- [x] DEFER-1：**Close mock 路径**；真实 DB/MQ/交易所 **Accept**（不阻塞 mock-L3 候选；阻塞「真实后端 L3」宣称）

---

## 10. W5 — 治理门禁与签字（DEFER-5 · DEFER-6 · DEFER-7）

### 10.1 公开 API / semver 门禁（DEFER-5）

| 任务 | 建议工具 | 完成判据 |
|------|----------|----------|
| API snapshot | `cargo public-api` 或 `rustdoc-json` diff | PR 上 breaking 可见 |
| semver 检查 | `cargo-semver-checks`（对稳定 crate） | 或文档约定 0.x 允许 break 但必须标签 |
| Additive Only 机控 | contracts：新增无 default 的 trait 方法 → CI fail | script 扫描 PR diff |
| Breaking 标签 | PR template 勾选 + CODEOWNERS | 流程 |

### 10.2 平台矩阵（DEFER-6）

二选一（W0 裁定）：

1. **仅 Linux x86_64 + MSRV 1.85**：写入 README/governance，关闭「未知跨平台」歧义。
2. **多平台**：增加 macOS/Windows CI job（可 scheduled）。

MSRV 1.85 已在 PR #98 CI 通过；W5 要求 **workflow 持续存在** 且文档声明一致。

### 10.3 统一持续门禁清单

| 门禁 | kernel | decimal | canonical | contracts | testkit |
|------|--------|---------|-----------|-----------|---------|
| test + clippy + fmt + doc | 已有 | 已有 | 已有 | 已有 | 已有 |
| line cov 100% | 已有 | 已有 | 已有 | 已有 | 已有 |
| loom | 已有 | n/a | n/a | n/a | n/a |
| miri/mutants | scheduled | **W1 补** | 可选 | 可选 | scheduled 已有 |
| fuzz | n/a | **W1 补** | 恶意 serde 可选 | n/a | n/a |
| API diff | **W5** | **W5** | **W5** | **W5** | **W5** |
| live contracts | n/a | n/a | n/a | **W4** | n/a |
| align script | n/a | n/a | 已有 | n/a | n/a |

### 10.4 中文错误与 lint 终检

```bash
# 用户可见 Display 抽查（示例）
rg -n 'impl (fmt::)?Display for' crates/kernel crates/testkit crates/types crates/contracts \
  -A6 | rg -n 'write!|Error|错误|失败' 

# 五 crate 属性
# forbid(unsafe_code) / deny(missing_docs) 建议 contracts/decimal/canonical 对齐 kernel
```

建议 W5 将 `contracts`/`decimalx`/`canonical` 提升到与 kernel 一致的：

- `#![forbid(unsafe_code)]`
- `#![deny(missing_docs)]`
- `#![deny(unreachable_pub)]`

（若 contracts 文档债过重，可分 PR：先 deny missing_docs 于新模块。）

### 10.5 §8 人工签字包（DEFER-7）

落盘 `docs/plans/artifacts/prod-signoff-YYYY-MM-DD.md`：

| 签字项 | 角色 | 证据指针 |
|--------|------|----------|
| decimalx L1 | owner | W1 CI 日志 |
| canonical L2 清单 | wire owner | W2 fixture + align |
| contracts L3 首批 | contracts owner | W3 suite + W4 live |
| kernel L1+L4 | owner | loom + MSRV + 矩阵声明 |
| testkit ManualClock L1 | owner | 测试 + 非 runtime 声明 |
| 剩余 Accept 风险 | maintainer | DEFER 表 |
| 回滚方案 | maintainer | 版本/feature flag/revert 路径 |
| **Release Ready** | maintainer | 全部勾选 |

### 10.6 验收 → 治理门禁（实现）/ L5（人签）

> **实现回写（2026-07-21）**：PR [#127](https://github.com/xhyperium/infra.rs/pull/127) 门禁与模板；签核草稿 [`releases/2026-07-21-signoff-DRAFT.md`](./releases/2026-07-21-signoff-DRAFT.md)。

- [x] API snapshot / public-api baselines 落盘（`docs/api-baselines/*` + `check-public-api.mjs` / `public-api.yml`）
- [x] 支持矩阵声明（`docs/governance/support-matrix.md`；DEFER-6 Accept 仅 Linux）
- [x] 签核模板 + **DRAFT** 证据指针（Agent 仅填充；**非**已签核）
- [ ] **DEFER-7 Maintainer 手签**（Signed-off-by / GO|NO-GO）— **未完成 · 阻塞 L5 与计划 §15 DONE**

**禁止**：Agent 代签 DEFER-7。

---

## 11. DEFER 处置表（W0 已冻结）

权威全文：[`artifacts/defer-disposition.md`](./artifacts/defer-disposition.md)。

| ID | 项 | 处置 | 波次 | 状态 |
|----|----|------|------|------|
| DEFER-1 | 真实后端验证入口 | **Close**（首批） | W4 | open · 已分类 |
| DEFER-2 | 全 trait 深度语义 | **Close 首批** + **Accept 二期** | W3 | open · 已分类 |
| DEFER-3 | 非 committed DTO | **Close 分批** | W2 | open · 已分类 |
| DEFER-4 | fuzz/oracle/mutants/Miri | **Close** | W1 | open · 已分类 |
| DEFER-5 | API snapshot / semver | **Close** | W5 | open · 已分类 |
| DEFER-6 | 非 Linux 矩阵 | **Accept** 仅 Linux | W0 | **closed as Accept** |
| DEFER-7 | §8 人工签字 | **Defer-with-sign** | W5 | open · 仅人签 |
| DEFER-8 | VenueAdapter override 门禁 | **Close** | W3 | open · 已分类 |

---

## 12. 建议 PR 切片（可并行）

| PR 切片 | 内容 | 依赖 | 风险 |
|---------|------|------|------|
| PR-A | W0 清单 + DEFER 表 + docs/plans 索引 | 无 | 低 |
| PR-B | W1 decimal oracle + 边界 + panicking 门禁 | 无 | 中（dev-dep） |
| PR-C | W1 fuzz/miri/mutants CI | PR-B 可并行 | 中 |
| PR-D | W2 Order wire v1.1 | W0 清单 | 中（消费者） |
| PR-E | W2 Tick/Trade | PR-D 后 | 中 |
| PR-F | W3 语义文档 + Fake suites | W0 | 中 |
| PR-G | W3 Venue override gate + 能力 trait 文档 | PR-F | 低 |
| PR-H | W4 postgres/redis live feature | PR-F | 高（CI 基础设施） |
| PR-I | W4 venue HTTP mock | PR-G | 中 |
| PR-J | W5 API diff + lint 对齐 + 矩阵声明 | 可早开 | 中 |
| PR-K | 签字包 + 对齐文/README 诚实升级 | 全部目标子集 | 低（流程） |

每 PR 必须：独立 worktree、`fmt`/`clippy -D warnings`/相关 test 绿、不宣称整体 Production Ready。

---

## 13. 验证总命令（签字前全量）

```bash
# 单元与静态
cargo test \
  -p contracts -p decimalx -p canonical -p kernel -p testkit \
  --all-targets
cargo clippy \
  -p contracts -p decimalx -p canonical -p kernel -p testkit \
  --all-targets -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS='-D warnings' cargo doc \
  -p contracts -p decimalx -p canonical -p kernel -p testkit \
  --no-deps
cargo test \
  -p contracts -p decimalx -p canonical -p kernel -p testkit \
  --doc

# 专项
RUSTFLAGS='--cfg loom' cargo test -p kernel \
  --test lifecycle_concurrency_loom --release
node scripts/quality-gates/check-canonical-align.mjs
cargo deny check
node scripts/quality-gates/check.mjs

# W1+ 证据（落地后）
# cargo test -p decimalx --features oracle
# cargo fuzz run ...
# 覆盖率门禁（按 crate workflow）

# W4 live（feature / ignore）
# cargo test -p postgresx --features live -- --ignored
```

---

## 14. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| live CI 不稳定/密钥 | W4 拖死 L3 签字 | mock/fixture 优先；live 仅 scheduled |
| wire 升格破坏 adapters | 编译红 | Additive 字段谨慎；先 fixture 后消费者 |
| oracle 与实现算法分歧 | 误判 bug | 文档化中间值溢出合同；oracle 对齐同一合同 |
| 范围膨胀到全部 trait/DTO | 永不签字 | W0 冻结首批；二期 Accept |
| Agent 误标 Production Ready | 治理事故 | PR 模板 + review 检查清单 + DEFER-7 人签 |
| 双平面 VenueAdapter 长期共存 | API 混乱 | W3 定删除条件与日期 |

---

## 15. 完成定义（本计划关闭条件）

本计划 **DONE** 当且仅当：

| # | 条件 | 实现回写状态（2026-07-21） |
|---|------|---------------------------|
| 1 | W0 清单冻结且无未分类 DEFER | ✅ 完成 |
| 2 | decimalx 达 L1（W1 验收勾选） | ✅ 验收项已勾；**L1 候选**（待人在签核包确认） |
| 3 | canonical 本轮 committed 子集达 L2（W2） | ✅ 验收项已勾；清单内 L2 |
| 4 | contracts 首批 trait 达 L3（W3+W4） | ✅ **mock-L3 候选**；真实后端 Accept |
| 5 | kernel/testkit 维持/完成 L1（+ kernel L4 矩阵声明） | ✅ 矩阵 + loom/MSRV 门禁在 |
| 6 | W5 门禁合并；**maintainer 完成 DEFER-7 签字包** | ⚠️ 门禁 ✅ · **人签 ❌** |
| 7 | 审计报告 follow-up 更新；**禁止**签字前改写整体 Production Ready | ✅ 见报告 **§12 post-W5 附录**；整体 PR 仍 **否** |

**计划关闭**：❌ 仍待 DEFER-7。

---

## 16. 与既有文档关系

| 文档 | 关系 |
|------|------|
| [core-crates-production-readiness.md](../report/2026-07-21/core-crates-production-readiness.md) | 输入审计；本计划执行其 §8/§11 |
| [types-ssot-alignment.md](../ssot/types-ssot-alignment.md) | W1/W2 完成后同步 |
| [contracts-ssot-alignment.md](../ssot/contracts-ssot-alignment.md) | W3/W4 完成后同步 |
| [kernel-ssot-alignment.md](../ssot/kernel-ssot-alignment.md) / [testkit-ssot-alignment.md](../ssot/testkit-ssot-alignment.md) | W5 矩阵声明同步 |
| `.agents/ssot/types/canonical/plan/production-upgrade.md` | 上游战役 M1/M3 历史；**本仓以 crates + 本计划为准** |
| `docs/governance/VERSIONING.md` | W5 semver 行为对齐 |

---

## 17. 立即下一步（合入后）

1. **Maintainer**：阅读 [`releases/2026-07-21-signoff-DRAFT.md`](./releases/2026-07-21-signoff-DRAFT.md)，手签正式 signoff（DEFER-7）。
2. 可选 follow-up：真实后端 live feature、二期 trait、非 Linux 矩阵、完整 `cargo fuzz`。
3. 合并后若 public-api baseline 漂移：`node scripts/quality-gates/check-public-api.mjs --update` 并开文档 PR。
4. 任何实现会话：**worktree 隔离**，禁止 main 直改。

---

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | 初版：基于审计报告 §11 DEFER 与 main（含 PR #98）源码事实的生产级修复方案 |
| 2026-07-21 | **W0 Frozen**：`artifacts/` 落盘 trait/wire/支持矩阵/DEFER 处置；beads `infra-asa.1`…`.6` |
| 2026-07-21 | **W0–W5 已合入 main**（PR #120–#128）；签核草稿；DEFER 表更新实现状态 |
| 2026-07-21 | **验收勾选回写**：§5.4–§10.6 按合入证据勾选；§15 状态表；**DEFER-7 仍未签**；审计报告增 §12 |
