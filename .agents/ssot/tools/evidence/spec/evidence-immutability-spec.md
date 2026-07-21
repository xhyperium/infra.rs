# Evidence Immutability Spec

```text
Spec ID:          SPEC-EVIDENCE-IMMUTABILITY-DRAFT-001
Title:            infra.rs Evidence Immutability (Repository-History Layer)
Status:           Draft
Created:          2026-07-19
Owner:            platform / security
Companion Spec:   SPEC-EVIDENCE-002 (Approved)
Relation:         补充而非替代
Activation Gate:  需 RFC + 两名 Maintainer approval + Risk Owner review 后方可接入 CI
```

---

## 1. 动机

审计 §4.2 与 §7.P2 指出：仓库现有 trusted contract（`main-first-contract` workflow）
仅校验 `evidence/**/*.contract.json` 的 append-only 语义（每个 PR 必须新增且只能新增
一个 Change Contract），**不**覆盖历史 Evidence 真值文件。

PR #721（commit `acbd06ca432fb6f73394b3ef09805a943512d35b`，`chore(agent): rename
.agent/specs to .agents/ssot`）机械改写路径时，连带修改了 138 份 `evidence/**/*.log`：

```bash
git diff --name-status 1df6cc505..acbd06ca4 -- 'evidence/**/*.log' \
  | awk '{print $1}' | sort | uniq -c
# → 138 M
```

`Harness Check` 在事后仅报 `log hash mismatch`，无法在 PR 合并前阻断，因为：

1. 没有正式 spec 声明「已合入 main 的 Evidence 文件不可改写」；
2. 没有正式 gate 在 PR diff 阶段扫描 Evidence 文件的 `M`/`D` 状态；
3. `main-first-contract` 的 trusted JSONL path ignore list 不覆盖 `.log`、
   `.md`、`.json` 等历史真值扩展名。

本 Spec 填补这一信任缺口。

## 2. 范围

本 Spec 约束以下路径上的「合入 main 之后」修改行为：

| Path Pattern | 类型 | Immutable 等级 |
|---|---|---|
| `evidence/**/*.log` | CLI / harness / xtask 输出真值 | L1 Strict |
| `evidence/**/*.contract.json` | Change Contract（已被 `main-first-contract` 保护） | L1 Strict（双重保护） |
| `evidence/**/*.json` | 结构化 Evidence 真值（非 contract） | L1 Strict |
| `evidence/**/*.md` | 人读 Evidence 摘要、README、approval packet | L2 Soft（见 §5 例外） |

等级语义：

- **L1 Strict**：合入 main 后，PR diff 中出现 `M`/`D` → 立即 FAIL，无白名单兜底。
- **L2 Soft**：合入 main 后，PR diff 中出现 `M`/`D` → FAIL，但允许通过
  correction evidence 显式声明覆盖（见 §5 例外 (c)）。

**不在范围内**：

- `evidence/README.md`（顶层索引，允许同步追加新条目；不允许改写既有段落语义）；
- 运行时 Evidence chain 内部记录的字段不可变性（由 SPEC-EVIDENCE-002 §9.2 与 §17.2
  覆盖）；
- Evidence 内容格式（由 SPEC-EVIDENCE-002 §11 canonical V1 规定）。

## 3. 不变量

```text
I1 已合入 main 的 Evidence 文件默认 immutable。
   PR diff 中 evidence/** 的 M 或 D 状态 → FAIL。

I2 新增（A）允许。
   Evidence 体系是 append-only；每个变更应通过新文件表达，不替换原始内容。

I3 路径迁移必须通过 correction evidence + manifest 表达，不替换原始内容。
   原文件保留原内容；新位置写新 Evidence；manifest 记录 from→to 映射。

I4 hash-bound Evidence 的内容变更必须 fail closed，输出精确路径。
   不静默跳过；不「自动选择较长链」；不自动修复（对齐 SPEC-EVIDENCE-002 §12.4 / §17.4）。
```

## 4. 判定逻辑

输入：`BASE_SHA`（PR base / 合并前 main）、`HEAD_SHA`（PR head）。

```text
git diff --name-status "$BASE_SHA".."$HEAD_SHA" -- \
  'evidence/**/*.log' \
  'evidence/**/*.contract.json' \
  'evidence/**/*.json' \
  'evidence/**/*.md'
