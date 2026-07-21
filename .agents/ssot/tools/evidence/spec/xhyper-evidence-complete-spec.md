# `evidence` 完整规范

```text
Spec ID:          SPEC-EVIDENCE-002
Title:            infra.rs Tamper-Evident Audit Evidence System
Status:           Approved
Target Status:    Stable
Owner:            platform / security
Approved:         2026-07-14 · ZoneCNH（会话「授权审批」· APPR-EVIDENCE-002-v1）
Current Path:     crates/evidence
Adapters Path:    crates/adapters/evidence/{memory,file,postgres,signer}
CLI Path:         tools/evidence-cli
Package:          evidence
Layer:            L0 Audit Core
Publish:          false
Current Version:  0.1.0
Target Version:   0.1.1
Supersedes:       existing evidence-spec.md and the evidence portions of ADR-010
Source Goal:      GOAL-EVIDENCE-AUDIT-TRUST
Note:             Approved ≠ package quality stable；§33 全闭合与 0.1.1 发布见 residual / A9
```

---

## 0. 文档定位

本文件定义 `evidence` 核心 crate 及其持久化、检查点、验证和工具链的完整生产合同。

`evidence` 的目标不是“写一条日志”，而是构建一条可以被独立验证的审计事实链，使系统能够回答：

```text
- 谁或哪个系统组件发起了操作；
- 操作针对什么对象；
- 输入和结果是什么；
- 操作何时发生、何时被记录；
- 记录在链中的确定位置是什么；
- 历史是否被修改、删除、重排、分叉或截尾；
- 记录是否已经达到要求的持久化等级；
- 当前链头是否被可信检查点或独立见证锚定。
```

本文获批后（**Approved 2026-07-14**；实现 cutover 已在 `feat/evidence-002-core-v1` / PR #253 落地）：

- runtime 核心为 `crates/evidence`（`tools/evidence` 已删除，不得回退）；
- 持久化实现位于 `crates/adapters/evidence/*`；
- 命令行工具位于 `tools/evidence-cli`；
- 领域模块不得自行定义 evidence 哈希格式；
- 所有生产记录必须使用本规范定义的 canonical V1；
- 旧六字段内存哈希链仅可作历史迁移输入，不得作为生产终态；
- **不得**将 Approved 误读为 package quality `stable` 或 §33 验收全闭合（见 approval-packet A9 / residual）。

---

# 1. 底层安全模型

## 1.1 Evidence 的准确定位

本系统提供的是：

```text
tamper-evident audit evidence
```

即“可检测篡改的审计证据”。

在没有可信签名检查点和独立外部锚点时，不得使用以下措辞：

```text
不可篡改
绝对可信
无法删除
永久证明
```

单独的哈希链只能检测当前可见记录集合中的部分修改、删除和重排。它不能单独证明：

- 链尾没有被截断；
- 整条链没有被替换；
- 创世链头是真实的；
- 记录来源身份是真实的；
- 摘要对应的原始数据仍然存在。

生产可信度来自以下组合：

```text
canonical encoding
+ domain-separated hashing
+ append-only persistence
+ contiguous sequence
+ idempotent append
+ crash recovery
+ signed checkpoint
+ independent external anchor
+ retained source artifacts
+ reproducible verifier
```

## 1.2 威胁模型

必须防御：

```text
T1  修改历史记录任意字段；
T2  删除中间记录；
T3  重排记录；
T4  重复插入记录；
T5  链尾截断；
T6  整链替换；
T7  并发写入形成分叉；
T8  重试导致重复审计；
T9  崩溃导致半条记录；
T10 磁盘短写、损坏或 fsync 失败；
T11 不同实现使用不同编码得到不同摘要；
T12 使用 Debug/JSON 非规范表示形成不稳定摘要；
T13 伪造 actor、producer 或 operation；
T14 将敏感原文写入 evidence；
T15 测试 mock 或内存实现误接入生产；
T16 旧版本 verifier 无法验证保留期内历史数据；
T17 时钟回拨或记录时间与事件时间混淆；
T18 evidence 写入失败但业务仍报告成功。
```

本规范不声称抵御：

```text
- 所有签名私钥和外部锚点同时被攻破；
- 调用方在摘要前故意提供虚假原始数据；
- 原始输入从未被保留；
- 主机内存和所有受信任执行环境同时被完全控制。
```

## 1.3 信任边界

```text
kernel
  提供错误和时间语义

evidence core
  提供模型、canonical 编码、摘要、封链和纯验证

evidence adapters
  提供持久化、原子追加、恢复和 durability

checkpoint signer
  提供签名和密钥身份

external anchor
  提供独立可信链头

domain/service
  提供规范化输入、操作身份、actor 和 subject

bootstrap
  组装生产实现并禁止 volatile adapter
```

---

# 2. 职责与非职责

## 2.1 Core 职责

`crates/evidence` 必须提供：

```text
- 证据基础值对象；
- EvidenceDraft；
- EvidenceRecordV1；
- ChainHead；
- AppendRequest / AppendReceipt；
- canonical V1 编码和解码；
- SHA-256 域分离摘要；
- genesis digest；
- record digest；
- 纯链验证；
- checkpoint 核心模型和 canonical preimage；
- EvidenceAppender / EvidenceReader 最小合同；
- EvidenceError 及到 XError 的确定映射。
```

## 2.2 Adapter 职责

`crates/adapters/evidence/*` 负责：

```text
- 内存测试实现；
- append-only 文件实现；
- PostgreSQL / transactional outbox 实现；
- durable flush / fsync；
- 单写者或事务级并发控制；
- 崩溃恢复；
- segment rotation；
- retention；
- checkpoint 存储；
- 签名服务接入；
- 独立锚点上传。
```

## 2.3 CLI 职责

`tools/evidence-cli` 负责：

```text
- verify；
- inspect；
- export；
- checkpoint verify；
- chain head 查询；
- golden vector 生成和比对；
- 迁移前后链一致性报告。
```

CLI 默认只读。任何修复或截断动作必须使用单独显式子命令，并输出 Evidence。

## 2.4 非职责

`evidence` 不承担：

```text
- tracing、日志或指标；
- 业务状态机；
- 订单、资金、持仓或宏观规则；
- 自动生成业务 operation 名称；
- 保存原始敏感 payload；
- 密钥托管；
- KMS/Vault 的具体实现；
- 网络传输；
- 业务事务编排；
- 任意 JSON 作为 canonical 格式；
- 替代源数据、数据库备份或事件日志；
- 用摘要证明调用方没有说谎。
```

---

# 3. 目标目录结构

```text
crates/
├── evidence/
│   ├── Cargo.toml
│   ├── README.md
│   ├── AGENTS.md
│   ├── CHANGELOG.md
│   ├── src/
│   │   ├── lib.rs
│   │   ├── error.rs
│   │   ├── ids.rs
│   │   ├── name.rs
│   │   ├── digest.rs
│   │   ├── draft.rs
│   │   ├── record.rs
│   │   ├── canonical/
│   │   │   ├── mod.rs
│   │   │   └── v1.rs
│   │   ├── chain.rs
│   │   ├── checkpoint.rs
│   │   ├── verify.rs
│   │   └── contracts.rs
│   └── tests/
│       ├── golden_vectors.rs
│       ├── canonical_properties.rs
│       ├── chain_properties.rs
│       ├── compile_fail.rs
│       └── fuzz_regressions.rs
│
└── adapters/
    └── evidence/
        ├── memory/
        ├── file/
        └── postgres/

tools/
└── evidence-cli/
```

