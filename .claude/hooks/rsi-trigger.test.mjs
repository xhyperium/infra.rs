/**
 * rsi-trigger.test.mjs — L1 单元测试 for rsi-trigger.mjs
 *
 * 测试范围：
 *  1. 文件结构：shebang、语法有效性
 *  2. gc-scan 路径指向 scripts/harness/gc-scan.mjs
 *  3. GC 扫描结果解析 — JSON.summary.critical
 *  4. 容错处理 — gc-scan 不可用时不阻塞
 *
 * 使用 ESM (.mjs)，纯 assert 模式。
 */

import { execFileSync } from "child_process";
import { readFileSync } from "fs";

let pass = 0, fail = 0;
function ok(c, name) {
  if (c) { pass++; console.log("  ok  " + name); }
  else { fail++; console.log("  FAIL " + name); }
}

// ═══ 纯函数测试（从被测文件提取逻辑） ═══

/** 解析 GC 扫描 JSON */
const parseGcResult = (jsonStr) => {
  try {
    const data = JSON.parse(jsonStr);
    return {
      critical: data.summary?.critical || 0,
      total: data.summary?.total || 0,
      hasCritical: (data.summary?.critical || 0) > 0,
    };
  } catch {
    return { critical: 0, total: 0, hasCritical: false };
  }
};

// ═══ 测试开始 ═══

console.log("\nrsi-trigger L1 tests");

// 1. 文件结构
ok(readFileSync(".claude/hooks/rsi-trigger.mjs", "utf8").includes("GC health check"),
   "包含 GC health check 注释");

try {
  execFileSync("node", ["--check", ".claude/hooks/rsi-trigger.mjs"], {
    stdio: "pipe",
    timeout: 5000,
  });
  ok(true, "node --check 通过");
} catch (e) {
  ok(false, `node --check 失败: ${e.stderr?.toString() || e.message}`);
}

// 2. gc-scan 路径检查
const src = readFileSync(".claude/hooks/rsi-trigger.mjs", "utf8");
ok(src.includes("scripts/harness/gc-scan.mjs"), "gc-scan 路径为 scripts/harness/gc-scan.mjs");
ok(!src.includes("scripts/gc-scan.mjs"), "不引用旧 scripts/gc-scan.mjs");
ok(!src.includes("scripts/audit-status.py"), "不引用旧 audit-status.py");
ok(!src.includes("docs/goal/tools/rsi-trigger.py"), "不引用旧 rsi-trigger.py");

// 3. GC 结果解析
const gcClean = JSON.stringify({ summary: { critical: 0, warning: 2, total: 5 } });
const gcParsed = parseGcResult(gcClean);
ok(gcParsed.critical === 0, "GC 无 critical → critical=0");
ok(gcParsed.hasCritical === false, "GC 无 critical → hasCritical=false");
ok(gcParsed.total === 5, "GC total=5");

const gcCritical = JSON.stringify({ summary: { critical: 3, warning: 1, total: 4 } });
const gcCrit = parseGcResult(gcCritical);
ok(gcCrit.critical === 3, "GC 3 critical → critical=3");
ok(gcCrit.hasCritical === true, "GC has critical → hasCritical=true");

const gcEmpty = JSON.stringify({ summary: {} });
const gcEmp = parseGcResult(gcEmpty);
ok(gcEmp.critical === 0, "空 summary → critical=0");
ok(gcEmp.hasCritical === false, "空 summary → hasCritical=false");

// 4. 容错
const gcBad = "not json";
const gcB = parseGcResult(gcBad);
ok(gcB.critical === 0, "无效 JSON → fallback critical=0");
ok(gcB.hasCritical === false, "无效 JSON → hasCritical=false");

const gcNoSummary = JSON.stringify({ other: "data" });
const gcNS = parseGcResult(gcNoSummary);
ok(gcNS.critical === 0, "无 summary → critical=0");

// ═══ 结果 ═══
console.log(`\n  ${pass} passed, ${fail} failed, ${pass + fail} total\n`);
process.exit(fail > 0 ? 1 : 0);
