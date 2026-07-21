#!/usr/bin/env node
/**
 * gh-sync-complex-test.mjs — interactiveReview 复杂输入场景验证
 *
 * 通过 _input 回调注入模拟按键序列，验证所有决策路径：
 *   - 批量 beads 获胜 (a)
 *   - 批量 GitHub 获胜 (A)
 *   - 中途退出 (q)
 *   - 混合决策 (b/g/s)
 *   - 空/空白输入 → 自动跳过
 *
 * SSOT: scripts/beads/gh-sync.mjs
 */

import { interactiveReview } from "./gh-sync.mjs";

// ====================== Helpers ======================

let _pass = 0;
let _fail = 0;

function ok(condition, name) {
  if (condition) { _pass++; console.log(`  PASS  ${name}`); }
  else { _fail++; console.log(`  FAIL  ${name}`); }
}

function eq(a, b, name) {
  const result = JSON.stringify(a) === JSON.stringify(b);
  ok(result, `${name} (expected ${JSON.stringify(b)}, got ${JSON.stringify(a)})`);
  return result;
}

function mkConflict(id, idx, opts = {}) {
  return {
    beadId: id,
    beadIssue: {
      id,
      title: opts.bTitle || `[P${idx % 4}] mock-${idx}: ${id}`,
      status: idx % 3 === 0 ? "in_progress" : idx % 3 === 1 ? "blocked" : "open",
      priority: idx % 5,
      issueType: ["bug", "feature", "task", "epic", "bug"][idx % 5],
      labels: opts.bLabels || [`p${idx % 5}`, `layer-${["kernel","transport","bootstrap","configx","contracts"][idx % 5]}`],
      updatedAt: `2026-07-21T${String(14 + idx % 4).padStart(2, "0")}:00:00Z`,
      description: opts.bDesc || `Mock beads ${id}. ` + "x".repeat(30),
    },
    ghIssue: {
      ghNumber: 100 + idx,
      title: opts.gTitle || `[P${idx % 4}] mock-${idx}: ${id}`,
      status: idx % 2 === 0 ? "open" : "in_progress",
      priority: idx % 5,
      issueType: ["bug", "feature", "task", "epic", "bug"][idx % 5],
      labels: opts.gLabels || [`p${idx % 5}`, `layer-${["kernel","transport","bootstrap","configx","contracts"][idx % 5]}`],
      updatedAt: `2026-07-21T${String(13 + idx % 4).padStart(2, "0")}:00:00Z`,
      description: opts.gDesc || `Mock GitHub ${id}. ` + "y".repeat(30),
    },
  };
}

function makeInput(answers) {
  let i = 0;
  return () => {
    const a = answers[i++] || "";
    console.log(`     [input: "${a}"]`);
    return a;
  };
}

// ====================== Test Scenarios ======================

console.log("╔══════════════════════════════════════════════════╗");
console.log("║  交互式审查复杂输入场景验证                     ║");
console.log("╚══════════════════════════════════════════════════╝");

// Scenario 1: Batch apply-all-beads (a)
console.log("\n=== 场景 1: 批量 beads 获胜 (a) ===");
{
  const conflicts = [
    mkConflict("test-a-1", 0),
    mkConflict("test-a-2", 1),
    mkConflict("test-a-3", 2),
    mkConflict("test-a-4", 3),
  ];
  const input = makeInput(["b", "a"]);  // b on 1st, a on 2nd → applies to all
  const d = interactiveReview(conflicts, {}, input);
  const expected = { "test-a-1": "push", "test-a-2": "push", "test-a-3": "push", "test-a-4": "push" };
  eq(d, expected, "批量 beads: 全部 push");
}

// Scenario 2: Batch apply-all-github (A)
console.log("\n=== 场景 2: 批量 GitHub 获胜 (A) ===");
{
  const conflicts = [
    mkConflict("test-A-1", 0),
    mkConflict("test-A-2", 1),
    mkConflict("test-A-3", 2),
  ];
  const input = makeInput(["A"]);  // Apply all GitHub on first conflict
  const d = interactiveReview(conflicts, {}, input);
  const expected = { "test-A-1": "pull", "test-A-2": "pull", "test-A-3": "pull" };
  eq(d, expected, "批量 GitHub: 全部 pull");
}

