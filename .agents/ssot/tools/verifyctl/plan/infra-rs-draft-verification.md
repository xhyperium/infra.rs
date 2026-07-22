# infra.rs draft verification 入库（只读快照）

> **来源**：
> **关联**：verifyctl 最小 plan/execute/report + evidence

---

以下是整合全部讨论的完整落地文档，保存为 `vibe-coding-verification.md` 即可使用。

```markdown
# Vibe Coding 自验证体系：设计与落地手册

> 版本：v1.0 ｜ 适用项目：infra.rs（可泛化至任何 AI 辅助开发的 Rust 项目）
>
> 核心命题：自验证不是让 AI 再检查一遍自己的代码——生成者与检查者共享盲区，
> 只能形成自我确认。有效机制是：**把模糊意图编译成可执行契约，让独立验证器
> 主动尝试推翻实现，并为每个结论生成与代码版本绑定的证据。**

---

## 0. 三条元原则（裁决一切设计争议）

1. **断言必须可证伪，证据必须可失效。**
   不能被机器推翻的声明（"看起来没问题"、"高置信度"）在体系内没有地位；
   不随代码/环境变化而失效的证据是过期担保。

2. **默认方向永远朝安全侧倾斜（fail-closed）。**
   未映射路径 → 全量验证；未声明依赖 → 标记 unattested；
   无法分类的变更 → 按最高风险处理。

3. **体系健康由两个回路决定，而非任何单点强度。**
   - 沉淀回路：缺陷 → 永久资产（让系统变强）
   - 裁剪回路：成本 → 检查删减（让系统不臃肿）
   季度回顾只问两个问题：逃逸缺陷都变成永久规则了吗？删掉了哪些零杀伤检查？

**最高纪律：可以复用计算结果，但不能复用未经证明的结论。**

---

## 1. 总体架构

```

自然语言 Goal
→ 可执行契约（人确认）
→ Builder 生成实现
→ 分层确定性检查（V0–V3）
→ 独立 Verifier 反证（结构化 probe）
→ Evidence 证据包（版本绑定）
→ Gate 机器裁决
→ Runtime 验证（replay / shadow / canary / 回滚）
→ 缺陷沉淀为永久资产 → 回到契约层

```

### 状态机（含降级规则）

```

DRAFT → LOCALLY_VERIFIED → INDEPENDENT_VERIFIED → MERGE_ELIGIBLE → RUNTIME_VERIFIED

```

| 事件 | 降级规则 |
| --- | --- |
| base 前进，无文件/依赖交集 | 自动 rebase，复验 V0/V1 后保级 |
| base 前进，有交集 | 降回 LOCALLY_VERIFIED |
| Cargo.lock / toolchain 变更 | Evidence 失效，降回 LOCALLY_VERIFIED |
| policy 收紧 | 仅对新 PR 生效；存量 PR 打标记，不强制重验 |

### 目录结构

```

.agent/
├── goals/<goal-id>/goal.yaml
├── specs/<module>/acceptance.yaml
├── invariants/registry.yaml        # 全局不变量注册表
└── verification/
├── profiles.toml               # fast / pr / depth
├── path-ownership.toml         # 非 Rust 输入 → crate 映射
└── probes.yaml                 # Verifier 反证清单

tools/
├── goalctl/       # Goal → 可执行契约；不变量对撞检查
├── verifyctl/     # 影响分析、命令编排、报告收集
└── evidence/      # 证据生成、摘要、签名、校验

crates/infra/gate/policy    # 纯机器裁决

.github/workflows/
├── ci-fast.yml
├── ci-depth.yml
└── runtime-validation.yml

```

### 统一入口

```bash
cargo verify fast      # 本地，30–90 秒
cargo verify pr        # Fast Gate，≤5 分钟
cargo verify depth     # Depth Gate，≤60 分钟
cargo evidence verify
cargo gate decide
```

---

## 2. Vibe-to-Contract：契约层

### 2.1 goal.yaml 模板

```yaml
id: GOAL-YYYY-NNN
outcome: <一句话业务结果>
risk: R0 | R1 | R2 | R3

