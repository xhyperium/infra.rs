# 生产签核包模板（Production Sign-off Pack）

> **DEFER-7**：仅模板。**禁止 Agent 代签。**  
> 复制本文件到 `docs/plans/releases/<version>-signoff.md`（或发布 PR 附件）后由 **Maintainer** 填写。

---

## 元信息

| 字段 | 值 |
|------|----|
| 版本 / Tag | `vX.Y.Z` |
| 关联 PR / Issue / beads | |
| 支持矩阵 | 见 [`support-matrix.md`](support-matrix.md)（Linux x86_64 + MSRV 1.85） |
| 签核日期 | `YYYY-MM-DD` |
| 签核人（Maintainer） | **必须人类**；GitHub `@handle` |

---

## 红线

```text
Maintainer only — Agent must not sign.
Agent 可准备证据清单与勾选建议，不得填写「签核人」或勾选「已签核」。
```

---

## L1 — 正确性与不变量

- [ ] 核心 crate 单元 / 集成测试在官方矩阵通过（`cargo test --workspace` 或 nextest）
- [ ] 无已知的公开可达 panic / 静默错误（对照生产就绪报告与 decimalx 合同）
- [ ] 破坏性 API 已在 CHANGELOG / PR 标题显式标注（若有）

**证据链接 / 命令输出摘要：**

```text
（粘贴 CI run URL 或本地验证摘要）
```

---

## L2 — API / SemVer

- [ ] `node scripts/quality-gates/check-public-api.mjs` 通过，或已批准 baseline 更新
- [ ] 若有 public API 删除/签名变更：MAJOR 或显式 breaking 标签 + 迁移说明
- [ ] 版本号符合 [`VERSIONING.md`](VERSIONING.md)

**证据：**

```text
```

---

## L3 — 平台与工具链

- [ ] 支持矩阵未静默扩大或缩小；变更已改 `support-matrix.md`
- [ ] MSRV 1.85 job 绿
- [ ] 仅声明 Linux x86_64 为官方支持（除非本签核显式升级矩阵）

**证据：**

```text
```

---

## L4 — 安全与供应链

- [ ] `cargo deny check` / security workflow 无未处理 CRITICAL
- [ ] 无密钥、证书、`.env`、`.claude/*.local.json` 进入版本库
- [ ] 依赖新增已说明理由（若有）

**证据：**

```text
```

---

## L5 — 可运维与文档

- [ ] 用户可见错误信息 / 迁移说明已更新（中文治理文档 + 必要 crate README）
- [ ] SSOT 对齐文档未宣称未落地能力为 COMPLETE/stable（诚实边界）
- [ ] 回滚路径已知（上一 tag / revert PR）

**证据：**

```text
```

---

## 签核结论（仅 Maintainer）

| 结论 | 选择（择一） |
|------|----------------|
| **GO** | 允许打 tag / 发布 |
| **GO with follow-ups** | 允许发布；follow-up 列表如下 |
| **NO-GO** | 阻塞发布；原因如下 |

**Follow-ups / 阻塞原因：**

```text
```

**Maintainer 手写签名（GitHub handle + 日期）：**

```text
Signed-off-by: @________  YYYY-MM-DD
```

---

## Agent 使用说明

1. Agent 可复制本模板、预填证据链接、列出未决项。  
2. Agent **不得**填写签核人、**不得**选择 GO/NO-GO 作为最终结论、**不得**代写 `Signed-off-by`。  
3. 完成后将路径交给 Maintainer；本模板本身不构成任何版本的生产批准。
