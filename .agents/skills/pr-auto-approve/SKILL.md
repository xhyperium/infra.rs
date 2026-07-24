---
name: pr-auto-approve
description: Approve open GitHub PRs as @liukongqiang5 via LIUKONGQIANG5_APPROVE_TOKEN (not author self-approve). ONLY when the user explicitly asks to approve/merge/auto-approve, or after explicit handoff to land a PR. Never call by default.
---

# PR Auto-Approve（@liukongqiang5）

用 **第二维护者账号** 对 open PR 提交 `APPROVE`，绕过「作者不能批自己」与
`require_last_push_approval`（最后 pusher 不能充当唯一批准）的限制。

**不是** 降低 Ruleset、**不是** `gh pr merge --admin` 绕过 CI。  
批准后是否可 merge 仍取决于 required checks / conversation resolution / auto-merge。

**政策 SSOT**：[`.agents/rules/生效规则摘要.md` §4](../../.agents/rules/生效规则摘要.md)。

## 身份与密钥

| 项 | 值 |
|----|-----|
| 批准账号 | **`liukongqiang5`**（@liukongqiang5） |
| Token 环境变量 | **`LIUKONGQIANG5_APPROVE_TOKEN`**（必填） |
| 默认仓库 | **当前仓库**（`gh`/`git origin` 自动识别；本仓为 `xhyperium/infra.rs`） |

脚本会先 `GET /user` 校验 token 登录名必须等于 `liukongqiang5`，否则 exit 2。

## 何时使用（硬条件）

**全部满足** 才可调用：

1. **用户明确要求** approve / auto-approve / 合入 / land /「继续合并」等（或当前任务已授权「开 PR 并合入」）  
2. PR **open**，作者 **≠** `liukongqiang5`  
3. 不把「检查仍红」说成已可合；required 未绿时仅可 `--auto` **排队**，不得谎称已合  

**默认禁止**：会话未获合入/批准指令时，Agent **只**开 PR 并报告状态，**不**调用本 skill。

**不要** 在以下情况使用：

- 用本 token 批准 **liukongqiang5 自己开的 PR**（self-approve 禁止）  
- required status check 仍红时「假装已可 merge」——本 skill 只做 APPROVE  
- 用 admin merge 代替本 skill（那是另一条路径）  
- 批量批准无关 PR、静默批准  

## 执行（Agent）

### 1. 确认环境

```bash
test -n "${LIUKONGQIANG5_APPROVE_TOKEN:-}" || echo "MISSING LIUKONGQIANG5_APPROVE_TOKEN"
```

### 2. 运行脚本（首选）

从仓库根运行（脚本自动识别当前 repo，一般**不必**再 export `PR_AUTO_APPROVE_REPO`）：

```bash
bash .claude/skills/pr-auto-approve/scripts/approve.sh <pr-number> [review-body]
```

示例：

```bash
bash .claude/skills/pr-auto-approve/scripts/approve.sh 12 \
  "Auto-approve @liukongqiang5: checks green; AI-first path."

# 仅当要批「另一个」仓库的 PR 时才显式指定：
PR_AUTO_APPROVE_REPO=xhyperium/other bash .claude/skills/pr-auto-approve/scripts/approve.sh 42
```

### 3. 可选：resolve review threads + auto-merge

本 skill **默认只 APPROVE**。若用户还要求合入，Agent 在 APPROVE 成功且 checks 绿后可：

```bash
# 解决未 resolve 的 conversation（ruleset required_review_thread_resolution）
# 见 scripts 外：gh api graphql resolveReviewThread

gh pr merge <n> --squash --auto   # 等人审/检查齐后自动合
# 或在已 APPROVED 且 checks 全绿时：
gh pr merge <n> --squash
```

### 4. 回报

向用户报告：

- token 身份是否为 @liukongqiang5  
- PR 号、作者、head SHA  
- APPROVE 是否幂等跳过 / 新建  
- 当前 `mergeStateStatus` / `reviewDecision`（`gh pr view`）

## 退出码

| 码 | 含义 |
|----|------|
| 0 | 已 APPROVED 或幂等成功 |
| 1 | 用法错误 |
| 2 | token 缺失或身份不是 liukongqiang5 |
| 3 | GitHub API 失败 |
| 4 | PR 非 open / token 用户即作者 |

## 可选环境变量

| 变量 | 默认 |
|------|------|
| `LIUKONGQIANG5_APPROVE_TOKEN` | （必填） |
| `PR_AUTO_APPROVE_EXPECTED_LOGIN` | `liukongqiang5` |
| `PR_AUTO_APPROVE_REPO` | 未设置时：`gh repo view` → `git origin` → `xhyperium/infra.rs` |
| `PR_AUTO_APPROVE_API` | `https://api.github.com` |

## 安全

- **禁止** 把 token 写入仓库、日志、commit、PR body  
- **禁止** 在 skill 输出中打印 token  
- 仅对 **open** PR 操作；合并后的 PR 直接失败  
- 不修改 Ruleset、不关闭 branch protection  

## 与 CICD 目标的关系

`CICD-3-5MIN-KERNEL-LOCAL` 等 recovery 合入时：作者推送后由本 skill 以 @liukongqiang5 批准，满足
`required_approving_review_count=1` 与 `require_last_push_approval=true`。

## 文件

| 路径 | 作用 |
|------|------|
| `scripts/approve.sh` | 可执行入口 |
| `agents/openai.yaml` | Codex/Agents 展示名 |
