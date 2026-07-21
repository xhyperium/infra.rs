# evidence 实现规范

> **SUPERSEDED（正式废止）**  
> 完整生产合同以 [`xhyper-evidence-complete-spec.md`](./xhyper-evidence-complete-spec.md)
> （**SPEC-EVIDENCE-002**，Status: **Approved** 2026-07-14）为准。  
> 本文档仅保留历史六字段 / `EvidenceSink` 原型说明；与 002 冲突时 **002 优先**。  
> 执行计划：[`plan/plan.md`](../plan/plan.md) · 人审：[`plan/approval-packet.md`](../plan/approval-packet.md)。

## 0. 文档定位与裁定边界

本文把 `docs/architecture/spec.md` 中已经批准的 `evidence` 约束整理为可验收的 L0 crate 合同，
不新增架构决策。权威顺序为 `CONSTITUTION.md`、xlib spec、Approved ADR；冲突时按此顺序处理。

本文用以下标签区分确定性：

- **Evidence**：权威文档已明确裁定，可直接实现；
- **Inference**：由已裁定约束唯一或近乎唯一推出，实施前仍应复核；
- **Unknown**：权威文档未裁定，不得在实现中静默选择。

引用只使用稳定章节或 ADR 锚点，不使用行号（xlib spec §0、§9）。

## 1. 目的与非目标

### 1.1 目的

**Evidence**：`evidence` 是 L0 Kernel 中负责“审计留痕、哈希链、不可篡改证据记录”的 crate，
最小公开面为 `EvidenceSink` 与 `EvidenceRecord`，且只依赖 `xlib_standard`（xlib spec §4.1）。
它必须支撑资金、持仓和宏观状态转移的证据记录（xlib spec §7），包括 no-lookahead 状态转移的
成功与失败尝试（ADR-002）。

### 1.2 非目标

- **Evidence**：不承担 tracing span 或指标采集；这些由 observex 或注入的
  `contracts::Instrumentation` 处理（xlib spec §7、ADR-005）。
- **Evidence**：不把 `EvidenceSink` 下沉到 `xlib_standard`（ADR-007）。
- **Evidence**：不承担领域状态机、资金或持仓业务规则；它只记录调用方提供的操作证据
  （xlib spec §4.1、§7）。
- **Inference**：不在本 crate 内引入存储、网络或运行时适配器；其批准依赖只有
  `xlib_standard`（xlib spec §4.1）。
- **Unknown**：持久化后端、序列化协议、签名或远端传输均未获批准，本文不选择实现。

## 2. 位置、依赖与 feature

### 2.1 Workspace 位置

**Evidence**：crate 路径固定为 `crates/xlib/evidence`，独立 `Cargo.toml`、独立三段式版本，
workspace 不使用统一版本号（xlib spec §3）。

### 2.2 依赖边界

生产依赖允许集为：

| 依赖 | 类型 | 依据 |
| --- | --- | --- |
| `xlib_standard` | 普通依赖 | xlib spec §4.1 |

除此之外的生产依赖均为 **Unknown / 未批准**。任何新增依赖必须遵循 CONSTITUTION Article VII，
涉及架构契约时还须按 Article VI 与 xlib spec §9 先获批准。尤其不能为了哈希、序列化或持久化
方便而静默加入第三方库。

**Evidence**：L2.5 可直接依赖 `evidence`；这是 xlib spec §2 R1 与 ADR-007 明确允许的
审计依赖，不应通过 `contracts` 间接绕行。`gate` 也可依赖本 crate（xlib spec §4.1）。

### 2.3 Feature 合同

**Evidence**：所有 trait 必须提供 `mock` feature（CONSTITUTION Article IX）。因此
`EvidenceSink` 的测试替身必须能通过 `mock` feature 获得。

**已裁定最小范围（ADR-010 §6，Proposed）**：`mock` feature（默认关闭）提供
`MockEvidenceSink`——记录每次 `record` 调用的参数副本供断言，`verify_chain` 恒为成功，不维护
真实哈希链。不依赖 `xlib_harness`（本 crate 手写最小实现）。与 `xlib_harness` 未来 `mock!` 宏的
集成方式、是否改用宏生成仍是 **Unknown**，不阻塞当前最小 Article IX 合规。

## 3. 公开 API 合同

### 3.1 已批准的最小公开面（ADR-010 §4，Proposed）

