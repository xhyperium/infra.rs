#!/usr/bin/env node

/**
 * sync-ssot-docs.mjs — 拉取最新 origin、处理冲突、提示解��
 *
 * 用法：
 *   node scripts/docs/sync-ssot-docs.mjs
 *
 * 步骤：
 *   1. `git pull --rebase origin main` 拉取最新
 *   2. 如有冲突，列出冲突文件并提示手动解决
 *   3. 无冲突时直接输出成功
 */

import { execSync } from "node:child_process";

function run(cmd, opts = {}) {
  try {
    return execSync(cmd, { encoding: "utf8", stdio: opts.silent ? "pipe" : "inherit", ...opts });
  } catch {
    if (opts.quiet) return null;
    return null; // Soft-fail: continue script execution on git errors
  }
}

// Step 1: pull with rebase
console.log("▶ git pull --rebase origin main");
try {
  run("git pull --rebase origin main", { silent: true });
} catch {
  // Pull failed — likely already up-to-date or non-tracking branch; continue
  console.log("  (non-tracking or already up-to-date, continuing)");
}

// Step 2: check for conflicts
const conflicts = run("git diff --name-only --diff-filter=U", { silent: true, quiet: true });
const conflictFiles = (conflicts || "").trim();

if (conflictFiles) {
  console.log("\n⚠️  冲突文件（需手动解决）：");
  conflictFiles.split("\n").forEach((f) => console.log(`   ${f}`));
  console.log("\n解决步骤：");
  console.log("  1. 编辑冲突文件，搜索 <<<<<<< 找到冲突标记");
  console.log("  2. 保留本地更改或 origin 版本，删除 <<<<<<< ======= >>>>>>> 标记");
  console.log("  3. git add <冲突文件>");
  console.log("  4. git rebase --continue");
  console.log("  5. 重新运行 node scripts/docs/sync-ssot-docs.mjs");
  process.exit(1);
}

console.log("✅ 无冲突，可继续编辑对齐文档");
console.log("\n下一步：");
console.log("  编辑 docs/ssot/*-ssot-alignment.md 或 gap-matrix.md");
console.log("  完成后提交：git add docs/ssot/ && git commit -m 'docs(ssot): sync alignment'");
