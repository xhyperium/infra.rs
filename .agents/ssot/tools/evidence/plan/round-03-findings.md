# Round 03 Findings — Evidence Plan Completeness

| 字段 | 值 |
|------|-----|
| round | `3` |
| focus | Security model T1–T18 · threat coverage · fail-closed · privacy §22 · what NOT claimed |
| result | **FAIL** |
| sources | `xhyper-evidence-complete-spec.md` · `plan/plan.md` · `plan/gap-matrix.md` · `plan/tasks.md` · `plan/approval-packet.md` · `.worktree/evidence-todo.md` |
| checklist | `plan.md` §4 全部 40 Check ID |
| baseline | `main@007ca7b5` · SPEC-EVIDENCE-002 **Proposed** |
| verifier | Verifier · 2026-07-14 |
| residual_open file | **ABSENT**（`T-RES-001` 仍 TODO；DEF 仅在 gap-matrix + todo） |

---

## result: FAIL

任一项 FAIL → 该轮 FAIL。本轮在安全闭环、fail-closed 全系统覆盖、§22 隐私/保留合同、原子性任务可追踪性上未达到「可执行计划完备」标准。

---

## failed_checks

| Check ID | 判定 | 证据摘要 |
|----------|------|----------|
| **R-SPEC-003** | **FAIL** | §33.6 映射存在，但不可执行：`Tier-A 原子性 → T-PG-004 T-ATOM via design` 中 **`T-ATOM` 非真实 Task ID**；`external Attempted+terminal` 仅括号备注「订单域后续」**无 Task / 无正式 DEFER(accepted)**；`source artifacts retention → T-POL-002 + docs`、`verifier/schema/keys 保留 → T-CP-006 + docs` 把 §22/§33.6 硬要求缩成文档附注，不可机器验收。 |
| **R-GAP-002** | **FAIL** | gap-matrix §2 列出 T1–T18 与「目标防御」短语，但 **无 T_n → Task ID 绑定表**。T14/T16 仅模型/文档级；T13「签名边界」无对应认证/签名校验任务（仅 `T-CORE-008` 存 `id_digest`）。todo §3 只有 Wave 指针，不等于覆盖策略可验证。 |
| **R-APPEND-002** | **FAIL** | 幂等/CAS 有 `T-MEM-002`/`T-PG-005`；**fail-closed 仅 `T-DOM-005`（domain_macro）**。Spec §13.7 要求 **所有** policy `required` 操作 evidence 未达 durability 则业务不得成功。`T-GATE-*` 无 fail-closed AC；`T-POL-002` 只登记不强制；`T-ARCH-006` 系统门禁未写清 fail-closed 测法。DEF-017 未在计划层真正闭合路径。 |
| **R-ATOM-001** | **FAIL** | Spec §15.1 要求 A 同事务 / B outbox / C SoT **三选一**。任务仅覆盖 B（及 PG 直接 append 的 A 变体）；**无 C source-of-truth 路径任务**；映射写 `T-ATOM via design` = 幽灵任务。 |
| **R-POL-001** | **FAIL** | `T-POL-001` AC 仅为「schema_version + 示例 chain/operation」。Spec §22.3 要求至少：record / checkpoint / signing public key / canonical schema / source artifact / verifier binary·source **六类 retention**。计划未把这些字段写入 policy 骨架 AC，也未映射到 `evidence-policy.toml` 结构验收。 |

### 本轮焦点相关、未单独升格为 failed 但须记入 omissions 的边界项

| Check ID | 判定 | 备注 |
|----------|------|------|
| R-SPEC-001 | PASS | `SPEC-EVIDENCE-002` 页眉存在 |
| R-SPEC-002 | PASS | gap-matrix §1 含 §0–§34 |
| R-GAP-001 | PASS | DEF-001…018 均在 gap-matrix §4 **与** `.worktree/evidence-todo.md` §1（18/18） |
| R-PATH-001…003 | PASS | plan §5 / W6 cutover 写明 |
| R-DEP-001/002 | PASS | core 白名单与禁 anyhow/serde/tokio 有任务 |
| R-API-* / R-CANON-* / R-TIME / R-CHAIN | PASS | W1 任务覆盖（非本轮深检对象） |
| R-APPEND-001 | PASS* | `T-CORE-022` 有 Durability 三态；\*生产默认 Durable 在 approval A6，adapter 侧细节见 R4 |
| R-READ-001 | PASS* | `T-MEM-003`「限制」偏薄（limit/损坏不静默跳过未写 AC） |
| R-CP-001 | PASS | T-CP-001…006 + TailTruncated |
| R-ERR-001 | PASS | T-CORE-020/021 |
| R-MEM-001 | PASS* | production_allowed + 禁伪 Durable 有；§19.3 bootstrap/systemd 清单未全覆盖（R4） |
| R-FILE-001 / R-PG-001 | PASS* | 任务存在；§20/§21 细节不足见 Round 4 |
| R-TEST-001 / R-CLI-001 / R-GATE-001 | PASS | 有任务骨架 |
| R-MIG-001 | PASS | plan §2/§6 + P0–P6 |
| R-EVID-001 | PASS | plan §8 含 `threat-model-review.md` 槽位 |
| R-DOWN-001 | PASS | W3 domain_macro + gate |
| R-GOV-001 | PASS | approval-packet 人审闸；Spec 仍 Proposed |
| R-FORBID-001 | PASS | plan 页眉 Forbidden 与执行步骤无「把 Proposed 当 Approved」矛盾 |
| R-TODO-001 | PASS | Waves + DEF-001…018 齐全 |
| R-CROSS-001 | PASS | plan §1.2 INFRA-003 边界 |
| R-HONEST-001 | PASS | 全文检索：未把 Proposed 写成 Approved；campaign ≠ stable；无假 §33 Done |

