/**
 * Worktree 路径策略（CLAUDE.md / CONTRIBUTING）
 *
 * 规范路径：`.worktrees/workspaces/<branch-name>`
 * 分支名中的 `/` 映射为目录名中的 `-`（与现有 workspaces 约定一致）。
 *
 * 供 `.claude/hooks/pre-tool-check.mjs` 与 `session-context.mjs` 共用。
 */

import { resolve, basename } from "path";
import { existsSync } from "fs";
import { homedir } from "os";

/** 人读规则说明（错误信息 / SessionStart 护栏） */
export const WORKTREE_PATH_RULE = ".worktrees/workspaces/<branch-name>";

/**
 * 将 git 分支名规范为 worktree 目录段。
 * @param {string} branchName
 * @returns {string}
 */
export function branchToWorktreeDirName(branchName) {
  const bare = String(branchName || "")
    .replace(/^refs\/heads\//, "")
    .trim();
  if (!bare) return "";
  return bare.replace(/\//g, "-");
}

/**
 * 给定仓库根与分支名，返回规范 worktree 绝对路径。
 * @param {string} projectRoot
 * @param {string} branchName
 * @returns {string}
 */
export function canonicalWorktreePath(projectRoot, branchName) {
  const dir = branchToWorktreeDirName(branchName);
  return resolve(projectRoot, ".worktrees", "workspaces", dir);
}

/**
 * 判断某分支当前检出路径是否符合规范。
 * @param {{ root: string, branchName: string, actualPath?: string }} opts
 * @returns {{ expectedPath: string, isRootCheckout: boolean, compliant: boolean }}
 */
export function describeBranchWorktreePath({ root, branchName, actualPath }) {
  const expectedPath = canonicalWorktreePath(root, branchName);
  const resolvedRoot = resolve(root);
  const resolvedActual = actualPath ? resolve(actualPath) : "";
  const isRootCheckout = Boolean(resolvedActual) && resolvedActual === resolvedRoot;
  const compliant =
    Boolean(resolvedActual) && resolve(resolvedActual) === resolve(expectedPath);
  return { expectedPath, isRootCheckout, compliant };
}

/**
 * 解析 `git worktree list --porcelain` 输出。
 * @param {string} text
 * @returns {{
 *   registered: Set<string>,
 *   branchToPath: Map<string, string>,
 *   pathToBranch: Map<string, string>,
 *   detachedPaths: string[],
 *   lockedPaths: Set<string>,
 * }}
 */
export function parseWorktreePorcelain(text) {
  const registered = new Set();
  const branchToPath = new Map();
  const pathToBranch = new Map();
  const detachedPaths = [];
  const lockedPaths = new Set();

  let currentPath = null;
  let currentBranch = null;
  let isDetached = false;
  let isLocked = false;

  const flush = () => {
    if (!currentPath) return;
    registered.add(currentPath);
    if (isDetached) {
      detachedPaths.push(currentPath);
    } else if (currentBranch) {
      const br = currentBranch.replace(/^refs\/heads\//, "");
      branchToPath.set(br, currentPath);
      pathToBranch.set(currentPath, br);
    }
    if (isLocked) lockedPaths.add(currentPath);
    currentPath = null;
    currentBranch = null;
    isDetached = false;
    isLocked = false;
  };

  for (const raw of String(text || "").split("\n")) {
    const line = raw.trimEnd();
    if (line.startsWith("worktree ")) {
      flush();
      currentPath = line.slice("worktree ".length).trim();
      currentBranch = null;
      isDetached = false;
      isLocked = false;
      continue;
    }
    if (line.startsWith("branch ")) {
      currentBranch = line.slice("branch ".length).trim();
      isDetached = false;
      continue;
    }
    if (line === "detached") {
      isDetached = true;
      currentBranch = null;
      continue;
    }
    if (line === "locked" || line.startsWith("locked ")) {
      isLocked = true;
      continue;
    }
    if (line === "") {
      flush();
    }
  }
  flush();

  return { registered, branchToPath, pathToBranch, detachedPaths, lockedPaths };
}

/**
 * 全表审计：扫描所有已登记 worktree，找出路径偏离规范的项，
 * 并检测旧规范残留 `~/.worktrees/<项目目录名>/` 是否存在。
 *
 * 复用 describeBranchWorktreePath 判定单分支合规性；
 * 主仓 root checkout 视为合规（不算偏离）。
 *
 * @param {{
 *   root: string,
 *   worktreeState: ReturnType<parseWorktreePorcelain>,
 *   homeDir?: string,
 * }} opts
 * @returns {{
 *   nonCompliant: Array<{ branch: string, path: string, expectedPath: string }>,
 *   legacyGlobalPaths: Array<{ path: string }>,
 * }}
 */
export function auditWorktreePaths({ root, worktreeState, homeDir }) {
  const nonCompliant = [];

  // 全表审计：遍历 branchToPath，判定每个 worktree 是否在规范路径下。
  // 主仓（branch === basename-less root checkout）由 describeBranchWorktreePath
  // 的 isRootCheckout 标记，不计入偏离。
  for (const [branch, path] of worktreeState.branchToPath || []) {
    const { expectedPath, isRootCheckout, compliant } = describeBranchWorktreePath({
      root,
      branchName: branch,
      actualPath: path,
    });
    if (!isRootCheckout && !compliant) {
      nonCompliant.push({ branch, path, expectedPath });
    }
  }

  // 旧路径残留检测：~/.worktrees/<项目目录名>/ 是否存在。
  const legacyGlobalPaths = [];
  const home = homeDir ?? homedir();
  if (home) {
    const projectName = basename(resolve(root));
    const legacyDir = resolve(home, ".worktrees", projectName);
    if (existsSync(legacyDir)) {
      legacyGlobalPaths.push({ path: legacyDir });
    }
  }

  return { nonCompliant, legacyGlobalPaths };
}

/**
 * 把审计结果格式化为人读警告行（供 SessionStart hook 直接 push 进输出）。
 * 每行已含前导空格，便于在 "---" 分隔的 SessionStart 输出中对齐。
 *
 * @param {{
 *   nonCompliant: Array<{ branch: string, path: string, expectedPath: string }>,
 *   legacyGlobalPaths: Array<{ path: string }>,
 * }} audit
 * @returns {string[]}
 */
export function formatAuditWarning(audit) {
  const lines = [];
  for (const { branch, path, expectedPath } of audit.nonCompliant || []) {
    lines.push(`   • 分支 ${branch} 工作区偏离规范：`);
    lines.push(`     当前: ${path}`);
    lines.push(`     期望: ${expectedPath}`);
    lines.push(`     迁移: git worktree move "${path}" "${expectedPath}"`);
  }
  for (const { path } of audit.legacyGlobalPaths || []) {
    lines.push(`   • 旧规范残留目录存在: ${path}`);
    lines.push(`     → 该目录下的工作区应迁至 .worktrees/workspaces/<branch>`);
  }
  return lines;
}