迁移完成后删除：

```text
tools/evidence
```

运行时 library 禁止放在 `tools/`。

---

# 4. 依赖合同

## 4.1 Core 内部依赖

```text
evidence → kernel
```

不得依赖其他 workspace crate。

禁止依赖：

```text
contracts
canonical
decimalx
observex
configx
resiliencx
schedulex
transportx
bootstrap
domain/*
adapters/*
services/*
apps/*
```

## 4.2 Core 外部依赖白名单

```toml
[dependencies]
kernel = { path = "../kernel" }
sha2 = { workspace = true }
thiserror = { workspace = true }
```

白名单到此为止。

明确禁止：

```text
anyhow
serde
serde_json
bincode
postcard
tokio
async-std
futures
tracing
log
chrono
time
uuid
rand
sqlx
reqwest
```

理由：

- canonical 编码必须由本 crate 显式实现；
- core 不承担 IO；
- ID 生成由调用方或组合层完成；
- 运行时和持久化依赖属于 adapter。

## 4.3 Dev dependencies

允许：

```text
proptest
trybuild
static_assertions
cargo-fuzz 相关测试依赖
```

## 4.4 Features

```toml
[features]
default = []
```

Core 不提供 `mock` feature。

测试替身必须位于：

```text
crates/adapters/evidence/memory
或
crates/testkit
```

此前“每个 trait 必须在自身 crate 提供 mock feature”的约定必须通过 ADR 修订。原因是 feature 会把测试行为带入生产依赖解析，并允许“verify 永远成功”的替身被错误装配。

---

# 5. Crate 级规则

`src/lib.rs` 必须包含：

```rust
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
```

禁止：

```text
unsafe
todo!
unimplemented!
生产代码 panic!
生产代码 unwrap/expect
锁中毒时返回空集合或零摘要
Debug/Display 参与摘要
未带 domain tag 的通用 hash_bytes
公开可变字段破坏 record 不变量
```

---

# 6. 基础值对象

## 6.1 Digest32

```rust
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Digest32([u8; 32]);
```

必须提供：

```rust
impl Digest32 {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self;
    pub const fn as_bytes(&self) -> &[u8; 32];
    pub const fn into_bytes(self) -> [u8; 32];
}
```

禁止：

```text
Default
全零值作为“无摘要”哨兵
从任意字符串隐式解析
Display 输出原始二进制
```

可提供固定长度小写 hex 编解码，但 hex 不是 canonical record 编码。

缺失摘要必须使用 `Option<Digest32>`，不得使用 `[0; 32]` 表达缺失。

## 6.2 ChainId

```rust
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct ChainId([u8; 32]);
```

`ChainId` 是逻辑审计流的稳定身份。

必须提供：

```rust
impl ChainId {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self;
    pub const fn as_bytes(&self) -> &[u8; 32];

    pub fn derive(
        namespace: &EvidenceName,
        subject_digest: Digest32,
    ) -> Self;
}
```

推导规则：

```text
SHA256(
  "XHYPER:EVIDENCE:CHAIN-ID:V1\0"
  || u16_be(namespace_len)
  || namespace_bytes
  || subject_digest
)
```

禁止随机生成策略进入 core。

链划分策略由 evidence-policy 声明（monorepo 历史路径：`.architecture/evidence-policy.toml`）。
**infra.rs**：本仓不维护 `.architecture`；本仓 evidence 以 `crates/evidence` 最小面为准，不以 archgate / 该 policy 路径为验收条件。

## 6.3 EventId

```rust
pub struct EventId([u8; 32]);
```

语义：

- 单条 evidence event 的幂等身份；
- 同一链内必须唯一；
- 重试同一事件必须复用同一 `EventId`；
- 不同事件不得复用。

## 6.4 OperationId

```rust
pub struct OperationId([u8; 32]);
```

语义：

- 一次逻辑操作的相关记录分组；
- `Attempted`、`Committed`、`Failed` 等记录可共享同一 `OperationId`；
- 每条记录仍使用不同 `EventId`。

## 6.5 EvidenceName

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EvidenceName(Box<str>);
```

允许格式：

```regex
[a-z0-9][a-z0-9._:/-]{0,127}
```

要求：

```text
- ASCII；
- 1..=128 bytes；
- 小写；
- 无前后空白；
- 无 Unicode 规范化问题；
- 不允许秘密、token、邮箱、姓名或原始用户 ID。
```

用于：

```text
producer
actor namespace
operation
chain namespace
signer key ID
```

非法名称返回 `EvidenceError::InvalidName`。

---

# 7. Actor 与 Subject

## 7.1 EvidenceActor

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceActor {
    namespace: EvidenceName,
    id_digest: Digest32,
}
```

语义：

```text
namespace:
  system
  service
  user
  external
  scheduler
  migration
  operator

id_digest:
  对规范化 actor identity 做域分离摘要
```

禁止在 record 中保存：

```text
API key
token
密码
邮箱
手机号
用户名原文
IP 原文
交易所 secret
个人敏感数据
```

## 7.2 Subject

record 使用：

```rust
subject_digest: Digest32
```

表示被操作的逻辑对象，例如：

```text
account
portfolio
position
order
market series
macro series
regime state
configuration snapshot
```

subject 的规范化规则由领域层定义并版本化。

---

# 8. EvidenceDraft

## 8.1 公开模型

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceDraft {
    event_id: EventId,
    operation_id: OperationId,
    producer: EvidenceName,
    actor: EvidenceActor,
    subject_digest: Digest32,
    operation: EvidenceName,
    event_time: Option<Timestamp>,
    input_digest: Digest32,
    outcome: EvidenceOutcome,
    metadata_digest: Option<Digest32>,
    correlation_id: Option<Digest32>,
}
```

字段必须私有。

## 8.2 EvidenceOutcome

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceOutcome {
    Attempted,
    Committed {
        output_digest: Digest32,
    },
    Rejected {
        reason_digest: Digest32,
    },
    Failed {
        error_digest: Digest32,
    },
    Cancelled {
        reason_digest: Digest32,
    },
    Compensated {
        result_digest: Digest32,
    },
}
```

语义：

```text
Attempted:
  操作即将产生外部或不可逆副作用。

Committed:
  状态变化或外部操作已确认成功。

Rejected:
  操作因输入、规则或前置条件被拒绝，未提交目标状态。

Failed:
  操作执行失败。

Cancelled:
  操作被明确取消。

Compensated:
  已提交操作执行了补偿行为。
```

禁止使用自由字符串代替 outcome 类型。

## 8.3 构造 API

必须提供：

