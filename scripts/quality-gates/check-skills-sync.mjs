#!/usr/bin/env node
// check-skills-sync.mjs — 验证 .claude/skills/ 与 .agents/skills/ 投影一致
//
// 规则来源：SSOT.md R2（投影同步）· .codex/AGENTS.md「禁止手工分叉」
//
// 用法：
//   node scripts/quality-gates/check-skills-sync.mjs         # 完整校验
//   node scripts/quality-gates/check-skills-sync.mjs --json   # JSON 输出

import { execSync } from "node:child_process";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..", "..");
const useJson = process.argv.includes("--json");

const SOURCE = join(ROOT, ".claude", "skills");
const PROJECTION = join(ROOT, ".agents", "skills");

function getDiff() {
  try {
    const output = execSync(
      `diff -rq "${SOURCE}/" "${PROJECTION}/" 2>&1`,
      {
        encoding: "utf8",
        cwd: ROOT,
        timeout: 10000,
        stdio: ["pipe", "pipe", "pipe"],
      }
    ).trim();
    return output ? output.split("\n").filter(Boolean) : [];
  } catch (err) {
    // diff 返回码 ≠ 0 表示有差异
    const output = (err.stdout || "").toString().trim();
    return output ? output.split("\n").filter(Boolean) : [];
  }
}

const diffs = getDiff();

if (useJson) {
  process.stdout.write(JSON.stringify({
    check: "skills-sync",
    source: ".claude/skills/",
    projection: ".agents/skills/",
    synchronized: diffs.length === 0,
    differences: diffs,
  }, null, 2) + "\n");
} else {
  if (diffs.length === 0) {
    console.log("✅ skills 投影同步：.claude/skills/ ≡ .agents/skills/");
  } else {
    console.error("❌ skills 投影不同步！");
    console.error("");
    console.error(`  源：.claude/skills/`);
    console.error(`  投影：.agents/skills/`);
    console.error("");
    console.error(`  发现 ${diffs.length} 处差异：`);
    for (const d of diffs) {
      console.error(`    ${d}`);
    }
    console.error("");
    console.error("  修复：rsync -a --delete .claude/skills/ .agents/skills/");
    console.error("");
    console.error("  规则：SSOT.md R2 — 禁止在投影目录手工分叉维护");
  }
}

process.exit(diffs.length === 0 ? 0 : 1);
