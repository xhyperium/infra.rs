import { execFileSync } from "child_process";
import { readFileSync, existsSync } from "fs";
import { join, dirname, resolve } from "path";
import { fileURLToPath } from "url";
import { WORKTREE_PATH_RULE, canonicalWorktreePath } from "../../scripts/worktree-policy.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, "../..");

// 读取 Harness 状态
let harnessState = { phase: "build", mode: "full" };
const statePath = join(projectRoot, ".claude/.harness-state");
if (existsSync(statePath)) {
  try {
    harnessState = { ...harnessState, ...JSON.parse(readFileSync(statePath, "utf-8")) };
  } catch {}
}
const isTweak = harnessState.mode === "tweak";
const isDesign = harnessState.phase === "design";

const input = readFileSync(0, "utf-8").trim();
if (!input) process.exit(0);

let call;
try {
  call = JSON.parse(input);
} catch {
  process.exit(0);
}

const tool = call.tool || "";
const args = call.input || {};
const filePath = args.file_path || args.path || "";

const tokenizeShellCommand = (command) => {
  const tokens = [];
  let current = "";
  let quote = null;
  let escaped = false;

  for (const ch of command) {
    if (escaped) {
      current += ch;
      escaped = false;
      continue;
    }
    if (quote === "'") {
      if (ch === "'") quote = null;
      else current += ch;
      continue;
    }
    if (ch === "\\") {
      escaped = true;
      continue;
    }
    if (ch === '"' || ch === "'") {
      quote = ch;
      continue;
    }
    if (/\s/.test(ch)) {
      if (current) {
        tokens.push(current);
        current = "";
      }
      continue;
    }
    current += ch;
  }

  if (escaped) current += "\\";
  if (current) tokens.push(current);
  return tokens;
};

const isBranchLikeRef = (ref) => {
  if (!ref) return false;
  try {
    execFileSync("git", ["check-ref-format", "--branch", ref], { stdio: "ignore", timeout: 3000 });
    return true;
  } catch {
    return false;
  }
};

const parseWorktreeAdd = (command) => {
  const tokens = tokenizeShellCommand(command);
  const gitIndex = tokens.indexOf("git");
  if (gitIndex < 0 || tokens[gitIndex + 1] !== "worktree" || tokens[gitIndex + 2] !== "add") {
    return null;
  }

  const args = tokens.slice(gitIndex + 3);
  const positional = [];
  let branch = null;
  let passthrough = false;

  for (let i = 0; i < args.length; i += 1) {
    const token = args[i];
    if (!passthrough && token === "--") {
      passthrough = true;
      continue;
    }
    if (!passthrough && (token === "-b" || token === "-B" || token === "--branch")) {
      branch = args[i + 1] || "";
      i += 1;
      continue;
    }
    if (!passthrough && token.startsWith("--branch=")) {
      branch = token.slice("--branch=".length);
      continue;
    }
    if (!passthrough && /^-[bB].+/.test(token)) {
      branch = token.slice(2);
      continue;
    }
    if (!passthrough && token.startsWith("-")) continue;
    positional.push(token);
  }

  return {
    path: positional[0] || "",
    commitish: positional[1] || "",
    branch,
  };
};

// 硬拦截：禁止 AI 直接修改 .env 文件（所有模式均生效）
const PROTECTED_FILES = [/(^|\/|\\)\.env$/, /(^|\/|\\)\.env\.local$/];

if (tool === "Write" || tool === "Edit") {
  const fullPath = resolve(projectRoot, filePath || "");
  const isProtected = PROTECTED_FILES.some((p) => p.test(fullPath));

  if (isProtected) {
    const result = {
      block: true,
      reason: `🔒 安全拦截：禁止直接修改 ${filePath}。请手动编辑此文件。`,
    };
    process.stdout.write(JSON.stringify(result));
    process.exit(0);
  }
}