```rust
impl EvidenceDraft {
    pub fn new(
        event_id: EventId,
        operation_id: OperationId,
        producer: EvidenceName,
        actor: EvidenceActor,
        subject_digest: Digest32,
        operation: EvidenceName,
        input_digest: Digest32,
        outcome: EvidenceOutcome,
    ) -> Self;

    pub fn with_event_time(
        self,
        event_time: Timestamp,
    ) -> Self;

    pub fn with_metadata_digest(
        self,
        metadata_digest: Digest32,
    ) -> Self;

    pub fn with_correlation_id(
        self,
        correlation_id: Digest32,
    ) -> Self;
}
```

调用方不得提供：

```text
recorded_at
sequence
previous_digest
record_digest
schema_version
```

这些字段由 appender 和 core 封链逻辑产生。

---

# 9. EvidenceRecordV1

## 9.1 公开模型

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRecordV1 {
    schema_version: u16,
    chain_id: ChainId,
    sequence: u64,
    recorded_at: Timestamp,

    event_id: EventId,
    operation_id: OperationId,
    producer: EvidenceName,
    actor: EvidenceActor,
    subject_digest: Digest32,
    operation: EvidenceName,
    event_time: Option<Timestamp>,
    input_digest: Digest32,
    outcome: EvidenceOutcome,
    metadata_digest: Option<Digest32>,
    correlation_id: Option<Digest32>,

    previous_digest: Digest32,
    record_digest: Digest32,
}
```

所有字段私有，只提供只读 getter。

## 9.2 不变量

```text
schema_version == 1
sequence >= 1
recorded_at 来自注入 Clock
第一条记录 previous_digest == genesis_digest(chain_id)
后续记录 previous_digest == 上一条 record_digest
record_digest == canonical preimage 的 SHA-256
event_id 在同一 chain 内唯一
sequence 在同一 chain 内连续且唯一
record 不可被调用方修改
```

## 9.3 封链函数

```rust
pub fn seal_record_v1(
    chain_id: ChainId,
    sequence: u64,
    recorded_at: Timestamp,
    previous_digest: Digest32,
    draft: EvidenceDraft,
) -> Result<EvidenceRecordV1, EvidenceError>;
```

只有 adapter/appender 可以决定 sequence 和 previous digest。

不得公开允许调用方任意填写 `record_digest` 的构造器。

---

# 10. 时间语义

## 10.1 recorded_at

```text
recorded_at:
  evidence appender 接受并封链该记录的时间。
```

要求：

- 由 appender 注入的 `kernel::Clock` 生成；
- Unix 纳秒；
- Clock 失败时 append 失败；
- 不得返回 0；
- 不得由调用方提供；
- 不要求记录间严格递增，因为墙钟可以回拨。

## 10.2 event_time

```text
event_time:
  被审计业务事件自身的时间。
```

要求：

- 由调用方提供；
- 可为空；
- 不替代 `recorded_at`；
- 不参与 sequence 分配；
- no-lookahead 规则使用 event_time 或领域时间，不使用 recorded_at。

当前单一 `ts` 字段必须迁移为上述两个字段。

---

# 11. Canonical V1 编码

## 11.1 总体规则

canonical 编码用于：

```text
record digest
跨实现验证
磁盘持久化 payload
golden vectors
签名 checkpoint
```

它不是：

```text
Debug
Display
JSON
serde 自动派生
数据库行的任意字段拼接
```

所有整数使用大端序。

所有可选字段使用显式 presence byte：

```text
0x00 = absent
0x01 = present
其他值非法
```

所有变长字段必须带长度前缀。

## 11.2 Record preimage

固定 domain tag：

```text
ASCII "XHYPER:EVIDENCE:RECORD:V1\0"
```

preimage 字段顺序：

```text
1.  domain_tag                     fixed bytes
2.  schema_version                 u16_be, value 1
3.  chain_id                       32 bytes
4.  sequence                       u64_be
5.  recorded_at_unix_nanos         i64_be

6.  event_id                       32 bytes
7.  operation_id                   32 bytes

8.  producer_len                   u16_be
9.  producer                       producer_len bytes

10. actor_namespace_len            u16_be
11. actor_namespace                actor_namespace_len bytes
12. actor_id_digest                32 bytes

13. subject_digest                 32 bytes

14. operation_len                  u16_be
15. operation                      operation_len bytes

16. event_time_present             u8
17. event_time_unix_nanos          i64_be when present

18. input_digest                   32 bytes

19. outcome_tag                    u8
20. outcome_digest                 32 bytes except Attempted

21. metadata_digest_present        u8
22. metadata_digest                32 bytes when present

23. correlation_id_present         u8
24. correlation_id                 32 bytes when present

25. previous_digest                32 bytes
```

Outcome tags：

```text
0x00 Attempted
0x01 Committed
0x02 Rejected
0x03 Failed
0x04 Cancelled
0x05 Compensated
```

## 11.3 record_digest

```text
record_digest = SHA256(record_preimage)
```

持久化 canonical record bytes：

```text
record_preimage || record_digest
```

`record_digest` 不再次参与自身摘要。

## 11.4 Genesis digest

```text
genesis_digest(chain_id) =
  SHA256(
    "XHYPER:EVIDENCE:GENESIS:V1\0"
    || chain_id
  )
```

禁止使用全零摘要作为 genesis。

## 11.5 内容摘要

禁止提供：

```rust
pub fn hash_bytes(data: &[u8]) -> Digest32;
```

必须提供带域分离的 API：

```rust
pub fn digest_canonical(
    domain: &EvidenceName,
    canonical_bytes: &[u8],
) -> Digest32;
```

输入：

```text
"XHYPER:EVIDENCE:CONTENT:V1\0"
|| u16_be(domain_len)
|| domain
|| u64_be(content_len)
|| canonical_bytes
```

领域层必须为输入、输出、错误和 metadata 使用不同 domain。

示例：

```text
domain_macro.point.v1
domain_macro.state.v1
domain_macro.error.v1
order.request.v1
order.response.v1
```

## 11.6 编码边界

下列两组输入必须产生不同 preimage 和 digest：

```text
producer="ab", operation="c"
producer="a",  operation="bc"
```

必须有 golden test 永久覆盖该边界。

## 11.7 Decoder

decoder 必须：

```text
- 拒绝未知 presence byte；
- 拒绝未知 outcome tag；
- 拒绝超长名称；
- 拒绝无效 ASCII；
- 拒绝 trailing bytes；
- 拒绝 truncated input；
- 拒绝 schema version 不支持；
- 不分配超出配置上限的内存；
- 对任意输入不 panic。
```

---

# 12. Chain 语义

## 12.1 ChainHead

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainHead {
    chain_id: ChainId,
    sequence: u64,
    digest: Digest32,
}
```

空链表示使用：

```rust
Option<ChainHead>
```

不得使用 sequence 0 + 零摘要模拟空链。

## 12.2 顺序

```text
首条记录 sequence = 1
下一条记录 sequence = previous.sequence + 1
sequence 溢出时拒绝 append 并冻结链
```

## 12.3 单链一致性

一个 ChainId 在任何时刻必须满足：