acceptance:
  - id: AC-001
    given: <前置条件>
    when: <触发动作>
    then: <可断言的结果>        # 禁止主观形容词、禁止无断言动词

invariants:
  - <必须始终成立的性质>        # 同步登记到全局 registry

forbidden:
  - <禁止的实现手段>            # 如 sleep 规避竞态、mock 关键持久化

not_in_scope:
  - <显式排除项>                # 反面清单,提高人工确认的信息密度

depends_on: []                  # 硬依赖 Goal
conflicts_hint: []              # 软冲突 Goal,建议串行
touches: []                     # 声明触及的 crate;越界改动 → BLOCKED
```

### 2.2 契约层的强制规则

- 每个 `AC-*` 必须映射到至少一个可执行检查，否则不能声称 Goal 完成。
- AC 加 lint：禁止不可证伪表述（"功能正常"、"性能良好"）。
- R3 变更 AC 数量异常少（<3）→ 触发人工审查。
- 契约生成时同步输出 **SPEC-GAP 预扫**（实现中的隐含决策）与
  **not_in_scope 反面清单**，供人一次性确认。
- **注意：契约层是全链最脆弱一环**——AC 编错，后面所有验证都在
  精确验证错误的东西。此环节的人工确认不可自动化。

### 2.3 不变量注册表（跨 Goal 共享真理）

```yaml
# .agent/invariants/registry.yaml
- id: INV-ORDER-001
  scope: order_execution
  statement: 一个 idempotency_key 至多对应一个已确认成交
  owner_goal: GOAL-2026-001
  enforced_by: [concurrent_duplicate_only_one_executes]
```

规则：
1. 新契约编译时，goalctl 对 scope 相交的所有不变量做**对撞检查**
   （compatible / conflicting / needs_human）——把契约冲突从
   "合并后运行时才炸"提前到"契约确认时暴露"。
2. 修改既有不变量必须声明 `supersedes:`，触发原 owner Goal 回归重跑。
3. 全局不变量的测试在任何触及其 scope 的 PR 中强制运行，
   无论该 PR 是否声明。

---

## 3. Builder / Verifier 分离

### 3.1 Builder

- 读取：Goal、Spec、架构约束。
- 产出：实现 + 基础测试（用 `#[verifies(AC-xxx)]` 宏绑定）。
- 执行 `cargo verify fast` 后提交。
- **无权给出最终 PASS。**

### 3.2 Verifier（独立上下文）

- 只读取：Goal、diff、接口签名、测试报告。
- **不读取** Builder 的解释和实现理由。
- 输出三态结论：`PASS / FAIL / BLOCKED`，禁止主观置信度。
- 实例按 Goal 一次性创建，验证后销毁，不复用上下文。
- 不得是依赖图上相邻 Goal 的 Builder（利益立场污染）。

### 3.3 独立性来源强度排序

| 强度 | 手段 | 说明 |
| --- | --- | --- |
| 最强 | differential testing、真实数据 replay | 与实现完全无关的参照系 |
| 中等 | 不同厂商模型做 Verifier | 切断模型级盲区 |
| 最弱 | 同模型 + 干净上下文 | 只切断对话级污染 |

**R3 模块必须包含前两类至少一项。**

### 3.4 结构化反证清单（禁止自由发挥）

```yaml
adversarial_probes:
  - id: P-BOUNDARY
    question: 每个 AC 的 given 取边界值和边界外一步时会怎样？
  - id: P-CONCURRENT
    question: 任意两个操作交错执行,不变量是否仍成立？
  - id: P-PARTIAL-FAILURE
    question: 每个外部调用在"已执行但未返回"时,重试路径是否安全？
  - id: P-TEST-VACUITY
    question: 把实现替换为 panic!/默认值,现有测试会失败吗？
  - id: P-SPEC-GAP
    question: 实现处理了哪些 AC 未提及的情况？需要人确认吗？
  - id: P-EXTERNAL-IDEMPOTENCY          # 来自逃逸缺陷沉淀
    question: 外部系统的哪些幂等假设未经真实环境验证？
```

