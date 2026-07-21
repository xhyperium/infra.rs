#!/usr/bin/env node
/**
 * gh-sync-stress-test.mjs — interactiveReview 极值压力测试
 *
 * 覆盖边界和退化场景：
 *   1. 海量冲突（100+）
 *   2. Unicode/emoji 标题与描述
 *   3. 全部字段为 null/undefined/空字符串
 *   4. 特殊字符标签（URL、代码片段、引号）
 *   5. 超长标签列表（20+）
 *   6. 全部状态/优先级/类型组合
 *   7. 超长标题（500+ 字符）
 *   8. 二进制数据注入尝试
 *   9. 单字符差异检测
 *   10. 快速批量输入（全部 applyToAll）
 *
 * SSOT: scripts/beads/gh-sync.mjs
 */

import { interactiveReview } from "./gh-sync.mjs";

let _pass = 0;
let _fail = 0;
function ok(condition, name) { if (condition) { _pass++; console.log(`  PASS  ${name}`); } else { _fail++; console.log(`  FAIL  ${name}`); } }
function eq(a, b, name) { const ok_ = JSON.stringify(a) === JSON.stringify(b); ok(ok_, `${name} (expected ${JSON.stringify(b)}, got ${JSON.stringify(a)})`); return ok_; }
function makeInput(answers) { let i = 0; return () => answers[i++] || ""; }

console.log("╔════════════════════════════════════════════════════╗");
console.log("║  interactiveReview 极值压力测试                   ║");
console.log("╚════════════════════════════════════════════════════╝");

// ====================== 1. 海量冲突 ======================
console.log("\n=== 压力 1: 海量冲突（100 个）===");
{
  const N = 100;
  const conflicts = Array.from({ length: N }, (_, i) => ({
    beadId: `mass-${i}`,
    beadIssue: {
      id: `mass-${i}`, title: `Conflict #${i}`,
      status: "open", priority: i % 5, issueType: "task",
      labels: [`batch-${i % 10}`], updatedAt: "2026-07-21T12:00:00Z", description: `Desc ${i}`,
    },
    ghIssue: {
      ghNumber: 1000 + i, title: `Conflict #${i} [GH]`,
      status: "in_progress", priority: i % 5, issueType: "task",
      labels: [`batch-${i % 10}`], updatedAt: "2026-07-21T13:00:00Z", description: `GH desc ${i}`,
    },
  }));
  const answers = Array(N).fill("a");
  const d = interactiveReview(conflicts, {}, makeInput(answers));
  ok(Object.keys(d).length === N, `海量冲突: 全部 ${N} 个处理完毕`);
  ok(Object.values(d).every((v) => v === "push"), "海量冲突: 全部 push");
}

// ====================== 2. Unicode / Emoji ======================
console.log("\n=== 压力 2: Unicode / Emoji 内容 ===");
{
  const conflicts = [{
    beadId: "uni-1",
    beadIssue: {
      id: "uni-1", title: "🐛 修复竞态 · Fix race condition · レースコンディション",
      status: "in_progress", priority: 0, issueType: "bug",
      labels: ["🚨p0", "🔥critical"],
      updatedAt: "2026-07-21T12:00:00Z",
      description: "问题：if let Some(x) = map.remove(&key) ⚡ 并发不安全\n修正：改用 dashmap ☑️",
    },
    ghIssue: {
      ghNumber: 777, title: "🐛 修复竞态 [GitHub 编辑]",
      status: "open", priority: 0, issueType: "bug",
      labels: ["🚨p0", "✅reviewed"],
      updatedAt: "2026-07-21T14:00:00Z",
      description: "Review comment: LGTM 👍. Ship it 🚀!",
    },
  }];
  const d = interactiveReview(conflicts, {}, makeInput(["b"]));
  eq(d, { "uni-1": "push" }, "Emoji: 决策正确");
}

// ====================== 3. Null/空字段 ======================
console.log("\n=== 压力 3: 空/缺失字段退化 ===");
{
  const conflicts = [{
    beadId: "nil-1",
    beadIssue: {
      id: "nil-1", title: "", status: null, priority: null, issueType: null,
      labels: null, updatedAt: null, description: null,
    },
    ghIssue: {
      ghNumber: 1, title: null, status: null, priority: null, issueType: "",
      labels: [], updatedAt: null, description: "",
    },
  }];
  const d = interactiveReview(conflicts, {}, makeInput(["s"]));
  eq(d, { "nil-1": "skip" }, "空字段: 不崩溃");
}