```text
- 逻辑单写顺序；
- 不允许两个不同记录占用同一 sequence；
- 不允许两个不同 record_digest 继承同一 head；
- append 必须线性化；
- 跨进程 writer 必须使用事务锁或 CAS。
```

## 12.4 分叉

检测到分叉时：

```text
- 立即停止该链写入；
- 返回 EvidenceError::ForkDetected；
- 进入隔离状态；
- 生成安全事件；
- 禁止自动选择“较长链”；
- 需要人工或批准的恢复流程。
```

## 12.5 链划分

禁止默认使用“全系统单一全局链”。

链策略必须按审计对象划分，例如：

```text
资金/持仓:
  per account or portfolio

订单:
  per account + venue partition

宏观数据:
  per provider + series or controlled partition

行情状态:
  per venue + instrument partition

配置与迁移:
  per environment + subsystem
```

每条策略必须登记：

```text
namespace
subject derivation
expected write rate
writer ownership
retention
checkpoint frequency
criticality
```

---

# 13. Append 合同

## 13.1 Durability

```rust
#[non_exhaustive]
pub enum Durability {
    Volatile,
    Process,
    Durable,
}
```

定义：

```text
Volatile:
  仅内存；进程退出丢失。

Process:
  已写入 OS/page cache，但未承诺掉电后存在。

Durable:
  adapter 已完成其合同要求的事务提交或 fsync，
  append 成功返回后，正常崩溃恢复必须可读取。
```

生产审计操作默认要求 `Durable`。

## 13.2 AppendRequest

```rust
pub struct AppendRequest {
    chain_id: ChainId,
    draft: EvidenceDraft,
    expected_head: Option<ChainHead>,
    required_durability: Durability,
}
```

`expected_head`：

```text
Some(head):
  CAS append；当前 head 不一致则 Conflict。

None:
  adapter 仍必须内部线性化，不能因此允许分叉。
```

## 13.3 AppendReceipt

```rust
pub struct AppendReceipt {
    event_id: EventId,
    head: ChainHead,
    durability: Durability,
    recorded_at: Timestamp,
}
```

只有达到 `required_durability` 才能返回成功。

## 13.4 EvidenceAppender

```rust
pub trait EvidenceAppender: Send + Sync {
    fn append(
        &self,
        request: AppendRequest,
    ) -> Result<AppendReceipt, EvidenceError>;
}
```

合同：

```text
- 验证 draft；
- 检查 event_id 幂等；
- 获取 Clock recorded_at；
- 原子读取 head；
- 分配 sequence；
- 计算 previous_digest；
- seal record；
- 持久化；
- 更新 head；
- 达到 durability；
- 返回 receipt。
```

## 13.5 幂等

同一 chain + event_id 重试：

### 内容完全相同

返回原始 `AppendReceipt`，不得追加第二条记录。

### 内容不同

返回：

```text
EvidenceError::IdempotencyConflict
```

并冻结相关业务操作，禁止覆盖旧记录。

## 13.6 Head CAS

当 `expected_head` 与实际 head 不一致：

```text
- 不追加；
- 不分配 sequence；
- 不产生部分记录；
- 返回 EvidenceError::HeadConflict。
```

## 13.7 Fail-closed

所有被 evidence-policy 标记为 `required` 的操作（monorepo 历史：`.architecture/evidence-policy.toml`；本仓不维护该路径）：

```text
evidence append 未成功达到要求 durability
→ 业务操作不得报告成功
```

不存在 `best_effort evidence`。

仅用于可观测性、调试或统计的数据不应使用 evidence；应使用 observex。

---

# 14. Reader 合同

```rust
pub trait EvidenceReader: Send + Sync {
    fn head(
        &self,
        chain_id: ChainId,
    ) -> Result<Option<ChainHead>, EvidenceError>;

    fn get_by_event_id(
        &self,
        chain_id: ChainId,
        event_id: EventId,
    ) -> Result<Option<EvidenceRecordV1>, EvidenceError>;

    fn read_range(
        &self,
        chain_id: ChainId,
        start_sequence: u64,
        limit: u32,
    ) -> Result<Vec<EvidenceRecordV1>, EvidenceError>;
}
```

限制：

```text
1 <= limit <= 10_000
start_sequence >= 1
返回顺序严格递增
不得静默跳过损坏记录
损坏必须返回错误
```

大规模流式读取可由 adapter 提供额外 API，但不能改变 core 合同。

---

# 15. 原子性合同

## 15.1 持久化业务状态

资金、持仓、账本、订单状态等持久化变化必须满足以下之一：

```text
A. 业务状态和 evidence record 在同一数据库事务提交；
B. 业务状态和 transactional outbox 在同一数据库事务提交；
C. evidence log 本身是状态的 source of truth。
```

禁止：

```text
先提交业务状态
→ 再调用远端 evidence
→ 失败后仍报告成功
```

## 15.2 Transactional outbox

PostgreSQL adapter 必须支持：

```text
business transaction
  INSERT/UPDATE business state
  INSERT evidence outbox row
COMMIT
```

dispatcher 将 outbox 转换为链记录时：

- 必须幂等；
- 使用稳定 `EventId`；
- 成功后标记 dispatched；
- 崩溃重试不得重复；
- outbox 不得在 evidence durable 前删除。

## 15.3 外部不可逆副作用

交易所下单、撤单、转账等无法与本地数据库形成单一事务。

必须使用至少两条 evidence：

```text
Attempted
  在发送外部命令前 durable 记录。

Committed / Rejected / Failed / Cancelled
  在得到结果后记录。
```

两条记录共享 `OperationId`，使用不同 `EventId`。

外部请求必须携带幂等 client order ID 或等价键。

## 15.4 纯内存状态

若状态是可重建、非资金关键且提交赋值不会失败：

```text
1. 计算 next_state；
2. 构造 Committed evidence；
3. durable append；
4. 原子替换内存 state。
```

若第 4 步可能失败，不得使用该模式，必须使用 outbox 或 event-sourced 设计。

## 15.5 Rejected 路径

no-lookahead、风险规则拒绝、非法状态转换等路径必须记录：

```text
EvidenceOutcome::Rejected
```

只有 evidence append 成功后才返回对应业务拒绝结果。

若 evidence append 失败，返回 evidence/storage failure，不能伪装成普通业务拒绝。

---

# 16. Checkpoint 与外部锚点

## 16.1 CheckpointV1

```rust
pub struct CheckpointV1 {
    schema_version: u16,
    chain_id: ChainId,
    sequence: u64,
    head_digest: Digest32,
    created_at: Timestamp,
    checkpoint_digest: Digest32,
}
```

checkpoint preimage：

```text
"XHYPER:EVIDENCE:CHECKPOINT:V1\0"
|| u16_be(schema_version)
|| chain_id
|| u64_be(sequence)
|| head_digest
|| i64_be(created_at_unix_nanos)
```

```text
checkpoint_digest = SHA256(checkpoint_preimage)
```

## 16.2 SignedCheckpointV1

```rust
pub struct SignedCheckpointV1 {
    checkpoint: CheckpointV1,
    signer_key_id: EvidenceName,
    signature_algorithm: SignatureAlgorithm,
    signature: Box<[u8]>,
}
```