```rust
pub struct EvidenceRecord {
    pub ts: i64,
    pub module: String,
    pub op: String,
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32],
    pub prev_hash: [u8; 32], // 由 sink 在写入时覆盖为真实链尾哈希
}

pub trait EvidenceSink: Send + Sync {
    fn record(&self, record: EvidenceRecord) -> XResult<()>;
    fn verify_chain(&self) -> XResult<()>;
}
```

**Evidence**：公开名称只有 `EvidenceSink` 与 `EvidenceRecord` 被 xlib spec §4.1 明确要求。
ADR-002 还要求调用形态能够表达 `record(EvidenceRecord)`。**已裁定（ADR-010 §4）**：
`record` 签名固定为 `fn record(&self, record: EvidenceRecord) -> XResult<()>`——取此签名的理由
是与已批准的 `EvidenceSink: Send + Sync` 及既有两处调用方（`domain_macro::Timeline::advance`、
`gate::Gate::register`）均以 `?` 传播错误的实际用法一致；这不构成对 E-01 剩余问题（是否应
改为不可失败、失败时 fail-open/fail-closed 语义）的裁定，那仍是 **Unknown**。
`EvidenceRecord::lookahead_violation(...)` / `EvidenceRecord::advance(...)` 这类**领域专用命名
构造器**明确**不**被批准进入本 crate（见 §3.3 讨论）——ADR-002 展示的调用形态只证明"需要能
构造出对应语义的记录"，不证明"构造器必须以领域词汇命名并定居 L0"；本 crate 只提供通用构造器
`EvidenceRecord::new(ts, module, op, input_hash, output_hash)`，领域专用命名留给调用方（如
`domain_macro`）自行包装。

### 3.2 EvidenceRecord 语义字段

**Evidence**：每条记录必须表达下列字段（xlib spec §7）：

| 字段 | 必需语义 | Rust 类型（ADR-010 §4，Proposed） |
| --- | --- | --- |
| `ts` | 记录关联的时间 | `i64`（epoch 毫秒，与 `xlib_standard::Clock` 单位一致；未接入 `Clock`——调用方自行提供 `ts`，见 §6） |
| `module` | 产生记录的模块 | `String` |
| `op` | 被审计操作 | `String` |
| `input_hash` | 输入摘要 | `[u8; 32]` |
| `output_hash` | 输出摘要 | `[u8; 32]` |
| `prev_hash` | 前一记录摘要，用于构成防篡改哈希链 | `[u8; 32]`；调用方构造时的值被 sink 覆盖 |

**已裁定（ADR-010 §3/§4）**：哈希算法为 SHA-256（摘要长度固定 32 字节）；本 crate 额外提供
非强制的 `pub fn hash_bytes(data: &[u8]) -> [u8; 32]` 便利函数，保持调用方与链内部算法一致。
记录自身的链哈希（用于下一条记录的 `prev_hash`）**不**作为 `EvidenceRecord` 的公开字段，
避免与上表六字段混淆；`InMemoryEvidenceSink` 通过 `chain_head()` 单独暴露当前链尾哈希。

字段的可见性（均为 `pub`，已裁定）、编码（内存内 Rust 类型，尚未定义任何持久化/跨进程 wire
格式）与 domain-separation 仍有以下 **Unknown**：算法迁移策略、跨进程/跨版本序列化格式、
是否需要摘要之外的规范编码（字段顺序以外的、面向持久化的 canonical form）。在这些问题裁定前，
不能宣称记录具备跨进程、跨版本或跨实现的可验证兼容性。

### 3.3 必须可表达的调用场景

公开合同最终必须覆盖：

1. **Evidence**：资金操作记录（xlib spec §7）；
2. **Evidence**：持仓操作记录（xlib spec §7）；
3. **Evidence**：宏观状态成功转移记录（xlib spec §7、ADR-002）；
4. **Evidence**：`ts <= last_ts` 的 no-lookahead 失败尝试记录，且调用方随后返回
   `XError::Invalid`（ADR-002）；
5. **Evidence**：受控 checkpoint/restore 中 restore 操作的记录（ADR-002）。

**已裁定（ADR-010 §4）**：场景专用构造器**不**属于本 crate；本 crate 只提供通用构造器
`EvidenceRecord::new(ts, module, op, input_hash, output_hash)`。上述五个场景均通过调用方
（如 `domain_macro`）传入恰当的 `module`/`op` 字符串与预先计算好的 `input_hash`/`output_hash`
来表达，不在 L0 内引入 `lookahead_violation`/`advance`/`restore` 等领域词汇——这与
xlib_standard-spec.md §1.2"公开名称不得提及…domain 类型"的层级中立原则一致。

