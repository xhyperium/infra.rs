#!/usr/bin/env node
/**
 * gh-sync.test.mjs — gh-sync.mjs 单元测试（无外部 CLI 调用）
 *
 * 用法:
 *   node scripts/beads/gh-sync.test.mjs
 *
 * SSOT: scripts/beads/gh-sync.mjs
 */

import {
  hashFields, extractFooter, buildFooter, removeFooter,
  normalizeBeadsIssue, normalizeGhIssue,
  mapGhStateToBeads, mapBeadsStatusToGh,
  buildGhLabels, ghLabelsToBeadsLabels,
  extractPriorityFromLabels, extractTypeFromLabels,
  deterministicColor, shEscape,
  resolveConflict, loadState, saveState, updateMapping,
} from "./gh-sync.mjs";

import { existsSync, unlinkSync, readFileSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { createHash } from "crypto";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..", "..");
const pass = 0;
const fail = 0;
let _pass = 0;
let _fail = 0;

function ok(condition, name) {
  if (condition) {
    _pass++;
    console.log(`  PASS  ${name}`);
  } else {
    _fail++;
    console.log(`  FAIL  ${name}`);
  }
}

function eq(a, b, name) {
  const result = JSON.stringify(a) === JSON.stringify(b);
  ok(result, `${name} (expected ${JSON.stringify(b)}, got ${JSON.stringify(a)})`);
  return result;
}

function section(title) {
  console.log(`\n=== ${title} ===`);
}

// ====================== hashFields ======================
section("hashFields");
{
  const a = hashFields({ title: "test", description: "desc", status: "open", labels: ["a", "b"], priority: 0, issueType: "task" });
  const b = hashFields({ title: "test", description: "desc", status: "open", labels: ["a", "b"], priority: 0, issueType: "task" });
  const c = hashFields({ title: "test2", description: "desc", status: "open", labels: ["a", "b"], priority: 0, issueType: "task" });
  ok(a === b, "hashFields: same data → same hash");
  ok(a !== c, "hashFields: different title → different hash");
  ok(typeof a === "string" && a.length === 12, "hashFields: returns 12-char hex string");
}

// ====================== extractFooter ======================
section("extractFooter / buildFooter / removeFooter");
{
  const footer = buildFooter("infra-test.1");
  ok(footer.includes("infra-test.1"), "buildFooter: includes bead ID");
  ok(footer.includes("<!-- beads-sync:"), "buildFooter: includes footer comment");

  const body = "Issue body\n\n" + footer;
  const parsed = extractFooter(body);
  ok(parsed && parsed.beadId === "infra-test.1", "extractFooter: parses bead ID from body");

  const noFooter = extractFooter("Just text");
  ok(noFooter === null, "extractFooter: no footer → null");

  const cleaned = removeFooter(body);
  ok(cleaned === "Issue body", "removeFooter: strips footer correctly");
  ok(!extractFooter(cleaned), "removeFooter: cleaned body has no footer");
}

// ====================== normalizeBeadsIssue ======================
section("normalizeBeadsIssue");
{
  const raw = {
    id: "infra-s9t.1",
    title: "Test issue",
    description: "Desc",
    status: "in_progress",
    priority: 2,
    issue_type: "bug",
    labels: ["p1", "security"],
    owner: "user@example.com",
    created_at: "2026-07-21T12:00:00Z",
    updated_at: "2026-07-21T13:00:00Z",
  };
  const n = normalizeBeadsIssue(raw);
  ok(n.source === "beads", "normalizeBeadsIssue: sets source=beads");
  ok(n.priority === 2, "normalizeBeadsIssue: priority preserved");
  ok(n.issueType === "bug", "normalizeBeadsIssue: issueType from issue_type");
  ok(n.labels.length === 2, "normalizeBeadsIssue: labels array");
  ok(n.assignee === "user@example.com", "normalizeBeadsIssue: assignee from owner");
}

// ====================== normalizeGhIssue ======================
section("normalizeGhIssue");
{
  const footer = buildFooter("infra-test.2");
  const raw = {
    number: 42,
    title: "GH issue",
    body: "GH body\n\n" + footer,
    labels: [{ name: "beads" }, { name: "status:in-progress" }, { name: "p0" }],
    state: "OPEN",
    stateReason: null,
    updatedAt: "2026-07-21T14:00:00Z",
    createdAt: "2026-07-21T12:00:00Z",
    assignees: [{ login: "ghuser" }],
    url: "https://github.com/xhyperium/infra.rs/issues/42",
  };
  const n = normalizeGhIssue(raw);
  ok(n.source === "github", "normalizeGhIssue: sets source=github");
  ok(n.ghNumber === 42, "normalizeGhIssue: ghNumber preserved");
  ok(n.status === "in_progress", "normalizeGhIssue: state mapped to in_progress");
  ok(n.description === "GH body", "normalizeGhIssue: footer removed from description");
  ok(n.labels.includes("p0"), "normalizeGhIssue: p0 label preserved");
  ok(!n.labels.includes("beads"), "normalizeGhIssue: beads label stripped");
  ok(n.assignee === "ghuser", "normalizeGhIssue: assignee from assignees");
}

// ====================== Status Mapping ======================
section("Status Mapping");
{
  eq(mapBeadsStatusToGh("open"), { state: "open", labels: [] }, "beads open → gh open");
  eq(mapBeadsStatusToGh("in_progress"), { state: "open", labels: ["status:in-progress"] }, "beads in_progress → gh open + label");
  eq(mapBeadsStatusToGh("blocked"), { state: "open", labels: ["status:blocked"] }, "beads blocked → gh open + label");
  eq(mapBeadsStatusToGh("closed"), { state: "closed", labels: ["status:closed"] }, "beads closed → gh closed + label");

  eq(mapGhStateToBeads("OPEN", null, []), "open", "gh OPEN → beads open");
  eq(mapGhStateToBeads("OPEN", null, ["status:in-progress"]), "in_progress", "gh OPEN + in-progress → beads in_progress");
  eq(mapGhStateToBeads("OPEN", null, ["status:blocked"]), "blocked", "gh OPEN + blocked → beads blocked");
  eq(mapGhStateToBeads("CLOSED", "COMPLETED", []), "closed", "gh CLOSED → beads closed");
}

// ====================== Priority Mapping ======================
section("Priority Mapping");
{
  eq(extractPriorityFromLabels(["priority:p0", "bug"]), 0, "extract p0");
  eq(extractPriorityFromLabels(["priority:p4"]), 4, "extract p4");
  eq(extractPriorityFromLabels(["bug"]), null, "no priority label → null");
}

// ====================== Type Extraction ======================
section("Type Extraction");
{
  eq(extractTypeFromLabels(["type:bug", "priority:p0"]), "bug", "extract type:bug");
  eq(extractTypeFromLabels(["type:feature"]), "feature", "extract type:feature");
  eq(extractTypeFromLabels(["bug"]), null, "no type label → null");
}

// ====================== Label Building ======================
section("Label Building");
{
  const beadIssue = { priority: 1, issueType: "task", status: "in_progress", labels: ["kernel", "p1"] };
  const labels = buildGhLabels(beadIssue);
  ok(labels.includes("beads"), "buildGhLabels: includes beads label");
  ok(labels.includes("priority:p1"), "buildGhLabels: includes priority label");
  ok(labels.includes("type:task"), "buildGhLabels: includes type label");
  ok(labels.includes("status:in-progress"), "buildGhLabels: includes status label");
  ok(labels.includes("kernel"), "buildGhLabels: includes custom label");

  const beadsLabels = ghLabelsToBeadsLabels(["beads", "priority:p0", "status:in-progress", "type:bug", "p0", "kernel"]);
  ok(beadsLabels.length === 2 && beadsLabels.includes("p0") && beadsLabels.includes("kernel"), "ghLabelsToBeadsLabels: strips synthetic labels");
}

// ====================== deterministicColor ======================
section("deterministicColor");
{
  const c1 = deterministicColor("beads");
  const c2 = deterministicColor("beads");
  const c3 = deterministicColor("bug");
  ok(/^[0-9a-f]{6}$/.test(c1), "deterministicColor: valid hex");
  ok(c1 === c2, "deterministicColor: deterministic for same input");
  ok(c1 !== c3, "deterministicColor: different for different input");
}

// ====================== Target resolution ======================
section("shEscape");
{
  ok(shEscape('hello"world') === 'hello\\"world', "shEscape: escapes double quotes");
  ok(shEscape("test$PATH") === "test\\$PATH", "shEscape: escapes dollar sign");
  ok(shEscape("simple") === "simple", "shEscape: leaves simple string untouched");
}

// ====================== State Management ======================
section("State Management");
{
  const testPath = join(ROOT, ".beads", "_test-gh-sync.json");
  const s = loadState();
  ok(s.version === 1, "loadState: returns default when no file");
  ok(s.mappings && typeof s.mappings === "object", "loadState: mappings object exists");

  updateMapping(s, "test-id", 42, { title: "T", updatedAt: "2026-01-01" }, { title: "T", updatedAt: "2026-01-02" });
  const m = s.mappings["test-id"];
  ok(m.ghNumber === 42, "updateMapping: ghNumber stored");
  ok(m.beadsHash, "updateMapping: beadsHash computed");
  ok(m.lastBeadsUpdated === "2026-01-01", "updateMapping: lastBeadsUpdated stored");
  ok(m.lastGhUpdated === "2026-01-02", "updateMapping: lastGhUpdated stored");

  saveState(s, { dryRun: false, _testPath: testPath });
  const loaded = loadState();
  // Cleanup test file if it was created
  if (existsSync(testPath)) unlinkSync(testPath);
}

// ====================== Conflict Resolution ======================
section("Conflict Resolution");
{
  const beadNew = { updatedAt: "2026-07-21T15:00:00Z" };
  const ghNew = { updatedAt: "2026-07-21T14:00:00Z" };
  const beadOld = { updatedAt: "2026-07-21T13:00:00Z" };

  eq(resolveConflict(beadNew, ghNew, {}), "push", "beads newer → push");
  eq(resolveConflict(beadOld, ghNew, {}), "pull", "github newer → pull");
  eq(resolveConflict(beadNew, beadNew, {}), "push", "tie → push (local wins)");
  eq(resolveConflict(beadOld, ghNew, { preferLocal: true }), "push", "prefer-local → push");
  eq(resolveConflict(beadNew, ghNew, { preferGithub: true }), "pull", "prefer-github → pull");
}

// ====================== Summary ======================
console.log(`\n=== RESULTS: ${_pass} passed, ${_fail} failed ===`);
process.exit(_fail > 0 ? 1 : 0);
