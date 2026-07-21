#!/usr/bin/env node
/**
 * secrets-migrate-all.mjs — 多源凭据迁移到 GitHub Secrets
 *
 * 统一迁移脚本，支持三种凭据来源：
 *
 * 来源 1 — secrets/env/{dev,prod}.md（结构化 Markdown 表）
 *   用法: node scripts/sre/secrets-migrate-all.mjs --source tables --env dev
 *
 * 来源 2 — secrets/.env（key=value 格式，GitHub PAT tokens）
 *   用法: node scripts/sre/secrets-migrate-all.mjs --source dotenv
 *
 * 来源 3 — deploy/stack/.env.example（Docker Compose 环境变量模板）
 *   用法: node scripts/sre/secrets-migrate-all.mjs --source stack
 *
 * 来源 4 — 全部来源
 *   用法: node scripts/sre/secrets-migrate-all.mjs --source all --env dev
 *
 * 通用标志:
 *   --dry-run        预览模式（默认）；凭据掩码显示，不上传
 *   --live           实际上传至 GitHub Actions Secrets
 *   --repo owner/name   目标仓库（默认从 gh repo view 自动检测）
 *
 * SSOT: 各来源凭据 → FOUNDATIONX_* / GITHUB_* / DOCKER_* secret 命名空间
 */

