#!/usr/bin/env node
/**
 * branch-cleanup.mjs — 分支清理工具
 *
 * 职责: 扫描并清理已合并的本地/远程分支及关联 worktree。
 *
 * 用法:
 *   node scripts/workflow/branch-cleanup.mjs [选项]
 *
 * 选项:
 *   --list          仅列出分支状态，不执行清理（默认）
 *   --clean         删除可安全清理的分支
 *   --force         强制删除（跳过确认）
 *   --dry-run       演练模式，仅输出计划
 *   --branch <name> 只处理指定分支（可重复）
 *   --prune-remote  同时删除已合并 PR 的远程分支
 *   --help          显示帮助信息
 *
 * 清理策略（安全默认）:
 *   - 不删除 main 分支
 *   - 不删除当前分支（除非显式指定 --branch）
 *   - 不删除未关联 PR 的分支（除非显式 --branch）
 *   - squash-merge 通过 gh pr view --json 判定合并状态
 *   - 关联的 .worktrees/<branch> 目录一并清理
 *
 * SSOT: docs/constitution/06-governance.md §6.0.5 / .agents/rules/worktree-policy.md
 */

import { execSync } from "child_process";
import { existsSync, rmSync } from "fs";
import { join, dirname, resolve, basename } from "path";
import readline from "readline";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..", "..");

// ── 颜色 ──────────────────────────────────────────────────
const C = {
  reset:  "\x1b[0m",    bold:   "\x1b[1m",   dim:    "\x1b[2m",
  red:    "\x1b[31m",   green:  "\x1b[32m",  yellow: "\x1b[33m",
  blue:   "\x1b[34m",   magenta:"\x1b[35m",  cyan:   "\x1b[36m",
};

function color(c, text) { return `${c}${text}${C.reset}`; }

// ── 工具函数 ──────────────────────────────────────────────
function git(cmd, opts = {}) {
  try {
    return execSync(`git -C '${ROOT}' ${cmd}`, {
      encoding: "utf8",
      stdio: opts.silent ? "pipe" : "inherit",
      ...opts,
    }).trim();
  } catch (e) {
    if (opts.allowFail) return "";
    throw e;
  }
}

function gh(args, opts = {}) {
  try {
    return execSync(`gh ${args}`, {
      encoding: "utf8",
      stdio: "pipe",
      cwd: ROOT,
      ...opts,
    }).trim();
  } catch {
    return null;
  }
}

function warn(msg)  { console.log(`  ${color(C.yellow, "⚠")} ${msg}`); }
function ok(msg)    { console.log(`  ${color(C.green, "✓")} ${msg}`); }
function info(msg)  { console.log(`  ${color(C.dim, "→")} ${msg}`); }
function die(msg, code = 1) {
  console.error(`${color(C.red, "ERROR:")} ${msg}`);
  process.exit(code);
}

// ── 分支状态枚举 ──────────────────────────────────────────
const Status = {
  ACTIVE:    { label: "ACTIVE",    emoji: "●", color: "green",  desc: "有未合并提交（活跃开发）" },
  MERGED:    { label: "MERGED",    emoji: "✓", color: "green",  desc: "PR 已合并，可安全删除" },
  STALE:     { label: "STALE",     emoji: "⚠", color: "yellow", desc: "远程分支已删除但本地仍存在" },
  NO_PR:     { label: "NO_PR",     emoji: "?", color: "yellow", desc: "未关联 PR（本地开发分支）" },
  CLOSED:    { label: "CLOSED",    emoji: "✗", color: "red",    desc: "PR 已关闭但未合并" },
  CURRENT:   { label: "CURRENT",   emoji: "★", color: "cyan",   desc: "当前所在分支" },
  MAIN:      { label: "MAIN",      emoji: "■", color: "blue",   desc: "主干分支（不删除）" },
  REMOTE_ONLY:{ label: "REMOTE",   emoji: "☁", color: "dim",    desc: "仅远程存在（本地无此分支）" },
  UNKNOWN:   { label: "UNKNOWN",   emoji: "?", color: "dim",    desc: "无法判定状态" },
};

// ── 分支扫描 ──────────────────────────────────────────────
function listLocalBranches() {
  const raw = git("for-each-ref --format='%(refname:short)|%(upstream:short)|%(upstream:track)|%(authordate:iso8601)|%(subject)' refs/heads/", { silent: true });
  if (!raw) return [];
  return raw.split("\n").map(line => {
    const [name, upstream, track, date, subject] = line.split("|");
    return { name, upstream, track, date, subject };
  });
}

function listRemoteBranches() {
  const raw = git("ls-remote --heads origin", { silent: true });
  if (!raw) return [];
  return raw.split("\n").map(line => {
    const parts = line.split("\t");
    const ref = parts[1] || "";
    const name = ref.replace("refs/heads/", "");
    return name;
  }).filter(b => b && b !== "HEAD");
}