每个 probe 必须给出 `refuted / not_refuted / needs_evidence`
+ 具体反例或推理链。禁止"整体看起来没问题"。

---

## 4. 五层验证体系与 Rust 工具分工

### 4.1 分层

| 层级 | 验证内容 | 示例 | 阻断性 |
| --- | --- | --- | --- |
| V0 | 结构与静态规则 | fmt、lint、依赖层级、API 规则 | 阻断 |
| V1 | 变更影响范围 | affected crate 编译 + 单测 | 阻断 |
| V2 | 契约与不变量 | property、negative、metamorphic | 阻断 |
| V3 | 对抗与深度验证 | mutation、fuzz、Miri、loom、Kani | 风险驱动 |
| V4 | 真实运行验证 | replay、shadow、canary、回滚 | 发布门禁 |

### 4.2 工具精确分工（防止虚假安全感）

| 工具 | 证明什么 | 证明不了什么 | 适用 |
| --- | --- | --- | --- |
| proptest | 采样范围内不变量成立 | 未采样输入 | 所有纯逻辑 |
| Miri | UB、内存错误 | 逻辑错误；不跑 FFI | unsafe 代码 |
| loom | **穷举**并发交错正确性 | 大状态空间；进程崩溃 | 锁、幂等控制 |
| cargo-fuzz | 崩溃、panic | 静默语义错误 | 解析器、协议边界 |
| Kani | 有界范围内的**证明** | 无界循环 | 数值核心（decimalx） |
| cargo-mutants | 测试套件杀伤力 | 实现正确性 | 全局质量度量 |

配对规则：
- **R3 + 并发 → loom 强制**（普通并发测试撞不到坏交错）
- **R3 + 数值 → Kani 或穷举证明强制**
- mutation 用**增量模式**（只对 diff 触及函数生成变异体），否则 60 分钟守不住

### 4.3 影响分析的两个 false-negative 陷阱

1. **Feature unification**：闭包计算必须基于 feature-resolved 依赖图，
   不能用裸 package 依赖图。
2. **非 Rust 输入**：build.rs、SQL 迁移、配置模板不在依赖边上。
   维护 `path-ownership.toml` 显式映射；**未映射路径默认触发全量验证**。

---

## 5. Evidence：可失效的证据

### 5.1 证据包格式

```json
{
  "goal_id": "GOAL-2026-001",
  "base_sha": "...", "head_sha": "...", "diff_digest": "...",
  "cargo_lock_digest": "...", "toolchain_digest": "...", "policy_digest": "...",
  "criteria": { "AC-001": ["test_concurrent_duplicate_order"] },
  "verifier_probes": { "refuted": 0, "needs_evidence": 0 },
  "unattested_dependencies": ["exchange_api@sim"],
  "commands": [], "reports": [],
  "runner_identity": "...",
  "verdict": "PASS"
}
```

失效条件（任一变化即失效）：Git SHA / diff、Cargo.lock、toolchain、
验证策略、测试输入、环境或外部服务版本。

### 5.2 不可固定的外部依赖

交易所 API、行情源等版本不可观测的依赖，显式记入
`unattested_dependencies`。**该字段非空的 R3 变更强制走 V4**——
shadow/canary 是唯一覆盖不可固定依赖的手段。

### 5.3 信任根分层（按需推进，勿过度工程）

| 层级 | 实现 | 防什么 | 时机 |
| --- | --- | --- | --- |
| L0 | digest 人工核对 | 无意的环境漂移 | Day 1 |
| L1 | GitHub OIDC 签名 runner_identity | 伪造"CI 跑过" | 启动即做 |
| L2 | Sigstore keyless 签名 + 透明日志 | 事后篡改 | 30 天内 |
| L3 | SLSA provenance | 供应链攻击 | 仅合规要求时 |

