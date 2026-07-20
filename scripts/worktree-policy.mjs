/**
 * Worktree 路径策略 v2
 *
 * 规范路径：`.worktrees/<branch-name>`
 *   - 分支 `/` 保留为目录分隔符
 *   - 与 `scripts/worktree.sh` / `worktree-activate.sh` 约定一致
 *
 * 旧路径（已废弃, 审计报告）：
 *   - `workspaces/` 模式：`.worktrees/workspaces/<branch>`
 *   - 全局模式：`~/.worktrees/<project>/`
 *
 * 供 `.claude/hooks/pre-tool-check.mjs` 与 `session-context.mjs` 共用。
 */

import { resolve, basename } from "path";
import { existsSync } from "fs";
import { homedir } from "os";

// ── 常量 ────────────────────────────────────

/** 人读规则说明 */
export const WORKTREE_PATH_RULE = ".worktrees/<branch-name>";

/** 旧路径模式关键词（审计时匹配） */
const LEGACY_WORKSPACES_SEGMENT = "workspaces";

// ── 路径工具 ────────────────────────────────

/**
 * 剥离 refs/heads/ 前缀，保留原始分支名含 `/`。
 */
export function bareBranch(branchName) {
  return String(branchName || "")
    .replace(/^refs\/heads\//, "")
    .trim();
}

/**
 * 返回规范 worktree 绝对路径。
 */
export function canonicalWorktreePath(projectRoot, branchName) {
  return resolve(projectRoot, ".worktrees", bareBranch(branchName));
}

/**
 * 从 CWD 推断当前所在的 worktree 名称（含 `/`），
 * 若不在任何 worktree 下则返回 null。
 */
export function detectWorktreeFromCwd({ projectRoot, cwd }) {
  const wtBase = resolve(projectRoot, ".worktrees");
  const resolvedCwd = resolve(cwd ?? process.cwd());
  if (!resolvedCwd.startsWith(wtBase + "/")) return null;
  return resolvedCwd.slice(wtBase.length + 1); // e.g. "feat/login"
}

// ── 合规判定 ────────────────────────────────

/**
 * 判断某分支检出路径是否符合规范。
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

// ── porcelain 解析 ──────────────────────────

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
      continue;
    }
    if (line.startsWith("branch ")) {
      currentBranch = line.slice("branch ".length).trim();
      continue;
    }
    if (line === "detached") { isDetached = true; continue; }
    if (line === "locked" || line.startsWith("locked ")) { isLocked = true; continue; }
    if (line === "") flush();
  }
  flush();

  return { registered, branchToPath, pathToBranch, detachedPaths, lockedPaths };
}

// ── 审计 ────────────────────────────────────

/**
 * 全表审计已登记 worktree，找出路径偏离规范 v2 的项，
 * 并检测旧约定残留（`workspaces/` 子目录模式及全局 `~/.worktrees/`）。
 */
export function auditWorktreePaths({ root, worktreeState, homeDir }) {
  const nonCompliant = [];
  const legacyPaths = [];

  // 1. 分支路径不符合新规范 .worktrees/<branch>
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

  // 2. workspaces/ 子目录旧模式残留
  const wtRoot = resolve(root, ".worktrees");
  const wsSubdir = resolve(wtRoot, LEGACY_WORKSPACES_SEGMENT);
  if (existsSync(wsSubdir)) {
    legacyPaths.push({
      path: wsSubdir,
      reason: "workspaces 子目录约定已废弃",
      migrate: `  mv '${wsSubdir}'/* '${wtRoot}/' && rmdir '${wsSubdir}'`,
    });
  }

  // 3. 全局旧路径 ~/.worktrees/<project>/
  const home = homeDir ?? homedir();
  if (home) {
    const projectName = basename(resolve(root));
    const globalLegacy = resolve(home, ".worktrees", projectName);
    if (existsSync(globalLegacy)) {
      legacyPaths.push({
        path: globalLegacy,
        reason: "全局 ~/.worktrees/ 约定已废弃",
        migrate: `  git worktree move "${globalLegacy}"-* 到 ${wtRoot}/<branch>`,
      });
    }
  }

  return { nonCompliant, legacyPaths };
}

// ── 格式化输出 ──────────────────────────────

/**
 * 审计结果 → 人读警告行。
 */
export function formatAuditWarning(audit) {
  const lines = [];

  for (const { branch, path, expectedPath } of audit.nonCompliant || []) {
    lines.push(`   • 分支 ${branch} 工作区路径偏离规范 v2：`);
    lines.push(`      当前: ${path}`);
    lines.push(`      规范: ${expectedPath}`);
    lines.push(`      迁移: git worktree move '${path}' '${expectedPath}'`);
  }

  for (const { path, reason, migrate } of audit.legacyPaths || []) {
    lines.push(`   • 旧规范残留: ${path}`);
    lines.push(`     ${reason}`);
    lines.push(`     ${migrate}`);
  }

  return lines;
}
