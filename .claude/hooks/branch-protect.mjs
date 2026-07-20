#!/usr/bin/env node
// Branch Protect — Stop Hook
//
// 信息护栏：会话收尾时检测当前分支是否有未合入 origin/main 的提交。
// 若存在未合并提交，向 stderr 输出醒目警告，提醒禁止直接删除分支或
// 切换到 main 前丢弃工作。此 hook 不阻塞会话，职责是信息提醒。
//
// 规则来源：CLAUDE.md §分支保护（禁止自动删除未合并分支）
//  "Stop/SessionEnd hook 切换 main 前必须验证：
//   git log origin/main..HEAD --oneline | wc -l（>0 则禁止删除）或 PR 已合并"
//
// 不阻塞说明：Stop hook 性质使然，仅输出 stderr 警告，
// process.exit(0) 无论是否检测到未合并提交。

import { execSync } from "child_process";

function run(cmd) {
  try {
    return execSync(cmd, {
      encoding: "utf8",
      cwd: process.cwd(),
      stdio: ["pipe", "pipe", "pipe"],
    }).trim();
  } catch {
    return "";
  }
}

function main() {
  // 1. 读取当前分支名
  const branch = run("git rev-parse --abbrev-ref HEAD");
  if (!branch) {
    // 非 git 仓库或 detached HEAD 无法判定，静默退出
    return;
  }

  // 4. 若在 main 分支，静默退出
  if (branch === "main" || branch === "master") {
    return;
  }

  // 2. 计算当前分支领先 origin/main 的未合并提交
  const aheadLog = run("git log --oneline origin/main..HEAD");
  const commits = aheadLog ? aheadLog.split("\n").filter(Boolean) : [];

  // 4. 若领先数为 0，静默退出
  if (commits.length === 0) {
    return;
  }

  // 3. 输出醒目警告到 stderr
  const preview = commits.slice(0, 5);
  const remaining = commits.length - preview.length;

  const lines = [];
  lines.push("");
  lines.push("══════════════════════════════════════════════════════");
  lines.push("[BranchProtect] ⚠️  检测到未合并提交，禁止误删分支！");
  lines.push("");
  lines.push(`  当前分支: ${branch}`);
  lines.push(`  未合并提交: ${commits.length} 个 (领先 origin/main)`);
  lines.push("");
  lines.push("  未合并提交预览:");
  for (const c of preview) {
    lines.push(`    ${c}`);
  }
  if (remaining > 0) {
    lines.push(`    ... (还有 ${remaining} 个)`);
  }
  lines.push("");
  lines.push("  ❌ 禁止操作:");
  lines.push("    - git branch -D " + branch + "  (强制删除分支会丢失提交)");
  lines.push("    - git checkout main 前未推送/未建 PR  (切换后分支可能被 GC)");
  lines.push("");
  lines.push("  ✅ 建议操作:");
  lines.push(`    $ git push -u origin HEAD          # 推送当前分支到远端`);
  lines.push(`    $ gh pr create --base main         # 创建 PR 合入 main`);
  lines.push("");
  lines.push("  规则来源: CLAUDE.md §分支保护");
  lines.push("══════════════════════════════════════════════════════");
  lines.push("");

  console.error(lines.join("\n"));
}

main();
process.exit(0);