开发机 Evidence 可帮助命中缓存，**不能单独成为合并凭证**。

---

## 6. Gate：纯机器裁决

### 6.1 裁决清单

- [ ] 所有 AC 有测试映射
- [ ] 所有强制命令成功
- [ ] Evidence 与当前提交完全一致
- [ ] 无未声明文件变更（超出 `touches` → BLOCKED）
- [ ] 无架构依赖或安全边界违反
- [ ] 无未处理的 Verifier 反例（refuted = 0）
- [ ] 风险等级匹配验证深度
- [ ] R3：canary 计划含**仓位处置预案**，否则 BLOCKED

### 6.2 AC 映射防伪（三层防御）

1. **语法层**：`#[verifies(AC-xxx)]` 宏 + goalctl 静态扫描完整性。
2. **空洞检测层（关键）**：R2+ 的每个 AC 做定向破坏——自动将对应逻辑
   替换为退化实现，确认绑定测试变红。等价于按 AC 分组的靶向 mutation，
   进 Depth Gate。
3. **语义层**：Verifier 逐条比对 AC 文本 vs 测试断言是否蕴含 then 子句。

### 6.3 合并列车

Gate 最终裁决对象是 **merge_result_sha**（GitHub merge queue），
不是 branch head——覆盖无文本交集的语义冲突。
这要求 Fast Gate 真正守住 3–5 分钟（否则队列吞吐崩溃）。

### 6.4 Override 规则

不禁止 override（否则第一次紧急 hotfix 就会推翻整个体系），但：
- 强制记录理由
- 自动创建补验证工单
- 计入 Gate Override Rate 指标

---

## 7. 风险分级

| 级别 | 范围 | 强制验证 |
| --- | --- | --- |
| R0 | 文档、注释 | 静态验证 |
| R1 | 普通内部逻辑 | 单测、property、影响范围 |
| R2 | 公共 API、持久化、并发 | 集成测试、故障注入、Miri/fuzz、AC 空洞检测 |
| R3 | 资金、交易执行、认证、迁移 | 独立验证（强独立源）、replay、shadow、canary、自动回滚、loom/Kani、仓位预案 |

`order execution / risk / accounting / position` 默认 R3。

---

## 8. Runtime 验证（交易系统特化）

副作用不可重放，shadow 必须分三层：

1. **Replay 决策层**：历史行情喂新旧逻辑，diff 决策输出，不真下单。
2. **Paper execution**：实时行情 + 模拟撮合。撮合偏差是**已知验证空洞**，显式记录。
3. **Canary 最小真实资金**：唯一能验证真实滑点与交易所行为的层。

**关键约束：回滚代码 ≠ 回滚仓位。**
canary 启动前必须定义异常时的仓位处置（平仓 / 移交旧系统接管），
无预案 → Gate BLOCKED。

---

## 9. 多 Agent 并行协调

### 9.1 冲突三层次（危险性与可见性成反比）

| 层次 | 发现者 |
| --- | --- |
| 文本冲突 | Git |
| 语义冲突（无文本交集的行为依赖） | merge queue 全量验证合并结果 |
| **契约冲突**（两个 Goal 各自自洽但互相矛盾） | **不变量注册表对撞检查** |

### 9.2 调度机制

- **乐观并行**，不做悲观锁；冲突者自动 rebase 重验（依赖状态机降级规则）。
- Goal 声明 `depends_on / conflicts_hint / touches`；
  `touches` 准确性由 Gate 的越界检查强制。
- Verifier 按 Goal 一次性分配，不跨 Goal 复用上下文，回避相邻 Goal 串谋。
- Depth Gate 预算：R3 抢占 R1；内容寻址缓存共享 base 结果；
  **队列时间计入 Feedback Time 指标**。