function listWorktrees() {
  const raw = git("worktree list", { silent: true, allowFail: true });
  if (!raw) return [];
  return raw.split("\n").map(line => {
    const parts = line.trim().split(/\s+/);
    const path = parts[0];
    const branch = parts.length >= 3 ? parts[2].replace(/[\[\]]/g, "") : "";
    const isBare = line.includes("(bare)");
    return { path, branch, isBare };
  }).filter(w => w.branch && w.branch !== "main");
}

// ── PR 状态查询 ───────────────────────────────────────────
/**
 * 查询分支关联的 PR 状态。返回:
 *   { merged: bool, closed: bool, state: "MERGED"|"OPEN"|"CLOSED"|null }
 */
function checkPrStatus(branchName) {
  // 跳过特殊分支名（包含 / 或 dolt 等非 PR 分支）
  if (branchName.startsWith("__dolt")) return null;

  const result = gh(`pr list --head '${branchName}' --state all --json state,mergedAt,closedAt --limit 1`, { allowFail: true });
  if (!result || result === "[]") return null;

  try {
    const parsed = JSON.parse(result);
    if (parsed.length === 0) return null;
    const pr = parsed[0];
    return {
      merged: pr.state === "MERGED",
      closed: pr.state === "CLOSED",
      state: pr.state,
    };
  } catch {
    return null;
  }
}

// ── 判断分支是否已合并到 main（检查 commit 是否可达） ──────
function isReachableFromMain(branchName) {
  const base = git("merge-base origin/main 'refs/heads/" + branchName + "'", { silent: true, allowFail: true });
  if (!base) return false;
  const branchHead = git("rev-parse 'refs/heads/" + branchName + "'", { silent: true, allowFail: true });
  if (!branchHead) return false;
  // 如果 merge-base 等于分支 HEAD，则该分支的所有提交都在 main 上
  return base === branchHead;
}

// ── 综合判定 ──────────────────────────────────────────────
function classifyBranch(branch, currentBranch) {
  const { name } = branch;

  // main 始终跳过
  if (name === "main") return { ...branch, status: Status.MAIN, safe: false, reason: "主干分支" };
  // 当前分支
  if (name === currentBranch) return { ...branch, status: Status.CURRENT, safe: false, reason: "当前分支" };

  // 检查是否远程分支已删除（stale upstream）
  const upstreamDeleted = branch.track && branch.track.includes("[gone]");
  if (upstreamDeleted) return { ...branch, status: Status.STALE, safe: true, reason: "远程分支已删除" };

  // 检查 commit 是否可达 main（squash merge 后通常不可达，但可用于普通 merge）
  const reachable = isReachableFromMain(name);

  // 查询 PR 状态
  const prStatus = checkPrStatus(name);

  if (prStatus) {
    if (prStatus.merged) {
      return { ...branch, status: Status.MERGED, safe: true, reason: "PR 已合并" };
    }
    if (prStatus.closed) {
      return { ...branch, status: Status.CLOSED, safe: false, reason: "PR 已关闭但未合并（需人工确认）" };
    }
    return { ...branch, status: Status.ACTIVE, safe: false, reason: "PR 开放中" };
  }

  // 无 PR 关联
  if (reachable) {
    return { ...branch, status: Status.MERGED, safe: true, reason: "commit 已包含在 main 中" };
  }

  return { ...branch, status: Status.NO_PR, safe: false, reason: "未关联 PR，可能存在未推送工作" };
}

// ── 清理操作 ───────────────────────────────────────────────
function deleteLocalBranch(name, dryRun) {
  if (dryRun) {
    info(`[DRY-RUN] git branch -D '${name}'`);
    return;
  }
  try {
    git(`branch -D '${name}'`);
    ok(`已删除本地分支: ${name}`);
  } catch (e) {
    warn(`删除本地分支失败: ${name} — ${String(e.message || e).slice(0, 100)}`);
  }
}

function deleteRemoteBranch(name, dryRun) {
  if (dryRun) {
    info(`[DRY-RUN] git push origin --delete '${name}'`);
    return;
  }
  try {
    git(`push origin --delete '${name}'`);
    ok(`已删除远程分支: ${name}`);
  } catch (e) {
    warn(`删除远程分支失败: ${name} — ${String(e.message || e).slice(0, 100)}`);
  }
}