## 4. 核心不变量

### 4.1 记录覆盖不变量

- **Evidence**：上述资金、持仓、宏观状态转移不得绕过 evidence（xlib spec §7）。
- **Evidence**：no-lookahead 成功与失败两条路径都必须尝试记录，不能只记录成功结果（ADR-002）。
- **Evidence**：restore 本身必须留痕（ADR-002）。

### 4.2 哈希链不变量

- **Evidence**：记录包含 `input_hash`、`output_hash`、`prev_hash`，目标是形成防篡改哈希链
  （xlib spec §7）。
- **Inference**：改变已纳入哈希计算的历史记录必须能够在链校验时被发现，否则"防篡改哈希链"
  的既定目标无法成立。**已裁定（ADR-010 §3/§4）的最小实现**：每条记录的链哈希 =
  `SHA256(prev_hash || ts.to_le_bytes() || module.as_bytes() || op.as_bytes() || input_hash ||
  output_hash)`；`verify_chain` 依此重算并比对，检测篡改/删除/重排（已有测试覆盖，见 §8）。
- **Unknown**：创世记录是否需要专门的 genesis 标识（当前空链链尾用 `[0u8; 32]` 占位，未裁定
  是否需要区分"真正的创世哈希"与"普通全零哈希"）、多链/分叉处理、actor 身份、面向持久化的
  规范编码（跨进程/跨版本）均未裁定（xlib spec §7、§10）。因此实现前必须先补齐这些合同才能
  用于生产持久化或跨进程校验；当前实现仅面向单进程内存场景。

### 4.3 分层不变量

- **Evidence**：本 crate 保持 L0 职责，不依赖 observex、contracts、领域层、L1 或适配器层
  （xlib spec §4.1、§7）。
- **Evidence**：领域层直接使用本 crate 是 R1 的批准例外，不得据此扩大到其他 L0/L1 依赖
  （xlib spec §2 R1、ADR-007）。

## 5. 错误与失败语义

### 5.1 已确定部分

**Evidence**：no-lookahead 失败必须记录证据，并由领域操作返回
共享 `XError`（ADR-002）。ADR 中的 `Invalid("lookahead violation")` 是骨架示例，消息字面量与准确
variant 不构成稳定契约；这也不裁定 `EvidenceSink::record` 的返回类型。

### 5.2 未确定部分（实现阻塞项）

1. **已裁定最小范围（ADR-010 §4）**：`record` 返回 `XResult<()>`——选择依据是与已有两处调用
   方（`domain_macro`、`gate`）均用 `?` 传播错误的现状一致，**不**代表已裁定"记录失败时
   业务应 fail-open 还是 fail-closed"这一更深的安全语义（下一条仍是 Unknown）。
2. 记录失败时，业务操作 fail-closed、fail-open、重试还是进入降级队列——**Unknown**；当前
   `InMemoryEvidenceSink`/`MockEvidenceSink` 仅在锁中毒时返回 `XError::Other`，属于最小实现
   细节，不构成对这一问题的裁定。
3. "先写证据还是先提交状态"的原子性边界——**Unknown**（调用方 `domain_macro::Timeline::advance`
   当前是"先写证据、写成功后才更新/返回"，这是调用方现状，不是本 crate 的公开契约）。
4. 失败记录的 `output_hash` 如何表示——**部分裁定**：`EvidenceRecord::new` 要求调用方总是提供
   一个 `output_hash`（不允许省略），因此失败路径需要调用方自行决定用什么摘要代表"失败输出"
   （如摘要化的错误上下文）；本 crate 不提供"无输出"的哨兵值，具体约定仍是 **Unknown**。
5. sink 部分写入、重复写入、进程崩溃后的恢复与幂等规则——**Unknown**（当前实现是纯内存，
   不涉及崩溃恢复）。
6. 错误是否复用 `XError`——**已裁定**沿用现有 `XError::Other`/`XError::Invalid`；更细的
   variant、source chain/backtrace 政策仍是 **Unknown**。

这些仍未解决的选择会改变资金与状态转移的安全语义，不能靠实现细节默认决定；持久化后端落地前
必须按 xlib spec §9 补充并批准合同。

## 6. 并发、顺序与生命周期

**Evidence**：权威文档只要求哈希链和 `prev_hash`，未裁定完整并发模型。