---

## 10. 缺陷沉淀（Compound Engineering）

每次逃逸缺陷必须产出五样资产：

```
缺陷 → ① 最小反例（录制/构造）
     → ② 回归测试（绑定新 AC）
     → ③ 不变量升级（写入 registry）
     → ④ Gate 规则（同 scope 的 PR 强制新检查）
     → ⑤ 模板/probe 清单更新（后续所有 Agent 自动继承）
```

自改进 Agent 定期分析：穿透 Gate 的缺陷、零杀伤测试、高耗时低收益检查、
反复出问题的模块、无法执行化的 AC。
**Agent 只能提案，不能自行降低强制门禁；优化目标必须是指标对的
帕累托改进，禁止单指标驱动。**

---

## 11. 指标体系（成对出现，互为张力）

| 主指标 | 制衡指标 | 防止的作弊 |
| --- | --- | --- |
| False Green Rate ↓ | Gate 误报率（BLOCKED 但实际无问题） | Gate 极端保守化 |
| Mutation Score ↑ | 逃逸缺陷率 | 针对 mutant 写断言 |
| AC Executable Coverage ↑ | AC 语义审查抽样 | AC 越写越窄 |
| Median Feedback Time ↓ | Depth Gate kill count | 慢而有效的检查被移出后失守 |
| Repeat Defect Rate ↓ | 缺陷分类稳定性 | 分类细化使"同类"永不重复 |

补充指标：
- **Gate Override Rate**（体系腐化的最早领先信号）
- Flaky Rate（**硬预算**：超阈值自动隔离 + 开缺陷单，隔离测试不计入 AC 覆盖）
- Evidence Reuse Rate、Mean Time to Rollback、Escape Defect Rate

每检查记录 **kill count / cost** 两数，季度裁剪
`kill count = 0 且 cost 高` 的检查。

---

## 12. 人工介入点（仅四个，其余人工介入 = 体系缺陷信号）

| 介入点 | 人确认什么 | 界面要求 |
| --- | --- | --- |
| ① 契约确认 | AC 总和 = 我的意图？ | 展示 AC + 反面清单 + SPEC-GAP，一次确认 |
| ② R3 canary 放行 | 真金白银启动 | 错误成本不可逆，保留人工 approve |
| ③ Gate Override | 紧急绕行 | 强制理由 + 补验证工单 + 计入指标 |
| ④ 验证策略删减 | Agent 提案的批准 | 附 kill count / cost 数据 |

---

## 13. 失效模式警戒清单（按概率排序）

1. **Gate Override 常态化**（组织纪律）——最快死法
2. **契约质量下降**（不可证伪 AC 混入）——AC lint 防御
3. **Flaky 腐蚀信任** → 重跑文化——硬预算防御
4. **验证膨胀** → 反馈变慢 → 绕路——裁剪回路防御
5. **Evidence 过度工程化**耗尽耐心——信任根分层推进防御

注意：五条里没有一条是"验证技术不够强"。

---

## 14. 落地计划

### MVA（1 天内五件事，按此优先级）

> 优先级修正：**Evidence 绑定 + Gate > AC 映射 > 影响分析 > 独立 Verifier**
> （没有 Evidence 体系时，Verifier 的结论同样是未经证明的断言）

1. 定义 `goal.yaml`，强制 `AC-*` 编号 + AC lint。
2. 建立 `fast / pr / depth` 三个 Profile，统一 `cargo verify` 入口。
3. `git diff + cargo metadata`（feature-resolved）计算受影响 crate 闭包。
4. 生成绑定 `head_sha + Cargo.lock + toolchain + policy` 的 Evidence（L1 信任根）。
5. 唯一 required check：`gate / merge-eligible`。

### 7 天

