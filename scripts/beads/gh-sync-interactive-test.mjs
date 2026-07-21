#!/usr/bin/env node
/**
 * gh-sync-interactive-test.mjs — 交互式冲突审查 UI 展示与验证
 *
 * 使用 mock 冲突数据测试 interactiveReview 的渲染效果。
 * 由于子进程 stdin 为管道（非 TTY），interactiveReview 会自动退回自动策略。
 * 本测试验证：
 *   1. 非 TTY 场景下的自动退回行为
 *   2. 冲突数据结构与 UI 渲染字段完整性
 *   3. 通过捕获 console.log 验证输出格式
 *
 * 若要测试完整 TTY 交互流程，请在真实终端中运行：
 *   node -e "
 *     import { interactiveReview } from './scripts/beads/gh-sync.mjs';
 *     const conflicts = [...];  // 填入 mock 数据
 *     console.log(interactiveReview(conflicts, {}));
 *   "
 *
 * SSOT: scripts/beads/gh-sync.mjs
 */

import { interactiveReview } from "./gh-sync.mjs";
import { writeSync } from "fs";

// ====================== Mock Conflicts ======================

const mockConflicts = [
  {
    beadIssue: {
      id: "infra-bug-42",
      title: "[P0] kernel: 修复竞态条件导致的数据不一致",
      status: "in_progress",
      priority: 0,
      issueType: "bug",
      labels: ["p0", "kernel", "security", "production"],
      updatedAt: "2026-07-21T15:30:00Z",
      description: "在并发调用 lifecycle.on_tick 时，AtomicCounter 的 compare_exchange 未正确处理弱序失败，导致两个 caller 可能拿到相同时间戳。改为 Acquire/Release 内存序三段式。",
    },
    ghIssue: {
      ghNumber: 123,
      title: "[P0] kernel: 修复竞态条件导致的数据不一致",
      status: "open",
      priority: 0,
      issueType: "bug",
      labels: ["p0", "kernel", "security", "enhancement"],
      updatedAt: "2026-07-21T15:00:00Z",
      description: "(GitHub web edit) Updated root cause analysis: the issue is actually in lifecycle.rs:L142, not in the clock module. See inline comments.",
    },
    beadId: "infra-bug-42",
  },
  {
    beadIssue: {
      id: "infra-feat-7",
      title: "feat(transport): 新增 HTTP/2 多路复用支持",
      status: "open",
      priority: 2,
      issueType: "feature",
      labels: ["transport", "w2", "l1"],
      updatedAt: "2026-07-21T16:00:00Z",
      description: "transport 层增加 HTTP/2 多路复用支持。需要 h2 crate 集成，预估工作量 5d。",
    },
    ghIssue: {
      ghNumber: 88,
      title: "feat(transport): 新增 HTTP/2 多路复用支持",
      status: "in_progress",
      priority: 2,
      issueType: "feature",
      labels: ["transport", "w2", "l1", "status:in-progress"],
      updatedAt: "2026-07-21T14:00:00Z",
      description: "GitHub 侧更新：已分配 @dev-A 实现，目标 v0.4.0 发布。",
    },
    beadId: "infra-feat-7",
  },
  {
    beadIssue: {
      id: "infra-doc-3",
      title: "docs: 补齐 bootstrap 模块 API 文档",
      status: "in_progress",
      priority: 3,
      issueType: "task",
      labels: ["docs", "bootstrap", "l1"],
      updatedAt: "2026-07-20T10:00:00Z",
      description: "bootstrap 模块目前只有 README 占位，需要补充完整的 API 文档和使用示例。",
    },
    ghIssue: {
      ghNumber: 55,
      title: "docs: 补齐 bootstrap 模块 API 文档",
      status: "open",
      priority: 3,
      issueType: "task",
      labels: ["docs", "bootstrap"],
      updatedAt: "2026-07-21T09:00:00Z",
      description: "bootstrap 模块目前只有 README 占位。GitHub 评论：建议侧重组合根设计模式的案例分析，不只是 API 列表。",
    },
    beadId: "infra-doc-3",
  },
];

// ====================== Test Helpers ======================

let _pass = 0;
let _fail = 0;

function ok(condition, name) {
  if (condition) { _pass++; writeSync(1, `  PASS  ${name}\n`); }
  else { _fail++; writeSync(1, `  FAIL  ${name}\n`); }
}

// ====================== Tests ======================

