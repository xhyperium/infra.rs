#!/usr/bin/env node
/**
 * worktree.test.mjs — L1 自检测试 for scripts/worktree/worktree.mjs
 */
import { existsSync, readFileSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { execFileSync } from "child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));

let pass = 0, fail = 0;
function ok(c, name) {
  if (c) { pass++; console.log('  ok  ' + name); }
  else   { fail++; console.log('  FAIL ' + name); }
}

const srcPath = join(__dirname, "worktree.mjs");

// ── L0: file existence ──
ok(existsSync(srcPath), "worktree.mjs exists");

// ── syntax check ──
try {
  execFileSync("node", ["--check", srcPath], { timeout: 10000, stdio: "pipe" });
  ok(true, "node --check pass");
} catch (e) {
  ok(false, "node --check: " + String(e.stderr || e.message).trim());
}

// ── source checks ──
const src = readFileSync(srcPath, "utf8");
ok(src.startsWith("#!/usr/bin/env node"), "has shebang");

// imports
ok(src.includes("child_process"), "imports child_process");
ok(src.includes("from \"fs\""), "imports fs");
ok(src.includes("from \"path\""), "imports path");
ok(src.includes("from \"url\""), "imports url");
ok(src.includes("from \"process\""), "imports process");

// subcommands
const subcommands = ["create", "go", "list", "remove", "prune", "current"];
for (const cmd of subcommands) {
  ok(src.includes(`case "${cmd}"`), `has subcommand: ${cmd}`);
}

// help/usage
ok(src.includes("usage:"), "has usage text");
ok(src.includes("{create|go|list|remove|prune|current}"), "usage lists all subcommands");

// error handling
ok(src.includes("die("), "has die function");
ok(src.includes("process.exit(1)"), "has error exit");

// path resolution
ok(src.includes("__dirname"), "uses __dirname");
ok(src.includes(".worktrees"), "uses .worktrees path");
ok(src.includes("WT_BASE"), "defines WT_BASE");

// ── CLI behavior ──
function run(args = []) {
  try {
    const out = execFileSync("node", [srcPath, ...args], {
      timeout: 15000,
      stdio: "pipe",
      encoding: "utf8",
    });
    return { exit: 0, out: out };
  } catch (e) {
    return { exit: e.status || 1, out: String(e.stderr || e.stdout || e.message) };
  }
}

// no args → usage + exit 1
const r0 = run([]);
ok(r0.exit !== 0, "no args exits non-zero");
ok(r0.out.includes("usage"), "no args shows usage");

// create without arg → die
const r1 = run(["create"]);
ok(r1.exit !== 0, "create without branch fails");
ok(r1.out.includes("usage") || r1.out.includes("create"), "create no-arg shows usage");

// remove without arg → die
const r2 = run(["remove"]);
ok(r2.exit !== 0, "remove without branch fails");

// go without arg → die
const r3 = run(["go"]);
ok(r3.exit !== 0, "go without branch fails");

// list succeeds
const r4 = run(["list"]);
ok(r4.exit === 0, "list exits zero");
ok(r4.out.includes("Worktree"), "list shows header");

// prune succeeds
const r5 = run(["prune"]);
ok(r5.exit === 0, "prune exits zero");

// current succeeds
const r6 = run(["current"]);
ok(r6.exit === 0, "current exits zero");

// invalid subcommand → usage
const r7 = run(["invalid-cmd-xyz"]);
ok(r7.exit !== 0, "invalid subcommand exits non-zero");
ok(r7.out.includes("usage"), "invalid subcommand shows usage");

// ── Result ──
console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail > 0 ? 1 : 0);
