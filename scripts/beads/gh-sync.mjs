#!/usr/bin/env node
/**
 * gh-sync.mjs — beads ↔ GitHub Issues 双向同步
 *
 * 将 beads 本地议题双向同步到 GitHub Issues，支持增量/全量、冲突策略、
 * 标签自动创建、以及 session-review hook 集成。
 *
 * 用法:
 *   node scripts/beads/gh-sync.mjs                        增量同步（默认）
 *   node scripts/beads/gh-sync.mjs --full                  全量重新扫描
 *   node scripts/beads/gh-sync.mjs --dry-run               仅预览，不做实际变更
 *   node scripts/beads/gh-sync.mjs --push-only             仅 beads → GitHub
 *   node scripts/beads/gh-sync.mjs --pull-only             仅 GitHub → beads
 *   node scripts/beads/gh-sync.mjs --prefer-local          冲突时以 beads 为准
 *   node scripts/beads/gh-sync.mjs --prefer-github         冲突时以 GitHub 为准
 *   node scripts/beads/gh-sync.mjs --include-closed        包含已关闭议题
 *   node scripts/beads/gh-sync.mjs --json                  JSON 输出（供 hook/脚本消费）
 *   node scripts/beads/gh-sync.mjs --incremental-only      仅增量（hook 安全模式，不触发全量）
 *
 * SSOT: 状态文件 .beads/gh-sync-state.json（gitignored）
 */

