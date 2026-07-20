#!/usr/bin/env node
import { execSync } from "child_process";
import { existsSync, readFileSync } from "fs";
import { join } from "path";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const root = execSync("git rev-parse --show-toplevel", { encoding: "utf8" }).trim();
const branch = execSync("git rev-parse --abbrev-ref HEAD", { encoding: "utf8" }).trim();
const scratch = process.env.SCRATCH || "/tmp/grok-check-canonical";
const ssot = ".agents/ssot/types/canonical";
const crate = "crates/types/canonical";

process.chdir(root);

// helpers
function fail(msg) { console.error("FAIL: " + msg); process.exit(1); }
function ok(msg) { console.log("OK: " + msg); }
function run(cmd) { execSync(cmd, { encoding: "utf8", stdio: "pipe" }); }
function quiet(cmd) { try { return execSync(cmd, { encoding: "utf8", stdio: "pipe" }).trim(); } catch { return ""; } }
function has(cmd) { try { execSync("command -v " + cmd, { encoding: "utf8", stdio: "pipe" }); return true; } catch { return false; } }

console.log("=== check-canonical-align branch=" + branch + " ===");

execSync("mkdir -p " + scratch);

// verify packages exist in workspace
let names;
try {
    const meta = JSON.parse(execSync("cargo metadata --no-deps --format-version 1", { encoding: "utf8" }));
    names = meta.packages.map(p => p.name);
} catch { fail("cargo metadata failed"); }
if (!names.includes("xhyper-canonical") || !names.includes("xhyper-decimalx"))
    fail("missing packages: " + JSON.stringify(names));
console.log("Packages: " + JSON.stringify(names));

// file existence checks
if (!existsSync(join(crate, "src/lib.rs"))) fail("no crate");
if (!existsSync("crates/types/decimal/src/lib.rs")) fail("no decimal");
if (!existsSync("fixtures/market/order_cancel_okx.json")) fail("no fixture");

// spec checks
const specMd = readFileSync(join(ssot, "spec/spec.md"), "utf8");
if (!specMd.includes("**Approved**")) fail("spec not Approved");
if (!specMd.match(/类型已删|类型已删除/)) fail("OrderId not deleted in spec");
if (!specMd.match(/纳秒|Unix \*\*ns\*\*/)) fail("ts ns missing");
if (specMd.includes("deprecated `OrderId`")) fail("spec deprecated OrderId");
if (!existsSync(join(ssot, "plan/alignment-matrix-infra-2026-07-21.md")) || !readFileSync(join(ssot, "plan/alignment-matrix-infra-2026-07-21.md"), "utf8").trim())
    fail("no matrix");

const planMd = quiet("cat " + join(ssot, "plan/plan.md"));
if (planMd.includes("deprecated `OrderId`")) fail("plan deprecated OrderId");
if (planMd.match(/OPEN-TIME \|.*\| OPEN \|/)) fail("OPEN-TIME OPEN");
const goalMd = quiet("cat " + join(ssot, "goal/goal.md"));
if (goalMd.includes("not started")) fail("goal placeholder");

// SAFE-15 / SAFE-16
const todo = quiet("cat " + join(ssot, "todo.md"));
const s15 = todo.split("\n").find(l => l.includes("SAFE-15")) || "";
if (!s15.includes("DEFERRED")) fail("SAFE-15 not DEFERRED: " + s15);
const s16 = todo.split("\n").find(l => l.includes("SAFE-16")) || "";
if (!s16.match(/HUMAN_ONLY|DEFERRED/)) fail("SAFE-16 not HUMAN: " + s16);

// T-10X
const tasks = quiet("cat " + join(ssot, "plan/tasks.md"));
const t10 = tasks.split("\n").find(l => l.includes("T-10X-001")) || "";
if (!t10.includes("DEFERRED")) fail("T-10X not DEFERRED: " + t10);

// crate source checks
const rg = has("rg") ? "rg" : "grep -rn";
if (quiet(`${rg} 'type OrderId' ${crate}/src`)) fail("type OrderId in crate");
if (quiet(`${rg} '\\\\bf32\\\\b|\\\\bf64\\\\b' ${crate}/src --glob '*.rs'`)) fail("f32/f64 in crate");

// spec mirror
const specMD5 = quiet("md5sum " + join(ssot, "spec/spec.md") + " | cut -d' ' -f1");
const mirrorMD5 = quiet("md5sum " + join(ssot, "spec/xhyper-canonical-complete-spec.md") + " | cut -d' ' -f1");
if (specMD5 !== mirrorMD5) fail("dual-mirror mismatch");
ok("authority facts");

// post-S1 checks
if (planMd.includes("当前仅 5 测")) fail("plan DRIFT-04 still claims 5 tests");
const cgoal = quiet("cat " + join(ssot, "20260717/xhyper-canonical-complete-goal.md"));
if (cgoal.includes("未假装批准")) fail("complete-goal §7 still pretends ID/time OPEN");
if (!quiet("cat " + join(ssot, "plan/approval-packet.md")).includes("SUPERSEDED for current-state"))
    fail("approval-packet.md missing SUPERSEDED banner");
if (quiet(`${rg} 'until_can_time_approved|ts unit remains OPEN' ${crate}/src`))
    fail("crate still has OPEN-time wording");

const driftOpen = planMd.split("\n").filter(l => l.match(/\| DRIFT-0[1-6] \|/) && l.match(/agent-safe 补测|agent-safe 修正$/));
if (driftOpen.length > 0) fail("plan DRIFT still has open agent-safe disposition");
ok("post-S1 drift/goal/approval/crate wording");

// cargo test + clippy + fmt
try { run("cargo test -p xhyper-canonical -p xhyper-decimalx 2>&1"); } catch { fail("tests failed"); }
try { run("cargo clippy -p xhyper-canonical -p xhyper-decimalx --all-targets -- -D warnings 2>&1"); } catch { fail("clippy failed"); }
try { run("cargo fmt -p xhyper-canonical -p xhyper-decimalx -- --check"); } catch { fail("fmt failed"); }

console.log("=== ALL CHECKS PASSED ===");
