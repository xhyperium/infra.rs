#!/usr/bin/env node
/**
 * check-linguist-generated.mjs — 校验 .gitattributes 中所有生成文件均已标记 linguist-generated
 *
 * 设计原则：
 *   - 已知生成文件列表由本脚本集中维护（SSOT）
 *   - 检查每个已知模式是否在 .gitattributes 中有对应的 linguist-generated=true 行
 *   - 同时检查 .gitattributes 中标注了 linguist-generated 的文件是否实际存在
 *
 * 用法: node scripts/quality-gates/check-linguist-generated.mjs
 * exit 0 = 全部通过, exit 1 = 存在违规
 */

import { readFileSync, existsSync, readdirSync, statSync } from "fs";
import { resolve, dirname, basename } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..", "..");
const GITATTR_PATH = resolve(ROOT, ".gitattributes");

// ═══════════════════════════════════════
// 已知生成文件清单（本仓 SSOT）
// ═══════════════════════════════════════
const REQUIRED_GENERATED = [
  "Cargo.lock",
  "docs/status/CI_WORKFLOW_MATRIX.generated.md",
  "docs/api-baselines/*.txt",
];

let pass = 0, fail = 0;

function ok(cond, msg) {
  if (cond) { pass++; console.log(`  ok  ${msg}`); }
  else      { fail++; console.log(`  FAIL ${msg}`); }
}

/**
 * 简单的 glob 匹配：仅支持单个 `*`（匹配不含 / 的任意字符串）
 * 不依赖任何第三方包。
 */
function simpleGlob(pattern) {
  const cwd = ROOT;
  const parts = pattern.split("/");
  const results = [];

  function walk(dir, depth) {
    if (depth >= parts.length) {
      results.push(dir);
      return;
    }
    const segment = parts[depth];

    if (segment === "*" || segment === "?" || segment.includes("*") || segment.includes("?")) {
      // 构建正则：* → .*  ? → .
      const reStr = "^" + segment.replace(/[.+^${}()|[\]\\]/g, "\\$&")
                                  .replace(/\*/g, "[^/]*")
                                  .replace(/\?/g, "[^/]") + "$";
      const re = new RegExp(reStr);
      try {
        const entries = readdirSync(dir);
        for (const entry of entries) {
          if (re.test(entry)) {
            walk(resolve(dir, entry), depth + 1);
          }
        }
      } catch (_) { /* skip missing dirs */ }
    } else {
      const next = resolve(dir, segment);
      if (depth === parts.length - 1) {
        if (existsSync(next)) results.push(next);
      } else {
        try {
          if (statSync(next).isDirectory()) {
            walk(next, depth + 1);
          }
        } catch (_) { /* skip */ }
      }
    }
  }

  walk(cwd, 0);
  return results.map((r) => r.replace(cwd, "").replace(/^\//, ""));
}

console.log("\ncheck-linguist-generated\n");
console.log(`  已知生成文件: ${REQUIRED_GENERATED.length} 个模式\n`);

// ── §1 文件存在 ──────────────────────────
ok(existsSync(GITATTR_PATH), ".gitattributes 存在");

// ── §2 读取并解析 .gitattributes ────────
let attrs;
try {
  const raw = readFileSync(GITATTR_PATH, "utf8");
  attrs = raw
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l !== "" && !l.startsWith("#"));
  ok(true, `.gitattributes 有 ${attrs.length} 条规则`);
} catch (e) {
  ok(false, `.gitattributes 读取失败: ${e.message}`);
  process.exit(1);
}

// ── §3 检查每个已知生成文件模式是否已在 .gitattributes 中标记 ──
const declaredPatterns = attrs.map((line) => {
  // gitattributes 格式: pattern attr1=val1 attr2=val2 ...
  const m = line.match(/^(\S+)\s+(.+)$/);
  if (!m) return null;
  return { pattern: m[1], raw: line, attrs: m[2] };
}).filter(Boolean);

for (const req of REQUIRED_GENERATED) {
  const found = declaredPatterns.find((d) => d.pattern === req);

  if (!found) {
    ok(false, `${req} — 未在 .gitattributes 中找到`);
    continue;
  }

  const hasLinguist = /linguist-generated\s*=\s*true/.test(found.attrs);
  ok(hasLinguist, `${req} → linguist-generated=true`);
}

// ── §4 检查标记了 linguist-generated 的文件是否存在 ──
for (const d of declaredPatterns) {
  if (!/linguist-generated\s*=\s*true/.test(d.attrs)) continue;

  const matches = simpleGlob(d.pattern);
  ok(matches.length > 0, `${d.pattern} → 匹配 ${matches.length} 个文件`);
}

// ── §5 汇总 ─────────────────────────────
console.log(`\n${pass} passed, ${fail} failed, ${pass + fail} total\n`);

if (fail > 0) {
  console.log("请在 .gitattributes 中为每个已知生成文件添加 linguist-generated=true");
  process.exit(1);
}
console.log("All linguist-generated attributes pass √\n");
process.exit(0);