function removeWorktree(branchName, dryRun) {
  const wtDir = join(ROOT, ".worktrees", branchName);

  // 检查 git worktree 是否存在该分支的注册
  const wtList = git("worktree list", { silent: true, allowFail: true });
  const hasRegistered = wtList && wtList.includes(branchName);

  if (hasRegistered) {
    if (dryRun) {
      info(`[DRY-RUN] git worktree remove '.worktrees/${branchName}' --force`);
    } else {
      try {
        git(`worktree remove '.worktrees/${branchName}' --force`);
        ok(`已移除 worktree 注册: ${branchName}`);
      } catch (e) {
        warn(`移除 worktree 注册失败: ${branchName}`);
      }
    }
  }

  // 清理 worktree 目录残留
  if (existsSync(wtDir)) {
    if (dryRun) {
      info(`[DRY-RUN] rm -rf '.worktrees/${branchName}'`);
    } else {
      try {
        rmSync(wtDir, { recursive: true, force: true });
        ok(`已删除 worktree 目录: .worktrees/${branchName}`);
      } catch (e) {
        warn(`删除 worktree 目录失败: .worktrees/${branchName}`);
      }
    }
  }
}

function pruneGit(dryRun) {
  if (dryRun) {
    info("[DRY-RUN] git worktree prune && git fetch origin --prune");
    return;
  }
  git("worktree prune", { silent: true, allowFail: true });
  ok("已清理 worktree 孤儿元数据");
  git("fetch origin --prune", { silent: true, allowFail: true });
  ok("已清理陈旧的远程跟踪引用");
}

// ── 展示分支列表 ──────────────────────────────────────────
function displayBranchList(branches) {
  console.log(`\n${color(C.bold, "分支状态一览")}`);
  console.log(`${color(C.dim, "─".repeat(80))}`);

  // 按状态分组
  const safe = branches.filter(b => b.safe);
  const unsafe = branches.filter(b => !b.safe);

  const statusColor = {
    ACTIVE:  C.green,   MERGED:  C.green,  STALE:   C.yellow,
    NO_PR:   C.yellow,  CLOSED:  C.red,    CURRENT: C.cyan,
    MAIN:    C.blue,    UNKNOWN: C.dim,
  };

  for (const b of branches) {
    const sc = statusColor[b.status.label] || C.dim;
    const indicator = b.safe ? color(C.green, "[可清理]") : color(C.dim, "[保留]");
    const st = b.status;
    console.log(
      `  ${indicator} ${color(sc + C.bold, st.emoji)} ${color(C.bold, b.name)}  ` +
      `${color(sc, st.label)} — ${color(C.dim, b.reason || st.desc)}`
    );
  }

  console.log(`\n${color(C.bold, "汇总:")} ${color(C.green, safe.length)} 可清理, ${color(C.yellow, unsafe.length)} 需保留`);
}

// ── 交互确认 ──────────────────────────────────────────────
async function confirm(message) {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  return new Promise(resolve => {
    rl.question(`${color(C.yellow, message + " [y/N] ")}`, (answer) => {
      rl.close();
      resolve(answer.toLowerCase() === "y" || answer.toLowerCase() === "yes");
    });
  });
}

// ── 参数解析 ──────────────────────────────────────────────
function parseArgs(argv) {
  const opts = {
    list: true,
    clean: false,
    force: false,
    dryRun: false,
    pruneRemote: false,
    branches: [],
    help: false,
  };
  let i = 0;
  while (i < argv.length) {
    switch (argv[i]) {
      case "--list":   opts.list = true; break;
      case "--clean":  opts.list = false; opts.clean = true; break;
      case "--force":  opts.force = true; break;
      case "--dry-run":opts.dryRun = true; break;
      case "--prune-remote": opts.pruneRemote = true; break;
      case "--branch":
        if (i + 1 >= argv.length) die("--branch 缺少参数");
        opts.branches.push(argv[++i]);
        break;
      case "--help":   opts.help = true; break;
      default:
        if (argv[i].startsWith("-")) die(`未知选项: ${argv[i]}`);
        break;
    }
    i++;
  }
  return opts;
}

function showHelp() {
  console.log(`branch-cleanup.mjs — 分支清理工具

用法:
  node scripts/workflow/branch-cleanup.mjs [选项]

选项:
  --list            仅列出分支状态（默认）
  --clean           执行清理（需配合 --force 或交互确认）
  --force           跳过交互确认
  --dry-run         演练模式，仅输出计划，不实际操作
  --branch <name>   只处理指定分支（可重复多次）
  --prune-remote    同时删除 PR 已合并的远程分支
  --help            显示帮助信息

安全护栏:
  - main 分支永不删除
  - 当前分支默认不删除
  - 未关联 PR 的分支不会被自动清理（除非显式指定 --branch）
  - squash-merge 通过 gh API 检测（不依赖 commit hash）

示例:
  # 查看所有分支状态
  node scripts/workflow/branch-cleanup.mjs --list

  # 演练清理
  node scripts/workflow/branch-cleanup.mjs --clean --dry-run

  # 强制清理所有安全分支（含远程）
  node scripts/workflow/branch-cleanup.mjs --clean --force --prune-remote

  # 只清理特定分支
  node scripts/workflow/branch-cleanup.mjs --clean --branch chore/old-work
`);
}

