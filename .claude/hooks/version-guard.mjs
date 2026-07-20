#!/usr/bin/env node
// Version Guard — Stop Hook
//
// 检查本次会话是否修改了追踪文件但未递增版本号。
// 输出警告到 stderr，不阻塞。
//
// 追踪文件（修改了这些文件就必须 bump 版本）：
//   - STATUS.md, README.md, ARCHITECTURE.md, module/README.md
//   - module/*/SPEC.md
//   - .repo-contract.yaml
//   - .foundationx/repo-contract.json
//   - .foundationx/status/index.json
//   - .foundationx/blockers.json
//   - foundation-bom.yaml
//
// 版本目标：
//   - release/manifest/latest.json version — 文档发布版本
//   - .repo-contract.yaml trust_hardening.ruleset — 信任规则版本
//
// 规则来源：CLAUDE.md §版本号自动递增
//  "每次更新迭代，版本号都要+1"

import { execSync } from "child_process";
import { readFileSync, existsSync } from "fs";

// ── 追踪文件 ──────────────────────────────────────────────────────
const TRACKED_FILES = [
  "STATUS.md", "README.md", "ARCHITECTURE.md", "module/README.md",
  "CLAUDE.md", "AGENTS.md", "CONSTITUTION.md",
  ".repo-contract.yaml", ".foundationx/repo-contract.json",
  ".foundationx/status/index.json", ".foundationx/blockers.json",
  "foundation-bom.yaml",
];

const SPEC_GLOB = "module/*/SPEC.md";

const MANIFEST_FILE = "release/manifest/latest.json";

function getChangedFiles() {
  try {
    const output = execSync("git diff --name-only HEAD", {
      encoding: "utf8",
      cwd: process.cwd(),
    }).trim();
    return output.split("\n").filter(Boolean);
  } catch {
    return [];
  }
}

function getCurrentVersion() {
  if (!existsSync(MANIFEST_FILE)) {
    return null;
  }
  try {
    const data = JSON.parse(readFileSync(MANIFEST_FILE, "utf8"));
    return data.version || null;
  } catch {
    return null;
  }
}

function main() {
  const changedFiles = getChangedFiles();

  if (changedFiles.length === 0) {
    return;
  }

  const trackedChanged = changedFiles.filter((f) =>
    TRACKED_FILES.includes(f) || f.match(/^module\/[^/]+\/SPEC\.md$/)
  );

  if (trackedChanged.length === 0) {
    return;
  }

  const currentVersion = getCurrentVersion();

  let lastBumpInfo = "";
  try {
    const log = execSync(
      `git log --oneline -10 -- ${MANIFEST_FILE}`,
      { encoding: "utf8" }
    ).trim();
    if (log) {
      lastBumpInfo = log.split("\n")[0];
    }
  } catch {
    // ignore
  }

  const manifestChanged = changedFiles.includes(MANIFEST_FILE);

  const lines = [];
  lines.push("");
  lines.push("══════════════════════════════════════════════════════");
  lines.push("[VersionGuard] 📋 本次会话修改了追踪文件，检查版本号...");
  lines.push("");
  lines.push(`  当前版本: ${currentVersion || "(未找到)"}`);
  lines.push(`  上次 bump: ${lastBumpInfo || "(无记录)"}`);
  lines.push(`  manifest 已变更: ${manifestChanged ? "是 ✅" : "否 ⚠️"}`);
  lines.push("");
  lines.push("  修改的追踪文件:");
  for (const f of trackedChanged) {
    lines.push(`    - ${f}`);
  }
  lines.push("");

  if (!manifestChanged) {
    lines.push("  ⚠️  警告: 追踪文件已修改但 release/manifest/latest.json 版本号未变！");
    lines.push("");
    lines.push("  CLAUDE.md 规则: 每次更新迭代，版本号都要 +1");
    lines.push("");
    lines.push("  自动 bump:");
    lines.push("    $ ./scripts/version-bump.sh              # patch bump");
    lines.push("    $ ./scripts/version-bump.sh --level minor # minor bump");
    lines.push("    $ ./scripts/version-bump.sh --dry-run      # 预览");
    lines.push("");
    lines.push("  Bump 级别选择:");
    lines.push("    PATCH: 错字修复、链接更新、说明澄清");
    lines.push("    MINOR: 新增模块/章节、架构描述变更");
    lines.push("    MAJOR: 治理体系重构、顶层架构重写");
  } else {
    lines.push("  ✅ manifest 版本号已变更，版本递增规则已满足。");
  }

  lines.push("══════════════════════════════════════════════════════");

  console.error(lines.join("\n"));
}

main();