**已裁定（现状）**：`EvidenceSink` 要求 `Send + Sync`（trait bound 已声明）；
`InMemoryEvidenceSink` 用单个 `RwLock<Vec<StoredRecord>>` 提供单进程内的全局顺序——
写入持锁读取链尾哈希、计算新记录哈希、追加，三步在同一次 `write()` 临界区内完成，
避免"先读链尾、后写入"之间的竞争窗口。

以下仍是 **Unknown / 实现阻塞项**（当前实现不构成对它们的裁定）：

- 是否需要按 actor/module/aggregate 分链，或由调用方提供链上下文（当前是单一全局链）；
- 多进程写入、崩溃一致性、背压与批处理语义（当前纯内存、单进程）；
- sink、链和 actor 的生命周期及关闭/flush 合同；
- 正式的并发正确性证明（loom 等并发模型检查）——当前只有单线程测试覆盖篡改/删除/重排检测，
  未覆盖并发写入下是否会产生无法解释的 `prev_hash` 分叉。

最低验收原则保持不变：并发正确性测试补齐前，不得宣称已验证多线程安全。

## 7. 序列化、哈希、身份与持久化

### 7.1 已确定部分

**Evidence**：记录包含 xlib spec §7 指定的六个语义字段，并以哈希链提供篡改检测目标。
**已裁定（ADR-010 §3/§4）**：哈希算法为 SHA-256，摘要长度 32 字节；链哈希输入顺序固定为
`prev_hash || ts || module || op | input_hash || output_hash`（§4.2）；`verify_chain` 校验
API 已实现（`fn verify_chain(&self) -> XResult<()>`），失败时返回 `XError::Invalid`
（"evidence chain broken" / "evidence hash mismatch"，字面消息不构成稳定契约）。

### 7.2 必须先裁定的开放合同

1. actor 的身份模型与必填性——**Unknown**；
2. genesis 的格式、信任根和多链规则——**Unknown**（当前空链链尾用 `[0u8; 32]` 占位）；
3. 面向持久化/跨进程的规范序列化（不同于 §7.1 已固定的哈希输入顺序，这里指磁盘/网络 wire
   格式的整数/字符串编码、版本字段）——**Unknown**；
4. ~~哈希算法、摘要长度~~——**已裁定**（SHA-256/32 字节，见 §7.1）；域分离和算法迁移策略仍是
   **Unknown**；
5. ~~链完整性校验 API~~——**已裁定**（`verify_chain`，见 §7.1）；范围仍限于单进程内存，跨进程/
   持久化场景的失败报告格式仍是 **Unknown**；
6. 持久化职责、durability、保留期和读取/导出 API——**Unknown**（当前 `InMemoryEvidenceSink`
   提供 `records()`/`chain_head()` 只读访问器，均是内存快照，非持久化导出契约）；
7. 记录 schema 演进及旧记录验证策略——**Unknown**；
8. 敏感输入是否先脱敏、由谁决定、如何证明摘要对应原输入——**Unknown**。

除已裁定项外，其余事项仍被 xlib spec §7、§10 明示为未完成，不能以"当前无未裁定架构问题"的
概括掩盖；它们是 `evidence` 用于生产持久化/跨进程场景前的实现合同阻塞项。

## 8. 测试合同

### 8.1 最小单元与性质测试

**已实现（ADR-010 §4/§6 最小范围内）**：`crates/xlib/evidence/src/lib.rs` 的
`#[cfg(test)] mod tests` 覆盖：

1. ✅ 连续 `record` 后每条记录的 `prev_hash` 指向已接受的前一记录（`record_and_verify_two_records`）；
2. ✅ 篡改任一字段后链校验失败（`tampering_with_stored_record_breaks_chain_verification`）；
3. ✅ 删除中间记录后链校验失败（`deleting_stored_record_breaks_chain_verification`）；重排/
   多字段组合篡改的独立测试仍未补齐；
4. 空链行为已测试（`verify_empty_chain_ok`，链尾为 `[0u8; 32]` 占位），但"正式 genesis 合同"
   仍是 §7.2 的 Unknown，本测试不构成对该合同的批准；
5. `domain_macro` 侧的 advance/lookahead_violation 区分见 §8.2；本 crate 自身不含领域场景
   命名测试（符合 §3.3 的层级中立裁定）；