V1 支持：

```text
Ed25519
外部 KMS/Vault 签名适配器
```

私钥不得进入 `evidence` core、配置文件、镜像或仓库。

## 16.3 CheckpointSigner

签名属于 adapter 合同：

```rust
pub trait CheckpointSigner: Send + Sync {
    fn sign(
        &self,
        checkpoint: &CheckpointV1,
    ) -> Result<SignedCheckpointV1, EvidenceError>;
}
```

## 16.4 CheckpointVerifier

```rust
pub trait CheckpointVerifier: Send + Sync {
    fn verify(
        &self,
        checkpoint: &SignedCheckpointV1,
    ) -> Result<(), EvidenceError>;
}
```

## 16.5 频率

默认生产上限：

```text
每 10,000 条记录
或
每 60 秒
取先到者
```

资金、持仓、订单链可配置更严格策略。

超过策略未生成 checkpoint：

```text
- 健康状态降级；
- 告警；
- 超过 hard deadline 后阻止 required chain 继续写入。
```

## 16.6 独立锚点

签名 checkpoint 必须复制到至少一个独立信任域：

```text
- 启用 Object Lock/WORM 的 OSS/S3；
- 独立审计数据库；
- 独立安全服务；
- 受控 release/evidence 存储。
```

本地同一磁盘副本不算独立锚点。

## 16.7 尾部截断检测

启动和定期验证必须：

```text
- 获取最新可信 signed checkpoint；
- 验证签名；
- 本地链 sequence 不得小于 checkpoint.sequence；
- checkpoint 位置 record_digest 必须等于 head_digest；
- 从 checkpoint 到当前 head 验证连续性。
```

本地链短于可信 checkpoint：

```text
EvidenceError::TailTruncated
```

禁止自动把 checkpoint 降级到较短链。

---

# 17. 纯验证合同

## 17.1 VerificationReport

```rust
pub struct VerificationReport {
    chain_id: ChainId,
    records_checked: u64,
    first_sequence: Option<u64>,
    last_sequence: Option<u64>,
    computed_head: Option<ChainHead>,
    checkpoint_sequence: Option<u64>,
    valid: bool,
}
```

## 17.2 验证内容

verifier 必须检测：

```text
- 非法 canonical 编码；
- schema version 不支持；
- chain_id 混入；
- sequence 从非 1 开始；
- sequence gap；
- sequence duplicate；
- previous_digest mismatch；
- record_digest mismatch；
- event_id duplicate；
- record reorder；
- 中间删除；
- 字段篡改；
- checkpoint mismatch；
- signature invalid；
- tail truncation；
- unexpected trailing bytes；
- 不符合 EvidenceName 规则；
- outcome tag 与 digest 不一致。
```

## 17.3 验证输入

验证器必须支持：

```text
- 从 genesis 全链验证；
- 从可信 checkpoint 继续验证；
- 指定 sequence 范围验证；
- 只读，不修改原始数据。
```

## 17.4 验证失败

任何结构或摘要错误都必须：

```text
- 返回确定的 EvidenceError；
- 报告 chain_id 和 sequence；
- 不输出敏感 payload；
- 不继续把损坏后的记录计为可信；
- 不自动修复。
```

---

# 18. EvidenceError

## 18.1 错误集合

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    InvalidName,
    InvalidDraft,
    InvalidEncoding,
    UnsupportedVersion,
    MissingChain,
    MissingRecord,
    IdempotencyConflict,
    HeadConflict,
    SequenceOverflow,
    SequenceGap,
    DuplicateSequence,
    DuplicateEventId,
    ChainIdMismatch,
    PreviousDigestMismatch,
    RecordDigestMismatch,
    ForkDetected,
    CheckpointMismatch,
    SignatureInvalid,
    TailTruncated,
    CorruptStorage,
    StorageUnavailable,
    DurabilityFailure,
    ClockUnavailable,
    SynchronizationFailure,
}
```

真实实现可使用结构化字段补充：

```text
chain_id
sequence
expected digest
actual digest
static context
source
```

不得把原始业务输入放入错误上下文。

## 18.2 到 XError 的映射

```text
InvalidName
InvalidDraft
InvalidEncoding
  → XError::Invalid

MissingChain
MissingRecord
  → XError::Missing

IdempotencyConflict
HeadConflict
DuplicateSequence
DuplicateEventId
  → XError::Conflict

StorageUnavailable
DurabilityFailure
ClockUnavailable
UnsupportedVersion
  → XError::Unavailable

SequenceOverflow
SequenceGap
ChainIdMismatch
PreviousDigestMismatch
RecordDigestMismatch
ForkDetected
CheckpointMismatch
SignatureInvalid
TailTruncated
CorruptStorage
  → XError::Invariant

SynchronizationFailure
  → XError::Internal
```

链损坏绝不映射为普通 `Invalid`。

---

# 19. Memory Adapter

## 19.1 定位

```text
package: evidence_memory
path: crates/adapters/evidence/memory
production_allowed: false
```

用途：

```text
- 单元测试；
- 合同测试；
- 本地开发；
- 故障注入。
```

## 19.2 能力

必须真实实现：

```text
- canonical seal；
- sequence；
- previous digest；
- event_id 幂等；
- head CAS；
- verify；
- 并发线性化。
```

禁止：

```text
verify 永远成功
锁中毒返回空 Vec
chain head 返回零摘要
声称 Durable
```

最高 durability：

```text
Volatile
```

请求 `Durable` 时必须返回 `DurabilityFailure`。

## 19.3 生产阻断

bootstrap 与 release gate 必须阻止（monorepo 历史另有 archgate 机控；**infra.rs 不移植 archgate**，不以 `cargo run -p archgate` 为验收）：

```text
evidence_memory
```

进入 production feature、release binary 或 systemd 部署清单。

---

# 20. File Adapter

## 20.1 定位

```text
package: evidence_file
path: crates/adapters/evidence/file
```

适用于：

```text
- 单机 append-only 审计；
- 独立于业务数据库的 evidence 链；
- 本地 durable buffer；
- checkpoint 前的可靠存储。
```

## 20.2 单写者

每个 ChainId 必须：

```text
- 单进程独占 writer lock；
- 多 reader；
- 第二 writer 启动失败；
- lock 不能仅依赖进程内 Mutex。
```

## 20.3 Segment 格式

每个 segment 包含：

```text
header:
  magic
  format_version
  chain_id
  first_sequence
  previous_segment_head
  created_at

frames:
  u32_be(payload_len)
  canonical_record_bytes
  u32_be(payload_len)
  fixed commit marker

footer when sealed:
  final_sequence
  final_head
  segment_digest