- 独立 Verifier + 结构化 probe 清单（含 P-TEST-VACUITY）
- kernel、decimalx、交易核心 property test
- 影响分析 + 内容寻址缓存
- 增量 mutation、Miri、fuzz 入 Depth Gate
- AC 定向破坏（空洞检测）
- 验证失败分类 + Flaky 隔离机制
- 不变量注册表初始化

### 30 天

- 真实数据 replay + Shadow Mode（三层）
- R3 启用 Canary + 自动回滚 + 仓位处置预案
- merge queue 接入，Gate 裁决 merge_result_sha
- 逃逸缺陷自动转化为回归测试候选（五资产链）
- Evidence 签名升级到 L2（Sigstore）
- kill count / cost 记录，启动裁剪回路
- 指标对上线，Gate Override Rate 进看板

### CI 时间硬约束

| 流程 | 目标 | 策略 |
| --- | --- | --- |
| 本地反馈 | 30–90 秒 | 受影响依赖闭包 |
| PR Fast Gate | ≤3–5 分钟 | 阻断合并；merge queue 吞吐的硬约束 |
| Depth Gate | ≤60 分钟 | 增量 mutation、缓存、R3 抢占 |
| Runtime Gate | 持续 | shadow、canary、指标监控 |

---

## 15. 体系边界（诚实声明）

本体系能压低三类风险：**实现与意图的偏差、验证结论的失效、同类错误的重复。**

对两类风险作用有限：
1. **意图本身错了**——体系保证精确得到你要的东西；要错了东西，
   它会精确交付错误。
2. **未知的未知**——验证空洞可标记、可兜底，不可穷尽。
   正确姿态不是承诺零逃逸，而是保证每次逃逸**代价有界、且只发生一次**。

> 最终目标不是证明"AI 写得很好"，而是构造一个环境：
> **错误的代码进入生产的路径越来越窄，
> 而每一次成功穿越的错误都会把自己走过的路堵死。**

---

## 附录 A：单 Goal 全流程速查（以幂等订单为例）

```
1. 意图 "重试别下重复单"
   → goalctl 编译契约 + SPEC-GAP 预扫
   → 人补 AC-003（查询失败 fail-closed）← 人工介入①
2. Builder 实现 + #[verifies] 绑定测试（loom 覆盖 AC-001）
   → cargo verify fast → 提交
3. Verifier 跑 probe 清单
   → P-PARTIAL-FAILURE refuted（崩溃窗口永久 InFlight）→ FAIL
4. Builder 修复（恢复协议 + 故障注入测试）→ 二轮 PASS
5. Depth Gate（loom + 定向 mutation + 故障注入）→ Evidence 生成
6. Gate 裁决:unattested_dependencies 非空 → 强制 V4
   → merge queue 对合并结果复验 → 入 main
7. Replay diff 零 → Shadow 一周 → Canary（人工放行 ← 介入②,附仓位预案）
8. 逃逸缺陷（交易所双 ack）→ 自动回滚 4 分钟
   → 五资产沉淀:反例/回归测试/不变量升级/Gate 规则/probe 更新
```

三个缺陷、三个不同的层捞住——分层不是冗余，
是每层覆盖一类本质不同的错误来源。

## 附录 B：完整地图

```
单 Goal 纵向:  意图 → 契约(人确认) → Builder → 分层验证(工具精确分工)
              → 独立反证(结构化 probe) → Evidence(信任分级) → Gate(机器裁决)
              → Runtime(replay/shadow/canary+仓位预案) → 缺陷沉淀

多 Goal 横向:  不变量注册表 → 依赖声明 → 乐观并行 + merge queue
              → Verifier 分配规则 → 预算调度

体系自身:     两个回路(沉淀/裁剪) + 成对指标 + 四个人工介入点
              + Override 可审计 + fail-closed 默认
```
````

---

文档已按"原则 → 机制 → 计划 → 附录"四层组织，每节自带可执行的规则清单和模板，可直接复制保存为 `.md` 文件。落地时建议从第 14 节的 MVA 五件事开始，其余章节作为各机制上线时的实施规范逐节启用。