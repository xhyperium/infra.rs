# PR #221 merged-by 审查 — 修复建议

## P0：增加 merged_by 空值校验

```diff
# evolution-pr-merge.yml — 解析 PR 信息 step

  merged_by = os.environ.get("PR_MERGED_BY", "").strip()
+ # 防御：拒绝空或无效的 merged_by
+ if not merged_by or merged_by in ("null", "None", "@"):
+     print("⚠️ merged_by 为空/无效，跳过归因")
+     merged_by = owner

  with open(os.environ["GITHUB_OUTPUT"], "a") as f:
      f.write(f"merged_by=@{merged_by}\n")
```

## P1：验证 merge_commit_sha 属于 main 分支

```diff
# evolution-pr-merge.yml — 在 checkout 之后添加新 step

+ - name: 验证 merge commit 完整性
+   if: github.event.pull_request.merged == true
+   run: |
+     MERGE_SHA="${{ github.event.pull_request.merge_commit_sha }}"
+     git fetch origin main
+     if ! git merge-base --is-ancestor "$MERGE_SHA" origin/main; then
+       echo "::error::merge_commit_sha ${MERGE_SHA} 不是 main 的祖先"
+       exit 1
+     fi
+     echo "✅ merge_commit_sha ${MERGE_SHA:0:7} 已验证"
```

## P2：考虑替换 pull_request 为 pull_request_target

```diff
# evolution-pr-merge.yml

  on:
-   pull_request:
+   pull_request_target:
      types: [closed]
      branches: [main]
```

**权衡**：`pull_request_target` 始终使用 main 分支的 workflow 定义，避免 PR 修改的 workflow 执行。但需要确认 EvolutionLedger 路径与 `pull_request_target` 上下文兼容。

## P3：pre-push hook — 验证 gh pr merge 返回的 SHA

```diff
# .githooks/pre-push

- if gh pr merge "$PR_NUM" --squash --delete-branch --admin 2>&1; then
+ MERGE_OUTPUT=$(gh pr merge "$PR_NUM" --squash --delete-branch --admin 2>&1)
+ MERGE_RC=$?
+ if [ "$MERGE_RC" -eq 0 ]; then
+     echo "$MERGE_OUTPUT"
+     # 提取 merge commit SHA 用于审计日志
+     SQUASH_SHA=$(echo "$MERGE_OUTPUT" | grep -oE '[0-9a-f]{40}' | head -1)
+     if [ -n "$SQUASH_SHA" ]; then
+         echo "   📌 squash SHA: ${SQUASH_SHA:0:7}"
+     fi
```
