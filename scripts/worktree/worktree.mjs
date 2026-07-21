#!/usr/bin/env node
/**
 * worktree.mjs — Git Worktree 管理工具（替代 worktree.mjs）
 *
 * 职责: 创建/列出/删除/清理 Git Worktree。
 *
 * 用法:
 *   node scripts/worktree/worktree.mjs create <branch>
 *   node scripts/worktree/worktree.mjs go <branch>         # 输出路径供 cd 使用
 *   node scripts/worktree/worktree.mjs list
 *   node scripts/worktree/worktree.mjs remove <branch>
 *   node scripts/worktree/worktree.mjs prune
 *   node scripts/worktree/worktree.mjs current
 *
 * SSOT: docs/constitution/06-governance.md §6.0.5 / docs/governance/worktree-policy.md
 * 替代: scripts/worktree/worktree.mjs (已迁移)
 */

import { execSync } from "child_process";
import { existsSync, mkdirSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import process from "process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const WT_BASE = resolve(ROOT, ".worktrees");

function git(cmd, opts = {}) {
  return execSync(`git -C ${ROOT} ${cmd}`, { encoding: "utf8", stdio: opts.silent ? "pipe" : "inherit", ...opts });
}

function die(msg, code = 1) {
  console.error(msg);
  process.exit(code);
}

const cmd = process.argv[2];
const arg = process.argv[3];

switch (cmd) {
  case "create": {
    if (!arg) die("usage: worktree.mjs create <branch>");
    const wtPath = resolve(WT_BASE, arg);
    mkdirSync(WT_BASE, { recursive: true });
    execSync("git fetch origin", { cwd: ROOT, stdio: "inherit" });
    git(`worktree add '${wtPath}' -b '${arg}' origin/main`);
    console.log(`Worktree 已创建`);
    console.log(`  cd ${wtPath}      # 或: wt ${arg}`);
    break;
  }

  case "go": {
    if (!arg) die("usage: worktree.mjs go <branch>");
    const wtPath = resolve(WT_BASE, arg);
    if (existsSync(wtPath)) {
      console.log(wtPath);
    } else {
      console.error(`ERROR: worktree 不存在: ${arg}`);
      git("worktree list", { silent: true });
      process.exit(1);
    }
    break;
  }

  case "list": {
    console.log("Worktrees:");
    const list = git("worktree list", { silent: true }).trim();
    for (const line of list.split("\n")) {
      const [path, hash, branch] = line.split(/\s+/);
      if (path === ROOT) {
        console.log(`  [main]  ${path}`);
      } else {
        const short = path.replace(WT_BASE + "/", "");
        console.log(`  [${short}]  ${path}`);
      }
    }
    break;
  }

  case "remove": {
    if (!arg) die("usage: worktree.mjs remove <branch>");
    const wtPath = resolve(WT_BASE, arg);
    if (existsSync(wtPath)) {
      git(`worktree remove '${wtPath}' --force`);
      console.log(`Worktree 已删除: ${arg}`);
    } else {
      console.error(`ERROR: worktree 不存在: ${arg}`);
      process.exit(1);
    }
    break;
  }

  case "prune": {
    git("worktree prune");
    console.log("已清理过期 worktree");
    break;
  }

  case "current": {
    git("worktree list", { silent: true });
    break;
  }

  default:
    console.log("usage: worktree.mjs {create|go|list|remove|prune|current} [branch]");
    process.exit(1);
}
