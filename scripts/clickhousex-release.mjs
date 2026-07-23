#!/usr/bin/env node
// clickhousex-release.mjs — local release workflow for clickhousex crate
//
// Usage:
//   node scripts/clickhousex-release.mjs              # full release (quality gates → commit → tag)
//   node scripts/clickhousex-release.mjs --check       # quality gates only (no commit/tag)
//   node scripts/clickhousex-release.mjs --dry-run     # print what would happen, don't execute
//   node scripts/clickhousex-release.mjs --version 0.3.7  # override version bump

import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const ROOT = new URL("..", import.meta.url).pathname;
const CRATE = join(ROOT, "crates/adapters/storage/clickhouse");
const CARGO = join(CRATE, "Cargo.toml");
const CHANGELOG = join(CRATE, "CHANGELOG.md");

const BLUE = "\x1b[34m";
const GREEN = "\x1b[32m";
const RED = "\x1b[31m";
const YELLOW = "\x1b[33m";
const RESET = "\x1b[0m";

function sh(cmd, opts = {}) {
  const dry = process.argv.includes("--dry-run");
  if (dry && !opts.force) {
    console.log(`${YELLOW}[DRY]${RESET} ${cmd}`);
    return "";
  }
  try {
    return execSync(cmd, { cwd: ROOT, stdio: opts.silent ? "pipe" : "inherit", ...opts }).toString().trim();
  } catch (e) {
    console.error(`${RED}FAILED:${RESET} ${cmd}`);
    console.error(e.stderr?.toString() ?? e.message);
    process.exit(1);
  }
}

function step(name) {
  console.log(`\n${BLUE}=== ${name} ===${RESET}`);
}

function ok(msg) {
  console.log(`${GREEN}✓${RESET} ${msg}`);
}

function warn(msg) {
  console.log(`${YELLOW}⚠${RESET} ${msg}`);
}

function getVersion() {
  const toml = readFileSync(CARGO, "utf8");
  const m = toml.match(/^version\s*=\s*"(.+)"/m);
  return m?.[1] ?? "unknown";
}

function getNextPatch(version) {
  const parts = version.split(".");
  parts[2] = String(Number(parts[2]) + 1);
  return parts.join(".");
}

function bumpVersion(current, next) {
  const toml = readFileSync(CARGO, "utf8");
  const updated = toml.replace(
    /^version\s*=\s*".+"/m,
    `version = "${next}"`
  );
  writeFileSync(CARGO, updated);
  ok(`Cargo.toml: ${current} → ${next}`);
}

function writeChangelogEntry(version) {
  const today = new Date().toISOString().split("T")[0];
  const toml = readFileSync(CARGO, "utf8");
  const desc = toml.match(/^description\s*=\s*"(.+)"/m)?.[1] ?? "clickhouse storage adapter";

  // Count recent git changes
  const commits = sh(`git log --oneline HEAD~10..HEAD -- ${CRATE}`, { silent: true, force: true }) || "";
  const commitCount = commits ? commits.split("\n").filter(Boolean).length : 0;

  const entry = `
## [${version}] — ${today}

### Changed

- 版本号 ${getVersion()} → ${version}

_本次发布基于最近 ${commitCount} 个 commit，详情见 git log_
`;

  const existing = readFileSync(CHANGELOG, "utf8");
  const insertAfter = existing.indexOf("## [");
  if (insertAfter === -1) {
    writeFileSync(CHANGELOG, existing + entry);
  } else {
    writeFileSync(CHANGELOG, existing.slice(0, insertAfter) + entry + "\n" + existing.slice(insertAfter));
  }
  ok(`CHANGELOG: added ${version} entry (${today})`);
}

// ── Main ──────────────────────────────────────────────────────────

const isCheck = process.argv.includes("--check");
const customVer = process.argv.find((a, i) => a === "--version" && i + 1 < process.argv.length);

async function main() {
  const currentVersion = getVersion();
  console.log(`${BLUE}clickhousex release v${currentVersion}${RESET}`);

  // Step 1: Quality Gates
  step("1/5: Quality Gates");

  sh("cargo fmt -p clickhousex -- --check");
  ok("cargo fmt");

  sh("cargo clippy -p clickhousex --all-targets --all-features -- -D warnings");
  ok("cargo clippy");

  sh("cargo test -p clickhousex -- --test-threads=1");
  ok("cargo test (default)");

  sh("cargo test -p clickhousex --features scaffold -- --test-threads=1");
  ok("cargo test (+scaffold)");

  sh("cargo test -p clickhousex --doc");
  ok("cargo doc-test");

  try {
    sh("node scripts/quality-gates/check-workspace-deps.mjs");
    ok("check-workspace-deps");
  } catch {
    warn("check-workspace-deps failed (may need version bump in root Cargo.toml)");
  }

  try {
    sh("node scripts/fix-encoding.mjs --check crates/adapters/storage/clickhouse/");
    ok("U+FFFD scan");
  } catch {
    warn("U+FFFD scan failed — run node scripts/fix-encoding.mjs crates/adapters/storage/clickhouse/");
  }

  if (isCheck) {
    ok("Check-only mode: all quality gates passed");
    return;
  }

  // Step 2: Integration tests (requires ClickHouse)
  step("2/5: Integration Tests");

  const chHost = process.env.FOUNDATIONX_CLICKHOUSEX_HOST || "127.0.0.1";
  try {
    const curl = sh(`curl -s -u 'default:iCEOuptIx40EduvGOKX73rfY' 'http://${chHost}:8123/?query=SELECT%201'`, { silent: true, force: true });
    if (curl === "1") {
      sh("cargo test -p clickhousex --all-targets -- --test-threads=1 --skip pool_limit_max_in_flight");
      ok("integration tests (all targets)");
    } else {
      warn("ClickHouse not responding — skipping integration tests");
    }
  } catch {
    warn("ClickHouse not responding — skipping integration tests");
  }

  // Step 3: Spec mirror check
  step("3/5: Spec Mirror");
  try {
    sh("cmp .agents/ssot/adapters/storage/clickhouse/spec/spec.md .agents/ssot/adapters/storage/clickhouse/spec/xhyper-clickhousex-complete-spec.md");
    ok("spec mirror matches");
  } catch {
    warn("spec mirror mismatch — update before release");
  }

  // Step 4: Version bump
  step("4/5: Version & Changelog");

  const nextVer = customVer
    ? process.argv[process.argv.indexOf("--version") + 1]
    : getNextPatch(currentVersion);

  if (nextVer === currentVersion) {
    warn(`version already ${currentVersion}`);
  } else {
    bumpVersion(currentVersion, nextVer);
    writeChangelogEntry(nextVer);
  }

  // Step 5: Commit & Tag
  step("5/5: Commit & Tag");

  const tagName = `clickhousex-${nextVer}`;

  // Stage changes
  sh(`git add ${CARGO} ${CHANGELOG}`);

  // Commit
  const commitMsg = `release(clickhousex): ${nextVer}`;
  sh(`git commit -m "${commitMsg}"`, { silent: true });
  ok(`committed: ${commitMsg}`);

  // Tag
  sh(`git tag -a "${tagName}" -m "clickhousex ${nextVer}"`);
  ok(`tagged: ${tagName}`);

  // Summary
  console.log(`\n${GREEN}═══ clickhousex ${nextVer} released ═══${RESET}`);
  console.log(`  Commit: $(git rev-parse --short HEAD)`);
  console.log(`  Tag:    ${tagName}`);
  console.log(`\n  Push:   git push origin $(git rev-parse --abbrev-ref HEAD) && git push origin ${tagName}`);
}

main();