6. ✅ `mock` feature 下 `MockEvidenceSink` 记录调用并可断言（`mock_evidence_sink_records_calls`）；
7. ❌ 未实现：并发模型的正式验证（loom 等）——见 §6 仍列的 Unknown；
8. ❌ 未实现：sink 故障/部分写入/恢复测试——当前只有纯内存实现，无持久化崩溃场景。

上述 7/8 两项在持久化与正式并发模型裁定前不适用，不构成本次验收缺口。

### 8.2 跨 crate 合同测试

- `domain_macro`：用 `xlib_harness`/mock sink 验证 ADR-002 的成功、失败、restore 三条路径；
- `gate`：若能力注册或解析产生审计记录，具体覆盖须由 gate 合同裁定，本文不预设；
- 真实持久化适配器：待持久化边界获批后，在对应适配器或 testkit 层测试，不把实现依赖拉入 L0。

## 9. CI、发布与兼容性

### 9.1 CI

crate 落地后必须执行 Constitution §4.5 的仓库门禁：

```text
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
node scripts/check.mjs
```

XLib §8 追加：

```text
cargo run -p xtask -- lint-deps
cargo deny check
```

可增加 `cargo test -p evidence --all-features` 等聚焦检查，但不得替代仓库门禁。

**Evidence**：xlib spec §2/§8 仍要求 `lint-deps` 覆盖 R1–R6；`lint-deps` 已通过 `check_r6`
提供源码级 `pub use` 静态扫描（见 [`xtask/xtask-spec.md`](../../xtask/xtask-spec.md)
§4.8）。该扫描是逐行文本匹配的最小实现，已知不处理 glob/别名/分组导入等场景（假阴性风险），
不得把"`lint-deps` 通过"解读为"已穷尽证明 R6 合规"。已知局限的记录/是否拆分独立命令见
[ADR-009](../../../../../docs/architecture/adr/009-r6-enforcement-boundary.md)（Proposed）。

### 9.2 版本规则

**Evidence**：crate 从 `0.1.0` 起步并独立发布，每次仅递增修订位。首个公开发布后的破坏性 API
调整必须记录 CHANGELOG、提供迁移说明并走 RFC；版本号仍仅递增修订位（xlib spec §5）。任何改变
xlib spec 规则、分层或契约清单的变更，无论 crate 版本，均须走 xlib spec §9。

**Inference**：`EvidenceRecord` 的字段类型、编码或哈希覆盖规则一旦用于持久化，即使 Rust API
未变，也可能破坏历史链验证；版本策略必须同时覆盖数据格式兼容性。具体 schema/version 字段仍为
**Unknown**。

`cargo-semver-checks` 对所有已有前一版本 tag 的 crate 生效，但不能证明持久化格式兼容。

## 10. 实现顺序与验收标准

### 10.1 开工门槛（现状）

**已裁定最小范围（ADR-010，Proposed）**：`EvidenceSink::record` 签名、`EvidenceRecord` 的 Rust
类型与构造边界、哈希算法与链校验流程、mock feature 的最小公共面均已裁定并实现（见 §3.1、§3.2、
§7.1、§2.3）。仍未裁定、用于生产持久化/跨进程场景前必须补齐：actor、genesis、面向持久化的
规范序列化、正式并发模型（§6、§7.2）、崩溃恢复（§5.2）。

这与 implementation-plan §4 的 week 1 排期一致，但排期不构成对缺失 API 的批准；ADR-010 待用户
批准后才对本文档具有约束力。

### 10.2 可执行验收标准

实现完成的定义（打勾项为当前已满足；ADR-010 待批准期间视为"实现已就位，待正式批准"）：

- [x] crate 位于 `crates/xlib/evidence`，版本从 `0.1.0` 起，生产依赖仅含已批准的
      `xlib_standard`（加 `sha2`，见 ADR-010 §3）；
- [x] 公开面提供已批准的 `EvidenceSink`、`EvidenceRecord`、`InMemoryEvidenceSink`、
      `MockEvidenceSink`、`hash_bytes`；
- [x] `EvidenceRecord` 表达 xlib spec §7 的六个字段；actor/genesis/version 合同仍未获批（见
      §7.2），本项验收范围明确不含它们；
- [x] 成功状态转移（`domain_macro::Timeline::advance`）与 no-lookahead 失败尝试均有测试证明
      已记录（`domain_macro` 测试）；restore 场景尚未实现（`Timeline` 当前无 checkpoint/restore
      API），不在本次验收范围；