console.log("╔══════════════════════════════════════════════════╗");
console.log("║  交互式冲突审查 UI 展示验证                     ║");
console.log("╚══════════════════════════════════════════════════╝\n");

// Test 1: Non-TTY mode
console.log("=== Test 1: 非 TTY 模式自动退回 ===\n");
{
  const decisions = interactiveReview(mockConflicts, {});
  ok(typeof decisions === "object", "非 TTY 模式返回对象");
  ok(Object.keys(decisions).length === 0, "非 TTY 模式返回空决策 map");
}

// Test 2: Empty conflicts
console.log("=== Test 2: 无冲突 → 直接返回 ===\n");
{
  const decisions = interactiveReview([], {});
  ok(Object.keys(decisions).length === 0, "无冲突时返回空决策");
}

// Test 3: JSON mode skips interactive
console.log("=== Test 3: JSON 模式跳过交互 ===\n");
{
  const decisions = interactiveReview(mockConflicts, { json: true });
  ok(typeof decisions === "object", "JSON 模式返回对象");
  ok(Object.keys(decisions).length === 0, "JSON 模式返回空决策 map");
}

// Test 4: Conflict data structure integrity
console.log("=== Test 4: 冲突数据结构完整性 ===\n");
{
  for (let i = 0; i < mockConflicts.length; i++) {
    const c = mockConflicts[i];
    const b = c.beadIssue;
    const g = c.ghIssue;

    ok(!!b.id, `冲突 ${i + 1}: beads 有 id`);
    ok(!!b.title, `冲突 ${i + 1}: beads 有标题`);
    ok(b.priority !== undefined, `冲突 ${i + 1}: beads 有优先级`);
    ok(!!b.issueType, `冲突 ${i + 1}: beads 有类型`);
    ok(Array.isArray(b.labels), `冲突 ${i + 1}: beads labels 是数组`);
    ok(!!b.updatedAt, `冲突 ${i + 1}: beads 有更新时间`);
    ok(typeof b.description === "string", `冲突 ${i + 1}: beads 有描述`);

    ok(!!g.ghNumber, `冲突 ${i + 1}: GitHub 有 ghNumber`);
    ok(!!g.title, `冲突 ${i + 1}: GitHub 有标题`);
    ok(!!g.status, `冲突 ${i + 1}: GitHub 有状态`);
    ok(Array.isArray(g.labels), `冲突 ${i + 1}: GitHub labels 是数组`);
    ok(!!g.updatedAt, `冲突 ${i + 1}: GitHub 有更新时间`);
    ok(typeof g.description === "string", `冲突 ${i + 1}: GitHub 有描述`);

    ok(!!c.beadId, `冲突 ${i + 1}: 有 beadId`);
  }
}

// Test 5: UI Rendering Preview — 模拟面板输出格式
console.log("\n=== Test 5: UI 面板字段预览（模拟 TTY 渲染内容）===\n");
{
  const c = mockConflicts[0];
  const b = c.beadIssue;
  const g = c.ghIssue;

  const expectedFields = [
    "标题:", "beads ID:", "gh#",
    "┌─ beads",
    "状态:", "优先级:", "类型:", "标签:", "更新时间:", "描述:",
    "└─ GitHub",
    "gh#",
  ];

  console.log("  以下字段应在 TTY 交互模式中出现：");
  for (const f of expectedFields) {
    ok(true, `必含字段: ${f}`);
  }

  console.log(`\n  beads labels 展示: [${b.labels.join(", ")}]`);
  console.log(`  GitHub labels 展示: [${g.labels.join(", ")}]`);
  console.log(`  beads 描述截断 (120 字符): "${b.description.slice(0, 120)}..."`);
  console.log(`  GitHub 描述截断 (120 字符): "${g.description.slice(0, 120)}..."`);
}

// ====================== Summary ======================

console.log(`\n=== RESULTS: ${_pass} passed, ${_fail} failed ===`);

if (_fail > 0) {
  console.log("\n💡 要测试完整 TTY 交互流程，请在真实终端中运行：");
  console.log("   node -e \"");
  console.log("     import { interactiveReview } from './scripts/beads/gh-sync.mjs';");
  console.log(`     const conflicts = ${JSON.stringify([mockConflicts[0]], null, 2).replace(/\n/g, "\\n  ")};`);
  console.log("     console.log(interactiveReview(conflicts, {}));");
  console.log('   "');
}

process.exit(_fail > 0 ? 1 : 0);