```

要求：

```text
- payload_len 有硬上限；
- 前后 length 必须相同；
- commit marker 完整后记录才算提交；
- record digest 检测内容损坏；
- segment digest 检测 segment 级修改；
- 未完成尾帧不得被当作记录。
```

## 20.4 Durability

`Durable` append 返回前必须：

```text
write full frame
→ fdatasync/fsync data
→ 必要时同步目录元数据
→ 更新 durable head
→ 返回 receipt
```

允许 group commit，但：

```text
- 调用方请求 Durable 时必须等待 group fsync；
- 不得在 fsync 前返回 Durable；
- batch 中任一失败必须明确报告。
```

## 20.5 恢复

启动时：

```text
1. 获取 writer lock；
2. 读取最新可信 checkpoint；
3. 扫描 segment；
4. 验证 header；
5. 验证所有完整 frame；
6. 检测不完整尾帧；
7. 验证 sequence 和 digest；
8. 对照 checkpoint；
9. 建立 head；
10. 才允许写入。
```

仅允许自动截断：

```text
没有 commit marker 的最后一个不完整 frame
且
其 sequence 高于最新 durable committed record
且
不违反可信 checkpoint
```

任何已提交 frame 损坏：

```text
进入 quarantine，禁止写入。
```

## 20.6 Segment rotation

默认：

```text
64 MiB
或
1,000,000 records
取先到者
```

rotation 必须：

```text
- seal footer；
- fsync；
- 创建新 segment；
- 新 segment header 引用旧 segment final head；
- 不允许跨 segment chain 断裂。
```

---

# 21. PostgreSQL / Outbox Adapter

## 21.1 定位

```text
package: evidence_postgres
path: crates/adapters/evidence/postgres
```

主要用于：

```text
- 资金；
- 持仓；
- 账本；
- 订单状态；
- 与业务状态同事务的 required evidence。
```

## 21.2 表级不变量

至少需要：

```text
evidence_chain_heads
evidence_records
evidence_outbox
evidence_checkpoints
```

唯一约束：

```text
(chain_id, sequence)
(chain_id, event_id)
```

chain head 行必须保存：

```text
chain_id
sequence
head_digest
updated_at
```

## 21.3 Append 事务

直接 append：

```text
BEGIN
SELECT chain head FOR UPDATE
check event_id
check expected head
allocate sequence
seal record
INSERT record
UPDATE chain head
COMMIT
```

失败必须整体回滚。

## 21.4 Outbox 事务

业务操作：

```text
BEGIN
update business state
insert evidence_outbox with stable event_id
COMMIT
```

dispatcher：

```text
claim outbox row
append idempotently
mark dispatched
```

禁止在 evidence append durable 前删除 outbox。

## 21.5 并发

必须通过：

```text
多进程并发 append
同 event_id 并发重试
同 expected head 竞争
事务 rollback
连接断开
deadlock retry
```

不允许产生 sequence gap 或 fork。

---

# 22. 数据保留与隐私

## 22.1 Evidence record

record 只保存：

```text
标识
时间
受控名称
摘要
链字段
```

不保存原始业务 payload。

## 22.2 原始 artifact

需要重放或独立证明时，原始 canonical artifact 必须存入独立内容寻址存储：

```text
object key = digest
```

该 artifact store 必须有：

```text
retention
访问控制
加密
完整性校验
删除审批
必要时 Object Lock
```

## 22.3 Retention

保留策略由链策略登记，至少包含：

```text
record retention
checkpoint retention
signing public key retention
canonical schema retention
source artifact retention
verifier binary/source retention
```

只保留 record 而删除 verifier/schema，视为证据不可验证。

## 22.4 删除

审计保留期内禁止物理删除 committed record。

法律或隐私要求必须删除源数据时：

```text
- evidence record 保留摘要；
- 记录 deletion/erasure evidence；
- 保存审批和策略版本；
- 不修改历史 record。
```

---

# 23. Schema 与算法演进

## 23.1 V1 冻结

一旦 V1 进入 production：

```text
- 字段顺序永久不变；
- 字节序永久不变；
- tag 永久不变；
- name 规则永久不变；
- outcome tag 永久不重用；
- SHA-256 规则永久不变。
```

任何变化必须定义：

```text
EvidenceRecordV2
new domain tag
new golden vectors
migration/dual-read policy
```

## 23.2 Reader 兼容性

保留期内：

```text
reader/verifier 必须支持所有仍被保留的版本。
```

不能因为升级 writer 而失去旧链验证能力。

## 23.3 算法迁移

哈希算法迁移不得原地改变 V1。

必须使用：

```text
新 schema version
或
明确 algorithm field + 新 domain tag
```

迁移 checkpoint 必须同时锚定：

```text
旧链最终 head
新链 genesis/initial head
迁移原因
批准信息
```

---

# 24. 测试合同

## 24.1 Golden vectors

仓库必须提交：

```text
tests/vectors/evidence-v1/
```

至少包括：

```text
empty/genesis
attempted
committed
rejected
failed
cancelled
compensated
all optional fields present
all optional fields absent
max name length
event_time before epoch
timestamp boundaries
multi-record chain
checkpoint
```

每个 vector 包含：

```text
semantic input
canonical hex
record digest
expected chain head
```

任何语言或 adapter 实现必须得到相同结果。

## 24.2 Canonical properties

必须覆盖：

```text
encode → decode → encode 稳定
不同语义值不会因字段边界得到相同 preimage
trailing bytes 拒绝
invalid tag 拒绝
invalid length 拒绝
truncation 拒绝
任意输入不 panic
```

强制回归：

```text
("ab", "c") != ("a", "bc")
```

## 24.3 Chain properties

必须覆盖：

```text
修改每一个字段 → verify 失败
删除中间记录 → 失败
重排 → 失败
重复 sequence → 失败
sequence gap → 失败
不同 chain_id 混入 → 失败
错误 previous digest → 失败
错误 record digest → 失败
event_id duplicate → 失败
```

## 24.4 Checkpoint properties

必须覆盖：

```text
修改 checkpoint 任意字段 → signature/digest 失败
本地链短于 checkpoint → TailTruncated
链头与 checkpoint 不一致 → CheckpointMismatch
旧 key 验证
key rotation
未知 key ID
无效 signature
```

## 24.5 Idempotency

必须覆盖：

```text
相同 event_id + 相同内容 → 返回相同 receipt
相同 event_id + 不同内容 → IdempotencyConflict
并发相同 event_id → 只产生一条 record
```

## 24.6 并发

每个 adapter 必须运行统一 conformance suite：

```text
1000+ concurrent append
多个 chain 并行
同 chain 顺序连续
无 fork
无 duplicate sequence
无 lost append
```

需要模型测试或数据库事务级证明。

## 24.7 Crash / Fault Injection

File adapter：

```text
kill -9 before write
kill -9 mid-frame
kill -9 after frame before fsync
kill -9 after fsync before receipt
disk full
permission denied
short write
fsync error
corrupt final frame
corrupt committed middle frame
```

Postgres adapter：

```text
connection drop
transaction rollback
deadlock
serialization failure
dispatcher crash
outbox replay
commit response lost
```

## 24.8 Fuzz

必须 fuzz：

```text
canonical decoder
record verifier
segment parser
checkpoint parser
CLI import
```

每个发现的 crash 必须沉淀为 regression corpus。

## 24.9 Coverage

```text
evidence core:
  line >= 95%
  branch >= 90%

production adapters:
  line >= 90%
  critical recovery paths = 100% scenario coverage