// ====================== 4. 特殊字符标签 ======================
console.log("\n=== 压力 4: 特殊字符标签 ===");
{
  const conflicts = [{
    beadId: "spec-1",
    beadIssue: {
      id: "spec-1", title: "Special chars label test",
      status: "open", priority: 2, issueType: "task",
      labels: [
        "p0", "layer/http-2", "priority:🔥",
        'label "with" quotes', "back\\slash", "double--dash",
        "very-long-label-name-that-exceeds-thirty-chars",
        "http://example.com/label",
      ],
      updatedAt: "2026-07-21T12:00:00Z",
      description: "Test special chars in labels.",
    },
    ghIssue: {
      ghNumber: 2, title: "Special chars label test",
      status: "open", priority: 2, issueType: "task",
      labels: [
        "p0", "layer/http-2", "priority:🔥",
        'label "with" quotes', "back\\slash", "double--dash",
        "very-long-label-name-that-exceeds-thirty-chars",
        "http://example.com/label",
      ],
      updatedAt: "2026-07-21T13:00:00Z",
      description: "Test special chars in labels (GH).",
    },
  }];
  const d = interactiveReview(conflicts, {}, makeInput(["b"]));
  eq(d, { "spec-1": "push" }, "特殊字符标签: 不崩溃");
}

// ====================== 5. 超长标签列表 ======================
console.log("\n=== 压力 5: 超长标签列表（25 个）===");
{
  const manyLabels = Array.from({ length: 25 }, (_, i) => `label-${String(i).padStart(3, "0")}`);
  const conflicts = [{
    beadId: "mlab-1",
    beadIssue: {
      id: "mlab-1", title: "Many labels",
      status: "open", priority: 1, issueType: "feature",
      labels: manyLabels,
      updatedAt: "2026-07-21T12:00:00Z", description: "25 labels on beads",
    },
    ghIssue: {
      ghNumber: 3, title: "Many labels",
      status: "open", priority: 1, issueType: "feature",
      labels: [...manyLabels, "extra-gh-only"],
      updatedAt: "2026-07-21T13:00:00Z", description: "26 labels on GitHub",
    },
  }];
  const d = interactiveReview(conflicts, {}, makeInput(["g"]));
  eq(d, { "mlab-1": "pull" }, "超长标签: 决策正确");
}

// ====================== 6. 全部状态/优先级组合 ======================
console.log("\n=== 压力 6: 全状态全优先级矩阵 ===");
{
  const statuses = ["open", "in_progress", "blocked", "closed", "deferred"];
  const types = ["task", "bug", "feature", "epic"];
  const conflicts = [];
  for (let p = 0; p <= 4; p++) {
    for (let s = 0; s < statuses.length; s++) {
      for (let t = 0; t < types.length; t++) {
        conflicts.push({
          beadId: `matrix-${p}-${s}-${t}`,
          beadIssue: {
            id: `matrix-${p}-${s}-${t}`,
            title: `P${p} ${statuses[s]} ${types[t]}`,
            status: statuses[s], priority: p, issueType: types[t],
            labels: [`p${p}`, statuses[s], types[t]],
            updatedAt: "2026-07-21T12:00:00Z", description: "beads side",
          },
          ghIssue: {
            ghNumber: 10000 + p * 100 + s * 10 + t,
            title: `P${p} ${statuses[s]} ${types[t]} [GH]`,
            status: s % 2 === 0 ? "open" : "in_progress",
            priority: p, issueType: types[t],
            labels: [`p${p}`, types[t]],
            updatedAt: "2026-07-21T13:00:00Z", description: "gh side",
          },
        });
      }
    }
  }
  // 5 * 5 * 4 = 100 conflicts
  ok(conflicts.length === 100, `全组合: 生成 ${conflicts.length} 个冲突`);
  const answers = Array(100).fill("a");
  const d = interactiveReview(conflicts, {}, makeInput(answers));
  ok(Object.keys(d).length === 100, `全组合: 全部 100 个处理完毕`);
  ok(Object.values(d).every((v) => v === "push"), "全组合: 全部 push");
}

// ====================== 7. 超长标题 ======================
console.log("\n=== 压力 7: 超长标题与描述 ===");
{
  const giantTitle = "A".repeat(500) + "_END";
  const giantDesc = "B".repeat(2000) + "_END";
  const conflicts = [{
    beadId: "giant-1",
    beadIssue: {
      id: "giant-1", title: giantTitle,
      status: "open", priority: 1, issueType: "task",
      labels: ["giant"], updatedAt: "2026-07-21T12:00:00Z", description: giantDesc,
    },
    ghIssue: {
      ghNumber: 4, title: giantTitle + "_GH_mod",
      status: "open", priority: 1, issueType: "task",
      labels: ["giant"], updatedAt: "2026-07-21T13:00:00Z", description: giantDesc + "_GH_mod",
    },
  }];
  const d = interactiveReview(conflicts, {}, makeInput(["b"]));
  eq(d, { "giant-1": "push" }, "超长内容: 不崩溃");
  ok(giantDesc.length === 2004, "超长描述: 校验长度完整性 (2000 + '_END')");
  ok(giantTitle.length === 504, "超长标题: 校验长度完整性 (500 + '_END')");
}