---

## omissions

### O3-1 — §22 隐私 / 保留（本轮核心遗漏）

| Spec | 计划现状 | 缺口 |
|------|----------|------|
| §22.1 record 仅标识/时间/受控名/摘要/链字段 | 隐含于 V1 模型任务 | **无显式 AC**：禁止 payload/String 原文字段、API 层拒绝敏感原文 |
| §22.2 独立 content-addressed artifact store | `T-POL-002 + docs` | **无** package/path/trait；**无** retention / ACL / 加密 / 完整性 / 删除审批 / Object Lock 合同任务 |
| §22.3 六类 retention | 未进 policy AC | record/checkpoint/key/schema/artifact/verifier 保留策略 **未任务化** |
| §22.4 审计期内禁物理删；erasure 时保留 digest + deletion/erasure evidence | **无 Task** | 法律删除路径与「不改历史 record」无执行项 |

gap-matrix 将 §22 标 ABSENT → W5/W7，但 **W5 任务表无任何 §22 专项 Task**（仅 checkpoint/CLI/obs/perf）。

### O3-2 — 威胁 T1–T18 覆盖质量

| ID | 计划「目标防御」 | 可追踪 Task | 严格评估 |
|----|------------------|-------------|---------|
| T1 改字段 | canonical + digest | T-CORE-025/030 | 策略充分 |
| T2 删中间 | sequence + previous | T-CORE-019/025/030 | 充分 |
| T3 重排 | sequence 连续 | 同上 | 充分 |
| T4 重复插入 | event_id 幂等 | T-MEM-002 | 充分（W2） |
| T5 链尾截断 | signed checkpoint | T-CP-004 | 充分（W5） |
| T6 整链替换 | independent anchor | T-CP-005 | 合同接口有；**真实锚点 Defer**——计划未标明 T6 在仅有接口时仍 OPEN 的验收边界 |
| T7 分叉 | CAS + ForkDetected | T-MEM-005 / T-PG-006 | 充分 |
| T8 重试重复 | event_id 幂等 | T-MEM-002 / T-DOM-003 | 充分 |
| T9 崩溃半条 | frame+commit | T-FILE-005/008 | 见 R4 细节 |
| T10 短写/fsync | Durable 合同 | T-FILE-004/008 | 见 R4 |
| T11 编码不一致 | golden V1 | T-CORE-026/027 | 充分 |
| T12 Debug/JSON | 禁 + domain digest | T-DOM-001 T-ARCH-004 | 充分 |
| **T13 伪造 actor** | id_digest + **签名边界** | T-CORE-008 仅存储 | **签名边界无任务**；信任边界要求 domain 提供身份，但「检测伪造」依赖外部签名——计划未写清 **不声称** 单独靠 record 防冒充 |
| **T14 敏感原文** | 仅 digest+受控名 | 无专项 | 模型暗示 ≠ 强制/测试/门禁 |
| T15 mock 生产 | 阻断 memory 生产图 | T-MEM-004/007 T-ARCH-005 T-CORE-034 | 较充分；bootstrap/systemd 见 R4 |
| **T16 旧 verifier** | schema retention | T-LEG-002 弱；keys/schema/verifier 保留靠 docs | **与 §22.3 同缺** |
| T17 时钟混淆 | recorded_at/event_time | T-CORE-011 | 充分 |
| **T18 写失败仍成功** | required + fail-closed | 仅 T-DOM-005 | **非全 required 面** |

### O3-3 — Fail-closed（§13.7 / §15 / DEF-017）

