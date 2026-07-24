#!/usr/bin/env node
/** 七包双栏自验证：STATUS 100% + test + cov-gate-100 */
import { execSync } from "child_process";
import { readFileSync } from "fs";

const pkgs = [
  "configx",
  "evidence",
  "observex",
  "resiliencx",
  "schedulex",
  "transportx",
  "contracts",
];

function run(cmd) {
  console.log("+", cmd);
  execSync(cmd, { stdio: "inherit" });
}

run("node scripts/docs/gen-crate-status.mjs --check");
const status = readFileSync("STATUS.md", "utf8");
for (const p of pkgs) {
  const re = new RegExp("`" + p + "`[\\s\\S]*?\\*\\*(\\d+)%\\*\\*");
  const m = status.match(re);
  if (!m || m[1] !== "100") {
    console.error("FAIL STATUS", p, m && m[1]);
    process.exit(1);
  }
  console.log("STATUS", p, "100%");
}
const list = pkgs.join(" -p ");
run(`cargo test -p ${list} --all-targets`);
for (const p of pkgs) {
  const path = p === "transportx" ? "crates/infra/transport" : `crates/${p}`;
  run(`node scripts/quality-gates/cov-gate-100.mjs -p ${p} --filter ${path}/src`);
}
console.log("verify-seven-dualbar: PASS");