import { execSync } from "child_process";
import { readFileSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..", "..");

// Source directories (can be overridden via CLI)
const PATHS = {
  tablesDir:  "/home/workspace/ZoneCNH/sre/secrets/env",
  dotenvFile: "/home/workspace/ZoneCNH/sre/secrets/.env",
  stackFile:  "/home/workspace/ZoneCNH/sre/deploy/stack/.env.example",
};

// ====================== CLI ======================

function parseArgs() {
  const a = process.argv.slice(2);
  const getVal = (flag) => a.includes(flag) ? (a[a.indexOf(flag) + 1] || "dev") : null;
  return {
    source: getVal("--source") || "all",
    env:    getVal("--env") || "dev",
    live:   a.includes("--live"),
    dryRun: !a.includes("--live"),
    repo:   getVal("--repo") || "",
  };
}

function die(msg, code = 1) { console.error(`❌ ${msg}`); process.exit(code); }
function info(msg) { console.log(`ℹ ${msg}`); }
function ok(msg)  { console.log(`✅ ${msg}`); }
function warn(msg){ console.warn(`⚠ ${msg}`); }

function run(cmd, opts = {}) {
  try {
    return execSync(cmd, { encoding: "utf-8", timeout: 15000, stdio: ["pipe", "pipe", "pipe"], ...opts }).trim();
  } catch (e) {
    if (opts.required) die(`${opts.label || "cmd"} failed: ${cmd}\n${e.stderr || e}`);
    return "";
  }
}

function mask(v) {
  if (!v) return "(empty)";
  if (v.length <= 6) return "*".repeat(v.length);
  return v.slice(0, 2) + "*".repeat(v.length - 4) + v.slice(-2);
}

// ====================== Core: Secret Upload ======================

function setSecret(name, value, opts) {
  if (opts.dryRun) {
    info(`  ${name} = ${mask(value)}`);
    return { ok: true, name };
  }
  try {
    const repoFlag = opts.repo ? ` --repo "${opts.repo}"` : "";
    run(`gh secret set "${name}" --body "${value.replace(/"/g, '\\"')}"${repoFlag}`, { required: true, label: name });
    ok(`  ${name} set`);
    return { ok: true, name };
  } catch {
    warn(`  ${name} FAILED`);
    return { ok: false, name };
  }
}

// ====================== Source 1: Markdown Tables ======================

function parseTablesS1(env) {
  const path = join(PATHS.tablesDir, `${env}.md`);
  if (!existsSync(path)) die(`Not found: ${path}`);
  const text = readFileSync(path, "utf-8");
  return parseMdTables(text);
}

function parseMdTables(text) {
  const entries = [];

  // PostgreSQL per-db
  for (const m of text.matchAll(/\|\s*(market_\w+|macro_\w+)\s*\|\s*[\d.]+\s*\|\s*5432\s*\|\s*(\w+)\s*\|\s*`([^`]+)`/g)) {
    entries.push({ prefix: "POSTGRESX", db: m[1], name: "PASSWORD", value: m[3] });
    entries.push({ prefix: "POSTGRESX", db: m[1], name: "USER", value: m[2] });
    entries.push({ prefix: "POSTGRESX", db: m[1], name: "DATABASE", value: m[1] });
  }
  // PostgreSQL admin
  const pgAdmin = text.match(/\|\s*PostgreSQL\s*\|\s*[\d.]+\s*\|\s*5432\s*\|\s*postgres\s*\|\s*`([^`]+)`/);
  if (pgAdmin) entries.push({ prefix: "POSTGRESX_ADMIN", name: "PASSWORD", value: pgAdmin[1] });

  // TDengine per-db
  for (const m of text.matchAll(/\|\s*(market_\w+|macro_\w+|macro_coinglass)\s*\|\s*[\d.]+\s*\|\s*6030\s*\/\s*6041\s*\|\s*(\w+)\s*\|\s*`([^`]+)`/g)) {
    entries.push({ prefix: "TAOSX", db: m[1], name: "PASSWORD", value: m[3] });
    entries.push({ prefix: "TAOSX", db: m[1], name: "USER", value: m[2] });
  }
  // TDengine root
  const tdRoot = text.match(/\|\s*TDengine\s*\|\s*[\d.]+\s*\|\s*6030\s*\/\s*6041\s*\|\s*root\s*\|\s*`([^`]+)`/);
  if (tdRoot) entries.push({ prefix: "TAOSX_ADMIN", name: "PASSWORD", value: tdRoot[1] });

  // Redis
  const redis = text.match(/\|\s*Redis\s*\|\s*[^\|]*\|\s*6379\s*\|\s*default\s*\|\s*`([^`]+)`/);
  if (redis) entries.push({ prefix: "REDISX", name: "PASSWORD", value: redis[1] });

  // Kafka
  const kafka = text.match(/\|\s*Kafka\s*\|\s*[^\|]*\|\s*9092\s*\|\s*admin\s*\|\s*`([^`]+)`/);
  if (kafka) entries.push({ prefix: "KAFKAX", name: "SASL_PASSWORD", value: kafka[1] });

  // ClickHouse
  const ch = text.match(/(?:user\s*`default`\s*\/\s*password\s*`([^`]+)`|\|\s*ClickHouse\s*\|\s*[^\|]*\|\s*[^\|]*\|\s*default\s*\|\s*`([^`]+)`)/);
  if (ch) entries.push({ prefix: "CLICKHOUSEX", name: "PASSWORD", value: ch[1] || ch[2] });

  // NATS
  const nats = text.match(/\*\*认证\*\*:.*?密码\s*`([^`]+)`/);
  if (nats) entries.push({ prefix: "NATS", name: "TOKEN", value: nats[1] });

  // FRED API key
  const fred = text.match(/api_key=(\S+)/);
  if (fred) entries.push({ prefix: "FRED", name: "API_KEY", value: fred[1] });

  return entries;
}

function entriesToSecrets(entries) {
  return entries
    .filter((e) => e.value && e.value.trim())
    .map((e) => ({
      name: `FOUNDATIONX_${e.prefix}${e.db ? "_" + e.db.toUpperCase() : ""}_${e.name}`,
      value: e.value,
    }));
}

// ====================== Source 2: .env (Dotenv) ======================

function parseDotenvS2() {
  const path = PATHS.dotenvFile;
  if (!existsSync(path)) die(`Not found: ${path}`);
  const text = readFileSync(path, "utf-8");
  return parseDotenv(text);
}

function parseDotenv(text) {
  const entries = [];
  for (const line of text.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const eq = trimmed.indexOf("=");
    if (eq < 0) continue;
    const key = trimmed.slice(0, eq).trim();
    const value = trimmed.slice(eq + 1).trim().replace(/^["']|["']$/g, "");
    if (value) entries.push({ prefix: "GITHUB", name: key.toUpperCase(), value });
  }
  return entries;
}

// ====================== Source 3: Docker Compose Stack ======================

function parseStackS3() {
  const path = PATHS.stackFile;
  if (!existsSync(path)) die(`Not found: ${path}`);
  const text = readFileSync(path, "utf-8");
  return parseStackEnv(text);
}

function parseStackEnv(text) {
  const entries = [];
  for (const line of text.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const m = trimmed.match(/^([A-Z_][A-Z0-9_]*)=(.*)$/);
    if (!m) continue;
    const value = m[2].trim().replace(/^["']|["']$/g, "");
    // Only extract password/secret/token fields
    if (/(PASSWORD|SECRET|TOKEN|KEY|PASS|ADMIN_PW)/i.test(m[1])) {
      entries.push({ prefix: "STACK", name: m[1], value });
    }
  }
  return entries;
}

// ====================== Main Orchestrator ======================

function migrateSource(label, entries, opts) {
  console.log(`\n── ${label} ──`);
  const secrets = entriesToSecrets(entries);
  let ok_ = 0, fail = 0;
  for (const s of secrets) {
    const r = setSecret(s.name, s.value, opts);
    r.ok ? ok_++ : fail++;
  }
  return { ok: ok_, fail, total: ok_ + fail, entries };
}

function main() {
  const opts = parseArgs();
  if (!opts.live) info("Default: --dry-run. Use --live to upload.");

  const totals = { ok: 0, fail: 0 };

  // Source 1: Markdown tables (dev/prod)
  if (opts.source === "tables" || opts.source === "all") {
    const envs = opts.env === "both" ? ["dev", "prod"] : [opts.env];
    for (const env of envs) {
      const entries = parseTablesS1(env);
      const r = migrateSource(`MD Tables (${env})`, entries, opts);
      totals.ok += r.ok; totals.fail += r.fail;
    }
  }

  // Source 2: .env (GitHub tokens)
  if (opts.source === "dotenv" || opts.source === "all") {
    const entries = parseDotenvS2();
    const r = migrateSource(".env (GitHub Tokens)", entries, opts);
    totals.ok += r.ok; totals.fail += r.fail;
  }

  // Source 3: Docker Compose stack
  if (opts.source === "stack" || opts.source === "all") {
    const entries = parseStackS3();
    const r = migrateSource("Docker Compose Stack", entries, opts);
    totals.ok += r.ok; totals.fail += r.fail;
  }

  console.log(`\n=== Migration Summary ===`);
  console.log(`OK: ${totals.ok} | Failed: ${totals.fail} | Total: ${totals.ok + totals.fail}`);
  if (totals.fail > 0) die(`${totals.fail} secrets failed to migrate`, 2);
}

main();

export { parseMdTables, parseDotenv, parseStackEnv, entriesToSecrets, mask, PATHS, setSecret };
