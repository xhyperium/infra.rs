#!/usr/bin/env node
/**
 * check-governance-drift.mjs — 治理执行层漂移检测
 *
 * 检查 hooks 与脚本中的路径/版本模型是否已对齐当前项目结构。
 * 检测项覆盖 infra-2ui 修复的回归风险点。
 *
 * 用法:
 *   node scripts/quality-gates/check-governance-drift.mjs
 *   node scripts/quality-gates/check-governance-drift.mjs --json
 */

import { readFileSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..", "..");
const jsonMode = process.argv.includes("--json");

/** @typedef {{ file: string, check: string, status: "PASS"|"FAIL", detail: string }} Finding */
/** @type {Finding[]} */
const findings = [];

/**
 * @param {string} file - relative file path
 * @param {string} check - check description
 * @param {boolean} pass - pass or fail
 * @param {string} detail - extra context
 */
function record(file, check, pass, detail = "") {
  findings.push({ file, check, status: pass ? "PASS" : "FAIL", detail });
}

// ── 检查项定义 ──────────────────────────────────────────────────

const checks = [];

// 检查 1: version-guard 不引用旧 release/manifest/latest.json
checks.push(() => {
  const f = ".claude/hooks/version-guard.mjs";
  if (!existsSync(join(root, f))) {
    record(f, "文件存在", false, "文件不存在");
    return;
  }
  const src = readFileSync(join(root, f), "utf8");
  const lines = src.split("\n");
  const functionalLines = lines.filter(
    (l) => !l.trim().startsWith("//") && l.includes("release/manifest/latest.json"),
  );
  record(f, "version-guard 不将 release/manifest/latest.json 用作功能路径",
    functionalLines.length === 0,
    functionalLines.length > 0
      ? `仍有 ${functionalLines.length} 处功能性引用`
      : "");
});

// 检查 2: version-guard 不引用旧 scripts/version-bump.sh
checks.push(() => {
  const f = ".claude/hooks/version-guard.mjs";
  if (!existsSync(join(root, f))) return;
  const src = readFileSync(join(root, f), "utf8");
  record(f, "version-guard 不引用 scripts/version-bump.sh",
    !src.includes("version-bump.sh"),
    src.includes("version-bump.sh") ? "仍有旧引用" : "");
});

// 检查 3: version-guard 引用 scripts/version/crate-bump.mjs
checks.push(() => {
  const f = ".claude/hooks/version-guard.mjs";
  if (!existsSync(join(root, f))) return;
  const src = readFileSync(join(root, f), "utf8");
  record(f, "version-guard 引用 crate-bump.mjs",
    src.includes("crate-bump.mjs"),
    !src.includes("crate-bump.mjs") ? "缺少新 bump 工具引用" : "");
});

// 检查 4: version-guard 不引用 module/*/SPEC.md 旧路径
checks.push(() => {
  const f = ".claude/hooks/version-guard.mjs";
  if (!existsSync(join(root, f))) return;
  const src = readFileSync(join(root, f), "utf8");
  // 忽略注释中的引用
  const lines = src.split("\n").filter((l) => !l.trim().startsWith("//"));
  const hasOldSpecPath = lines.some((l) => l.includes("module/") && l.includes("SPEC.md"));
  record(f, "version-guard 不引用 module/*/SPEC.md 旧路径",
    !hasOldSpecPath,
    hasOldSpecPath ? "仍有旧模块 SPEC 路径" : "");
});

// 检查 5: session-review 引用 scripts/harness/gc-scan.mjs
checks.push(() => {
  const f = ".claude/hooks/session-review.mjs";
  if (!existsSync(join(root, f))) {
    record(f, "文件存在", false, "文件不存在");
    return;
  }
  const src = readFileSync(join(root, f), "utf8");
  record(f, "session-review 引用 scripts/harness/gc-scan.mjs",
    src.includes("scripts/harness/gc-scan.mjs"),
    !src.includes("scripts/harness/gc-scan.mjs") ? "未引用正确路径" : "");
  record(f, "session-review 不引用旧 scripts/gc-scan.mjs",
    !src.includes('"scripts/gc-scan.mjs"'),
    src.includes('"scripts/gc-scan.mjs"') ? "仍有旧 gc-scan 路径" : "");
});

// 检查 6: rsi-trigger 不引用旧 Python 脚本
checks.push(() => {
  const f = ".claude/hooks/rsi-trigger.mjs";
  if (!existsSync(join(root, f))) {
    record(f, "文件存在", false, "文件不存在");
    return;
  }
  const src = readFileSync(join(root, f), "utf8");
  record(f, "rsi-trigger 不引用 audit-status.py",
    !src.includes("audit-status.py"),
    src.includes("audit-status.py") ? "仍有旧引用" : "");
  record(f, "rsi-trigger 不引用 rsi-trigger.py",
    !src.includes("docs/goal/tools/rsi-trigger.py"),
    src.includes("docs/goal/tools/rsi-trigger.py") ? "仍有旧引用" : "");
});

// 检查 7: rsi-trigger 引用 scripts/harness/gc-scan.mjs
checks.push(() => {
  const f = ".claude/hooks/rsi-trigger.mjs";
  if (!existsSync(join(root, f))) return;
  const src = readFileSync(join(root, f), "utf8");
  record(f, "rsi-trigger gc-scan 路径为 scripts/harness/gc-scan.mjs",
    src.includes("scripts/harness/gc-scan.mjs"),
    !src.includes("scripts/harness/gc-scan.mjs") ? "路径未对齐" : "");
  record(f, "rsi-trigger 不引用旧 scripts/gc-scan.mjs",
    !src.includes('"scripts/gc-scan.mjs"'),
    src.includes('"scripts/gc-scan.mjs"') ? "仍有旧 gc-scan 路径" : "");
});

// 检查 8: gc-scan.mjs 存在于 scripts/harness/
checks.push(() => {
  const exists = existsSync(join(root, "scripts/harness/gc-scan.mjs"));
  record("scripts/harness/gc-scan.mjs", "gc-scan.mjs 在 scripts/harness/",
    exists,
    exists ? "" : "文件不存在");
});

// 检查 9: crate-bump.mjs 存在于 scripts/version/
checks.push(() => {
  const exists = existsSync(join(root, "scripts/version/crate-bump.mjs"));
  record("scripts/version/crate-bump.mjs", "crate-bump.mjs 在 scripts/version/",
    exists,
    exists ? "" : "文件不存在");
});

// 检查 10: 旧 release/manifest/latest.json 已移除
checks.push(() => {
  const removed = !existsSync(join(root, "release/manifest/latest.json"));
  record("release/manifest/latest.json", "旧 manifest 已移除",
    removed,
    removed ? "" : "文件仍存在");
});

// 检查 11: 旧 scripts/version-bump.sh 已移除
checks.push(() => {
  const removed = !existsSync(join(root, "scripts/version-bump.sh"));
  record("scripts/version-bump.sh", "旧 version-bump.sh 已移除",
    removed,
    removed ? "" : "文件仍存在");
});

// ── 执行检查 ──────────────────────────────────────────────────

for (const check of checks) {
  check();
}

// ── 输出 ──────────────────────────────────────────────────────

if (jsonMode) {
  const fails = findings.filter((f) => f.status === "FAIL").length;
  console.log(JSON.stringify({
    summary: { total: findings.length, pass: findings.length - fails, fail: fails },
    findings,
  }));
} else {
  let failCount = 0;
  for (const f of findings) {
    const mark = f.status === "PASS" ? "  PASS" : "  FAIL";
    console.log(`${mark}  ${f.file}: ${f.check}${f.detail ? ` — ${f.detail}` : ""}`);
    if (f.status === "FAIL") failCount++;
  }
  console.log(`\n${findings.length - failCount}/${findings.length} 通过`);
  if (failCount > 0) {
    console.error(`${failCount} 项 FAIL — 治理漂移检测不通过`);
    process.exit(1);
  }
}