```

## 24.10 Mutation testing

Core mutation score：

```text
>= 90%
```

不得存活：

```text
字段顺序变化
长度前缀删除
字节序反转
domain tag 删除
previous digest 不参与 hash
sequence 不参与 hash
event_id 幂等检查删除
verify 比较反转
checkpoint 对比删除
错误映射降级
```

## 24.11 Miri

Core 和 memory adapter 定期执行：

```text
cargo miri test
```

---

# 25. CLI 合同

## 25.1 命令

```text
evidence-cli verify
evidence-cli inspect
evidence-cli head
evidence-cli export
evidence-cli checkpoint verify
evidence-cli vectors verify
evidence-cli repair-tail
```

## 25.2 默认行为

```text
- 只读；
- 不修改链；
- 输出 human-readable + --json；
- JSON 不是 canonical；
- 错误写 stderr；
- 敏感信息不输出；
- 支持指定 chain 和 sequence 范围。
```

## 25.3 退出码

```text
0  valid / success
2  invalid arguments
3  chain invalid
4  checkpoint/signature invalid
5  storage unavailable
6  unsupported version
7  repair required
```

## 25.4 repair-tail

只允许处理：

```text
最后一个未提交、无 commit marker 的不完整 frame
```

执行前必须：

```text
- 只读验证；
- 生成 repair plan；
- 用户显式确认；
- 备份原文件；
- 输出 repair evidence；
- 不得跨越可信 checkpoint。
```

---

# 26. 架构政策清单

必须新增策略声明（monorepo 历史路径如下；**infra.rs 不维护 `.architecture`，本仓 evidence 以 `crates/evidence` 最小面为准**）：

```text
.architecture/evidence-policy.toml   # monorepo-only；非本仓验收路径
```

示例：

```toml
schema_version = 1

[[chain]]
namespace = "domain_macro.series"
producer = "domain_macro"
subject_strategy = "provider-series-v1"
criticality = "required"
atomicity = "stage-then-append"
durability = "durable"
checkpoint_max_records = 10000
checkpoint_max_seconds = 60
retention_days = 2555
writer = "macro-engine"

[[operation]]
producer = "domain_macro"
name = "advance"
required = true
allowed_outcomes = ["committed", "rejected", "failed"]

[[operation]]
producer = "domain_macro"
name = "restore"
required = true
allowed_outcomes = ["committed", "failed"]
```

每个 required operation 必须登记：

```text
producer
operation
subject strategy
chain strategy
actor strategy
input canonical domain
output/error canonical domain
atomicity
durability
checkpoint policy
retention
owner
```

---

# 27. 机器门禁

## 27.1 Core 门禁

```text
EVIDENCE-PATH-001:
  runtime evidence package 位于 tools/ → fail。

EVIDENCE-DEP-001:
  core 内部依赖除 kernel 外不允许。

EVIDENCE-DEP-002:
  core 外部依赖只能是 sha2、thiserror。

EVIDENCE-ANYHOW-001:
  core 或公开签名出现 anyhow → fail。

EVIDENCE-CANONICAL-001:
  record hash 未使用 canonical V1 → fail。

EVIDENCE-DOMAIN-001:
  出现无 domain tag 的通用 hash_bytes → fail。

EVIDENCE-DEBUG-HASH-001:
  format!("{:?}") / Display / to_string 结果进入 digest → fail。

EVIDENCE-JSON-HASH-001:
  JSON bytes 直接进入 evidence digest 且无批准 schema → fail。

EVIDENCE-GENESIS-001:
  全零 digest 作为 genesis → fail。

EVIDENCE-PUBLIC-001:
  EvidenceRecord 字段公开可写 → fail。
```

## 27.2 Adapter 门禁

```text
EVIDENCE-DURABILITY-001:
  production required chain 使用 Volatile/Process → fail。

EVIDENCE-MEMORY-PROD-001:
  evidence_memory 出现在 release dependency graph → fail。

EVIDENCE-IDEMPOTENCY-001:
  adapter 未通过 event_id conformance suite → fail。

EVIDENCE-CONCURRENCY-001:
  adapter 未通过 concurrent append suite → fail。

EVIDENCE-RECOVERY-001:
  production adapter 无 crash recovery evidence → fail。

EVIDENCE-FSYNC-001:
  file adapter 声称 Durable 但没有 fsync contract → fail。
```

## 27.3 系统门禁

```text
EVIDENCE-POLICY-001:
  required operation 未登记 → fail。

EVIDENCE-COVERAGE-001:
  required operation 无成功/失败路径测试 → fail。

EVIDENCE-ATOMICITY-001:
  Tier-A 状态变化没有事务/outbox/source-of-truth 证明 → fail。

EVIDENCE-CHECKPOINT-001:
  production chain 无 checkpoint policy → fail。

EVIDENCE-ANCHOR-001:
  production signed checkpoint 无独立 anchor → fail。

EVIDENCE-SCHEMA-001:
  schema 变化未新建版本 → fail。

EVIDENCE-VECTOR-001:
  golden vectors 漂移但无 Approved RFC → fail。
