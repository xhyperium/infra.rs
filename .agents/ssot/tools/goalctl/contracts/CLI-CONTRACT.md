# goalctl CLI 合同

```text
Document:     CLI-CONTRACT
Version:      1.0.0
Status:       ACTIVE（PR-0A 形状；实现未授权）
Package:      xhyper-goalctl（planned）
Binary:       goalctl
Authority:    DECISION-PACK-001 · CR-20260716
```

## 1. 命令面

### Phase 1（0.1.0 Complete）

| 命令 | 作用 | 最早波次 |
|------|------|----------|
| `goalctl --version` / `goalctl version` | 版本字符串 | PR-1 |
| `goalctl doctor` | 仓库健康与边界检查 | PR-1 |
| `goalctl index` | 确定性 Repository Index | PR-1 |
| `goalctl resolve` | Authority Snapshot | PR-2 |
| `goalctl artifact inspect\|index` | Artifact 解析与索引 | PR-2 |
| `goalctl reconcile` | 五维状态调和 | PR-3 |
| `goalctl compile` | Task Pack / Prompt Pack | PR-4 |

### 明确不在 Phase 1

`run` / `apply` / `pr` / `merge` / `gate pass` / `agent` / 任何写代码或改 Task Pack 的命令。

## 2. 全局标志

| 标志 | 说明 |
|------|------|
| `--json` | stdout **仅** JSON（见 cli-output.schema.json）；人类可读日志走 stderr |
| `--state-dir <path>` | 覆盖运行态根（D05）；测试/CI 必用 |
| `--repo-root <path>` | 可选；默认自 cwd 向上发现含 `.git` 与 workspace `Cargo.toml` 的根 |
| `--source-commit <sha>` | 可选；默认 `HEAD` 解析的 40 位 SHA；禁止用 branch 名当作 subject |
| `--trust-level <level>` | `TRUSTED_INTERNAL` / `TRUSTED_BOT` / `UNTRUSTED_FORK` / `UNTRUSTED_EXTERNAL_SOURCE`；默认策略见下 |
| `--help` | 非 0 数据路径；exit 0 |

默认 trust-level：

- 本仓 canonical remote / monorepo 身份匹配 → 可 `TRUSTED_INTERNAL`
- 否则 → `UNTRUSTED_EXTERNAL_SOURCE` 或 `DEGRADED` 身份，且不得输出「可发布 Evidence」语义

## 3. Exit codes

| Code | 名 | 含义 |
|-----:|----|------|
| 0 | OK | 成功；无 error 级 diagnostic |
| 1 | USAGE | 参数/子命令错误 |
| 2 | POLICY | 策略失败（含 `.config/goal` 存在、Authority Policy 缺失、scope 非法） |
| 3 | NOT_PROVEN | 证据不足 / UNKNOWN；**不是**成功 |
| 4 | CONFLICT | 同级声明冲突 BLOCKED |
| 5 | IO | 文件系统/Git 读取失败 |
| 6 | SCHEMA | 输入/输出 schema 校验失败 |
| 7 | INTERNAL | 内部不变量破坏 |
| 10 | UNSUPPORTED | 命令在当前版本未实现（版本矩阵） |

规则：

- `--json` 时，结构化失败仍应尽量写出合法 cli-output（`ok=false`）再以非 0 退出。
- 不得用 exit 0 表示「有 warning 但假装 VERIFIED」。

## 4. stdout / stderr

| 流 | `--json` | 默认（人类） |
|----|----------|----------------|
| stdout | **仅** 一个 JSON 文档（canonical） | 表格/文本摘要 |
| stderr | diagnostics 人类行、进度、调试 | 同左 |

禁止：JSON 与日志混写 stdout；禁止 ANSI 进入 `--json` stdout。

## 5. Diagnostic 码（GC-*）

前缀 **`GC-`** = goalctl **Diagnostic**，**不是** Goal Gate G0–G11。

goalctl **不得** 自行改写 `.agents/ssot/**/gate/` 为 PASS。

| 码 | 严重度 | 含义 |
|----|--------|------|
| GC-CONFIG-GOAL-PRESENT | error | 发现 `.config/goal` |
| GC-AUTHORITY-POLICY-MISSING | error | 无 authority-policy.yaml |
| GC-AUTHORITY-CONFLICT | error | 同 rank 结构冲突 |
| GC-NOT-PROVEN | error/warn | 证据不足 |
| GC-STALE-EVIDENCE | error | Evidence 未绑当前 subject |
| GC-SCHEMA-INVALID | error | schema 失败 |
| GC-STATE-DIR | info | 报告 resolved state-dir |
| GC-IDENTITY-DEGRADED | warning | repository identity 非 FULL |
| GC-UNSUPPORTED-COMMAND | error | 版本未实现该命令 |
| GC-LEGACY-NO-PASS | error | Legacy 叙述试图充当正式 PASS |

可在实现中 additive 扩展 `GC-*`；删除或改义需升 CLI 合同 minor/major。

## 6. doctor 最低检查（PR-1）

MUST 报告：

1. repo root
2. source_commit（40 hex）
3. repository_identity + confidence
4. resolved `state_dir`
5. `config_goal_present == false`（true → exit POLICY）
6. `.agents/ssot` 是否存在
7. authority-policy 路径是否可读
8. Cargo metadata 是否可加载（失败 → diagnostic，是否 fail 由实现表定义，默认 error）
9. dirty worktree（warning；Snapshot 不得被 dirty 内容污染）

## 7. index 最低行为（PR-1）

- 输出符合 `repository-index.schema.json`
- 模块按 `module` 排序
- 路径仓库相对
- 同 `source_commit` 两次运行 canonical JSON 一致

## 8. 兼容性

- CLI 合同 **1.x**：additive 标志/诊断码允许；删除标志或改变 exit 语义 → major
- JSON `schema_version` 与本目录 schema 对齐
- 未实现命令：exit **10** + `GC-UNSUPPORTED-COMMAND`，不得假装成功

## 9. 非目标

- 不启动 Agent
- 不 `git push` / 开 PR
- 不写 G0–G11 PASS
- 不创建 `.config/goal`
- 不把业务状态写入 Cargo `target` / `.cargo/target` / 历史 `../.cargo/target`