```

按行解析 `<status>\t<path>`：

| Status | 判定 | 备注 |
|---|---|---|
| `A` | PASS | append 允许 |
| `M` | FAIL | 修改历史 Evidence |
| `D` | FAIL | 删除历史 Evidence |
| `R` (rename) | FAIL 除非 manifest 校正 | 见 §5 (c) |
| `C` (copy) | PASS | 等同 append |
| 其他 / 解析失败 | FAIL | fail closed |

输出：JSON

```json
{
  "status": "PASS | FAIL",
  "violations": [
    { "status": "M", "path": "evidence/ci/foo.log" }
  ],
  "base_sha": "...",
  "head_sha": "..."
}
```

退出码：`0` = PASS；`1` = FAIL；`2` = SKIP（仅当 Evidence 目录不存在或 git 不可用）。

## 5. 例外

仅以下情况允许改写或删除历史 Evidence，**且必须留下 correction evidence 记录**：

- **(a) 法务 / 合规要求**：例如 GDPR 删除请求、法院命令、隐私违规原文。必须在
  correction evidence 中引用具体法规条款，并由 Risk Owner 签字。
- **(b) Risk Owner 批准**：correction evidence 必须包含 Risk Owner 显式 approval
  摘要、批准时间、批准范围（精确路径列表）。
- **(c) 通过正式 RFC 的路径迁移**：例如 `.agent/specs/` → `.agents/ssot/` 这类
  结构性重命名。原文件保留（或转为指针），新位置写新 Evidence；manifest 文件
  `evidence/corrections/<YYYY-MM-DD>-<slug>.json` 记录 `from` → `to` 映射、
  RFC 编号、approval 列表。Gate 在 manifest 中找到匹配条目时把对应路径
  的 `M`/`R`/`D` 判定降级为 PASS（manifest 本身是 append-only，受本 spec 保护）。

例外 (c) 的 correction evidence 路径示例：

```text
evidence/corrections/2026-07-19-specs-to-ssot.json
```

Schema（草案）：

```json
{
  "correction_id": "2026-07-19-specs-to-ssot",
  "rfc": "docs/specs/agent-ssot-migration.md",
  "approved_by": ["maintainer-A", "maintainer-B"],
  "risk_owner_approval_digest": "sha256:...",
  "moves": [
    { "from": "evidence/ci/old.log", "to": "evidence/ci/new.log",
      "content_preserved": true }
  ]
}
```

`content_preserved: true` 时 gate 仍要校验 `from` 与 `to` 的内容 digest 相同；
不同 → FAIL。

## 6. Gate 接口

```text
Gate 名:           evidence-immutability
实现:              .agent/gates/evidence-immutability.sh
输入:
  BASE_SHA  (环境变量或 $1)
  HEAD_SHA  (环境变量或 $2)
输出:
  stdout   JSON { "status": "...", "violations": [...], "base_sha": "...", "head_sha": "..." }
  exit 0   PASS
  exit 1   FAIL（含至少一条 violation）
  exit 2   SKIP（evidence/ 目录不存在 或 git 不可用）
```

**当前状态：草案，不接入 `runner.sh`**（runner.sh 是硬编码 gate 函数列表，不动态扫描
`*.sh`）。接入需 RFC 批准后修改 `runner.sh` 显式注册 `gate_evidence_immutability`。

## 7. 激活路径

```text
Step 1  Draft Spec             ← 本文件（2026-07-19）
Step 2  Gate 草案              ← .agent/gates/evidence-immutability.sh（手动执行）
Step 3  正式 RFC               ← docs/specs/evidence-immutability.md
Step 4  两名 Maintainer approval + Risk Owner review
Step 5  SPEC-EVIDENCE-002 集成  ← 在 §0 或 §22 引用本 spec 作为历史层补充
Step 6  MAIN_FIRST_POLICY 升级  §12 Draft → Normative
Step 7  runner.sh 注册         gate_evidence_immutability 加入 P1_GATES
Step 8  CI 接入                .github/workflows/ci.yml 调用
```

每步必须独立批准。在 Step 7 之前，gate 仅手动执行；不得宣称 Evidence immutability
已 enforced。Step 8 之前不得宣称 CI 强制。

## 8. 与 SPEC-EVIDENCE-002 的关系（补充而非替代）

| 层 | Spec | 职责 |
|---|---|---|
| 运行时 | SPEC-EVIDENCE-002 §9.2 / §12 / §17 | chain record 不变量、fork 检测、tail truncation、签名 checkpoint |
| 仓库历史 | 本 Spec（Draft） | `git diff` 层 M/D 阻断、correction evidence、manifest 校正 |

两者不重叠：
- 运行时篡改（不动文件，只动数据库行）由 SPEC-EVIDENCE-002 检测；
- 文件历史改写（不动数据库，只动 git tree）由本 Spec 检测；
- 同时改写文件和数据库 → 两者都失败（defense in depth）。

本 Spec 不修改 SPEC-EVIDENCE-002 的任何 P0 冻结条款（`hash_bytes`、Debug→digest、
生产 InMemory、「不可篡改」措辞）。

## 9. 不目标

- 不规定 Evidence 内容格式（canonical V1 编码归 SPEC-EVIDENCE-002 §11）。
- 不规定 Evidence 保留期限（retention 由后续 `evidence-retention-spec` 承担）。
- 不规定运行时 chain 验证（归 SPEC-EVIDENCE-002 §17 与 `evidence-cli verify`）。
- 不替代 `main-first-contract`（contract 仍由 trusted workflow 验证；本 spec 仅延伸
  保护范围到 `.log`、`.json`、`.md`）。
- 不阻止新增 Evidence（append 是允许的，I2）。

## 10. 状态

**Draft**。激活要求：

- 正式 RFC（`docs/specs/evidence-immutability.md`）经 `Approved`；
- 两名 Maintainer approval；
- Risk Owner review（资金 / 合规 / 安全维度签字）。

在以上条件满足前，本 Spec 不具强制性，gate 不接入 CI，policy §12 保持 Draft。