import { execSync } from "child_process";
import { existsSync, readFileSync, writeFileSync, mkdirSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { createHash } from "crypto";

// ====================== A. Setup / Helpers ======================

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..", "..");
const STATE_PATH = join(ROOT, ".beads", "gh-sync-state.json");
const BEADS_LABEL = "beads";
const FOOTER_PATTERN = /<!--\s*beads-sync:\s*([^\s|]+)\s*\|\s*synced:\s*([^\s]+)\s*-->/;
const DEFAULT_STATE = {
  version: 1,
  repo: "",
  createdAt: null,
  lastFullSync: null,
  lastIncrementalSync: null,
  mappings: {},
};

/** 执行 shell 命令，返回标准输出截断字符串；失败返回 "" */
function run(cmd, opts = {}) {
  try {
    return execSync(cmd, {
      cwd: ROOT,
      encoding: "utf-8",
      timeout: opts.timeout || 30_000,
      stdio: ["pipe", "pipe", "pipe"],
      ...opts,
    }).trim();
  } catch {
    if (opts.required) die(`${opts.label || "command"} failed: ${cmd}`, 2);
    return "";
  }
}

/** 执行命令并以 JSON 解析结果；失败返回 opts.default */
function jsonRun(cmd, opts = {}) {
  const raw = run(cmd, opts);
  if (!raw) return opts.default !== undefined ? opts.default : null;
  try {
    return JSON.parse(raw);
  } catch {
    warn(`JSON parse failed for: ${cmd.split(" ")[0]}`);
    return opts.default !== undefined ? opts.default : null;
  }
}

function die(msg, code = 1) {
  console.error(`❌ ${msg}`);
  process.exit(code);
}

function info(msg) {
  if (QUIET) return;
  console.log(`ℹ ${msg}`);
}

function ok(msg) {
  if (QUIET) return;
  console.log(`✅ ${msg}`);
}

function warn(msg) {
  if (QUIET) return;
  console.warn(`⚠ ${msg}`);
}

/** SHA-256 哈希关键字段用于变更检测 */
function hashFields(issue) {
  const payload = [
    issue.title || "",
    issue.description || "",
    issue.status || "",
    (issue.labels || []).sort().join(","),
    issue.priority !== undefined ? String(issue.priority) : "",
    issue.issueType || "",
  ].join("|");
  return createHash("sha256").update(payload).digest("hex").slice(0, 12);
}

/** 对 shell 字符串中的特殊字符做转义 */
function shEscape(s) {
  return String(s)
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\$/g, "\\$")
    .replace(/`/g, "\\`");
}

/** 解析命令行参数 */
function parseArgs() {
  const argv = process.argv.slice(2);
  return {
    dryRun: argv.includes("--dry-run"),
    json: argv.includes("--json"),
    pushOnly: argv.includes("--push-only"),
    pullOnly: argv.includes("--pull-only"),
    preferLocal: argv.includes("--prefer-local"),
    preferGithub: argv.includes("--prefer-github"),
    includeClosed: argv.includes("--include-closed"),
    full: argv.includes("--full"),
    incrementalOnly: argv.includes("--incremental-only"),
  };
}

let QUIET = false;

// ====================== B. State Management ======================

function loadState() {
  if (!existsSync(STATE_PATH)) return JSON.parse(JSON.stringify(DEFAULT_STATE));
  try {
    const raw = readFileSync(STATE_PATH, "utf-8");
    const state = JSON.parse(raw);
    if (state.version !== 1) {
      warn("State file version mismatch; treating as fresh sync");
      return JSON.parse(JSON.stringify(DEFAULT_STATE));
    }
    return state;
  } catch {
    warn("State file corrupted; treating as fresh sync");
    return JSON.parse(JSON.stringify(DEFAULT_STATE));
  }
}

function saveState(state, opts) {
  if (opts.dryRun) return;
  const dir = dirname(STATE_PATH);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  writeFileSync(STATE_PATH, JSON.stringify(state, null, 2), "utf-8");
}

function updateMapping(state, beadId, ghNumber, beadData, ghData) {
  const existing = state.mappings[beadId] || {};
  state.mappings[beadId] = {
    ghNumber,
    ghNodeId: ghData?.node_id || existing.ghNodeId || null,
    lastSyncedAt: new Date().toISOString(),
    lastBeadsUpdated: beadData?.updatedAt || existing.lastBeadsUpdated,
    lastGhUpdated: ghData?.updatedAt || existing.lastGhUpdated,
    beadsHash: beadData ? hashFields(beadData) : existing.beadsHash,
    ghHash: ghData ? hashFields(ghData) : existing.ghHash,
  };
}

// ====================== C. Label Management ======================

function getGhLabels() {
  const raw = jsonRun("gh label list --json name,color --limit 200", { default: [] });
  return new Set(raw.map((l) => l.name));
}

function deterministicColor(label) {
  let hash = 5381;
  for (let i = 0; i < label.length; i++) {
    hash = ((hash << 5) + hash + label.charCodeAt(i)) | 0;
  }
  const r = (128 + ((hash & 0xff) % 96)).toString(16).padStart(2, "0");
  const g = (128 + (((hash >> 8) & 0xff) % 96)).toString(16).padStart(2, "0");
  const b = (128 + (((hash >> 16) & 0xff) % 96)).toString(16).padStart(2, "0");
  return `${r}${g}${b}`;
}

function ensureGhLabels(beadsLabels, opts) {
  const existing = getGhLabels();
  const allLabels = [BEADS_LABEL, "priority:p0", "priority:p1", "priority:p2", "priority:p3", "priority:p4",
    "status:in-progress", "status:blocked", "status:closed", "status:deferred",
    "type:task", "type:bug", "type:feature", "type:epic",
    ...beadsLabels];

  let created = 0;
  for (const name of allLabels) {
    if (existing.has(name)) continue;
    const color = deterministicColor(name);
    if (opts.dryRun) {
      info(`  [dry-run] would create label: ${name} (#${color})`);
    } else {
      run(`gh label create "${shEscape(name)}" --color "${color}" --force`);
      created++;
    }
  }
  if (created) ok(`Created ${created} GitHub labels`);
}

// ====================== D. Beads Data Access ======================

function getAllBeads(includeClosed) {
  const status = includeClosed ? "all" : "open";
  const list = jsonRun(`bd list --status=${status} --json`, { required: true, label: "bd list" });
  return (list || []).map(normalizeBeadsIssue);
}

function getChangedBeads(since) {
  if (!since || since < "2020-01-01") return getAllBeads();
  const list = jsonRun(`bd query "updated>${since}" --json`, { default: [] });
  return (list || []).map(normalizeBeadsIssue);
}

function getBeadById(id) {
  const raw = jsonRun(`bd show "${id}" --json`, { default: null });
  return raw ? normalizeBeadsIssue(raw) : null;
}

function getBeadsLabels() {
  try {
    const all = getAllBeads(true);
    const labels = new Set();
    for (const b of all) {
      for (const l of b.labels || []) labels.add(l);
    }
    return [...labels];
  } catch {
    return [];
  }
}

function normalizeBeadsIssue(raw) {
  return {
    id: raw.id,
    title: raw.title || "",
    description: raw.description || "",
    status: raw.status || "open",
    priority: raw.priority,
    issueType: raw.issue_type || "task",
    labels: raw.labels || [],
    assignee: raw.owner || null,
    createdAt: raw.created_at || null,
    updatedAt: raw.updated_at || null,
    source: "beads",
  };
}

// ====================== E. GitHub Data Access ======================

function getGhIssues(state, since) {
  let cmd = "gh issue list";
  cmd += ` --state ${state || "all"}`;
  cmd += " --json number,title,body,labels,state,stateReason,updatedAt,createdAt,assignees,url,number";
  cmd += " --limit 200";
  if (since && since >= "2020-01-01") {
    cmd += ` --search "updated:>=${since.slice(0, 10)}"`;
  }
  const list = jsonRun(cmd, { default: [] });
  return (list || []).map((raw) => normalizeGhIssue(raw));
}

function getGhIssueByNumber(number) {
  const raw = jsonRun(`gh issue view ${number} --json number,title,body,labels,state,stateReason,updatedAt,createdAt,assignees,url`, { default: null });
  return raw ? normalizeGhIssue(raw) : null;
}

function findGhIssuesByFooter() {
  const list = jsonRun("gh issue list --state all --json number,title,body,labels,state,updatedAt,createdAt --limit 200", { default: [] });
  return (list || []).filter((raw) => extractFooter(raw.body)).map((raw) => normalizeGhIssue(raw));
}

function normalizeGhIssue(raw) {
  const footer = extractFooter(raw.body || "");
  const labels = (raw.labels || []).map((l) => (typeof l === "string" ? l : l.name)).filter(Boolean);
  return {
    id: `gh-${raw.number}`,
    ghNumber: raw.number,
    title: raw.title || "",
    description: footer ? removeFooter(raw.body) : (raw.body || ""),
    status: mapGhStateToBeads(raw.state, raw.stateReason, labels),
    priority: extractPriorityFromLabels(labels),
    issueType: extractTypeFromLabels(labels) || "task",
    labels: ghLabelsToBeadsLabels(labels),
    assignee: raw.assignees?.[0]?.login || null,
    createdAt: raw.createdAt || null,
    updatedAt: raw.updatedAt || null,
    source: "github",
    _footer: footer,
    _url: raw.url || null,
  };
}

// Status mapping: GitHub → beads
function mapGhStateToBeads(state, stateReason, labels) {
  if (state === "CLOSED") {
    if (stateReason === "COMPLETED" || labels.includes("status:closed")) return "closed";
    if (stateReason === "NOT_PLANNED") return "closed";
    return "closed";
  }
  if (labels.includes("status:in-progress")) return "in_progress";
  if (labels.includes("status:blocked")) return "blocked";
  if (labels.includes("status:deferred")) return "deferred";
  return "open";
}

// Status mapping: beads → GitHub state + labels
function mapBeadsStatusToGh(beadsStatus) {
  switch (beadsStatus) {
    case "closed": return { state: "closed", labels: ["status:closed"] };
    case "in_progress": return { state: "open", labels: ["status:in-progress"] };
    case "blocked": return { state: "open", labels: ["status:blocked"] };
    case "deferred": return { state: "open", labels: ["status:deferred"] };
    default: return { state: "open", labels: [] };
  }
}

function buildGhLabels(beadIssue) {
  const labels = [BEADS_LABEL];
  if (beadIssue.priority !== undefined && beadIssue.priority !== null) {
    labels.push(`priority:p${beadIssue.priority}`);
  }
  if (beadIssue.issueType) {
    labels.push(`type:${beadIssue.issueType}`);
  }
  const statusInfo = mapBeadsStatusToGh(beadIssue.status);
  labels.push(...statusInfo.labels);
  for (const l of beadIssue.labels || []) {
    if (!labels.includes(l)) labels.push(l);
  }
  return labels;
}

function ghLabelsToBeadsLabels(ghLabels) {
  return ghLabels.filter((l) =>
    l !== BEADS_LABEL &&
    !l.startsWith("priority:") &&
    !l.startsWith("status:") &&
    !l.startsWith("type:")
  );
}

function extractPriorityFromLabels(labels) {
  for (const l of labels) {
    const m = l.match(/^priority:p(\d)$/);
    if (m) return parseInt(m[1], 10);
  }
  return null;
}

function extractTypeFromLabels(labels) {
  for (const l of labels) {
    const m = l.match(/^type:(.+)$/);
    if (m) return m[1];
  }
  return null;
}

// ====================== Footer Operations ======================

function extractFooter(body) {
  const m = (body || "").match(FOOTER_PATTERN);
  if (!m) return null;
  return { beadId: m[1], syncedAt: m[2] };
}

function buildFooter(beadId) {
  const ts = new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
  return `\n\n---\n\n<!-- beads-sync: ${beadId} | synced: ${ts} -->`;
}

function removeFooter(body) {
  return (body || "").replace(/\n*---\n*\n*<!-- beads-sync:.*?-->/s, "").trim();
}

// ====================== F. Sync Engine ======================

function resolveConflict(beadIssue, ghIssue, opts) {
  if (opts.preferLocal) return "push";
  if (opts.preferGithub) return "pull";

  const beadTime = Date.parse(beadIssue.updatedAt || 0);
  const ghTime = Date.parse(ghIssue.updatedAt || 0);

  if (beadTime > ghTime) return "push";
  if (ghTime > beadTime) return "pull";
  return "push"; // tie → local wins
}

function pushToGithub(beadIssue, mapping, state, opts, results) {
  const body = removeFooter(beadIssue.description || "") + buildFooter(beadIssue.id);
  const labels = buildGhLabels(beadIssue);
  const labelFlags = labels.map((l) => `--add-label "${shEscape(l)}"`).join(" ");
  const title = shEscape(beadIssue.title);

  if (mapping && mapping.ghNumber) {
    // Update existing
    if (opts.dryRun) {
      info(`  [dry-run] would update gh#${mapping.ghNumber}: ${beadIssue.title}`);
    } else {
      run(`gh issue edit ${mapping.ghNumber} --title "${title}" --body "${body}" ${labelFlags}`, { required: true, label: "gh issue edit" });
      const statusInfo = mapBeadsStatusToGh(beadIssue.status);
      if (statusInfo.state === "closed") {
        run(`gh issue close ${mapping.ghNumber} -r completed`);
      } else {
        try { run(`gh issue reopen ${mapping.ghNumber}`); } catch { /* already open */ }
      }
    }
    updateMapping(state, beadIssue.id, mapping.ghNumber, beadIssue, null);
    results.updated.push({ beadId: beadIssue.id, ghNumber: mapping.ghNumber });
  } else {
    // Create new
    if (opts.dryRun) {
      info(`  [dry-run] would create: ${beadIssue.title}`);
      results.created.push({ beadId: beadIssue.id, ghNumber: "(dry-run)" });
    } else {
      const url = run(`gh issue create --title "${title}" --body "${body}" ${labelFlags}`, { required: true, label: "gh issue create" });
      const m = (url || "").match(/(\d+)$/);
      if (m) {
        const ghNumber = parseInt(m[1], 10);
        updateMapping(state, beadIssue.id, ghNumber, beadIssue, null);
        results.created.push({ beadId: beadIssue.id, ghNumber });
        ok(`Created gh#${ghNumber}: ${beadIssue.title}`);
      }
    }
  }
}

function pullToBeads(ghIssue, mapping, state, opts, results) {
  const beadId = mapping ? Object.keys(state.mappings).find((k) => state.mappings[k].ghNumber === ghIssue.ghNumber) : null;
  if (!beadId) {
    // Create new beads issue
    if (opts.dryRun) {
      info(`  [dry-run] would create beads: ${ghIssue.title}`);
    } else {
      const id = run(`bd create "${shEscape(ghIssue.title)}"
        --description "${shEscape(ghIssue.description || '')}"
        --assignee "${shEscape(ghIssue.assignee || '')}"
        ${ghIssue.labels.map((l) => `--add-label "${shEscape(l)}"`).join(" ")}
        --json`, { required: false, label: "bd create" });
      if (id) {
        try {
          const parsed = JSON.parse(id);
          updateMapping(state, parsed.id, ghIssue.ghNumber, null, ghIssue);
          results.created.push({ beadId: parsed.id, ghNumber: ghIssue.ghNumber });
          ok(`Created beads ${parsed.id}: ${ghIssue.title}`);
        } catch {
          warn(`Failed to parse bd create output: ${id}`);
        }
      }
    }
    return;
  }

  if (opts.dryRun) {
    info(`  [dry-run] would update beads ${beadId}: ${ghIssue.title}`);
    results.updated.push({ beadId, ghNumber: ghIssue.ghNumber });
    return;
  }

  run(`bd update ${beadId} --title "${shEscape(ghIssue.title)}"`, { required: false });
  if (ghIssue.description) {
    run(`bd update ${beadId} --description "${shEscape(ghIssue.description)}"`, { required: false });
  }

  // Sync labels: set to GitHub labels (minus synthetic)
  const currentLabels = ghLabelsToBeadsLabels(ghIssue.labels);
  if (currentLabels.length > 0) {
    // Clear and re-set labels is complex; add new ones, rely on bd label management
    for (const l of currentLabels) {
      run(`bd label ${beadId} "${shEscape(l)}"`, { required: false });
    }
  }

  if (ghIssue.status === "closed") {
    try { run(`bd close ${beadId}`); } catch { /* may already be closed */ }
  }

  updateMapping(state, beadId, ghIssue.ghNumber, null, ghIssue);
  results.updated.push({ beadId, ghNumber: ghIssue.ghNumber });
}

/** 核心同步 orchestrator */
function syncAll(opts) {
  const state = loadState();
  if (!state.repo) state.repo = run("gh repo view --json nameWithOwner -q .nameWithOwner") || "";
  if (!state.createdAt) state.createdAt = new Date().toISOString();

  const results = { created: [], updated: [], closed: [], skipped: [], conflicts: [], errors: [] };
  info(`Starting sync (${opts.full ? "full" : "incremental"}, ${opts.dryRun ? "DRY-RUN" : "LIVE"})`);

  // STEP 1: Labels
  if (!opts.pullOnly) {
    info("Ensuring GitHub labels...");
    const beadsLabels = getBeadsLabels();
    ensureGhLabels(beadsLabels, opts);
  }

  // STEP 2: Gather issues
  const lastSync = opts.full ? null : (state.lastIncrementalSync || state.lastFullSync);
  const since = lastSync || "2020-01-01T00:00:00Z";

  let beadsIssues = [];
  let ghIssues = [];

  if (!opts.pullOnly) {
    if (opts.full) {
      beadsIssues = getAllBeads(opts.includeClosed);
    } else {
      beadsIssues = getChangedBeads(since);
    }
    info(`Beads: ${beadsIssues.length} issues`);
  }

  if (!opts.pushOnly) {
    ghIssues = getGhIssues(opts.includeClosed ? "all" : "open", opts.full ? null : since);
    info(`GitHub: ${ghIssues.length} issues`);
  }

  // STEP 2b: Footer reconciliation (existing sync)
  if (Object.keys(state.mappings).length > 0) {
    const footerIssues = findGhIssuesByFooter();
    for (const fi of footerIssues) {
      if (!ghIssues.some((g) => g.ghNumber === fi.ghNumber)) {
        ghIssues.push(fi);
      }
    }
  }

  // STEP 3: Build lookup maps
  const beadMap = {};
  for (const b of beadsIssues) beadMap[b.id] = b;
  const ghMap = {};
  for (const g of ghIssues) ghMap[g.ghNumber] = g;

  // STEP 4: Process known mappings
  for (const [beadId, mapping] of Object.entries(state.mappings)) {
    const beadIssue = beadMap[beadId];
    const ghIssue = ghMap[mapping.ghNumber];

    if (!beadIssue && !ghIssue) {
      results.skipped.push({ beadId, reason: "both deleted" });
      continue;
    }
    if (!beadIssue && ghIssue) {
      if (!opts.dryRun && !opts.pullOnly) {
        run(`gh issue close ${mapping.ghNumber} -r "not planned" -c "Beads issue deleted"`);
      }
      results.closed.push({ beadId, ghNumber: mapping.ghNumber, reason: "bead deleted" });
      delete state.mappings[beadId];
      continue;
    }
    if (beadIssue && !ghIssue) {
      if (beadIssue.status !== "closed" || opts.includeClosed) {
        pushToGithub(beadIssue, mapping, state, opts, results);
      } else {
        results.skipped.push({ beadId, reason: "gh deleted, bead closed" });
      }
      continue;
    }

    // Both exist — check changes
    let beadsChanged = beadIssue.updatedAt > (mapping.lastBeadsUpdated || "1970-01-01");
    let ghChanged = ghIssue.updatedAt > (mapping.lastGhUpdated || "1970-01-01");

    if (!beadsChanged) {
      beadsChanged = hashFields(beadIssue) !== mapping.beadsHash;
    }
    if (!ghChanged) {
      ghChanged = hashFields(ghIssue) !== mapping.ghHash;
    }

    if (!beadsChanged && !ghChanged) continue;

    if (beadsChanged && !ghChanged) {
      pushToGithub(beadIssue, mapping, state, opts, results);
    } else if (!beadsChanged && ghChanged) {
      pullToBeads(ghIssue, mapping, state, opts, results);
    } else {
      const dir = resolveConflict(beadIssue, ghIssue, opts);
      if (dir === "push") {
        pushToGithub(beadIssue, mapping, state, opts, results);
      } else {
        pullToBeads(ghIssue, mapping, state, opts, results);
      }
    }
  }

  // STEP 5: New beads → GitHub
  if (!opts.pullOnly) {
    for (const beadIssue of beadsIssues) {
      if (state.mappings[beadIssue.id]) continue;
      pushToGithub(beadIssue, null, state, opts, results);
    }
  }

  // STEP 6: New GitHub with footer → beads
  if (!opts.pushOnly) {
    for (const ghIssue of ghIssues) {
      const footer = extractFooter(ghIssue.description || ghIssue._footer?.beadId ? (ghIssue.description || "") : "");
      if (!footer) continue;
      const known = Object.values(state.mappings).some((m) => m.ghNumber === ghIssue.ghNumber);
      if (known) continue;
      pullToBeads(ghIssue, null, state, opts, results);
    }
  }

  // STEP 7: Save
  saveState(state, opts);
  results._state = state;
  return results;
}

// ====================== G. Report ======================

function generateReport(results, opts) {
  const n = results.created.length + results.updated.length + results.closed.length;
  if (opts.json) {
    console.log(JSON.stringify({
      summary: {
        created: results.created.length,
        updated: results.updated.length,
        closed: results.closed.length,
        skipped: results.skipped.length,
        conflicts: results.conflicts.length,
        errors: results.errors.length,
        total: n,
        dryRun: opts.dryRun,
      },
      details: results,
    }, null, 2));
    return;
  }

  if (n === 0) {
    ok("Nothing to sync — both sides are in sync");
    return;
  }

  ok(`Sync complete: ${results.created.length} created, ${results.updated.length} updated, ${results.closed.length} closed, ${results.skipped.length} skipped`);
  for (const c of results.created) {
    info(`  + ${c.beadId} → gh#${c.ghNumber}`);
  }
  for (const u of results.updated) {
    info(`  ~ ${u.beadId} ↔ gh#${u.ghNumber}`);
  }
  for (const c of results.closed) {
    info(`  x ${c.beadId}: ${c.reason}`);
  }
}

// ====================== H. Main ======================

function main() {
  const opts = parseArgs();

  // Hook mode: --incremental-only is a fire-and-forget background safe mode
  if (opts.incrementalOnly) {
    opts.full = false;
    const state = loadState();
    if (!state.lastIncrementalSync && !state.lastFullSync) {
      // First run in hook mode — do a quick full sync to establish baseline
      opts.full = true;
    }
  }

  // Quiet mode for JSON output
  if (opts.json) QUIET = true;

  try {
    const results = syncAll(opts);
    generateReport(results, opts);
    process.exit(0);
  } catch (e) {
    if (opts.json) {
      console.error(JSON.stringify({ error: e.message }));
    } else {
      die(e.message || "Internal error", 2);
    }
    process.exit(2);
  }
}

// Only run main if executed directly (not when imported by test)
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

// Exports for testing
export {
  ROOT, STATE_PATH, FOOTER_PATTERN, BEADS_LABEL,
  hashFields, extractFooter, buildFooter, removeFooter,
  normalizeBeadsIssue, normalizeGhIssue,
  mapGhStateToBeads, mapBeadsStatusToGh,
  buildGhLabels, ghLabelsToBeadsLabels,
  extractPriorityFromLabels, extractTypeFromLabels,
  deterministicColor, shEscape,
  resolveConflict, loadState, saveState, updateMapping,
  syncAll, getAllBeads, getChangedBeads, getGhIssues,
  buildFooter as _buildFooter,
};
