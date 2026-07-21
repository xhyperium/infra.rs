#!/usr/bin/env node
// check-decimal-no-panicking-ops.mjs
//
// 拒绝生产路径（adapters / bootstrap 等）使用 decimalx panicking 运算符
// `+`/`-`/`*` 于 Decimal 值，或调用 `.rescale(`。
//
// 扫描根：crates/adapters, bootstrap/src, observex/src, resiliencx/src, transport/src
// 排除：types/decimal 自身、tests 目录、*_test.rs
//
// 用法：
//   node scripts/quality-gates/check-decimal-no-panicking-ops.mjs
//   node scripts/quality-gates/check-decimal-no-panicking-ops.mjs --allow path
// 退出码：0 通过；1 发现违规。
//
// 注：完整 decimalx 硬化证据在 W1（infra-asa.2）；本脚本为静态启发式门禁，可独立合入。
import { readFileSync, readdirSync, statSync, existsSync } from "fs";
import { join, relative, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..", "..");

const SCAN_ROOTS = [
  "crates/adapters",
  "crates/bootstrap/src",
  "crates/observex/src",
  "crates/resiliencx/src",
  "crates/transport/src",
];

/** 明显的 panicking 生产调用形态（保守启发式，非完整类型分析）。 */
const PATTERNS = [
  {
    name: "Decimal.rescale(",
    re: /\.rescale\s*\(/g,
    hint: "使用 checked_rescale",
  },
  {
    // a + b 当上下文像 Decimal 很难静态判断；捕获显式 Decimal::... +
    name: "Decimal::… + / - / *",
    re: /Decimal\s*::\s*\w+[^\n]{0,80}[\+\-\*]/g,
    hint: "使用 checked_add/sub/mul",
  },
];

function walk(dir, out = []) {
  if (!existsSync(dir)) return out;
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    let st;
    try {
      st = statSync(p);
    } catch {
      continue;
    }
    if (st.isDirectory()) {
      if (name === "target" || name === "tests") continue;
      walk(p, out);
    } else if (name.endsWith(".rs") && !name.endsWith("_test.rs")) {
      out.push(p);
    }
  }
  return out;
}

function scanFile(abs) {
  const text = readFileSync(abs, "utf8");
  // 无 decimal 引用则跳过（避免误报其它类型的 +）
  if (!/\bdecimalx\b|\bDecimal\b/.test(text)) return [];
  const hits = [];
  const lines = text.split(/\r?\n/);
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line.trimStart().startsWith("//")) continue;
    for (const { name, re, hint } of PATTERNS) {
      re.lastIndex = 0;
      if (re.test(line)) {
        hits.push({ line: i + 1, name, hint, text: line.trim() });
      }
    }
  }
  return hits;
}

const allow = new Set();
const args = process.argv.slice(2);
for (let i = 0; i < args.length; i++) {
  if (args[i] === "--allow" && args[i + 1]) {
    allow.add(args[++i]);
  }
}

const files = [];
for (const rel of SCAN_ROOTS) {
  walk(join(root, rel), files);
}

let failed = 0;
const report = [];
for (const abs of files) {
  const rel = relative(root, abs).replace(/\\/g, "/");
  if ([...allow].some((a) => rel === a || rel.startsWith(a + "/"))) continue;
  const hits = scanFile(abs);
  for (const h of hits) {
    failed++;
    report.push(`${rel}:${h.line}: ${h.name} — ${h.hint}\n  ${h.text}`);
  }
}

if (failed > 0) {
  console.error(`decimal panicking-ops gate: ${failed} hit(s)\n`);
  for (const r of report) console.error(r);
  console.error(
    "\n资金路径请用 checked_*。若为误报，使用 --allow <rel-path> 并在 PR 说明。",
  );
  process.exit(1);
}

console.log(
  `decimal panicking-ops gate: OK (${files.length} files scanned, 0 hits)`,
);
process.exit(0);