- 规范：policy `required` → append 未达 durability → **业务不得报告成功**；不存在 best_effort evidence。
- 计划：domain_macro 单点测试；gate 迁移无对等 AC；无「policy required × 调用点」矩阵任务；无 fail-open 静态/集成门禁用例列表。
- approval 不可豁免项含 fail-closed，但 **无可执行关闭证据路径** → 与 R-APPEND-002 同 FAIL。

### O3-4 — What NOT claimed（§1.1 / §1.2）

**已覆盖：**

- plan §0.1 / Forbidden / T-DOC-001：无签名检查点+独立锚点时禁「不可篡改 / 绝对可信 / 无法删除 / 永久证明」。

**未覆盖（遗漏）：**

Spec §1.2 **明确不声称抵御**：

1. 所有签名私钥与外部锚点同时被攻破；
2. 调用方在摘要前故意提供虚假原始数据；
3. 原始输入从未被保留；
4. 主机内存与全部 TEE 同时被完全控制。

上述四条 **未** 进入 plan / approval / todo / gap。风险：实现方或对外材料把 T1–T18「有覆盖策略」误读为「可抵御上述场景」→ **false-pass / overclaim**。

### O3-5 — 信任边界 §1.3

checkpoint signer / external anchor / bootstrap 禁止 volatile 在任务中部分出现，但 **bootstrap 组装合同**（生产只装 durable）无独立 Task；仅靠 archgate release 图不够覆盖「systemd 部署清单」（§19.3）。

### O3-6 — residual 纪律

- R-GAP-001 靠 todo/gap 中的 DEF ID **PASS**。
- `T-RES-001` residual-open **未初始化** → 无统一 OPEN/CLOSED/DEFER 状态机文件；§33.1「无未登记安全 Unknown」依赖未来 residual，当前计划完备性仍脆弱。

---

## false_pass_risks

1. **R-GAP-002 字面 PASS 风险**：仅看 gap-matrix 有 T1–T18 行会误 PASS；无 Task 绑定与 §22/T14/T16 时威胁台账是装饰性的。本轮已严格判 FAIL。
2. **「T18 / DEF-017 已有 T-DOM-005」风险**：单域测试 ≠ 全 required 面 fail-closed；stable/§33.6 前易假绿。
3. **「T-POL-002 + docs」覆盖 §22 风险**：政策登记不能替代 artifact store 合同与 retention 字段；§33.6 source artifacts 勾选会假 PASS。
4. **T6/T13 overclaim**：有 anchor 接口 + actor 字段 ≠ 已防御整链替换/身份伪造；缺 §1.2 非声称声明时对外可信度表述易越界。
5. **Proposed vs 计划完成感**：plan 诚实标注 Proposed，但若仅因 DEF 全登记就宣称「安全模型计划完备」会假 PASS——本轮 FAIL 阻断。
6. **threat-model-review.md 仅目录槽位**（plan §8）：无强制 W0/W7 产出内容要求，可能空文件勾 §32。

---

## notes

### DEF-001…018 在 todo 的核验

| DEF | todo | Wave 标注 |
|-----|------|-----------|
| DEF-001…018 | **全部 18 项出现**于 `.worktree/evidence-todo.md` §1 | 与 gap-matrix 一致 |

**R-GAP-001 = PASS**（本轮不因 DEF 缺失而 FAIL）。

### 与 checklist 40 项的本轮摘要

```text
failed_checks (strict R3):
  R-SPEC-003, R-GAP-002, R-APPEND-002, R-ATOM-001, R-POL-001

pass: 其余 35 项（含 * 薄弱但本轮不升格 FAIL 者，详见上表）
```

### 关闭本轮 FAIL 的最低补强（建议，非本文件职责执行）

1. 增加 **T_n → Task** 表（T1–T18 每行 ≥1 可验收 Task 或正式 DEFER）。
2. 新增 §22 任务：artifact store 合同、六类 retention 进 policy schema、erasure/deletion evidence。
3. 扩展 fail-closed：`required` 操作矩阵 + gate/其他调用方测试 + 门禁；消灭 `T-ATOM via design`。
4. 在 plan/approval 写入 Spec §1.2 **不声称** 列表，并约束对外措辞。
5. 落地 `T-RES-001` residual-open，T6/T13/§22 未闭合项显式 OPEN。

### 诚实声明

- 本轮为 **计划完备性** 检查，**不是** 实现验收；代码侧安全能力仍为 OPEN（todo 已标明）。
- Spec 状态仍为 **Proposed**；本轮 FAIL **不** 授权改写为 Approved。
- 未将 SKIP/DEFER 记为 PASS。
