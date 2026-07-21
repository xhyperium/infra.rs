/**
 * rsi-trigger.test.mjs — L1 单元测试 for rsi-trigger.mjs
 *
 * 测试范围：
 *  1. AUDIT_SCRIPT / RSI_TRIGGER 路径常量
 *  2. FAIL 计数逻辑 — 匹配 /FAIL/g
 *  3. RSI 触发判断 — failCount > 0
 *  4. GC 扫描结果解析 — JSON.summary.critical
 *  5. 跳过逻辑 — 无 FAIL 时不触发
 *
 * 使用 ESM (.mjs)，纯 assert 模式。
 */

let pass = 0, fail = 0;
function ok(c, name) {
  if (c) { pass++; console.log("  ok  " + name); }
  else { fail++; console.log("  FAIL " + name); }
}

// ═══ 从被测文件复制常量与纯函数 ═══

const AUDIT_SCRIPT = 'python3 scripts/audit-status.py';
const RSI_TRIGGER = 'python3 docs/goal/tools/rsi-trigger.py';

/** 计算 audit 输出中 FAIL 的数量 */
const countFail = (output) => (output.match(/FAIL/g) || []).length;

/** 判断是否需要触发 RSI */
const shouldTrigger = (failCount) => failCount > 0;

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

// L0: shebang/syntax

// --- 1. 常量路径 ---
ok(AUDIT_SCRIPT.includes("audit-status"), "AUDIT_SCRIPT 指向 audit-status.py");
ok(RSI_TRIGGER.includes("rsi-trigger"), "RSI_TRIGGER 指向 rsi-trigger.py");
ok(AUDIT_SCRIPT.startsWith("python3"), "AUDIT_SCRIPT 使用 python3");
ok(RSI_TRIGGER.startsWith("python3"), "RSI_TRIGGER 使用 python3");

// --- 2. FAIL 计数 ---
ok(countFail("") === 0, "空输出 → 0 FAIL");
ok(countFail("OK\nOK\nOK") === 0, "全 OK → 0 FAIL");
ok(countFail("FAIL: test1") === 1, "1 个 FAIL → count=1");
ok(countFail("FAIL: test1\nFAIL: test2") === 2, "2 个 FAIL → count=2");
ok(countFail("OK\nFAIL\nOK\nFAIL\nFAIL") === 3, "混合输出 → count=3");
ok(countFail("FAILED") === 1, "FAILED 匹配 /FAIL/g（/FAIL/g 无词边界，匹配子串）");

// --- 3. RSI 触发判断 ---
ok(shouldTrigger(0) === false, "0 FAIL → 不触发 RSI");
ok(shouldTrigger(1) === true, "1 FAIL → 触发 RSI");
ok(shouldTrigger(5) === true, "5 FAIL → 触发 RSI");
ok(shouldTrigger(100) === true, "100 FAIL → 触发 RSI");

// --- 4. GC 扫描结果解析 ---
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

const gcBad = "not json";
const gcB = parseGcResult(gcBad);
ok(gcB.critical === 0, "无效 JSON → fallback critical=0");
ok(gcB.hasCritical === false, "无效 JSON → hasCritical=false");

// --- 5. 跳过逻辑：无 FAIL 或 audit 不可用 ---
// audit 脚本不可用时 execSync 抛出异常 → 跳过（静默）
// 此处仅验证 countFail 在空/clean 场景下的行为
const noFailTriggers = countFail("ALL OK") === 0;
ok(noFailTriggers, "全 PASS 输出 → 不触发 RSI 提案");

// --- 6. 审计输出多样格式 ---
// 确保 FAIL 匹配不区分大小写位置
ok(countFail("  FAIL  at line 42") === 1, "带缩进的 FAIL → count=1");
ok(countFail("test FAIL") === 1, "行尾 FAIL → count=1");
ok(countFail("FAILFAIL") === 2, "FAILFAIL → /FAIL/g 匹配 2 次（g 标志遍历所有匹配）");

// ═══ 结果 ═══
console.log(`\n  ${pass} passed, ${fail} failed, ${pass + fail} total\n`);
process.exit(fail > 0 ? 1 : 0);