// ── 插件检查：识别陈旧的 Dolt 分支 ─────────────────────────
function findDoltBranches(remoteBranches) {
  return remoteBranches.filter(b => b.startsWith("__dolt"));
}

// ── 主逻辑 ────────────────────────────────────────────────
async function main() {
  const opts = parseArgs(process.argv.slice(2));

  if (opts.help) {
    showHelp();
    process.exit(0);
  }

  const currentBranch = git("rev-parse --abbrev-ref HEAD", { silent: true });
  const allLocal = listLocalBranches();
  const allRemote = listRemoteBranches();

  // 过滤目标分支
  let candidates;
  if (opts.branches.length > 0) {
    candidates = allLocal.filter(b => opts.branches.includes(b.name));
    if (candidates.length === 0) {
      warn("指定的分支不存在于本地");
    }
  } else {
    candidates = allLocal;
  }

  // 分类
  const classified = candidates
    .map(b => classifyBranch(b, currentBranch))
    .sort((a, b) => {
      // safe first, then by name
      if (a.safe !== b.safe) return a.safe ? -1 : 1;
      return a.name.localeCompare(b.name);
    });

  // 仅远程的分支
  const localNames = new Set(allLocal.map(b => b.name));
  const remoteOnly = allRemote.filter(r => !localNames.has(r));

  // --list 模式
  if (opts.list) {
    displayBranchList(classified);

    if (remoteOnly.length > 0) {
      console.log(`\n${color(C.dim, "仅远程的分支:")}`);
      for (const r of remoteOnly) {
        console.log(`  ${color(C.dim, "☁")} ${color(C.dim, r)}`);
      }
    }

    const doltBranches = findDoltBranches(allRemote);
    if (doltBranches.length > 0) {
      console.log(`\n${color(C.dim, "Dolt 分支 (beads 数据同步):")}`);
      for (const d of doltBranches) {
        console.log(`  ${color(C.dim, "@")} ${color(C.dim, d)}`);
      }
    }

    // 显示关联的 worktree
    const worktrees = listWorktrees();
    if (worktrees.length > 0) {
      console.log(`\n${color(C.dim, "关联 Worktree:")}`);
      for (const wt of worktrees) {
        const matched = classified.find(b => b.name === wt.branch);
        const note = matched && matched.safe ? color(C.green, " (可清理)") : "";
        console.log(`  ${color(C.dim, "📂")} ${color(C.dim, wt.path)} → ${color(C.dim, wt.branch)}${note}`);
      }
    }

    const safeCount = classified.filter(b => b.safe).length;
    if (safeCount > 0) {
      console.log(color(C.green, `\n发现 ${safeCount} 个可安全清理的分支。执行清理:`));
      console.log(color(C.dim, `  node scripts/workflow/branch-cleanup.mjs --clean --dry-run`));
    }

    return;
  }

  // --clean 模式
  const safeToClean = classified.filter(b => b.safe);

  if (safeToClean.length === 0) {
    console.log(color(C.dim, "没有可安全清理的分支。"));
    return;
  }

  console.log(`\n${color(C.bold, "准备清理以下分支:")}`);
  for (const b of safeToClean) {
    const wc = b.status.color || "green";
    console.log(`  ${color(C[wc], b.status.emoji)} ${color(C.bold, b.name)} — ${color(C.dim, b.reason)}`);
  }

  // 确认
  if (!opts.force) {
    const confirmed = await confirm("确认删除以上分支?");
    if (!confirmed) {
      console.log(color(C.dim, "已取消。"));
      return;
    }
  }

  // 执行清理
  console.log(color(C.bold, `\n执行清理...`));
  const isDry = opts.dryRun;
  if (isDry) console.log(color(C.yellow, "(DRY-RUN 模式 — 不会实际操作)\n"));

  for (const b of safeToClean) {
    console.log(color(C.dim, `\n处理: ${b.name}`));
    deleteLocalBranch(b.name, isDry);
    removeWorktree(b.name, isDry);

    if (opts.pruneRemote && (b.status === Status.MERGED || b.status === Status.STALE)) {
      deleteRemoteBranch(b.name, isDry);
    }
  }

  pruneGit(isDry);

  console.log(`\n${color(C.green, "✓ 分支清理完成")}`);
}

// Run
main().catch((e) => {
  console.error(`${color(C.red, "ERROR:")} ${e.message || e}`);
  process.exit(1);
});
