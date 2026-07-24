#!/usr/bin/env node
// Codex Worktree Guard — SessionStart / UserPromptSubmit Hook
//
// Codex 不支持 PreToolUse block，此钩子通过上下文注入和告警实现 advisory 级约束。
//
// 功能：
//   1. SessionStart：检测当前工作目录是否在 worktree 内，注入禁止事项
//   2. UserPromptSubmit（当 CODEX_HOOK_EVENT=prompt）：扫描用户 prompt 中的危险命令模式
//
// 约束来源：.agents/rules/worktree-policy.md · 宪章 §6.0.5

import { execSync } from "node:child_process";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, "..", "..");

const LINES = "═".repeat(60);
const isPromptMode = process.env.CODEX_HOOK_EVENT === "prompt" ||
  process.argv.includes("--prompt-mode");

function getCwd() {
  try {
    return process.cwd();
  } catch {
    return "";
  }
}

function isInWorktree(cwd) {
  return cwd.includes(".worktrees/") || cwd.includes(".worktree/");
}

function getGitBranch() {
  try {
    return execSync("git rev-parse --abbrev-ref HEAD", {
      encoding: "utf8",
      cwd: getCwd(),
      timeout: 3000,
      stdio: ["pipe", "pipe", "pipe"],
    }).trim();
  } catch {
    return "unknown";
  }
}

// SessionStart 模式：注入 worktree 约束上下文
function sessionStartCheck() {
  const cwd = getCwd();
  const inWorktree = isInWorktree(cwd);
  const branch = getGitBranch();

  const lines = [];
  lines.push("");
  lines.push(LINES);
  lines.push("[Codex Worktree Guard] Worktree 合规检查");
  lines.push("");

  if (inWorktree) {
    lines.push(`  ✅ 当前在 worktree 内: ${cwd}`);
    lines.push(`  分支: ${branch}`);
    lines.push("");
    lines.push("  约束提醒：");
    lines.push("    • 禁止推 main、禁止 force push");
    lines.push("    • 禁止修改 .github/CODEOWNERS");
    lines.push("    • 禁止使用 INFRA_WORKTREE_BYPASS=1");
    lines.push("    • 危险命令(rm -rf / git push --force)仅在 .worktrees/ 或 /tmp/ 内允许");
  } else {
    lines.push(`  ⛔ 当前在主仓: ${cwd}`);
    lines.push("");
    lines.push("  ╔══════════════════════════════════════════════════╗");
    lines.push("  ║  Codex 必须在 worktree 内开发（宪章 §6.0.5）      ║");
    lines.push("  ╚══════════════════════════════════════════════════╝");
    lines.push("");
    lines.push("  开工步骤：");
    lines.push("    node scripts/worktree/worktree.mjs create <type>/<id>-<slug>");
    lines.push("    cd .worktrees/<branch-name>");
    lines.push("");
    lines.push("  禁止事项：");
    lines.push("    • 不得在主仓直接 Write/Edit 已跟踪文件");
    lines.push("    • 不得在 main 上 git commit");
    lines.push("    • 不得在主仓 git checkout -b / switch -c");
  }

  lines.push("");
  lines.push(LINES);
  lines.push("");

  // Codex 钩子 stdout 会被注入为上下文
  process.stdout.write(lines.join("\n"));
}

// UserPromptSubmit 模式：检测危险命令模式
function promptCheck() {
  const prompt = process.stdin.read() || "";

  const dangerousPatterns = [
    { pattern: /rm\s+-rf?\s+/g, label: "rm -rf（危险删除）" },
    { pattern: /git\s+push\s+--force/g, label: "git push --force（强制推送）" },
    { pattern: /git\s+push\s+-f\b/g, label: "git push -f（强制推送）" },
    { pattern: /git\s+reset\s+--hard/g, label: "git reset --hard（硬重置）" },
    { pattern: /git\s+clean\s+-fd/g, label: "git clean -fd（清理未跟踪）" },
    { pattern: /DROP\s+TABLE/gi, label: "DROP TABLE（删表）" },
  ];

  const warnings = [];
  for (const { pattern, label } of dangerousPatterns) {
    if (pattern.test(prompt)) {
      warnings.push(label);
      pattern.lastIndex = 0;
    }
  }

  if (warnings.length > 0) {
    const lines = [];
    lines.push("");
    lines.push(LINES);
    lines.push("[Codex Worktree Guard] ⚠️  检测到危险命令模式");
    lines.push("");
    for (const w of warnings) {
      lines.push(`    • ${w}`);
    }
    lines.push("");
    lines.push("  请确认：仅在 .worktrees/ 或 /tmp/ 路径内执行；");
    lines.push("  主仓禁止 force push / reset --hard / clean -fd。");
    lines.push(LINES);
    lines.push("");
    process.stdout.write(lines.join("\n"));
  }
}

if (isPromptMode) {
  promptCheck();
} else {
  sessionStartCheck();
}