// ISC-5: 分支命名 lint（不受 tweak/design 模式豁免，放在危险命令拦截块之外）
// CONSTITUTION §0.2.2 要求 {type}/{module}-{描述}；违规 block:true 并给改名建议
if (tool === "Bash" || tool === "PowerShell") {
  const cmd = args.command || "";
  let branchName = null;
  const c1 = cmd.match(/\bgit\s+(?:checkout\s+-b|switch\s+-c)\s+(\S+)/);
  if (c1) branchName = c1[1];
  if (!branchName) {
    const worktreeAdd = parseWorktreeAdd(cmd);
    if (worktreeAdd && worktreeAdd.branch) branchName = worktreeAdd.branch;
  }
  if (branchName) {
    const ALLOWED_BRANCH = /^(docs|feat|feature|fix|test|refactor|chore|governance|benchmark)\//;
    if (!ALLOWED_BRANCH.test(branchName)) {
      process.stdout.write(JSON.stringify({
        block: true,
        reason: `🏷️ 分支命名违规：\`${branchName}\` 缺少 type/ 前缀（CONSTITUTION §0.2.2 要求 {type}/{module}-{描述}）。\n   → 建议改名：docs/${branchName}-<描述>\n   → 合法前缀：docs/feat/feature/fix/test/refactor/chore/governance/benchmark\n   → 示例：git checkout -b docs/${branchName}-<描述>`,
      }));
      process.exit(0);
    }
  }

  const worktreeAdd = parseWorktreeAdd(cmd);
  const attachedBranch = worktreeAdd && (worktreeAdd.branch || (isBranchLikeRef(worktreeAdd.commitish) ? worktreeAdd.commitish : null));
  if (worktreeAdd && attachedBranch && worktreeAdd.path) {
    const actualPath = resolve(projectRoot, worktreeAdd.path);
    const expectedPath = canonicalWorktreePath(projectRoot, attachedBranch);
    if (actualPath !== expectedPath) {
      process.stdout.write(JSON.stringify({
        block: true,
        reason: `🧱 worktree 路径违规：\`git worktree add\` 创建分支附着工作区时，路径必须遵守 ${WORKTREE_PATH_RULE}。\n   分支: ${attachedBranch}\n   实际: ${actualPath}\n   期望: ${expectedPath}\n   → 请改为：git worktree add ${expectedPath} ${worktreeAdd.branch ? `-b ${attachedBranch}` : attachedBranch}`,
      }));
      process.exit(0);
    }
  }

  // === stash pop/apply 跨基线告警（#5）===
  // 不阻塞（信息护栏，与 branch-protect.mjs 同策略）：检测 git stash pop|apply，
  // 若当前分支不在 stash 来源分支的祖先链上（基线偏离），stderr 告警可能冲突。
  const stashMatch = cmd.match(/\bgit\s+stash\s+(pop|apply)(?:\s+stash@\{(\d+)\})?/);
  if (stashMatch) {
    const stashIdx = stashMatch[2] || "0";
    try {
      const stashLine = execFileSync("git", ["stash", "list"], { encoding: "utf-8", timeout: 3000 });
      const target = stashLine.split("\n").find((l) => l.startsWith(`stash@{${stashIdx}}:`)) || "";
      const onMatch = target.match(/(?:On|WIP on) ([^:]+):/);
      const srcBranch = onMatch ? onMatch[1].trim() : "";
      const curBranch = execFileSync("git", ["symbolic-ref", "--short", "HEAD"], { encoding: "utf-8", timeout: 3000 }).trim();
      if (srcBranch && curBranch && srcBranch !== curBranch) {
        // 判断当前分支是否在来源分支祖先链上（即来源分支包含当前分支基线）
        let baseDiverged = true;
        try {
          // merge-base --is-ancestor <cur> <src>：cur 是 src 祖先 → 基线一致，不告警
          execFileSync("git", ["merge-base", "--is-ancestor", curBranch, srcBranch], { stdio: "ignore", timeout: 3000 });
          baseDiverged = false;
        } catch {}
        if (baseDiverged) {
          console.error(`\n══════════════════════════════════════════════════════\n[StashGuard] ⚠️ stash 跨基线 pop 可能冲突\n\n  stash 来源分支: ${srcBranch}\n  当前分支: ${curBranch}\n  基线偏离：当前分支不在 ${srcBranch} 的祖先链上，pop 可能产生冲突。\n\n  ✅ 建议：\n    - 先切到 ${srcBranch} 再 pop，或\n    - 用 \`git stash branch stash@{${stashIdx}}\` 从 stash 创建新分支\n══════════════════════════════════════════════════════\n`);
        }
      }
    } catch {}
  }
}

// 危险命令拦截（tweak/design 模式下放行，含 .worktree/ 安全路径例外）
if (!isTweak && !isDesign) {
  const DANGEROUS_COMMANDS = [
    { pattern: /rm -rf/, label: "rm -rf", alt: "使用 trash <file> 或 git rm <file>" },
    { pattern: /git push --force/, label: "git push --force", alt: "使用 git push --force-with-lease" },
  ];
  if (tool === "Bash" || tool === "PowerShell") {
    const matched = DANGEROUS_COMMANDS.find((d) => d.pattern.test(args.command || ""));
    if (matched) {
      // 安全路径例外：允许清理临时工作区和部署目录
      const cmd = args.command || "";
      const isSafeRm = matched.label === "rm -rf" && (
        /\.worktree\/deploy\b/.test(cmd) ||
        /\.worktree\/workspaces\b/.test(cmd) ||
        /\.worktree\/omx-team\b/.test(cmd) ||
        /\/tmp\//.test(cmd)
      );
      if (!isSafeRm) {
        process.stdout.write(JSON.stringify({
          block: true,
          reason: `⚠️ 安全拦截：${matched.label} 被禁用\n   → 替代方案：${matched.alt}\n   → 如需强制执行，请在终端手动输入命令\n   → 当前模式=${harnessState.mode}，切换为 tweak 模式可放行\n   → 安全路径例外：.worktree/deploy/ .worktree/workspaces/ /tmp/`,
        }));
        process.exit(0);
      }
    }
  }
}

process.exit(0);