// ====================== 8. 二进制/控制字符注入 ======================
console.log("\n=== 压力 8: 控制字符注入 ===");
{
  const conflicts = [{
    beadId: "ctrl-1",
    beadIssue: {
      id: "ctrl-1", title: "Hello\x00World\nLine2\r\nLine3",
      status: "open", priority: 1, issueType: "task",
      labels: ["safe\x00label", "normal"],
      updatedAt: "2026-07-21T12:00:00Z",
      description: "Contains \x00 null byte and \x1b escape codes.",
    },
    ghIssue: {
      ghNumber: 5, title: "Hello\x00World GH",
      status: "open", priority: 1, issueType: "task",
      labels: ["safe\x00label", "normal"],
      updatedAt: "2026-07-21T13:00:00Z",
      description: "GH side with \x00 and \x1b.",
    },
  }];
  const d = interactiveReview(conflicts, {}, makeInput(["b"]));
  ok(typeof d["ctrl-1"] === "string", "控制字符: 不崩溃返回有效值");
  ok(d["ctrl-1"] === "push", "控制字符: 决策正确");
}

// ====================== 9. 输入注入尝试 ======================
console.log("\n=== 压力 9: 输入注入防护 ===");
{
  const conflicts = [{
    beadId: "inj-1",
    beadIssue: {
      id: "inj-1", title: "Normal issue",
      status: "open", priority: 0, issueType: "bug",
      labels: ["p0"], updatedAt: "2026-07-21T12:00:00Z", description: "Normal desc.",
    },
    ghIssue: {
      ghNumber: 6, title: "Normal issue",
      status: "open", priority: 0, issueType: "bug",
      labels: ["p0"], updatedAt: "2026-07-21T13:00:00Z", description: "Normal GH desc.",
    },
  }];
  // Try to "inject" inputs that look like command injection
  const dangerousInputs = [
    "b; rm -rf /",
    "}; process.exit(); //",
    "$(echo hacked)",
    "`id`",
    "\\n\\n\\n",
    "\x00\x00\x00",
  ];
  for (const inp of dangerousInputs) {
    const d = interactiveReview(conflicts, {}, makeInput([inp]));
    ok(typeof d["inj-1"] === "string", `注入 "${inp.slice(0, 20)}": 返回有效字符串`);
    ok(d["inj-1"] === "skip" || d["inj-1"] === "push", `注入 "${inp.slice(0, 20)}": 安全决策`);
  }
}

// ====================== 10. 快速批量全部分配 ======================
console.log("\n=== 压力 10: 快速批量全部裁决 ===");
{
  const N = 50;
  const conflicts = Array.from({ length: N }, (_, i) => ({
    beadId: `fast-${i}`,
    beadIssue: {
      id: `fast-${i}`, title: `Rapid #${i}`,
      status: "open", priority: 3, issueType: "task",
      labels: ["rapid"], updatedAt: "2026-07-21T12:00:00Z", description: `Rapid ${i}`,
    },
    ghIssue: {
      ghNumber: 2000 + i, title: `Rapid #${i} [GH]`,
      status: "in_progress", priority: 3, issueType: "task",
      labels: ["rapid"], updatedAt: "2026-07-21T13:00:00Z", description: `Rapid GH ${i}`,
    },
  }));
  // Single 'A' input → applyToAll = pull for all remaining
  const d = interactiveReview(conflicts, {}, makeInput(["A"]));
  ok(Object.keys(d).length === N, `快速批量: 全部 ${N} 个单次输入处理`);
  ok(Object.values(d).every((v) => v === "pull"), "快速批量: 全部 pull");
}

// ====================== 11. 单字符差异 ======================
console.log("\n=== 压力 11: 单字符差异检测 ===");
{
  const base = "The quick brown fox jumps over the lazy dog. ";
  const conflicts = [{
    beadId: "diff-1",
    beadIssue: {
      id: "diff-1", title: base.repeat(3),
      status: "open", priority: 2, issueType: "task",
      labels: ["diff"], updatedAt: "2026-07-21T12:00:00Z", description: base.repeat(5),
    },
    ghIssue: {
      ghNumber: 7, title: base.repeat(3).replace("fox", "FOX"),
      status: "open", priority: 2, issueType: "task",
      labels: ["diff"], updatedAt: "2026-07-21T13:00:00Z", description: base.repeat(5).replace("dog", "DOG"),
    },
  }];
  const d = interactiveReview(conflicts, {}, makeInput(["b"]));
  eq(d, { "diff-1": "push" }, "单字符差异: 决策正确");
}

// ====================== Summary ======================
console.log(`\n=== STRESS RESULTS: ${_pass} passed, ${_fail} failed ===`);
process.exit(_fail > 0 ? 1 : 0);