// Scenario 3: Quit mid-review (q)
console.log("\n=== 场景 3: 中途退出 (q) ===");
{
  const conflicts = [
    mkConflict("test-q-1", 0),
    mkConflict("test-q-2", 1),
    mkConflict("test-q-3", 2),
  ];
  const input = makeInput(["b", "q"]);  // b on 1st, then quit
  const d = interactiveReview(conflicts, {}, input);
  eq(d, { "test-q-1": "push" }, "中途退出: 仅处理第 1 个冲突");
  ok(!d["test-q-2"], "中途退出: 第 2 个冲突未处理");
  ok(!d["test-q-3"], "中途退出: 第 3 个冲突未处理");
}

// Scenario 4: Mixed decisions (b, g, s)
console.log("\n=== 场景 4: 混合决策 (b/g/s) ===");
{
  const conflicts = [
    mkConflict("test-mix-1", 0, {
      bLabels: ["p0", "kernel", "security"],
      gLabels: ["p0", "kernel", "enhancement"],
    }),
    mkConflict("test-mix-2", 1, {
      bLabels: ["p1", "transport"],
      gLabels: ["p1", "transport", "status:in-progress"],
    }),
    mkConflict("test-mix-3", 2, {
      bLabels: ["p2", "bootstrap", "docs"],
      gLabels: ["docs", "bootstrap"],
    }),
  ];
  const input = makeInput(["b", "g", "s"]);
  const d = interactiveReview(conflicts, {}, input);
  const expected = { "test-mix-1": "push", "test-mix-2": "pull", "test-mix-3": "skip" };
  eq(d, expected, "混合决策: b→push, g→pull, s→skip");
}

// Scenario 5: Apply-all-beads on first conflict via "a"
console.log("\n=== 场景 5: 首个冲突直接批量 (a) ===");
{
  const conflicts = [
    mkConflict("test-a1-1", 0),
    mkConflict("test-a1-2", 1),
  ];
  const input = makeInput(["a"]);  // 'a' on first → applies to all
  const d = interactiveReview(conflicts, {}, input);
  const expected = { "test-a1-1": "push", "test-a1-2": "push" };
  eq(d, expected, "首项直接 a: 全部 push");
}

// Scenario 6: Empty input → skip
console.log("\n=== 场景 6: 无效输入 → 跳过 ===");
{
  const conflicts = [
    mkConflict("test-bad-1", 0),
    mkConflict("test-bad-2", 1),
  ];
  const input = makeInput(["", "xyz"]);  // empty, then garbage
  const d = interactiveReview(conflicts, {}, input);
  eq(d, { "test-bad-1": "skip", "test-bad-2": "skip" }, "无效输入: 全部 skip");
}

// Scenario 7: White-labeled input normalizes correctly
console.log("\n=== 场景 7: 输入规范化（大写/空格）===");
{
  const conflicts = [mkConflict("test-norm-1", 0)];
  const input = makeInput(["  B  "]);  // uppercase with whitespace
  const d = interactiveReview(conflicts, {}, input);
  eq(d, { "test-norm-1": "push" }, "输入规范化: B → b → push");
}

// Scenario 8: Long description truncation display check
console.log("\n=== 场景 8: 长描述截断展示 ===");
{
  const longText = "A".repeat(200) + "_END_MARKER";
  const conflicts = [mkConflict("test-long-1", 0, { bDesc: longText, gDesc: longText })];
  const input = makeInput(["b"]);
  const d = interactiveReview(conflicts, {}, input);
  ok(d["test-long-1"] === "push", "长描述: 决策正确");
  // Description display is validated by visual inspection in console output above
}

// Scenario 9: Multiple different labels comparison
console.log("\n=== 场景 9: 标签差异渲染 ===");
{
  const conflicts = [
    mkConflict("test-labels-1", 0, {
      bLabels: ["p0", "kernel", "security", "production", "urgent"],
      gLabels: ["p0", "kernel", "enhancement", "draft"],
    }),
  ];
  const input = makeInput(["b"]);
  const d = interactiveReview(conflicts, {}, input);
  ok(d["test-labels-1"] === "push", "标签差异: 决策正确");
  // Visual: beads has 5 labels, GitHub has 4 — difference visible in output
}

// ====================== Summary ======================

console.log(`\n=== RESULTS: ${_pass} passed, ${_fail} failed ===`);
process.exit(_fail > 0 ? 1 : 0);