- [x] 篡改、删除测试均令链校验失败；重排测试尚未补充（§8.1 item 3）；
- [ ] sink 故障与并发写入测试——未实现（§8.1 item 7/8，持久化/正式并发模型未裁定前不适用）；
- [x] `mock` feature 可供消费者测试 `EvidenceSink` 调用（`MockEvidenceSink`）；
- [x] §9.1 的仓库门禁全部通过（`cargo build`/`test`/`clippy -D warnings`/`fmt --check`/
      `xtask lint-deps`；`cargo-deny` 未在本次验证环境安装，未执行）；
- [ ] 公共 API 与持久化格式的兼容性变更遵守 §9.2——尚无持久化格式，暂不适用。

## 11. 未解决事项登记

| ID | 未解决事项 | 状态 | 阻塞范围 | 权威依据 |
| --- | --- | --- | --- | --- |
| E-01 | `EvidenceSink` 方法、返回值、对象安全与所有权 | **部分裁定（ADR-010 §4）**：`record(&self, record: EvidenceRecord) -> XResult<()>`，`Send + Sync` | API/错误处理 | xlib spec §4.1；ADR-002 仅为示例 |
| E-02 | `EvidenceRecord` Rust 类型、构造器与可见性 | **已裁定（ADR-010 §4）**：六字段 `pub`，通用构造器 `new()`，不含领域专用构造器 | API | xlib spec §4.1、§7 |
| E-03 | actor 身份模型 | 未裁定 | 格式/审计归因 | xlib spec §7、§10 |
| E-04 | genesis、链标识、分叉与完整性校验 | **部分裁定**：`verify_chain` 已实现；genesis/多链身份未裁定 | 哈希链 | xlib spec §7、§10 |
| E-05 | 规范序列化、哈希算法、域分离与迁移 | **部分裁定（ADR-010 §3）**：哈希算法 SHA-256；持久化级规范序列化、域分离、迁移未裁定 | 跨版本验证 | xlib spec §7 |
| E-06 | 持久化、durability、幂等、重试与崩溃恢复 | 未裁定 | 可靠性 | 未裁定 |
| E-07 | 记录失败时业务 fail-open/fail-closed 与原子性 | 未裁定（`record` 返回 `XResult<()>` 只是接口形态，非安全语义裁定） | 资金/状态安全 | ADR-002 未裁定 sink 失败 |
| E-08 | 并发顺序、`Send + Sync`、背压、flush/close | **部分裁定**：`Send + Sync` 已声明；单锁全局顺序是现状而非正式裁定；背压/flush/close 未裁定 | 并发/生命周期 | 未裁定 |
| E-09 | mock feature 的具体 API 与依赖方式 | **部分裁定（ADR-010 §6）**：`MockEvidenceSink` 记录调用历史；与 `xlib_harness` 集成方式未裁定 | 测试 | CONSTITUTION Article IX 只规定必须存在 |
| E-10 | `XError` variant、source chain/backtrace 政策 | 未裁定（沿用现有 `Other`/`Invalid`，未新增 variant） | 错误兼容性 | implementation-plan 提案未获 spec 批准 |

未标注"已裁定"/"部分裁定"的事项必须保持显式未知，直至按 xlib spec §9 或对应的非架构 API 评审
流程裁定；"部分裁定"项的剩余部分同样不得在后续实现中以默认值、私有约定或新依赖的形式静默
固化。

## 12. 可追溯性摘要

| 契约范围 | 稳定依据锚点 |
| --- | --- |
| L0 职责、路径、公开名与依赖 | XLib spec §§1、3、4.1；ADR-007 |
| 记录字段与必须留痕的操作 | XLib spec §7；ADR-002 |
| `EvidenceRecord`/`EvidenceSink` 六字段与 `record()` 签名回填 | ADR-010（Proposed）§4 |
| 哈希算法（SHA-256）与 `mock` feature 最小范围回填 | ADR-010（Proposed）§3、§6 |
| trait 的 mock feature | Constitution Article IX |
| 测试与仓库门禁 | Constitution §§4.4–4.5；XLib spec §§6、8 |
| 独立版本与契约变更 | Constitution §7.3；XLib spec §§5、9 |
| actor、genesis 与校验缺口 | XLib spec §§7、10 |
| R6 静态扫描（`check_r6`）与已知局限 | XLib spec §§2、8；[`xtask/xtask-spec.md`](../../xtask/xtask-spec.md) §4.8；[ADR-009](../../../../../docs/architecture/adr/009-r6-enforcement-boundary.md)（Proposed） |