```

---

# 28. CI 命令

Core：

```bash
cargo fmt -- --check
cargo clippy -p evidence --all-targets -- -D warnings
cargo test -p evidence
cargo llvm-cov -p evidence --fail-under-lines 95
cargo mutants -p evidence
cargo miri test -p evidence
# monorepo-only（infra.rs 不移植 archgate，不作为本仓 CI 硬门禁）:
# cargo run -p archgate -- --json
cargo run -p xtask -- lint-deps
cargo run -p xtask -- crate-standard --check
```

Adapters：

```bash
cargo test -p evidence_memory
cargo test -p evidence_file
cargo test -p evidence_postgres
cargo test -p evidence_file --test crash_recovery
cargo test -p evidence_postgres --test concurrency
```

工具：

```bash
cargo run -p evidence-cli -- vectors verify
cargo run -p evidence-cli -- verify <fixture>
```

Nightly：

```text
full mutation
full fuzz corpus
Miri
adapter chaos
checkpoint key rotation test
historical schema compatibility test
```

---

# 29. 性能与容量合同

## 29.1 Core

```text
- seal/verify 单条记录 O(record size)；
- 除名称和可选 signature 外避免不必要分配；
- canonical encoder 可写入调用方 buffer；
- 不复制大型业务 payload；
- record 固定字段保持小型。
```

## 29.2 Adapter

每个生产 adapter 必须发布基准：

```text
append p50/p95/p99
durable append p50/p95/p99
records/sec per chain
records/sec multi-chain
recovery time
verify throughput
checkpoint latency
storage bytes/record
```

不得通过降低 durability 冒充性能提升。

## 29.3 背压

当 durable storage 无法跟上：

```text
- required operation fail-closed；
- 明确返回 StorageUnavailable / DurabilityFailure；
- 不无限缓存于内存；
- 不丢弃旧 evidence；
- 不静默切换到 Volatile。
```

---

# 30. Observability

Evidence core 不依赖 observex。

Adapter 和 bootstrap 必须暴露：

```text
evidence_append_total
evidence_append_failures_total
evidence_append_latency_seconds
evidence_durability_failures_total
evidence_chain_head_sequence
evidence_checkpoint_age_seconds
evidence_checkpoint_failures_total
evidence_verify_failures_total
evidence_outbox_backlog
evidence_recovery_seconds
evidence_fork_detected_total
```

禁止将 record 原始字段、actor digest 或敏感上下文作为高基数 metric label。

---

# 31. 迁移计划

## 31.1 当前问题

```text
- runtime crate 位于 tools/evidence；
- 单一 ts 混合 event time / recorded time；
- module/op 直接拼接产生编码歧义；
- 全零 genesis；
- 无 chain_id；
- 无 sequence；
- 无 event_id 幂等；
- 无 operation_id；
- 无 actor/subject；
- 无 canonical wire；
- 无 checkpoint；
- 无 external anchor；
- 无 durable adapter；
- Debug 字符串参与 digest；
- lock poison 静默返回空值；
- Mock verify 永远成功；
- chain corruption 映射为 Invalid；
- fail-open/fail-closed 未裁定。
```

## 31.2 迁移阶段

### P0：冻结错误扩散

```text
- 禁止新增 hash_bytes 调用；
- 禁止新增 Debug → hash；
- 禁止新增 InMemoryEvidenceSink 生产使用；
- 当前 quality 降为 incubating；
- 当前“不可篡改”措辞改为 tamper-evident prototype。
```

### P1：Core V1

```text
- 建立 crates/evidence；
- 实现值对象；
- 实现 EvidenceDraft；
- 实现 EvidenceRecordV1；
- 实现 canonical V1；
- 实现 genesis / record digest；
- 实现 verifier；
- 提交 golden vectors。
```

### P2：Compatibility bridge

提供迁移 adapter：

```text
LegacyEvidenceRecord
→ 读取旧六字段记录
→ 生成 migration manifest
→ 不伪造为原生 V1 record
```

旧记录不能无损补出：

```text
chain_id
sequence
event_id
operation_id
actor
subject
recorded_at/event_time 分离
```

因此不得把旧链静默重编码后声称历史连续。

正确做法：

```text
V1 新链 genesis
→ 首条 migration record 引用旧链最终 digest 和 migration manifest digest
→ 旧链保留只读 verifier
```

### P3：Domain migration

```text
- domain_macro 改用 canonical point/state；
- 删除 Debug hash；
- 成功/拒绝/失败使用 typed outcome；
- 引入稳定 EventId / OperationId；
- 明确 chain policy；
- 补成功和失败合同测试。
```

### P4：Durable adapters

```text
- memory conformance；
- file adapter；
- postgres outbox；
- crash recovery；
- durability receipt。
```

### P5：Checkpoint

```text
- CheckpointV1；
- signer adapter；
- verifier adapter；
- key rotation；
- independent anchor；
- tail truncation test。
```

### P6：Cutover

```text
- bootstrap 强制 production adapter；
- 删除 tools/evidence；
- 删除旧 EvidenceSink；
- 删除 mock feature；
- 删除 hash_bytes；
- 更新架构规范和 dependency law；
- registry 达到验收后标 stable。
```

---

# 32. Evidence 自身的 Evidence

每次 evidence 系统变更必须生成：

```text
evidence/system/<date>-<change-id>/
├── manifest.json
├── spec-version.txt
├── commit.txt
├── toolchain.txt
├── commands.log
├── fmt.log
├── clippy.log
├── tests.log
├── coverage.json
├── mutants.json
├── fuzz-summary.json
├── golden-vector-diff.txt
├── adapter-conformance.json
├── recovery-tests.json
├── public-api.diff
├── schema-compatibility.md
├── threat-model-review.md
└── verdict.md
```

禁止：

```text
- 使用被测 evidence 系统自身作为唯一可信结果；
- 没有独立 CI artifact；
- SKIP 计 PASS；
- 旧 commit 结果冒充当前结果；
- 手写 digest 冒充工具输出。
```

Evidence 系统验收必须至少有一个独立验证器重新计算全部 golden vectors。

---

# 33. 完成定义

只有全部满足，`evidence` 才允许标记 3/3、5/5、stable。

## 33.1 规格闭合

```text
[ ] SPEC-EVIDENCE-002 Approved
[ ] 旧 spec superseded
[ ] ADR 冲突已修订
[ ] 路径和 package 对齐
[ ] architecture registry 对齐
[ ] evidence-policy.toml 已建立
[ ] 无未登记安全 Unknown
```

## 33.2 Core 闭合

```text
[ ] crates/evidence 已落地
[ ] canonical V1 冻结
[ ] 无字段拼接歧义
[ ] 无全零 genesis
[ ] ChainId / sequence / EventId 完整
[ ] recorded_at / event_time 分离
[ ] actor / subject 完整
[ ] typed outcome 完整
[ ] 无 generic hash_bytes
[ ] 无 Debug/JSON hash
[ ] 无 anyhow
[ ] record 字段私有
```

## 33.3 Adapter 闭合

```text
[ ] memory adapter 仅测试
[ ] file adapter durable
[ ] postgres/outbox adapter atomic
[ ] 并发 conformance 通过
[ ] idempotency 通过
[ ] crash recovery 通过
[ ] disk full / short write / fsync failure 通过
[ ] production 不会降级到 volatile
```

## 33.4 Checkpoint 闭合

```text
[ ] signed checkpoint
[ ] key rotation
[ ] independent anchor
[ ] tail truncation 可检测
[ ] full chain replacement 可检测
[ ] startup verify
```

## 33.5 测试闭合

```text
[ ] golden vectors
[ ] property tests
[ ] fuzz
[ ] line coverage >= 95% core
[ ] branch coverage >= 90% core
[ ] mutation score >= 90%
[ ] Miri
[ ] adapter chaos
[ ] historical schema verification
```

## 33.6 系统闭合

```text
[ ] required operations 全部登记
[ ] required operations 全部 fail-closed
[ ] Tier-A 全部具备事务/outbox/source-of-truth
[ ] external side effects 具备 Attempted + terminal evidence
[ ] source artifacts 有 retention
[ ] verifier/schema/public keys 保留期完整
[ ] CI Evidence 可追溯到当前 commit
```

---

# 34. 最终裁定

`evidence` 的生产终态不是：

```text
Vec<Record> + SHA256(prev || fields)
```

而是：

```text
typed evidence draft
→ canonical V1
→ domain-separated digest
→ contiguous chain
→ idempotent linearizable append
→ durable persistence
→ crash recovery
→ signed checkpoint
→ independent anchor
→ reproducible verification
```

系统只在以下条件成立时才能声称“关键操作可审计”：

```text
操作身份明确；
actor 和 subject 明确；
输入与结果使用稳定 canonical digest；
业务成功与 durable evidence 之间没有未声明窗口；
链没有 gap、fork 或 duplicate；
可信 checkpoint 能检测尾部截断和整链替换；
历史 schema、验证器和公钥在保留期内仍可用。
```

任何绕开这些条件的“快速 evidence”都不是简化，而是在审计信任链中制造不可证明的空洞。
